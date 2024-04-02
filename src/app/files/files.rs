use std::path::PathBuf;

use crate::{app::{info_mode::InfoMode, popup_state::PopupState, App}, headers::header::Header};

use super::path_result::PathResult;

impl <'a> App<'a>
{
    pub(in crate::app) fn go_to_path(&mut self, currently_open_path: &PathBuf, path: &str, scroll: usize, popup: &mut Option<PopupState>) -> std::io::Result<()>
    {
        let contents = self.find_dir_contents(currently_open_path, path)?;
        let selected = contents.into_iter().skip(scroll).next().expect("Scroll out of bounds for go_to_path.");

        if selected.is_dir()
        {
            self.open_dir(popup, &selected.path().to_string_lossy())?;
        }
        else
        {
            self.open_file(&selected.path().to_string_lossy())?;
            *popup = None;
        }
        
        Ok(())
    }

    pub(in crate::app) fn get_current_dir(&self) -> PathBuf
    {
        let current_path = self.path.clone();
        if current_path.is_dir()
        {
            current_path
        }
        else 
        {
            current_path.parent().expect("A file should have a parent directory.").to_path_buf()
        }
    }

    pub(in crate::app) fn find_dir_contents(&self, currently_open_path: &PathBuf, path: &str) -> std::io::Result<Vec<PathResult>>
    {
        let upper_dir_path = currently_open_path.parent();
        let mut ret = Vec::new();
        if let Some(upper_dir_path) = upper_dir_path
        {
            ret.push(PathResult::Directory(upper_dir_path.to_string_lossy().to_string()));
        }

        let dir = if path.len() == 0
        {
            currently_open_path.clone()
        }
        else
        {
            currently_open_path.join(path)
        };
        let entries = std::fs::read_dir(dir)?;
        for entry in entries
        {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            if path.is_dir()
            {
                ret.push(PathResult::Directory(path_str));
            }
            else
            {
                ret.push(PathResult::File(path_str));
            }
        }

        Ok(ret)
    }

    pub(in crate::app) fn open_dir(&mut self, popup: &mut Option<PopupState>, path: &str) -> std::io::Result<()>
    {
        let path = PathBuf::from(path).canonicalize()?;
        *popup = Some(PopupState::Open { 
            currently_open_path: path.clone(), 
            path: "".into(), 
            cursor: 0, 
            results: self.find_dir_contents(&path, "")?, 
            scroll: 0 });
        Ok(())
    }

    pub(in crate::app) fn open_file(&mut self, path: &str) -> std::io::Result<()>
    {
        self.path = path.into();
        self.dirty = false;
        self.address_last_row = 0;
        self.hex_last_byte_index = 0;
        self.hex_cursor = (0,0);
        self.text_last_byte_index = 0;
        self.text_cursor = (0,0);
        self.assembly_scroll = 0;
        self.info_mode = InfoMode::Text;
        self.scroll = 0;
        self.cursor = (0,0);

        self.data = std::fs::read(&self.path)?;
        self.text_view = Self::bytes_to_styled_text(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        self.hex_view = Self::bytes_to_styled_hex(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        
        self.address_view = Self::addresses(&self.color_settings, self.data.len(), self.block_size, self.blocks_per_row);
        //Self::print_loading_status(&self.color_settings, "Decoding binary data...", terminal)?;
        self.hex_view = Self::bytes_to_styled_hex(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        self.text_view = Self::bytes_to_styled_text(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        self.header = Header::parse_header(&self.data);
        //Self::print_loading_status(&self.color_settings, "Disassembling executable...", terminal)?;
        (self.assembly_offsets, self.assembly_instructions) = Self::sections_from_bytes(&self.data, &self.header);
        //Self::print_loading_status(&self.color_settings, "Opening ui...", terminal)?;

        Ok(())
    }
}