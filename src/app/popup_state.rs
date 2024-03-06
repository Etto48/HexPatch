use ratatui::{layout::Rect, text::{Line, Span, Text}, Frame};

use super::{color_settings::ColorSettings, App};

pub enum PopupState
{
    Patch(String),
    JumpToAddress(String),
    QuitDirtySave(bool),
    SaveAndQuit(bool),
    Save(bool),
    Help
}

impl <'a> App<'a>
{
    pub(super) fn fill_popup(color_settings: &ColorSettings, popup_state: &PopupState, f: &Frame, popup_title: &mut &str, popup_text: &mut Text, popup_rect: &mut Rect)
    {
        match &popup_state
        {
            PopupState::Patch(assembly) =>
            {
                *popup_title = "Patch";
                popup_text.lines.extend(
                    vec![
                        Line::raw("Enter the assembly to patch:"),
                        Line::raw(assembly.clone()),
                        Line::from(vec![Span::styled("Ok", color_settings.ok_selected)])
                    ]
                );
            }
            PopupState::JumpToAddress(address) =>
            {
                *popup_title = "Jump";
                popup_text.lines.extend(
                    vec![
                        Line::raw("Enter the address to jump to:"),
                        Line::raw(address.clone()),
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
                *popup_rect = Rect::new(f.size().width / 2 - 15, f.size().height / 2 - 5, 30, 10);
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