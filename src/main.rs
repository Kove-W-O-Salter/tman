#[macro_use(load_yaml)]
extern crate clap;
mod logger;
mod trash;

fn main() {
    trash::Trash::new().unwrap().main().unwrap()
}
