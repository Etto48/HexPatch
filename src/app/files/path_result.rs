use std::error::Error;

use ratatui::text::{Line, Span};

use crate::app::settings::color_settings::ColorSettings;

use super::{filesystem::FileSystem, path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathResult {
    path: String,
    is_dir: bool,
}

impl PathResult {
    pub fn new(path: &str, filesystem: &FileSystem) -> Result<Self, Box<dyn Error>> {
        let path = filesystem.canonicalize(path)?;
        let is_dir = filesystem.is_dir(&path);
        Ok(Self { path, is_dir })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub fn to_line(
        &self,
        color_settings: &ColorSettings,
        is_selected: bool,
        base_path: &str,
    ) -> Line<'static> {
        let mut ret = Line::raw("");
        let style = if is_selected {
            color_settings.path_selected
        } else if self.is_dir() {
            color_settings.path_dir
        } else {
            color_settings.path_file
        };
        let path = path::diff(&self.path, base_path);
        ret.spans.push(Span::styled(path.to_string(), style));

        ret.left_aligned()
    }
}
