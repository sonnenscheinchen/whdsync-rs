use crate::whdload::WhdloadItem;
use anyhow::Result;
use core::time::Duration;
use std::sync::Mutex;
use suppaftp::{FtpError, FtpStream, Status};

type QUEUE<'a> = &'a Mutex<Vec<WhdloadItem>>;
type REQUEUE<'a> = Option<&'a Mutex<Vec<WhdloadItem>>>;

pub fn run_downloader(items: QUEUE, ftp: &mut FtpStream, failed: REQUEUE) -> Result<()> {
    loop {
        let maybe_item = { items.lock().unwrap().pop() };
        if let Some(item) = maybe_item {
            println!("{:?}: {}", ftp, item.path);
            match ftp.retr_as_stream(&item.path.replace(" ", "_")) {
            //match ftp.retr_as_stream(&item.path) {
                Ok(mut stream) => {
                    item.save_file(&mut stream)?;
                    ftp.finalize_retr_stream(stream)?;
                }
                Err(error) => {
                    if let Some(f) = failed {
                        // requeue silently
                        f.lock().unwrap().push(item);
                    } else {
                        eprintln!("Failed to download {}", &item.path)
                    }
                    match error {
                        FtpError::UnexpectedResponse(response) => match response.status {
                            Status::FileUnavailable => {} // ignore here, requeued earlier
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
    //println!("end: {:?}", d);
    Ok(())
}
