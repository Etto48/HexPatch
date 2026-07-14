use std::error::Error;

use crate::headers::encoder::Encoder;
use crate::headers::Header;
use rbpf::assembler;

pub fn assemble(
    asm: &str,
    starting_virtual_address: u64,
    header: &Header,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let encoder = header
        .get_encoder()
        .map_err(|e| t!("errors.create_encoder", e = e))?;

    let out: Vec<u8> = match encoder {
        Encoder::Keystone(keystone) => {
            keystone
                .asm(asm.to_string(), starting_virtual_address)
                .map_err(|e| t!("errors.assemble", e = e))?
                .bytes
        }
        Encoder::EBPF => assembler::assemble(asm).map_err(|e| t!("errors.assemble", e = e))?,
    };

    Ok(out)
}
