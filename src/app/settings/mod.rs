mod settings;
pub use settings::Settings;

pub mod app_settings;
pub mod color_settings;
pub mod key_settings;
#[macro_use]
pub mod register_key_settings_macro;
#[macro_use]
pub mod register_color_settings_macro;
pub mod settings_value;
