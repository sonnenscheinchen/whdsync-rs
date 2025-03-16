use super::remote::create_ftp_stream;
use super::whdload::WhdloadItem;
use super::Credentials;
use anyhow::{Error, Result};
use std::sync::Mutex;
use std::thread;
use suppaftp::{FtpError, FtpStream, Status};

type Queue<'a> = &'a Mutex<Vec<WhdloadItem>>;
type Requeue = Vec<WhdloadItem>;

// pub const FTP1: &str = "localhost:2121";
// pub const FTP2: &str = "localhost:2122";
// pub const FTP3: &str = "localhost:2123";
pub const FTP1: &str = "ftp.grandis.nu:21";
pub const FTP2: &str = "ftp2.grandis.nu:21";
pub const FTP3: &str = "grandis.nu:21";

pub fn download(
    to_download: Vec<WhdloadItem>,
    mut ftp2: &mut FtpStream,
    login: &Credentials,
) -> Result<Vec<WhdloadItem>, Error> {
    let num_downloads = to_download.len();
    let queue = Mutex::new(to_download);

    thread::scope(|scope| {
        let mut threads = vec![];
        let mut failed_downloads = vec![];

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
        Ok(failed_downloads)
    })
}

pub fn run_downloader(
    items: Queue,
    ftp: &mut FtpStream,
    is_primary: bool,
    tag: &str,
) -> Result<Requeue, Error> {
    let mut requeue = vec![];
    loop {
        let maybe_item = { items.lock().unwrap().pop() };
        if let Some(item) = maybe_item {
            let path = item.get_remote_path();
            println!("[{}]: {}", tag, path);
            match ftp.retr_as_stream(&path) {
                Ok(mut stream) => {
                    item.save_file(&mut stream)?;
                    ftp.finalize_retr_stream(stream)?;
                }
                Err(error) => {
                    if !is_primary {
                        requeue.push(item); // requeue silently
                    } else {
                        eprintln!("Failed to download {} from primary FTP server", path);
                        return Err(error.into());
                    }
                    match error {
                        FtpError::UnexpectedResponse(response) => match response.status {
                            Status::FileUnavailable => {} // ignore, requeued earlier
                            _ => return Err(FtpError::UnexpectedResponse(response).into()),
                        },
                        _ => return Err(error.into()),
                    }
                }
            }
        } else {
            break;
        }
    }
    Ok(requeue)
}
