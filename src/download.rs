use super::whdload::WhdloadItem;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

type Queue<'a> = &'a Vec<WhdloadItem>;
type Requeue = Vec<WhdloadItem>;

// pub const FTP1: &str = "localhost:2121";
// pub const FTP2: &str = "localhost:2122";
pub const FTP1: &str = "ftp.grandis.nu:21";
pub const FTP2: &str = "ftp2.grandis.nu:21";
pub const HTTP2: &str = "http://ftp.grandis.nu/turran/FTP/Retroplay%20WHDLoad%20Packs/";
pub const HTTP1: &str = "http://ftp2.grandis.nu/turran/FTP/Retroplay%20WHDLoad%20Packs/";

static INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn download(queue: Vec<WhdloadItem>) -> Result<Vec<WhdloadItem>, ureq::Error> {
    let num_downloads = queue.len();
    thread::scope(|scope| {
        let mut threads = vec![];
        let mut failed_downloads = vec![];

        if num_downloads > 2 {
            threads.push(scope.spawn(|| run_downloader(&queue, false, "FTP1-1")));
        };

        if num_downloads > 4 {
            threads.push(scope.spawn(|| run_downloader(&queue, false, "FTP1-2")));
        };

        if let Err(e) = run_downloader(&queue, true, "FTP2-1") {
            eprintln!("Primary downloader finished unexpectly.");
            eprintln!("{e}");
            {
                INDEX.store(num_downloads + 1, Ordering::SeqCst);
            };
            eprintln!("Waiting for threads to finish.");
            for t in threads {
                let _ = t.join().unwrap();
            }
            INDEX.store(0, Ordering::SeqCst);
            return Err(e);
        }
        for t in threads {
            match t.join().unwrap() {
                Ok(mut failed) => failed_downloads.append(&mut failed),
                Err(e) => eprintln!("{e}"),
            };
        }
        INDEX.store(0, Ordering::SeqCst);
        Ok(failed_downloads)
    })
}

pub fn run_downloader(items: Queue, is_primary: bool, tag: &str) -> Result<Requeue, ureq::Error> {
    let mut requeue = vec![];
    let base_url = if is_primary { HTTP2 } else { HTTP1 };

    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent("whdsync-rs/0.3.0")
        .build();

    let agent = ureq::Agent::new_with_config(config);

    while let Some(item) = items.get(INDEX.fetch_add(1, Ordering::SeqCst)) {
        let path = item.get_remote_path();
        println!("[{tag}]: {path}");
        let url = format!("{base_url}{path}", path = path.replace('&', "%26"));
        match agent.get(url).call() {
            Ok(mut response) => {
                let body = response.body_mut();
                let server_size = body.content_length().unwrap_or_default();
                let xml_size = item.get_file_size();
                if server_size != xml_size {
                    eprintln!("Warning: {path} File size mismatch dat-file: {xml_size}, server: {server_size}");
                    if !is_primary {
                        requeue.push(item.clone());
                        continue;
                    } else {
                        // not sure what to do here... :-\
                    }
                }
                let mut reader = body.as_reader();
                let _ = item
                    .save_file(&mut reader)
                    .map_err(|e| eprintln!("Error: Failed to save file: {e}"));
            }
            Err(error) => match error {
                ureq::Error::StatusCode(404) => {
                    if is_primary {
                        eprintln!("Error: File {path} not found on primary server");
                        return Err(error);
                    } else {
                        requeue.push(item.clone());
                        eprintln!("{error}");
                    };
                }
                _ => {
                    if is_primary {
                        eprintln!("Error: Failed to download {path} from primary server: {error}");
                        return Err(error);
                    } else {
                        requeue.push(item.clone());
                        eprintln!("{error}");
                        break;
                    };
                }
            },
        }
    }
    Ok(requeue)
}
