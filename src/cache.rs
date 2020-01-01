use std::io::BufReader;
use std::fs::OpenOptions;
use std::path::PathBuf;
use serde::{
    Serialize,
    Deserialize
};
use serde_json::{
    from_reader,
    to_writer,
};
use super::error::{
    Result,
    Error,
};
use super::dictionary::Dictionary;

pub struct Cache {
    dictionary: Dictionary<String, Entry>,
    cache_path: PathBuf
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    name: String,
    id: String,
    origin: String
}

impl Cache {
    pub fn new(cache_path: PathBuf) -> Result<Cache> {
        let dictionary = {
            let cache_handle = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(cache_path.clone())?;

            from_reader(BufReader::new(cache_handle))
        }.unwrap_or(Dictionary::new());

        Ok(Cache {
            dictionary,
            cache_path
        })
    }

    pub fn commit(&self) -> Result<()> {
        let cache_handle = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.cache_path)?;

        to_writer(cache_handle, &self.dictionary)?;

        Ok(())
    }

    pub fn push(&mut self, name: String, id: String, origin: String) {
        self.dictionary.push(name.clone(), Entry::new(name, id, origin));
    }

    pub fn pop<F>(&mut self, predicate: F) -> Result<Vec<Entry>> where
        F: Fn(&String) -> bool {
        if !self.dictionary.contains_key(&predicate) {
            Err(Error::MissingTargetPredicate)?
        }

        Ok(self.dictionary.pop(predicate, |_| true))
    }

    pub fn values<F>(&self, predicate: F) -> Vec<Entry> where
        F: Fn(&String) -> bool {
        self.dictionary.values(predicate)
    }
}

impl Entry {
    pub fn new(name: String, id: String, origin: String) -> Entry {
        Entry {
            name,
            id,
            origin
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn origin(&self) -> &String {
        &self.origin
    }
}