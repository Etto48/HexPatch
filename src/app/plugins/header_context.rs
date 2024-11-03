use std::collections::HashMap;

use mlua::UserData;
use object::{Architecture, Endianness};

use crate::headers::{bitness::Bitness, custom_header::CustomHeader, section::Section};

#[derive(Debug, Clone, Default)]
pub struct HeaderContext {
    pub bitness: Option<Bitness>,
    pub endianness: Option<Endianness>,
    pub entry: Option<u64>,
    pub architecture: Option<Architecture>,
    pub sections: Vec<Section>,
    pub symbols: HashMap<u64, String>,
}

impl HeaderContext {
    pub fn try_into_custom_header(self) -> Option<CustomHeader> {
        if let (Some(bitness), Some(endianness), Some(entry), Some(architecture)) =
            (self.bitness, self.endianness, self.entry, self.architecture)
        {
            let symbols_by_name = self.symbols.iter().map(|(k, v)| (v.clone(), *k)).collect();
            Some(CustomHeader {
                bitness,
                entry,
                endianness,
                architecture,
                sections: self.sections,
                symbols: self.symbols,
                symbols_by_name,
            })
        } else {
            None
        }
    }
}

impl UserData for HeaderContext {
    fn add_methods<'lua, M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("set_bitness", |_, this, bitness: u8| {
            if this.bitness.is_some() {
                Err(mlua::Error::external("bitness already set"))
            } else {
                match bitness {
                    32 => {
                        this.bitness = Some(Bitness::Bit32);
                        Ok(())
                    }
                    64 => {
                        this.bitness = Some(Bitness::Bit64);
                        Ok(())
                    }
                    _ => Err(mlua::Error::external("invalid bitness")),
                }
            }
        });

        methods.add_method_mut("set_entry", |_, this, entry_point: u64| {
            if this.entry.is_some() {
                Err(mlua::Error::external("entry point already set"))
            } else {
                this.entry = Some(entry_point);
                Ok(())
            }
        });

        methods.add_method_mut("set_endianness", |_, this, endianness: String| {
            if this.endianness.is_some() {
                Err(mlua::Error::external("endianness already set"))
            } else {
                match endianness.as_str() {
                    "little" => {
                        this.endianness = Some(Endianness::Little);
                        Ok(())
                    }
                    "big" => {
                        this.endianness = Some(Endianness::Big);
                        Ok(())
                    }
                    _ => Err(mlua::Error::external("invalid endianness")),
                }
            }
        });

        methods.add_method_mut("set_architecture", |_, this, architecture: String| {
            if this.architecture.is_some() {
                Err(mlua::Error::external("architecture already set"))
            } else {
                let architecture = match architecture.as_str() {
                    "Unknown" => Architecture::Unknown,
                    "Aarch64" => Architecture::Aarch64,
                    "Aarch64_Ilp32" => Architecture::Aarch64_Ilp32,
                    "Arm" => Architecture::Arm,
                    "Avr" => Architecture::Avr,
                    "Bpf" => Architecture::Bpf,
                    "Csky" => Architecture::Csky,
                    "I386" => Architecture::I386,
                    "X86_64" => Architecture::X86_64,
                    "X86_64_X32" => Architecture::X86_64_X32,
                    "Hexagon" => Architecture::Hexagon,
                    "LoongArch64" => Architecture::LoongArch64,
                    "Mips" => Architecture::Mips,
                    "Mips64" => Architecture::Mips64,
                    "Msp430" => Architecture::Msp430,
                    "PowerPc" => Architecture::PowerPc,
                    "PowerPc64" => Architecture::PowerPc64,
                    "Riscv32" => Architecture::Riscv32,
                    "Riscv64" => Architecture::Riscv64,
                    "S390x" => Architecture::S390x,
                    "Sbf" => Architecture::Sbf,
                    "Sharc" => Architecture::Sharc,
                    "Sparc" => Architecture::Sparc,
                    "Sparc32Plus" => Architecture::Sparc32Plus,
                    "Sparc64" => Architecture::Sparc64,
                    "Wasm32" => Architecture::Wasm32,
                    "Wasm64" => Architecture::Wasm64,
                    "Xtensa" => Architecture::Xtensa,
                    _ => return Err(mlua::Error::external("invalid architecture")),
                };
                this.architecture = Some(architecture);
                Ok(())
            }
        });

        methods.add_method_mut(
            "add_section",
            |_, this, (name, virtual_address, file_offset, size): (String, u64, u64, u64)| {
                this.sections.push(Section {
                    name,
                    virtual_address,
                    file_offset,
                    size,
                });
                Ok(())
            },
        );

        methods.add_method_mut("add_symbol", |_, this, (address, name): (u64, String)| {
            this.symbols.insert(address, name);
            Ok(())
        });
    }
}
