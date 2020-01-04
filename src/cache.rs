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
    key: String,
    origin: String,
    history: Vec<String>
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

    pub fn push(&mut self, key: String, origin: String, timestamp: String) {
        let mut done: bool = false;

        for entry in self.entries.iter_mut() {
            if entry.key() == &key {
                entry.push(timestamp.clone());
                done = true;
                break;
            }
        }

        if !done {
            self.entries.push(Entry::new(key, origin, vec![timestamp]))
        }
    }

    pub fn pop<KP, VP>(&mut self, key_predicate: KP, version_predicate: VP) -> Result<Vec<Entry>>
    where
        KP: Fn(&String) -> bool,
        VP: Fn(&String) -> bool
    {
        let mut popped: Vec<Entry> = vec![];
        let mut indices: Vec<usize> = vec![];
        let mut occurred: bool = false;

        for (index, entry) in self.entries.iter_mut().enumerate() {
            if key_predicate(entry.key()) {
                popped.push(Entry::new(entry.key().clone(), entry.origin().clone(), entry.pop(&version_predicate)));
                
                if entry.history().len() == 0 {
                    indices.push(index);
                }

                occurred = true;
            }
        }

        for index in indices.iter() {
            self.entries.remove(*index);
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
    pub fn new(key: String, origin: String, history: Vec<String>) -> Entry {
        Entry {
            key,
            origin,
            history
        }
    }

    pub fn push(&mut self, timestamp: String) {
        self.history.push(timestamp);
    }

    pub fn pop<P>(&mut self, predicate: P) -> Vec<String>
    where
        P: Fn(&String) -> bool
    {
        let mut indices: Vec<usize> = vec![];
        let mut popped: Vec<String> = vec![];
        let mut shift_factor: usize = 0;

        for (index, timestamp) in self.history.iter().enumerate() {
            if predicate(&timestamp) {
                indices.push(index);
            }
        }

        for index in indices.iter() {
            popped.push(self.history.remove(index - shift_factor));
            shift_factor += 1;
        }

        popped
    }

    pub fn key(&self) -> &String {
        &self.key
    }

    pub fn origin(&self) -> &String {
        &self.origin
    }

    pub fn history(&self) -> &Vec<String> {
        &self.history
    }
}
