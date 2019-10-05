extern crate dirs;
extern crate regex;

use std::io::{
    Result,
    Error,
    ErrorKind,
    Write,
};
use std::fs::{
    rename,
    create_dir,
    read_dir,
    read_to_string,
    remove_file,
    remove_dir_all,
    File,
};
use std::path::{
    PathBuf,
};
use dirs::home_dir;
use clap::{
    App,
    AppSettings,
};
use regex::Regex;

pub struct Trash {
    meta_directory: PathBuf,
    data_directory: PathBuf,
}

impl Trash {
    pub fn new() -> Result<Trash> {
        let mut directory = home_dir().unwrap_or_default();

        directory.push(".trash");

        let mut meta_directory = directory.clone();
        let mut data_directory = directory.clone();

        meta_directory.push("meta");
        data_directory.push("data");

        create_dir(&directory).unwrap_or_default();
        create_dir(&meta_directory).unwrap_or_default();
        create_dir(&data_directory).unwrap_or_default();

        Ok(Trash {
            meta_directory: meta_directory,
            data_directory: data_directory
        })
    }

    pub fn main(&mut self) -> Result<()> {
        let cli = load_yaml!("cli.yml");
        let matches = App::from_yaml(cli).settings(&[
            AppSettings::DisableHelpFlags,
            AppSettings::VersionlessSubcommands,
            AppSettings::SubcommandRequiredElseHelp,
        ]).get_matches();

        match matches.subcommand() {
            ("delete", Some(sub_matches)) =>
                sub_matches.values_of("FILE").unwrap().try_for_each(|file| self.delete(file))?,
            ("restore", Some(sub_matches)) =>
                sub_matches.values_of("FILE").unwrap().try_for_each(|file| self.restore(file))?,
            ("list", Some(sub_matches)) =>
                self.list(
                    Regex::new(sub_matches.value_of("PATTERN").unwrap_or("")).unwrap(),
                    sub_matches.is_present("simple")
                )?,
            ("empty", Some(_)) => self.empty()?,
            _ => Trash::e_invalid_input()?
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
        } else {
            Trash::e_not_found()?;
        }

        Ok(())
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

    pub fn list(&self, pattern: Regex, simple: bool) -> Result<()> {
        let items = read_dir(&self.data_directory)?;
        let item_count = read_dir(&self.data_directory)?.count();

        if item_count > 0 {
            if !simple {
                println!("Listings in trash:")
            }

            for item in items {
                let item = item.unwrap().file_name().into_string().unwrap();
                
                if pattern.is_match(item.as_str()) {
                    if simple {
                        println!("{}", item);
                    } else {
                        println!(" • {}", item);
                    }
                }
            }
        } else {
            println!("Your trash is empty.");
        }

        Ok(())
    }

    pub fn empty(&self) -> Result<()> {
        for item in read_dir(self.data_directory.clone())? {
            let item = item?.path();

            if item.is_dir() {
                remove_dir_all(item)?;
            } else {
                remove_file(item)?;
            }
        }

        for item in read_dir(self.meta_directory.clone())? {
            remove_file(item?.path())?;
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