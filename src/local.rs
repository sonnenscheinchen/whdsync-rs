use crate::whdload::{Collection, WhdloadItem};
use glob::glob;
use std::{fs::remove_file, path::PathBuf};

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

pub fn remove_old_dats(remove_old_files: bool) {
    for cat in CATEGORIES {
        let mut dats: Vec<PathBuf> = glob(cat)
            .unwrap()
            .filter_map(|p| p.ok())
            .collect();
        dats.sort_unstable();
        if dats.pop().is_some() {
            for d in dats {
                let s = d.to_string_lossy();
                if remove_old_files {
                    match remove_file(&d) {
                        Ok(()) => println!("[DEL]: {s}"),
                        Err(e) => println!("Failed to delete {s}: {e}"),
                    }
                } else {
                    println!("[KEEP]: {s}");
                }
            }
        }
    }
}
