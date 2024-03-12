use ratatui::{layout::Rect, text::{Line, Span, Text}, Frame};

use super::{color_settings::ColorSettings, App};

#[derive(Clone, Debug)]
pub enum PopupState
{
    Log(usize),
    Patch
    {
        assembly: String,
        cursor: usize
    },
    JumpToAddress
    {
        address: String,
        cursor: usize
    },
    QuitDirtySave(bool),
    SaveAndQuit(bool),
    Save(bool),
    Help
}

impl <'a> App<'a>
{

    fn get_line_from_string_and_cursor(color_settings: &ColorSettings, s: &str, cursor: usize) -> Line<'a>
    {
        let string = s.to_string();
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
                *popup_rect = Rect::new(f.size().width / 2 - 16, f.size().height / 2 - 3, 32, 7);
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, assembly, *cursor);
                popup_text.lines.extend(
                    vec![
                        Line::raw("Enter the assembly to patch:"),
                        Line::raw("──────────────────────────────"),
                        editable_string.left_aligned(),
                        Line::raw("──────────────────────────────"),
                        Line::from(vec![Span::styled("Ok", color_settings.ok_selected)])
                    ]
                );
            }
            PopupState::JumpToAddress {address, cursor} =>
            {
                *popup_title = "Jump";
                *popup_rect = Rect::new(f.size().width / 2 - 16, f.size().height / 2 - 3, 32, 7);
                let editable_string = Self::get_line_from_string_and_cursor(color_settings, address, *cursor);
                popup_text.lines.extend(
                    vec![
                        Line::raw("Enter the address to jump to:"),
                        Line::raw("──────────────────────────────"),
                        editable_string.right_aligned(),
                        Line::raw("──────────────────────────────"),
                        Line::from(vec![Span::styled("Ok", color_settings.ok_selected)])
                    ]
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
                                Span::raw(": Jump to address")
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
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("Ok", color_settings.ok_selected),
                            ]
                        )
                    ]
                );
            }
        }
    }
}