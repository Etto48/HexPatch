use mlua::IntoLua;

use crate::app::{
    asm::assembly_line::AssemblyLine, frame_info::InfoViewFrameInfo,
    popup::popup_state::PopupState, App,
};

use super::{point::Point, rect_borders::RectBorders, ui_location_info::UiLocationInfo};

#[derive(Clone, Debug)]
pub struct UiLocation {
    pub info: UiLocationInfo,
    pub relative_location: Point,
}

impl<'lua> IntoLua<'lua> for UiLocation {
    fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let ret = lua.create_table()?;
        ret.set("info", self.info)?;
        ret.set("relative_location", self.relative_location)?;
        Ok(mlua::Value::Table(ret))
    }
}

impl App {
    fn get_hex_and_text_view_byte_info(
        &self,
        relative_location: Point,
        is_text: bool,
        borders: RectBorders,
    ) -> (Option<u64>, Option<u64>, Option<u8>, Option<bool>) {
        let (byte_size, offset_x) = if is_text {
            if borders.top || borders.right {
                return (None, None, None, None);
            }
            (2, relative_location.x as usize)
        } else {
            if borders.left || borders.top || borders.right {
                return (None, None, None, None);
            }
            (3, relative_location.x as usize - 1)
        };
        let current_block = self.last_frame_info.blocks_per_row
            * (self.last_frame_info.scroll + (relative_location.y - 1) as usize)
            + offset_x / (self.block_size * byte_size + 1); //This should not underflow because of the previous check

        let block_offset = offset_x % (self.block_size * byte_size + 1);
        let block_offset_in_bytes = block_offset / byte_size;
        let current_byte = current_block * self.block_size + block_offset_in_bytes;
        if block_offset % byte_size == byte_size - 1
            || block_offset == (self.block_size * byte_size)
            || current_byte >= self.last_frame_info.file_size
        {
            (None, None, None, None)
        } else {
            (
                Some(current_byte as u64),
                Some(
                    self.header
                        .physical_to_virtual_address(current_byte as u64)
                        .unwrap_or(current_byte as u64),
                ),
                Some(self.data.bytes()[current_byte]),
                Some(block_offset % byte_size == 0),
            )
        }
    }

    pub(in crate::app) fn get_ui_location(&self, global_location: Point) -> Option<UiLocation> {
        if let (Some(popup_rect), Some(popup_info)) = (&self.last_frame_info.popup, &self.popup) {
            if let Some((relative_location, _borders)) =
                global_location.get_relative_location(popup_rect)
            {
                let name = match popup_info {
                    PopupState::Open { .. } => "Open",
                    PopupState::Run { .. } => "Run",
                    PopupState::FindText { .. } => "FindText",
                    PopupState::FindSymbol { .. } => "FindSymbol",
                    PopupState::Log(_) => "Log",
                    PopupState::InsertText { .. } => "InsertText",
                    PopupState::Patch { .. } => "Patch",
                    PopupState::JumpToAddress { .. } => "JumpToAddress",
                    PopupState::QuitDirtySave(_) => "QuitDirtySave",
                    PopupState::SaveAndQuit(_) => "SaveAndQuit",
                    PopupState::SaveAs { .. } => "SaveAs",
                    PopupState::Save(_) => "Save",
                    PopupState::Help(_) => "Help",
                    PopupState::Custom { .. } => "Custom",
                }
                .into();
                return Some(UiLocation {
                    info: UiLocationInfo::Popup { name },
                    relative_location,
                });
            }
        }
        if let Some((relative_location, _borders)) =
            global_location.get_relative_location(&self.last_frame_info.scroll_bar)
        {
            Some(UiLocation {
                info: UiLocationInfo::ScrollBar,
                relative_location,
            })
        } else if let Some((relative_location, _borders)) =
            global_location.get_relative_location(&self.last_frame_info.status_bar)
        {
            Some(UiLocation {
                info: UiLocationInfo::StatusBar,
                relative_location,
            })
        } else if let Some((relative_location, borders)) =
            global_location.get_relative_location(&self.last_frame_info.address_view)
        {
            let file_address = if borders.left || borders.top {
                None
            } else {
                let bytes_per_row = self.last_frame_info.blocks_per_row * self.block_size;
                let starting_byte = self.last_frame_info.scroll * bytes_per_row;
                let offset = relative_location.y - 1; // This should not underflow because of the previous check
                let current_byte = starting_byte + (offset as usize * bytes_per_row);
                if current_byte < self.last_frame_info.file_size {
                    Some(current_byte as u64)
                } else {
                    None
                }
            };
            Some(UiLocation {
                info: UiLocationInfo::AddressView { file_address },
                relative_location,
            })
        } else if let Some((relative_location, borders)) =
            global_location.get_relative_location(&self.last_frame_info.hex_view)
        {
            let (file_address, virtual_address, byte, high) =
                self.get_hex_and_text_view_byte_info(relative_location, false, borders);

            Some(UiLocation {
                info: UiLocationInfo::HexView {
                    file_address,
                    virtual_address,
                    byte,
                    high,
                },
                relative_location,
            })
        } else if let Some((relative_location, borders)) =
            global_location.get_relative_location(&self.last_frame_info.info_view)
        {
            match &self.last_frame_info.info_view_frame_info {
                InfoViewFrameInfo::TextView => {
                    let (file_address, virtual_address, byte, _high) =
                        self.get_hex_and_text_view_byte_info(relative_location, true, borders);

                    Some(UiLocation {
                        info: UiLocationInfo::TextView {
                            file_address,
                            virtual_address,
                            byte,
                            character: byte.and_then(|b| char::from_u32(b as u32)),
                        },
                        relative_location,
                    })
                }
                InfoViewFrameInfo::AssemblyView { scroll } => {
                    let (section, file_address, virtual_address, instruction) =
                        if borders.top || borders.right {
                            (None, None, None, None)
                        } else {
                            let instruction_offset = scroll + relative_location.y as usize - 1;
                            if let Some(assembly_line) =
                                self.assembly_instructions.get(instruction_offset)
                            {
                                match assembly_line {
                                    AssemblyLine::Instruction(instruction_tag) => (
                                        Some(
                                            self.header
                                                .get_text_section()
                                                .map(|s| s.name.clone())
                                                .unwrap_or(".text".into()),
                                        ),
                                        Some(instruction_tag.file_address),
                                        Some(instruction_tag.instruction.ip()),
                                        Some(instruction_tag.instruction.to_string()),
                                    ),
                                    AssemblyLine::SectionTag(section_tag) => (
                                        Some(section_tag.name.clone()),
                                        Some(section_tag.file_address),
                                        Some(section_tag.virtual_address),
                                        None,
                                    ),
                                }
                            } else {
                                (None, None, None, None)
                            }
                        };
                    Some(UiLocation {
                        info: UiLocationInfo::AssemblyView {
                            section,
                            file_address,
                            virtual_address,
                            instruction,
                        },
                        relative_location,
                    })
                }
            }
        } else {
            None
        }
    }
}
