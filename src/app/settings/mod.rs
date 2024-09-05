mod settings;
pub use settings::Settings;

pub mod app_settings;
pub mod color_settings;
pub mod key_settings;
#[macro_use]
pub mod register_key_settings_macro;
#[macro_use]
pub mod register_color_settings_macro;
#[macro_use]
pub mod edit_color_settings;
pub mod settings_value;
pub mod theme_preference;
pub mod verbosity;
