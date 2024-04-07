use std::fmt::Display;

use capstone::Insn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    mnemonic: String,
    operands: String,
    virtual_address: u64,
    bytes: Vec<u8>,
}

impl Instruction {
    pub fn new(instruction: &Insn) -> Self {
        let mnemonic = instruction.mnemonic().expect("Failed to get mnemonic");
        let operands = instruction.op_str().expect("Failed to get operands");
        let virtual_address = instruction.address();
        let bytes = instruction.bytes().to_vec();
        Instruction {
            mnemonic: mnemonic.to_string(),
            operands: operands.to_string(),
            virtual_address,
            bytes,
        }
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn operands(&self) -> &str {
        &self.operands
    }

    pub fn ip(&self) -> u64 {
        self.virtual_address
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.mnemonic, self.operands)
    }
}