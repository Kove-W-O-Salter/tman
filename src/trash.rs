use std::fs::{
    rename,
    create_dir,
    canonicalize,
    remove_file,
    remove_dir_all,
};
use std::path::PathBuf;
use dirs::home_dir;
use clap::{
    App,
    AppSettings,
    Arg
};
use chrono::Utc;
use regex::Regex;
use super::cache::Cache;
use super::error::{
    Result,
    Error,
};

pub struct Trash {
    cache: Cache,
    data_path: PathBuf,
}

impl Trash {
    pub fn new() -> Result<Trash> {
        let mut directory = home_dir().unwrap_or_default();

        directory.push(".trash");

        let mut cache_path = directory.clone();
        let mut data_path = directory.clone();

        cache_path.push("cache.json");
        data_path.push("data");

        create_dir(&directory).unwrap_or_default();
        create_dir(&data_path).unwrap_or_default();

        let cache = Cache::new(cache_path)?;

        Ok(Trash {
            cache: cache,
            data_path: data_path,
        })
    }

    pub fn main(&mut self) -> Result<()> {
        let max_argument_values: u64 = 18_446_744_073_709_551_615;

        let matches = App::new("Trash")
            .name("trash")
            .version("1.0")
            .author("Kove Salter <kove.w.o.salter@gmail.com>")
            .about("Safely manage your trash")
            .setting(AppSettings::ArgRequiredElseHelp)
            .arg(Arg::with_name("delete")
                .long("delete")
                .short("d")
                .help("Delete an item, storing it in the trash")
                .takes_value(true)
                .value_name("FILES")
                .max_values(max_argument_values)
                .conflicts_with_all(&[ "restore", "list", "pattern", "empty" ]))
            .arg(Arg::with_name("restore")
                .long("restore")
                .short("r")
                .help("Restore files from the trash")
                .takes_value(true)
                .value_name("FILES")
                .max_values(max_argument_values)
                .conflicts_with_all(&[ "delete", "list", "pattern", "empty" ]))
            .arg(Arg::with_name("list")
                .long("list")
                .short("l")
                .help("List items in the trash")
                .conflicts_with_all(&[ "delete", "restore", "empty" ]))
            .arg(Arg::with_name("pattern")
                .long("pattern")
                .short("p")
                .help("Set a pattern for --list")
                .value_name("PATTERN")
                .takes_value(true)
                .requires("list")
                .conflicts_with_all(&[ "delete", "restore", "empty" ]))
            .arg(Arg::with_name("empty")
                .long("empty")
                .short("e")
                .help("Permenantly delete all trash items")
                .takes_value(false)
                .conflicts_with_all(&[ "delete", "restore", "list", "pattern" ]))
            .get_matches();

        if let Some(mut files) = matches.values_of("delete") {
            files.try_for_each(|file| self.delete(String::from(file)))?;
        } else if let Some(mut files) = matches.values_of("restore") {
            files.try_for_each(|file| self.restore(String::from(file)))?;
        } else if matches.is_present("list") {
            self.list(Regex::new(matches.value_of("pattern").unwrap_or(""))?, false)?;
        } else if matches.is_present("empty") {
            self.empty()?;
        } else {
            Err(Error::InvalidArguments)?;
        }

        self.cache.commit()
    }

    pub fn delete(&mut self, target: String) -> Result<()> {
        if PathBuf::from(&target).exists() {
            let location = canonicalize(PathBuf::from(&target))?;
            let name = location.file_name().unwrap().to_str().unwrap();

            let mut destination = self.data_path.clone();
            let id = format!("{:?}", Utc::now());
            
            destination.push(id.clone());
            
            self.cache.push(String::from(name), id, String::from(location.to_str().unwrap()));

            rename(location, destination)?;

            Ok(())
        } else {
            Err(Error::MissingTarget(target.clone()))
        }
    }

    pub fn restore(&mut self, target: String) -> Result<()> {
        let mut location: PathBuf;
        let mut destination: PathBuf;
        let mut ensure_unique: bool = false;
        let entries = self.cache.pop(|key| key == &target)?;

        if entries.len() > 1 {
            ensure_unique = true;
        }

        for entry in entries {
            location = self.data_path.clone();
            destination = if ensure_unique {
                PathBuf::from(format!("{}_{}", entry.origin(), entry.id()))
            } else {
                PathBuf::from(entry.origin())
            };

            location.push(entry.id());

            if location.exists() {
                rename(location.clone(), destination)?;
            } else {
                Err(Error::MissingTarget(entry.name().clone()))?;
            }
        }

        Ok(())
    }

    pub fn list(&self, pattern: Regex, simple: bool) -> Result<()> {
        let entries = self.cache.values(|key| pattern.is_match(key.as_str()));

        if !entries.is_empty() {
            for entry in entries.iter() {
                if simple {
                    println!("{}", entry.name());
                } else {
                    println!("🢒 {} ({}) <- {}", entry.name(), entry.id(), entry.origin());
                }
            }
        } else {
            println!("Your trash is empty.");
        }

        Ok(())
    }

    pub fn empty(&mut self) -> Result<()> {
        let mut location;

        for entry in self.cache.pop(|_| true)?.iter() {
            location = PathBuf::new();
            location.push(&self.data_path);
            location.push(entry.id());

            if location.is_dir() {
                remove_dir_all(location)?;
            } else {
                remove_file(location)?;
            }
        }

        Ok(())
    }
}