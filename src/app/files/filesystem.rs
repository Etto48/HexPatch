use crate::app::ssh::connection::Connection;

pub enum FileSystem
{
    Local,
    Remote(Connection)
}