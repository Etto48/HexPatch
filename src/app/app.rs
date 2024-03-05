use std::{path::PathBuf, time::Duration};

use crossterm::event;
use ratatui::{backend::Backend, layout::Rect, style::{Color, Style}, text::{Line, Span, Text}, widgets::{Block, Borders, ScrollbarState}, Frame};

use super::{info_mode::InfoMode, paragraph::Paragraph, popup_state::PopupState};

pub struct App<'a> 
{
    pub(super) path: PathBuf,
    pub(super) output: String,
    pub(super) dirty: bool,
    pub(super) data: Vec<u8>,
    pub(super) address_view: Text<'a>,
    pub(super) hex_view: Text<'a>,
    pub(super) text_view: Text<'a>,
    pub(super) assembly_view: Text<'a>,
    pub(super) assembly_offsets: Vec<usize>,
    pub(super) assembly_scroll: usize,
    pub(super) info_mode: InfoMode,
    pub(super) scroll: u16,
    pub(super) cursor: (u16, u16),
    pub(super) poll_time: Duration,
    pub(super) needs_to_exit: bool,
    pub(super) screen_size: (u16, u16),

    pub(super) popup: Option<PopupState>,

    pub(super) block_size: usize,
    pub(super) blocks_per_row: usize,
}

impl <'a> App<'a>
{
    pub fn new(file_path: PathBuf) -> Result<Self,String>
    {
        let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
        if data.len() > 0xFFFF
        {
            return Err("File is too large (size > 0xFFFF Bytes), ratatui does not support this many lines for some reason.".to_string());
        }
        let block_size = 8;
        let blocks_per_row = 3;
        let address_view = Self::addresses(data.len(), block_size, blocks_per_row);
        let hex_view = Self::bytes_to_styled_hex(&data, block_size, blocks_per_row);
        let text_view = Self::bytes_to_styled_text(&data, block_size, blocks_per_row);
        let (assembly_view, assembly_offsets) = Self::assembly_from_bytes(&data);
        Ok(App{
            path: file_path,
            data,
            output: "Press H to view a help page.".to_string(),
            dirty: false,
            address_view,
            hex_view,
            text_view, 
            assembly_view,
            assembly_offsets,
            assembly_scroll: 0,
            info_mode: InfoMode::Text,
            scroll: 0,
            cursor: (0,0),
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size: (1,1),

            popup: None,

            block_size,
            blocks_per_row,
        })
    }

