use std::{collections::HashMap, io::Write};

use object::{Object, ObjectSection, ObjectSymbol};
use pdb::FallibleIterator;

use crate::app::files::{filesystem::FileSystem, path};

use super::{bitness::Bitness, section::Section};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Coff,
    CoffBig,
    Elf32,
    Elf64,
    MachO32,
    MachO64,
    Pe32,
    Pe64,
    Xcoff32,
    Xcoff64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericHeader {
    pub(super) file_type: FileType,
    pub(super) architecture: object::Architecture,
    pub(super) bitness: Bitness,
    pub(super) endianness: object::Endianness,
    pub(super) entry: u64,
    pub(super) sections: Vec<Section>,
    pub(super) symbols: HashMap<u64, String>,
    pub(super) symbols_by_name: HashMap<String, u64>,
}

impl GenericHeader {
    fn demangle_symbol(symbol: &str) -> String {
        let name = symbolic_demangle::demangle(symbol);
        name.to_string()
    }

    pub fn parse_header(bytes: &[u8], file_path: &str, filesystem: &FileSystem) -> Option<Self> {
        let header = object::File::parse(bytes);
        if let Ok(header) = header {
            let file_type = match &header {
                object::File::Coff(_) => FileType::Coff,
                object::File::CoffBig(_) => FileType::CoffBig,
                object::File::Elf32(_) => FileType::Elf32,
                object::File::Elf64(_) => FileType::Elf64,
                object::File::MachO32(_) => FileType::MachO32,
                object::File::MachO64(_) => FileType::MachO64,
                object::File::Pe32(_) => FileType::Pe32,
                object::File::Pe64(_) => FileType::Pe64,
                object::File::Xcoff32(_) => FileType::Xcoff32,
                object::File::Xcoff64(_) => FileType::Xcoff64,
                _ => return None,
            };

            let architecture = header.architecture();

            let bitness = if header.is_64() {
                Bitness::Bit64
            } else {
                Bitness::Bit32
            };

            let endianness = header.endianness();

            let mut entry = header.entry();

            let sections: Vec<Section> = header
                .sections()
                .map(|section| Section {
                    name: section.name().unwrap_or_default().to_string(),
                    file_offset: section.file_range().unwrap_or_default().0,
                    virtual_address: section.address(),
                    size: section.file_range().unwrap_or_default().1,
                })
                .filter(|section| section.size > 0)
                .collect();

            match header {
                object::File::MachO32(_) | object::File::MachO64(_) => {
                    if let Some(text) = sections.iter().find(|section| section.name == "__text") {
                        let text_offset = text.virtual_address.saturating_sub(text.file_offset);
                        entry += text_offset;
                    }
                }
                _ => {}
            }

            let mut symbols: Vec<(u64, String)> = header
                .symbols()
                .map(|symbol| {
                    (
                        symbol.address(),
                        symbol.name().unwrap_or_default().to_string(),
                    )
                })
                .collect();

            let address_base = header.relative_address_base();

            // this is used only for pe files
            let (debug_data_directory, section_table) = match &header {
                object::File::Pe32(pe) => (
                    pe.data_directory(object::pe::IMAGE_DIRECTORY_ENTRY_DEBUG),
                    Some(pe.section_table()),
                ),
                object::File::Pe64(pe) => (
                    pe.data_directory(object::pe::IMAGE_DIRECTORY_ENTRY_DEBUG),
                    Some(pe.section_table()),
                ),
                _ => (None, None),
            };

            if let Some(debug_data_directory) = debug_data_directory {
                let section_table = section_table.expect("PE file should have a section table");
                let debug_data_dir_content = debug_data_directory.data(bytes, &section_table);
                if let Ok(debug_data_dir_content) = debug_data_dir_content {
                    let mut pdb_file_path = None;
                    for entry in debug_data_dir_content.chunks_exact(28) {
                        let _characteristics =
                            u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
                        let _time_date_stamp =
                            u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]);
                        let _major_version = u16::from_le_bytes([entry[8], entry[9]]);
                        let _minor_version = u16::from_le_bytes([entry[10], entry[11]]);
                        let ty = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]);
                        let size_of_data =
                            u32::from_le_bytes([entry[16], entry[17], entry[18], entry[19]]);
                        let _address_of_raw_data =
                            u32::from_le_bytes([entry[20], entry[21], entry[22], entry[23]]);
                        let pointer_to_raw_data =
                            u32::from_le_bytes([entry[24], entry[25], entry[26], entry[27]]);

                        if ty == 2 {
                            let data_header_size = 24;
                            let path = String::from_utf8_lossy(
                                &bytes[data_header_size + pointer_to_raw_data as usize
                                    ..(pointer_to_raw_data + size_of_data) as usize],
                            )
                            .trim_end_matches('\0')
                            .to_string();
                            pdb_file_path = Some(path);
                            break;
                        }
                    }
                    if let Some(pdb_file_path) = pdb_file_path {
                        let pdb_absolute_path = if path::is_absolute(&pdb_file_path) {
                            pdb_file_path
                        } else {
                            path::join(
                                path::parent(file_path).unwrap_or("./"),
                                &pdb_file_path,
                                filesystem.separator(),
                            )
                        };
                        let file = filesystem.read(&pdb_absolute_path);
                        if let Ok(file) = file {
                            // TODO: maybe there is a better way to do this without writing to a file
                            let mut tmp_file =
                                tempfile::tempfile().expect("Failed to create a temporary file");
                            tmp_file
                                .write_all(&file)
                                .expect("Failed to write to a temporary file");

                            let pdb = pdb::PDB::open(tmp_file);
                            if let Ok(mut pdb) = pdb {
                                let symbol_table = pdb.global_symbols();
                                let address_map = pdb.address_map();
                                if let (Ok(symbol_table), Ok(address_map)) =
                                    (symbol_table, address_map)
                                {
                                    let mut iter = symbol_table.iter();
                                    while let Ok(Some(symbol)) = iter.next() {
                                        if let Ok(pdb::SymbolData::Public(public_symbol)) =
                                            symbol.parse()
                                        {
                                            let address = public_symbol.offset.to_rva(&address_map);
                                            if let Some(address) = address {
                                                let name =
                                                    public_symbol.name.to_string().to_string();
                                                symbols
                                                    .push((address.0 as u64 + address_base, name));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let symbols: HashMap<u64, String> = symbols
                .into_iter()
                .map(|(address, name)| (address, Self::demangle_symbol(&name)))
                .collect();

            let symbols_by_name = symbols
                .iter()
                .map(|(address, name)| (name.clone(), *address))
                .collect();

            Some(GenericHeader {
                file_type,
                architecture,
                bitness,
                endianness,
                entry,
                sections,
                symbols,
                symbols_by_name,
            })
        } else {
            None
        }
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}
