use anyhow::Result;
use bimap::BiMap;
use instr::Instruction::{self, *};
use instr::Operands::*;
use opcode::Opcode::{self, *};
use stats::BinaryStats;
use std::convert::TryInto;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use structopt::StructOpt;

mod instr;
mod opcode;
mod stats;

#[derive(StructOpt, Debug)]
#[structopt(name = "stew3d")]
#[doc(hidden)]
struct Opt {
    /// The binary to disassemble. If none provided, reads from stdin.
    #[structopt(name = "FILE")]
    file: Option<String>,

    /// Show statistics about the binary.
    #[structopt(short, long)]
    stats: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Reads the file given by command line arguments and invokes the disassembler on its contents.
fn run() -> Result<()> {
    let opt = Opt::from_args();
    let mut buffer = Vec::new();

    let bytes_read = match opt.file {
        None => io::stdin().read_to_end(&mut buffer)?,
        Some(ref filename) => File::open(&filename)?.read_to_end(&mut buffer)?,
    };

    let instrs = disassemble(&buffer)?;

    println!(
        "\nDisassembly of file `{}` ({} bytes)\n",
        &opt.file.unwrap_or_else(|| "stdin".into()),
        bytes_read
    );

    if opt.stats {
        println!("{}", BinaryStats::new(&instrs));
    }

    for ins in instrs {
        let bytes_str = ins
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{:6} {:8} | {}",
            format!("{:02x}:", ins.addr()),
            bytes_str,
            ins
        );
    }

    Ok(())
}

/// Represents possible errors that can occur while disassembling. `InvalidOpcode`
/// indicates an opcode outside the valid range was encountered. `UnexpectedEndOfFile`
/// indicates we were in the middle of parsing the operands for an instruction,
/// but encountered the end of input before all the operands were provided.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidOpcode(u8, usize),
    UnexpectedEndOfFile(Opcode),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpcode(opcode, addr) => {
                write!(
                    f,
                    "invalid opcode encountered at byte {}: `{:x}`",
                    addr, opcode
                )
            }
            Self::UnexpectedEndOfFile(opcode) => write!(
                f,
                "unexpected end of file while processing instruction with opcode {:02x}",
                *opcode as u8
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Parses a slice of bytes into an assembly program (list of instructions).
///
/// # Examples
/// ```
/// // outi 1; hlt
/// let bytes = [0xc1, 0x01, 0xc7];
/// assert_eq!(
///     disassemble(&bytes).unwrap(),
///     vec![Instr(0x00, OUTI, One(0x01)), Instr(0x02, HLT, Zero)],
/// );
/// ```
fn disassemble(bytes: &[u8]) -> Result<Vec<Instruction>, Error> {
    let mut bytes = bytes.iter();
    let mut instrs = Vec::new();

    // Gensym is used to generate unique label names
    let mut gensym_counter: usize = 0;
    let mut gensym = move |base: &str| -> String {
        gensym_counter += 1;
        format!("{}{}", base, gensym_counter - 1)
    };

    // This map maintains a bidirectional correspondence between addresses and labels
    let mut label_addr_map: BiMap<usize, String> = BiMap::new();

    let mut addr = 0; // current address in binary

    while let Some(&opcode) = bytes.next() {
        let opcode: Opcode = match opcode.try_into() {
            Ok(opcode) => opcode,
            Err(_) => return Err(Error::InvalidOpcode(opcode, addr)),
        };
        let size = opcode.instruction_size();

        // Expect another byte in the input stream and error with unexpected
        // end of input if no more bytes.
        let mut expect_operand = || bytes.next().ok_or(Error::UnexpectedEndOfFile(opcode));

        let ins = match size {
            // Opcode + no operands
            1 => Instr(addr, opcode, Zero),
            // Opcode + single operand
            2 => {
                let operand = *expect_operand()?;

                match opcode {
                    // If the instruction is a jump (needs labels)
                    JMP | JE | JNE | JL | JLE | JG | JGE | JA | JAE | JB | JBE | CALL => {
                        // Check map for label already generated for this address
                        match label_addr_map.get_by_left(&(operand as usize)) {
                            Some(label) => Jump(addr, opcode, operand, label.clone()),
                            None => {
                                // No label for this address, generate a new one and
                                // insert it into the map.
                                let new_label = gensym("l");
                                label_addr_map.insert(operand as usize, new_label.clone());
                                Jump(addr, opcode, operand, new_label.clone())
                            }
                        }
                    }
                    _ => Instr(addr, opcode, One(operand)),
                }
            }
            // Opcode + two operands
            3 => {
                let operand1 = *expect_operand()?;
                let operand2 = *expect_operand()?;
                Instr(addr, opcode, Two(operand1, operand2))
            }
            // All instructions are currently between 1-3 bytes in size.
            _ => unreachable!(),
        };

        instrs.push(ins);
        addr += size;
    }

    let mut addr: usize = 0;
    let mut with_labels = Vec::with_capacity(instrs.len());
    for ins in &instrs {
        // If a label points at this address, add one
        if let Some(label) = label_addr_map.get_by_left(&addr) {
            with_labels.push(Label(addr as usize, label.clone()));
        }

        let opcode = match ins {
            Jump(_, opcode, _, _) => opcode,
            Instr(_, opcode, _) => opcode,
            _ => unreachable!(),
        };

        addr += opcode.instruction_size();
        with_labels.push(ins.clone());
    }

    Ok(with_labels)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_disassembly() {
        let b = [0x7f, 0x0a, 0xbc, 0x05, 0xc7, 0x0c, 0x04, 0xbd];
        assert_eq!(
            disassemble(&b).unwrap(),
            vec![
                Instr(0x00, MVI_A, One(0x0a)),
                Jump(0x02, CALL, 0x05, String::from("l0")),
                Instr(0x04, HLT, Zero),
                Label(0x05, String::from("l0")),
                Instr(0x05, ADDI_A, One(0x04)),
                Instr(0x07, RET, Zero)
            ]
        );
    }

    #[test]
    fn errs_on_invalid_opcode() {
        // df is above OPCODE_MAX
        let b = [0x80, 0x05, 0xc5, 0xdf, 0xc7];
        assert_eq!(disassemble(&b), Err(Error::InvalidOpcode(0xdf, 3)));
    }

    #[test]
    fn errs_on_unexpected_eof() {
        // 97 (lds byte, a) expects a byte operand
        let b = [0xc8, 0xc8, 0x6f, 0x97];
        assert_eq!(disassemble(&b), Err(Error::UnexpectedEndOfFile(LDS_A)));
    }
}
