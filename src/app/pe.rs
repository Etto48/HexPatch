#[derive(Debug, Clone, Copy)]
pub enum MachineType
{
    Unknown = 0x0,
    Alpha = 0x184,
    Alpha64AndAxp64 = 0x284,
    Am33 = 0x1d3,
    Amd64 = 0x8664,
    Arm = 0x1c0,
    Arm64 = 0xaa64,
    ArmNT = 0x1c4,
    Ebc = 0xebc,
    I386 = 0x14c,
    Ia64 = 0x200,
    LoongArch32 = 0x6232,
    LoongArch64 = 0x6264,
    M32R = 0x9041,
    Mips16 = 0x266,
    MipsFpu = 0x366,
    MipsFpu16 = 0x466,
    PowerPC = 0x1f0,
    PowerPCFP = 0x1f1,
    R4000 = 0x166,
    RiscV32 = 0x5032,
    RiscV64 = 0x5064,
    RiscV128 = 0x5128,
    Sh3 = 0x1a2,
    Sh3Dsp = 0x1a3,
    Sh4 = 0x1a6,
    Sh5 = 0x1a8,
    Thumb = 0x1c2,
    WceMipsV2 = 0x169,
}

#[derive(Debug, Clone, Copy)]
pub enum OptionalHeaderMagic
{
    PE32 = 0x10B,
    PE32Plus = 0x20B,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionalHeader
{
    pub magic: OptionalHeaderMagic,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct PEHeader
{
    pub signature_offset: usize,
    pub machine: MachineType,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,

    pub optional_header: OptionalHeader,
}

impl PEHeader
{
    pub fn parse_header(bytes: &[u8]) -> Option<PEHeader>
    {
        if bytes.len() <= 0x3C
        {
            return None;
        }
        let signature_offset = bytes[0x3c];
        if bytes.len() <= signature_offset as usize + 4
        {
            return None;
        }
        if &bytes[signature_offset as usize..signature_offset as usize + 4] != ['P' as u8, 'E' as u8, 0x00, 0x00]
        {
            return None;
        }
        let header_size = 20;
        let signature_size = 4;
        if bytes.len() <= signature_offset as usize + header_size + signature_size
        {
            return None;
        }
        let machine = u16::from_le_bytes([bytes[signature_offset as usize + 4], bytes[signature_offset as usize + 5]]);

        let machine = match machine {
            0x0 => MachineType::Unknown,
            0x184 => MachineType::Alpha,
            0x284 => MachineType::Alpha64AndAxp64,
            0x1d3 => MachineType::Am33,
            0x8664 => MachineType::Amd64,
            0x1c0 => MachineType::Arm,
            0xaa64 => MachineType::Arm64,
            0x1c4 => MachineType::ArmNT,
            0xebc => MachineType::Ebc,
            0x14c => MachineType::I386,
            0x200 => MachineType::Ia64,
            0x6232 => MachineType::LoongArch32,
            0x6264 => MachineType::LoongArch64,
            0x9041 => MachineType::M32R,
            0x266 => MachineType::Mips16,
            0x366 => MachineType::MipsFpu,
            0x466 => MachineType::MipsFpu16,
            0x1f0 => MachineType::PowerPC,
            0x1f1 => MachineType::PowerPCFP,
            0x166 => MachineType::R4000,
            0x5032 => MachineType::RiscV32,
            0x5064 => MachineType::RiscV64,
            0x5128 => MachineType::RiscV128,
            0x1a2 => MachineType::Sh3,
            0x1a3 => MachineType::Sh3Dsp,
            0x1a6 => MachineType::Sh4,
            0x1a8 => MachineType::Sh5,
            0x1c2 => MachineType::Thumb,
            0x169 => MachineType::WceMipsV2,
            _ => MachineType::Unknown,
        };

        let number_of_sections = u16::from_le_bytes([bytes[signature_offset as usize + 6], bytes[signature_offset as usize + 7]]);
        let time_date_stamp = u32::from_le_bytes([bytes[signature_offset as usize + 8], bytes[signature_offset as usize + 9], bytes[signature_offset as usize + 10], bytes[signature_offset as usize + 11]]);
        let pointer_to_symbol_table = u32::from_le_bytes([bytes[signature_offset as usize + 12], bytes[signature_offset as usize + 13], bytes[signature_offset as usize + 14], bytes[signature_offset as usize + 15]]);
        let number_of_symbols = u32::from_le_bytes([bytes[signature_offset as usize + 16], bytes[signature_offset as usize + 17], bytes[signature_offset as usize + 18], bytes[signature_offset as usize + 19]]);
        let size_of_optional_header = u16::from_le_bytes([bytes[signature_offset as usize + 20], bytes[signature_offset as usize + 21]]);
        let characteristics = u16::from_le_bytes([bytes[signature_offset as usize + 22], bytes[signature_offset as usize + 23]]);

        let optional_header = if size_of_optional_header != 0
        {
            let magic = u16::from_le_bytes([bytes[signature_offset as usize + 24], bytes[signature_offset as usize + 25]]);
            let magic = match magic
            {
                0x10B => Some(OptionalHeaderMagic::PE32),
                0x20B => Some(OptionalHeaderMagic::PE32Plus),
                _ => None,
            };
            if let Some(magic) = magic
            {
                let major_linker_version = bytes[signature_offset as usize + 26];
                let minor_linker_version = bytes[signature_offset as usize + 27];
                let size_of_code = u32::from_le_bytes([bytes[signature_offset as usize + 28], bytes[signature_offset as usize + 29], bytes[signature_offset as usize + 30], bytes[signature_offset as usize + 31]]);
                let size_of_initialized_data = u32::from_le_bytes([bytes[signature_offset as usize + 32], bytes[signature_offset as usize + 33], bytes[signature_offset as usize + 34], bytes[signature_offset as usize + 35]]);
                let size_of_uninitialized_data = u32::from_le_bytes([bytes[signature_offset as usize + 36], bytes[signature_offset as usize + 37], bytes[signature_offset as usize + 38], bytes[signature_offset as usize + 39]]);
                let address_of_entry_point = u32::from_le_bytes([bytes[signature_offset as usize + 40], bytes[signature_offset as usize + 41], bytes[signature_offset as usize + 42], bytes[signature_offset as usize + 43]]);
                let base_of_code = u32::from_le_bytes([bytes[signature_offset as usize + 44], bytes[signature_offset as usize + 45], bytes[signature_offset as usize + 46], bytes[signature_offset as usize + 47]]);
                
                Some(OptionalHeader{
                    magic,
                    major_linker_version,
                    minor_linker_version,
                    size_of_code,
                    size_of_initialized_data,
                    size_of_uninitialized_data,
                    address_of_entry_point,
                    base_of_code,
                })
            }
            else 
            {
                None
            }
        }
        else
        {
            None
        };
        if let Some(optional_header) = optional_header
        {
            Some(PEHeader {
                signature_offset: signature_offset as usize,
                machine,
                number_of_sections,
                time_date_stamp,
                pointer_to_symbol_table,
                number_of_symbols,
                size_of_optional_header,
                characteristics,
                optional_header,
            })    
        }
        else 
        {
            None
        }

        
    }

    pub fn bitness(&self) -> u32
    {
        // TODO: check if this is correct
        match self.machine
        {
            MachineType::Unknown => 64,
            MachineType::Alpha => 32,
            MachineType::Alpha64AndAxp64 => 64,
            MachineType::Am33 => 32,
            MachineType::Amd64 => 64,
            MachineType::Arm => 32,
            MachineType::Arm64 => 64,
            MachineType::ArmNT => 32,
            MachineType::Ebc => 64,
            MachineType::I386 => 32,
            MachineType::Ia64 => 64,
            MachineType::LoongArch32 => 32,
            MachineType::LoongArch64 => 64,
            MachineType::M32R => 32,
            MachineType::Mips16 => 16,
            MachineType::MipsFpu => 32,
            MachineType::MipsFpu16 => 16,
            MachineType::PowerPC => 32,
            MachineType::PowerPCFP => 32,
            MachineType::R4000 => 32,
            MachineType::RiscV32 => 32,
            MachineType::RiscV64 => 64,
            MachineType::RiscV128 => 128,
            MachineType::Sh3 => 32,
            MachineType::Sh3Dsp => 32,
            MachineType::Sh4 => 32,
            MachineType::Sh5 => 32,
            MachineType::Thumb => 32,
            MachineType::WceMipsV2 => 32,
        }
    }
}