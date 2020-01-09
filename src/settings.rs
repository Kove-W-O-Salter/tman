use serde::{Serialize, Deserialize};
use serde_json::{to_writer_pretty, from_reader};
use std::io::{BufWriter, BufReader};
use std::fs::{File, OpenOptions};
use std::path::{PathBuf};
use std::convert::{From};
use super::error::{Result, Error};

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    use_unicode: bool,
    use_colors: bool
}

impl Settings {
    pub fn new(path: &PathBuf) -> Result<Settings> {
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(PathBuf::from(path))?;
        
        match from_reader(BufReader::new(&file)) {
            Err(json_error) => {
                if json_error.is_eof() {
                    to_writer_pretty(BufWriter::new(&file), &Settings::default())?;
                    Ok(Settings::default())
                } else {
                    Err(Error::from(json_error))
                }
            },
            Ok(settings) => Ok(settings)
        }
    }

    pub fn use_unicode(&self) -> bool {
        self.use_unicode
    }

    pub fn use_colors(&self) -> bool {
        self.use_colors
    }
}