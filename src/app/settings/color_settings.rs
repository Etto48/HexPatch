use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

use crate::app::App;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorSettings
{
    pub address_selected: Style,
    pub address_default: Style,

    pub hex_selected: Style,
    pub hex_null: Style,
    pub hex_alphanumeric: Style,
    pub hex_symbol: Style,
    pub hex_end_of_line: Style,
    pub hex_whitespace: Style,
    pub hex_current_instruction: Style,
    pub hex_current_section: Style,
    pub hex_default: Style,

    pub text_selected: Style,

    pub assembly_symbol: Style,
    pub assembly_selected: Style,
    pub assembly_address: Style,
    pub assembly_virtual_address: Style,
    pub assembly_nop: Style,
    pub assembly_bad: Style,
    pub assembly_section: Style,
    pub assembly_entry_point: Style,
    pub assembly_default: Style,

    pub patch_patched_less_or_equal: Style,
    pub patch_patched_greater: Style,
    pub patch_old_instruction: Style,
    pub patch_old_rest: Style,
    pub patch_line_number: Style,

    pub help_command: Style,
    pub hep_description: Style,

    pub yes: Style,
    pub yes_selected: Style,
    pub no: Style,
    pub no_selected: Style,
    pub menu_text: Style,
    pub menu_text_selected: Style,

    pub insert_text_status: Style,

    pub command_name: Style,
    pub command_description: Style,
    pub command_selected: Style,

    pub path_dir: Style,
    pub path_file: Style,
    pub path_selected: Style,

    pub log_info: Style,
    pub log_debug: Style,
    pub log_warning: Style,
    pub log_error: Style,
    pub log_message: Style,

    pub status_bar: Style,
    pub status_info: Style,
    pub status_debug: Style,
    pub status_warning: Style,
    pub status_error: Style,

    pub scrollbar: Style,
    pub placeholder: Style,
}

impl Default for ColorSettings
{
    fn default() -> Self
    {
        let status_bar_bg = Color::Rgb(255, 223, 168);
        Self
        {
            address_selected: Style::default().fg(Color::Black).bg(Color::White),
            address_default: Style::default().fg(Color::DarkGray),

            hex_selected: Style::default().fg(Color::Black).bg(Color::White),
            hex_null: Style::default().fg(Color::DarkGray),
            hex_alphanumeric: Style::default().fg(Color::Rgb(204, 152, 113)),
            hex_symbol: Style::default().fg(Color::Rgb(204, 152, 113)).add_modifier(Modifier::DIM),
            hex_end_of_line: Style::default().fg(Color::LightRed),
            hex_whitespace: Style::default().fg(Color::Rgb(244, 202, 183)),
            hex_current_instruction: Style::default().fg(Color::Black).bg(Color::Rgb(215, 170, 92)),
            hex_current_section: Style::default().fg(Color::Black).bg(Color::Rgb(215, 170, 92)),
            hex_default: Style::default(),

            text_selected: Style::default().fg(Color::Black).bg(Color::White),

            assembly_symbol: Style::default().fg(Color::LightGreen),
            assembly_selected: Style::default().fg(Color::Black).bg(Color::White),
            assembly_address: Style::default().fg(Color::DarkGray),
            assembly_virtual_address: Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
            assembly_nop: Style::default().fg(Color::DarkGray),
            assembly_bad: Style::default().fg(Color::LightRed),
            assembly_section: Style::default().fg(Color::LightBlue),
            assembly_entry_point: Style::default().fg(Color::Yellow),
            assembly_default: Style::default().fg(Color::Rgb(204, 152, 113)),

            patch_patched_less_or_equal: Style::default().fg(Color::Green),
            patch_patched_greater: Style::default().fg(Color::Yellow),
            patch_old_instruction: Style::default().fg(Color::Red),
            patch_old_rest: Style::default().fg(Color::DarkGray),
            patch_line_number: Style::default().fg(Color::DarkGray),

            help_command: Style::default().fg(Color::LightGreen),
            hep_description: Style::default().fg(Color::Gray),

            yes: Style::default().fg(Color::Green),
            yes_selected: Style::default().fg(Color::Black).bg(Color::Green),
            no: Style::default().fg(Color::Red),
            no_selected: Style::default().fg(Color::Black).bg(Color::Red),
            menu_text: Style::default().fg(Color::White),
            menu_text_selected: Style::default().fg(Color::Black).bg(Color::White),

            insert_text_status: Style::default().fg(Color::Black).bg(status_bar_bg),

            command_name: Style::default().fg(Color::LightGreen),
            command_description: Style::default().fg(Color::Gray),
            command_selected: Style::default().fg(Color::Black).bg(Color::White),

            path_dir: Style::default().fg(Color::Blue),
            path_file: Style::default().fg(Color::Yellow),
            path_selected: Style::default().fg(Color::Black).bg(Color::White),

            log_info: Style::default().fg(Color::LightBlue),
            log_debug: Style::default().fg(Color::LightGreen),
            log_warning: Style::default().fg(Color::Yellow),
            log_error: Style::default().fg(Color::Red),
            log_message: Style::default().fg(Color::White),

            status_bar: Style::default().fg(Color::Black).bg(status_bar_bg),
            status_info: Style::default().fg(Color::Blue).bg(status_bar_bg),
            status_debug: Style::default().fg(Color::Green).bg(status_bar_bg),
            status_warning: Style::default().fg(Color::Yellow).bg(status_bar_bg),
            status_error: Style::default().fg(Color::Red).bg(status_bar_bg),

            scrollbar: Style::default().fg(status_bar_bg).bg(Color::DarkGray),
            placeholder: Style::default().fg(Color::DarkGray),
        }
    }
}

impl App
{
    pub(in crate::app) fn get_style_for_byte(color_settings: &ColorSettings, byte: u8) -> Style
    {
        match byte
        {
            // null
            0x00 => color_settings.hex_null,
            // newline
            0x0A | 0x0C | 0x0D => color_settings.hex_end_of_line,
            // whitespace
            0x20 | 0x09 | 0x0B => color_settings.hex_whitespace,
            // numbers
            0x30..=0x39 |
            // uppercase
            0x41..=0x5A |
            // lowercase
            0x61..=0x7A => color_settings.hex_alphanumeric,
            // special characters
            0x20..=0x7E => color_settings.hex_symbol,
            _ => color_settings.hex_default
        }
    }
}