use super::{elf::ElfHeader, pe::PEHeader};

#[derive(Debug)]
pub enum Header
{
    Elf(ElfHeader),
    PE(PEHeader),
}

impl Header
{
    pub fn parse_header(bytes: &[u8]) -> Option<Header>
    {
        let elf_header = ElfHeader::parse_header(bytes);
        match elf_header
        {
            Some(header) => return Some(Header::Elf(header)),
            None => {},
        };
        let pe_header = PEHeader::parse_header(bytes);
        match pe_header
        {
            Some(header) => return Some(Header::PE(header)),
            None => {},
        };
        None
    }

    pub fn bitness(&self) -> u32
    {
        match self
        {
            Header::Elf(header) => header.bitness(),
            Header::PE(header) => header.bitness(),
        }
    }

    pub fn entry_point(&self) -> u64
    {
        match self
        {
            Header::Elf(header) => header.entry_point,
            Header::PE(header) => header.optional_header.address_of_entry_point as u64,
        }
    }
}