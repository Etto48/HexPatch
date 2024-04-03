#[derive(Debug, clap::Parser)]
#[command(name = "hex-patch", about, version, author)]
pub struct Args
{
    #[arg(index = 1, help = "The starting path of the editor", default_value = "./")]
    pub path: std::path::PathBuf,
}