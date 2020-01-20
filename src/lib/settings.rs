use serde::{ Serialize, Deserialize };
use serde_json::{ to_writer_pretty, from_reader };
use std::io::{ BufWriter, BufReader };
use std::fs::{ File, OpenOptions };
use std::path::{ PathBuf };
use std::convert::{ From };

use super::error::{ Result, Error };

///
/// A structure holding the state of the programs settings.
/// 
/// # Example
/// 
/// ```
/// let settings_file: PathBuf = PathBuf::from("./settings.json");
/// let settings: Settings = Settings::new(&settings_file);
/// ```
///
#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    /// Use unicode characters in the programs output.
    use_unicode: bool,
    /// Use ANSI formatting in the programs output.
    use_colors: bool
}

impl Settings {
    ///
    /// Load the settings state from the JSON file, path.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let settings_path: PathBuf = PathBuf:from("./settings.json");
    /// let settings: Settings = Settings::new(&settings_path)?;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Throughs a errors for IO and JSON.
    /// 
    pub fn new(path: &PathBuf) -> Result<Settings> {
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(PathBuf::from(path))?;
        
        //
        // Write the default settings to the file and use them, if it did not
        // exist prior to opening.
        //
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

    ///
    /// Get the `use_unicode` setting.
    /// 
    /// # Example
    /// 
    /// ```
    /// settings.use_unicode();
    /// ```
    ///
    pub fn use_unicode(&self) -> bool {
        self.use_unicode
    }

    ///
    /// Get the `use_colors` setting.
    /// 
    /// # Example
    /// 
    /// ```
    /// settings.use_colors();
    /// ```
    ///
    pub fn use_colors(&self) -> bool {
        self.use_colors
    }
}