use std::{error::Error, path::PathBuf};

use crate::{app::{info_mode::InfoMode, notification::NotificationLevel, popup_state::PopupState, App}, headers::header::Header};

use super::path_result::PathResult;

impl <'a> App<'a>
{
    pub(in crate::app) fn go_to_path(&mut self, currently_open_path: &PathBuf, path: &str, scroll: usize, popup: &mut Option<PopupState>) -> Result<(), Box<dyn Error>>
    {
        let contents = Self::find_dir_contents(currently_open_path, path)?;
        if contents.is_empty()
        {
            return Err(format!("No files found that matches \"{}\"", path).into());
        }
        let selected = contents.into_iter().skip(scroll).next().expect("Scroll out of bounds for go_to_path.");

        if selected.is_dir()
        {
            Self::open_dir(popup, currently_open_path.join(selected.path()))?;
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

    pub(in crate::app) fn find_dir_contents(currently_open_path: &PathBuf, path: &str) -> Result<Vec<PathResult>, Box<dyn Error>>
    {
        let mut ret = Vec::new();

        let entries = std::fs::read_dir(currently_open_path)?;
        let mut entries = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().strip_prefix(currently_open_path)
                .expect("The entry should be a child of the currently open path.").to_string_lossy().to_string())
            .collect::<Vec<_>>();


        if currently_open_path.parent().is_some()
        {
            entries.insert(0, "..".into());
        }

        let entries = entries
            .into_iter()
            .filter(|entry| entry.to_lowercase().starts_with(&path.to_lowercase()));

        for entry in entries
        {
            let path = currently_open_path.join(entry);
            if let Ok(path) = Self::path_canonicalize(path, Some(&currently_open_path))
            {
                ret.push(PathResult::new(&path, &currently_open_path)?);
            }
        }

        Ok(ret)
    }

    pub(in crate::app) fn open_dir(popup: &mut Option<PopupState>, path: PathBuf) -> Result<(), Box<dyn Error>>
    {
        let path = Self::path_canonicalize(path, None)?;
        *popup = Some(PopupState::Open { 
            currently_open_path: path.clone(),
            path: "".into(), 
            cursor: 0, 
            results: Self::find_dir_contents(&path, "")?, 
            scroll: 0 });
        Ok(())
    }

    pub fn log_header_info(&mut self)
    {
        if self.header != Header::None
        {
            match &self.header
            {
                Header::Elf(_) => self.log(NotificationLevel::Info,"Loaded ELF file."),
                Header::PE(_) => self.log(NotificationLevel::Info,"Loaded PE file."),
                Header::None => unreachable!(),
            }
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
        
        self.log(NotificationLevel::Info, "Press H for a list of commands.");
    }

    pub(in crate::app) fn open_file(&mut self, path: &str) -> std::io::Result<()>
    {
        self.log(NotificationLevel::Info, &format!("Opening file: \"{}\"", path));

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
        self.log_header_info();

        Ok(())
    }

    pub(in crate::app) fn path_canonicalize(path: PathBuf, base_path: Option<&PathBuf>) -> Result<PathBuf, Box<dyn Error>>
    {
        let path_res = path.canonicalize();
        let mut path = match path_res
        {
            Ok(path) => path,
            Err(_) => {
                return Err(format!("Failed to canonicalize path \"{}\"", path.to_string_lossy()).into());
            }
        };
        if let Some(base_path) = base_path
        {
            let base_path = base_path.canonicalize()?;
            path = pathdiff::diff_paths(&path, base_path).expect("Failed to get relative path.");
        }
        Ok(path)
    }
}