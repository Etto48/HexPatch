#![allow(clippy::module_inception)]
use std::error::Error;

use ratatui::{backend::Backend, Terminal};

use crate::{
    app::{
        data::Data, info_mode::InfoMode, log::NotificationLevel, popup::popup_state::PopupState,
        App,
    },
    get_app_context,
    headers::Header,
};

use super::{filesystem::FileSystem, path, path_result::PathResult};

impl App {
    pub(in crate::app) fn go_to_path<B: Backend>(
        &mut self,
        currently_open_path: &str,
        path: &str,
        scroll: usize,
        popup: &mut Option<PopupState>,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        let contents = Self::find_dir_contents(currently_open_path, path, &self.filesystem)?;
        if contents.is_empty() {
            return Err(format!("No files found that matches \"{}\"", path).into());
        }
        let selected = contents
            .into_iter()
            .nth(scroll)
            .expect("Scroll out of bounds for go_to_path.");

        if self.filesystem.is_dir(selected.path()) {
            Self::open_dir(popup, selected.path(), &mut self.filesystem)?;
        } else {
            self.open_file(selected.path(), terminal)?;
            *popup = None;
        }

        Ok(())
    }

    pub(in crate::app) fn get_current_dir(&self) -> String {
        let current_path = self.filesystem.pwd();
        if self.filesystem.is_dir(current_path) {
            current_path.to_owned()
        } else {
            path::parent(current_path)
                .expect("A file should have a parent directory.")
                .to_owned()
        }
    }

    pub(in crate::app) fn find_dir_contents(
        currently_open_path: &str,
        path: &str,
        filesystem: &FileSystem,
    ) -> Result<Vec<PathResult>, Box<dyn Error>> {
        let mut ret = Vec::new();
        let (selected_dir, file_name) = if path::is_absolute(path) {
            if filesystem.is_dir(path) {
                (filesystem.canonicalize(path)?, "".to_string())
            } else if let Some(parent) = path::parent(path) {
                if filesystem.is_dir(parent) {
                    (
                        filesystem.canonicalize(parent)?,
                        path::filename(path).map_or("".into(), |name| name.to_string()),
                    )
                } else {
                    (currently_open_path.to_string(), path.to_string())
                }
            } else {
                (currently_open_path.to_string(), path.to_string())
            }
        } else {
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

        for entry in entries {
            if let Ok(result) = PathResult::new(
                &path::join(&selected_dir, &entry, filesystem.separator()),
                filesystem,
            ) {
                ret.push(result);
            }
        }

        Ok(ret)
    }

    pub(in crate::app) fn open_dir(
        popup: &mut Option<PopupState>,
        path: &str,
        filesystem: &mut FileSystem,
    ) -> Result<(), Box<dyn Error>> {
        let path = filesystem.canonicalize(path)?;
        *popup = Some(PopupState::Open {
            currently_open_path: path.clone(),
            path: "".into(),
            cursor: 0,
            results: Self::find_dir_contents(&path, "", filesystem)?,
            scroll: 0,
        });
        Ok(())
    }

    pub fn log_header_info(&mut self) {
        if self.header != Header::None {
            match &self.header {
                Header::GenericHeader(header) => self.log(
                    NotificationLevel::Info,
                    &format!("File type: {:?}", header.file_type()),
                ),
                // TODO: maybe add info for a more detailed log
                Header::CustomHeader(_) => self.log(NotificationLevel::Info, "File type: Custom"),
                Header::None => unreachable!(),
            }
            self.log(
                NotificationLevel::Info,
                &format!("Architecture: {:?}", self.header.architecture()),
            );
            self.log(
                NotificationLevel::Info,
                &format!("Bitness: {}", self.header.bitness()),
            );
            self.log(
                NotificationLevel::Info,
                &format!("Entry point: {:#X}", self.header.entry_point()),
            );
            for section in self.header.get_sections() {
                self.log(NotificationLevel::Info, &format!("Section: {}", section));
            }
        } else {
            self.log(NotificationLevel::Info, "No header found. Assuming 64-bit.");
        }

        self.log(
            NotificationLevel::Info,
            &format!(
                "Press {} for a list of commands.",
                Self::key_event_to_string(self.settings.key.help)
            ),
        );
    }

    pub(in crate::app) fn open_file<B: Backend>(
        &mut self,
        path: &str,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        self.log(
            NotificationLevel::Info,
            &format!("Opening file: \"{}\"", path),
        );

        self.filesystem.cd(path);
        self.info_mode = InfoMode::Text;
        self.scroll = 0;
        self.cursor = (0, 0);

        self.screen_size = Self::get_size(terminal)?;
        self.block_size = 8;
        self.vertical_margin = 2;
        self.blocks_per_row = Self::calc_blocks_per_row(
            self.block_size,
            self.screen_size.0,
            self.fullscreen,
            self.selected_pane,
        );

        Self::print_loading_status(
            &self.settings.color,
            &format!("Opening \"{}\"...", path),
            terminal,
        )?;
        self.data = Data::new(
            self.filesystem.read(self.filesystem.pwd())?,
            self.settings.app.history_limit,
        );

        Self::print_loading_status(&self.settings.color, "Decoding binary data...", terminal)?;

        self.header = self.parse_header();

        Self::print_loading_status(
            &self.settings.color,
            "Disassembling executable...",
            terminal,
        )?;

        (self.assembly_offsets, self.assembly_instructions) =
            Self::sections_from_bytes(self.data.bytes(), &self.header);

        Self::print_loading_status(&self.settings.color, "Opening ui...", terminal)?;
        self.log_header_info();
        let mut app_context = get_app_context!(self);
        self.plugin_manager.on_open(&mut app_context);

        Ok(())
    }

    pub(in crate::app) fn save_file_as(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path::parent(path) {
            self.filesystem.mkdirs(parent)?;
        };

        self.filesystem.create(path)?;
        self.filesystem.cd(&self.filesystem.canonicalize(path)?);
        self.save_file()?;
        Ok(())
    }

    pub(in crate::app) fn save_file(&mut self) -> Result<(), Box<dyn Error>> {
        let mut app_context = get_app_context!(self);
        self.plugin_manager.on_save(&mut app_context);
        self.filesystem
            .write(self.filesystem.pwd(), self.data.bytes())?;
        self.data.reset_dirty();
        match &self.filesystem {
            FileSystem::Local { path } => {
                self.log(NotificationLevel::Info, &format!("Saved to {}", path));
            }
            FileSystem::Remote { path, connection } => {
                self.log(
                    NotificationLevel::Info,
                    &format!("Saved to {} at {}", path, connection),
                );
            }
        }

        Ok(())
    }
}
