use std::{error::Error, io::{Read, Write}, process::{Command, Stdio}};

fn discover_assembler() -> Option<String> {
    // List of common assembler executables
    let assemblers = ["nasm", "yasm", "as", "llvm-as"];

    // Search for each assembler in PATH
    for assembler in &assemblers {
        if let Ok(path) = which::which(assembler) {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    None
}

pub fn assemble(asm: &str, bitness: u32, starting_virtual_address: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut input_file = tempfile::NamedTempFile::new()?;
    let output_file = tempfile::NamedTempFile::new()?;

    input_file.write_all(format!("bits {}\n", bitness).as_bytes())?;
    if starting_virtual_address != 0
    {
        input_file.write_all(format!("org {:#x}\n", starting_virtual_address).as_bytes())?;
    }

    input_file.write_all(asm.as_bytes())?;

    let assembler = discover_assembler().ok_or("No assembler found")?;

    let out = Command::new(assembler)
        .arg("-o")
        .arg(output_file.path())
        .arg("-f")
        .arg("bin")
        .arg("-s")
        .arg(input_file.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    
    if !out.status.success() || out.stderr.len() > 0 || out.stdout.len() > 0 {
        let error_str = String::from_utf8_lossy(&out.stdout);
        let error = error_str.split(':').last().unwrap_or("Unknown error").trim();
        return Err(format!("Assembler failed: {}", error).into());
    }
    else {
        let mut output = Vec::new();
        output_file.as_file().read_to_end(&mut output)?;
        Ok(output)
    }
}