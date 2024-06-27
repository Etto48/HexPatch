use std::error::Error;

use crate::{app::{info_mode::InfoMode, log::NotificationLevel, popup::{binary_choice::BinaryChoice, popup_state::PopupState, simple_choice::SimpleChoice}, App}, fuzzer::fuzzy_search_in_place, get_app_context};

use super::command_info::CommandInfo;

impl App
{
    pub(in crate::app) fn find_commands(&mut self, command: &str) -> Vec<CommandInfo>
    {
        let mut commands = CommandInfo::full_list_of_commands(&self.plugin_manager);
        //commands.retain(|c| c.command.contains(command));
        fuzzy_search_in_place(command, &mut commands);
        commands
    }

    pub(in crate::app) fn run_command(&mut self, command: &str, scroll: usize) -> Result<(), Box<dyn Error>>
    {
        let command_opt = self.find_commands(command).into_iter().nth(scroll);
        let command_info = command_opt.expect("Scroll out of bounds for run_command.");
        self.popup = None;
        match command_info.command.as_str()
        {
            "quit" => {
                self.quit(None)?;
            }
            "dquit" => {
                self.quit(Some(false))?;
            }
            "xquit" => {
                self.quit(Some(true))?;
            }
            "save" => {
                if self.dirty
                {
                    self.save_file()?;
                }
            }
            "saveas" => {
                self.request_popup_save_as();
            }
            "help" => {
                self.request_popup_help();
            }
            "open" => {
                self.request_open()?;
            }
            "log" => {
                self.request_popup_log();
            }
            "run" => {
                self.request_popup_run();
            }
            "ftext" => {
                self.request_popup_find_text();
            }
            "fsym" => {
                self.request_popup_find_symbol();
            }
            "text" => {
                self.request_popup_text();
            }
            "patch" => {
                self.request_popup_patch();
            }
            "jump" => {
                self.request_popup_jump();
            }
            "view" => {
                self.request_view_change();
            }
            any_other_command => {
                let mut app_context = get_app_context!(self);
                self.plugin_manager.run_command(
                    any_other_command,
                    &mut app_context
                )?;
            }
        }
        Ok(())
    }

    pub(in crate::app) fn quit(&mut self, save: Option<bool>) -> Result<(), Box<dyn Error>>
    {
        match save
        {
            Some(true) => {
                self.log(NotificationLevel::Debug, "Saving and quitting...");
                if self.dirty
                {
                    self.save_file()?;
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

    pub(in crate::app) fn request_quit(&mut self)
    {
        if self.dirty
        {
            self.popup = Some(PopupState::QuitDirtySave(SimpleChoice::Cancel));
        }
        else
        {
            self.needs_to_exit = true;
        }
    }

    pub(in crate::app) fn request_save(&mut self)
    {
        if self.dirty
        {
            self.popup = Some(PopupState::Save(BinaryChoice::No));
        }
    }

    pub(in crate::app) fn request_save_and_quit(&mut self)
    {
        if self.dirty
        {
            self.popup = Some(PopupState::SaveAndQuit(BinaryChoice::No));
        }
        else
        {
            self.needs_to_exit = true;
        }
    }

    pub(in crate::app) fn request_open(&mut self) -> Result<(), Box<dyn Error>>
    {
        let mut new_popup = None;
        Self::open_dir(&mut new_popup, &self.get_current_dir(), &mut self.filesystem)?;
        self.popup = new_popup;
        Ok(())
    }

    pub(in crate::app) fn request_popup_save_as(&mut self)
    {
        let path = self.filesystem.pwd().to_string();
        let cursor = path.len();
        self.popup = Some(PopupState::SaveAs { 
                path, 
                cursor
            }
        );
    }

    pub(in crate::app) fn request_popup_help(&mut self)
    {
        self.popup = Some(PopupState::Help(0));
    }

    pub(in crate::app) fn request_popup_log(&mut self)
    {
        self.logger.reset_notification_level();
        self.popup = Some(PopupState::Log(0));
    }

    pub(in crate::app) fn request_popup_run(&mut self)
    {
        self.popup = Some(PopupState::Run { 
            command: String::new(), 
            cursor: 0, 
            results: self.find_commands(""), 
            scroll: 0 }
        );
    }

    pub(in crate::app) fn request_popup_find_symbol(&mut self)
    {
        self.popup = Some(PopupState::FindSymbol { 
            filter: String::new(), 
            symbols: Vec::new(), 
            cursor: 0, 
            scroll: 0 }
        );
    }

    pub(in crate::app) fn request_popup_find_text(&mut self)
    {
        self.popup = Some(PopupState::FindText { 
            text: self.text_last_searched_string.clone(), 
            cursor: 0 }
        );
    }

    pub(in crate::app) fn request_popup_text(&mut self)
    {
        self.popup = Some(PopupState::InsertText { 
            text: String::new(), 
            cursor: 0 }
        );
    }

    pub(in crate::app) fn request_popup_patch(&mut self)
    {
        self.popup = Some(PopupState::Patch { 
            assembly: String::new(), 
            preview: Ok(Vec::new()), 
            cursor: 0}
        );
    }

    pub(in crate::app) fn request_popup_jump(&mut self)
    {
        self.popup = Some(PopupState::JumpToAddress { 
            location: String::new(), 
            cursor: 0}
        );
    }

    pub(in crate::app) fn request_view_change(&mut self)
    {
        match self.info_mode {
            InfoMode::Text => 
            {
                self.info_mode = InfoMode::Assembly;
            },
            InfoMode::Assembly => 
            {
                self.info_mode = InfoMode::Text;
            },
        }
    }
}