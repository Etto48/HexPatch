#[derive(Debug, clap::Parser)]
#[command(name = "hex-patch", about, version, author)]
pub struct Args
{
    #[arg(short, long, help = "SSH connection string to the remote server")]
    pub ssh: Option<String>,
    #[arg(index = 1, help = "The starting path of the editor", default_value = "./")]
    pub path: String,
}