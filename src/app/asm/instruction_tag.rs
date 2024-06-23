use crate::app::instruction::Instruction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionTag
{
    pub instruction: Instruction,
    pub file_address: u64,
}