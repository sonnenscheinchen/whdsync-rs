use super::{FTP1, FTP2, FTP3};
use netrc::Netrc;
use std::env::var;

const FTP_SERVERS: &[&str] = &[FTP1, FTP2, FTP3];

pub struct Credentials {
    pub username: String,
    pub password: String,
    pub is_anonymous: bool,
}

impl Credentials {
    pub fn from_env() -> Option<Credentials> {
        match (var("TURRAN_USER"), var("TURRAN_PASSWORD")) {
            (Ok(username), Ok(password)) => Some(Credentials {
                username,
                password,
                is_anonymous: false,
            }),
            (_, _) => None,
        }
    }
    pub fn from_netrc() -> Option<Credentials> {
        let nrc = Netrc::new().ok()?;
        for (host, auth) in nrc.hosts {
            if FTP_SERVERS.iter().any(|s| {
                s.split(':')
                    .next()
                    .is_some_and(|name| name == host.to_ascii_lowercase())
            }) {
                return Some(Credentials {
                    username: auth.login,
                    password: auth.password,
                    is_anonymous: false,
                });
            }
        }
        None
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            username: "ftp".into(),
            password: "amiga".into(),
            is_anonymous: true,
        }
    }
}

#[test]
fn test_my_credentials_from_netrc() {
    let c = Credentials::from_netrc();
    assert!(c.is_some());
}

#[test]
fn test_my_credentials_from_env() {
    let c = Credentials::from_env();
    assert!(c.is_some());
}
