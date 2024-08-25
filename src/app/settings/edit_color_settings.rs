#[macro_export]
macro_rules! EditColorSettings {(
    $(#[$attr:meta])*
    $pub:vis struct $color_settings:ident {
        $(
            $(#[$field_attr:meta])*
            $field_pub:vis $field_name:ident: $field_type:ty,
        )*
    }) => {
        impl $color_settings
        {
            pub fn edit_color_settings(&mut self, data: &std::collections::HashMap<String, ratatui::style::Style>) -> Result<(), String>
            {
                for (key, value) in data.iter()
                {
                    match key.as_str() {
                        $(
                            stringify!($field_name) => self.$field_name = value.clone(),
                        )*
                        key => {
                            return Err(format!("Unknown field: {}", key));
                        }
                    }
                }
                Ok(())
            }
        }
    };
}
