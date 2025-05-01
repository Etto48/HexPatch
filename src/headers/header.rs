use std::collections::HashMap;

use capstone::{
    arch::{self, BuildsCapstone},
    Capstone, CsResult,
};
use hexpatch_keystone::{Arch, Error, Keystone, Mode};
use mlua::UserData;
use object::{Architecture, Endianness};

use crate::app::files::filesystem::FileSystem;

use super::{
    bitness::Bitness, custom_header::CustomHeader, generic::GenericHeader, section::Section,
};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum Header {
    GenericHeader(GenericHeader),
    CustomHeader(CustomHeader),
    #[default]
    None,
}

impl Header {
    pub fn parse_header(bytes: &[u8], file_path: &str, filesystem: &FileSystem) -> Header {
        let header = GenericHeader::parse_header(bytes, file_path, filesystem);
        match header {
            Some(header) => Header::GenericHeader(header),
            None => Header::None,
        }
    }

    pub fn bitness(&self) -> u32 {
        match self {
            Header::GenericHeader(header) => match header.bitness {
                Bitness::Bit32 => 32,
                Bitness::Bit64 => 64,
            },
            Header::CustomHeader(header) => match header.bitness {
                Bitness::Bit32 => 32,
                Bitness::Bit64 => 64,
            },
            Header::None => 64,
        }
    }

    pub fn endianness(&self) -> Endianness {
        match self {
            Header::GenericHeader(header) => header.endianness,
            Header::CustomHeader(header) => header.endianness,
            Header::None => object::Endianness::Little,
        }
    }

    pub fn entry_point(&self) -> u64 {
        match self {
            Header::GenericHeader(header) => header.entry,
            Header::CustomHeader(header) => header.entry,
            Header::None => 0,
        }
    }

    pub fn architecture(&self) -> Architecture {
        match self {
            Header::GenericHeader(header) => header.architecture,
            Header::CustomHeader(header) => header.architecture,
            Header::None => Architecture::Unknown,
        }
    }

    pub fn get_sections(&self) -> Vec<Section> {
        match self {
            Header::GenericHeader(header) => header.sections.clone(),
            Header::CustomHeader(header) => header.sections.clone(),
            Header::None => Vec::new(),
        }
    }

    pub fn get_text_section(&self) -> Option<Section> {
        match self {
            Header::GenericHeader(header) => {
                for section in &header.sections {
                    if section.name == ".text" || section.name == "__text" {
                        return Some(section.clone());
                    }
                }
                None
            }
            Header::CustomHeader(header) => {
                for section in &header.sections {
                    if section.name == ".text" || section.name == "__text" {
                        return Some(section.clone());
                    }
                }
                None
            }
            Header::None => None,
        }
    }

    pub fn get_symbols(&self) -> Option<&HashMap<u64, String>> {
        match self {
            Header::GenericHeader(header) => Some(&header.symbols),
            Header::CustomHeader(header) => Some(&header.symbols),
            Header::None => None,
        }
    }

    pub fn symbol_to_address(&self, symbol: &str) -> Option<u64> {
        match self {
            Header::GenericHeader(header) => header.symbols_by_name.get(symbol).cloned(),
            Header::CustomHeader(header) => header.symbols_by_name.get(symbol).cloned(),
            Header::None => None,
        }
    }

    pub fn virtual_to_physical_address(&self, virtual_address: u64) -> Option<u64> {
        self.get_sections()
            .iter()
            .find(|x| {
                virtual_address >= x.virtual_address && virtual_address < x.virtual_address + x.size
            })
            .map(|x| x.file_offset + virtual_address - x.virtual_address)
    }

    pub fn physical_to_virtual_address(&self, physical_address: u64) -> Option<u64> {
        self.get_sections()
            .iter()
            .find(|x| {
                physical_address >= x.file_offset && physical_address < x.file_offset + x.size
            })
            .map(|x| x.virtual_address + physical_address - x.file_offset)
    }

