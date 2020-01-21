use std::io::{ BufReader, BufWriter, Seek, SeekFrom };
use std::fs::{ OpenOptions, File };
use std::path::{ PathBuf };
use serde::{ Serialize, Deserialize };
use serde_json::{ from_reader, to_writer };
use chrono::{ Utc };
use uuid::{ Uuid };

use super::error::{ Result, Error };

///
/// A map of key value pairs representing version controlled files in the trash,
/// which stored as a JSON file.
/// Each item is distinguished by it's key -- a combination of it's filename and
/// origin.
/// Each item's value represents a series of it's versions, stored in reverse
/// chronology (oldest to newest).
/// 
/// # Example
/// 
/// ```
/// let file: PathBuf = PathBuf::from("./cache.json");
/// let cache: Cache = Cache::new(&file)?;
/// ```
/// 
pub struct Cache {
    /// The entries.
    entries: Vec<Entry>,
    /// The physical file.
    file: File
}

///
/// A unique cache with a physical representation in a file with the name uuid.
/// 
/// # Example
/// 
/// ```
/// let name: String = "Bilbo.txt".to_string();
/// let origin: String = "/home/Bilbo/Bilbo.txt".to_string();
/// let entry: Entry = Entry::new(
///     Key::new(name, origin),
///     Uuid::new_v4(),
///     vec![]
/// );
/// ```
///
#[derive(Serialize, Deserialize)]
pub struct Entry {
    /// The unique key.
    key: Key,
    /// The UUID representing the entry's physical directory.
    uuid: Uuid,
    /// The versions of the entry.
    history: Vec<String>
}

///
/// A unique key representing a unique file in the trash with a combination of
/// it's filename and original location.
/// 
/// # Example
/// 
/// ```
/// let name: String = "Bilbo.txt".to_string();
/// let origin: String = "/home/Bilbo/Bilbo.txt".to_string();
/// let key: Key = Key::new(name, origin);
/// ```
///
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Key {
    /// The filename.
    name: String,
    /// The original location.
    origin: String
}

///
/// A search predicate for versions.
/// 
/// # Example
/// 
/// ```
/// let predicate: VersionPredicate = VersionPredicate::Latest;
/// ```
///
pub enum VersionPredicate<'a> {
    /// Match all versions.
    All,
    /// Match the latest version.
    Latest,
    /// Match a specific version.
    Specific(&'a str)
}

impl Cache {
    ///
    /// Create a new `Cache` object that stores it's data in `path`.
    /// 
    /// # Example
    /// 
    /// ```
    /// let path: PathBuf = PathBuf::from("./cache.json");
    /// let cache: Cache = Cache::new(&path)?;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Fails if there is an error parsing the JSON file.
    ///
    pub fn new(path: &PathBuf) -> Result<Cache> {
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let entries: Vec<Entry> = from_reader(BufReader::new(&file)).unwrap_or(vec![]);

        Ok(Cache {
            entries,
            file
        })
    }

    ///
    /// Push a new version of an item onto the cache.
    /// The item will be created, if it does not already exist.
    /// A UUID, representing the directory name of the item, and a timestamp,
    /// representing the version of the item, are returned.
    /// 
    /// # Example
    /// 
    /// ```
    /// let name: String = "Bilbo.txt".to_string();
    /// let origin: String = "/home/Bilbo/Bilbo.txt".to_string();
    /// let (uuid, version): (Uuid, String) = cache.push(name, origin);
    /// ```
    ///
    pub fn push(&mut self, name: String, origin: String) -> (Uuid, String) {
        let mut done: bool = false;
        let key: Key = Key::new(name, origin.clone());
        //
        // Here the `uuid` must be optional since have an initial uuid does not
        // make sense.
        //
        let mut uuid: Option<Uuid> = None;
        let version: String = format!("{}", Utc::now());
        
        for entry in self.entries.iter_mut() {
            if entry.key() == &key {
                entry.push(version.clone());
                uuid = Some(entry.uuid().clone());
                done = true;
                break;
            }
        }
        
        //
        // Create the item if no versions were pushed -- the item does not exist.
        //
        if !done {
            uuid = Some(Uuid::new_v4());
            self.entries.push(Entry::new(key, uuid.clone().unwrap(), vec![version.clone()]));
        }

        (uuid.unwrap(), version)
    }

