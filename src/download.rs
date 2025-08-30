use super::remote::create_ftp_stream;
use super::whdload::WhdloadItem;
use super::Credentials;
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
    ftp2: &mut FtpStream,
    login: &Credentials,
) -> Result<Vec<WhdloadItem>, FtpError> {
    let num_downloads = to_download.len();
    let queue = Mutex::new(to_download);

    thread::scope(|scope| {
        let mut threads = vec![];
        let mut failed_downloads = vec![];

        if num_downloads > 2 {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP1, login)
                    .and_then(|mut ftp1| run_downloader(&queue, &mut ftp1, false, "FTP1-1"))
            }));
        };

        if num_downloads > 4 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP3, login)
                    .and_then(|mut ftp3| run_downloader(&queue, &mut ftp3, false, "FTP3-1"))
            }));
        };

        if num_downloads > 6 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP2, login)
                    .and_then(|mut ftp2| run_downloader(&queue, &mut ftp2, false, "FTP2-2"))
            }));
        };

        if num_downloads > 8 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP1, login)
                    .and_then(|mut ftp1| run_downloader(&queue, &mut ftp1, false, "FTP1-2"))
            }));
        };

        if num_downloads > 10 && !login.is_anonymous {
            threads.push(scope.spawn(|| {
                create_ftp_stream(FTP3, login)
                    .and_then(|mut ftp3| run_downloader(&queue, &mut ftp3, false, "FTP3-2"))
            }));
        };

        if let Err(e) = run_downloader(&queue, ftp2, true, "FTP2-1") {
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
) -> Result<Requeue, FtpError> {
    let mut requeue = vec![];
    while let Some(item) = { items.lock().unwrap().pop() } {
        let path = item.get_remote_path();
        println!("[{tag}]: {path}");
        match ftp.retr_as_stream(&path) {
            Ok(mut stream) => {
                item.save_file(&mut stream).unwrap();
                ftp.finalize_retr_stream(stream)?;
            }

            Err(FtpError::UnexpectedResponse(response))
                if response.status == Status::FileUnavailable && !is_primary =>
            {
                requeue.push(item)
            }
            Err(error) => {
                if is_primary {
                    eprintln!("Failed to download {path} from primary FTP server");
                    return Err(error);
                } else {
                    requeue.push(item);
                    eprintln!("{error}");
                    break;
                };
            }
        };
    }
    Ok(requeue)
}
