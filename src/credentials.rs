

pub struct Credentials {
    pub username: String,
    pub password: String,
    is_anonymous: bool,
    max_connections: u8,
}

impl Credentials {
    pub fn new() -> Self {
        Self {
            username: "ftp".into(),
            password: "amiga".into(),
            is_anonymous: true,
            max_connections: 1
        }
    }
}