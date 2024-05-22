use std::error::Error;

use ratatui::text::{Line, Span};

use super::{notification::NotificationLevel, popup_state::PopupState, settings::color_settings::ColorSettings, App};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command
{
    Quit,
    QuitWithoutSave,
    QuitWithSave,
    SaveAs,
    Save,
    Help,
    Open,
    Log,
    Run,
    FindText,
    FindSymbol,
    Text,
    Patch,
    JumpToAddress,
    SwitchView,

    Empty,
    Unknown,
}

impl Command
{
    pub fn get_commands() -> Vec<&'static str>
    {
        vec![
            "quit",
            "dquit",
            "xquit",
            "save",
            "saveas",
            "help",
            "open",
            "log",
            "run",
            "ftext",
            "fsym",
            "text",
            "patch",
            "jump",
            "view",
        ]
    }
    pub fn from_string(command: &str) -> Command
    {
        match command
        {
            "quit" => Command::Quit,
            "dquit" => Command::QuitWithoutSave,
            "xquit" => Command::QuitWithSave,
            "save" => Command::Save,
            "saveas" => Command::SaveAs,
            "help" => Command::Help,
            "open" => Command::Open,
            "log" => Command::Log,
            "run" => Command::Run,
            "ftext" => Command::FindText,
            "fsym" => Command::FindSymbol,
            "text" => Command::Text,
            "patch" => Command::Patch,
            "jump" => Command::JumpToAddress,
            "view" => Command::SwitchView,
            "" => Command::Empty,
            _ => Command::Unknown,
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings, selected: bool) -> Line<'static>
    {
        let (s0, s1) = if selected {
            (color_settings.command_selected, color_settings.command_selected)
        } else {
            (color_settings.command_name, color_settings.command_description)
        };
        match self
        {
            Command::Quit => Line::from(vec![Span::styled("quit", s0), Span::styled(" Quit the program.", s1)]),
            Command::QuitWithoutSave => Line::from(vec![Span::styled("dquit", s0), Span::styled(" Quit the program without saving.", s1)]),
            Command::QuitWithSave => Line::from(vec![Span::styled("xquit", s0), Span::styled(" Save and quit the program.", s1)]),
            Command::SaveAs => Line::from(vec![Span::styled("saveas", s0), Span::styled(" Save the current file as a new file.", s1)]),
            Command::Save => Line::from(vec![Span::styled("save", s0), Span::styled(" Save the current file.", s1)]),
            Command::Help => Line::from(vec![Span::styled("help", s0), Span::styled(" Display the help page.", s1)]),
            Command::Open => Line::from(vec![Span::styled("open", s0), Span::styled(" Open a file.", s1)]),
            Command::Log => Line::from(vec![Span::styled("log", s0), Span::styled(" Open the log.", s1)]),
            Command::Run => Line::from(vec![Span::styled("run", s0), Span::styled(" Run a command.", s1)]),
            Command::FindText => Line::from(vec![Span::styled("ftext", s0), Span::styled(" Find text.", s1)]),
            Command::FindSymbol => Line::from(vec![Span::styled("fsym", s0), Span::styled(" Find a symbol.", s1)]),
            Command::Text => Line::from(vec![Span::styled("text", s0), Span::styled(" Insert text.", s1)]),
            Command::Patch => Line::from(vec![Span::styled("patch", s0), Span::styled(" Patch assembly.", s1)]),
            Command::JumpToAddress => Line::from(vec![Span::styled("jump", s0), Span::styled(" Jump to address.", s1)]),
            Command::SwitchView => Line::from(vec![Span::styled("view", s0), Span::styled(" Switch between text and assembly.", s1)]),

            Command::Empty => Line::from(vec![Span::styled("", s0), Span::styled("", s1)]),
            Command::Unknown => Line::from(vec![Span::styled("Unknown command", s0), Span::styled(" Unknown command", s1)]),
        }.left_aligned()
    }
}

impl App
{
    pub(super) fn find_commands(&mut self, command: &str) -> Vec<Command>
    {
        let ret = self.commands.fuzzy_search_sorted(command);
        ret.into_iter().map(|cmd| Command::from_string(&cmd)).collect()
    }