    ///
    /// Remove items or versions of items from the cache using predicates.
    /// `key_predicate` determines which items will be operated on.
    /// `version_predicate` determines which versions of said items will be operated on.
    /// 
    /// # Example
    /// 
    /// ```
    /// let entries: Vec<(bool, Entry)> = cache.pop(|_| true, VersionPredicate::All)?;
    /// ```
    ///
    /// # Errors
    /// 
    /// Fails when no entries satisfy the `key_predicate`.
    ///
    pub fn pop<KP>(&mut self, key_predicate: KP, version_predicate: VersionPredicate) -> Result<Vec<(bool, Entry)>>
    where
        KP: Fn(&Key) -> bool
    {
        let mut popped: Vec<(bool, Entry)> = vec![];
        let mut indices: Vec<usize> = vec![];
        let mut shift_factor: usize = 0;
        let mut occurred: bool = false;
        #[allow(unused_assignments)]
        let mut empty: bool = false;
        let mut victim_entry: Entry;

        //
        // Pop required versions from the required entries, marking which
        // entries are now empty.
        //
        for (index, entry) in self.entries.iter_mut().enumerate() {
            if key_predicate(entry.key()) {
                victim_entry = Entry::new(entry.key().clone(), entry.uuid().clone(), entry.pop(&version_predicate));
                empty = entry.history().len() == 0;

                popped.push((empty, victim_entry));
                
                if empty {
                    indices.push(index);
                }

                occurred = true;
            }
        }

        //
        // Remove all emptied entries from the cache.
        // `shift_factor` is required to account for the vector shrinking.
        //
        for index in indices.iter() {
            self.entries.remove(*index - shift_factor);
            shift_factor += 1;
        }

        if occurred {
            Ok(popped)
        } else {
            Err(Error::MissingTargetPredicate)
        }
    }

    ///
    /// Gain a reference to the entries.
    /// 
    /// # Example
    /// 
    /// ```
    /// let entries: &Vec<Entry> = cache.entries();
    /// ```
    ///
    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    ///
    /// Commit changes to the cache.
    /// 
    /// # Example
    /// 
    /// ```
    /// cache.end()?;
    /// ```
    ///
    pub fn end(&mut self) -> Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;

        to_writer(BufWriter::new(&self.file), &self.entries)?;

        Ok(())
    }
}

impl Entry {
    ///
    /// Create a new entry.
    /// 
    /// # Example
    /// 
    /// ```
    /// let name: String = "Bilbo.txt".to_striong();
    /// let origin: String = "/home/Bilbo/Bilbo.txt".to_string();
    /// let entry: Entry = Entry::new(
    ///     Key::new(name, origin),
    ///     Uuid::new_v4(),
    ///     vec![]
    /// );
    /// ```
    ///
    pub fn new(key: Key, uuid: Uuid, history: Vec<String>) -> Entry {
        Entry {
            key,
            uuid,
            history
        }
    }

    ///
    /// Push a new version into an entry.
    /// 
    /// # Example
    /// 
    /// ```
    /// entry.push(format!("{}", Utc::now()));
    /// ```
    ///
    pub fn push(&mut self, version: String) {
        self.history.push(version);
    }

    ///
    /// Remove all versions that satisfy `predicate` from the history.
    ///
    pub fn pop(&mut self, predicate: &VersionPredicate) -> Vec<String> {
        let mut popped: Vec<String> = vec![];

        match predicate {
            VersionPredicate::All => {
                popped = self.history.clone();
                self.history.truncate(0);
            },
            VersionPredicate::Latest => {
                popped.push(self.history.pop().unwrap());
            },
            VersionPredicate::Specific(target_version) => {
                for (index, version) in self.history.iter().enumerate() {
                    if &version == target_version {
                        popped.push(self.history.remove(index));
                        break;
                    }
                }
            }
        }

        popped
    }

    ///
    /// Get a reference to the entry's key.
    /// 
    /// # Example
    /// 
    /// ```
    /// let key: &Key = entry.key();
    /// ```
    ///
    pub fn key(&self) -> &Key {
        &self.key
    }

    ///
    /// Get a reference to the entry's UUID.
    /// 
    /// # Example
    /// 
    /// ```
    /// let key: &Uuid = entry.uuid();
    /// ```
    ///
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    ///
    /// Get a reference to the entry's history.
    /// 
    /// # Example
    /// 
    /// ```
    /// let key: &Vec<String> = entry.history();
    /// ```
    ///
    pub fn history(&self) -> &Vec<String> {
        &self.history
    }
}

impl Key {
    ///
    /// Create a new key.
    /// 
    /// # Example
    /// 
    /// ```
    /// let name: String = "Bilbo.txt".to_string();
    /// let origin: String = "/home/Bilbo/Bilbo.txt".to_string();
    /// let key: Key = Key::new(name, origin);
    /// ```
    /// 
    pub fn new(name: String, origin: String) -> Key {
        Key {
            name,
            origin
        }
    }

    ///
    /// Get a reference to the key's name.
    /// 
    /// # Example
    /// 
    /// ```
    /// let name: &String = key.name();
    /// ```
    ///
    pub fn name(&self) -> &String {
        &self.name
    }

    ///
    /// Get a reference to the key's original location.
    /// 
    /// # Example
    /// 
    /// ```
    /// let origin: &String = key.origin();
    /// ```
    ///
    pub fn origin(&self) -> &String {
        &self.origin
    }
}