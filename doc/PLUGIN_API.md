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
|`new_bytes`|`Vec<u8>`|The new bytes of the file.|
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
|`data`|`Vec<u8>`|The current file's data.|
|`offset`|`usize`|The current offset in the file.|
|`current_instruction`|`Option<InstructionInfo>`|The current instruction at the current offset.|
|`header`|`Header`|The header of the file, if not valid or present, the default header will be used.|

And the following functions:
| Function | Arguments | Description |
|----------|-----------|-------------|
|`log`|`(level: u8, message: String)`|Logs a message in the UI.|
|`add_command`|`(command_name: String)`|Registers a command, this must be called to make the command appear in the command list.|
|`remove_command`|`(command_name: String)`|Removes a command, this removes the command from the command list.|
|`open_popup`|`(popup_handler: String)`|Opens a popup, each time the popup is drawn the handler function is called|

For more information on the types, see the following sections.

### Vec\<u8\>

A vector of bytes.

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

### MutString
