#![allow(clippy::module_inception)]
use std::time::Duration;

use crossterm::event;
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, ScrollbarOrientation, ScrollbarState},
};
use termbg::Theme;

use super::{
    asm::assembly_line::AssemblyLine,
    comments::Comments,
    data::Data,
    files::filesystem::FileSystem,
    frame_info::{FrameInfo, InfoViewFrameInfo},
    help::HelpLine,
    info_mode::InfoMode,
    log::{logger::Logger, NotificationLevel},
    pane::Pane,
    plugins::plugin_manager::PluginManager,
    popup::popup_state::PopupState,
    settings::{color_settings::ColorSettings, Settings},
    widgets::logo::Logo,
};

use crate::{args::Args, get_app_context, headers::Header};

pub struct App {
    pub(super) plugin_manager: PluginManager,
    pub(super) filesystem: FileSystem,
    pub(super) header: Header,
    pub(super) logger: Logger,
    pub(super) help_list: Vec<HelpLine>,
    pub(super) data: Data,
    pub(super) comments: Comments,
    pub(super) assembly_offsets: Vec<usize>,
    pub(super) assembly_instructions: Vec<AssemblyLine>,
    pub(super) text_last_searched_string: String,
    pub(super) info_mode: InfoMode,
    pub(super) scroll: usize,
    pub(super) cursor: (u16, u16),
    pub(super) selected_pane: Pane,
    pub(super) fullscreen: bool,
    pub(super) poll_time: Duration,
    pub(super) needs_to_exit: bool,
    pub(super) screen_size: (u16, u16),

    pub(super) settings: Settings,

    pub(super) popup: Option<PopupState>,

    pub(super) vertical_margin: u16,
    pub(super) block_size: usize,
    pub(super) blocks_per_row: usize,

    pub(super) last_frame_info: FrameInfo,
}

