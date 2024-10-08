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
    Tera(tera::Error),
    Value,
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Io(err) => write!(f, "{}", err),
            ErrorKind::Json(err) => write!(f, "{}", err),
            ErrorKind::Tera(err) => write!(f, "{:?}", err),
            ErrorKind::Value => write!(f, "{}", "Value error"),
            ErrorKind::Other => write!(f, "{}", "Other"),
        }
    }
}

impl Error {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Error {
            msg: msg.into(),
            kind,
        }
    }

    pub fn value(msg: impl Into<String>) -> Self {
        Error {
            msg: msg.into(),
            kind: ErrorKind::Value,
        }
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Error {
            msg: msg.into(),
            kind: ErrorKind::Other,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(error) => Some(error),
            ErrorKind::Json(error) => Some(error),
            ErrorKind::Tera(error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        write!(f, "{}", "\n\nSource:\n")?;
        write!(f, "{}", self.kind)?;
        Ok(())
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

impl From<tera::Error> for Error {
    fn from(error: tera::Error) -> Self {
        Error::new(ErrorKind::Tera(error), "Tera template error")
    }
}
