use std::env;
use std::fs;
use std::fmt;
use std::io;
use std::ops;

const MACHINE_MEMORY_SIZE: usize = 256;

struct Memory ([u8; MACHINE_MEMORY_SIZE]);

#[derive(Debug)]
struct Machine {
    pc: u8,
    memory: Memory,
    acc: u8,
}

impl Memory {
    fn from_program(program: &Vec<u8>) -> Self {
        assert!(program.len() <= MACHINE_MEMORY_SIZE);

        let mut out = [0; MACHINE_MEMORY_SIZE];
        out[..program.len()].copy_from_slice(program);
        Memory(out)
    }
}

impl ops::Index<u8> for Memory {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl ops::IndexMut<u8> for Memory {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl ops::Index<ops::RangeInclusive<u8>> for Memory {
    type Output = [u8];

    fn index(&self, index: ops::RangeInclusive<u8>) -> &Self::Output {
        let (start, end) = index.into_inner();
        &self.0[start as usize..=end as usize]
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        // represent memory as hex block
        let mut out = String::new();
        for byte in self.0.iter() {
            out.push_str(&format!("{:02x} ", byte));
        }

        out.fmt(f)
    }
}

impl Machine {
    fn from_program(program: &Vec<u8>) -> Self {
        Machine {
            pc: 0,
            acc: 0,
            memory: Memory::from_program(program)
        }
    }

    fn run(&mut self) {

        loop {
            let current_op_code = self.memory[self.pc];
            let instruction_len = match current_op_code {

                // load
                0x10 => {
                    let src = self.memory[self.pc + 1];
                    self.acc = self.memory[src];
                    2
                }

                // load immediate
                0x11 => {
                    self.acc = self.memory[self.pc + 1];
                    2
                }

                // store
                0x12 => {
                    let dst = self.memory[self.pc + 1];
                    self.memory[dst] = self.acc;
                    2
                }

                // jump
                0x20 => {
                    self.pc = self.memory[self.pc + 1];
                    0
                }

                // jump zero
                0x21 => {
                    if self.acc == 0 {
                        self.pc = self.memory[self.pc + 1];
                        0
                    } else {
                        2
                    }
                }

                // add immediate
                0x31 => {
                    self.acc = self.acc.wrapping_add(self.memory[self.pc + 1]);
                    2
                }

                // substract immediate
                0x33 => {
                    self.acc = self.acc.wrapping_sub(self.memory[self.pc + 1]);
                    2
                }

                // print
                0x40 => {
                    let src = self.memory[self.pc + 1];
                    let len = self.memory[src];
                    let chars = &self.memory[src+1..=src+len];
                    println!("{}", String::from_utf8_lossy(chars));
                    2
                }

                // stop
                0x50 => return,

                opcode => panic!("Unknown instruction {:02x} in position: {:02x}", opcode, self.pc),
            };

            self.pc = self.pc.wrapping_add(instruction_len);
        }
    }
}

fn main() -> Result<(), io::Error> {

    // get program file name from command line args
    let prog_name = if let Some(name) = env::args().nth(1) {
        name
    } else {
        println!("No program found to emulte!");
        std::process::exit(1);
    };
    println!("Program name: {}", prog_name);

    let program = fs::read(prog_name)?;
    println!("Raw program: {:02x?}", program);

    let mut machine = Machine::from_program(&program);
    println!("{:#?}", machine);

    println!("Running program:");
    machine.run();

    Ok(())
}
