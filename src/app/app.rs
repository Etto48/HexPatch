#![allow(clippy::module_inception)]
use std::time::Duration;

use crossterm::event;
use ratatui::{backend::Backend, layout::Rect, text::{Line, Text}, widgets::{Block, Borders, Clear}};

use super::{assembly::AssemblyLine, files::filesystem::FileSystem, help::HelpLine, info_mode::InfoMode, log::{logger::Logger, NotificationLevel}, plugins::plugin_manager::PluginManager, popup_state::PopupState, settings::{color_settings::ColorSettings, Settings}, widgets::{logo::Logo, scrollbar::Scrollbar}};

use crate::{args::Args, headers::Header};

pub struct App 
{
    pub(super) plugin_manager: PluginManager,
    pub(super) filesystem: FileSystem,
    pub(super) header: Header,
    pub(super) logger: Logger,
    pub(super) help_list: Vec<HelpLine>,
    pub(super) dirty: bool,
    pub(super) data: Vec<u8>,
    pub(super) assembly_offsets: Vec<usize>,
    pub(super) assembly_instructions: Vec<AssemblyLine>,
    pub(super) text_last_searched_string: String,
    pub(super) info_mode: InfoMode,
    pub(super) scroll: usize,
    pub(super) cursor: (u16, u16),
    pub(super) poll_time: Duration,
    pub(super) needs_to_exit: bool,
    pub(super) screen_size: (u16, u16),

    pub(super) settings: Settings,

    pub(super) popup: Option<PopupState>,

    pub(super) vertical_margin: u16,
    pub(super) block_size: usize,
    pub(super) blocks_per_row: usize,
}

