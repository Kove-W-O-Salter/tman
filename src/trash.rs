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
    ArgMatches,
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
        let mut directory: PathBuf = home_dir().unwrap_or_default();

        directory.push(".trash");

        let mut cache_path: PathBuf = directory.clone();
        let mut data_path: PathBuf = directory.clone();

        cache_path.push("cache.json");
        data_path.push("data");

        create_dir(&directory).unwrap_or_default();
        create_dir(&data_path).unwrap_or_default();

        Ok(Trash {
            cache: Cache::new(&cache_path)?,
            data_path: data_path,
        })
    }

    pub fn main(&mut self) -> Result<()> {
        let max_argument_values: u64 = 18_446_744_073_709_551_615;

        let matches: ArgMatches<'static> = App::new("Trash")
            .name("trash")
            .version("1.0")
            .author("Kove Salter <kove.w.o.salter@gmail.com>")
            .about("Safely manage your trash")
            .setting(AppSettings::ArgRequiredElseHelp)
            .arg(Arg::with_name("delete")
                .long("delete")
                .short("D")
                .help("Delete an item, storing it in the trash")
                .takes_value(true)
                .value_name("FILES")
                .max_values(max_argument_values)
                .conflicts_with_all(&[ "restore", "list", "pattern", "simple", "empty" ]))
            .arg(Arg::with_name("restore")
                .long("restore")
                .short("R")
                .help("Restore files from the trash")
                .takes_value(true)
                .value_name("FILES")
                .max_values(max_argument_values)
                .conflicts_with_all(&[ "delete", "list", "pattern", "simple", "empty" ]))
            .arg(Arg::with_name("list")
                .long("list")
                .short("L")
                .help("List items in the trash")
                .conflicts_with_all(&[ "delete", "restore", "empty" ]))
            .arg(Arg::with_name("pattern")
                .long("pattern")
                .short("p")
                .help("Set a pattern for --list")
                .takes_value(true)
                .value_name("PATTERN")
                .requires("list")
                .conflicts_with_all(&[ "delete", "restore", "empty" ]))
            .arg(Arg::with_name("simple")
                .long("simple")
                .short("s")
                .help("Use simple list format for --list")
                .requires("list")
                .conflicts_with_all(&[ "delete", "restore", "empty" ]))
            .arg(Arg::with_name("empty")
                .long("empty")
                .short("E")
                .help("Permenantly delete all trash items")
                .takes_value(false)
                .conflicts_with_all(&[ "delete", "restore", "list", "pattern", "simple" ]))
            .get_matches();

        if let Some(mut files) = matches.values_of("delete") {
            files.try_for_each(|file| self.delete(String::from(file)))?;
        } else if let Some(mut files) = matches.values_of("restore") {
            files.try_for_each(|file| self.restore(String::from(file)))?;
        } else if matches.is_present("list") {
            self.list(Regex::new(matches.value_of("pattern").unwrap_or(""))?, matches.is_present("simple"))?;
        } else if matches.is_present("empty") {
            self.empty()?;
        } else {
            Err(Error::InvalidArguments)?;
        }

        self.cache.end()?;

        Ok(())
    }

    pub fn delete(&mut self, target: String) -> Result<()> {
        if PathBuf::from(&target).exists() {
            let location: PathBuf = canonicalize(PathBuf::from(&target))?;
            let key: String = String::from(location.file_name().unwrap().to_str().unwrap());

            let mut destination: PathBuf = self.data_path.clone();
            let timestamp = format!("{:?}", Utc::now());
            
            destination.push(timestamp.clone());
            
            self.cache.push(key, String::from(location.to_str().unwrap()), timestamp);

            rename(location, destination)?;

            Ok(())
        } else {
            Err(Error::MissingTarget(target))
        }
    }

    pub fn restore(&mut self, target_key: String) -> Result<()> {
        let mut location: PathBuf;
        let mut destination: PathBuf;
        let entries = self.cache.pop(|key| key == &target_key, |_| true)?;

        for entry in entries {
            for version in entry.history() {
                location = self.data_path.clone();
                destination = PathBuf::from(format!("{}_{}", entry.origin(), version));

                location.push(version);

                if location.exists() {
                    rename(location.clone(), destination)?;
                } else {
                    Err(Error::MissingTarget(version.clone()))?;
                }
            }
        }

        Ok(())
    }

    pub fn list(&self, pattern: Regex, simple: bool) -> Result<()> {
        let mut empty: bool = true;
        let show_all: bool = pattern.as_str().is_empty();

        if !simple {
            if show_all {
                println!("Showing results in trash.");
            } else {
                println!("Showing results for '{}' in trash.", pattern.as_str());
            }
        }

        for entry in self.cache.entries().iter() {
            if pattern.is_match(entry.key()) {
                if simple {
                    println!("{}", entry.key());
                } else {
                    println!("  * {} <- {}", entry.key(), entry.origin());
                    
                    for version in entry.history().iter() {
                        println!("    * {}", version)
                    }

                    empty = false;
                }
            }
        }

        if !simple {
            if empty && show_all {
                println!("Your trash is empty!");
            } else if empty {
                println!("No results for '{}'.", pattern.as_str());
            }
        }

        Ok(())
    }

    pub fn empty(&mut self) -> Result<()> {
        let mut location;

        for entry in self.cache.pop(|_| { true }, |_| { true })? {
            location = PathBuf::new();
            location.push(&self.data_path);
            location.push(entry.key());

            if location.is_dir() {
                remove_dir_all(location)?;
            } else {
                remove_file(location)?;
            }
        }

        Ok(())
    }
}
