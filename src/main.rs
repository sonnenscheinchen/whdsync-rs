mod download;
mod local;
mod remote;
mod whdload;
mod credentials;
use std::env::{args, set_current_dir};
use std::fs::remove_file;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{thread, vec};
use download::run_downloader;
use anyhow::{bail, Result};
use credentials::Credentials;
use remote::{create_ftp_stream, find_remote_files};
use local::{find_local_files, remove_old_dats};
use whdload::WhdloadItem;

// pub const FTP1: &str = "localhost:2121";
// pub const FTP2: &str = "localhost:2122";
// pub const FTP3: &str = "localhost:2123";
pub const FTP1: &str = "ftp.grandis.nu:21";
pub const FTP2: &str = "ftp2.grandis.nu:21";
pub const FTP3: &str = "grandis.nu:21";

fn main() -> Result<()> {
    println!("whdsnc2 version 0.2.0");

    let target_dir = match args().nth(1) {
        Some(dir) => PathBuf::from(dir),
        None => bail!("Need a valid target directory."),
    };

    set_current_dir(&target_dir)?;

    let login = Credentials::new_from_netrc().unwrap_or_default();
    let mut ftp2 = create_ftp_stream(FTP2, &login)?;

    let t = thread::spawn(find_local_files);
    let remotefiles = find_remote_files(&mut ftp2)?;
    let localfiles = t.join().unwrap();

    let to_download: Vec<WhdloadItem> = remotefiles.difference(&localfiles).cloned().collect();

    let num_downloads = to_download.len();

    if num_downloads == 0 {
        println!("Collection is up to date.");
        return Ok(());
    } else {
        println!("Downloading {num_downloads} files.");
    }

    let queue = Mutex::new(to_download);
    let mut failed_downloads = vec![];

    let dl_scope = thread::scope(|scope| {
        let mut threads = vec![];
        let primary = scope.spawn(|| run_downloader(&queue, &mut ftp2, true, "FTP2-1"));

        if num_downloads > 2 {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP1, &login)
                    .and_then(|mut ftp1| run_downloader(&queue, &mut ftp1, false, "FTP1-1"))
            }));
        };

        if num_downloads > 4 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP3, &login)
                    .and_then(|mut ftp3| run_downloader(&queue, &mut ftp3, false, "FTP3-1"))
            }));
        };

        if num_downloads > 6 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP2, &login)
                    .and_then(|mut ftp2| run_downloader(&queue, &mut ftp2, false, "FTP2-2"))
            }));
        };

        if num_downloads > 8 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP1, &login)
                    .and_then(|mut ftp1| run_downloader(&queue, &mut ftp1, false, "FTP1-2"))
            }));
        };

        if num_downloads > 10 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP3, &login)
                    .and_then(|mut ftp3| run_downloader(&queue, &mut ftp3, false, "FTP3-2"))
            }));
        };

        if let Err(e) = primary.join().unwrap() {
            eprintln!("Primary downloader finished unexpectly.");
            eprintln!("{e}");
            {
                queue.lock().unwrap().clear();
            };
            eprintln!("Waiting for threads to finish.");
            for t in threads {
                let _ = t.join().unwrap();
            }
            return Err(e);
        }
        for t in threads {
            match t.join().unwrap() {
                Ok(mut failed) => failed_downloads.append(&mut failed),
                Err(e) => eprintln!("{e}"),
            };
        }
        Ok(())
    });

    if dl_scope.is_err() {
        bail!("Fatal error. Can't continue.")
    };

    let num_failed = failed_downloads.len();

    let success = if num_failed == 0 {
        true
    } else {
        println!("Trying to redownload {num_failed} files.");
        let queue = Mutex::new(failed_downloads);
        run_downloader(&queue, &mut ftp2, false, "FTP2-1").map_or_else(
            |e| {
                eprintln!("{e}");
                false
            },
            |still_failed| still_failed.is_empty(),
        )
    };

    if success {
        for f in localfiles.difference(&remotefiles) {
            println!("[DEL]: {}", f.path);
            let _ = remove_file(&f.path);
        }
        remove_old_dats();
        println!("Finished successfully.");
    } else {
        println!("Sync completed with errors.");
    }

    Ok(())
}
