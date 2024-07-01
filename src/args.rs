use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(name = "hex-patch", about, version, author)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "The connection string to the remote server, in the format <user>@<host>[:<port>]"
    )]
    pub ssh: Option<String>,
    #[arg(
        short = 'w',
        long,
        help = "The password to use for SSH connection, if not specified, keypair authentication will be used.",
        requires = "ssh"
    )]
    pub password: Option<String>,
    #[arg(short, long, help = "The configuration file to use")]
    pub config: Option<PathBuf>,
    #[arg(short, long, help = "The plugin directory to use")]
    pub plugins: Option<PathBuf>,
    #[arg(
        index = 1,
        help = "The starting path of the editor",
        default_value = "./"
    )]
    pub path: String,
}
