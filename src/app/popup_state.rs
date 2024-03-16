use std::error::Error;

use ratatui::{layout::Rect, text::{Line, Span, Text}, Frame};

use super::{color_settings::ColorSettings, App};

#[derive(Clone, Debug)]
pub enum PopupState
{
    FindSymbol
    {
        filter: String,
        cursor: usize,
        symbols: Vec<(u64, String)>,
        scroll: usize
    },
    Log(usize),
    Patch
    {
        assembly: String,
        cursor: usize
    },
    JumpToAddress
    {
        location: String,
        cursor: usize
    },
    QuitDirtySave(bool),
    SaveAndQuit(bool),
    Save(bool),
    Help(usize)
}

impl <'a> App<'a>
{

    pub(super) fn get_scrollable_popup_line_count(&self) -> Result<usize, String>
    {
        let screen_height = self.screen_size.1 as isize;
        let lines = match &self.popup
        {
            Some(PopupState::FindSymbol{ .. }) => screen_height - 6 - 2,
            Some(PopupState::Log(_)) => screen_height - 4 - 2,
            Some(PopupState::Help(_)) => screen_height - 4 - 2,
            _ => 0
        };

        if lines <= 0
        {
            return Err("Screen too small".to_string());
        }
        else 
        {
            return Ok(lines as usize);
        }
    }

    pub(super) fn resize_popup_if_needed(popup: &mut Option<PopupState>)
    {
        match popup
        {
            Some(PopupState::FindSymbol { scroll, .. }) |
            Some(PopupState::Log(scroll)) |
            Some(PopupState::Help(scroll)) =>
            {
                *scroll = 0;
            }
            _ => {}
        }
    }

    fn get_line_from_string_and_cursor(color_settings: &ColorSettings, s: &str, cursor: usize, placeholder: &str) -> Line<'a>
    {
        let string = s.to_string();
        if string.len() == 0
        {
            return Line::from(vec![Span::raw(" "), Span::styled(placeholder.to_string(), color_settings.placeholder), Span::raw(" ")]);
        }
        let mut spans = vec![Span::raw(" ")];
        for (i, c) in string.chars().enumerate()
        {
            if i == cursor
            {
                spans.push(Span::styled(c.to_string(), color_settings.ok_selected));
            }
            else
            {
                spans.push(Span::raw(c.to_string()));
            }
        }
        
        spans.push(Span::styled(" ", if cursor == string.len() {
            color_settings.ok_selected
        } else {
            color_settings.ok
        }));
        Line::from(spans)
    }

