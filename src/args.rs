#[derive(Debug, clap::Parser)]
pub struct Args
{
    #[clap(index = 1, help = "The file to open in the hex editor", default_value = "./")]
    pub path: std::path::PathBuf,
}