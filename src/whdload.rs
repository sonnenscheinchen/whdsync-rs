use std::{os::unix::fs::MetadataExt, path::PathBuf};
use glob::GlobError;


#[derive(Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct WhdloadItem {
    pub path: String,
    pub size: u64,
}

impl WhdloadItem {
    pub fn new_from_path(p: Result<&PathBuf, &GlobError>) -> Option<WhdloadItem> {
        let size = p.ok()?.metadata().ok()?.size();
        let path = p.ok()?.to_string_lossy().to_string();
        Some(WhdloadItem {
            path,
            size,
        })
    }
}
