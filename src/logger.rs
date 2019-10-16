use std::io;

pub struct Logger {
    application_name: String,
}

impl Logger {
    pub fn new(application_name: String) -> Logger {
        Logger {
            application_name: application_name,
        }
    }

    pub fn error(&self, e: Error) -> io::Result<()> {
        println!("{}: error: {}!", self.application_name, e.to_string());
        e.to_result()
    }
}

pub enum Error {
    InvalidCommandLine,
    MissingTargetFile(String),
}

impl Error {
    pub fn to_string(&self) -> String {
        match &self {
            Error::InvalidCommandLine => format!("invalid command line"),
            Error::MissingTargetFile(path) => format!("missing target file '{}'", path),
        }
    }

    pub fn to_result(&self) -> io::Result<()> {
        match &self {
            Error::InvalidCommandLine => Err(io::Error::from(io::ErrorKind::NotFound)),
            Error::MissingTargetFile(_) => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }
}