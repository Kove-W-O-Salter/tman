#[macro_use(load_yaml)]
extern crate clap;
mod trash;

use std::io::Result;
use trash::Trash;

fn main() -> Result<()> {
    return Trash::new().main();
}
