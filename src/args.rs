#[derive(Debug, clap::Parser)]
pub struct Args
{
    #[clap(index = 1, help = "The file to open in the hex editor")]
    pub file: std::path::PathBuf,
}