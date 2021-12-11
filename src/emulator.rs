use crate::parser::{decode_instruction, ALEOp, CMPOp, Instruction};
use byteorder::{ByteOrder, LittleEndian};
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
struct Registers {
    data: [u8; 16],
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut names = String::new();
        let mut values = String::new();
        for r in 0..=15 {
            names.push_str(&format!("r{:<2} ", r));
            values.push_str(&format!("{:02x}h ", self.data[r]));
        }
        write!(f, "{}\n{}", names, values)
    }
}

impl Index<usize> for Registers {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        &self.data[index]
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

#[derive(Debug)]
pub struct State {
    cycles: usize,
    memory: [u8; 4096],
    pc: usize,
    flagg: bool,
    returnpointers: Vec<usize>,
    pub finished: bool,
    registers: Registers,
    stdin: Vec<u8>,
    stdinpointer: usize,
    stdout: Vec<u8>,
}

impl State {
    pub fn new(mem: [u8; 4096], stdin: Vec<u8>) -> State {
        State {
            cycles: 0,
            memory: mem,
            pc: 0,
            flagg: false,
            finished: false,
            registers: Registers { data: [0; 16] },
            stdin,
            returnpointers: Vec::new(),
            stdinpointer: 0,
            stdout: Vec::new(),
        }
    }

    pub fn display(&self) -> String {
        format!(
            "pc: {:#6x} flagg: {} sykler: {}\n{}\n{}",
            self.pc,
            self.flagg,
            self.cycles,
            self.registers,
            self.next_instruction()
        )
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn dump_memory(&self, from: usize, to: usize) {
        if from > to || to > 0xfff {
            eprintln!("Error: Memory adress incorrect.")
        } else {
            for (i, addr) in (from..to).enumerate() {
                if i % 8 == 0 {
                    print!("{:04x}: ", addr);
                }
                print!("{:02x} ", &self.memory[addr]);
                if i % 8 == 7 {
                    println!();
                }
            }
            println!();
        }
    }

    pub fn next_instruction(&self) -> Instruction {
        let op = LittleEndian::read_u16(&self.memory[self.pc..self.pc + 2]);
        decode_instruction(op)
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn execute(&mut self) {
        let instruction = self.next_instruction();
        self.pc += 2;
        self.cycles += 1;

        match instruction {
            Instruction::Sett(ra, rb) => self.registers[ra] = self.registers[rb],
            Instruction::SettImmediate(ra, val) => self.registers[ra] = val,
            Instruction::Stopp => self.finished = true,
            Instruction::Hopp(loc) => self.pc = usize::from(loc),
            Instruction::BHopp(loc) => {
                if self.flagg {
                    self.pc = usize::from(loc);
                }
            }
            Instruction::Les(r) => {
                self.registers[r] = self.stdin[self.stdinpointer];
                self.stdinpointer += 1;
            }
            Instruction::Skriv(r) => self.stdout.push(self.registers[r]),
            Instruction::Tur(adr) => {
                self.returnpointers.push(self.pc);
                self.pc = usize::from(adr);
            }
            Instruction::Retur => self.pc = self.returnpointers.pop().unwrap(),
            Instruction::Finn(adr) => {
                let lsb = adr as u8;
                let msb = (adr >> 8) as u8;
                self.registers[0] = lsb;
                self.registers[1] = msb;
            }
            Instruction::Last(n) => {
                let msb = u16::from(self.registers[1]);
                let adr: u16 = (msb << 8 | u16::from(self.registers[0])) & 0xfff;
                self.registers[n] = self.memory[adr as usize];
            }
            Instruction::Lagr(n) => {
                let msb = u16::from(self.registers[1]);
                let adr: u16 = (msb << 8 | u16::from(self.registers[0])) & 0xfff;
                self.memory[adr as usize] = self.registers[n];
            }
            Instruction::ALE(op, ra, rb) => match op {
                ALEOp::Pluss => {
                    self.registers[ra] =
                        ((self.registers[ra] as u16 + self.registers[rb] as u16) % 0x100) as u8
                }
                ALEOp::Minus => {
                    self.registers[ra] =
                        ((self.registers[ra] as u16 - self.registers[rb] as u16) % 0x100) as u8
                }
                ALEOp::Eller => self.registers[ra] |= self.registers[rb],
                ALEOp::XEller => self.registers[ra] ^= self.registers[rb],
                ALEOp::Og => self.registers[ra] &= self.registers[rb],
                ALEOp::Vskift => self.registers[ra] <<= self.registers[rb],
                ALEOp::Hskift => self.registers[ra] >>= self.registers[rb],
            },
            Instruction::Sammenligne(op, ra, rb) => {
                let a = self.registers[ra];
                let b = self.registers[rb];

                self.flagg = match op {
                    CMPOp::Lik => a == b,
                    CMPOp::Me => a < b,
                    CMPOp::Mel => a <= b,
                    CMPOp::Se => a > b,
                    CMPOp::Sel => a >= b,
                    CMPOp::Ulik => a != b,
                }
            }
            Instruction::Nope => (),
            Instruction::Data(_) => (),
        }
        if self.pc >= self.memory.len() {
            self.finished = true;
        }
    }
}
