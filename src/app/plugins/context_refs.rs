use crate::{app::{log::logger::Logger, popup_state::PopupState, settings::Settings}, headers::Header};

use super::instruction_info::InstructionInfo;

#[macro_export]
macro_rules! get_context_refs {
    ($app:ident) => {
        $crate::app::plugins::context_refs::ContextRefs::new(
            $app.get_cursor_position().global_byte_index, 
            $app.get_current_instruction()
                .map(|i| i.into()), 
            &mut $app.data, 
            &$app.header, 
            &$app.settings, 
            &mut $app.logger, 
            &mut $app.popup
        )
    };
}

pub struct ContextRefs<'app> {
    pub data: &'app mut Vec<u8>,
    pub offset: usize,
    pub current_instruction: Option<InstructionInfo>,
    pub header: &'app Header,
    pub settings: &'app Settings,
    pub logger: &'app mut Logger,
    pub popup: &'app mut Option<PopupState>,
}

impl<'app> ContextRefs<'app> {
    pub fn new(
        offset: usize,
        current_instruction: Option<InstructionInfo>,
        data: &'app mut Vec<u8>,
        header: &'app Header, 
        settings: &'app Settings, 
        logger: &'app mut Logger, 
        popup: &'app mut Option<PopupState>) -> Self {
        Self {
            data,
            offset,
            current_instruction,
            header,
            settings,
            logger,
            popup,
        }
    }
}