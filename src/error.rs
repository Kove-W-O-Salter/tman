pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    InvalidArguments,
    InvalidJSON(usize, usize),
    InvalidRegex(regex::Error),
    MissingTarget(String),
    MissingTargetPredicate,
    Unknown,
}

impl Error {
    fn print(&self) -> String {
        format!("trash: error: {}!", match self {
            Error::InvalidArguments => String::from("invalid arguments"),
            Error::InvalidJSON(line, column) => format!("syntax error on line {}, column {}, of settings.json or cache.json", line, column),
            Error::InvalidRegex(regex_error) => {
                String::from(
                    match regex_error {
                    regex::Error::Syntax(_) => "syntax error in regular expression",
                    regex::Error::CompiledTooBig(_) => "oversized regular expression",
                    _ => "invalid regular expression"
                    }
                )
            },
            Error::MissingTarget(target) => format!("could not locate '{}'", target),
            Error::MissingTargetPredicate => String::from("could not locate any target satisfying given conditions"),
            Error::Unknown => String::from("unknown")
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
    fn from(regex_error: regex::Error) -> Self {
        Error::InvalidRegex(regex_error)
    }
}

pub fn finish<T>(result: Result<T>) {
    match result {
        Ok(_) => (),
        Err(error) => println!("{}", error.print()),
    }
}