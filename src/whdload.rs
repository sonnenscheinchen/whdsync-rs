use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::convert::TryFrom;
use anyhow::Error;


#[derive(Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct WhdloadItem {
    pub path: String,
    pub size: u64,
}

impl TryFrom<PathBuf> for WhdloadItem {
    type Error = Error;

    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        let size = p.metadata()?.size();
        let path = p.to_string_lossy().to_string();
        Ok(WhdloadItem {
            path,
            size,
        })
    }
}