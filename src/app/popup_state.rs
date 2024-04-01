use std::error::Error;

use ratatui::{layout::Rect, text::{Line, Span, Text}, Frame};

use super::{assembly::AssemblyLine, color_settings::ColorSettings, App};

#[derive(Clone, Debug)]
pub enum PopupState
{
    Run
    {
        command: String,
        cursor: usize
    },
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
        preview: Result<Vec<u8>,String>,
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
            Some(PopupState::Patch{..}) => screen_height - 6 - 2,
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

    pub(super) fn get_patch_preview(&self, color_settings: &ColorSettings, preview: &Result<Vec<u8>,String>) -> Line<'a>
    {
        let mut preview_string = Line::raw(" ");
        match preview
        {
            Ok(preview) =>
            {
                let old_instruction = self.get_current_instruction();
                if let AssemblyLine::Instruction(instruction) = old_instruction
                {
                    let old_bytes_offset = instruction.file_address as usize;
                    let old_bytes_len = instruction.instruction.len();
                    let patch_len = preview.len();
                    let max_instruction_length = std::cmp::min(16, self.data.len() - old_bytes_offset);
                    let old_bytes_with_max_possible_length = &self.data[old_bytes_offset..old_bytes_offset + max_instruction_length];
                    for (i, byte) in old_bytes_with_max_possible_length.iter().enumerate()
                    {
                        if i < patch_len
                        {
                            
                            let style = if i >= old_bytes_len 
                            {
                                color_settings.patch_patched_greater
                            }
                            else
                            {
                                color_settings.patch_patched_less_or_equal
                            };
                            preview_string.spans.push(Span::styled(format!("{:02X} ", preview[i]), style));
                        }
                        else if i < old_bytes_len
                        {
                            let style = color_settings.patch_old_instruction;
                            preview_string.spans.push(Span::styled(format!("{:02X} ", byte), style));
                        }
                        else
                        {
                            let style = color_settings.patch_old_rest;
                            preview_string.spans.push(Span::styled(format!("{:02X} ", byte), style));
                        };
                        
                    }

                }
                else 
                {
                    if preview.is_empty()
                    {
                        preview_string.spans.push(Span::styled("Preview", color_settings.placeholder));
                    }
                    else 
                    {
                        for byte in preview.iter()
                        {
                            let style = Self::get_style_for_byte(color_settings, *byte);
                            preview_string.spans.push(Span::styled(format!("{:02X} ", byte), style));
                        }       
                    }
                }
            }
            Err(e) =>
            {
                preview_string.spans.push(Span::styled(e.clone(), color_settings.log_error));
            }
        }
        preview_string
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
                spans.push(Span::styled(c.to_string(), color_settings.menu_text_selected));
            }
            else
            {
                spans.push(Span::raw(c.to_string()));
            }
        }
        
        spans.push(Span::styled(" ", if cursor == string.len() {
            color_settings.menu_text_selected
        } else {
            color_settings.menu_text
        }));
        Line::from(spans)
    }

    pub(super) fn get_multiline_from_string_and_cursor(color_settings: &ColorSettings, s: &str, cursor: usize, placeholder: &str) -> (Vec<Line<'a>>, usize)
    {
        let string = s.to_string();
        if string.len() == 0
        {
            return (vec![Line::from(vec![Span::styled("   1 ", color_settings.patch_line_number), Span::styled(placeholder.to_string(), color_settings.placeholder), Span::raw(" ")]).left_aligned()], 0);
        }
        let mut lines = Vec::new();
        let mut selected_line = 0;
        let mut current_line = vec![Span::styled("   1 ", color_settings.patch_line_number)];
        for (i, c) in string.chars().enumerate()
        {
            let style = if i == cursor
            {
                selected_line = lines.len();
                color_settings.menu_text_selected
            }
            else
            {
                color_settings.menu_text
            };
            if c == '\n'
            {
                current_line.push(Span::styled(" ", style));
                lines.push(Line::from(current_line).left_aligned());
                current_line = vec![Span::styled(format!(" {:3} ", lines.len() + 1), color_settings.patch_line_number)];
            }
            else
            {
                current_line.push(Span::styled(c.to_string(), style));
            }
        }
        if cursor == string.len()
        {
            selected_line = lines.len();
            current_line.push(Span::styled(" ", color_settings.menu_text_selected));
        }
        if current_line.len() > 0
        {
            lines.push(Line::from(current_line).left_aligned());
        }
        (lines, selected_line)
    }

    pub(super) fn fill_popup(&'a self, color_settings: &ColorSettings, popup_state: &PopupState, f: &Frame, popup_title: &mut &str, popup_text: &mut Text<'a>, popup_rect: &mut Rect) -> Result<(), Box<dyn Error>>
    {
        match &popup_state
        {
            PopupState::Run { command, cursor } =>
            {
                *popup_title = "Run";
                let width = 60;
                let height = 3;
                *popup_rect = Rect::new(f.size().width / 2 - width as u16/2, f.size().height / 2 - height as u16 / 2, width as u16, height as u16);
                let mut editable_string = Self::get_line_from_string_and_cursor(color_settings, command, *cursor, "Command");
                editable_string.spans.insert(0, Span::styled(" >", color_settings.menu_text));
                popup_text.lines.extend(
                    vec![editable_string.left_aligned()]
                );
            },
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
                            vec![Line::from(vec![Span::styled("▲", color_settings.menu_text)])]
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
                            symbols_as_lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
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
                        popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
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
                        popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
                    }
                    else
                    {
                        popup_text.lines.push(Line::raw(""));
                    }
                }
            }
            PopupState::Patch {assembly,preview,  cursor} =>
            {
                *popup_title = "Patch";
                let available_editable_text_lines = self.get_scrollable_popup_line_count()?;
                let height = 6 + available_editable_text_lines as u16;

                let width = 60;
                *popup_rect = Rect::new(f.size().width / 2 - width/2, f.size().height / 2 - height/2, width, height);
                let (editable_lines, selected_line) = Self::get_multiline_from_string_and_cursor(color_settings, assembly, *cursor, "Assembly");
                let preview_line = self.get_patch_preview(color_settings, preview);
                popup_text.lines.extend(
                    vec![
                        preview_line.left_aligned(),
                        Line::raw("─".repeat(width as usize)),
                    ]
                );
                let skip_lines = 0.max(selected_line as isize - (available_editable_text_lines as isize - 1) / 2) as usize;
                let skip_lines = skip_lines.min(editable_lines.len().saturating_sub(available_editable_text_lines as usize));
                if skip_lines == 0
                {
                    popup_text.lines.push(Line::raw(""));
                }
                else 
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
                }
                let editable_lines_count = editable_lines.len();
                popup_text.lines.extend(editable_lines.into_iter().skip(skip_lines).take(available_editable_text_lines as usize));
                if editable_lines_count as isize - skip_lines as isize > available_editable_text_lines as isize
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
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
                let width = 50;
                *popup_rect = Rect::new(f.size().width / 2 - width / 2, f.size().height / 2 - height as u16 / 2, width, height as u16);
                *popup_title = "Help";
                if *scroll > 0
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
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
                    popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
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