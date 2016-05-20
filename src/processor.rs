use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};

pub struct Processor {
    registers: [u8; 16],
    I: u16,
    delay: u8,
    sound: u8,
    PC: u16,
    SP: u8,
    stack: [u16; 16],
    rom: Vec<u8>
}

impl Processor {
    pub fn new<F>(file: F) -> Processor
        where F: AsRef<Path>
    {
        let mut f = File::open(file).expect("Error openning file");
        let mut buf = vec![];
        f.read_to_end(&mut buf).unwrap();
        Processor { registers: [0; 16], I: 0, delay: 0, sound: 0, PC: 0, SP: 0,
                    stack: [0; 16], rom: buf }
    }

    pub fn run(&mut self) -> i32 {
        let mut cursor = Cursor::new(&self.rom);
        loop {
            cursor.set_position(self.PC as u64);
            debug!("Cursor in {:x}", self.PC);
            let data = cursor.read_u8().unwrap();
            debug!("Data {:x}", data);
            let instruction = data >> 4;
            debug!("Instruction {:x}", instruction);
            match instruction {
                0x1 => {
                    cursor.set_position((self.PC - 1) as u64);
                    self.PC = cursor.read_u16::<BigEndian>().unwrap() & 0xFFF;
                },
                0x4 => {
                    if (data & 0xF) != cursor.read_u8().unwrap() {
                        self.PC += 4
                    } else {
                        self.PC += 2
                    }
                },
                0x2 => {
                    // Call a function and increase stack.
                    cursor.set_position((self.PC -1) as u64);
                    self.PC = cursor.read_u16::<BigEndian>().unwrap() & 0xFFF;
                    self.SP += 1;
                },
                0x6 => {
                    // Set value on register
                    let location = (data & 0xF) as usize;
                    let value = cursor.read_u8().unwrap();
                    debug!("Setting {:x} on register {:x}", value, location);
                    self.registers[location] = value;
                    self.PC += 2;
                },
                0x7 => {
                    // Increment value on register
                    // TODO: Overflow value
                    let location = (data & 0xF) as usize;
                    let value = cursor.read_u8().unwrap();
                    self.registers[location] += value;
                    self.PC += 2;
                },
                0x8 => {
                    let subcommand = cursor.read_u8().unwrap() & 0xF;
                    match subcommand {
                        0 => {
                            self.registers[(data & 0xF) as usize] = subcommand >> 4
                        },
                        1 => {
                            self.registers[(data & 0xF) as usize] = (data & 0xF) | (subcommand >> 4)
                        },
                        2 => {
                            self.registers[(data & 0xF) as usize] = (data & 0xF) & (subcommand >> 4)
                        },
                        3 => {
                            self.registers[(data & 0xF) as usize] = (data & 0xF) ^ (subcommand >> 4)
                        },
                        4 => {
                            // TODO: Check for overflow
                            self.registers[(data & 0xF) as usize] = (data & 0xF) + (subcommand >> 4)
                        },
                        _ => panic!("Subcommand {:x} not found in command 0x8", subcommand)
                    };
                    self.PC += 2
                },
                0xA => {
                    // Set I value
                    cursor.set_position((self.PC - 1) as u64);
                    let value = cursor.read_u16::<BigEndian>().unwrap() & 0xFFF;
                    debug!("Setting I to {:x}", value);
                    self.I = value;
                    self.PC += 2;
                },
                0xD => {
                    // Display sprite on screen
                    self.PC += 2;
                },
                0xE => {
                    let value = data & 0xF;
                    let subcommand = cursor.read_u8().unwrap();
                    match subcommand {
                        0x9E => {
                            // If a key with value is pressed skip next
                            // instruction PC += 4
                            if (false) {
                                self.PC += 2
                            } else {
                                self.PC += 4
                            }
                        },
                        0xA1 => {
                            // If a key with value is not pressed skip next
                            // instruction PC += 4
                            if (false) {
                                self.PC += 2
                            } else {
                                self.PC += 4
                            }
                        },
                        unknown_code => {
                            panic!("Unkown subcode {:x} not found on code {:x}",
                                   0xE, unknown_code);
                        }
                    }
                },
                unknown_code => {
                    panic!("Unkown code: {:x}", unknown_code)
                }
            }
        }
        0
    }

    pub fn clear_screen(&self) {
    }
}
