mod lib;

use lib::{ TMan };
use lib::error::{ finish };

fn main() {
    match TMan::new() {
        Ok(mut tman) => finish(tman.main()),
        error => finish(error),
    }
}