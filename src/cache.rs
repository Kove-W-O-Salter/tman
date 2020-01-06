use std::io::{ BufReader, BufWriter, Seek, SeekFrom };
use std::fs::{ OpenOptions, File };
use std::path::{ PathBuf };
use serde::{ Serialize, Deserialize };
use serde_json::{ from_reader, to_writer };
use super::error::{ Result, Error };

pub struct Cache {
    entries: Vec<Entry>,
    file: File
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    key: Key,
    history: Vec<String>
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Key {
    name: String,
    origin: String
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

    pub fn push(&mut self, name: String, origin: String, version: String) {
        let mut done: bool = false;
        let key: Key = Key::new(name, origin);

        for entry in self.entries.iter_mut() {
            if entry.key() == &key {
                entry.push(version.clone());
                done = true;
                break;
            }
        }

        if !done {
            self.entries.push(Entry::new(key, vec![version]))
        }
    }

    pub fn pop<KP, VP>(&mut self, key_predicate: KP, version_predicate: VP) -> Result<Vec<Entry>>
    where
        KP: Fn(&Key) -> bool,
        VP: Fn(&String) -> bool
    {
        let mut popped: Vec<Entry> = vec![];
        let mut indices: Vec<usize> = vec![];
        let mut shift_factor: usize = 0;
        let mut occurred: bool = false;

        for (index, entry) in self.entries.iter_mut().enumerate() {
            if key_predicate(entry.key()) {
                popped.push(Entry::new(entry.key().clone(), entry.pop(&version_predicate)));
                
                if entry.history().len() == 0 {
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
    pub fn new(key: Key, history: Vec<String>) -> Entry {
        Entry {
            key,
            history
        }
    }

    pub fn push(&mut self, version: String) {
        self.history.push(version);
    }

    pub fn pop<P>(&mut self, predicate: P) -> Vec<String>
    where
        P: Fn(&String) -> bool
    {
        let mut indices: Vec<usize> = vec![];
        let mut popped: Vec<String> = vec![];
        let mut shift_factor: usize = 0;

        for (index, version) in self.history.iter().enumerate() {
            if predicate(&version) {
                indices.push(index);
            }
        }

        for index in indices.iter() {
            popped.push(self.history.remove(index - shift_factor));
            shift_factor += 1;
        }

        popped
    }

    pub fn key(&self) -> &Key {
        &self.key
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