    pub(super) fn fill_popup(&'a self, color_settings: &ColorSettings, popup_state: &PopupState, f: &Frame, popup_title: &mut &str, popup_text: &mut Text<'a>, popup_rect: &mut Rect) -> Result<(), Box<dyn Error>>
    {
        match &popup_state
        {
            PopupState::FindSymbol{ filter, symbols, cursor, scroll } =>
            {
                *popup_title = "Find Symbol";
                let width = 60;
                let max_symbols = self.get_scrollable_popup_line_count()?;
                let height = max_symbols + 2 + 4;
                let mut selection = *scroll;
                let scroll = if *scroll > symbols.len() - max_symbols/2
                {
                    symbols.len().saturating_sub(max_symbols)
                }
                else if *scroll < max_symbols/2
                {
                    0
                }
                else
                {
                    *scroll - max_symbols/2
                };
                selection = selection.saturating_sub(scroll);


                *popup_rect = Rect::new(f.size().width / 2 - width as u16/2, f.size().height / 2 - height as u16 / 2, width as u16, height as u16);
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, filter, *cursor, "Filter");
                if self.header.get_symbols().is_some()
                {
                    let symbols_as_lines = if symbols.len() > 0 || filter.len() == 0
                    {
                        let additional_vector = if filter.len() == 0
                        {
                            if let Some(symbol_table) = self.header.get_symbols()
                            {
                                symbol_table.iter().skip(scroll).take(max_symbols + 1).map(|(k,v)|(*k, v.clone())).collect()
                            }
                            else 
                            {
                                Vec::new()
                            }
                        }
                        else
                        {
                            Vec::new()
                        };

                        let symbol_to_line_lambda = |(i, (address, name)): (usize, &(u64, String))| {
                            let short_name = name.chars().take(width-19).collect::<String>();
                            let space_count = (width - short_name.len() - 19 + 1).clamp(0, width);
                            let (style_sym, sytle_empty, style_addr) = if i == selection
                            {
                                (color_settings.assembly_selected, color_settings.assembly_selected, color_settings.assembly_selected)
                            }
                            else
                            {
                                (color_settings.assembly_symbol, color_settings.assembly_symbol, color_settings.assembly_address)
                            };
                            Line::from(vec![
                            Span::styled(short_name, style_sym), 
                            Span::styled(" ".repeat(space_count), sytle_empty), 
                            Span::styled(format!("{:16X}", address), style_addr)]).left_aligned()
                        };
                        let symbol_line_iter = symbols.iter().skip(scroll).take(max_symbols).enumerate().map(symbol_to_line_lambda);
                        let mut symbols_as_lines = if scroll > 0
                        {
                            vec![Line::from(vec![Span::styled("▲", color_settings.ok)])]
                        }
                        else
                        {
                            vec![Line::raw("")]
                        };

                        symbols_as_lines.extend(symbol_line_iter);
                        symbols_as_lines.extend(additional_vector.iter().take(max_symbols).enumerate().map(symbol_to_line_lambda));
                        if symbols_as_lines.len() < max_symbols
                        {
                            symbols_as_lines.extend(vec![Line::raw(""); max_symbols - symbols_as_lines.len()]);
                        }

                        if symbols.len() as isize - scroll as isize > max_symbols as isize || additional_vector.len() > max_symbols
                        {
                            symbols_as_lines.push(Line::from(vec![Span::styled("▼", color_settings.ok)]));
                        }
                        else
                        {
                            symbols_as_lines.push(Line::raw(""));
                        }
                        
                        symbols_as_lines
                    }
                    else
                    {
                        let mut lines = vec![Line::raw("No symbols found.").left_aligned()];
                        lines.extend(vec![Line::raw(""); 7]);
                        lines
                    };
                    popup_text.lines.extend(
                        vec![
                            editable_string.left_aligned(),
                            Line::raw("─".repeat(width)),
                        ]
                    );
                    popup_text.lines.extend(symbols_as_lines);
                }
                else 
                {
                    popup_text.lines.extend(
                        vec![
                            Line::raw("No symbol table found.").left_aligned(),
                        ]
                    );    
                }
                
            }
            PopupState::Log(scroll) =>
            {
                *popup_title = "Log";
                let max_lines = self.get_scrollable_popup_line_count()?;
                let height = max_lines + 4;
                *popup_rect = Rect::new(f.size().width / 2 - 30, f.size().height / 2 - height as u16 / 2, 60, height as u16);
                if self.log.len() > 0
                {
                    if self.log.len() as isize - *scroll as isize > max_lines as isize
                    {
                        popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.ok)]));
                    }
                    else
                    {
                        popup_text.lines.push(Line::raw(""));
                    }
                    // take the last 8 lines skipping "scroll" lines from the bottom
                    for line in self.log.iter().rev().skip(*scroll).take(max_lines).rev()
                    {
                        popup_text.lines.push(line.to_line(color_settings));
                    }
                    if *scroll > 0
                    {
                        popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.ok)]));
                    }
                    else
                    {
                        popup_text.lines.push(Line::raw(""));
                    }
                }
            }
            PopupState::Patch {assembly, cursor} =>
            {
                *popup_title = "Patch";
                *popup_rect = Rect::new(f.size().width / 2 - 30, f.size().height / 2 - 3, 60, 3);
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, assembly, *cursor, "Assembly");
                popup_text.lines.extend(
                    vec![editable_string.left_aligned()]
                );
            }
            PopupState::JumpToAddress {location: address, cursor} =>
            {
                *popup_title = "Jump";
                *popup_rect = Rect::new(f.size().width / 2 - 30, f.size().height / 2 - 3, 60, 3);
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, address, *cursor, "Location");
                popup_text.lines.extend(
                    vec![editable_string.left_aligned()]
                );
            }
            PopupState::SaveAndQuit(yes_selected) =>
            {
                *popup_title = "Save and Quit";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file will be saved and the program will quit."),
                        Line::raw("Are you sure?"),
                        Line::from(vec![
                            Span::styled("Yes", color_settings.yes),
                            Span::raw("  "),
                            Span::styled("No", color_settings.no)
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = color_settings.yes_selected;
                }
                else
                {
                    popup_text.lines[2].spans[2].style = color_settings.no_selected;
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
                            Span::styled("Yes", color_settings.yes),
                            Span::raw("  "),
                            Span::styled("No", color_settings.no)
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = color_settings.yes_selected;
                }
                else
                {
                    popup_text.lines[2].spans[2].style = color_settings.no_selected;
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
                            Span::styled("Yes", color_settings.yes),
                            Span::raw("  "),
                            Span::styled("No", color_settings.no)
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = color_settings.yes_selected;
                }
                else
                {
                    popup_text.lines[2].spans[2].style = color_settings.no_selected;
                }
            },
            PopupState::Help(scroll) =>
            {
                let max_lines = self.get_scrollable_popup_line_count()?;
                let height = max_lines + 4;
                *popup_rect = Rect::new(f.size().width / 2 - 15, f.size().height / 2 - height as u16 / 2, 30, height as u16);
                *popup_title = "Help";
                if *scroll > 0
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.ok)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
                popup_text.lines.extend(
                    self.help_list
                        .iter()
                        .skip(*scroll)
                        .take(max_lines)
                        .map(|h| h.to_line(&self.color_settings)
                    )
                );
                if self.help_list.len() as isize - *scroll as isize > max_lines as isize
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.ok)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
            }
        }
        Ok(())
    }
}