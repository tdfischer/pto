use std::fmt;

#[derive(Clone)]
pub struct Auth {
    pub password: Option<String>,
    pub username: Option<String>
}

impl fmt::Debug for Auth {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "<Auth>")
    }
}

#[derive(Debug)]
pub struct AuthSession {
    auth: Auth
}

impl AuthSession {
    pub fn new() -> Self {
        AuthSession {
            auth: AuthSession::new_auth()
        }
    }

    fn new_auth() -> Auth {
        Auth {
            password: None,
            username: None
        }
    }

    pub fn consume(&mut self) -> Auth {
        let ret = self.auth.clone();
        self.auth = AuthSession::new_auth();
        ret
    }

    pub fn set_password(&mut self, password: String) {
        self.auth.password = Some(password);
    }

    pub fn set_username(&mut self, username: String) {
        self.auth.username = Some(username);
    }
}
