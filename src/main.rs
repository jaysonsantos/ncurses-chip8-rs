extern crate byteorder;
extern crate env_logger;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate ncurses;

mod processor;

use std::process;
use std::env::args;

use ncurses::{printw};

use processor::Processor;

fn main() {
    env_logger::init().unwrap();
    if let Some(path) = args().nth(1) {
        let mut p = Processor::new(path);
        process::exit(p.run())
    }
    printw("Testing");
    println!("You must provide a ROM file.");
    process::exit(1);
}