    pub(super) fn run_command(&mut self, command: &str, scroll: usize) -> Result<(), Box<dyn Error>>
    {
        let command_opt = self.find_commands(command).into_iter().nth(scroll);
        let command_enum = command_opt.expect("Scroll out of bounds for run_command.");
        self.popup = None;
        match command_enum
        {
            Command::Quit => {
                self.quit(None)?;
            }
            Command::QuitWithoutSave => {
                self.quit(Some(false))?;
            }
            Command::QuitWithSave => {
                self.quit(Some(true))?;
            }
            Command::SaveAs => {
                self.request_popup_save_as();
            }
            Command::Save => {
                if self.dirty
                {
                    self.save_data()?;
                }
            }
            Command::Help => {
                self.request_popup_help();
            }
            Command::Open => {
                self.request_open()?;
            }
            Command::Log => {
                self.request_popup_log();
            }
            Command::Run => {
                self.request_popup_run();
            }
            Command::FindText => {
                self.request_popup_find_text();
            }
            Command::FindSymbol => {
                self.request_popup_find_symbol();
            }
            Command::Text => {
                self.request_popup_text();
            }
            Command::Patch => {
                self.request_popup_patch();
            }
            Command::JumpToAddress => {
                self.request_popup_jump();
            }
            Command::SwitchView => {
                self.request_view_change();
            }

            Command::Empty => {}
            Command::Unknown => {
                self.log(NotificationLevel::Error, &format!("Unknown command: \"{}\"", command));
            }
        }
        Ok(())
    }

    pub(super) fn quit(&mut self, save: Option<bool>) -> Result<(), Box<dyn Error>>
    {
        match save
        {
            Some(true) => {
                self.log(NotificationLevel::Debug, "Saving and quitting...");
                if self.dirty
                {
                    self.save_data()?;
                }
                self.needs_to_exit = true;       
            }
            Some(false) => {
                self.log(NotificationLevel::Debug, "Quitting without saving...");
                self.needs_to_exit = true;
            }
            None => {
                self.log(NotificationLevel::Debug, "Quitting...");
                if self.dirty
                {
                    self.log(NotificationLevel::Warning, "You have unsaved changes.")
                }
                else
                {
                    self.needs_to_exit = true;
                }
            }
        }
        Ok(())
    }

    pub(super) fn request_quit(&mut self)
    {
        if self.dirty
        {
            self.popup = Some(PopupState::QuitDirtySave(false));
        }
        else
        {
            self.needs_to_exit = true;
        }
    }

    pub(super) fn request_save(&mut self)
    {
        if self.dirty
        {
            self.popup = Some(PopupState::Save(false));
        }
    }

    pub(super) fn request_save_and_quit(&mut self)
    {
        if self.dirty
        {
            self.popup = Some(PopupState::SaveAndQuit(false));
        }
        else
        {
            self.needs_to_exit = true;
        }
    }

    pub(super) fn request_open(&mut self) -> Result<(), Box<dyn Error>>
    {
        let mut new_popup = None;
        Self::open_dir(&mut new_popup, &self.get_current_dir(), &mut self.filesystem)?;
        self.popup = new_popup;
        Ok(())
    }

    pub(super) fn request_popup_save_as(&mut self)
    {
        let path = self.filesystem.pwd().to_string();
        let cursor = path.len();
        self.popup = Some(PopupState::SaveAs { 
                path, 
                cursor
            }
        );
    }

    pub(super) fn request_popup_help(&mut self)
    {
        self.popup = Some(PopupState::Help(0));
    }

    pub(super) fn request_popup_log(&mut self)
    {
        self.notification.reset();
        self.popup = Some(PopupState::Log(0));
    }

    pub(super) fn request_popup_run(&mut self)
    {
        self.popup = Some(PopupState::Run { 
            command: String::new(), 
            cursor: 0, 
            results: self.find_commands(""), 
            scroll: 0 }
        );
    }

    pub(super) fn request_popup_find_symbol(&mut self)
    {
        self.popup = Some(PopupState::FindSymbol { 
            filter: String::new(), 
            symbols: Vec::new(), 
            cursor: 0, 
            scroll: 0 }
        );
    }

    pub(super) fn request_popup_find_text(&mut self)
    {
        self.popup = Some(PopupState::FindText { 
            text: self.text_last_searched_string.clone(), 
            cursor: 0 }
        );
    }

    pub(super) fn request_popup_text(&mut self)
    {
        self.popup = Some(PopupState::InsertText { 
            text: String::new(), 
            cursor: 0 }
        );
    }

    pub(super) fn request_popup_patch(&mut self)
    {
        self.popup = Some(PopupState::Patch { 
            assembly: String::new(), 
            preview: Ok(Vec::new()), 
            cursor: 0}
        );
    }

    pub(super) fn request_popup_jump(&mut self)
    {
        self.popup = Some(PopupState::JumpToAddress { 
            location: String::new(), 
            cursor: 0}
        );
    }

    pub(super) fn request_view_change(&mut self)
    {
        match self.info_mode {
            super::info_mode::InfoMode::Text => 
            {
                self.info_mode = super::info_mode::InfoMode::Assembly;
            },
            super::info_mode::InfoMode::Assembly => 
            {
                self.info_mode = super::info_mode::InfoMode::Text;
            },
        }
    }
}