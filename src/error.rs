pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    InvalidArguments,
    InvalidJSON(usize, usize),
    InvalidRegex,
    MissingTarget(String),
    MissingTargetPredicate,
    Unknown,
}

impl Error {
    fn print(&self) -> String {
        format!("trash: error: {}!", match self {
            Error::InvalidArguments => format!("invalid arguments"),
            Error::InvalidJSON(line, column) => format!("syntax error on line {}, column {}, of settings.json or cache.json", line, column),
            Error::InvalidRegex => format!("invalid regex"),
            Error::MissingTarget(target) => format!("could not locate '{}'", target),
            Error::MissingTargetPredicate => format!("could not locate any entries satisfying the predicate"),
            Error::Unknown => format!("unknown")
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::Unknown
    }
}

impl From<serde_json::Error> for Error {
    fn from(json_error: serde_json::Error) -> Self {
        if json_error.is_syntax() || json_error.is_data() {
            Error::InvalidJSON(json_error.line(), json_error.column())
        } else {
            Error::Unknown
        }
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