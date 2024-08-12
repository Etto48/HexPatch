use mlua::IntoLua;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiLocationInfo {
    AddressView {
        file_address: Option<u64>,
    },
    HexView {
        file_address: Option<u64>,
        high: Option<bool>,
        virtual_address: Option<u64>,
        byte: Option<u8>,
    },
    TextView {
        file_address: Option<u64>,
        virtual_address: Option<u64>,
        byte: Option<u8>,
        character: Option<char>,
    },
    AssemblyView {
        section: Option<String>,
        file_address: Option<u64>,
        virtual_address: Option<u64>,
        instruction: Option<String>,
    },
    StatusBar,
    ScrollBar,
    Popup {
        name: String,
    },
}

impl<'lua> IntoLua<'lua> for UiLocationInfo {
    fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let ret = lua.create_table()?;
        match self {
            UiLocationInfo::AddressView { file_address } => {
                ret.set("type", "AddressView")?;
                ret.set("file_address", file_address)?;
            }
            UiLocationInfo::HexView {
                file_address,
                high,
                virtual_address,
                byte,
            } => {
                ret.set("type", "HexView")?;
                ret.set("file_address", file_address)?;
                ret.set("high", high)?;
                ret.set("virtual_address", virtual_address)?;
                ret.set("byte", byte)?;
            }
            UiLocationInfo::TextView {
                file_address,
                virtual_address,
                byte,
                character,
            } => {
                ret.set("type", "TextView")?;
                ret.set("file_address", file_address)?;
                ret.set("virtual_address", virtual_address)?;
                ret.set("byte", byte)?;
                ret.set("character", character.map(|c| c.to_string()))?;
            }
            UiLocationInfo::AssemblyView {
                section,
                file_address,
                virtual_address,
                instruction,
            } => {
                ret.set("type", "AssemblyView")?;
                ret.set("section", section)?;
                ret.set("file_address", file_address)?;
                ret.set("virtual_address", virtual_address)?;
                ret.set("instruction", instruction)?;
            }
            UiLocationInfo::StatusBar => {
                ret.set("type", "StatusBar")?;
            }
            UiLocationInfo::ScrollBar => {
                ret.set("type", "ScrollBar")?;
            }
            UiLocationInfo::Popup { name } => {
                ret.set("type", "Popup")?;
                ret.set("name", name)?;
            }
        }
        Ok(mlua::Value::Table(ret))
    }
}
