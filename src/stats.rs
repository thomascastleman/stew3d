use crate::instr::Instruction::{self, *};
use std::fmt;

/// `BinaryStats` contains information about a given binary, such as:
///   - Number of instructions
///   - Size of program (bytes)
///   - Breakdown of bytes between opcodes/operands
///   - Breakdown of one-/two-/three-byte instructions
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
            total_instrs: count_instrs(|ins| !matches!(ins, Label(_, _))),
            total_bytes: sum_up(|ins| ins.size()),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::instr::Operands::*;
    use crate::opcode::Opcode::*;

    #[test]
    fn small_program() {
        // 00:    7f ff    |   mvi 255, a
        // 02:             | l0:
        // 02:    be       |   out a
        // 03:    67       |   dcr a
        // 04:    a1       |   cmp a, z
        // 05:    b3 02    |   jne l0
        let bytes = [
            Instr(0x00, MVI_A, One(0xff)),
            Label(0x02, "l0".into()),
            Instr(0x02, OUT_A, Zero),
            Instr(0x03, DCR_A, Zero),
            Instr(0x04, CMP_A_Z, Zero),
            Jump(0x05, JNE, 0x02, "l0".into()),
        ];

        let stats = BinaryStats::new(&bytes[..]);
        assert_eq!(
            stats,
            BinaryStats {
                total_instrs: 5,
                total_bytes: 7,
                opcode_bytes: 5,
                operand_bytes: 2,
                single_byte_instrs: 3,
                two_byte_instrs: 2,
                three_byte_instrs: 0,
            }
        );
    }

    #[test]
    fn empty() {
        let stats = BinaryStats::new(&[]);
        assert_eq!(
            stats,
            BinaryStats {
                total_instrs: 0,
                total_bytes: 0,
                opcode_bytes: 0,
                operand_bytes: 0,
                single_byte_instrs: 0,
                two_byte_instrs: 0,
                three_byte_instrs: 0,
            }
        );
    }
}
