use std::{error::Error, path::{Path, PathBuf}};

use crate::app::ssh::connection::Connection;

use super::file::File;

#[derive(Clone)]
pub enum FileSystem
{
    Local{
        path: PathBuf
    },
    Remote{
        path: PathBuf,
        connection: Connection
    }
}

impl FileSystem
{
    pub fn new_local(path: &Path) -> Result<Self, Box<dyn Error>>
    {
        Ok(Self::Local { path: path.canonicalize()? })
    }

    pub fn new_remote(path: &Path, connection_str: &str) -> Result<Self, Box<dyn Error>>
    {
        let connection = Connection::new(connection_str)?;
        Ok(Self::Remote { path: connection.canonicalize(path)?, connection })
    }

    pub fn pwd(&self) -> &Path
    {
        match self
        {
            Self::Local { path } |
            Self::Remote { path, .. } => &path
        }
    }

    pub fn cd(&mut self, path: &Path)
    {
        match self
        {
            Self::Local { path: current } |
            Self::Remote { path: current, connection: _ } => *current = path.into()
        }
    }

    pub fn ls(&self, path: &Path) -> Result<Vec<File>, Box<dyn Error>>
    {
        let mut ret = match self
        {
            Self::Local { .. } => {
                let dir = std::fs::read_dir(path)?;
                let mut ret = Vec::new();
                for f in dir
                {
                    let f = f?;
                    ret.push(File{
                        path: f.path(),
                        is_dir: f.file_type()?.is_dir()
                    })
                }
                Ok(ret)
            }
            Self::Remote { connection, .. } => connection.ls(path)
        }?;
        if path.parent().is_some()
        {
            ret.insert(0, File{
                path: path.join(".."),
                is_dir: true
            });
        }
        Ok(ret)
    }

    pub fn read(&self, path: &Path) -> Result<Vec<u8>, Box<dyn Error>>
    {
        match self
        {
            Self::Local { .. } => Ok(std::fs::read(path)?),
            Self::Remote { connection, .. } => connection.read(path)
        }
    }

    pub fn write(&self, path: &Path, data: &[u8]) -> Result<(), Box<dyn Error>>
    {
        match self
        {
            Self::Local { .. } => Ok(std::fs::write(path, data)?),
            Self::Remote { connection, .. } => connection.write(path, data)
        }
    }

    pub fn is_file(&self, path: &Path) -> bool
    {
        match self
        {
            Self::Local { .. } => path.is_file(),
            Self::Remote { connection, .. } => connection.is_file(path)
        }
    }

    pub fn is_dir(&self, path: &Path) -> bool
    {
        match self
        {
            Self::Local { .. } => path.is_dir(),
            Self::Remote { connection, .. } => connection.is_dir(path)
        }
    }

}

impl Default for FileSystem
{
    fn default() -> Self {
        Self::Local { path: "./".into() }
    }
}