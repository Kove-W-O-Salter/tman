extern crate clap;
extern crate dirs;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate failure;
extern crate chrono;
extern crate console;

pub mod cache;
pub mod error;
pub mod settings;

use std::fs::{ rename, create_dir, canonicalize, remove_dir_all };
use std::path::{ PathBuf };
use dirs::{ home_dir };
use clap::{ App, AppSettings, ArgMatches, Arg };
use regex::{ Regex };
use console::{ Term, Style, StyledObject };
use uuid::{ Uuid };

use cache::{ Cache, VersionPredicate };
use error::{ Result, Error };
use settings::{ Settings };

///
/// The application and all of it's resources.
/// 
/// # Example
/// 
/// ```
/// let app: TMan = TMan::new()?;
/// app.main()?;
/// ```
///
pub struct TMan {
    /// The cache.
    cache: Cache,
    /// A console.
    stdout: Term,
    /// Settings.
    settings: Settings,
    /// Location of file store.
    data_path: PathBuf
}

impl TMan {
    ///
    /// Create a new application, loading it's settings and cache whilst
    /// creating all missing directories.
    /// 
    /// # Example
    /// 
    /// ```
    /// let app: TMan = TMan::new();
    /// ```
    /// 
    /// # Errors
    /// 
    /// Fails on failed initialisation of cache and on failed initialisation
    /// of settings.
    ///
    pub fn new() -> Result<TMan> {
        let mut directory: PathBuf = home_dir().unwrap_or_default();

        directory.push(".tman");

        let mut cache_path: PathBuf = directory.clone();
        let mut settings_path: PathBuf = directory.clone();
        let mut data_path: PathBuf = directory.clone();

        cache_path.push("cache.json");
        settings_path.push("settings.json");
        data_path.push("data");

        create_dir(&directory).unwrap_or_default();
        create_dir(&data_path).unwrap_or_default();

        Ok(TMan {
            cache: Cache::new(&cache_path)?,
            stdout: Term::stdout(),
            settings: Settings::new(&settings_path)?,
            data_path: data_path
        })
    }

    ///
    /// Run the application, parsing the command line arguments.
    /// 
    /// # Example
    /// 
    /// ```
    /// app.main()?;
    /// ```
    /// 
    pub fn main(&mut self) -> Result<()> {
        let max_argument_values: u64 = std::u64::MAX;

        let matches: ArgMatches<'static> = App::new("TMan")
            .name("tman")
            .version("1.0.0")
            .author("Kove Salter <kove.w.o.salter@gmail.com>")
            .about("Safely manage your trash")
            .setting(AppSettings::ArgRequiredElseHelp)
            .help(
r#"USAGE:
    tman <ACTION>

ACTIONS:
    --delete             -D    <FILE_1>...    Delete specified files
    --restore            -R    <FILE>         Restore specified file
        --origin         -o    <PATH>         Set the origin
        --version        -v                   Set the revision
            <VERSION>                         Use a specific version
            latest                            Use the newest version (default)
            all                               Use all versions
    --list               -L                   List items in the trash
        --pattern        -p    <REGEX>        Set the search pattern
        --simple         -p                   Set the simple mode
    --empty              -E                   Permenantly delete trash content"#
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

    ///
    /// Move a target file to the trash.
    /// 
    /// # Example
    /// 
    /// ```
    /// app.delete(String::from("./Bilbo.txt"))?;
    /// ```
    /// 
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

    ///
    /// Restore a target files version to it's original location.
    /// 
    /// # Example
    /// 
    /// ```
    /// app.restore(String::from("Bilbo.txt"), None, None);
    /// ```
    ///
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
                Some("latest") | None => VersionPredicate::Latest,
                Some(target_version) => VersionPredicate::Specific(&target_version)
            }
        )?;

        for (empty, entry) in entries {
            for version in entry.history() {
                location = self.data_path.clone();
                // Ensure unique names by appending the verssion timestamp to
                // the destination file name, when more than one versions are
                // being restored.
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

            // Remove the directory if all entries are restored.
            if empty {
                location.pop();
                remove_dir_all(&location)?;
            }
        }

        Ok(())
    }

    ///
    /// List the contents of the trash.
    /// 
    /// # Example
    /// 
    /// ```
    /// app.list(Regex::from_str("")?, false)?;
    /// ```
    ///
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

    ///
    /// Delete everything in the trash.
    /// 
    /// # Example
    /// 
    /// ```
    /// app.empty()?;
    /// ```
    ///
    pub fn empty(&mut self) -> Result<()> {
        let mut location: PathBuf;

        for (_, entry) in self.cache.pop(|_| { true }, VersionPredicate::All)? {
            location = PathBuf::from(&self.data_path);
            location.push(entry.uuid().to_string());

            remove_dir_all(&location)?;
        }

        Ok(())
    }

    ///
    /// Insert a unicode character if `use_unicode` is enabled, else use a
    /// default ASCII character.
    /// 
    /// # Example
    /// 
    /// ```
    /// let output: &'static str = app.unicode("\u{2022}", "*");
    /// ```
    ///
    pub fn unicode<'a>(&self, unicode: &'a str, ascii: &'a str) -> &'a str {
        if self.settings.use_unicode() {
            unicode
        } else {
            ascii
        }
    }

    ///
    /// Format text with ANSI styles if the `use_colors` setting is enabled.
    /// 
    /// # Example
    /// 
    /// ```
    /// let output: StyledObject<&'static str> = app.color("Bold Face", Style::new().bold());
    /// ```
    ///
    pub fn color<'a>(&self, text: &'a str, color: &Style) -> StyledObject<&'a str> {
        if self.settings.use_colors() {
            color.apply_to(text)
        } else {
            Style::new().apply_to(text)
        }
    }
}
