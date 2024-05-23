#![allow(clippy::module_inception)]
use std::error::Error;

use ratatui::{backend::Backend, Terminal};

use crate::{app::{info_mode::InfoMode, notification::NotificationLevel, popup_state::PopupState, App}, headers::Header};

use super::{filesystem::FileSystem, path_result::PathResult, path};

impl App
{
    pub(in crate::app) fn go_to_path<B: Backend>(
        &mut self, 
        currently_open_path: &str, 
        path: &str, 
        scroll: usize, 
        popup: &mut Option<PopupState>,
        terminal: &mut Terminal<B>
    ) -> Result<(), Box<dyn Error>>
    {
        let contents = Self::find_dir_contents(currently_open_path, path, &self.filesystem)?;
        if contents.is_empty()
        {
            return Err(format!("No files found that matches \"{}\"", path).into());
        }
        let selected = contents.into_iter().nth(scroll).expect("Scroll out of bounds for go_to_path.");

        if self.filesystem.is_dir(selected.path())
        {
            Self::open_dir(popup, selected.path(), &mut self.filesystem)?;
        }
        else
        {
            self.open_file(selected.path(), Some(terminal))?;
            *popup = None;
        }
        
        Ok(())
    }

    pub(in crate::app) fn get_current_dir(&self) -> String
    {
        let current_path = self.filesystem.pwd();
        if self.filesystem.is_dir(current_path)
        {
            current_path.to_owned()
        }
        else 
        {
            path::parent(current_path).expect("A file should have a parent directory.").to_owned()
        }
    }

    pub(in crate::app) fn find_dir_contents(currently_open_path: &str, path: &str, filesystem: &FileSystem) -> Result<Vec<PathResult>, Box<dyn Error>>
    {
        let mut ret = Vec::new();
        let (selected_dir, file_name) = if path::is_absolute(path)
        {
            if filesystem.is_dir(path)
            {
                (filesystem.canonicalize(path)?, "".to_string())
            }
            else if let Some(parent) = path::parent(path)
            {
                if filesystem.is_dir(parent)
                {
                    (filesystem.canonicalize(parent)?, path::filename(path).map_or("".into(), |name| name.to_string()))
                }
                else
                {
                    (currently_open_path.to_string(), path.to_string())
                }
            }
            else
            {
                (currently_open_path.to_string(), path.to_string())
            }
        }
        else
        {
            (currently_open_path.to_string(), path.to_string())
        };

        let entries = filesystem.ls(&selected_dir)?;
        let entries = entries
            .into_iter()
            .map(|entry| path::diff(&entry, &selected_dir).to_string())
            .collect::<Vec<_>>();

        let entries = entries
            .into_iter()
            .filter(|entry| entry.to_lowercase().starts_with(&file_name.to_lowercase()));

        for entry in entries
        {
            if let Ok(result) = PathResult::new(&path::join(&selected_dir, &entry, filesystem.separator()), filesystem)
            {
                ret.push(result);
            }
        }

        Ok(ret)
    }

    pub(in crate::app) fn open_dir(popup: &mut Option<PopupState>, path: &str, filesystem: &mut FileSystem) -> Result<(), Box<dyn Error>>
    {
        let path = filesystem.canonicalize(path)?;
        *popup = Some(PopupState::Open { 
            currently_open_path: path.clone(),
            path: "".into(), 
            cursor: 0, 
            results: Self::find_dir_contents(&path, "", filesystem)?, 
            scroll: 0 });
        Ok(())
    }

    pub fn log_header_info(&mut self)
    {
        if self.header != Header::None
        {
            match &self.header
            {
                Header::GenericHeader(header) => self.log(NotificationLevel::Info, &format!("File type: {:?}", header.file_type())),
                Header::None => unreachable!(),
            }
            self.log(NotificationLevel::Info, &format!("Architecture: {:?}", self.header.architecture()));
            self.log(NotificationLevel::Info, &format!("Bitness: {}", self.header.bitness()));
            self.log(NotificationLevel::Info, &format!("Entry point: {:#X}", self.header.entry_point()));
            for section in self.header.get_sections()
            {
                self.log(NotificationLevel::Info, &format!("Section: {}", section));
            }
        }
        else
        {
            self.log(NotificationLevel::Info, "No header found. Assuming 64-bit.");
        }
        
        self.log(NotificationLevel::Info, &format!("Press {} for a list of commands.", Self::key_event_to_string(self.settings.key.help)));
    }

    pub(in crate::app) fn open_file<B: Backend>(&mut self, path: &str, mut terminal: Option<&mut Terminal<B>>) -> Result<(), Box<dyn Error>>
    {
        self.log(NotificationLevel::Info, &format!("Opening file: \"{}\"", path));

        self.filesystem.cd(path);
        self.dirty = false;
        self.info_mode = InfoMode::Text;
        self.scroll = 0;
        self.cursor = (0,0);

        (self.screen_size, terminal) = if let Some(terminal) = terminal {(Self::get_size(terminal)?, Some(terminal))} else {((0,0), None)};
        self.block_size = 8;
        self.vertical_margin = 2;
        self.blocks_per_row = Self::calc_blocks_per_row(self.block_size, self.screen_size.0);

        terminal = if let Some(terminal) = terminal
        {
            Self::print_loading_status(&self.settings.color, &format!("Opening \"{}\"...", path), terminal)?;
            Some(terminal)
        } else {None};
        self.data = self.filesystem.read(self.filesystem.pwd())?;
        
        terminal = if let Some(terminal) = terminal
        {
            Self::print_loading_status(&self.settings.color, "Decoding binary data...", terminal)?;
            Some(terminal)
        } else {None};
        self.header = Header::parse_header(&self.data, path, &self.filesystem);

        terminal = if let Some(terminal) = terminal
        {
            Self::print_loading_status(&self.settings.color, "Disassembling executable...", terminal)?;
            Some(terminal)
        } else {None};
        (self.assembly_offsets, self.assembly_instructions) = Self::sections_from_bytes(&self.data, &self.header);

        if let Some(terminal) = terminal
        {
            Self::print_loading_status(&self.settings.color, "Opening ui...", terminal)?;
        }
        self.log_header_info();

        Ok(())
    }
}