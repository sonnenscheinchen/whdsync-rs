use crate::credentials::Credentials;
use crate::whdload::{Collection, WhdloadItem};
use anyhow::{anyhow, Result};
use bytes::Buf;
use crc32fast::hash;
use std::fs::{read, write};
use std::path::PathBuf;
use std::thread;
use suppaftp::{list, types::FileType, FtpStream};
use zune_inflate::{DeflateDecoder, DeflateOptions};

const ZIP_HEADER: &[u8; 4] = &[0x50, 0x4b, 0x03, 0x04];

pub fn create_ftp_stream(host: &str, login: &Credentials) -> Result<FtpStream> {
    println!("Connecting to {host} ...");
    let mut stream = FtpStream::connect(host)?;
    stream.login(&login.username, &login.password)?;
    stream.transfer_type(FileType::Binary)?;
    stream.cwd("Retroplay WHDLoad Packs")?;
    Ok(stream)
}

pub fn find_remote_files(stream: &mut FtpStream) -> Result<Collection> {
    eprintln!("Collecting remote files.");

    let mut remote_files = Collection::with_capacity(5000);

    let dat_files: Vec<list::File> = stream
        .list(None)?
        .iter()
        .filter_map(|f| list::File::from_posix_line(f).ok())
        .filter(|f| f.is_file() && f.name().starts_with("Commodore Amiga - WHDLoad"))
        .collect();

    let mut threads = Vec::with_capacity(dat_files.len());

    for dat in dat_files {
        let local_file = PathBuf::from(dat.name());
        let data = if local_file.is_file() && local_file.metadata()?.len() == dat.size() as u64 {
            read(local_file)?
        } else {
            let retr = stream.retr_as_buffer(dat.name())?.into_inner();
            write(dat.name(), &retr)?;
            retr
        };

        threads.push(thread::spawn(move || parse_xml(unzip_data(&data)?)));
    }

    for t in threads {
        remote_files.extend(t.join().unwrap()?);
    }

    eprintln!("Collecting remote files finished.");
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
    let mut result: Vec<WhdloadItem> = Vec::with_capacity(4000);

    let opts = roxmltree::ParsingOptions {
        allow_dtd: true,
        nodes_limit: 100_000,
    };
    let doc = roxmltree::Document::parse_with_options(&xml_string, opts)?;
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
            result.push(WhdloadItem { path, size });
        }
    }

    Ok(result)
}
