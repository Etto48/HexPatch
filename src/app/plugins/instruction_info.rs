use mlua::UserData;

use crate::app::asm::assembly_line::AssemblyLine;

#[derive(Debug, Clone)]
pub struct InstructionInfo
{
    pub instruction: String,
    pub physical_address: u64,
    pub virtual_address: u64,
    pub size: usize,
}

impl InstructionInfo
{
    pub fn new(instruction: impl Into<String>, physical_address: u64, virtual_address: u64, size: usize) -> Self
    {
        Self
        {
            instruction: instruction.into(),
            physical_address,
            virtual_address,
            size,
        }
    }
}

impl From<&AssemblyLine> for InstructionInfo
{
    fn from(value: &AssemblyLine) -> Self {
        match value
        {
            AssemblyLine::Instruction(i) => 
                InstructionInfo::new(
                    i.instruction.to_string(), 
                    i.file_address, 
                    i.instruction.ip(), 
                    i.instruction.len()
                ),
            AssemblyLine::SectionTag(s) =>
                InstructionInfo::new(
                    format!(".{}:",s.name),
                    s.file_address,
                    s.virtual_address,
                    s.size
                )
        }
    }
}

impl UserData for InstructionInfo 
{
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F)
    {
        fields.add_field_method_get("instruction", |_lua, this|
            Ok(this.instruction.clone())
        );
        fields.add_field_method_get("physical_address", |_lua, this|
            Ok(this.physical_address)
        );
        fields.add_field_method_get("virtual_address", |_lua, this|
            Ok(this.virtual_address)
        );
        fields.add_field_method_get("size", |_lua, this|
            Ok(this.size)
        );
    }
}