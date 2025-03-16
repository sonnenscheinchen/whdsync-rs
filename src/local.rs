use crate::whdload::{Collection, WhdloadItem};
use glob::glob;
use std::fs::remove_file;

const CATEGORIES: [&str; 5] = [
    "Commodore Amiga - WHDLoad - Demos (*).zip",
    "Commodore Amiga - WHDLoad - Demos - Beta & Unreleased (*).zip",
    "Commodore Amiga - WHDLoad - Games (*).zip",
    "Commodore Amiga - WHDLoad - Games - Beta & Unreleased (*).zip",
    "Commodore Amiga - WHDLoad - Magazines (*).zip",
];

const LHA_FILES: &str = "Commodore Amiga - WHDLoad -*/[0|A-Z]/*.l??";

pub fn find_local_files() -> Collection {
    eprintln!("Collecting local files.");
    let files: Collection = glob(LHA_FILES)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter_map(|e| WhdloadItem::try_from(e).ok())
        .collect();
    eprintln!("Collecting local files finished.");
    files
}

pub fn remove_old_dats() {
    for cat in CATEGORIES {
        let mut dats: Vec<String> = glob(cat)
            .unwrap()
            .filter_map(|p| p.ok())
            .map(|p| p.to_string_lossy().into())
            .collect();
        dats.sort_unstable_by(|left, right| {
            let split_left = left.split('-').rev();
            let split_right = right.split('-').rev();
            split_left.cmp(split_right)
        });
        if dats.pop().is_some() {
            for d in dats {
                match remove_file(&d) {
                    Ok(()) => println!("[DEL]: {}", d),
                    Err(e) => println!("Failed to delete {}: {}", d, e),
                }
            }
        }
    }
}
