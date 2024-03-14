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
    Help
}

impl <'a> App<'a>
{

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

    pub(super) fn fill_popup(&'a self, color_settings: &ColorSettings, popup_state: &PopupState, f: &Frame, popup_title: &mut &str, popup_text: &mut Text<'a>, popup_rect: &mut Rect)
    {
        match &popup_state
        {
            PopupState::FindSymbol{ filter, symbols, cursor, scroll } =>
            {
                *popup_title = "Find Symbol";
                let width = 60;
                let max_symbols = 8;
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
                

                *popup_rect = Rect::new(f.size().width / 2 - width as u16/2, f.size().height / 2 - 7, width as u16, 14);
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, filter, *cursor, "Filter");
                if self.header.get_symbols().is_some()
                {
                    let symbols_as_lines = if symbols.len() > 0
                    {
                        let symbol_line_iter = symbols.iter().skip(scroll).take(8).enumerate().map(|(i, (address, name))| {
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
                        });
                        let mut symbols_as_lines = if scroll > 0
                        {
                            vec![Line::from(vec![Span::styled("▲", color_settings.ok)])]
                        }
                        else
                        {
                            vec![Line::raw("")]
                        };

                        symbols_as_lines.extend(symbol_line_iter);
                        if symbols_as_lines.len() < max_symbols
                        {
                            symbols_as_lines.extend(vec![Line::raw(""); max_symbols - symbols_as_lines.len()]);
                        }

                        if symbols.len() as isize - scroll as isize > max_symbols as isize
                        {
                            symbols_as_lines.push(Line::from(vec![Span::styled("▼", color_settings.ok)]));
                        }
                        else
                        {
                            symbols_as_lines.push(Line::raw(""));
                        }
                        symbols_as_lines
                    }
                    else if filter.len() > 0
                    {
                        let mut lines = vec![Line::raw("No symbols found.").left_aligned()];
                        lines.extend(vec![Line::raw(""); 7]);
                        lines
                    }
                    else
                    {
                        let mut lines = vec![Line::raw("Type to search.").left_aligned()];
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
                *popup_rect = Rect::new(f.size().width / 2 - 30, f.size().height / 2 - 6, 60, 12);
                if self.log.len() > 0
                {
                    if self.log.len() as isize - *scroll as isize > 8
                    {
                        popup_text.lines.push(Line::from(vec![Span::styled("▲", color_settings.ok)]));
                    }
                    else
                    {
                        popup_text.lines.push(Line::raw(""));
                    }
                    // take the last 8 lines skipping "scroll" lines from the bottom
                    for line in self.log.iter().rev().skip(*scroll).take(8).rev()
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
            PopupState::Help =>
            {
                *popup_rect = Rect::new(f.size().width / 2 - 15, f.size().height / 2 - 5, 30, 11);
                *popup_title = "Help";
                popup_text.lines.extend(
                    vec![
                        Line::from(
                            vec![
                                Span::styled("^S", color_settings.help_command),
                                Span::raw(": Save")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("^X", color_settings.help_command),
                                Span::raw(": Save and Quit")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("^C", color_settings.help_command),
                                Span::raw(": Quit")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" V", color_settings.help_command),
                                Span::raw(": Switch info view")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" J", color_settings.help_command),
                                Span::raw(": Jump to location")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" S", color_settings.help_command),
                                Span::raw(": Symbol search")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" P", color_settings.help_command),
                                Span::raw(": Patch assembly")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" L", color_settings.help_command),
                                Span::raw(": Log")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" H", color_settings.help_command),
                                Span::raw(": Help")
                            ]
                        ).left_aligned()
                    ]
                );
            }
        }
    }
}