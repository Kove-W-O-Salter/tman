use std::io::{
    // Result,
    // Error,
    // ErrorKind,
    BufReader,
};
use std::fs::{
    OpenOptions,
};
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

pub struct Cache {
    pub items: Vec<Item>,
    pub cache_file: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    pub name: String,
    pub origin: String,
}

impl Cache {
    pub fn new(cache_file: &PathBuf) -> Result<Cache> {
        let items = {
            let cache_handle = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(cache_file)?;

            from_reader(BufReader::new(cache_handle))
        }?;

        Ok(Cache {
            items: items,
            cache_file: cache_file.clone(),
        })
    }

    pub fn commit(&self) -> Result<()> {
        let cache_handle = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.cache_file)?;

        to_writer(cache_handle, &self.items)?;

        Ok(())
    }

    pub fn conflicts(&self, expect: bool, name: String) -> Result<()> {
        if self.items.iter().any(|item| item.name == name) == expect {
            Ok(())
        } else {
            Err(Error::MissingTarget(name.clone()))
        }
    }

    pub fn add_item(&mut self, name: String, origin: String) -> Result<()> {
        self.conflicts(false, name.clone())?;
        self.items.push(Item::new(name, origin));
        Ok(())
    }

    pub fn remove_item(&mut self, name: String) -> Result<String> {
        self.conflicts(true, name.clone())?;
        match self.items.iter().position(|item| item.name == name) {
            Some(index) => {
                let item = self.items[index].clone();
                self.items.remove(index);
                Ok(item.origin)
            },
            None => Err(Error::MissingTarget(name.clone())),
        }
    }
}

impl Item {
    pub fn new(name: String, origin: String) -> Item {
        Item {
            name: name,
            origin: origin,
        }
    }
}