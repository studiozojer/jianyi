use std::fmt;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Usage(String),
    NotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Generic(m) => write!(f, "{m}"),
            Error::Usage(m) => write!(f, "usage: {m}"),
            Error::NotFound(m) => write!(f, "not found: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::Generic(_) => 1,
            Error::Usage(_) => 2,
            Error::NotFound(_) => 4,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Error::Generic(_) => "generic",
            Error::Usage(_) => "usage",
            Error::NotFound(_) => "not_found",
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Generic(format!("json: {e}"))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Generic(format!("io: {e}"))
    }
}
