use crate::Opcode;
use std::fmt;
use Instruction::*;
use Opcode::*;
use Operands::*;

/// Encodes the operands of an instruction. Currently, instructions can have
/// between 0-2 single-byte operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operands {
    Zero,
    One(u8),
    Two(u8, u8),
}

/// An instruction that has been reconstructed via disassembly. For the purposes
/// of turning raw addresses in jump instructions into labels, this type is
/// split into three variants:
///
/// - `Label` represents a label that has been inserted by the disassembler.
/// - `Jump` represents any instruction which requires a jump target.
/// - `Instr` represents all other instructions.
///
/// The first field of each variant is a `usize` address indicating where
/// in the program the instruction/label occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// `Label` contains an address and a name for the label.
    Label(usize, String),
    /// `Jump` contains an address, an opcode (of the jump), a target address, and a target label.
    Jump(usize, Opcode, u8, String),
    /// `Instr` contains an address, an opcode, and operands.
    Instr(usize, Opcode, Operands),
}

impl Instruction {
    /// Extracts the address in the binary of a given instruction. Labels, jumps
    /// and other instructions all have this component, so this is defined
    /// for any instruction.
    pub fn addr(&self) -> usize {
        match self {
            Label(addr, _) | Jump(addr, _, _, _) | Instr(addr, _, _) => *addr,
        }
    }

    /// Convert an instruction into the sequence of bytes used to represent it.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Label(_, _) => Vec::new(),
            Jump(_, op, target, _) => vec![*op as u8, *target],
            Instr(_, op, operands) => {
                let op = *op as u8;
                match operands {
                    Zero => vec![op],
                    One(first) => vec![op, *first],
                    Two(first, second) => vec![op, *first, *second],
                }
            }
        }
    }

    /// Determines the number of bytes to encode this instruction.
    pub fn size(&self) -> usize {
        self.to_bytes().len()
    }

    /// Determines the number of operands in this instruction.
    pub fn num_operands(&self) -> usize {
        match self {
            Label(_, _) => 0,      // labels have no operands
            Jump(_, _, _, _) => 1, // the jump target
            Instr(_, _, operands) => match operands {
                Zero => 0,
                One(_) => 1,
                Two(_, _) => 2,
            },
        }
    }

    /// Determines the number of opcodes in this instruction. Really, all
    /// instructions have 1 opcode, but this ensures that labels don't count
    /// as having opcodes.
    pub fn num_opcodes(&self) -> usize {
        match self {
            Label(_, _) => 0,
            _ => 1,
        }
    }
}

