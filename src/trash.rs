extern crate dirs;
extern crate regex;

use std::io::{Result, Error, ErrorKind};
use std::fs::{rename, create_dir, read_dir};
use dirs::home_dir;
use std::path::{PathBuf, Path};
use clap::{App};
use regex::Regex;

pub struct Trash {
    directory: PathBuf,
    verbose: bool
}

impl Trash {
    pub fn new() -> Trash {
        let mut directory = home_dir().unwrap_or(PathBuf::from(String::from("/root")));

        directory.push(".trash");

        return Trash {
            directory: directory,
            verbose: false
        };
    }

    pub fn main(&mut self) -> Result<()> {
        let cli = load_yaml!("cli.yml");
        let matches = App::from_yaml(cli).get_matches();

        self.init()?;

        if matches.is_present("verbose") {
            self.verbose = true;
        }

        if matches.is_present("restore") {
            return self.restore(matches.value_of("INPUT").unwrap());
        } else if matches.is_present("list") {
            match Regex::new(matches.value_of("INPUT").unwrap()) {
                Ok(pattern) => return self.list(pattern),
                Err(_) => return Trash::e_invalid_input()
            }
        } else {
            return self.delete(matches.value_of("INPUT").unwrap())
        }
    }

    pub fn init(&self) -> Result<()> {
        if !self.directory.exists() {
            return create_dir(&self.directory);
        } else {
            return Ok(());
        }
    }

    pub fn delete(&self, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            let mut destination = PathBuf::from(self.directory.as_path());

            self.log(format!("deleting {}...", path).as_str());

            destination.push(path);
            return rename(path, destination);
        } else {
            return Trash::e_not_found();
        }
    }

    pub fn restore(&self, name: &str) -> Result<()> {
        let mut location = PathBuf::from(self.directory.as_path());

        location.push(name);

        if location.exists() {
            let mut destination = PathBuf::from(String::from("."));

            self.log(format!("restoring {}...", name).as_str());

            destination.push(name);
            return rename(location, destination);
        } else {
            return Trash::e_not_found();
        }
    }

    pub fn list(&self, pattern: Regex) -> Result<()> {
        let dir_entries = read_dir(&self.directory)?;

        for entry in dir_entries {
            match entry?.file_name().into_string() {
                Ok(entry) => {
                    if pattern.is_match(entry.as_str()) {
                        println!("  • {}", entry);
                    }
                },
                Err(_) => ()
            }
        }

        return Ok(());
    }

    pub fn e_not_found<T>() -> Result<T> {
        return Err(Error::from(ErrorKind::NotFound));
    }

    pub fn e_invalid_input<T>() -> Result<T> {
        return Err(Error::from(ErrorKind::InvalidInput));        
    }

    fn log(&self, message: &str) -> () {
        if self.verbose {
            println!("trash: info: {}!", message);
        }
    }
}