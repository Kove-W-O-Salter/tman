///
/// A custom result type with specialized errors.
///
pub type Result<T> = std::result::Result<T, Error>;

///
/// Custom errors that may occur.
///
pub enum Error {
    /// Program was called with invalid arguments.
    InvalidArguments,
    /// There was a JSON error in the settings or cache.
    InvalidJSON(usize, usize),
    /// An invalid regular expression was passed as an argument.
    InvalidRegex(regex::Error),
    /// Could not locate a target file or entry.
    MissingTarget(String),
    /// Could not locate a target file or entry satisfying a predicate.
    MissingTargetPredicate,
    /// A unknown error.
    Unknown,
}

impl Error {
    ///
    /// Display the error as a `String`.
    /// 
    /// # Example
    /// 
    /// ```
    /// Error::InvalidArguments.pring();
    /// ```
    ///
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

///
/// Conversions from IO errors to custom errors.
///
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::Unknown
    }
}

///
/// Conversions from JSON errors to custom errors.
///
impl From<serde_json::Error> for Error {
    fn from(json_error: serde_json::Error) -> Self {
        if json_error.is_syntax() || json_error.is_data() {
            Error::InvalidJSON(json_error.line(), json_error.column())
        } else {
            Error::Unknown
        }
    }
}

///
/// Conversions from regex errors to trash errors.
///
impl From<regex::Error> for Error {
    fn from(regex_error: regex::Error) -> Self {
        Error::InvalidRegex(regex_error)
    }
}

///
/// Finish a `Result` computating, writing to stdout on error and doing nothing
/// on success.
/// 
/// # Example
/// 
/// ```
/// finish(Err(Error::InvalidArguments));
/// ```
///
pub fn finish<T>(result: Result<T>) {
    match result {
        Ok(_) => (),
        Err(error) => println!("{}", error.print()),
    }
}