#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionTag
{
    pub name: String,
    pub file_address: u64,
    pub virtual_address: u64,
    pub size: usize
}