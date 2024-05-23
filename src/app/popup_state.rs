use std::error::Error;

use ratatui::text::{Line, Span, Text};

use super::{assembly::AssemblyLine, files::{path_result::PathResult, path}, run_command::Command, settings::color_settings::ColorSettings, App};

#[derive(Clone, Debug)]
pub enum PopupState
{
    Open
    {
        currently_open_path: String,
        path: String,
        cursor: usize,
        results: Vec<PathResult>,
        scroll: usize
    },
    Run
    {
        command: String,
        cursor: usize,
        results: Vec<Command>,
        scroll: usize
    },
    FindText
    {
        text: String,
        cursor: usize,
    },
    FindSymbol
    {
        filter: String,
        cursor: usize,
        symbols: Vec<(u64, String)>,
        scroll: usize
    },
    Log(usize),
    InsertText
    {
        text: String,
        cursor: usize
    },
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
    SaveAs{
        path: String,
        cursor: usize,
    },
    Save(bool),
    Help(usize)
}

impl App
{

    pub(super) fn get_scrollable_popup_line_count(&self) -> usize
    {
        let screen_height = self.screen_size.1 as isize;
        let lines = match &self.popup
        {
            Some(PopupState::Open{..}) => screen_height - 7 - 2,
            Some(PopupState::Run{..}) => screen_height - 6 - 2,
            Some(PopupState::FindSymbol{ .. }) => screen_height - 6 - 2,
            Some(PopupState::Log(_)) => screen_height - 4 - 2,
            Some(PopupState::Help(_)) => screen_height - 4 - 2,
            Some(PopupState::Patch{..}) => screen_height - 6 - 2,
            Some(PopupState::InsertText{..}) => screen_height - 5 - 2,
            _ => unimplemented!("Popup is not supposed to have scrollable lines")
        };

        if lines <= 0
        {
            1
        }
        else 
        {
            lines as usize
        }
    }

