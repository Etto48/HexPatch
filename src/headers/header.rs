use std::{collections::HashMap, fmt::Display};

use capstone::{arch::{self, BuildsCapstone}, Capstone, CsResult};
use keystone_engine::{Arch, Keystone, KeystoneError, Mode};
use object::Architecture;

use super::generic::GenericHeader;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section
{
    pub name: String,
    pub virtual_address: u64,
    pub file_offset: u64,
    pub size: u64,
}

impl Display for Section
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{}: [{:X} - {:X})", self.name, self.file_offset, self.file_offset + self.size)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Header
{
    GenericHeader(GenericHeader),
    None,
}

impl Header
{
    pub fn parse_header(bytes: &[u8]) -> Header
    {
        let header = GenericHeader::parse_header(bytes);
        match header
        {
            Some(header) => Header::GenericHeader(header),
            None => Header::None,
        }
    }

    pub fn bitness(&self) -> u32
    {
        match self
        {
            Header::GenericHeader(header) => match header.bitness
            {
                super::generic::Bitness::Bit32 => 32,
                super::generic::Bitness::Bit64 => 64,
            },
            Header::None => 64,
        }
    }

    pub fn entry_point(&self) -> u64
    {
        match self
        {
            Header::GenericHeader(header) => header.entry,
            Header::None => 0,
        }
    }

    pub fn architecture(&self) -> Architecture
    {
        match self
        {
            Header::GenericHeader(header) => header.architecture,
            Header::None => Architecture::Unknown,
        }
    }

    pub fn get_sections(&self) -> Vec<Section>
    {
        match self
        {
            Header::GenericHeader(header) => header.sections.clone(),
            Header::None => Vec::new(),
        }
    }

    pub fn get_text_section(&self) -> Option<Section>
    {
        match self
        {
            Header::GenericHeader(header) => 
            {
                for section in &header.sections
                {
                    if section.name == ".text" || section.name == "__text"
                    {
                        return Some(section.clone())
                    }
                }
                None
            },
            Header::None => None,
        }
    }

    pub fn get_symbols(&self) -> Option<&HashMap<u64,String>>
    {
        match self
        {
            Header::GenericHeader(header) => Some(&header.symbols),
            Header::None => None,
        }
    }

    pub fn symbol_to_address(&self, symbol: &str) -> Option<u64>
    {
        match self
        {
            Header::GenericHeader(header) => header.symbols_by_name.get(symbol).cloned(),
            Header::None => None,
        }
    }

    pub fn virtual_to_physical_address(&self, virtual_address: u64) -> Option<u64>
    {
        self.get_sections()
            .iter()
            .find(|x| virtual_address >= x.virtual_address && virtual_address < x.virtual_address + x.size)
            .map(|x| x.file_offset + virtual_address - x.virtual_address)
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

    pub(super) fn get_encoder_for_arch(architecture: &Architecture) -> Result<Keystone, KeystoneError>
    {
        match architecture
        {
            Architecture::Aarch64 =>
            {
                Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN)
            },
            Architecture::Aarch64_Ilp32 => 
            {
                Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN)
            },
            Architecture::Arm => 
            {
                Keystone::new(Arch::ARM, Mode::ARM)
            },
            Architecture::I386 => 
            {
                Keystone::new(Arch::X86, Mode::MODE_32)
            },
            Architecture::X86_64 => 
            {
                Keystone::new(Arch::X86, Mode::MODE_64)
            },
            Architecture::X86_64_X32 => 
            {
                Keystone::new(Arch::X86, Mode::MODE_32)
            },
            Architecture::Hexagon => 
            {
                Keystone::new(Arch::HEXAGON, Mode::MODE_32)
            },
            Architecture::Mips => 
            {
                Keystone::new(Arch::MIPS, Mode::MIPS32)
            },
            Architecture::Mips64 => 
            {
                Keystone::new(Arch::MIPS, Mode::MIPS64)
            },
            Architecture::PowerPc => 
            {
                Keystone::new(Arch::PPC, Mode::PPC32)
            },
            Architecture::PowerPc64 => 
            {
                Keystone::new(Arch::PPC, Mode::PPC64)
            },
            Architecture::S390x => 
            {
                Keystone::new(Arch::SYSTEMZ, Mode::MODE_32)
            },
            Architecture::Sparc64 => 
            {
                Keystone::new(Arch::SPARC, Mode::SPARC64)
            },
            _ => Keystone::new(Arch::X86, Mode::MODE_64),
        }
    }

    pub fn get_decoder(&self) -> CsResult<Capstone>
    {
        let ret = match self
        {
            Header::GenericHeader(header) => Self::get_decoder_for_arch(&header.architecture),
            Header::None => Capstone::new().x86().mode(capstone::arch::x86::ArchMode::Mode64).build(),
        };
        ret.map(|mut cs| {
            cs.set_skipdata(true).expect("Failed to set skipdata");
            cs
        })
    }

    pub fn get_encoder(&self) -> Result<Keystone, KeystoneError>
    {
        match self {
            Header::GenericHeader(header) => Self::get_encoder_for_arch(&header.architecture),
            Header::None => Keystone::new(Arch::X86, Mode::MODE_64),
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::headers::generic::FileType;

    use super::*;
    #[test]
    fn test_parse_elf()
    {
        let data = include_bytes!("../../test/elf.bin");
        let header = Header::parse_header(data);
        if let Header::GenericHeader(header) = header
        {
            assert_eq!(header.file_type, FileType::Elf64);
            assert_eq!(header.architecture, Architecture::X86_64);
            assert_eq!(header.endianness, object::Endianness::Little);
        }
        else
        {
            panic!("Failed to parse ELF header.");
        }
    }

    #[test]
    fn test_parse_pe()
    {
        let data = include_bytes!("../../test/pe.bin");
        let header = Header::parse_header(data);
        if let Header::GenericHeader(header) = header
        {
            assert_eq!(header.file_type, FileType::Pe64);
            assert_eq!(header.architecture, Architecture::X86_64);
            assert_eq!(header.endianness, object::Endianness::Little);
        }
        else
        {
            panic!("Failed to parse PE header.");
        }
    }

    #[test]
    fn test_parse_macho()
    {
        let data = include_bytes!("../../test/macho.bin");
        let header = Header::parse_header(data);
        if let Header::GenericHeader(header) = header
        {
            assert_eq!(header.file_type, FileType::MachO64);
            assert_eq!(header.architecture, Architecture::X86_64);
            assert_eq!(header.endianness, object::Endianness::Little);
        }
        else
        {
            panic!("Failed to parse Mach-O header.");
        }
    }
}