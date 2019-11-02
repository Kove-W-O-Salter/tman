pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    InvalidArguments,
    InvalidCache,
    InvalidRegex,
    MissingTarget(String),
    Unknown,
}

impl Error {
    fn print(&self) -> String {
        format!("trash: error: {}!", match self {
            Error::InvalidArguments => format!("invalid arguments"),
            Error::InvalidCache => format!("invalid cache"),
            Error::InvalidRegex => format!("invalid regex"),
            Error::MissingTarget(target) => format!("could not locate '{}'", target),
            Error::Unknown => format!("unknown"),
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::Unknown
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error::InvalidCache
    }
}

impl From<regex::Error> for Error {
    fn from(_: regex::Error) -> Self {
        Error::InvalidRegex
    }
}

pub fn finish<T>(result: Result<T>) {
    match result {
        Ok(_) => (),
        Err(error) => println!("{}", error.print()),
    }
}
