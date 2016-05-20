extern crate byteorder;
#[macro_use] extern crate log;
extern crate env_logger;

mod processor;

use std::process;
use std::env::args;

use processor::Processor;

fn main() {
    env_logger::init().unwrap();
    if let Some(path) = args().nth(1) {
        let mut p = Processor::new(path);
        process::exit(p.run())
    }
    println!("You must provide a ROM file.");
    process::exit(1);
}
