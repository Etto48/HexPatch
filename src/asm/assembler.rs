use std::{error::Error, io::{Read, Write}, process::{Command, Stdio}};

fn discover_assembler() -> Option<String> {
    // List of common assembler executables
    let assemblers = ["nasm", "yasm", "gas", "llvm-as"];

    // Search for each assembler in PATH
    for assembler in &assemblers {
        if let Ok(path) = which::which(assembler) {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    None
}

pub fn assemble(asm: &str, bitness: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut input_file = tempfile::NamedTempFile::new()?;
    let output_file = tempfile::NamedTempFile::new()?;

    if bitness != 16
    {
        input_file.write_all(format!("bits {}\n", bitness).as_bytes())?;
    }

    input_file.write_all(asm.as_bytes())?;

    let assembler = discover_assembler().ok_or("No assembler found")?;

    let out = Command::new(assembler)
        .arg("-o")
        .arg(output_file.path())
        .arg("-f")
        .arg("bin")
        .arg(input_file.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    
    if !out.status.success() {
        return Err(format!("Assembler failed: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    else {
        let mut output = Vec::new();
        output_file.as_file().read_to_end(&mut output)?;
        Ok(output)
    }
}