use netrc::Netrc;

const FTP_SERVERS: [&str; 3] = ["ftp2.grandis.nu", "ftp.grandis.nu", "grandis.nu"];
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub is_anonymous: bool,
}

impl Credentials {
    pub fn new_from_netrc() -> Option<Credentials> {
        let nrc = Netrc::new().ok()?;
        for (host, auth) in nrc.hosts {
            if FTP_SERVERS
                .iter()
                .any(|s| *s == host.to_lowercase())
            {
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