impl App {
    pub(super) fn print_loading_status<B: Backend>(
        color_settings: &ColorSettings,
        status: &str,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<(), String> {
        terminal
            .draw(|f| {
                let area = f.area();
                let mut text = Text::default();
                for _ in 0..(area.height.saturating_sub(1)) {
                    text.lines.push(ratatui::text::Line::default());
                }
                text.lines
                    .push(Line::styled(status.to_string(), color_settings.menu_text));
                let paragraph = ratatui::widgets::Paragraph::new(text)
                    .block(Block::default().borders(Borders::NONE));
                let logo = Logo::default();
                let logo_size = logo.get_size();
                f.render_widget(paragraph, area);
                if logo_size.0 < area.width && logo_size.1 < area.height {
                    f.render_widget(
                        logo,
                        Rect::new(
                            area.width / 2 - logo_size.0 / 2,
                            area.height / 2 - logo_size.1 / 2,
                            logo_size.0,
                            logo_size.1,
                        ),
                    );
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub(super) fn get_size<B: Backend>(
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<(u16, u16), String> {
        terminal
            .size()
            .map_err(|e| e.to_string())
            .map(|s| (s.width, s.height))
    }

    pub(super) fn switch_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    pub fn new<B: Backend>(
        args: Args,
        terminal: &mut ratatui::Terminal<B>,
        terminal_theme: Result<Theme, termbg::Error>,
    ) -> Result<Self, String> {
        let mut logger = Logger::default();
        let terminal_theme = match terminal_theme {
            Ok(theme) => theme,
            Err(e) => {
                logger.log(
                    NotificationLevel::Debug,
                    t!("errors.detect_terminal_theme", e = e),
                );
                Theme::Dark
            }
        };
        let settings = match Settings::load_or_create(args.config.as_deref(), terminal_theme) {
            Ok(settings) => settings,
            Err(e) => {
                logger.log(NotificationLevel::Error, t!("errors.load_settings", e = e));
                Settings::default()
            }
        };
        logger.change_limit(settings.app.log_limit);
        logger.change_verbosity(settings.app.log_level);
        Self::print_loading_status(
            &settings.color,
            &t!("app.messages.opening_path", path = &args.path),
            terminal,
        )?;

        let filesystem = if let Some(ssh) = &args.ssh {
            FileSystem::new_remote(&args.path, ssh, args.password.as_deref())
                .map_err(|e| t!("errors.connect_ssh", ssh = ssh, e = e))?
        } else {
            FileSystem::new_local(&args.path).map_err(|e| e.to_string())?
        };
        let screen_size = Self::get_size(terminal)?;

        let mut app = App {
            filesystem,
            screen_size,
            help_list: Self::help_list(&settings.key),
            settings,
            logger,
            ..Default::default()
        };

        let mut app_context = get_app_context!(app);
        app.plugin_manager = match PluginManager::load(args.plugins.as_deref(), &mut app_context) {
            Ok(plugins) => plugins,
            Err(e) => {
                app.log(NotificationLevel::Error, t!("errors.load_plugins", e = e));
                PluginManager::default()
            }
        };

        if app.filesystem.is_file(app.filesystem.pwd()) {
            let path = app.filesystem.pwd().to_string();
            app.open_file(&path, terminal).map_err(|e| e.to_string())?;
        } else {
            let dir = app.filesystem.pwd().to_string();
            Self::open_dir(&mut app.popup, &dir, &mut app.filesystem).map_err(|e| e.to_string())?;
        }

        Ok(app)
    }

    pub fn draw<B: Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|f| {
            let screen_size = (f.area().width, f.area().height);
            self.resize_to_size(screen_size.0, screen_size.1);

            let min_width = self.block_size as u16 * 3 + 17 + 3;
            if f.area().width < min_width {
                return;
            }
            let status_rect = Rect::new(0, f.area().height - 1, f.area().width, 1);
            let address_rect = Rect::new(0, 0, 17, f.area().height - status_rect.height);
            let hex_editor_rect: Rect;
            let info_view_rect: Rect;
            if self.fullscreen {
                hex_editor_rect = Rect::new(
                    address_rect.width,
                    0,
                    f.area().width - address_rect.width - 2,
                    f.area().height - status_rect.height,
                );
                info_view_rect = Rect::new(
                    address_rect.width,
                    0,
                    f.area().width - address_rect.width - 2,
                    f.area().height - status_rect.height,
                );
            } else {
                hex_editor_rect = Rect::new(
                    address_rect.width,
                    0,
                    (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as u16,
                    f.area().height - status_rect.height,
                );
                info_view_rect = Rect::new(
                    address_rect.width + hex_editor_rect.width,
                    0,
                    f.area().width - hex_editor_rect.width - address_rect.width - 2,
                    f.area().height - status_rect.height,
                );
            }

            let scrollbar_rect = Rect::new(f.area().width - 1, 0, 1, f.area().height);

            let status_block = ratatui::widgets::Paragraph::new(self.build_status_bar())
                .block(Block::default().borders(Borders::NONE));

            let scrolled_amount = self.get_cursor_position().global_byte_index;
            let total_amount = self.data.len();
            let scrollbar = ratatui::widgets::Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(self.settings.color.scrollbar)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(total_amount).position(scrolled_amount);

            let mut info_view_frame_info = InfoViewFrameInfo::TextView;

            if !self.data.is_empty() {
                let line_start_index = self.scroll;
                let line_end_index = (self.scroll + f.area().height as usize).saturating_sub(2);

                let address_view = self.get_address_view(line_start_index, line_end_index);
                let hex_view = self.get_hex_view(line_start_index, line_end_index);

                let address_block = ratatui::widgets::Paragraph::new(address_view).block(
                    Block::default()
                        .title(t!("app.address_view_title"))
                        .borders(Borders::LEFT | Borders::TOP),
                );

                let editor_title = t!(
                    "app.hex_view_title",
                    dirty = if self.data.dirty() { " *" } else { "" }
                );

                let hex_border_style: Style;
                let pretty_border_style: Style;
                if self.selected_pane == Pane::Hex {
                    hex_border_style = self.settings.color.pane_selected;
                    pretty_border_style = self.settings.color.pane;
                } else {
                    hex_border_style = self.settings.color.pane;
                    pretty_border_style = self.settings.color.pane_selected;
                }

                let hex_editor_block = ratatui::widgets::Paragraph::new(hex_view).block(
                    Block::default()
                        .title(editor_title)
                        .borders(Borders::LEFT | Borders::TOP | Borders::RIGHT)
                        .border_style(hex_border_style),
                );

                let info_view_block_flags = if self.fullscreen {
                    Borders::TOP | Borders::RIGHT | Borders::LEFT
                } else {
                    Borders::TOP | Borders::RIGHT
                };

                let info_view_block = match &self.info_mode {
                    InfoMode::Text => {
                        let text_subview_lines =
                            self.get_text_view(line_start_index, line_end_index);
                        info_view_frame_info = InfoViewFrameInfo::TextView;
                        let mut text_subview = Text::default();
                        text_subview
                            .lines
                            .extend(text_subview_lines.iter().cloned());
                        ratatui::widgets::Paragraph::new(text_subview).block(
                            Block::default()
                                .title(t!("app.text_view_title"))
                                .borders(info_view_block_flags)
                                .border_style(pretty_border_style),
                        )
                    }
                    InfoMode::Assembly => {
                        let assembly_start_index = self.get_assembly_view_scroll();
                        info_view_frame_info = InfoViewFrameInfo::AssemblyView {
                            scroll: assembly_start_index,
                        };
                        let assembly_end_index = (assembly_start_index + f.area().height as usize
                            - 2)
                        .min(self.assembly_instructions.len());
                        let assembly_subview_lines =
                            &self.assembly_instructions[assembly_start_index..assembly_end_index];
                        let mut assembly_subview = Text::default();
                        let address_min_width = self
                            .assembly_instructions
                            .last()
                            .map(|x| format!("{:X}", x.file_address()).len() + 1)
                            .unwrap_or(1);
                        assembly_subview
                            .lines
                            .extend(assembly_subview_lines.iter().map(|x| {
                                x.to_line(
                                    &self.settings.color,
                                    self.get_cursor_position().global_byte_index,
                                    &self.header,
                                    address_min_width,
                                    &self.comments,
                                )
                            }));
                        ratatui::widgets::Paragraph::new(assembly_subview).block(
                            Block::default()
                                .title(t!("app.assembly_view_title"))
                                .borders(info_view_block_flags)
                                .border_style(pretty_border_style),
                        )
                    }
                };

                f.render_widget(address_block, address_rect);
                if self.fullscreen {
                    match self.selected_pane {
                        Pane::Hex => f.render_widget(hex_editor_block, hex_editor_rect),
                        Pane::View => f.render_widget(info_view_block, info_view_rect),
                    }
                } else {
                    f.render_widget(hex_editor_block, hex_editor_rect);
                    f.render_widget(info_view_block, info_view_rect);
                }
            }
            f.render_widget(status_block, status_rect);
            f.render_stateful_widget(scrollbar, scrollbar_rect, &mut scrollbar_state);

            let mut this_frame_info = FrameInfo {
                popup: None,
                status_bar: status_rect,
                scroll_bar: scrollbar_rect,
                address_view: address_rect,
                hex_view: if !self.fullscreen || self.selected_pane == Pane::Hex {
                    Some(hex_editor_rect)
                } else {
                    None
                }, // only save the hex view rect if it's visible, we need to know if it's visible to
                // determine the cursor position
                info_view: if !self.fullscreen || self.selected_pane == Pane::View {
                    Some(info_view_rect)
                } else {
                    None
                }, // only save the info view rect if it's visible, we need to know if it's visible to
                // determine the cursor position
                info_view_frame_info,
                blocks_per_row: self.blocks_per_row,
                scroll: self.scroll,
                file_size: self.data.len(),
            };

            // Draw popup
            if self.popup.is_some() {
                let mut popup_text = Text::default();
                let mut popup_title = t!("app.default_popup_title").into();

                let mut popup_width = 60;
                let mut popup_height = 5;

                let popup_result = self.fill_popup(
                    &mut popup_title,
                    &mut popup_text,
                    &mut popup_height,
                    &mut popup_width,
                );

                popup_height = popup_height.min(f.area().height.saturating_sub(2) as usize);
                popup_width = popup_width.min(f.area().width.saturating_sub(1) as usize);
                let popup_rect = Rect::new(
                    (f.area().width / 2).saturating_sub((popup_width / 2 + 1) as u16),
                    (f.area().height / 2).saturating_sub((popup_height / 2) as u16),
                    popup_width as u16,
                    popup_height as u16,
                );

                match popup_result {
                    Ok(()) => {
                        let popup = ratatui::widgets::Paragraph::new(popup_text)
                            .block(Block::default().title(popup_title).borders(Borders::ALL))
                            .alignment(ratatui::layout::Alignment::Center);
                        f.render_widget(Clear, popup_rect);
                        f.render_widget(popup, popup_rect);
                    }
                    Err(e) => {
                        self.logger.log(
                            NotificationLevel::Error,
                            t!("app.messages.popup_error", e = e),
                        );
                    }
                }
                this_frame_info.popup = Some(popup_rect)
            }
            self.last_frame_info = this_frame_info;
        })?;

        Ok(())
    }

    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.screen_size = (terminal.size()?.width, terminal.size()?.height);
        self.resize_to_size(self.screen_size.0, self.screen_size.1);

        while !self.needs_to_exit {
            if event::poll(self.poll_time)? {
                while event::poll(Duration::from_millis(0))? {
                    let event = event::read()?;
                    let event_result = self.handle_event(event, terminal);
                    if let Err(e) = event_result {
                        self.log(NotificationLevel::Error, e.to_string());
                    }
                }
            }

            self.draw(terminal)?;
        }

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            plugin_manager: PluginManager::default(),
            filesystem: FileSystem::default(),
            header: Header::None,
            logger: Logger::default(),
            help_list: Self::help_list(&Settings::default().key),
            data: Data::default(),
            comments: Comments::default(),
            assembly_offsets: Vec::new(),
            assembly_instructions: Vec::new(),
            text_last_searched_string: String::new(),
            info_mode: InfoMode::Text,
            scroll: 0,
            cursor: (0, 0),
            selected_pane: Pane::Hex,
            fullscreen: false,
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size: (0, 0),

            settings: Settings::default(),

            popup: None,

            vertical_margin: 2,
            block_size: 8,
            blocks_per_row: 1,

            last_frame_info: FrameInfo {
                popup: None,
                status_bar: Rect::default(),
                scroll_bar: Rect::default(),
                address_view: Rect::default(),
                hex_view: Some(Rect::default()),
                info_view: Some(Rect::default()),
                info_view_frame_info: InfoViewFrameInfo::TextView,
                blocks_per_row: 1,
                scroll: 0,
                file_size: 0,
            },
        }
    }
}