/// The tab character that is used to indent instructions in the disassembly.
const TAB: &str = "  ";

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Label(_, name) => write!(f, "{}:", name),
            Jump(_, op, _, target) => {
                let jmp = match op {
                    JMP => "jmp",
                    JE => "je",
                    JNE => "jne",
                    JG => "jg",
                    JGE => "jge",
                    JL => "jl",
                    JLE => "jle",
                    JA => "ja",
                    JAE => "jae",
                    JB => "jb",
                    JBE => "jbe",
                    CALL => "call",
                    _ => unreachable!(),
                };
                write!(f, "{}{} {}", TAB, jmp, target)
            }
            Instr(_, op, operands) => {
                let str = match operands {
                    Zero => match op {
                        ADD_A_A => "add a, a",
                        ADD_A_B => "add a, b",
                        ADD_A_C => "add a, c",
                        ADD_A_SP => "add a, sp",
                        ADD_B_A => "add b, a",
                        ADD_B_B => "add b, b",
                        ADD_B_C => "add b, c",
                        ADD_B_SP => "add b, sp",
                        ADD_C_A => "add c, a",
                        ADD_C_B => "add c, b",
                        ADD_C_C => "add c, c",
                        ADD_C_SP => "add c, sp",

                        ADDC_A_A => "addc a, a",
                        ADDC_A_B => "addc a, b",
                        ADDC_A_C => "addc a, c",
                        ADDC_A_SP => "addc a, sp",
                        ADDC_B_A => "addc b, a",
                        ADDC_B_B => "addc b, b",
                        ADDC_B_C => "addc b, c",
                        ADDC_B_SP => "addc b, sp",
                        ADDC_C_A => "addc c, a",
                        ADDC_C_B => "addc c, b",
                        ADDC_C_C => "addc c, c",
                        ADDC_C_SP => "addc c, sp",

                        SUB_B_A => "sub b, a",
                        SUB_C_A => "sub c, a",
                        SUB_A_B => "sub a, b",
                        SUB_C_B => "sub c, b",
                        SUB_A_C => "sub a, c",
                        SUB_B_C => "sub b, c",
                        SUB_A_SP => "sub a, sp",
                        SUB_B_SP => "sub b, sp",
                        SUB_C_SP => "sub c, sp",

                        SUBB_B_A => "subb b, a",
                        SUBB_C_A => "subb c, a",
                        SUBB_A_B => "subb a, b",
                        SUBB_C_B => "subb c, b",
                        SUBB_A_C => "subb a, c",
                        SUBB_B_C => "subb b, c",
                        SUBB_A_SP => "subb a, sp",
                        SUBB_B_SP => "subb b, sp",
                        SUBB_C_SP => "subb c, sp",

                        AND_B_A => "and b, a",
                        AND_C_A => "and c, a",
                        AND_A_B => "and a, b",
                        AND_C_B => "and c, b",
                        AND_A_C => "and a, c",
                        AND_B_C => "and b, c",

                        OR_B_A => "or b, a",
                        OR_C_A => "or c, a",
                        OR_A_B => "or a, b",
                        OR_C_B => "or c, b",
                        OR_A_C => "or a, c",
                        OR_B_C => "or b, c",

                        XOR_B_A => "xor b, a",
                        XOR_C_A => "xor c, a",
                        XOR_A_B => "xor a, b",
                        XOR_C_B => "xor c, b",
                        XOR_A_C => "xor a, c",
                        XOR_B_C => "xor b, c",

                        NOT_A => "not a",
                        NOT_B => "not b",
                        NOT_C => "not c",

                        NEG_A => "neg a",
                        NEG_B => "neg b",
                        NEG_C => "neg c",

                        INR_A => "inr a",
                        INR_B => "inr b",
                        INR_C => "inr c",
                        INR_SP => "inr sp",

                        INR2_A => "inr2 a",
                        INR2_B => "inr2 b",
                        INR2_C => "inr2 c",
                        INR2_SP => "inr2 sp",

                        INR3_A => "inr3 a",
                        INR3_B => "inr3 b",
                        INR3_C => "inr3 c",
                        INR3_SP => "inr3 sp",

                        DCR_A => "dcr a",
                        DCR_B => "dcr b",
                        DCR_C => "dcr c",
                        DCR_SP => "dcr sp",

                        DCR2_A => "dcr2 a",
                        DCR2_B => "dcr2 b",
                        DCR2_C => "dcr2 c",
                        DCR2_SP => "dcr2 sp",

                        DCR3_A => "dcr3 a",
                        DCR3_B => "dcr3 b",
                        DCR3_C => "dcr3 c",
                        DCR3_SP => "dcr3 sp",

                        MOV_A_B => "mov a, b",
                        MOV_A_C => "mov a, c",
                        MOV_B_A => "mov b, a",
                        MOV_B_C => "mov b, c",
                        MOV_C_A => "mov c, a",
                        MOV_C_B => "mov c, b",
                        MOV_Z_A => "mov z, a",
                        MOV_Z_B => "mov z, b",
                        MOV_Z_C => "mov z, c",
                        MOV_SP_A => "mov sp, a",
                        MOV_SP_B => "mov sp, b",
                        MOV_SP_C => "mov sp, c",

                        LD_A_A => "ld a, a",
                        LD_B_A => "ld b, a",
                        LD_C_A => "ld c, a",
                        LD_A_B => "ld a, b",
                        LD_B_B => "ld b, b",
                        LD_C_B => "ld c, b",
                        LD_A_C => "ld a, c",
                        LD_B_C => "ld b, c",
                        LD_C_C => "ld c, c",

                        ST_A_A => "st a, a",
                        ST_A_B => "st a, b",
                        ST_A_C => "st a, c",
                        ST_B_A => "st b, a",
                        ST_B_B => "st b, b",
                        ST_B_C => "st b, c",
                        ST_C_A => "st c, a",
                        ST_C_B => "st c, b",
                        ST_C_C => "st c, c",
                        ST_Z_A => "st z, a",
                        ST_Z_B => "st z, b",
                        ST_Z_C => "st z, c",

                        CMP_A_B => "cmp a, b",
                        CMP_A_C => "cmp a, c",
                        CMP_A_Z => "cmp a, z",
                        CMP_B_A => "cmp b, a",
                        CMP_B_C => "cmp b, c",
                        CMP_B_Z => "cmp b, z",
                        CMP_C_A => "cmp c, a",
                        CMP_C_B => "cmp c, b",
                        CMP_C_Z => "cmp c, z",
                        CMP_Z_A => "cmp z, a",
                        CMP_Z_B => "cmp z, b",
                        CMP_Z_C => "cmp z, c",

                        RET => "ret",

                        OUT_A => "out a",
                        OUT_B => "out b",
                        OUT_C => "out c",

                        DD_A => "dd a",
                        DD_B => "dd b",
                        DD_C => "dd c",

                        HLT => "hlt",
                        NOP => "nop",

                        _ => unreachable!(),
                    }
                    .into(),
                    One(first) => match op {
                        ADDI_A => format!("addi {}, a", first),
                        ADDI_B => format!("addi {}, b", first),
                        ADDI_C => format!("addi {}, c", first),
                        ADDI_SP => format!("addi {}, sp", first),

                        ADDCI_A => format!("addci {}, a", first),
                        ADDCI_B => format!("addci {}, b", first),
                        ADDCI_C => format!("addci {}, c", first),
                        ADDCI_SP => format!("addci {}, sp", first),

                        SUBI_A => format!("subi {}, a", first),
                        SUBI_B => format!("subi {}, b", first),
                        SUBI_C => format!("subi {}, c", first),
                        SUBI_SP => format!("subi {}, sp", first),

                        SUBBI_A => format!("subbi {}, a", first),
                        SUBBI_B => format!("subbi {}, b", first),
                        SUBBI_C => format!("subbi {}, c", first),
                        SUBBI_SP => format!("subbi {}, sp", first),

                        ANI_A => format!("ani {}, a", first),
                        ANI_B => format!("ani {}, b", first),
                        ANI_C => format!("ani {}, c", first),

                        ORI_A => format!("ori {}, a", first),
                        ORI_B => format!("ori {}, b", first),
                        ORI_C => format!("ori {}, c", first),

                        XRI_A => format!("xri {}, a", first),
                        XRI_B => format!("xri {}, b", first),
                        XRI_C => format!("xri {}, c", first),

                        MVI_A => format!("mvi {}, a", first),
                        MVI_B => format!("mvi {}, b", first),
                        MVI_C => format!("mvi {}, c", first),

                        LDS_A => format!("lds {}, a", first),
                        LDS_B => format!("lds {}, b", first),
                        LDS_C => format!("lds {}, c", first),

                        STS_A => format!("sts a, {}", first),
                        STS_B => format!("sts b, {}", first),
                        STS_C => format!("sts c, {}", first),
                        STS_Z => format!("sts z, {}", first),

                        CMPI_A_BYTE => format!("cmpi a, {}", first),
                        CMPI_BYTE_A => format!("cmpi {}, a", first),
                        CMPI_B_BYTE => format!("cmpi b, {}", first),
                        CMPI_BYTE_B => format!("cmpi {}, b", first),
                        CMPI_C_BYTE => format!("cmpi c, {}", first),
                        CMPI_BYTE_C => format!("cmpi {}, c", first),

                        OUTI => format!("outi {}", first),
                        DIC => format!("dic {}", first),
                        DID => format!("did {}", first),
                        _ => unreachable!(),
                    },
                    Two(first, second) => match op {
                        STSI => format!("stsi {}, {}", first, second),
                        _ => unreachable!(),
                    },
                };

                write!(f, "{}{}", TAB, str)
            }
        }
    }
}
