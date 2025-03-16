use anyhow::{Error, Result};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs::{create_dir_all, File};
use std::io::{copy, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub type Collection = HashSet<WhdloadItem>;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct WhdloadItem {
    pub path: String,
    pub size: u64,
}

impl TryFrom<PathBuf> for WhdloadItem {
    type Error = Error;

    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        let size = p.metadata()?.len();
        let path = p.to_string_lossy().to_string();
        Ok(WhdloadItem { path, size })
    }
}

impl WhdloadItem {
    pub fn save_file(&self, mut reader: impl Read) -> Result<(), Error> {
        let mut dir = PathBuf::from(&self.path);
        dir.pop();
        if !dir.is_dir() {
            create_dir_all(&dir)?;
        };
        let mut outbuf = BufWriter::new(File::create(&self.path)?);
        let mut inbuf = BufReader::new(&mut reader);
        copy(&mut inbuf, &mut outbuf)?;
        outbuf.flush()?;
        Ok(())
    }
}
