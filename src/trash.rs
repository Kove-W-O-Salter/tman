use std::fs::{
    rename,
    create_dir,
    remove_file,
    remove_dir_all,
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

        let cache = Cache::new(&cache_path)?;

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
        let location = PathBuf::from(&target);
        let name = location.file_name().unwrap().to_str().unwrap();

        if location.exists() {
            let mut destination = self.data_path.clone();

            destination.push(name);

            self.cache.add_item(String::from(name), target.clone())?;

            rename(location, destination)?;

            Ok(())
        } else {
            Err(Error::MissingTarget(target.clone()))
        }
    }

    pub fn restore(&mut self, name: String) -> Result<()> {
        let mut location = self.data_path.clone();

        location.push(&name);

        if location.exists() {
            let origin = self.cache.remove_item(name.clone())?;

            rename(location, PathBuf::from(origin))?;

            Ok(())
        } else {
            Err(Error::MissingTarget(name.clone()))
        }
    }

    pub fn list(&self, pattern: Regex, simple: bool) -> Result<()> {
        let mut items: Vec<String> = self.cache.items.clone().iter().map(|item| item.name.clone()).collect();
        
        items.sort();

        if !items.is_empty() {
            for item in items {
                if pattern.is_match(item.as_str()) {
                    if simple {
                        println!("{}", item);
                    } else {
                        println!("🢒 {}", item);
                    }
                }
            }
        } else {
            println!("Your trash is empty.");
        }

        Ok(())
    }

    pub fn empty(&mut self) -> Result<()> {
        let mut location;
        let mut name;

        for item in self.cache.items.clone() {
            name = item.name;
            location = PathBuf::new();
            location.push(&self.data_path);
            location.push(&name);

            if location.is_dir() {
                remove_dir_all(location)?;
            } else {
                remove_file(location)?;
            }

            self.cache.remove_item(name.clone())?;
        }

        Ok(())
    }
}