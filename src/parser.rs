use byteorder::{ByteOrder, LittleEndian};
use std::fmt;

pub struct Line {
    pub pos: usize,
    pub op: u16,
    pub instruction: Instruction,
}

pub enum ALEOp {
    Og,
    Eller,
    XEller,
    Vskift,
    Hskift,
    Pluss,
    Minus,
}

impl fmt::Display for ALEOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cmd = match self {
            ALEOp::Og => "OG",
            ALEOp::Eller => "ELLER",
            ALEOp::XEller => "XELLER",
            ALEOp::Vskift => "VSKIFT",
            ALEOp::Hskift => "HSKIFT",
            ALEOp::Pluss => "PLUSS",
            ALEOp::Minus => "MINUS",
        };
        write!(f, "{}", cmd)
    }
}

pub enum CMPOp {
    Lik,
    Ulik,
    Me,
    Mel,
    Se,
    Sel,
}

impl fmt::Display for CMPOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cmd = match self {
            CMPOp::Lik => "LIK",
            CMPOp::Ulik => "ULIK",
            CMPOp::Me => "ME",
            CMPOp::Mel => "MEL",
            CMPOp::Se => "SE",
            CMPOp::Sel => "SEL",
        };
        write!(f, "{}", cmd)
    }
}

pub enum Instruction {
    SettImmediate(usize, u8),
    Sett(usize, usize),
    Nope,
    Stopp,
    ALE(ALEOp, usize, usize),
    Sammenligne(CMPOp, usize, usize),
    Hopp(u16),
    BHopp(u16),
    Les(usize),
    Skriv(usize),
    Finn(u16),
    Last(usize),
    Lagr(usize),
    Tur(u16),
    Retur,
    Data(u16),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Sett(o1, o2) => write!(f, "SETT\tr{}, r{}", o1, o2),
            Instruction::SettImmediate(o1, o2) => write!(f, "SETT\tr{}, {}", o1, o2),
            Instruction::Nope => write!(f, "NOPE"),
            Instruction::Stopp => write!(f, "STOPP"),
            Instruction::ALE(i, o1, o2) => write!(f, "{}\tr{}, r{}", i, o1, o2),
            Instruction::Sammenligne(i, o1, o2) => write!(f, "{}\tr{}, r{}", i, o1, o2),
            Instruction::Hopp(o1) => write!(f, "HOPP\t{:#06x}", o1),
            Instruction::BHopp(o1) => write!(f, "BHOPP\t{:#06x}", o1),
            Instruction::Les(o1) => write!(f, "LES\tr{}", o1),
            Instruction::Skriv(o1) => write!(f, "SKRIV\tr{}", o1),
            Instruction::Finn(o1) => write!(f, "FINN\t{:#06x}", o1),
            Instruction::Last(o1) => write!(f, "LAST\tr{}", o1),
            Instruction::Lagr(o1) => write!(f, "LAGR\tr{}", o1),
            Instruction::Tur(o1) => write!(f, "TUR\t{:#06x}", o1),
            Instruction::Retur => write!(f, "RETUR"),
            Instruction::Data(d) => write!(f, ".DATA\t{:#x},{:#x}", (d >> 8), d & 0xff),
        }
    }
}

pub fn decode_instruction(op: u16) -> Instruction {
    let opclass = op & 0xf;
    let operation = ((op >> 4) & 0xf) as usize;
    let value = (op >> 8) as u8;
    let address = (op >> 4) as u16;
    let arg1 = ((op >> 8) & 0xf) as usize;
    let arg2 = ((op >> 12) & 0xf) as usize;

    let data = Instruction::Data(op);

    match opclass {
        0x0 => Instruction::Stopp,
        0x1 => Instruction::SettImmediate(operation, value),
        0x2 => Instruction::Sett(operation, arg1),
        0x3 => Instruction::Finn(address),
        0x4 => {
            if operation == 0 {
                Instruction::Last(arg1)
            } else if operation == 1 {
                Instruction::Lagr(arg1)
            } else {
                data
            }
        }
        0x5 => {
            if let Some(aleop) = match operation {
                0 => Some(ALEOp::Og),
                1 => Some(ALEOp::Eller),
                2 => Some(ALEOp::XEller),
                3 => Some(ALEOp::Vskift),
                4 => Some(ALEOp::Hskift),
                5 => Some(ALEOp::Pluss),
                6 => Some(ALEOp::Minus),
                _ => None,
            } {
                Instruction::ALE(aleop, arg1, arg2)
            } else {
                data
            }
        }
        0x6 => match operation {
            0 => Instruction::Les(arg1),
            1 => Instruction::Skriv(arg1),
            _ => data,
        },
        0x7 => {
            if let Some(cmpop) = match operation {
                0 => Some(CMPOp::Lik),
                1 => Some(CMPOp::Ulik),
                2 => Some(CMPOp::Me),
                3 => Some(CMPOp::Mel),
                4 => Some(CMPOp::Se),
                5 => Some(CMPOp::Sel),
                _ => None,
            } {
                Instruction::Sammenligne(cmpop, arg1, arg2)
            } else {
                data
            }
        }
        0x8 => Instruction::Hopp(address),
        0x9 => Instruction::BHopp(address),
        0xa => Instruction::Tur(address),
        0xb => Instruction::Retur,
        0xc => Instruction::Nope,
        _ => data,
    }
}

pub fn disassemble(input: &[u8]) -> Vec<Line> {
    if input[0..7] != vec![0x2e, 0x53, 0x4c, 0x45, 0x44, 0x45, 0x38] {
        panic!("Not valid SLEDE8 file (missing magic bytes)");
    }

    let mut program: &[u8] = &input[7..input.len()];
    if program.len() % 2 != 0 {
        program = &input[7..input.len() - 1];
    }
    program
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| {
            let combined = LittleEndian::read_u16(chunk);
            Line {
                pos: i * 2,
                op: combined,
                instruction: decode_instruction(combined),
            }
        })
        .collect()
}