impl App
{
    pub(super) fn print_loading_status<B: Backend>(color_settings: &ColorSettings, status: &str, terminal: &mut ratatui::Terminal<B>) -> Result<(), String>
    {
        terminal.draw(|f|{
            let size = f.size();
            let mut text = Text::default();
            for _ in 0..(size.height.saturating_sub(1))
            {
                text.lines.push(ratatui::text::Line::default());
            }
            text.lines.push(Line::styled(status.to_string(), color_settings.menu_text));
            let paragraph = ratatui::widgets::Paragraph::new(text)
                .block(Block::default().borders(Borders::NONE));
            let logo = Logo::default();
            let logo_size = logo.get_size();
            f.render_widget(paragraph, size);
            if logo_size.0 < size.width && logo_size.1 < size.height
            {
                f.render_widget(logo, Rect::new( size.width / 2 - logo_size.0 / 2, size.height / 2 - logo_size.1 / 2, logo_size.0, logo_size.1));
            }
        }).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub(super) fn get_size<B: Backend>(terminal: &mut ratatui::Terminal<B>) -> Result<(u16, u16), String>
    {
        terminal.size().map_err(|e| e.to_string()).map(|s| (s.width, s.height))
    }

    pub fn new<B: Backend>(args: Args, terminal: &mut ratatui::Terminal<B>) -> Result<Self,String>
    {
        let mut logger = Logger::new();
        let mut settings = match Settings::load_or_create(args.config.as_deref())
        {
            Ok(settings) => settings,
            Err(e) => {
                logger.log(NotificationLevel::Error, 
                    &format!("Error loading settings: {e}"));
                Settings::default()
            },
        };
        let plugin_manager = match PluginManager::load(args.plugins.as_deref(), &mut logger, &mut settings)
        {
            Ok(plugins) => plugins,
            Err(e) => {
                logger.log(NotificationLevel::Error, 
                    &format!("Error loading plugins: {e}"));
                PluginManager::default()
            },
        };
        Self::print_loading_status(&settings.color, &format!("Opening \"{}\"...", args.path), terminal)?;

        let filesystem = if let Some(ssh) = &args.ssh
        {
            FileSystem::new_remote(&args.path, ssh, args.password.as_deref())
                .map_err(|e|format!("Failed to connect to {}: {e}", ssh))?
        }
        else
        {
            FileSystem::new_local(&args.path)
                .map_err(|e|e.to_string())?
        };
        let screen_size = Self::get_size(terminal)?;

        let mut app = App
        {
            plugin_manager,
            filesystem,
            screen_size,
            help_list: Self::help_list(&settings.key),
            settings,
            logger,
            ..Default::default()
        };

        if app.filesystem.is_file(app.filesystem.pwd())
        {
            let path = app.filesystem.pwd().to_string();
            app.open_file(&path, Some(terminal)).map_err(|e| e.to_string())?;
        }
        else
        {
            let dir = app.filesystem.pwd().to_string();
            Self::open_dir(&mut app.popup, &dir, &mut app.filesystem).map_err(|e| e.to_string())?;
        }

        Ok(app)
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> Result<(),Box<dyn std::error::Error>>
    {
        self.screen_size = (terminal.size()?.width, terminal.size()?.height);
        self.resize_to_size(self.screen_size.0, self.screen_size.1);

        while !self.needs_to_exit
        {
            if event::poll(self.poll_time)?
            {
                while event::poll(Duration::from_millis(0))?
                {
                    let event = event::read()?;
                    let event_result = self.handle_event(event, terminal);
                    if let Err(e) = event_result
                    {
                        self.log(NotificationLevel::Error, &e.to_string());
                    }
                }
            }

            terminal.draw(|f| {
                let min_width = self.block_size as u16 * 3 + 17 + 3;
                if f.size().width < min_width
                {
                    return;
                }
                let output_rect = Rect::new(0, f.size().height - 1, f.size().width, 1);
                let address_rect = Rect::new(0, 0, 17, f.size().height - output_rect.height);
                let hex_editor_rect = Rect::new(address_rect.width, 0, (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as u16, f.size().height - output_rect.height);
                let info_view_rect = Rect::new(address_rect.width + hex_editor_rect.width, 0, f.size().width - hex_editor_rect.width - address_rect.width - 2, f.size().height - output_rect.height);
                let scrollbar_rect = Rect::new(f.size().width - 1, 0, 1, f.size().height);

                let output_block = ratatui::widgets::Paragraph::new(self.build_status_bar())
                    .block(Block::default().borders(Borders::NONE));
                
                let scrolled_amount = self.get_cursor_position().global_byte_index;
                let total_amount = self.data.len();
                let scrollbar = Scrollbar::new(scrolled_amount, total_amount, self.settings.color.scrollbar);

                if !self.data.is_empty()
                {
                    let line_start_index = self.scroll;
                    let line_end_index = (self.scroll + f.size().height as usize).saturating_sub(2);

                    let address_view = self.get_address_view(line_start_index,line_end_index);
                    let hex_view = self.get_hex_view(line_start_index,line_end_index);

                    let address_block = ratatui::widgets::Paragraph::new(address_view)
                        .block(Block::default().title("Address").borders(Borders::LEFT | Borders::TOP));
                    
                    let editor_title = format!("Hex Editor{}", if self.dirty { " *"} else {""});

                    let hex_editor_block = ratatui::widgets::Paragraph::new(hex_view)
                        .block(Block::default().title(editor_title).borders(Borders::LEFT | Borders::TOP | Borders::RIGHT));
                    
                    let info_view_block = 
                    match &self.info_mode 
                    {
                        InfoMode::Text =>
                        {
                            let text_subview_lines = self.get_text_view(line_start_index,line_end_index);
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
                            let address_min_width = self.assembly_instructions.last().map(|x| format!("{:X}",x.file_address()).len() + 1).unwrap_or(1);
                            assembly_subview.lines.extend(assembly_subview_lines.iter().map(|x| x.to_line(&self.settings.color, self.get_cursor_position().global_byte_index, &self.header, address_min_width)));
                            ratatui::widgets::Paragraph::new(assembly_subview)
                                .block(Block::default().title("Assembly View").borders(Borders::TOP | Borders::RIGHT))
                        }
                    };

                    f.render_widget(address_block, address_rect);
                    f.render_widget(hex_editor_block, hex_editor_rect);
                    f.render_widget(info_view_block, info_view_rect);
                }
                f.render_widget(output_block, output_rect);
                f.render_widget(scrollbar, scrollbar_rect);

                if let Some(popup_state) = &self.popup 
                {
                    let mut popup_text = Text::default();
                    let mut popup_title = "Popup";

                    let mut popup_width = 60;
                    let mut popup_height = 5;

                    let popup_result = self.fill_popup(&self.settings.color, popup_state, &mut popup_title, &mut popup_text, &mut popup_height, &mut popup_width);
                    popup_height = popup_height.min(f.size().height.saturating_sub(2) as usize);
                    popup_width = popup_width.min(f.size().width.saturating_sub(1) as usize);
                    let popup_rect = Rect::new((f.size().width / 2).saturating_sub((popup_width / 2 + 1) as u16), (f.size().height / 2).saturating_sub((popup_height / 2) as u16), popup_width as u16, popup_height as u16);

                    if popup_result.is_ok()
                    {
                        let popup = ratatui::widgets::Paragraph::new(popup_text)
                            .block(Block::default().title(popup_title).borders(Borders::ALL))
                            .alignment(ratatui::layout::Alignment::Center);
                        f.render_widget(Clear, popup_rect);
                        f.render_widget(popup, popup_rect);
                    }
                }

            })?;
        }

        Ok(())
    }
}

impl Default for App
{
    fn default() -> Self {
        App{
            plugin_manager: PluginManager::default(),
            filesystem: FileSystem::default(),
            header: Header::None,
            logger: Logger::new(),
            help_list: Self::help_list(&Settings::default().key),
            data: Vec::new(),
            dirty: false,
            assembly_offsets: Vec::new(),
            assembly_instructions: Vec::new(),
            text_last_searched_string: String::new(),
            info_mode: InfoMode::Text,
            scroll: 0,
            cursor: (0,0),
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size: (0,0),

            settings: Settings::default(),

            popup: None,

            vertical_margin: 2,
            block_size: 8,
            blocks_per_row: 1,
        }
    }
}