use super::{elf::ElfHeader, pe::PEHeader};

#[derive(Debug)]
pub enum Header
{
    Elf(ElfHeader),
    PE(PEHeader),
    None,
}

impl Header
{
    pub fn parse_header(bytes: &[u8]) -> Header
    {
        let elf_header = ElfHeader::parse_header(bytes);
        match elf_header
        {
            Some(header) => return Header::Elf(header),
            None => {},
        };
        let pe_header = PEHeader::parse_header(bytes);
        match pe_header
        {
            Some(header) => return Header::PE(header),
            None => {},
        };
        Header::None
    }

    pub fn bitness(&self) -> u32
    {
        match self
        {
            Header::Elf(header) => header.bitness(),
            Header::PE(header) => header.bitness(),
            Header::None => 64,
        }
    }

    pub fn entry_point(&self) -> u64
    {
        match self
        {
            Header::Elf(header) => header.entry_point,
            Header::PE(header) => header.optional_header.address_of_entry_point as u64,
            Header::None => 0,
        }
    }
}