use std::{path::PathBuf, thread, time::Duration};

use crossterm::event;
use ratatui::{backend::Backend, layout::Rect, text::{Line, Text}, widgets::{Block, Borders}};

use super::{assembly::AssemblyLine, color_settings::{self, ColorSettings}, header::Header, info_mode::InfoMode, log::LogLine, logo::Logo, notification::NotificationLevel, popup_state::PopupState, scrollbar::Scrollbar};

pub struct App<'a> 
{
    pub(super) path: PathBuf,
    pub(super) header: Header,
    pub(super) log: Vec<LogLine>,
    pub(super) notificaiton: NotificationLevel,
    pub(super) dirty: bool,
    pub(super) data: Vec<u8>,
    pub(super) address_view: Text<'a>,
    pub(super) hex_view: Text<'a>,
    pub(super) text_view: Text<'a>,
    pub(super) assembly_offsets: Vec<usize>,
    pub(super) assembly_instructions: Vec<AssemblyLine>,
    pub(super) address_last_row: usize,
    pub(super) hex_last_byte_index: usize,
    pub(super) hex_cursor: (usize, usize),
    pub(super) text_last_byte_index: usize,
    pub(super) text_cursor: (usize, usize),
    pub(super) assembly_scroll: usize,
    pub(super) info_mode: InfoMode,
    pub(super) scroll: usize,
    pub(super) cursor: (u16, u16),
    pub(super) poll_time: Duration,
    pub(super) needs_to_exit: bool,
    pub(super) screen_size: (u16, u16),

    pub(super) color_settings: ColorSettings,

    pub(super) popup: Option<PopupState>,

    pub(super) vertical_margin: u16,
    pub(super) block_size: usize,
    pub(super) blocks_per_row: usize,
}

