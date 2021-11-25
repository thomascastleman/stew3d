use crate::instr::Instruction::{self, *};
use std::fmt;

/// `BinaryStats` contains information about a given binary, such as:
///   - Number of instructions
///   - Size of program (bytes)
///   - Breakdown of bytes between opcodes/operands
///   - Breakdown of one-/two-/three-byte instructions
pub struct BinaryStats {
    total_instrs: usize,
    total_bytes: usize,
    opcode_bytes: usize,
    operand_bytes: usize,
    single_byte_instrs: usize,
    two_byte_instrs: usize,
    three_byte_instrs: usize,
}

impl BinaryStats {
    /// Analyzes the given program to collect the statistics found in a `BinaryStats` struct.
    pub fn new(instrs: &[Instruction]) -> Self {
        let sum_up = |f: fn(&Instruction) -> usize| instrs.iter().map(f).sum();
        let count_instrs = |pred: fn(&&Instruction) -> bool| instrs.iter().filter(pred).count();

        BinaryStats {
            total_instrs: sum_up(|ins| ins.size()),
            total_bytes: count_instrs(|ins| !matches!(ins, Label(_, _))),
            opcode_bytes: sum_up(|ins| ins.num_opcodes()),
            operand_bytes: sum_up(|ins| ins.num_operands()),
            single_byte_instrs: count_instrs(|ins| ins.size() == 1),
            two_byte_instrs: count_instrs(|ins| ins.size() == 2),
            three_byte_instrs: count_instrs(|ins| ins.size() == 3),
        }
    }
}

impl fmt::Display for BinaryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let percentage = |num: usize, denom: usize| (num as f64 / denom as f64) * 100.0;
        writeln!(f, "Program size: {} bytes", self.total_bytes)?;
        writeln!(f, "Instructions: {}", self.total_instrs)?;
        writeln!(
            f,
            "Opcodes:      {} ({:.2}%)",
            self.opcode_bytes,
            percentage(self.opcode_bytes, self.total_bytes),
        )?;
        writeln!(
            f,
            "Operands:     {} ({:.2}%)",
            self.operand_bytes,
            percentage(self.operand_bytes, self.total_bytes),
        )?;

        writeln!(f, "Instruction breakdown:")?;
        writeln!(
            f,
            "  1-byte: {} ({:.2}%)",
            self.single_byte_instrs,
            percentage(self.single_byte_instrs, self.total_instrs),
        )?;
        writeln!(
            f,
            "  2-byte: {} ({:.2}%)",
            self.two_byte_instrs,
            percentage(self.two_byte_instrs, self.total_instrs),
        )?;
        writeln!(
            f,
            "  3-byte: {} ({:.2}%)",
            self.three_byte_instrs,
            percentage(self.three_byte_instrs, self.total_instrs),
        )
    }
}
