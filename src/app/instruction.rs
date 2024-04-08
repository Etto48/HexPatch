use std::{collections::HashMap, fmt::Display};

use capstone::Insn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    mnemonic: String,
    operands: String,
    virtual_address: u64,
    bytes: Vec<u8>,
}

impl Instruction {
    pub fn new(instruction: &Insn, symbols: Option<&HashMap<u64,String>>) -> Self {
        let mnemonic = instruction.mnemonic().expect("Failed to get mnemonic");
        let operands = instruction.op_str().expect("Failed to get operands");
        let operands = operands.split(", ").collect::<Vec<_>>();
        let mut operands_string = String::new();
        for (i,operand) in operands.iter().enumerate() {
            let mut found_symbol = false;
            if let Some(symbols) = &symbols {
                if let Some(operand) = operand.strip_prefix("0x")
                {
                    if let Ok(operand_address) = u64::from_str_radix(operand, 16)
                    {
                        if let Some(symbol) = symbols.get(&operand_address) {
                            found_symbol = true;
                            operands_string.push_str(symbol);
                        }
                    }
                }
            }
            if !found_symbol {
                operands_string.push_str(operand);
            }
            if i < operands.len() - 1 {
                operands_string.push_str(", ");
            }
        }
        let virtual_address = instruction.address();
        let bytes = instruction.bytes().to_vec();
        Instruction {
            mnemonic: mnemonic.to_string(),
            operands: operands_string,
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