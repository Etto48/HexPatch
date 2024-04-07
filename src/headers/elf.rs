use std::{collections::HashMap, rc::Rc};

use capstone::{arch::{self, BuildsCapstone}, Capstone, CsResult};
use object::{read::elf::{ElfFile32, ElfFile64}, BigEndian, LittleEndian, Object, ObjectSection, ObjectSymbol, SectionKind};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Bitness
{
    Bit32 = 0x01,
    Bit64 = 0x02
}

impl Bitness
{
    pub fn to_num_bits(&self) -> u32
    {
        match self
        {
            Bitness::Bit32 => 32,
            Bitness::Bit64 => 64
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Endianness
{
    Little = 0x01,
    Big = 0x02
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Section
{
    pub name: String,
    pub section_type: SectionKind,
    pub address: u64,
    pub offset: u64,
    pub size: u64,
    pub address_alignment: u64,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ElfHeader
{
    pub bitness: Bitness,
    pub endianness: Endianness,
    pub entry_point: u64,
    pub section_table: Vec<Section>,
    pub symbol_table: Rc<HashMap<u64, String>>,
    pub inverse_symbol_table: HashMap<String, u64>
}

enum ElfVariant<'data>
{
    Elf64Little(ElfFile64<'data,LittleEndian>),
    Elf64Big(ElfFile64<'data,BigEndian>),
    Elf32Little(ElfFile32<'data,LittleEndian>),
    Elf32Big(ElfFile32<'data,BigEndian>)
}

impl <'data> ElfVariant<'data>
{
    pub fn new(bytes: &'data [u8]) -> Option<Self>
    {
        if bytes.len() < 4
        {
            return None;
        }
        let ident = &bytes[0..4];
        if ident[0] != 0x7F || ident[1] != 'E' as u8 || ident[2] != 'L' as u8 || ident[3] != 'F' as u8
        {
            return None;
        }
        let bitness = match bytes[4]
        {
            1 => Bitness::Bit32,
            2 => Bitness::Bit64,
            _ => return None
        };
        let endianness = match bytes[5]
        {
            1 => Endianness::Little,
            2 => Endianness::Big,
            _ => return None
        };

        match (bitness, endianness)
        {
            (Bitness::Bit32, Endianness::Little) => Some(ElfVariant::Elf32Little(ElfFile32::parse(bytes).ok()?)),
            (Bitness::Bit32, Endianness::Big) => Some(ElfVariant::Elf32Big(ElfFile32::parse(bytes).ok()?)),
            (Bitness::Bit64, Endianness::Little) => Some(ElfVariant::Elf64Little(ElfFile64::parse(bytes).ok()?)),
            (Bitness::Bit64, Endianness::Big) => Some(ElfVariant::Elf64Big(ElfFile64::parse(bytes).ok()?)),
        }
    }
}


impl ElfHeader
{
    pub fn parse_header(bytes: &[u8]) -> Option<Self>
    {
        let header = ElfVariant::new(bytes)?;
        let bitness = match header
        {
            ElfVariant::Elf32Little(_) | ElfVariant::Elf32Big(_) => Bitness::Bit32,
            ElfVariant::Elf64Little(_) | ElfVariant::Elf64Big(_) => Bitness::Bit64
        };

        let endianness = match header
        {
            ElfVariant::Elf32Little(_) => Endianness::Little,
            ElfVariant::Elf32Big(_) => Endianness::Big,
            ElfVariant::Elf64Little(_) => Endianness::Little,
            ElfVariant::Elf64Big(_) => Endianness::Big
        };

        let entry_point = match &header
        {
            ElfVariant::Elf64Little(h) => h.raw_header().e_entry.get(LittleEndian::default()),
            ElfVariant::Elf64Big(h) => h.raw_header().e_entry.get(BigEndian::default()),
            ElfVariant::Elf32Little(h) => h.raw_header().e_entry.get(LittleEndian::default()) as u64,
            ElfVariant::Elf32Big(h) => h.raw_header().e_entry.get(BigEndian::default()) as u64,
        };

        let sections: Vec<Section> = match &header
        {
            ElfVariant::Elf64Little(h) => h.sections().map(|s| Section
                {
                    name: s.name().unwrap_or_default().to_string(),
                    section_type: s.kind(),
                    address: s.address(),
                    offset: s.file_range().unwrap_or((0,0)).0 as u64,
                    size: s.file_range().unwrap_or((0,0)).1 as u64,
                    address_alignment: s.align() as u64,
                }).collect(),
            ElfVariant::Elf64Big(h) => h.sections().map(|s| Section
                {
                    name: s.name().unwrap_or_default().to_string(),
                    section_type: s.kind(),
                    address: s.address(),
                    offset: s.file_range().unwrap_or((0,0)).0 as u64,
                    size: s.file_range().unwrap_or((0,0)).1 as u64,
                    address_alignment: s.align() as u64,
                }).collect(),
            ElfVariant::Elf32Little(h) => h.sections().map(|s| Section
                {
                    name: s.name().unwrap_or_default().to_string(),
                    section_type: s.kind(),
                    address: s.address(),
                    offset: s.file_range().unwrap_or((0,0)).0 as u64,
                    size: s.file_range().unwrap_or((0,0)).1 as u64,
                    address_alignment: s.align() as u64,
                }).collect(),
            ElfVariant::Elf32Big(h) => h.sections().map(|s| Section
                {
                    name: s.name().unwrap_or_default().to_string(),
                    section_type: s.kind(),
                    address: s.address(),
                    offset: s.file_range().unwrap_or((0,0)).0 as u64,
                    size: s.file_range().unwrap_or((0,0)).1 as u64,
                    address_alignment: s.align() as u64,
                }).collect(),
        };

        let sections = sections.into_iter().filter(|s| s.size != 0).collect();

        let symbols: Vec<(u64, String)> = match header
        {
            ElfVariant::Elf64Little(h) => h.symbols().map(
                |s| (s.address(), s.name().map(|n|n.to_string()).unwrap_or(format!("s_{:#x}", s.address())))).collect(),
            ElfVariant::Elf64Big(h) => h.symbols().map(
                |s| (s.address(), s.name().map(|n|n.to_string()).unwrap_or(format!("s_{:#x}", s.address())))).collect(),
            ElfVariant::Elf32Little(h) => h.symbols().map(
                |s| (s.address(), s.name().map(|n|n.to_string()).unwrap_or(format!("s_{:#x}", s.address())))).collect(),
            ElfVariant::Elf32Big(h) => h.symbols().map(
                |s| (s.address(), s.name().map(|n|n.to_string()).unwrap_or(format!("s_{:#x}", s.address())))).collect(),
        };

        let symbols: HashMap<u64, String> = symbols.into_iter().map(|(k,v)| {
            let mangled_name = v;
            let demangled_name = cpp_demangle::Symbol::new(mangled_name.as_str());
            if let Ok(name) = demangled_name
            {
                (k, name.to_string())
            }
            else
            {
                (k, mangled_name)
            }
        }).collect();

        let inverse_symbol_table = symbols.iter().map(|(k,v)| (v.clone(), *k)).collect();

        Some(ElfHeader {
            bitness,
            endianness,
            entry_point,
            section_table: sections,
            symbol_table: Rc::new(symbols),
            inverse_symbol_table
        })
    }

    pub fn bitness(&self) -> u32
    {
        self.bitness.to_num_bits()
    }

    pub fn print_header(data: &[u8])
    {
        let header = ElfHeader::parse_header(data).expect("File is not ELF");
        dbg!(header);
    }

    pub fn get_symbols(&self) -> Rc<HashMap<u64,String>>
    {
        self.symbol_table.clone()
    }

    pub fn get_decoder(&self) -> CsResult<Capstone>
    {
        // TODO: Add support for other architectures
        Capstone::new().x86().mode(arch::x86::ArchMode::Mode64).build()
    }
}