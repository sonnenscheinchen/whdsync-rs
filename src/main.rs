use crate::download::run_downloader;
use crate::whdload::WhdloadItem;
use anyhow::{bail, Result};
use std::env::{args, set_current_dir};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
mod credentials;
use credentials::Credentials;
mod download;
mod localfiles;
mod remotefiles;
use remotefiles::create_ftp_stream;
mod whdload;

// const FTP1_HOST: &str = "localhost:2121";
// const FTP2_HOST: &str = "localhost:2121";
const FTP1_HOST: &str = "ftp.grandis.nu:21";
const FTP2_HOST: &str = "ftp2.grandis.nu:21";
const FTP3_HOST: &str = "grandis.nu:21";

fn main() -> Result<()> {
    println!("Hello, world!");

    let target_dir = match args().nth(1) {
        Some(dir) => PathBuf::from(dir),
        None => bail!("Need a valid target directory."),
    };

    set_current_dir(&target_dir)?;

    let login = Credentials::new_from_netrc().unwrap_or_default();
    let mut ftp2 = create_ftp_stream(FTP2_HOST, &login)?;

    let t = thread::spawn(localfiles::find_local_files);
    let remotefiles = remotefiles::find_remote_files(&mut ftp2)?;
    let localfiles = t.join().unwrap();

    let mut to_download = vec![];

    for rf in &remotefiles {
        if localfiles.binary_search(rf).is_err() {
            println!("dl: {:?}", rf);
            to_download.push(rf.clone());
        }
    }

    let num_downloads = to_download.len();

    if num_downloads == 0 {
        println!("Collection is up to date.");
        return Ok(());
    } else {
        println!("Downloading {num_downloads} files.");
    }

    let mutex = Mutex::new(to_download);
    let failed_downloads = Mutex::new(Vec::<WhdloadItem>::new());

    thread::scope(|scope| {
        scope.spawn(|| {
            let _ = run_downloader(&mutex, &mut ftp2, None).map_err(|e| {
                eprintln!("Primary downloader finished unexpectly.");
                panic!("{e}");
            });
        });

        if num_downloads > 1 {
            scope.spawn(|| {
                if let Ok(mut ftp1) = create_ftp_stream(FTP1_HOST, &login) {
                    let _ = run_downloader(&mutex, &mut ftp1, Some(&failed_downloads))
                        .map_err(|e| eprintln!("{e}"));
                };
            });
        };

        if num_downloads > 2 && !login.is_anonymous {
            scope.spawn(|| {
                if let Ok(mut ftp2_2) = create_ftp_stream(FTP2_HOST, &login) {
                    run_downloader(&mutex, &mut ftp2_2, None);
                };
            });
        };

        if num_downloads > 3 && !login.is_anonymous {
            scope.spawn(|| {
                if let Ok(mut ftp1_2) = create_ftp_stream(FTP1_HOST, &login) {
                    run_downloader(&mutex, &mut ftp1_2, Some(&failed_downloads));
                };
            });
        };

        if num_downloads > 4 && !login.is_anonymous {
            scope.spawn(|| {
                if let Ok(mut ftp3_1) = create_ftp_stream(FTP3_HOST, &login) {
                    run_downloader(&mutex, &mut ftp3_1, Some(&failed_downloads));
                };
            });
        };

        if num_downloads > 5 && !login.is_anonymous {
            scope.spawn(|| {
                if let Ok(mut ftp3_2) = create_ftp_stream(FTP3_HOST, &login) {
                    run_downloader(&mutex, &mut ftp3_2, Some(&failed_downloads));
                };
            });
        };
    });

    let num_failed = { failed_downloads.lock().unwrap().len() };

    let success = if num_failed == 0 {
        true
    } else {
        println!("Trying to re-download {num_failed} files.");
        let still_failed_downloads = Mutex::new(Vec::<WhdloadItem>::new());
        match run_downloader(&failed_downloads, &mut ftp2, Some(&still_failed_downloads)) {
            Ok(()) => {
                if still_failed_downloads.lock().unwrap().len() == 0 {
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                eprintln!("{e}");
                false
            }
        }
    };

    if success {
        println!("Finished successfully.");
        for lf in &localfiles {
            if remotefiles.binary_search(lf).is_err() {
                println!("del: {:?}", lf)
            }
        }
        localfiles::remove_old_dats()?;
    } else {
        println!("Sync completed with errors.");
    }

    Ok(())
}