    pub(super) fn fill_popup(popup_state: &PopupState, f: &Frame, popup_title: &mut &str, popup_text: &mut Text, popup_rect: &mut Rect)
    {
        match &popup_state
        {
            PopupState::SaveAndQuit(yes_selected) =>
            {
                *popup_title = "Save and Quit";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file will be saved and the program will quit."),
                        Line::raw("Are you sure?"),
                        Line::from(vec![
                            Span::styled("Yes", Style::default().fg(Color::Green)),
                            Span::raw("  "),
                            Span::styled("No", Style::default().fg(Color::Red))
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = Style::default().fg(Color::White).bg(Color::Green);
                }
                else
                {
                    popup_text.lines[2].spans[2].style = Style::default().fg(Color::White).bg(Color::Red);
                }
            },
            PopupState::Save(yes_selected) =>
            {
                *popup_title = "Save";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file will be saved."),
                        Line::raw("Are you sure?"),
                        Line::from(vec![
                            Span::styled("Yes", Style::default().fg(Color::Green)),
                            Span::raw("  "),
                            Span::styled("No", Style::default().fg(Color::Red))
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = Style::default().fg(Color::White).bg(Color::Green);
                }
                else
                {
                    popup_text.lines[2].spans[2].style = Style::default().fg(Color::White).bg(Color::Red);
                }
            },
            PopupState::QuitDirtySave(yes_selected) =>
            {
                *popup_title = "Quit";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file has been modified."),
                        Line::raw("Do you want to save before quitting?"),
                        Line::from(vec![
                            Span::styled("Yes", Style::default().fg(Color::Green)),
                            Span::raw("  "),
                            Span::styled("No", Style::default().fg(Color::Red))
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = Style::default().fg(Color::White).bg(Color::Green);
                }
                else
                {
                    popup_text.lines[2].spans[2].style = Style::default().fg(Color::White).bg(Color::Red);
                }
            },
            PopupState::Help =>
            {
                *popup_rect = Rect::new(f.size().width / 2 - 15, f.size().height / 2 - 4, 30, 8);
                *popup_title = "Help";
                popup_text.lines.extend(
                    vec![
                        Line::from(
                            vec![
                                Span::styled("^S", Style::default().fg(Color::Green)),
                                Span::raw(": Save")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("^X", Style::default().fg(Color::Green)),
                                Span::raw(": Save and Quit")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("^C", Style::default().fg(Color::Green)),
                                Span::raw(": Quit")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" V", Style::default().fg(Color::Green)),
                                Span::raw(": Switch info view")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" H", Style::default().fg(Color::Green)),
                                Span::raw(": Help")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("Ok", Style::default().fg(Color::Black).bg(Color::White)),
                            ]
                        )
                    ]
                );
            }
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> Result<(),Box<dyn std::error::Error>>
    {
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

            terminal.hide_cursor()?;

            terminal.draw(|f| {
                self.screen_size = (f.size().width, f.size().height);
                let output_rect = Rect::new(0, f.size().height - 1, f.size().width, 1);
                let address_rect = Rect::new(0, 0, 17, f.size().height - output_rect.height);
                let hex_editor_rect = Rect::new(address_rect.width, 0, (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as u16, f.size().height - output_rect.height);
                let mut info_view_rect = Rect::new(address_rect.width + hex_editor_rect.width, 0, (self.block_size * 2 * self.blocks_per_row + self.blocks_per_row) as u16 - 1, f.size().height - output_rect.height);

                let output_block = ratatui::widgets::Paragraph::new(Text::raw(&self.output))
                    .block(Block::default().borders(Borders::LEFT));
                let address_block = Paragraph::new(&self.address_view)
                    .block(Block::default().title("Address").borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM))
                    .scroll((self.scroll, 0));
                
                let editor_title = format!("Hex Editor{}", if self.dirty { " *"} else {""});

                let hex_editor_block = Paragraph::new(&self.hex_view)
                    .block(Block::default().title(editor_title).borders(Borders::LEFT | Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
                    .scroll((self.scroll, 0));
                
                let info_view_block = 
                match &self.info_mode 
                {
                    InfoMode::Text =>
                    {
                        Paragraph::new(&self.text_view)
                            .block(Block::default().title("Text View").borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
                            .scroll((self.scroll, 0))
                    },
                    InfoMode::Assembly =>
                    {
                        info_view_rect.width = f.size().width - address_rect.width - hex_editor_rect.width - 2;
                        Paragraph::new(&self.assembly_view)
                            .block(Block::default().title("Assembly View").borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
                            .scroll((self.get_assembly_view_scroll() as u16, 0))
                    }
                };

                let scrollbar = ratatui::widgets::Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                    .track_symbol(Some("█"))
                    .track_style(Style::default().fg(Color::DarkGray))
                    .begin_symbol(None)
                    .end_symbol(None);
                let mut scrollbar_state = ScrollbarState::new(self.hex_view.lines.len()).position(self.scroll as usize + self.cursor.1 as usize);

                f.render_widget(output_block, output_rect);
                f.render_widget(address_block, address_rect);
                f.render_widget(hex_editor_block, hex_editor_rect);
                f.render_widget(info_view_block, info_view_rect);
                f.render_stateful_widget(scrollbar, f.size(), &mut scrollbar_state);

                if let Some(popup_state) = &self.popup 
                {
                    let clear = ratatui::widgets::Clear::default();

                    let mut popup_text = Text::default();
                    let mut popup_title = "Popup";

                    let mut popup_rect = Rect::new(f.size().width / 2 - 27, f.size().height / 2 - 2, 54, 5);

                    Self::fill_popup(popup_state, f, &mut popup_title, &mut popup_text, &mut popup_rect);

                    let popup = ratatui::widgets::Paragraph::new(popup_text)
                        .block(Block::default().title(popup_title).borders(Borders::ALL))
                        .alignment(ratatui::layout::Alignment::Center);
                    f.render_widget(clear, popup_rect);
                    f.render_widget(popup, popup_rect);
                }

            })?;
            
            if self.popup.is_none()
            {
                terminal.set_cursor(self.cursor.0 + 18, self.cursor.1 + 1)?;
                terminal.show_cursor()?;   
            }
        }

        Ok(())
    }
}

