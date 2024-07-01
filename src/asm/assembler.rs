use std::error::Error;

use crate::headers::Header;

pub fn assemble(
    asm: &str,
    starting_virtual_address: u64,
    header: &Header,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let encoder = header
        .get_encoder()
        .map_err(|e| format!("Failed to get encoder: {}", e))?;

    let out = encoder
        .asm(asm.to_string(), starting_virtual_address)
        .map_err(|e| format!("Failed to assemble: {}", e))?;
    Ok(out.bytes)
}
