use std::error::Error;

use crate::{
    app::{
        info_mode::InfoMode,
        log::NotificationLevel,
        popup::{
            binary_choice::BinaryChoice, popup_state::PopupState, simple_choice::SimpleChoice,
        },
        App,
    },
    fuzzer::fuzzy_search_in_place,
    get_app_context,
};

use super::command_info::CommandInfo;

impl App {
    pub(in crate::app) fn find_commands(&mut self, command: &str) -> Vec<CommandInfo> {
        let mut commands = CommandInfo::full_list_of_commands(&self.plugin_manager);
        //commands.retain(|c| c.command.contains(command));
        fuzzy_search_in_place(command, &mut commands);
        commands
    }

    pub(in crate::app) fn run_command(
        &mut self,
        command: &str,
        scroll: usize,
    ) -> Result<(), Box<dyn Error>> {
        let command_opt = self.find_commands(command).into_iter().nth(scroll);
        let command_info = command_opt.expect(&t!("errors.run_command_scroll_out_of_bounds"));
        self.popup = None;
        match command_info.command.as_str() {
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
                if self.data.dirty() {
                    self.save_file()?;
                }
            }
            "saveas" => {
                self.request_popup_save_as();
            }
            "csave" => {
                self.save_comments(None);
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
            "fcom" => {
                self.request_popup_find_comment();
            }
            "ecom" => {
                self.request_popup_edit_comment();
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
            "undo" => {
                self.undo();
            }
            "redo" => {
                self.redo();
            }
            any_other_command => {
                let mut app_context = get_app_context!(self);
                self.plugin_manager
                    .run_command(any_other_command, &mut app_context)?;
            }
        }
        Ok(())
    }

    pub(in crate::app) fn quit(&mut self, save: Option<bool>) -> Result<(), Box<dyn Error>> {
        match save {
            Some(true) => {
                self.log(
                    NotificationLevel::Debug,
                    t!("app.messages.saving_and_quitting"),
                );
                if self.data.dirty() {
                    self.save_file()?;
                }
                self.needs_to_exit = true;
            }
            Some(false) => {
                self.log(
                    NotificationLevel::Debug,
                    t!("app.messages.quitting_without_saving"),
                );
                self.needs_to_exit = true;
            }
            None => {
                self.log(NotificationLevel::Debug, t!("app.messages.quitting"));
                if self.data.dirty() {
                    self.log(
                        NotificationLevel::Warning,
                        t!("app.messages.unsaved_changes"),
                    )
                } else {
                    self.needs_to_exit = true;
                }
            }
        }
        Ok(())
    }

    pub(in crate::app) fn request_quit(&mut self) {
        if self.data.dirty() {
            self.popup = Some(PopupState::QuitDirtySave(SimpleChoice::Cancel));
        } else {
            self.needs_to_exit = true;
        }
    }

    pub(in crate::app) fn request_save(&mut self) {
        if self.data.dirty() {
            self.popup = Some(PopupState::Save(BinaryChoice::No));
        }
    }

    pub(in crate::app) fn request_save_and_quit(&mut self) {
        if self.data.dirty() {
            self.popup = Some(PopupState::SaveAndQuit(BinaryChoice::No));
        } else {
            self.needs_to_exit = true;
        }
    }

    pub(in crate::app) fn request_open(&mut self) -> Result<(), Box<dyn Error>> {
        let mut new_popup = None;
        Self::open_dir(
            &mut new_popup,
            &self.get_current_dir(),
            &mut self.filesystem,
        )?;
        self.popup = new_popup;
        Ok(())
    }

    pub(in crate::app) fn request_popup_save_as(&mut self) {
        let path = self.filesystem.pwd().to_string();
        let cursor = path.len();
        self.popup = Some(PopupState::SaveAs { path, cursor });
    }

    pub(in crate::app) fn request_popup_help(&mut self) {
        self.popup = Some(PopupState::Help(0));
    }

    pub(in crate::app) fn request_popup_log(&mut self) {
        self.logger.reset_notification_level();
        self.popup = Some(PopupState::Log(0));
    }

    pub(in crate::app) fn request_popup_run(&mut self) {
        self.popup = Some(PopupState::Run {
            command: String::new(),
            cursor: 0,
            results: self.find_commands(""),
            scroll: 0,
        });
    }

    pub(in crate::app) fn request_popup_find_symbol(&mut self) {
        self.popup = Some(PopupState::FindSymbol {
            filter: String::new(),
            symbols: Vec::new(),
            cursor: 0,
            scroll: 0,
        });
    }

    pub(in crate::app) fn request_popup_edit_comment(&mut self) {
        let comment = self
            .comments
            .get(&(self.get_cursor_position().global_byte_index as u64))
            .cloned()
            .unwrap_or_default();
        let cursor = comment.len();
        self.popup = Some(PopupState::EditComment { comment, cursor });
    }

    pub(in crate::app) fn request_popup_find_comment(&mut self) {
        self.popup = Some(PopupState::FindComment {
            filter: String::new(),
            comments: Vec::new(),
            cursor: 0,
            scroll: 0,
        });
    }

    pub(in crate::app) fn request_popup_find_text(&mut self) {
        self.popup = Some(PopupState::FindText {
            text: self.text_last_searched_string.clone(),
            cursor: 0,
        });
    }

    pub(in crate::app) fn request_popup_text(&mut self) {
        self.popup = Some(PopupState::InsertText {
            text: String::new(),
            cursor: 0,
        });
    }

    pub(in crate::app) fn request_popup_patch(&mut self) {
        self.popup = Some(PopupState::Patch {
            assembly: String::new(),
            preview: Ok(Vec::new()),
            cursor: 0,
        });
    }

    pub(in crate::app) fn request_popup_jump(&mut self) {
        self.popup = Some(PopupState::JumpToAddress {
            location: String::new(),
            cursor: 0,
        });
    }

    pub(in crate::app) fn request_view_change(&mut self) {
        match self.info_mode {
            InfoMode::Text => {
                self.info_mode = InfoMode::Assembly;
            }
            InfoMode::Assembly => {
                self.info_mode = InfoMode::Text;
            }
        }
    }

    pub(in crate::app) fn undo(&mut self) {
        if let Some(change) = self.data.undo().cloned() {
            let instruction_offset = self.get_instruction_at(change.offset()).file_address();
            let instruction_offset = change
                .offset()
                .checked_sub(instruction_offset as usize)
                .unwrap();
            self.edit_assembly(change.offset() + instruction_offset);
        } else {
            self.log(
                NotificationLevel::Warning,
                t!("app.messages.nothing_to_undo"),
            )
        }
    }

    pub(in crate::app) fn redo(&mut self) {
        if let Some(change) = self.data.redo().cloned() {
            let instruction_offset = self.get_instruction_at(change.offset()).file_address();
            let instruction_offset = change
                .offset()
                .checked_sub(instruction_offset as usize)
                .unwrap();
            self.edit_assembly(change.offset() + instruction_offset);
        } else {
            self.log(
                NotificationLevel::Warning,
                t!("app.messages.nothing_to_redo"),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::app::asm::assembly_line::AssemblyLine;

    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut app = App::mockup(vec![0x90; 4]);
        app.patch_bytes(&[0, 0], false);
        if let AssemblyLine::Instruction(instruction) = app.assembly_instructions[1].clone() {
            assert_eq!(instruction.instruction.mnemonic, "add");
        } else {
            panic!("Expected an instruction.")
        }

        app.undo();
        assert_eq!(app.data.bytes(), vec![0x90; 4].as_slice());
        if let AssemblyLine::Instruction(instruction) = app.assembly_instructions[1].clone() {
            assert_eq!(instruction.instruction.mnemonic, "nop");
        } else {
            panic!("Expected an instruction.")
        }

        app.redo();
        assert_eq!(app.data.bytes(), vec![0x00, 0x00, 0x90, 0x90].as_slice());
        if let AssemblyLine::Instruction(instruction) = app.assembly_instructions[1].clone() {
            assert_eq!(instruction.instruction.mnemonic, "add");
        } else {
            panic!("Expected an instruction.")
        }
    }
}
