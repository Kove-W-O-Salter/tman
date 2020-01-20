mod lib;

use lib::{ Trash };
use lib::error::{ finish };

fn main() {
    match Trash::new() {
        Ok(mut trash) => finish(trash.main()),
        error => finish(error),
    }
}