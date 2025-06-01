use std::error::Error;

use crate::headers::Header;

pub fn assemble(
    asm: &str,
    starting_virtual_address: u64,
    header: &Header,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let encoder = header
        .get_encoder()
        .map_err(|e| t!("errors.create_encoder", e = e))?;

    let out = encoder
        .asm(asm.to_string(), starting_virtual_address)
        .map_err(|e| t!("errors.assemble", e = e))?;
    Ok(out.bytes)
}
