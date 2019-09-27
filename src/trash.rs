extern crate dirs;
extern crate regex;

use std::io::{Result, Error, ErrorKind, Write};
use std::fs::{rename, create_dir, read_dir, read_to_string, remove_file, File};
use dirs::home_dir;
use std::path::{PathBuf};
use clap::{App};
use regex::Regex;

pub struct Trash {
    meta_directory: PathBuf,
    data_directory: PathBuf
}

impl Trash {
    pub fn new() -> Result<Trash> {
        let mut directory = home_dir().unwrap_or(PathBuf::from(String::from("/root")));

        directory.push(".trash");

        let mut meta_directory = directory.clone();
        let mut data_directory = directory.clone();

        meta_directory.push("meta");
        data_directory.push("data");

        for path in [&directory, &meta_directory, &data_directory].iter() {
            if !PathBuf::from(path).exists() {
                create_dir(path)?;
            }
        }

        Ok(Trash {
            meta_directory: meta_directory,
            data_directory: data_directory
        })
    }

    pub fn main(&mut self) -> Result<()> {
        let cli = load_yaml!("cli.yml");
        let matches = App::from_yaml(cli).get_matches();

        if matches.is_present("restore") {
            self.restore(matches.value_of("INPUT").unwrap())?;
        } else if matches.is_present("list") {
            match Regex::new(matches.value_of("INPUT").unwrap()) {
                Ok(pattern) => self.list(pattern)?,
                Err(_) => Trash::e_invalid_input()?
            };
        } else {
            self.delete(matches.value_of("INPUT").unwrap())?;
        }

        Ok(())
    }

    pub fn delete(&self, target: &str) -> Result<()> {
        let target_location = PathBuf::from(String::from(target));
        let target_name = target_location.file_name().unwrap().to_str().unwrap();

        if target_location.exists() {
            let mut data_destination = self.data_directory.clone();
            let mut meta_destination = self.meta_directory.clone();

            data_destination.push(target_name);
            meta_destination.push(target_name);

            let mut meta_file = File::create(meta_destination.clone())?;

            meta_file.write_all(target_location.as_path().to_str().unwrap().as_bytes())?;
            meta_file.sync_data()?;

            rename(target_location, data_destination)?;

            return Ok(());
        } else {
            return Trash::e_not_found();
        }
    }

    pub fn restore(&self, name: &str) -> Result<()> {
        let mut data_location = self.data_directory.clone();
        let mut meta_location = self.meta_directory.clone();

        data_location.push(name);
        meta_location.push(name);

        if data_location.exists() && meta_location.exists() {
            let destination = read_to_string(&meta_location)?;

            rename(data_location, destination)?;
            remove_file(meta_location)?;
        } else {
            Trash::e_not_found()?;
        }

        Ok(())
    }

    pub fn list(&self, pattern: Regex) -> Result<()> {
        let dir_entries = read_dir(self.data_directory.clone())?;

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

        Ok(())
    }

    pub fn e_not_found<T>() -> Result<T> {
        return Err(Error::from(ErrorKind::NotFound));
    }

    pub fn e_invalid_input<T>() -> Result<T> {
        return Err(Error::from(ErrorKind::InvalidInput));        
    }
}