use std::env::args;
use std::process;

use ncurses::addstr;

use crate::processor::Processor;

mod processor;

fn main() {
    env_logger::init();
    if let Some(path) = args().nth(1) {
        let mut p = Processor::new(path);
        process::exit(p.run())
    }
    addstr("Testing");
    println!("You must provide a ROM file.");
    process::exit(1);
}
