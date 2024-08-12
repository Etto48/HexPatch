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
