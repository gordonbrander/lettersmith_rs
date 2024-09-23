use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Json(serde_json::Error),
    Liquid(liquid::Error),
    ValueError,
    Other,
}

impl Error {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Error {
            msg: msg.into(),
            kind,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(error) => Some(error),
            ErrorKind::Json(error) => Some(error),
            ErrorKind::Liquid(error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::new(ErrorKind::Io(error), "IO error")
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::new(ErrorKind::Json(error), "JSON error")
    }
}

impl From<liquid::Error> for Error {
    fn from(error: liquid::Error) -> Self {
        Error::new(ErrorKind::Liquid(error), "Liquid error")
    }
}
