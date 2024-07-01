use std::{error::Error, path::Path};

use crate::app::ssh::connection::Connection;

use super::path;

pub enum FileSystem {
    Local {
        path: String,
    },
    Remote {
        path: String,
        connection: Connection,
    },
}

impl FileSystem {
    pub fn new_local(path: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self::Local {
            path: Path::new(path)
                .canonicalize()
                .map(|path| path.to_string_lossy().to_string())?,
        })
    }

    pub fn new_remote(
        path: &str,
        connection_str: &str,
        password: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let connection = Connection::new(connection_str, password)?;
        Ok(Self::Remote {
            path: connection.canonicalize(path)?,
            connection,
        })
    }

    pub fn separator(&self) -> char {
        match self {
            Self::Local { .. } => std::path::MAIN_SEPARATOR,
            Self::Remote { connection, .. } => connection.separator(),
        }
    }

    pub fn pwd(&self) -> &str {
        match self {
            Self::Local { path } | Self::Remote { path, .. } => path,
        }
    }

    pub fn cd(&mut self, path: &str) {
        match self {
            Self::Local { path: current }
            | Self::Remote {
                path: current,
                connection: _,
            } => *current = path.into(),
        }
    }

    pub fn ls(&self, path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut ret = match self {
            Self::Local { .. } => {
                let dir = std::fs::read_dir(path)?;
                let mut ret = Vec::new();
                for f in dir {
                    let f = f?;
                    ret.push(f.path().to_string_lossy().to_string())
                }
                Ok(ret)
            }
            Self::Remote { connection, .. } => connection.ls(path),
        }?;
        if path::parent(path).is_some() {
            ret.insert(0, path::join(path, "..", self.separator()));
        }
        Ok(ret)
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Self::Local { .. } => Ok(std::fs::read(path)?),
            Self::Remote { connection, .. } => connection.read(path),
        }
    }

    pub fn mkdirs(&self, path: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Local { .. } => std::fs::create_dir_all(path)?,
            Self::Remote { connection, .. } => connection.mkdirs(path)?,
        }
        Ok(())
    }

    pub fn create(&self, path: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Local { .. } => {
                std::fs::File::create(path)?;
            }
            Self::Remote { connection, .. } => {
                connection.create(path)?;
            }
        }
        Ok(())
    }

    pub fn write(&self, path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Local { .. } => Ok(std::fs::write(path, data)?),
            Self::Remote { connection, .. } => connection.write(path, data),
        }
    }

    pub fn is_file(&self, path: &str) -> bool {
        match self {
            Self::Local { .. } => Path::new(path).is_file(),
            Self::Remote { connection, .. } => connection.is_file(path),
        }
    }

    pub fn is_dir(&self, path: &str) -> bool {
        match self {
            Self::Local { .. } => Path::new(path).is_dir(),
            Self::Remote { connection, .. } => connection.is_dir(path),
        }
    }

    pub fn canonicalize(&self, path: &str) -> Result<String, Box<dyn Error>> {
        match self {
            Self::Local { .. } => Ok(Path::new(path)
                .canonicalize()
                .map(|path| path.to_string_lossy().to_string())?),
            Self::Remote { connection, .. } => connection.canonicalize(path),
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::Local { path: "./".into() }
    }
}
