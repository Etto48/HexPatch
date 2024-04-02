use std::{error::Error, path::PathBuf};

use ratatui::text::{Line, Span};

use crate::app::color_settings::ColorSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathResult
{
    path: PathBuf,
}

impl PathResult
{
    pub fn new(path: &PathBuf, base_path: &PathBuf) -> Result<Self, Box<dyn Error>>
    {
        let path = base_path.join(path).canonicalize()?;

        Ok(Self
        {
            path,
        })
    }

    pub fn path(&self) -> &PathBuf
    {
        &self.path
    }

    pub fn is_dir(&self) -> bool
    {
        self.path.is_dir()
    }

    pub fn to_line(&self, color_settings: &ColorSettings, is_selected: bool, base_path: &PathBuf) -> Line<'static>
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
        let path = pathdiff::diff_paths(&self.path, base_path).unwrap_or(self.path.clone());
        ret.spans.push(Span::styled(path.to_string_lossy().to_string(), style));

        ret.left_aligned()
    }
}