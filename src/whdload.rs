use anyhow::{Error, Result};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs::{create_dir_all, File};
use std::io::{copy, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub type Collection = HashSet<WhdloadItem>;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Clone)]
pub struct WhdloadItem {
    path: String,
    size: u64,
}

impl TryFrom<PathBuf> for WhdloadItem {
    type Error = Error;

    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        let size = p.metadata()?.len();
        let path = if cfg!(target_os = "windows") {
            p.to_string_lossy().replace('\\', "/")
        } else {
            p.to_string_lossy().to_string()
        };
        Ok(WhdloadItem { path, size })
    }
}

impl WhdloadItem {
    pub fn new(path: String, size: u64) -> Self {
        Self { path, size }
    }
    pub fn save_file(&self, mut reader: impl Read) -> Result<(), Error> {
        let path = PathBuf::from(&self.get_local_path());
        let dir = path.parent().unwrap();
        if !dir.is_dir() {
            create_dir_all(dir)?;
        };
        let mut outbuf = BufWriter::new(File::create(&path)?);
        let mut inbuf = BufReader::new(&mut reader);
        copy(&mut inbuf, &mut outbuf)?;
        outbuf.flush()?;
        Ok(())
    }
    pub fn get_remote_path(&self) -> String {
        self.path.replace(" ", "_")
    }
    pub fn get_local_path(&self) -> String {
        if cfg!(target_os = "windows") {
            self.path.replace('/', "\\")
        } else {
            self.path.clone()
        }
    }
}
