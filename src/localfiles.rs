use crate::whdload::{Collection, WhdloadItem};
use anyhow::Result;
use glob::glob;
use std::path::PathBuf;

const CATEGORIES: [&str; 5] = [
    "Commodore Amiga - WHDLoad - Demos (*).zip",
    "Commodore Amiga - WHDLoad - Demos - Beta & Unreleased (*).zip",
    "Commodore Amiga - WHDLoad - Games (*).zip",
    "Commodore Amiga - WHDLoad - Games - Beta & Unreleased (*).zip",
    "Commodore Amiga - WHDLoad - Magazines (*).zip",
];

const LHA_FILES: &str = "Commodore Amiga - WHDLoad -*/[0|A-Z]/*.l??";

pub fn find_local_files() -> Collection {
    eprint!("Collecting local files.");
    let files: Collection = glob(LHA_FILES)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter_map(|e| WhdloadItem::try_from(e).ok())
        .collect();
    eprint!("Collecting local files finished.");
    files
}

pub fn remove_old_dats() -> Result<()> {
    for cat in CATEGORIES {
        let mut dats: Vec<PathBuf> = glob(cat).unwrap().filter_map(|f| f.ok()).collect();
        dats.sort_unstable_by(|a, b| {
            a.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&b.metadata().unwrap().modified().unwrap())
        });
        println!("{:?}", dats);
        if dats.pop().is_some() {
            for d in dats {
                println!("del: {}", d.display());
            }
        }
    }

    Ok(())
}
