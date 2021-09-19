use std::convert::TryFrom;

const OPCODE_MIN: u8 = 0x00;
const OPCODE_MAX: u8 = 0xc8;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    ADD_A_A = OPCODE_MIN,
    ADD_A_B,
    ADD_A_C,
    ADD_A_SP,
    ADD_B_A,
    ADD_B_B,
    ADD_B_C,
    ADD_B_SP,
    ADD_C_A,
    ADD_C_B,
    ADD_C_C,
    ADD_C_SP,

    ADDI_A,
    ADDI_B,
    ADDI_C,
    ADDI_SP,

    ADDC_A_A,
    ADDC_A_B,
    ADDC_A_C,
    ADDC_A_SP,
    ADDC_B_A,
    ADDC_B_B,
    ADDC_B_C,
    ADDC_B_SP,
    ADDC_C_A,
    ADDC_C_B,
    ADDC_C_C,
    ADDC_C_SP,

    ADDCI_A,
    ADDCI_B,
    ADDCI_C,
    ADDCI_SP,

    SUB_B_A,
    SUB_C_A,
    SUB_A_B,
    SUB_C_B,
    SUB_A_C,
    SUB_B_C,
    SUB_A_SP,
    SUB_B_SP,
    SUB_C_SP,

    SUBI_A,
    SUBI_B,
    SUBI_C,
    SUBI_SP,

    SUBB_B_A,
    SUBB_C_A,
    SUBB_A_B,
    SUBB_C_B,
    SUBB_A_C,
    SUBB_B_C,
    SUBB_A_SP,
    SUBB_B_SP,
    SUBB_C_SP,

    SUBBI_A,
    SUBBI_B,
    SUBBI_C,
    SUBBI_SP,

    AND_B_A,
    AND_C_A,
    AND_A_B,
    AND_C_B,
    AND_A_C,
    AND_B_C,

    ANI_A,
    ANI_B,
    ANI_C,

    OR_B_A,
    OR_C_A,
    OR_A_B,
    OR_C_B,
    OR_A_C,
    OR_B_C,

    ORI_A,
    ORI_B,
    ORI_C,

    XOR_B_A,
    XOR_C_A,
    XOR_A_B,
    XOR_C_B,
    XOR_A_C,
    XOR_B_C,

    XRI_A,
    XRI_B,
    XRI_C,

    NOT_A,
    NOT_B,
    NOT_C,

    NEG_A,
    NEG_B,
    NEG_C,

    INR_A,
    INR_B,
    INR_C,
    INR_SP,

    INR2_A,
    INR2_B,
    INR2_C,
    INR2_SP,

    INR3_A,
    INR3_B,
    INR3_C,
    INR3_SP,

    DCR_A,
    DCR_B,
    DCR_C,
    DCR_SP,

    DCR2_A,
    DCR2_B,
    DCR2_C,
    DCR2_SP,

    DCR3_A,
    DCR3_B,
    DCR3_C,
    DCR3_SP,

    MOV_A_B,
    MOV_A_C,
    MOV_B_A,
    MOV_B_C,
    MOV_C_A,
    MOV_C_B,
    MOV_Z_A,
    MOV_Z_B,
    MOV_Z_C,
    MOV_SP_A,
    MOV_SP_B,
    MOV_SP_C,

    MVI_A,
    MVI_B,
    MVI_C,

    LD_A_A,
    LD_B_A,
    LD_C_A,
    LD_A_B,
    LD_B_B,
    LD_C_B,
    LD_A_C,
    LD_B_C,
    LD_C_C,

    ST_A_A,
    ST_A_B,
    ST_A_C,
    ST_B_A,
    ST_B_B,
    ST_B_C,
    ST_C_A,
    ST_C_B,
    ST_C_C,
    ST_Z_A,
    ST_Z_B,
    ST_Z_C,

    LDS_A,
    LDS_B,
    LDS_C,

    STS_A,
    STS_B,
    STS_C,
    STS_Z,

    STSI,

    CMP_A_B,
    CMP_A_C,
    CMP_A_Z,
    CMP_B_A,
    CMP_B_C,
    CMP_B_Z,
    CMP_C_A,
    CMP_C_B,
    CMP_C_Z,
    CMP_Z_A,
    CMP_Z_B,
    CMP_Z_C,

    CMPI_A_BYTE,
    CMPI_BYTE_A,
    CMPI_B_BYTE,
    CMPI_BYTE_B,
    CMPI_C_BYTE,
    CMPI_BYTE_C,

    JMP,
    JE,
    JNE,
    JG,
    JGE,
    JL,
    JLE,
    JA,
    JAE,
    JB,
    JBE,
    CALL,
    RET,

    OUT_A,
    OUT_B,
    OUT_C,

    OUTI,
    DIC,
    DID,

    DD_A,
    DD_B,
    DD_C,

    HLT,
    NOP = OPCODE_MAX,
}

impl TryFrom<u8> for Opcode {
    type Error = crate::Error;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            OPCODE_MIN..=OPCODE_MAX => {
                // SAFETY: The byte is within the valid range of opcodes.
                Ok(unsafe { std::mem::transmute(byte) })
            }
            _ => Err(crate::Error::InvalidOpcode(byte)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operands {
    Zero,
    One(u8),
    Two(u8, u8),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Label(u8, String),
    Jump(Opcode, u8, String),
    Instr(Opcode, Operands),
}
