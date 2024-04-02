use std::path::PathBuf;

use ratatui::text::{Line, Span};

use crate::app::color_settings::ColorSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResult
{
    File(String),
    Directory(String),
}

impl PathResult
{
    pub fn path(&self) -> PathBuf
    {
        PathBuf::from(match self
        {
            PathResult::File(path) => path,
            PathResult::Directory(path) => path,
        })
    }

    pub fn is_dir(&self) -> bool
    {
        match self
        {
            PathResult::File(_) => false,
            PathResult::Directory(_) => true,
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings, is_selected: bool) -> Line<'static>
    {
        let mut ret = Line::raw("");
        let style = if is_selected
        {
            color_settings.path_selected
        }
        else
        {
            if self.is_dir()
            {
                color_settings.path_dir
            }
            else
            {
                color_settings.path_file
            }
        };
        let path = match self
        {
            PathResult::File(path) =>
            {
                path
            },
            PathResult::Directory(path) =>
            {
                path
            },
        };
        ret.spans.push(Span::styled(path.clone(), style));

        ret.left_aligned()
    }
}