use std::io::{ BufReader, BufWriter, Seek, SeekFrom };
use std::fs::{ OpenOptions, File };
use std::path::{ PathBuf };
use serde::{ Serialize, Deserialize };
use serde_json::{ from_reader, to_writer };
use chrono::{ Utc };
use uuid::{ Uuid };
use super::error::{ Result, Error };

pub struct Cache {
    entries: Vec<Entry>,
    file: File
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    key: Key,
    uuid: Uuid,
    history: Vec<String>
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Key {
    name: String,
    origin: String
}

pub enum VersionPredicate<'a> {
    Any,
    Newest,
    Specific(&'a str)
}

impl Cache {
    pub fn new(path: &PathBuf) -> Result<Cache> {
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(PathBuf::from(path))?;
        let entries: Vec<Entry> = from_reader(BufReader::new(&file)).unwrap_or(vec![]);

        Ok(Cache {
            entries,
            file
        })
    }

    pub fn push(&mut self, name: String, origin: String) -> (Uuid, String) {
        let mut done: bool = false;
        let key: Key = Key::new(name, origin.clone());
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
        
        if !done {
            uuid = Some(Uuid::new_v4());
            self.entries.push(Entry::new(key, uuid.clone().unwrap(), vec![version.clone()]));
        }

        (uuid.unwrap(), version)
    }

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

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn end(&mut self) -> Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;

        to_writer(BufWriter::new(&self.file), &self.entries)?;

        Ok(())
    }
}

impl Entry {
    pub fn new(key: Key, uuid: Uuid, history: Vec<String>) -> Entry {
        Entry {
            key,
            uuid,
            history
        }
    }

    pub fn push(&mut self, version: String) {
        self.history.push(version);
    }

    pub fn pop(&mut self, predicate: &VersionPredicate) -> Vec<String> {
        let mut indices: Vec<usize> = vec![];
        let mut popped: Vec<String> = vec![];
        let mut shift_factor: usize = 0;

        match predicate {
            VersionPredicate::Any => {
                popped = self.history.clone();
                self.history.truncate(0);
            },
            VersionPredicate::Newest => popped.push(self.history.pop().unwrap()),
            VersionPredicate::Specific(target_version) => {
                for (index, version) in self.history.iter().enumerate() {
                    if &version == target_version {
                        indices.push(index);
                    }
                }

                for index in indices.iter() {
                    popped.push(self.history.remove(index - shift_factor));
                    shift_factor += 1;
                }
            }
        }

        popped
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn history(&self) -> &Vec<String> {
        &self.history
    }
}

impl Key {
    pub fn new(name: String, origin: String) -> Key {
        Key {
            name,
            origin
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn origin(&self) -> &String {
        &self.origin
    }
}