impl <'a> App<'a>
{
    pub(super) fn print_loading_status<B: Backend>(color_settings: &ColorSettings, status: &str, terminal: &mut ratatui::Terminal<B>) -> Result<(), String>
    {
        terminal.draw(|f|{
            let size = f.size();
            let mut text = Text::default();
            for _ in 0..(size.height.saturating_sub(3))
            {
                text.lines.push(ratatui::text::Line::default());
            }
            text.lines.push(Line::styled(status.to_string(), color_settings.ok));
            let paragraph = ratatui::widgets::Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL));
            let logo = Logo::new();
            let logo_size = logo.get_size();
            f.render_widget(paragraph, size);
            if logo_size.0 < size.width && logo_size.1 < size.height
            {
                f.render_widget(logo, Rect::new( size.width / 2 - logo_size.0 / 2, size.height / 2 - logo_size.1 / 2, logo_size.0, logo_size.1));
            }
        }).map_err(|e| e.to_string())?;
        thread::sleep(Duration::from_millis(1000));
        Ok(())
    }

    pub(super) fn get_size<B: Backend>(terminal: &mut ratatui::Terminal<B>) -> Result<(u16, u16), String>
    {
        terminal.size().map_err(|e| e.to_string()).map(|s| (s.width, s.height))
    }

    pub fn new<B: Backend>(file_path: PathBuf, terminal: &mut ratatui::Terminal<B>) -> Result<Self,String>
    {
        let color_settings = color_settings::ColorSettings::default();
        Self::print_loading_status(&color_settings, &format!("Opening {}...", file_path.to_string_lossy()), terminal)?;
        let canonical_path = file_path.canonicalize().map_err(|e| e.to_string())?;
        let data = std::fs::read(&canonical_path).map_err(|e| e.to_string())?;
        let screen_size = Self::get_size(terminal)?;
        let block_size = 8;
        let vertical_margin = 2;
        let blocks_per_row = Self::calc_blocks_per_row(block_size, screen_size.0);
        let address_view = Self::addresses(&color_settings, data.len(), block_size, blocks_per_row);
        Self::print_loading_status(&color_settings, "Decoding binary data...", terminal)?;
        let hex_view = Self::bytes_to_styled_hex(&color_settings, &data, block_size, blocks_per_row);
        let text_view = Self::bytes_to_styled_text(&color_settings, &data, block_size, blocks_per_row);
        let header = Header::parse_header(&data);
        Self::print_loading_status(&color_settings, "Disassembling executable...", terminal)?;
        let (assembly_offsets, assembly_instructions) = Self::sections_from_bytes(&data, &header);
        Self::print_loading_status(&color_settings, "Opening ui...", terminal)?;
        Ok(App{
            path: canonical_path,
            header,
            log: Vec::new(),
            notificaiton: NotificationLevel::None,
            data,
            dirty: false,
            address_view,
            hex_view,
            text_view,
            assembly_offsets,
            assembly_instructions,
            address_last_row: 0,
            hex_last_byte_index: 0,
            hex_cursor: (0,0),
            text_last_byte_index: 0,
            text_cursor: (0,0),
            assembly_scroll: 0,
            info_mode: InfoMode::Text,
            scroll: 0,
            cursor: (0,0),
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size,

            color_settings,

            popup: None,

            vertical_margin,
            block_size,
            blocks_per_row,
        })
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> Result<(),Box<dyn std::error::Error>>
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
            self.log(NotificationLevel::Info, &format!("Entry point: 0x{:X}", self.header.entry_point()));
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

        self.screen_size = (terminal.size()?.width, terminal.size()?.height);
        self.resize_if_needed(self.screen_size.0);

        while self.needs_to_exit == false 
        {
            if event::poll(self.poll_time)?
            {
                while event::poll(Duration::from_millis(0))?
                {
                    let event = event::read()?;
                    self.handle_event(event)?;
                }
            }

            terminal.draw(|f| {
                self.screen_size = (f.size().width, f.size().height);
                let output_rect = Rect::new(0, f.size().height - 1, f.size().width, 1);
                let address_rect = Rect::new(0, 0, 17, f.size().height - output_rect.height);
                let hex_editor_rect = Rect::new(address_rect.width, 0, (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as u16, f.size().height - output_rect.height);
                let info_view_rect = Rect::new(address_rect.width + hex_editor_rect.width, 0, f.size().width - hex_editor_rect.width - address_rect.width - 2, f.size().height - output_rect.height);
                let scrollbar_rect = Rect::new(f.size().width - 1, 0, 1, f.size().height);

                let output_block = ratatui::widgets::Paragraph::new(self.build_status_bar())
                    .block(Block::default().borders(Borders::NONE));
                
                let line_start_index = self.scroll;
                let line_end_index = (self.scroll + f.size().height as usize - 2).min(self.hex_view.lines.len());

                let address_subview_lines = &self.address_view.lines[line_start_index..line_end_index];
                let mut address_subview = Text::default();
                address_subview.lines.extend(address_subview_lines.iter().cloned());

                let hex_subview_lines = &self.hex_view.lines[line_start_index..line_end_index];
                let mut hex_subview = Text::default();
                hex_subview.lines.extend(hex_subview_lines.iter().cloned());

                let address_block = ratatui::widgets::Paragraph::new(address_subview)
                    .block(Block::default().title("Address").borders(Borders::LEFT | Borders::TOP));
                
                let editor_title = format!("Hex Editor{}", if self.dirty { " *"} else {""});

                let hex_editor_block = ratatui::widgets::Paragraph::new(hex_subview)
                    .block(Block::default().title(editor_title).borders(Borders::LEFT | Borders::TOP | Borders::RIGHT));
                
                let info_view_block = 
                match &self.info_mode 
                {
                    InfoMode::Text =>
                    {
                        let text_subview_lines = &self.text_view.lines[line_start_index..line_end_index];
                        let mut text_subview = Text::default();
                        text_subview.lines.extend(text_subview_lines.iter().cloned());
                        ratatui::widgets::Paragraph::new(text_subview)
                            .block(Block::default().title("Text View").borders(Borders::TOP | Borders::RIGHT))
                    },
                    InfoMode::Assembly =>
                    {
                        let assembly_start_index = self.get_assembly_view_scroll();
                        let assembly_end_index = (assembly_start_index + f.size().height as usize - 2).min(self.assembly_instructions.len());
                        let assembly_subview_lines = &self.assembly_instructions[assembly_start_index..assembly_end_index];
                        let mut assembly_subview = Text::default();
                        assembly_subview.lines.extend(assembly_subview_lines.iter().map(|x| x.to_line(&self.color_settings, self.get_cursor_position().global_byte_index, &self.header)));
                        ratatui::widgets::Paragraph::new(assembly_subview)
                            .block(Block::default().title("Assembly View").borders(Borders::TOP | Borders::RIGHT))
                    }
                };

                
                let scrolled_amount = self.get_cursor_position().global_byte_index;
                let total_amount = self.data.len();
                let scrollbar = Scrollbar::new(scrolled_amount, total_amount, self.color_settings.scrollbar);

                f.render_widget(output_block, output_rect);
                f.render_widget(address_block, address_rect);
                f.render_widget(hex_editor_block, hex_editor_rect);
                f.render_widget(info_view_block, info_view_rect);
                f.render_widget(scrollbar, scrollbar_rect);

                if let Some(popup_state) = &self.popup 
                {
                    let clear = ratatui::widgets::Clear::default();

                    let mut popup_text = Text::default();
                    let mut popup_title = "Popup";

                    let mut popup_rect = Rect::new(f.size().width / 2 - 27, f.size().height / 2 - 2, 54, 5);

                    self.fill_popup(&self.color_settings, popup_state, f, &mut popup_title, &mut popup_text, &mut popup_rect);

                    let popup = ratatui::widgets::Paragraph::new(popup_text)
                        .block(Block::default().title(popup_title).borders(Borders::ALL))
                        .alignment(ratatui::layout::Alignment::Center);
                    f.render_widget(clear, popup_rect);
                    f.render_widget(popup, popup_rect);
                }

            })?;
        }

        Ok(())
    }
}

