use std::fmt::{Debug, Formatter, Result};

#[derive(PartialEq)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn from(msg: &str) -> Error {
        Error { msg: String::from(msg) }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("RegexError")
            .field("Internal Error", &self.msg)
            .finish()
    }
}
