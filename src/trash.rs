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
        let cli = load_yaml!("cli.yml");
        let matches = App::from_yaml(cli).settings(&[
            AppSettings::DisableHelpFlags,
            AppSettings::VersionlessSubcommands,
            AppSettings::SubcommandRequiredElseHelp,
        ]).get_matches();

        match matches.subcommand() {
            ("delete", Some(sub_matches)) =>
                sub_matches.values_of("FILE").unwrap().try_for_each(|file| self.delete(String::from(file))),
            ("restore", Some(sub_matches)) =>
                sub_matches.values_of("FILE").unwrap().try_for_each(|file| self.restore(String::from(file))),
            ("list", Some(sub_matches)) =>
                self.list(
                    Regex::new(sub_matches.value_of("PATTERN").unwrap_or("")).unwrap(),
                    sub_matches.is_present("simple")
                ),
            ("empty", Some(_)) => self.empty(),
            _ => Err(Error::InvalidArguments)
        }?;

        self.cache.commit()
    }

    pub fn delete(&mut self, target: String) -> Result<()> {
        let location = canonicalize(PathBuf::from(&target))?;
        let name = location.file_name().unwrap().to_str().unwrap();

        if location.exists() {
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