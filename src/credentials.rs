use super::{FTP1, FTP2, FTP3};
use netrc::Netrc;

const FTP_SERVERS: &[&str] = &[FTP1, FTP2, FTP3];
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub is_anonymous: bool,
}

impl Credentials {
    pub fn new_from_netrc() -> Option<Credentials> {
        let nrc = Netrc::new().ok()?;
        dbg!(&nrc);
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
fn test_my_credentials() {
    let c = Credentials::new_from_netrc().unwrap_or_default();
    assert_eq!(false, c.is_anonymous)
}
