use std::fs::{ rename, create_dir, canonicalize, remove_dir_all };
use std::path::{ PathBuf };
use dirs::{ home_dir };
use clap::{ App, AppSettings, ArgMatches, Arg };
use regex::{ Regex };
use console::{ Term, Style, StyledObject };
use uuid::{ Uuid };
use super::cache::{ Cache, VersionPredicate };
use super::error::{ Result, Error };
use super::settings::{ Settings };

pub struct Trash {
    cache: Cache,
    stdout: Term,
    settings: Settings,
    data_path: PathBuf
}

impl Trash {
    pub fn new() -> Result<Trash> {
        let mut directory: PathBuf = home_dir().unwrap_or_default();

        directory.push(".trash");

        let mut cache_path: PathBuf = directory.clone();
        let mut settings_path: PathBuf = directory.clone();
        let mut data_path: PathBuf = directory.clone();

        cache_path.push("cache.json");
        settings_path.push("settings.json");
        data_path.push("data");

        create_dir(&directory).unwrap_or_default();
        create_dir(&data_path).unwrap_or_default();

        Ok(Trash {
            cache: Cache::new(&cache_path)?,
            stdout: Term::stdout(),
            settings: Settings::new(&settings_path)?,
            data_path: data_path
        })
    }

    pub fn main(&mut self) -> Result<()> {
        let max_argument_values: u64 = std::u64::MAX;

        let matches: ArgMatches<'static> = App::new("Trash")
            .name("trash")
            .version("1.0")
            .author("Kove Salter <kove.w.o.salter@gmail.com>")
            .about("Safely manage your trash")
            .setting(AppSettings::ArgRequiredElseHelp)
            .help(
r#"USAGE:
    trash <ACTION>

ACTIONS:
    --delete         -D    <FILE_1>...    Trash specified files
    --restore        -R    <FILE>         Restore specified file
        --origin     -o    <PATH>         Set the origin
        --version    -v    <VERSION>      Set the revision
    --list           -L                   List items in the trash
        --pattern    -p    <REGEX>        Set the search pattern
        --simple     -p                   Set the simple mode
    --empty          -E                   Permenantly delete trash content"#
            )
            .arg(Arg::with_name("delete")
                .long("delete")
                .short("D")
                .help("Delete an item, storing it in the trash")
                .takes_value(true)
                .value_name("FILES")
                .max_values(max_argument_values)
                .conflicts_with_all(&[ "restore", "origin", "version", "list", "pattern", "simple", "empty" ]))
            .arg(Arg::with_name("restore")
                .long("restore")
                .short("R")
                .help("Restore files from the trash")
                .takes_value(true)
                .value_name("FILES")
                .max_values(max_argument_values)
                .conflicts_with_all(&[ "delete", "list", "pattern", "simple", "empty" ]))
            .arg(Arg::with_name("origin")
                .long("origin")
                .short("o")
                .help("Set the origin for restore")
                .takes_value(true)
                .value_name("PATH")
                .requires("restore")
                .conflicts_with_all(&[ "delete", "list", "pattern", "simple", "empty" ]))
            .arg(Arg::with_name("version")
                .long("version")
                .short("v")
                .help("Set the version for restore")
                .takes_value(true)
                .value_name("VERSION")
                .requires("restore")
                .conflicts_with_all(&[ "delete", "list", "pattern", "simple", "empty" ]))
            .arg(Arg::with_name("list")
                .long("list")
                .short("L")
                .help("List items in the trash")
                .conflicts_with_all(&[ "delete", "restore", "origin", "version", "empty" ]))
            .arg(Arg::with_name("pattern")
                .long("pattern")
                .short("p")
                .help("Set a pattern for --list")
                .takes_value(true)
                .value_name("PATTERN")
                .requires("list")
                .conflicts_with_all(&[ "delete", "restore", "origin", "version", "empty" ]))
            .arg(Arg::with_name("simple")
                .long("simple")
                .short("s")
                .help("Use simple list format for --list")
                .requires("list")
                .conflicts_with_all(&[ "delete", "restore", "origin", "version", "empty" ]))
            .arg(Arg::with_name("empty")
                .long("empty")
                .short("E")
                .help("Permenantly delete all trash items")
                .takes_value(false)
                .conflicts_with_all(&[ "delete", "restore", "origin", "version", "list", "pattern", "simple" ]))
            .get_matches();

        if let Some(mut files) = matches.values_of("delete") {
            files.try_for_each(|file| self.delete(String::from(file)))?;
        } else if let Some(file) = matches.value_of("restore") {
            self.restore(file, matches.value_of("origin"), matches.value_of("version"))?;
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
        let origin: PathBuf = canonicalize(&target)?;
        let name: String = origin.file_name().unwrap().to_str().unwrap().to_string();
        let mut destination: PathBuf = self.data_path.clone();

        let (uuid, version): (Uuid, String) = self.cache.push(name, origin.to_str().unwrap().to_string());

        destination.push(uuid.to_string());

        create_dir(&destination).unwrap_or_default();

        destination.push(&version);

        rename(origin, destination)?;

        Ok(())
    }

    pub fn restore(&mut self, target_name: &str, target_origin: Option<&str>, target_version: Option<&str>) -> Result<()> {
        let mut location: PathBuf = PathBuf::default();
        #[allow(unused_assignments)]
        let mut destination: PathBuf = PathBuf::default();
        let entries = self.cache.pop(
            |key| {
                if let Some(target_origin) = target_origin {
                    key.name() == &target_name && key.origin() == &target_origin
                } else {
                    key.name() == &target_name
                }
            },
            match target_version {
                Some("all") => VersionPredicate::All,
                Some("newest") => VersionPredicate::Newest,
                Some(target_version) => VersionPredicate::Specific(&target_version),
                None => VersionPredicate::Newest
            }
        )?;

        for (empty, entry) in entries {
            for version in entry.history() {
                location = self.data_path.clone();
                destination = if entry.history().len() > 1 {
                    PathBuf::from(format!("{}_{}", entry.key().origin(), version))
                } else {
                    PathBuf::from(entry.key().origin())
                };

                location.push(entry.uuid().to_string());
                location.push(version);

                if location.exists() {
                    rename(location.clone(), destination)?;
                } else {
                    Err(Error::MissingTarget(version.clone()))?;
                }
            }

            if empty {
                location.pop();
                remove_dir_all(&location)?;
            }
        }

        Ok(())
    }

    pub fn list(&self, pattern: Regex, simple: bool) -> Result<()> {
        let mut empty: bool = true;
        let show_all: bool = pattern.as_str().is_empty();
        let name_style = Style::new().bold();
        let origin_style = Style::new().dim().italic();
        let version_style = Style::new();

        if !simple {
            if show_all {
                self.stdout.write_line("Showing results in trash.")?;
            } else {
                self.stdout.write_line(format!("Showing results for '{}' in trash.", pattern.as_str()).as_str())?;
            }
        }

        for entry in self.cache.entries().iter() {
            if pattern.is_match(entry.key().name()) {
                if simple {
                    self.stdout.write_line(format!("{}", entry.key().name()).as_str())?;
                } else {
                    self.stdout.write_line(format!("  {} {} {} {}", self.unicode("\u{2022}", "*"), self.color(entry.key().name(), &name_style), self.unicode("\u{2190}", "<-"), self.color(entry.key().origin(), &origin_style)).as_str())?;
                    for version in entry.history().iter().rev() {
                        self.stdout.write_line(format!("    {} {}", self.unicode("\u{2192}", "->"), self.color(version, &version_style)).as_str())?;
                    }

                    empty = false;
                }
            }
        }

        if !simple {
            if empty && show_all {
                self.stdout.write_line("Your trash is empty!")?;
            } else if empty {
                self.stdout.write_line(format!("No results for '{}'.", pattern.as_str()).as_str())?;
            }
        }

        Ok(())
    }

    pub fn empty(&mut self) -> Result<()> {
        let mut location: PathBuf;

        for (_, entry) in self.cache.pop(|_| { true }, VersionPredicate::All)? {
            location = PathBuf::from(&self.data_path);
            location.push(entry.uuid().to_string());

            remove_dir_all(&location)?;
        }

        Ok(())
    }

    pub fn unicode<'a>(&self, unicode: &'a str, ascii: &'a str) -> &'a str {
        if self.settings.use_unicode() {
            unicode
        } else {
            ascii
        }
    }

    pub fn color<'a>(&self, text: &'a str, color: &Style) -> StyledObject<&'a str> {
        if self.settings.use_colors() {
            color.apply_to(text)
        } else {
            Style::new().apply_to(text)
        }
    }
}
