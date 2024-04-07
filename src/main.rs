use anyhow::{bail, Result};
use std::env::{args, set_current_dir};
use std::path::PathBuf;
use std::thread;
use suppaftp::FtpStream;
mod localfiles;
mod remotefiles;
mod whdload;
mod credentials;

fn main() -> Result<()> {
    println!("Hello, world!");

    let target_dir = match args().nth(1) {
        Some(dir) => PathBuf::from(dir),
        None => bail!("Need a valid target directory."),
    };

    if !target_dir.is_dir() {
        bail!("Target directory does not exist.")
    };

    set_current_dir(&target_dir)?;

    let login = credentials::Credentials::new();
    let mut ftp2 = FtpStream::connect("ftp2.grandis.nu:21")?;
    ftp2.login(login.username, login.password)?;

    let (localfiles, remotefiles) = thread::scope(|s| {
        let t1 = s.spawn(
            localfiles::find_local_files
        );
        let t2 = s.spawn(|| 
            remotefiles::find_remote_files(&mut ftp2)
        );
        (t1.join().unwrap(), t2.join().unwrap())
    });

    let local = localfiles?;
    let remote = remotefiles?;

    //local.retain(|w| remote.binary_search(w).is_err());
    for rf in &remote {
        if local.binary_search(rf).is_err() {
            println!("dl: {:?}", rf)
        }
    }

    for lf in &local {
        if remote.binary_search(lf).is_err() {
            println!("del: {:?}", lf)
        }
    }


    ftp2.quit()?;
    Ok(())

}
