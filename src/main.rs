mod credentials;
mod download;
mod local;
mod remote;
mod whdload;
use anyhow::{bail, Result};
use credentials::Credentials;
use download::*;
use local::{find_local_files, remove_old_dats};
use remote::{create_ftp_stream, find_remote_files};
use std::env::{args, set_current_dir};
use std::fs::remove_file;
use std::path::PathBuf;
use std::thread;
use whdload::WhdloadItem;

fn main() -> Result<()> {
    println!("whdsync-rs version 0.3.0");

    let target_dir = match args().nth(1) {
        Some(arg) => PathBuf::from(arg),
        None => bail!("Need a valid target directory."),
    };

    set_current_dir(&target_dir)?;

    let remove_old_files = args()
        .nth(2)
        .is_some_and(|arg| arg == "-d" || arg == "--delete");

    let login = Credentials::from_env()
        .or(Credentials::from_netrc())
        .unwrap_or_default();

    let mut ftp2 = create_ftp_stream(FTP2, &login)?;

    let t = thread::spawn(find_local_files);
    let remotefiles = find_remote_files(&mut ftp2)?;
    let localfiles = t.join().unwrap();

    let mut to_download: Vec<WhdloadItem> = remotefiles.difference(&localfiles).cloned().collect();

    let num_downloads = to_download.len();

    if num_downloads == 0 {
        println!("Collection is up to date.");
        return Ok(());
    } else {
        println!("Downloading {num_downloads} files.");
    }

    to_download.sort_unstable();

    let mut failed_downloads = download(to_download)?;

    failed_downloads.sort_unstable();

    let num_failed = failed_downloads.len();

    let success = if num_failed == 0 {
        true
    } else {
        println!("Trying to redownload {num_failed} files.");
        run_downloader(&failed_downloads, false, "FTP2-1").map_or_else(
            |e| {
                eprintln!("{e}");
                false
            },
            |still_failed| still_failed.is_empty(),
        )
    };

    if success {
        for f in localfiles.difference(&remotefiles) {
            let path = f.get_local_path();
            if remove_old_files {
                println!("[DEL]: {path}");
                let _ = remove_file(&path);
            } else {
                println!("[KEEP]: {path}");
            }
        }
        remove_old_dats(remove_old_files);
        println!("Finished successfully.");
    } else {
        println!("Sync completed with errors.");
    }

    Ok(())
}
