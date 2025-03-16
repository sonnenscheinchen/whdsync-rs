use crate::whdload::WhdloadItem;
use anyhow::{Error, Result};
use std::sync::Mutex;
use suppaftp::{FtpError, FtpStream, Status};

type Queue<'a> = &'a Mutex<Vec<WhdloadItem>>;
type Requeue = Vec<WhdloadItem>;

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
            println!("[{}]: {}", tag, item.path);
            match ftp.retr_as_stream(item.path.replace(" ", "_")) {
                Ok(mut stream) => {
                    item.save_file(&mut stream)?;
                    ftp.finalize_retr_stream(stream)?;
                }
                Err(error) => {
                    if !is_primary {
                        requeue.push(item); // requeue silently
                    } else {
                        eprintln!("Failed to download {} from primary FTP server", &item.path);
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
