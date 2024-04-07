use crate::whdload::WhdloadItem;
use glob::glob;
use anyhow::Result;

pub fn find_local_files() -> Result<Vec<WhdloadItem>> {
    let pattern = "Commodore Amiga - WHDLoad -*/[0|A-Z]/*.lha".to_owned();
    let mut files: Vec<WhdloadItem> = glob(&pattern)
        .unwrap()
        .filter_map(|e| WhdloadItem::new_from_path(e.as_ref()))
        .collect();
    files.sort_unstable();
    Ok(files)
}
