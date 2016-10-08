use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;

use byteorder::{BigEndian, ReadBytesExt};
use log::LogLevel::{Debug, Trace};

lazy_static! {
    pub static ref SLEEP_TIME: Duration = Duration::from_millis(1000 / 1);
}

pub struct Processor {
    registers: [u8; 16],
    i: u16,
    delay: u8,
    sound: u8,
    pc: u16,
    sp: u8,
    rom: Vec<u8>,
}

impl Processor {
    pub fn new<F>(file: F) -> Processor
        where F: AsRef<Path>
    {
        let mut f = File::open(file).expect("Error openning file");
        let mut buf = vec![];
        f.read_to_end(&mut buf).unwrap();
        debug!("File size {} bytes", buf.len());
        Processor {
            registers: [0; 16],
            i: 0,
            delay: 0,
            sound: 0,
            pc: 200,
            sp: 0,
            rom: buf,
        }
    }

    pub fn run(&mut self) -> i32 {
        let mut cursor = Cursor::new(&self.rom);
        loop {
            if log_enabled!(Trace) {
                let memory: Vec<String> = self.registers.iter().map(|e| format!("{:x}", e)).collect();
                trace!("Memory {:?}", memory);
            }

            cursor.set_position(self.pc as u64);
            if log_enabled!(Debug) {
                let mut rom = self.rom.clone();
                let rest = rom.split_off(self.pc as usize);
                let hex_rom: Vec<String> = rest.iter().map(|e| format!("{:x}", e)).collect();
                debug!("Rest of the rom {:?} rom size {} {:x}",
                       hex_rom,
                       self.rom.len(),
                       self.rom.len());
            }
            debug!("Cursor in {:x}", self.pc);
            let data = cursor.read_u8().unwrap();
            debug!("Data {:x}", data);
            let instruction = data >> 4;
            debug!("Instruction {:x}", instruction);
            match instruction {
                0x1 => {
                    // TODO: Fix this
                    cursor.set_position((self.pc) as u64);
                    let addr = cursor.read_u16::<BigEndian>().unwrap();
                    debug!("Jumpping to {:x} on {:x}", (addr & 0xFFF) - 0x200, addr);
                    self.pc = addr & 0xFFF - 0x200;
                }
                0x4 => {
                    if (data & 0xF) != cursor.read_u8().unwrap() {
                        self.pc += 4
                    } else {
                        self.pc += 2
                    }
                }
                0x5 => {
                    let x = data & 0xF;
                    let y = cursor.read_u8().unwrap() >> 4;
                    if self.registers[x as usize] == self.registers[y as usize] {
                        self.pc += 4
                    } else {
                        self.pc += 2
                    }
                }
                0x2 => {
                    // Call a function and increase stack.
                    cursor.set_position((self.pc - 1) as u64);
                    self.pc = cursor.read_u16::<BigEndian>().unwrap() & 0xFFF;
                    self.sp += 1;
                }
                0x6 => {
                    // Set value on register
                    let location = (data & 0xF) as usize;
                    let value = cursor.read_u8().unwrap();
                    debug!("Setting {:x} on register {:x}", value, location);
                    self.registers[location] = value;
                    self.pc += 2;
                }
                0x7 => {
                    // Increment value on register
                    // TODO: Overflow value
                    let location = (data & 0xF) as usize;
                    let value = cursor.read_u8().unwrap();
                    self.registers[location] += value;
                    self.pc += 2;
                }
                0x8 => {
                    let subcommand = cursor.read_u8().unwrap() & 0xF;
                    match subcommand {
                        0 => self.registers[(data & 0xF) as usize] = subcommand >> 4,
                        1 => {
                            self.registers[(data & 0xF) as usize] = (data & 0xF) | (subcommand >> 4)
                        }
                        2 => {
                            self.registers[(data & 0xF) as usize] = (data & 0xF) & (subcommand >> 4)
                        }
                        3 => {
                            self.registers[(data & 0xF) as usize] = (data & 0xF) ^ (subcommand >> 4)
                        }
                        4 => {
                            // TODO: Check for overflow
                            self.registers[(data & 0xF) as usize] = (data & 0xF) + (subcommand >> 4)
                        }
                        _ => panic!("Subcommand {:x} not found in command 0x8", subcommand),
                    };
                    self.pc += 2
                }
                0xA => {
                    // Set I value
                    cursor.set_position((self.pc - 1) as u64);
                    let value = cursor.read_u16::<BigEndian>().unwrap() & 0xFFF;
                    debug!("Setting I to {:x}", value);
                    self.i = value;
                    self.pc += 2;
                }
                0xD => {
                    // Display sprite on screen
                    self.pc += 2;
                }
                0xE => {
                    let value = data & 0xF;
                    let subcommand = cursor.read_u8().unwrap();
                    match subcommand {
                        0x9E => {
                            // If a key with value is pressed skip next
                            // instruction pc += 4
                            if false {
                                self.pc += 2
                            } else {
                                self.pc += 4
                            }
                        }
                        0xA1 => {
                            // If a key with value is not pressed skip next
                            // instruction pc += 4
                            if false {
                                self.pc += 2
                            } else {
                                self.pc += 4
                            }
                        }
                        unknown_code => {
                            panic!("Unknown subcode {:x} not found on code {:x}",
                                   0xE,
                                   unknown_code);
                        }
                    }
                }
                0xF => {
                    let subcommand = cursor.read_u8().unwrap();
                    debug!("Subcommand {:x}", subcommand);
                    match subcommand {
                        0x07 => {
                            self.registers[(instruction & 0xF) as usize] = self.delay;
                            self.pc += 2;
                        }
                        0x0A => {
                            // TODO: Wait for key
                            // self.registers[(instruction & 0xF) as usize] = key

                        }
                        0x15 => {
                            self.delay = self.registers[(instruction & 0xF) as usize];
                            self.pc += 2;

                        }
                        0x18 => {
                            self.sound = self.registers[(instruction & 0xF) as usize];
                            self.pc += 2
                        }
                        0x1E => {
                            self.i = self.i + (self.registers[(instruction & 0xF) as usize] as u16);
                            self.pc += 2
                        }
                        0x29 => {
                            panic!("Not implemented yet")
                        }
                        0x33 => {
                            panic!("Not implemented yet")}
                        0x55 => {
                            panic!("Not implemented yet")}
                        0x65 => {
                            // TODO: Really iterate over data and why?
                            self.pc += 2
                        }
                        unknown_code => {
                            panic!("Unkown subcode {:x} not found on code {:x}",
                                   0xE,
                                   unknown_code);
                        }
                    }
                }
                unknown_code => panic!("Unkown code: {:x}", unknown_code),
            }
             sleep(*SLEEP_TIME);
        }
    }
}
