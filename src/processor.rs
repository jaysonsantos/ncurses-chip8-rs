use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use byteorder::{BigEndian, ReadBytesExt};
use enum_primitive_derive::Primitive;
use log::{debug, log_enabled, trace};
use log::Level;
use num_traits::FromPrimitive;

const SLEEP_TIME: Duration = Duration::from_secs(1);

pub struct Processor {
    registers: [u8; 16],
    i: u16,
    delay: u8,
    sound: u8,
    pc: u16,
    sp: u8,
    rom: Cursor<Vec<u8>>,
}

#[derive(Primitive, Debug)]
#[repr(u8)]
pub enum Instruction {
    Goto = 0x1,
    Call = 0x2,
    SkipNotEqual = 0x4,
    SkipEqual = 0x5,
    Set = 0x6,
    Add = 0x7,
    ZeroXEightFindBetterName = 0x8,
    SetIValue = 0xA,
    Draw = 0xD,
    SkipWithKey = 0xE, // Skip with key pressed or not
    ZeroXFFindBetterName = 0xF // Skip with key pressed or not
}


impl Processor {
    pub fn new<F>(file: F) -> Processor
    where
        F: AsRef<Path>,
    {
        let mut f = File::open(file).expect("Error opening file");
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
            rom: Cursor::new(buf),
        }
    }

    pub fn run(&mut self) -> i32 {
        loop {
            if log_enabled!(Level::Trace) {
                let memory: Vec<String> =
                    self.registers.iter().map(|e| format!("{:x}", e)).collect();
                trace!("Memory {:?}", memory);
            }

            self.rom.set_position(self.pc as u64);
            debug!("Cursor in {:x}", self.pc);
            let data = self.rom.read_u8().unwrap();
            debug!("Data {:x}", data);
            let instruction = data >> 4;
            let instruction = Instruction::from_u8(instruction).unwrap();
            debug!("Instruction {:?}", instruction);
            match instruction {
                Instruction::Goto => {
                    // TODO: Fix this
                    self.rom.set_position((self.pc) as u64);
                    let addr = self.rom.read_u16::<BigEndian>().unwrap();
                    debug!("Jumping to {:x} on {:x}", (addr & 0xFFF) - 0x200, addr);
                    self.pc = addr & (0xFFF - 0x200);
                }
                Instruction::SkipNotEqual => {
                    if (data & 0xF) != self.rom.read_u8().unwrap() {
                        self.pc += 4
                    } else {
                        self.increment_pc()
                    }
                }
                Instruction::SkipEqual => {
                    let x = data & 0xF;
                    let y = self.rom.read_u8().unwrap() >> 4;
                    if self.registers[x as usize] == self.registers[y as usize] {
                        self.pc += 4
                    } else {
                        self.increment_pc()
                    }
                }
                Instruction::Call => {
                    // Call a function and increase stack.
                    self.rom.set_position((self.pc - 1) as u64);
                    self.pc = self.rom.read_u16::<BigEndian>().unwrap() & 0xFFF;
                    self.sp += 1;
                }
                Instruction::Set => {
                    // Set value on register
                    let location = (data & 0xF) as usize;
                    let value = self.rom.read_u8().unwrap();
                    debug!("Setting {:x} on register {:x}", value, location);
                    self.registers[location] = value;
                    self.increment_pc();
                }
                Instruction::Add => {
                    // Increment value on register
                    // TODO: Overflow value
                    let location = (data & 0xF) as usize;
                    let value = self.rom.read_u8().unwrap();
                    self.registers[location] += value;
                    self.increment_pc();
                }
                Instruction::ZeroXEightFindBetterName => {
                    let subcommand = self.rom.read_u8().unwrap() & 0xF;
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
                    self.increment_pc()
                }
                Instruction::SetIValue => {
                    // Set I value
                    self.rom.set_position((self.pc - 1) as u64);
                    let value = self.rom.read_u16::<BigEndian>().unwrap() & 0xFFF;
                    debug!("Setting I to {:x}", value);
                    self.i = value;
                    self.increment_pc();
                }
                Instruction::Draw => {
                    // Display sprite on screen
                    self.increment_pc();
                }
                Instruction::SkipWithKey => {
                    let value = data & 0xF;
                    let subcommand = self.rom.read_u8().unwrap();
                    match subcommand {
                        0x9E => {
                            // If a key with value is pressed skip next
                            // instruction pc += 4
                            if false {
                                self.increment_pc()
                            } else {
                                self.pc += 4
                            }
                        }
                        0xA1 => {
                            // If a key with value is not pressed skip next
                            // instruction pc += 4
                            if false {
                                self.increment_pc()
                            } else {
                                self.pc += 4
                            }
                        }
                        unknown_code => {
                            panic!(
                                "Unknown subcode {:x} not found on code {:x}",
                                0xE, unknown_code
                            );
                        }
                    }
                }
                Instruction::ZeroXFFindBetterName => {
                    let subcommand = self.rom.read_u8().unwrap();
                    debug!("Subcommand {:x}", subcommand);
                    match subcommand {
                        0x07 => {
                            self.registers[instruction as usize & 0xF] = self.delay;
                            self.increment_pc();
                        }
                        0x0A => {
                            // TODO: Wait for key
                            // self.registers[instruction as usize & 0xF] = key
                        }
                        0x15 => {
                            self.delay = self.registers[instruction as usize & 0xF];
                            self.increment_pc();
                        }
                        0x18 => {
                            self.sound = self.registers[instruction as usize & 0xF];
                            self.increment_pc()
                        }
                        0x1E => {
                            self.i += self.registers[instruction as usize & 0xF] as u16;
                            self.increment_pc()
                        }
                        0x29 => panic!("Not implemented yet"),
                        0x33 => panic!("Not implemented yet"),
                        0x55 => panic!("Not implemented yet"),
                        0x65 => {
                            // TODO: Really iterate over data and why?
                            self.increment_pc()
                        }
                        unknown_code => {
                            panic!(
                                "Unkown subcode {:x} not found on code {:x}",
                                0xE, unknown_code
                            );
                        }
                    }
                }
            }
            sleep(SLEEP_TIME);
        }
    }

    fn increment_pc(&mut self) {
        self.pc += 2
    }
}
