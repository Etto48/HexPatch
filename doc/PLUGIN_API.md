# HexPatch Plugin API

This document describes the API that plugins can use to interact with HexPatch.

## Functions

For the explaination of the types used in the functions, see the [Types](#types) section.

### Initialization

```lua
function init(context) end
```

This function is called when the plugin is loaded.

| Argument | Type | Description |
|----------|------|-------------|
|`context`|`Context`|The application context.|

### Events

#### On Open

```lua
function on_open(context) end
```

This function is called when a file is opened.

| Argument | Type | Description |
|----------|------|-------------|
|`context`|`Context`|The application context.|

#### On Close

```lua
function on_save(context) end
```

This function is called when a file is saved (both save and save as).

| Argument | Type | Description |
|----------|------|-------------|
|`context`|`Context`|The application context.|

#### On Edit

```lua
function on_edit(new_bytes, context) end
```

This function is called when the user edits the file, either directly or through a command.

| Argument | Type | Description |
|----------|------|-------------|
|`new_bytes`|`Data`|The new bytes of the file.|
|`context`|`Context`|The application context.|

#### On Key

```lua
function on_key(key_event, context) end
```

This function is called when a key is pressed.

| Argument | Type | Description |
|----------|------|-------------|
|`key_event`|`KeyEvent`|The key event.|
|`context`|`Context`|The application context.|

#### On Mouse

```lua
function on_mouse(mouse_event, context) end
```

This function is called when an action is performed with the mouse.

| Argument | Type | Description |
|----------|------|-------------|
|`mouse_event`|`MouseEvent`|The mouse event.|
|`context`|`Context`|The application context.|

### Commands

```lua
function COMMAND_NAME(context) end
```

This function is called when the user runs the command `COMMAND_NAME`.
The command must be registered using `context.add_command("COMMAND_NAME", "COMMAND_DESCRIPTION")`.

| Argument | Type | Description |
|----------|------|-------------|
|`context`|`Context`|The application context.|

### Popups

```lua
function FILL_POPUP_NAME(popup_text, popup_title, context) end
```

This function is called each time the popup `POPUP_NAME` is drawn.
The popup must be opened using `context.open_popup("POPUP_NAME")`.

| Argument | Type | Description |
|----------|------|-------------|
|`popup_text`|`Text`|The content of the popup.|
|`popup_title`|`MutString`|The title of the popup.|
|`context`|`Context`|The application context.|

## Types

### Context

This table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`data`|`Data`|The current file's data.|
|`offset`|`usize`|The current offset in the file.|
|`settings`|`Settings`|The settings of the application.|
|`current_instruction`|`Option<InstructionInfo>`|The current instruction at the current offset. `nil` if the current offset is not in an instruction or in a data section. The InstructionInfo type is explained at [InstructionInfo](#instructioninfo).|
|`header`|`Header`|The header of the file, if not valid or present, the default header will be used.|

And the following functions:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`log`|`(level: u8, message: String)`|Logs a message in the UI.|
|`add_command`|`(command_name: String)`|Registers a command, this must be called to make the command appear in the command list.|
|`remove_command`|`(command_name: String)`|Removes a command, this removes the command from the command list.|
|`open_popup`|`(popup_handler: String)`|Opens a popup, each time the popup is drawn the handler function is called|

For more information on the types, see the following sections.

### Settings

This type contains the settings of the application.
A setting can be accessed using the `.` operator with its full name (dots are replaced with underscores).
e.g. `context.settings.color_address_selected` or `context.settings.key_up`.

WARNING: You should get and set the setting altogether, e.g. `context.settings.color_address_selected = {fg = "Red"}`. Trying to set a single field will not work.

To access custom settings, use the following functions:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`get_custom`|`(setting_name: String) -> CustomSetting`|Gets the value of a custom setting.|
|`set_custom`|`(setting_name: String, value: CustomSetting)`|Sets the value of a custom setting.|

### Data

A mutable vector of bytes.

The following functions are available:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`get`|`(index: usize) -> u8`|Gets the byte at the specified index. THE INDEX IS 0 BASED!|
|`set`|`(index: usize, value: u8)`|Sets the byte at the specified index. THE INDEX IS 0 BASED!|
|`len`|`() -> usize`|Gets the length of the vector.|

### KeyEvent

This table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`code`|`String`|The key that was pressed, a list of possible values is available at [KeyEvent.code](#keyeventcode)|
|`modifiers`|`Table`|A table containing the modifiers that were pressed, the entries are explained at [KeyEvent.modifiers](#keyeventmodifiers)|
|`kind`|`String`|The kind of key event, either `Press`, `Repeat` or `Release`|
|`state`|`Table`|A table containing the state of the keys, the entries are explained at [KeyEvent.state](#keyeventstate)|

#### KeyEvent.code

The following values are possible for the `code` field:

- `Backspace`
- `Enter`
- `Left`
- `Right`
- `Up`
- `Down`
- `Home`
- `End`
- `PageUp`
- `PageDown`
- `Tab`
- `BackTab`
- `Delete`
- `Insert`
- `Fn` where `n` is the function key number.
- `c` where `c` is the character that was pressed (can be either lower or upper case).
- `Null`
- `Esc`
- `CapsLock`
- `ScrollLock`
- `NumLock`
- `PrintScreen`
- `Pause`
- `Menu`
- `KeypadBegin`
- `Media(Play)`
- `Media(Pause)`
- `Media(PlayPause)`
- `Media(Reverse)`
- `Media(Stop)`
- `Media(FastForward)`
- `Media(Rewind)`
- `Media(TrackNext)`
- `Media(TrackPrevious)`
- `Media(Record)`
- `Media(LowerVolume)`
- `Media(RaiseVolume)`
- `Media(MuteVolume)`
- `Modifier(LeftShift)`
- `Modifier(LeftControl)`
- `Modifier(LeftAlt)`
- `Modifier(LeftSuper)`
- `Modifier(LeftHyper)`
- `Modifier(LeftMeta)`
- `Modifier(RightShift)`
- `Modifier(RightControl)`
- `Modifier(RightAlt)`
- `Modifier(RightSuper)`
- `Modifier(RightHyper)`
- `Modifier(RightMeta)`
- `Modifier(IsoLevel3Shift)`
- `Modifier(IsoLevel5Shift)`

#### KeyEvent.modifiers

The table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`alt`|`bool`|Whether the alt key was pressed.|
|`control`|`bool`|Whether the control key was pressed.|
|`hyper`|`bool`|Whether the hyper key was pressed.|
|`meta`|`bool`|Whether the meta key was pressed.|
|`shift`|`bool`|Whether the shift key was pressed.|
|`super`|`bool`|Whether the super key was pressed.|

#### KeyEvent.state

The table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`caps_lock`|`bool`|Whether the caps lock key is active.|
|`keypad`|`bool`|Whether the event origins from the keypad.|
|`num_lock`|`bool`|Whether the num lock key is active.|

### MouseEvent

This table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`kind`|`String`|The kind of mouse event, a list of possible values is available at [MouseEvent.kind](#mouseeventkind)|
|`column`|`usize`|The column of the terminal where the event happened.|
|`row`|`usize`|The row of the terminal where the event happened.|
|`modifiers`|`Table`|A table containing the modifiers that were pressed, the entries are explained at [KeyEvent.modifiers](#keyeventmodifiers) (This is the same table of the KeyEvent)|

#### MouseEvent.kind

The following values are possible for the `kind` field:

- `Down(Left)`
- `Down(Right)`
- `Down(Middle)`
- `Up(Left)`
- `Up(Right)`
- `Up(Middle)`
- `Drag(Left)`
- `Drag(Right)`
- `Drag(Middle)`
- `Moved`
- `ScrollDown`
- `ScrollUp`
- `ScrollLeft`
- `ScrollRight`

### Text

To add text to a popup, use the following functions:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`push_line`|`(line: String)`|Adds a line of text to the popup.|
|`push_span`|`(span: String)`|Adds text without a newline to the popup.|

### MutString

This type is a mutable string, it can be manipulated using the following functions:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`get`|`() -> String`|Gets the string.|
|`set`|`(value: String)`|Sets the string.|

### InstructionInfo

This table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`instruction`|`String`|The line of assembly code, the format depends on the architecture.|
|`physical_address`|`u64`|The offset in the file where the instruction starts.|
|`virtual_address`|`u64`|The virtual address at which the instruction will be mapped.|
|`size`|`usize`|The size of the instruction in bytes.|

### Header

This type has the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`bitness`|`u32`|The bitness of the file. (The default value is 64)|
|`entry_point`|`u64`|The virtual address of the entry point. (The default value is 0)|
|`architecture`|`String`|The architecture of the file, the possible values are listed in [Header.architecture](#headerarchitecture).|
|`sections`|`Vec<Section>`|The sections of the file, the vector is a lua vector. The Section type is explained at [Section](#section). (The default value is an empty vector.)|
|`text_section`|`Option<Section>`|The text section of the file. `nil` if not present. The Section type is explained at [Section](#section). (The default value is `nil`)|
|`symbols`|`Vec<String>`|The symbols of the file. (The default value is an empty vector.)|

And the following functions:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`symbol_to_address`|`(symbol: String) -> Option<u64>`|Gets the virtual address of a symbol. `nil` if no such symbol is found.|
|`virtual_to_physical_address`|`(virtual_address: u64) -> Option<u64>`|Gets the file offset of a virtual address. `nil` if no section contains the virtual address specified.|

#### Header.architecture

The following values are possible for the `architecture` field:

- `Unknown` (default value)
- `Aarch64`
- `Aarch64_Ilp32`
- `Arm`
- `Avr`
- `Bpf`
- `Csky`
- `I386`
- `X86_64`
- `X86_64_X32`
- `Hexagon`
- `LoongArch64`
- `Mips`
- `Mips64`
- `Msp430`
- `PowerPc`
- `PowerPc64`
- `Riscv32`
- `Riscv64`
- `S390x`
- `Sbf`
- `Sharc`
- `Sparc64`
- `Wasm32`
- `Wasm64`
- `Xtensa`

### Section

This type has the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`name`|`String`|The name of the section.|
|`virtual_address`|`u64`|The starting virtual address of the section.|
|`file_offset`|`u64`|The starting file offset of the section.|
|`size`|`usize`|The size of the section.|

### Style

This table contains the following fields:
| Field | Type | Description |
|-------|------|-------------|
|`fg`|`Option<Color>`|The foreground color. `nil` if transparent. The Color type is explained at [Color](#color).|
|`bg`|`Option<Color>`|The background color. `nil` if transparent. The Color type is explained at [Color](#color).|
|`underline`|`Option<Color>`|The underline color. `nil` if transparent. The Color type is explained at [Color](#color).|
|`add_modifier`|`u16`|A bitflag of the modifiers to add. The possible values are listed in [Style.modifier](#stylemodifier).|
|`sub_modifier`|`u16`|A bitflag of the modifiers to remove. The possible values are listed in [Style.modifier](#stylemodifier).|

### Style.modifier

The bitflags work as follows:
| Attribute | Bitflag |
|-----------|---------|
|`BOLD`|0b0000_0000_0001|
|`DIM`|0b0000_0000_0010|
|`ITALIC`|0b0000_0000_0100|
|`UNDERLINED`|0b0000_0000_1000|
|`SLOW_BLINK`|0b0000_0001_0000|
|`RAPID_BLINK`|0b0000_0010_0000|
|`REVERSED`|0b0000_0100_0000|
|`HIDDEN`|0b0000_1000_0000|
|`CROSSED_OUT`|0b0001_0000_0000|

### Color

This type is serialized and deserialized as a string.
The following values are possible:

- Standard colors:
  - `Black`
  - `Red`
  - `Green`
  - `Yellow`
  - `Blue`
  - `Magenta`
  - `Cyan`
  - `Gray` this is sometimes called `silver` or `white`, in this case `White` means bright white.
  - `DarkGray` this is sometimes called `light black` or `bright black`, in this case we use `DarkGray`
  - `LightRed`
  - `LightGreen`
  - `LightYellow`
  - `LightBlue`
  - `LightMagenta`
  - `LightCyan`
  - `White`
- Indexed 8-bit colors:
  - `#I` where I is an index from 0 to 255.
- RBG colors:
  - `#RRGGBB` where RR, GG and BB are hexadecimal values from 00 to FF.

### CustomSetting

A custom setting can be one of the following types:

- `bool`
- `i64`
- `f64`
- `String`
- `Style`
- `KeyEvent`

In the case of a `Style` or `KeyEvent`, the value is effectively a table with the same fields as the type.
