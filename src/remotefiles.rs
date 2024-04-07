use std::io::{BufWriter, Write};
use std::{env::current_dir, os::unix::fs::MetadataExt};
use std::fs::read;
use std::fs::File;
use suppaftp::{FtpStream, list};
use anyhow::{anyhow, Result};
use zune_inflate::{DeflateDecoder, DeflateOptions};
use crc32fast::hash;
use bytes::Buf;
use crate::whdload::WhdloadItem;

const ZIP_HEADER: &[u8; 4] = &[0x50, 0x4b, 0x03, 0x04];

pub fn find_remote_files(stream: &mut FtpStream) -> Result<Vec<WhdloadItem>> {
    let mut remote_files = vec![];
    let mut dat_files = vec![];
    stream.cwd("Retroplay WHDLoad Packs")?;

    for line in stream.list(None)?.iter() {
        if let Ok(f) = list::File::from_posix_line(line) {
            if f.is_file() && f.name().starts_with("Commodore Amiga - WHDLoad") {
                dat_files.push(f);
            }
        }
    }

    let current_dir = current_dir()?;
    for dat in dat_files {
        let local_file = current_dir.with_file_name(dat.name());
        let data = if local_file.is_file() && local_file.metadata()?.size() == dat.size() as u64 {
            read(local_file)?
        } else {
            let retr = stream.retr_as_buffer(dat.name())?.into_inner();
            let mut buffer = BufWriter::new(File::create(dat.name())?);
            buffer.write_all(&retr)?;
            buffer.flush()?;
            retr
        };
        let xml_string = unzip_data(&data)?;
        remote_files.append(&mut parse_xml(xml_string)?);
    }
    remote_files.sort_unstable();
    Ok(remote_files)
}

fn unzip_data(mut data: &[u8]) -> Result<String> {
    if &data[..4] != ZIP_HEADER {
        return Err(anyhow!("Invalid zip file."));
    };
    data.advance(14);
    let provided_checksum = data.get_u32_le();
    let comp_size = data.get_u32_le() as usize;
    let uncomp_size = data.get_u32_le() as usize;
    let filename_len = data.get_u16_le() as usize;
    let extrafield_len = data.get_u16_le() as usize;
    data.advance(filename_len + extrafield_len);
    let range = &data[..comp_size];
    let options = DeflateOptions::default()
        .set_limit(2 * uncomp_size)
        .set_size_hint(uncomp_size);
    let mut decoder = DeflateDecoder::new_with_options(range, options);
    let decoded = decoder
        .decode_deflate()
        .map_err(|_| anyhow!("Failed to deflate data."))?;
    let calculated_checksum = hash(&decoded);
    if provided_checksum != calculated_checksum {
        Err(anyhow!("Checksum error."))
    } else {
        Ok(String::from_utf8(decoded)?)
    }
}

fn parse_xml(xml_string: String) -> Result<Vec<WhdloadItem>> {
    let mut set = vec![];
    let doc = roxmltree::Document::parse(&xml_string)?;
    let mut descendants = doc.descendants();
    let description = descendants
        .find_map(|n| n.has_tag_name("description").then(|| n.text().unwrap()))
        .unwrap();
    for machine_node in descendants.filter(|n| n.has_tag_name("machine")) {
        let letter = machine_node.attribute("name").unwrap();
        for rom_node in machine_node.descendants().filter(|n| n.has_tag_name("rom")) {
            let name = rom_node.attribute("name").unwrap();
            let size: u64 = rom_node.attribute("size").unwrap().parse()?;
            let path = format!("{description}/{letter}/{name}");
            set.push(WhdloadItem {path, size});
        }
    }
    Ok(set)
}