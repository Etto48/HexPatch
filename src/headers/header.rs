use std::{collections::HashMap, fmt::Display, rc::Rc};

use capstone::{arch::{self, BuildsCapstone}, Capstone, CsResult};
use object::Architecture;

use super::{elf::ElfHeader, pe::PEHeader};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section
{
    pub name: String,
    pub virtual_address: u64,
    pub address: u64,
    pub size: u64,
}

impl Display for Section
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{}: [{:X} - {:X})", self.name, self.address, self.address + self.size)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
            Header::PE(header) => header.entry_point,
            Header::None => 0,
        }
    }

    pub fn get_sections(&self) -> Vec<Section>
    {
        match self
        {
            Header::Elf(header) => 
            {
                let mut sections = Vec::new();
                for section in &header.section_table
                {
                    sections.push(Section
                    {
                        name: section.name.clone(),
                        virtual_address: section.address as u64,
                        address: section.offset as u64,
                        size: section.size as u64,
                    })
                }
                sections
            
            },
            Header::PE(header) => 
            {
                let mut sections = Vec::new();
                for section in &header.section_table
                {
                    sections.push(Section
                    {
                        name: section.name.clone(),
                        virtual_address: section.virtual_address as u64,
                        address: section.pointer_to_raw_data as u64,
                        size: section.size_of_raw_data as u64,
                    })
                }
                sections
            },
            Header::None => Vec::new(),
        }
    }

    pub fn get_text_section(&self) -> Option<Section>
    {
        match self
        {
            Header::Elf(header) => 
            {
                for section in &header.section_table
                {
                    if section.name == ".text"
                    {
                        return Some(Section
                        {
                            name: section.name.clone(),
                            virtual_address: section.address as u64,
                            address: section.offset as u64,
                            size: section.size as u64,
                        });
                    }
                }
                None
            },
            Header::PE(header) => 
            {
                for section in &header.section_table
                {
                    if section.name == ".text"
                    {
                        return Some(Section
                        {
                            name: section.name.clone(),
                            virtual_address: section.virtual_address as u64,
                            address: section.pointer_to_raw_data as u64,
                            size: section.size_of_raw_data as u64,
                        });
                    }
                }
                None
            },
            Header::None => None,
        }
    }

    pub fn get_symbols(&self) -> Option<Rc<HashMap<u64,String>>>
    {
        match self
        {
            Header::Elf(header) => 
            {
                Some(header.get_symbols())
            },
            Header::PE(header) => 
            {
                Some(header.get_symbols())
            },
            Header::None => None,
            
        }
    }

    pub fn symbol_to_address(&self, symbol: &str) -> Option<u64>
    {
        match self
        {
            Header::Elf(header) => 
            {
                header.inverse_symbol_table.get(symbol).map(|x| *x)
            },
            Header::PE(header) => 
            {
                header.inverse_symbol_table.get(symbol).map(|x| *x)
            },
            Header::None => None,
        }
    }

    pub fn virtual_to_physical_address(&self, virtual_address: u64) -> Option<u64>
    {
        self.get_sections()
            .iter()
            .find(|x| virtual_address >= x.virtual_address && virtual_address < x.virtual_address + x.size)
            .map(|x| x.address + virtual_address - x.virtual_address)
    }

    pub(super) fn get_decoder_for_arch(architecture: &Architecture) -> CsResult<Capstone>
    {
        match architecture {
            Architecture::Aarch64 => 
            {
                Capstone::new().arm64().mode(arch::arm64::ArchMode::Arm).build()
            },
            Architecture::Aarch64_Ilp32 => 
            {
                Capstone::new().arm64().mode(arch::arm64::ArchMode::Arm).build()
            },
            Architecture::Arm => 
            {
                Capstone::new().arm().mode(arch::arm::ArchMode::Arm).build()
            },
            Architecture::I386 => 
            {
                Capstone::new().x86().mode(arch::x86::ArchMode::Mode32).build()
            },
            Architecture::X86_64 => 
            {
                Capstone::new().x86().mode(arch::x86::ArchMode::Mode64).build()
            },
            Architecture::X86_64_X32 => 
            {
                Capstone::new().x86().mode(arch::x86::ArchMode::Mode64).build()
            },
            Architecture::Mips => 
            {
                Capstone::new().mips().mode(arch::mips::ArchMode::Mips32).build()
            },
            Architecture::Mips64 => 
            {
                Capstone::new().mips().mode(arch::mips::ArchMode::Mips64).build()
            },
            Architecture::PowerPc => 
            {
                Capstone::new().ppc().mode(arch::ppc::ArchMode::Mode32).build()
            },
            Architecture::PowerPc64 => 
            {
                Capstone::new().ppc().mode(arch::ppc::ArchMode::Mode64).build()
            },
            Architecture::Riscv32 => 
            {
                Capstone::new().riscv().mode(arch::riscv::ArchMode::RiscV32).build()
            },
            Architecture::Riscv64 => 
            {
                Capstone::new().riscv().mode(arch::riscv::ArchMode::RiscV64).build()
            },
            Architecture::S390x => 
            {
                Capstone::new().sysz().mode(arch::sysz::ArchMode::Default).build()
            },
            Architecture::Sparc64 => 
            {
                Capstone::new().sparc().mode(arch::sparc::ArchMode::V9).build()
            },
            _ => Capstone::new().x86().mode(arch::x86::ArchMode::Mode64).build(),
        }
    }

    pub fn get_decoder(&self) -> CsResult<Capstone>
    {
        let ret = match self
        {
            Header::Elf(header) => 
            {
                header.get_decoder()
            },
            Header::PE(header) => 
            {
                header.get_decoder()
            },
            Header::None => Capstone::new().x86().mode(capstone::arch::x86::ArchMode::Mode64).build(),
        };
        match ret
        {
            Ok(mut cs) => {
                cs.set_skipdata(true).expect("Failed to set skipdata");
                Ok(cs)
            }
            Err(e) => Err(e),
        }
    }
}