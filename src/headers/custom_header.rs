use std::collections::HashMap;

use object::{Architecture, Endianness};

use super::{bitness::Bitness, section::Section};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomHeader {
    pub bitness: Bitness,
    pub entry: u64,
    pub endianness: Endianness,
    pub architecture: Architecture,
    pub sections: Vec<Section>,
    pub symbols: HashMap<u64, String>,
    pub symbols_by_name: HashMap<String, u64>,
}
