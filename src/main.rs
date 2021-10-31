use bimap::BiMap;
use instr::Instruction::{self, *};
use instr::Operands::*;
use opcode::InstrSize;
use opcode::Opcode::{self, *};
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, Read};
use structopt::StructOpt;

mod instr;
mod opcode;

#[derive(StructOpt, Debug)]
#[structopt(name = "stew3d")]
struct Opt {
    /// The binary to disassemble.
    #[structopt(name = "FILE")]
    file: String,

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

fn run() -> io::Result<()> {
    let opt = Opt::from_args();
    let filename = opt.file;

    let mut f = File::open(&filename)?;
    let mut buffer = Vec::new();
    let file_size = f.read_to_end(&mut buffer)?;

    println!(
        "\nDisassembly of file `{}` ({} bytes)\n",
        &filename, file_size
    );

    match disassemble(&buffer) {
        Err(e) => {
            // TODO: this shouldn't be Debug printing
            eprintln!("{:?}", e)
        }
        Ok(instrs) => {
            if opt.stats {
                show_stats(&instrs);
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
        }
    };

    Ok(())
}

/// Prints stats about the analyzed binary.
fn show_stats(instrs: &[Instruction]) {
    let sum_up = |f: fn(&Instruction) -> usize| instrs.iter().map(f).sum();
    let count_instrs = |pred: fn(&&Instruction) -> bool| instrs.iter().filter(pred).count();
    let percentage = |num: usize, denom: usize| (num as f64 / denom as f64) * 100.0;

    let total_bytes = sum_up(|ins| ins.size());
    let total_instrs = count_instrs(|ins| !matches!(ins, Label(_, _)));
    let opcode_bytes: usize = sum_up(|ins| ins.num_opcodes());
    let operand_bytes: usize = sum_up(|ins| ins.num_operands());

    println!("{} instructions ({} bytes)", total_instrs, total_bytes);
    println!(
        "Opcode bytes: {:.2}% ({} bytes)",
        percentage(opcode_bytes, total_bytes),
        opcode_bytes
    );
    println!(
        "Operand bytes: {:.2}% ({} bytes)",
        percentage(operand_bytes, total_bytes),
        operand_bytes
    );

    let single_byte_intrs = count_instrs(|ins| ins.size() == 1);
    let two_byte_intrs = count_instrs(|ins| ins.size() == 2);
    let three_byte_intrs = count_instrs(|ins| ins.size() == 3);

    println!(
        "1-byte instructions: {:.2}% ({})",
        percentage(single_byte_intrs, total_instrs),
        single_byte_intrs
    );
    println!(
        "2-byte instructions: {:.2}% ({})",
        percentage(two_byte_intrs, total_instrs),
        two_byte_intrs
    );
    println!(
        "3-byte instructions: {:.2}% ({})",
        percentage(three_byte_intrs, total_instrs),
        three_byte_intrs
    );

    println!();
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
fn disassemble(bytes: &[u8]) -> Result<Vec<Instruction>, Error> {
    let mut bytes = bytes.iter();
    let mut instrs = Vec::new();

    // This map maintains a bidirectional correspondence between addresses and labels
    let mut label_addr_map: BiMap<usize, String> = BiMap::new();

    let mut addr = 0; // current address in binary

    while let Some(&opcode) = bytes.next() {
        let opcode: Opcode = opcode.try_into()?;
        let size = opcode.size_of_ins()?;

        // Expect another byte in the input stream and error with unexpected
        // end of input if no more bytes.
        let mut expect_operand = || bytes.next().ok_or(Error::UnexpectedEndOfFile(opcode));

        let ins = match size {
            // opcode + no operands
            InstrSize::OneByte => Instr(addr, opcode, Zero),
            // opcode + single operand
            InstrSize::TwoByte => {
                let operand = *expect_operand()?;

                match opcode {
                    // If the instruction is a jump (needs labelss)
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
            // opcode + two operands
            InstrSize::ThreeByte => {
                let operand1 = *expect_operand()?;
                let operand2 = *expect_operand()?;
                Instr(addr, opcode, Two(operand1, operand2))
            }
        };

        instrs.push(ins);
        addr += size.to_number();
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

        addr += opcode.size_of_ins()?.to_number();
        with_labels.push(ins.clone());
    }

    Ok(with_labels)
}
