use std::fmt;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    Bincode(bincode::Error),
    Closed,
    IdCollision,
    Custom(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Error {
    pub fn custom(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Custom(Box::new(err))
    }
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IoError. {e}"),
            Error::Bincode(e) => write!(f, "Bincode. {e}"),
            Error::Closed => write!(f, "Connection closed."),
            Error::IdCollision => write!(f, "Id collision occured."),
            Error::Custom(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::Bincode(e)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
