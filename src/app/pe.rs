use std::{collections::HashMap, fs::File, rc::Rc};

use object::{pe::ImageNtHeaders64, read::pe::PeFile, LittleEndian, Object, ObjectSymbol};
use pdb::FallibleIterator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section
{
    pub name: String,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PEHeader
{
    pub entry_point: u64,
    pub bitness: u32,
    pub section_table: Vec<Section>,
    pub symbol_table: Rc<HashMap<u64, String>>,
    pub inverse_symbol_table: HashMap<String, u64>
}

impl PEHeader
{
    pub fn parse_header(bytes: &[u8]) -> Option<PEHeader>
    {
        let header = PeFile::<ImageNtHeaders64>::parse(bytes);
        match header
        {
            Ok(header) => 
            {
                let entry_point = header.nt_headers().optional_header.address_of_entry_point.get(LittleEndian::default()) as u64;
                let bitness = if header.is_64() { 64 } else { 32 };

                let mut section_table = Vec::new();
                let section_table_in_header = header.section_table();
                for section in section_table_in_header.iter()
                {
                    let name = String::from_utf8_lossy(&section.name).trim_end_matches('\0').to_string();
                    section_table.push(Section
                    {
                        name,
                        virtual_size: section.virtual_size.get(LittleEndian::default()),
                        virtual_address: section.virtual_address.get(LittleEndian::default()),
                        size_of_raw_data: section.size_of_raw_data.get(LittleEndian::default()),
                        pointer_to_raw_data: section.pointer_to_raw_data.get(LittleEndian::default()),
                        pointer_to_relocations: section.pointer_to_relocations.get(LittleEndian::default()),
                        pointer_to_linenumbers: section.pointer_to_linenumbers.get(LittleEndian::default()),
                        number_of_relocations: section.number_of_relocations.get(LittleEndian::default()),
                        number_of_linenumbers: section.number_of_linenumbers.get(LittleEndian::default()),
                        characteristics: section.characteristics.get(LittleEndian::default()),
                    });
                }

                let mut symbols = HashMap::new();
                for symbol in header.symbols()
                {
                    symbols.insert(symbol.address(), symbol.name().map(|s|s.to_string()).unwrap_or(format!("s_0x{:x}", symbol.address())));
                }
                let mut pdb_file_path = None;
                if let Some(data_dir) = header.data_directory(6)
                {
                    let debug_dir = data_dir.data(bytes, &section_table_in_header);
                    if let Ok(debug_dir) = debug_dir
                    {
                        for entry in debug_dir.chunks_exact(28)
                        {
                            let _characteristics = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
                            let _time_date_stamp = u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]);
                            let _major_version = u16::from_le_bytes([entry[8], entry[9]]);
                            let _minor_version = u16::from_le_bytes([entry[10], entry[11]]);
                            let ty = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]);
                            let size_of_data = u32::from_le_bytes([entry[16], entry[17], entry[18], entry[19]]);
                            let _address_of_raw_data = u32::from_le_bytes([entry[20], entry[21], entry[22], entry[23]]);
                            let pointer_to_raw_data = u32::from_le_bytes([entry[24], entry[25], entry[26], entry[27]]);

                            if ty == 2
                            {
                                let data_header_size = 24;
                                let path = String::from_utf8_lossy(&bytes[data_header_size+pointer_to_raw_data as usize..(pointer_to_raw_data + size_of_data) as usize]).trim_end_matches('\0').to_string();
                                pdb_file_path = Some(path);
                                break;
                            }
                        }
                    }
                }
                if let Some(pdb_file_path) = pdb_file_path
                {
                    let file = File::open(pdb_file_path);
                    if let Ok(file) = file
                    {
                        let pdb = pdb::PDB::open(file);
                        if let Ok(mut pdb) = pdb
                        {
                            let symbol_table = pdb.global_symbols();
                            let address_map = pdb.address_map();
                            match (symbol_table, address_map)
                            {
                                (Ok(symbol_table), Ok(address_map)) =>
                                {
                                    let mut iter = symbol_table.iter();
                                    while let Ok(Some(symbol)) = iter.next()
                                    {
                                        match symbol.parse()
                                        {
                                            Ok(pdb::SymbolData::Public(public_symbol)) =>
                                            {
                                                let address = public_symbol.offset.to_rva(&address_map);
                                                if let Some(address) = address
                                                {
                                                    let mangled_name = public_symbol.name.to_string().to_string();
                                                    let name = cpp_demangle::Symbol::new(&mangled_name);
                                                    if let Ok(name) = name
                                                    {
                                                        symbols.insert(address.0 as u64, name.to_string());
                                                    }
                                                    else 
                                                    {
                                                        symbols.insert(address.0 as u64, mangled_name);    
                                                    }
                                                    
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let inverse_symbol_table = symbols.iter().map(|(k, v)| (v.clone(), *k)).collect();

                Some(PEHeader
                {
                    entry_point,
                    bitness,
                    section_table,
                    symbol_table: Rc::new(symbols),
                    inverse_symbol_table
                })
            },
            Err(_) => None,
        }
    }

    pub fn get_symbols(&self) -> Rc<HashMap<u64, String>>
    {
        self.symbol_table.clone()
    }

    pub fn bitness(&self) -> u32
    {
        self.bitness
    }
}