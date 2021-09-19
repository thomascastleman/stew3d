use bimap::BiMap;
use instr::Instruction::{self, *};
use instr::Opcode::{self, *};
use instr::Operands::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, Read};

mod instr;

fn main() -> io::Result<()> {
    // TODO: take this as CLI arg
    let filename = "data/multiple-labels.3000.b";

    let mut f = File::open(filename)?;
    let mut buffer = Vec::new();
    let file_size = f.read_to_end(&mut buffer)?;

    println!("file is {} bytes", file_size);

    match parse(&buffer) {
        Err(e) => eprintln!("{:?}", e),
        Ok(instrs) => {
            println!("{:#?}", instrs);
        }
    };

    Ok(())
}

static mut GENSYM_COUNTER: usize = 0;

/// Generates a unique name, given a basename.
fn gensym(base: &str) -> String {
    // SAFETY: There is only one thread which accesses GENSYM_COUNTER.
    let name = format!("{}{}", base, unsafe { GENSYM_COUNTER });
    unsafe {
        GENSYM_COUNTER += 1;
    }
    name
}

#[derive(Debug)]
pub enum Error {
    InvalidOpcode(u8),
    UnexpectedEndOfFile(Opcode),
}

/// Parses a slice of bytes into an assembly program (list of instructions).
fn parse(bytes: &[u8]) -> Result<Vec<Instruction>, Error> {
    let mut bytes = bytes.iter();
    let mut instrs = Vec::new();

    // This map maintains a bidirectional correspondence between addresses and labels
    let mut label_addr_map: BiMap<u8, String> = BiMap::new();

    while let Some(&opcode) = bytes.next() {
        let opcode: Opcode = opcode.try_into()?;
        let size = size_from_opcode(opcode)?;

        // Expect another byte in the input stream and error with unexpected
        // end of input if no more bytes.
        let mut expect_operand = || bytes.next().ok_or(Error::UnexpectedEndOfFile(opcode));

        let ins = match size {
            // opcode + no operands
            InstrSize::OneByte => Instr(opcode, Zero),
            // opcode + single operand
            InstrSize::TwoByte => {
                let operand = *expect_operand()?;

                match opcode {
                    // If the instruction is a jump (needs labelss)
                    JMP | JE | JNE | JL | JLE | JG | JGE | JA | JAE | JB | JBE | CALL => {
                        // Check map for label already generated for this address
                        match label_addr_map.get_by_left(&operand) {
                            Some(label) => Jump(opcode, operand, label.clone()),
                            None => {
                                // No label for this address, generate a new one and
                                // insert it into the map.
                                let new_label = gensym("l");
                                label_addr_map.insert(operand, new_label.clone());
                                Jump(opcode, operand, new_label.clone())
                            }
                        }
                    }
                    _ => Instr(opcode, One(operand)),
                }
            }
            // opcode + two operands
            InstrSize::ThreeByte => {
                let operand1 = *expect_operand()?;
                let operand2 = *expect_operand()?;
                Instr(opcode, Two(operand1, operand2))
            }
        };

        instrs.push(ins);
    }

    let mut addr = 0;
    let mut with_labels = Vec::with_capacity(instrs.len());
    for ins in &instrs {
        // If a label points at this address, add one
        if let Some(label) = label_addr_map.get_by_left(&addr) {
            with_labels.push(Label(addr, label.clone()));
        }

        let opcode = match ins {
            Jump(opcode, _, _) => opcode,
            Instr(opcode, _) => opcode,
            _ => panic!("TODO: unreachable arm"),
        };

        addr += match size_from_opcode(*opcode)? {
            InstrSize::OneByte => 1,
            InstrSize::TwoByte => 2,
            InstrSize::ThreeByte => 3,
        };

        with_labels.push(ins.clone());
    }

    Ok(with_labels)
}

enum InstrSize {
    OneByte,
    TwoByte,
    ThreeByte,
}

/// Determines the size of an instruction, given its opcode.
fn size_from_opcode(opcode: Opcode) -> Result<InstrSize, Error> {
    match opcode as u8 {
        // add, addc, sub, subb, and, or, xor, not, neg, inr, inr2, inr3,
        // dcr, dcr2, dcr3, mov, ld, st, cmp, ret, out, dd, hlt, nop
        0x00..=0x0b
        | 0x10..=0x1b
        | 0x20..=0x28
        | 0x2d..=0x35
        | 0x3a..=0x3f
        | 0x43..=0x48
        | 0x4c..=0x51
        | 0x55..=0x7e
        | 0x82..=0x96
        | 0x9f..=0xaa
        | 0xbd..=0xc0
        | 0xc4..=0xc8 => Ok(InstrSize::OneByte),

        // addi, addci, subi, subbi, ani, ori, xri, mvi, lds, sts, cmpi,
        // jmp, je, jne, jg, jge, jl, jle, ja, jae, jb, jbe, call, outi,
        // dic, did,
        0x0c..=0x0f
        | 0x1c..=0x1f
        | 0x29..=0x2c
        | 0x36..=0x39
        | 0x40..=0x42
        | 0x49..=0x4b
        | 0x52..=0x54
        | 0x7f..=0x81
        | 0x97..=0x9d
        | 0xab..=0xbc
        | 0xc1..=0xc3 => Ok(InstrSize::TwoByte),

        // stsi
        0x9e => Ok(InstrSize::ThreeByte),

        opcode => Err(Error::InvalidOpcode(opcode)),
    }
}
