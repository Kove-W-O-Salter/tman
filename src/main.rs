#[macro_use(load_yaml)]
extern crate clap;
extern crate dirs;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate failure;
mod trash;
mod cache;
mod error;

fn main() {
    match trash::Trash::new() {
        Ok(mut trash) => error::finish(trash.main()),
        error => error::finish(error),
    }
}