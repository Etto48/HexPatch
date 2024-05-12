use std::{error::Error, io::{Read, Write}, net::TcpStream, path::Path};

use ssh2::Session;

pub struct Connection
{
    session: Session
}

impl Connection
{
    pub fn new(connection_str: &str) -> Result<Self, Box<dyn Error>>
    {
        let (username, host) = 
        if let Some((username, host)) = connection_str.split_once('@')
        {
            (username, host)
        }
        else
        {
            return Err("Invalid connection string".into())
        };

        let (hostname, port) = 
        if let Some((hostname, port)) = host.split_once(':')
        {
            if let Some(port) = port.parse::<u16>().ok()
            {
                (hostname, port)
            }
            else
            {
                return Err("Invalid port".into())
            }
        }
        else
        {
            (host, 22)
        };

        let tcp = TcpStream::connect((hostname, port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        session.userauth_agent(username)?;
        if !session.authenticated()
        {
            return Err("Failed to authenticate".into())
        }

        Ok(Self {
            session
        })
    }

    pub fn read(&self, path: &Path) -> Result<Vec<u8>, Box<dyn Error>>
    {
        let (mut remote_file, stat) = self.session.scp_recv(path)?;
        let mut data = Vec::new();
        data.reserve(stat.size() as usize);
        remote_file.read_to_end(&mut data)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(data)
    }

    pub fn write(&self, path: &Path, data: &[u8]) -> Result<(), Box<dyn Error>>
    {
        let mut remote_file = self.session.scp_send(path, 0o644, data.len() as u64, None)?;
        remote_file.write(data)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }
}