    pub(super) fn get_patch_preview(&self, color_settings: &ColorSettings, preview: &Result<Vec<u8>,String>) -> Line<'static>
    {
        let mut preview_string = Line::raw(" ");
        match preview
        {
            Ok(preview) =>
            {
                let old_instruction = self.get_current_instruction();
                if let Some(old_instruction) = old_instruction
                {
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
                    else if preview.is_empty() 
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

    fn get_line_from_string_and_cursor(color_settings: &ColorSettings, s: &str, cursor: usize, placeholder: &str, available_width: usize, show_cursor: bool) -> Line<'static>
    {
        let string = s.to_string();
        if string.is_empty()
        {
            return Line::from(vec![Span::raw(" "), Span::styled(placeholder.to_string(), color_settings.placeholder), Span::raw(" ")]);
        }
        let mut spans = vec![];
        
        

        let available_width = available_width.saturating_sub(2);

        let skip = 0.max(cursor as isize - (available_width as isize - 1) / 2) as usize;
        let skip = skip.min(string.len().saturating_sub(available_width));

        if skip > 0
        {
            spans.push(Span::styled("<", color_settings.menu_text_selected));
        }
        else
        {
            spans.push(Span::raw(" "));
        }

        for (i, c) in string.chars().enumerate().skip(skip).take(available_width)
        {
            if i == cursor && show_cursor
            {
                spans.push(Span::styled(c.to_string(), color_settings.menu_text_selected));
            }
            else
            {
                spans.push(Span::raw(c.to_string()));
            }
        }

        if s.len() as isize - skip as isize > available_width as isize
        {
            spans.push(Span::styled(">", color_settings.menu_text_selected));
        }
        else
        {
            spans.push(Span::styled(" ", if cursor == string.len() {
                color_settings.menu_text_selected
            } else {
                color_settings.menu_text
            }));
        }

        Line::from(spans)
    }

    fn get_line_number_string(line_number: usize, char_for_line_count: usize) -> String
    {
        format!("{:width$}", line_number, width = char_for_line_count)
    }

    fn get_multiline_from_string_and_cursor(color_settings: &ColorSettings, s: &str, cursor: usize, placeholder: &str, available_width: usize) -> (Vec<Line<'static>>, usize)
    {
        let string = s.to_string();
        let line_count = &string.chars().filter(|c| *c == '\n').count() + 1;
        let char_for_line_count = line_count.to_string().len();
        if string.is_empty()
        {
            return (vec![Line::from(vec![Span::styled(Self::get_line_number_string(1, char_for_line_count), color_settings.patch_line_number), Span::raw(" "), Span::styled(placeholder.to_string(), color_settings.placeholder)]).left_aligned()], 0);
        }
        let mut lines = Vec::new();
        let mut selected_line = 0;
        let mut current_line = String::new();
        let mut start_of_line_index = 0;
        for (i, c) in string.chars().enumerate()
        {
            if i == cursor
            {
                selected_line = lines.len();
            }
            if c == '\n'
            {
                let line_number = Span::styled(Self::get_line_number_string(lines.len() + 1, char_for_line_count), color_settings.patch_line_number);
                let mut line_cursor = cursor as isize - start_of_line_index as isize;
                let mut show_cursor = true;
                if line_cursor > current_line.len() as isize || line_cursor < 0
                {
                    show_cursor = false;
                    line_cursor = 0;
                }
                else {
                    current_line.push(' ');
                }
                start_of_line_index = i + 1;
                let used_width = line_number.content.len();
                let mut line = Self::get_line_from_string_and_cursor(color_settings, &current_line, line_cursor as usize, "", available_width - used_width, show_cursor);
                line.spans.insert(0, line_number);
                lines.push(line.left_aligned());
                current_line.clear();
            }
            else
            {
                current_line.push(c);
            }
        }
        if cursor == string.len()
        {
            if current_line.is_empty()
            {
                current_line.push(' ');
            }
            selected_line = lines.len();
        }
        let line_number = Span::styled(Self::get_line_number_string(lines.len() + 1, char_for_line_count), color_settings.patch_line_number);
        let mut line_cursor = cursor as isize - start_of_line_index as isize;
        let mut show_cursor = true;
        if line_cursor > current_line.len() as isize || line_cursor < 0
        {
            show_cursor = false;
            line_cursor = 0;
        }
        let used_width = line_number.content.len();
        let mut line = Self::get_line_from_string_and_cursor(color_settings, &current_line, line_cursor as usize, "", available_width - used_width, show_cursor);
        line.spans.insert(0, line_number);
        lines.push(line.left_aligned());
        (lines, selected_line)
    }

    pub(super) fn fill_popup(&self, color_settings: &ColorSettings, popup_state: &PopupState, popup_title: &mut &str, popup_text: &mut Text<'static>, height: &mut usize, width: &mut usize) -> Result<(), Box<dyn Error>>
    {
        match &popup_state
        {
            PopupState::Open { currently_open_path, path, cursor, results, scroll } =>
            {
                *popup_title = "Open";
                let available_width = width.saturating_sub(2);
                let max_results = self.get_scrollable_popup_line_count();
                *height = max_results + 2 + 5;
                
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, path, *cursor, "Path", available_width, true);

                let (prefix, currently_open_path_text) = if let Some(parent) = path::parent(currently_open_path)
                {
                    (
                        ".../",
                        path::diff(currently_open_path, parent)
                    )
                }
                else
                {
                    (
                        "",
                        currently_open_path.as_str()
                    )
                };
                

                popup_text.lines.extend(
                    vec![
                        Line::styled(
                            format!(" {}{}", prefix, currently_open_path_text), 
                            color_settings.path_dir
                        ).left_aligned(),
                        editable_string.left_aligned(), 
                        Line::raw("─".repeat(*width))
                    ]
                );
                let skip = 0.max(*scroll as isize - max_results as isize / 2) as usize;
                let skip = skip.min(results.len().saturating_sub(max_results));
                let relative_scroll = *scroll - skip;
                let results_iter = results
                    .iter()
                    .skip(skip)
                    .take(max_results)
                    .enumerate()
                    .map(
                        |(i,p)| 
                        p.to_line(color_settings, relative_scroll == i, currently_open_path)
                    );
                if skip > 0
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
                popup_text.lines.extend(results_iter);
                if results.len() as isize - skip as isize > max_results as isize
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
            },
            PopupState::Run { command, cursor, results, scroll } =>
            {
                *popup_title = "Run";
                let available_width = width.saturating_sub(2);
                let max_results = self.get_scrollable_popup_line_count();
                *height = max_results + 2 + 4;
                let mut editable_string = Self::get_line_from_string_and_cursor(color_settings, command, *cursor, "Command", available_width, true);
                editable_string.spans.insert(0, Span::styled(" >", color_settings.menu_text));
                popup_text.lines.extend(
                    vec![
                        editable_string.left_aligned(),
                        Line::raw("─".repeat(*width)),
                    ]
                );
                let skip = 0.max(*scroll as isize - max_results as isize / 2) as usize;
                let skip = skip.min(results.len().saturating_sub(max_results));
                let relative_scroll = *scroll - skip;
                let results_iter = results.iter().skip(skip).take(max_results).enumerate().map(|(i,c)| c.to_line(color_settings, relative_scroll == i));
                if skip > 0
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
                popup_text.lines.extend(results_iter);
                if results.len() as isize - skip as isize > max_results as isize
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }

            },
            PopupState::FindText { text, cursor } =>
            {
                *popup_title = "Find Text";
                let available_width = width.saturating_sub(2);
                *height = 3;
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, text, *cursor, "Text", available_width, true);
                popup_text.lines.extend(
                    vec![editable_string.left_aligned()]
                );
            }
            PopupState::FindSymbol{ filter, symbols, cursor, scroll } =>
            {
                *popup_title = "Find Symbol";
                let available_width = width.saturating_sub(2);
                let max_symbols = self.get_scrollable_popup_line_count();
                *height = max_symbols + 2 + 4;
                let mut selection = *scroll;
                let symbols_len = if !symbols.is_empty()
                {
                    symbols.len()
                }
                else if let Some(symbol_table) = self.header.get_symbols()
                {
                    
                    symbol_table.len()
                }
                else 
                {
                    0
                };
                let scroll = if *scroll as isize > symbols_len as isize - (max_symbols as isize)/2
                {
                    symbols_len.saturating_sub(max_symbols)
                }
                else if *scroll < max_symbols/2
                {
                    0
                }
                else
                {
                    scroll.saturating_sub(max_symbols/2)
                };
                selection = selection.saturating_sub(scroll);


                let editable_string = Self::get_line_from_string_and_cursor(color_settings, filter, *cursor, "Filter", available_width, true);
                if self.header.get_symbols().is_some()
                {
                    let symbols_as_lines = if !symbols.is_empty() || filter.is_empty()
                    {
                        let additional_vector = if filter.is_empty()
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
                            let short_name = name.chars().take(width.saturating_sub(19)).collect::<String>();
                            let space_count = (width.saturating_sub(short_name.len() + 19) + 1).clamp(0, *width);
                            let (style_sym, style_empty, style_addr) = if i == selection
                            {
                                (color_settings.assembly_selected, color_settings.assembly_selected, color_settings.assembly_selected)
                            }
                            else
                            {
                                (color_settings.assembly_symbol, color_settings.assembly_symbol, color_settings.assembly_address)
                            };
                            Line::from(vec![
                            Span::styled(short_name, style_sym), 
                            Span::styled(" ".repeat(space_count), style_empty), 
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
                            Line::raw("─".repeat(*width)),
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
                let max_lines = self.get_scrollable_popup_line_count();
                *height = max_lines + 4;
                if !self.log.is_empty()
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
            PopupState::InsertText {text, cursor} =>
            {
                *popup_title = "Text";
                let available_editable_text_lines = self.get_scrollable_popup_line_count();
                *height = 3 + 2 + available_editable_text_lines;
                let available_width = width.saturating_sub(2);
                let (editable_lines, selected_line) = Self::get_multiline_from_string_and_cursor(color_settings, text, *cursor, "Text", available_width);
                let skip_lines = 0.max(selected_line as isize - (available_editable_text_lines as isize - 1) / 2) as usize;
                let skip_lines = skip_lines.min(editable_lines.len().saturating_sub(available_editable_text_lines));
                if skip_lines == 0
                {
                    popup_text.lines.push(Line::raw(""));
                }
                else 
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
                }
                let editable_lines_count = editable_lines.len();
                popup_text.lines.extend(editable_lines.into_iter().skip(skip_lines).take(available_editable_text_lines));
                for _ in 0..(available_editable_text_lines as isize - editable_lines_count as isize)
                {
                    popup_text.lines.push(Line::raw(""));
                }
                if editable_lines_count as isize - skip_lines as isize > available_editable_text_lines as isize
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▼", color_settings.menu_text)]));
                }
                else
                {
                    popup_text.lines.push(Line::raw(""));
                }
                let status = format!("{}B",text.as_bytes().len());
                let padding = width.saturating_sub(status.len());
                popup_text.lines.push(Line::styled(format!("{}{}",status, " ".repeat(padding)), color_settings.insert_text_status).left_aligned())
            }
            PopupState::Patch {assembly,preview,  cursor} =>
            {
                *popup_title = "Patch";
                let available_editable_text_lines = self.get_scrollable_popup_line_count();
                *height = 6 + available_editable_text_lines;
                let available_width = width.saturating_sub(2);
                let (editable_lines, selected_line) = Self::get_multiline_from_string_and_cursor(color_settings, assembly, *cursor, "Assembly", available_width);
                let preview_line = self.get_patch_preview(color_settings, preview);
                popup_text.lines.extend(
                    vec![
                        preview_line.left_aligned(),
                        Line::raw("─".repeat(*width)),
                    ]
                );
                let skip_lines = 0.max(selected_line as isize - (available_editable_text_lines as isize - 1) / 2) as usize;
                let skip_lines = skip_lines.min(editable_lines.len().saturating_sub(available_editable_text_lines));
                if skip_lines == 0
                {
                    popup_text.lines.push(Line::raw(""));
                }
                else 
                {
                    popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.menu_text)]));
                }
                let editable_lines_count = editable_lines.len();
                popup_text.lines.extend(editable_lines.into_iter().skip(skip_lines).take(available_editable_text_lines));
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
                let available_width = width.saturating_sub(2);
                *height = 3;
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, address, *cursor, "Location", available_width, true);
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
            PopupState::SaveAs { path, cursor } =>
            {
                *popup_title = "Save As";
                let available_width = width.saturating_sub(2);
                *height = 3;
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, path, *cursor, "Path", available_width, true);
                popup_text.lines.extend(
                    vec![editable_string.left_aligned()]
                );
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
                let max_lines = self.get_scrollable_popup_line_count();
                *height = max_lines + 4;
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
                        .map(|h| h.to_line(&self.settings.color)
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