    pub(super) fn get_decoder_for_arch(architecture: &Architecture) -> CsResult<Capstone> {
        match architecture {
            Architecture::Aarch64 => Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .build(),
            Architecture::Aarch64_Ilp32 => Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .build(),
            Architecture::Arm => Capstone::new().arm().mode(arch::arm::ArchMode::Arm).build(),
            Architecture::I386 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode32)
                .build(),
            Architecture::X86_64 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .build(),
            Architecture::X86_64_X32 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .build(),
            Architecture::Mips => Capstone::new()
                .mips()
                .mode(arch::mips::ArchMode::Mips32)
                .build(),
            Architecture::Mips64 => Capstone::new()
                .mips()
                .mode(arch::mips::ArchMode::Mips64)
                .build(),
            Architecture::PowerPc => Capstone::new()
                .ppc()
                .mode(arch::ppc::ArchMode::Mode32)
                .build(),
            Architecture::PowerPc64 => Capstone::new()
                .ppc()
                .mode(arch::ppc::ArchMode::Mode64)
                .build(),
            Architecture::Riscv32 => Capstone::new()
                .riscv()
                .mode(arch::riscv::ArchMode::RiscV32)
                .build(),
            Architecture::Riscv64 => Capstone::new()
                .riscv()
                .mode(arch::riscv::ArchMode::RiscV64)
                .build(),
            Architecture::S390x => Capstone::new()
                .sysz()
                .mode(arch::sysz::ArchMode::Default)
                .build(),
            Architecture::Sparc64 => Capstone::new()
                .sparc()
                .mode(arch::sparc::ArchMode::V9)
                .build(),
            _ => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .build(),
        }
    }

    pub(super) fn get_encoder_for_arch(architecture: &Architecture) -> Result<Keystone, Error> {
        match architecture {
            Architecture::Aarch64 => Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN),
            Architecture::Aarch64_Ilp32 => Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN),
            Architecture::Arm => Keystone::new(Arch::ARM, Mode::ARM),
            Architecture::I386 => Keystone::new(Arch::X86, Mode::MODE_32),
            Architecture::X86_64 => Keystone::new(Arch::X86, Mode::MODE_64),
            Architecture::X86_64_X32 => Keystone::new(Arch::X86, Mode::MODE_32),
            Architecture::Hexagon => Keystone::new(Arch::HEXAGON, Mode::MODE_32),
            Architecture::Mips => Keystone::new(Arch::MIPS, Mode::MIPS32),
            Architecture::Mips64 => Keystone::new(Arch::MIPS, Mode::MIPS64),
            Architecture::PowerPc => Keystone::new(Arch::PPC, Mode::PPC32),
            Architecture::PowerPc64 => Keystone::new(Arch::PPC, Mode::PPC64),
            Architecture::S390x => Keystone::new(Arch::SYSTEMZ, Mode::MODE_32),
            Architecture::Sparc64 => Keystone::new(Arch::SPARC, Mode::SPARC64),
            _ => Keystone::new(Arch::X86, Mode::MODE_64),
        }
    }

    pub fn get_decoder(&self) -> CsResult<Capstone> {
        let ret = match self {
            Header::GenericHeader(header) => Self::get_decoder_for_arch(&header.architecture),
            Header::CustomHeader(header) => Self::get_decoder_for_arch(&header.architecture),
            Header::None => Capstone::new()
                .x86()
                .mode(capstone::arch::x86::ArchMode::Mode64)
                .build(),
        };
        ret.map(|mut cs| {
            cs.set_skipdata(true).expect("Failed to set skipdata");
            cs
        })
    }

    pub fn get_encoder(&self) -> Result<Keystone, Error> {
        match self {
            Header::GenericHeader(header) => Self::get_encoder_for_arch(&header.architecture),
            Header::CustomHeader(header) => Self::get_encoder_for_arch(&header.architecture),
            Header::None => Keystone::new(Arch::X86, Mode::MODE_64),
        }
    }
}

impl UserData for Header {
    fn add_fields<'lua, F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("bitness", |_, this| Ok(this.bitness()));
        fields.add_field_method_get("entry_point", |_, this| Ok(this.entry_point()));
        fields.add_field_method_get("architecture", |_, this| {
            Ok(format!("{:?}", this.architecture()))
        });
        fields.add_field_method_get("sections", |_, this| Ok(this.get_sections()));
        fields.add_field_method_get("text_section", |_, this| Ok(this.get_text_section()));
        fields.add_field_method_get("symbols", |_, this| {
            Ok(this
                .get_symbols()
                .map(|x| x.values().cloned().collect::<Vec<_>>()))
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("symbol_to_address", |_, this, symbol: String| {
            Ok(this.symbol_to_address(&symbol))
        });
        methods.add_method(
            "virtual_to_physical_address",
            |_, this, virtual_address: u64| Ok(this.virtual_to_physical_address(virtual_address)),
        );
    }
}

#[cfg(test)]
mod test {
    use crate::headers::generic::FileType;

    use super::*;
    #[test]
    fn test_parse_elf() {
        let data = include_bytes!("../../test/elf.bin");
        let header = Header::parse_header(data, "./elf.bin", &FileSystem::new_local(".").unwrap());
        if let Header::GenericHeader(header) = &header {
            assert_eq!(header.file_type, FileType::Elf64);
        } else {
            panic!("Failed to parse ELF header.");
        }
        assert_eq!(header.architecture(), Architecture::X86_64);
        assert_eq!(header.bitness(), 64);
        assert_eq!(header.endianness(), Endianness::Little);
    }

    #[test]
    fn test_parse_pe() {
        let data = include_bytes!("../../test/pe.bin");
        let header = Header::parse_header(data, "./pe.bin", &FileSystem::new_local(".").unwrap());
        if let Header::GenericHeader(header) = &header {
            assert_eq!(header.file_type, FileType::Pe64);
        } else {
            panic!("Failed to parse PE header.");
        }
        assert_eq!(header.architecture(), Architecture::X86_64);
        assert_eq!(header.bitness(), 64);
        assert_eq!(header.endianness(), Endianness::Little);
    }

    #[test]
    fn test_parse_macho() {
        let data = include_bytes!("../../test/macho.bin");
        let header =
            Header::parse_header(data, "./macho.bin", &FileSystem::new_local(".").unwrap());
        if let Header::GenericHeader(header) = &header {
            assert_eq!(header.file_type, FileType::MachO64);
        } else {
            panic!("Failed to parse Mach-O header.");
        }
        assert_eq!(header.architecture(), Architecture::X86_64);
        assert_eq!(header.bitness(), 64);
        assert_eq!(header.endianness(), Endianness::Little);
    }
}
