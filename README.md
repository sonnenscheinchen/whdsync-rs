# About
A command line tool for Linux and Windows to sync Retroplay's WHDLoad collection from the TURRAN FTP server to your local drive.
The tool is (over)optimized for speed, not for features. There are no filtering capabilities. It's get all or nothing.

# How it works
Probably tl;dr, but here we go...
First it scans for files in the directory you provided and builds a local collection. At the same time it connects to the primary server updates the dat files, unzips, parses the XML and builds a remote collection. The diff of the two collections are the files to be downloaded. Depending on the size of the diff, the tool will open up to two connections on the servers to distribute the load and increase download speed. If a file was not found on the mirror (out of sync) it will be downloaded from the primary FTP. If all this succeeds the tool will (optionally) remove you old files. If time changes and the collection gets updated just run the tool again to get the newest stuff.

# How to build
Install `rust` and `cargo` and clone the repo.
```
cd whdsync-rs
cargo build --release
```
The binary is `target/release/whdsync-rs`. Copy it anywhere you like.

# Usage
If you have an account on the FTP you can put your username/password in the [.netrc](https://www.gnu.org/software/inetutils/manual/html_node/The-_002enetrc-file.html) (not supported under Windows). This allows the tool to use two connections per server and probably avoids server-full-issues. If you start from scratch create a new directory and point the tool to it. This will basically match the "root" directory "Retroplay WHDLoad Packs" on the FTP.

Run `whdsync-rs /path/to/target-dir` to download and update new files.
Run `whdsync-rs /path/to/target-dir -d` or `whdsync-rs /path/to/target-dir --delete` to also delete old files.
