use std::{error::Error, fmt::Display, path::PathBuf, sync::Arc};

use russh::client::{self, AuthResult, Handler};
use russh::keys::key::PrivateKeyWithHashAlg;
use russh_sftp::client::SftpSession;

use crate::app::files::path;

pub struct SSHClient;
impl Handler for SSHClient {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct Connection {
    runtime: tokio::runtime::Runtime,
    sftp: SftpSession,
    connection_str: String,
}

impl Connection {
    fn get_key_files() -> Result<(PathBuf, PathBuf), String> {
        let home_dir = dirs::home_dir().ok_or_else(|| "Home directory not found".to_string())?;

        let ssh_dir = home_dir.join(".ssh");
        if !ssh_dir.is_dir() {
            return Err("SSH directory not found".into());
        }
        if ssh_dir.join("id_rsa").is_file() {
            Ok((ssh_dir.join("id_rsa"), ssh_dir.join("id_rsa.pub")))
        } else if ssh_dir.join("id_ed25519").is_file() {
            Ok((ssh_dir.join("id_ed25519"), ssh_dir.join("id_ed25519.pub")))
        } else if ssh_dir.join("id_ecdsa").is_file() {
            Ok((ssh_dir.join("id_ecdsa"), ssh_dir.join("id_ecdsa.pub")))
        } else if ssh_dir.join("id_dsa").is_file() {
            Ok((ssh_dir.join("id_dsa"), ssh_dir.join("id_dsa.pub")))
        } else {
            Err("No private key found".into())
        }
    }

    pub fn new(connection_str: &str, password: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (username, host) = connection_str
            .split_once('@')
            .ok_or_else(|| Box::<dyn Error>::from("Invalid connection string"))?;

        let (hostname, port) =
            host.split_once(':')
                .map_or(Ok((host, 22)), |(hostname, port)| {
                    port.parse::<u16>()
                        .map(|port| (hostname, port))
                        .map_err(|_| Box::<dyn Error>::from("Invalid port"))
                })?;

        let config = client::Config::default();

        let mut session = runtime.block_on(client::connect(
            config.into(),
            (hostname, port),
            SSHClient {},
        ))?;
        if let Some(password) = password {
            if let AuthResult::Failure {
                remaining_methods: _,
            } = runtime.block_on(session.authenticate_password(username, password))?
            {
                return Err("Authentication failed".into());
            }
        } else {
            let (private_key, _public_key) = Self::get_key_files()?;
            let keypair = russh::keys::load_secret_key(private_key, None)?;
            let keypair = PrivateKeyWithHashAlg::new(Arc::new(keypair), None);
            if let AuthResult::Failure {
                remaining_methods: _,
            } = runtime.block_on(session.authenticate_publickey(username, keypair))?
            {
                return Err("Authentication failed".into());
            }
        }

        let channel = runtime.block_on(session.channel_open_session())?;
        runtime.block_on(channel.request_subsystem(true, "sftp"))?;

        let sftp = runtime.block_on(SftpSession::new(channel.into_stream()))?;

        Ok(Self {
            runtime,
            sftp,
            connection_str: connection_str.to_string(),
        })
    }

    pub fn separator(&self) -> char {
        match self.runtime.block_on(self.sftp.canonicalize("/")) {
            Ok(_) => '/',
            Err(_) => '\\',
        }
    }

    pub fn canonicalize(&self, path: &str) -> Result<String, Box<dyn Error>> {
        Ok(self.runtime.block_on(self.sftp.canonicalize(path))?)
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let remote_file = self.runtime.block_on(self.sftp.read(path))?;
        Ok(remote_file)
    }

    pub fn mkdirs(&self, path: &str) -> Result<(), Box<dyn Error>> {
        self.runtime.block_on(async {
            let mut paths = vec![path];
            let mut current = path;
            while let Some(parent) = path::parent(current) {
                paths.push(parent);
                current = parent;
            }
            paths.reverse();
            for path in paths {
                if self.sftp.read_dir(path).await.is_ok() {
                    continue;
                };
                self.sftp.create_dir(path).await?;
            }
            Ok::<(), Box<dyn Error>>(())
        })?;
        Ok(())
    }

    pub fn create(&self, path: &str) -> Result<(), Box<dyn Error>> {
        self.runtime.block_on(self.sftp.create(path))?;
        Ok(())
    }

    pub fn write(&self, path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.runtime.block_on(self.sftp.write(path, data))?;
        Ok(())
    }

    pub fn ls(&self, path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let dir = self.runtime.block_on(self.sftp.read_dir(path))?;
        dir.into_iter()
            .map(|entry| Ok(path::join(path, &entry.file_name(), self.separator()).to_string()))
            .collect()
    }

    pub fn is_file(&self, path: &str) -> bool {
        self.runtime
            .block_on(self.sftp.metadata(path))
            .is_ok_and(|metadata| !metadata.is_dir())
    }

    pub fn is_dir(&self, path: &str) -> bool {
        self.runtime
            .block_on(self.sftp.metadata(path))
            .is_ok_and(|metadata| metadata.is_dir())
    }
}

impl Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.connection_str)
    }
}
