#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/logo.ico");
    res.set("OriginalFilename", "hex-patch.exe");
    res.set("FileDescription", "HexPatch - Binary Patcher and Editor");
    res.set("LegalCopyright", "Copyright (c) 2024 Ettore Ricci");
    res.compile()?;
    Ok(())
}

#[cfg(unix)]
fn main() {}