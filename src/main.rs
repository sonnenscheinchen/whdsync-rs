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

    //let mut localfiles = vec![];
    //let mut remotefiles: Vec<whdload::WhdloadItem> = vec![];

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


    // let t1 = thread::spawn(|| {
    //     localfiles::find_local_files()
    // });
    
    // let t2 = thread::spawn(|| {
    //     remotefiles::find_remote_files(&mut ftp2)
    // });


    println!("l: {:#?}", local);
    println!("r: {:#?}", remote);

    ftp2.quit()?;
    Ok(())

}
