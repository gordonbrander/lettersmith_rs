use std::fmt;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Error { msg: msg.into() }
    }

    /// Create error from any type that implements error trait
    pub fn from_error(err: impl std::error::Error) -> Self {
        Error::new(err.to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error {
            msg: format!("{}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            msg: format!("{}", err),
        }
    }
}
