# Settings

Settings are stored inside `settings.json`. This file is located in the config folder of HexPatch.
The config folder is located in the following locations:

- Windows: `%APPDATA%\HexPatch`
- Linux: `$HOME/.config/HexPatch` or `$XDG_CONFIG_HOME/HexPatch`
- macOS: `$HOME/Library/Application Support/HexPatch`

You can specify a different configuration file to use with the `--config` flag.

If the file or directory doesn't exist, it will be created with the default settings.

Settings are stored in JSON format and have the following structure:

```json
{
    "color": {
        //color settings, used to define the style of the UI
    },
    "key": {
        //key settings, used to define the keybindings
    },
    "app": {
        //app settings, used to define the behavior of the app
    },
    "custom": {
        //custom settings, used to define plugin related settings in a Key-Value format
    }
}
```

If a setting is not present in the file, the default value will be used.

You can find the default settings [here](https://github.com/Etto48/hexpatch/blob/master/test/default_settings.json).
You can also generate the same file by running `hexpatch --config <CONFIG_PATH>` passing in a path that doesn't exist yet, the file will be created there.

## Color

### Style

Every setting in this categroy is a style object, a style object has the following fields:
| Field | Description |
|-------|-------------|
|fg|Foreground color, can be a string containing an RGB HEX color, a color name, a color index or can be `null` to be transparent.|
|bg|Background color, can be a string containing an RGB HEX color, a color name, a color index or can be `null` to be transparent.|
|underline_color|Underline color, can be a string containing an RGB HEX color, a color name, a color index or can be `null` to be transparent.|
|add_modifier|Additional modifier to apply to the style, it is a string containing a `\|`-separated list of modifiers to add. See [Style.Modifier](#stylemodifier).|
|sub_modifier|Subtractive modifier to apply to the style, it is a string containing a `\|`-separated list of modifiers to remove. See [Style.Modifier](#stylemodifier).|

### Style.Modifier

The following modifiers are available:

- BOLD
- DIM
- ITALIC
- UNDERLINED
- SLOW_BLINK
- RAPID_BLINK
- REVERSED
- HIDDEN
- CROSSED_OUT

### Color fields

The following styles can be customized in the color settings:
| Name | Description |
|------|-------------|
|address_selected|The selected address in the address view.|
|address_default|Any other address in the address row.|
|hex_selected|The selected half-byte in the hex view.|
|hex_null|Zero bytes in the hex and text view.|
|hex_alphanumeric|Alphanumeric bytes in the hex and text view.|
|hex_symbol|Printable symbol bytes in the hex and text view.|
|hex_end_of_line|End of line bytes in the hex and text view.|
|hex_whitespace|Whitespace bytes in the hex and text view.|
|hex_current_instruction|Bytes composing the selected instruction.|
|hex_current_section|Bytes composing the selected section.|
|hex_default|Default style for bytes in hex and text view.|
|text_selected|Selected byte in the text view.|
|assembly_symbol|Symbol in the assembly view and other related popups.|
|assembly_selected|Selected instruction in the assembly view.|
|assembly_address|File address in the assembly view.|
|assembly_virtual_address|Virtual address in the assembly view.|
|assembly_nop|`nop` mnemonic in the assembly view.|
|assembly_bad|Bad instructions in the assembly view.|
|assembly_section|Section in the assembly view.|
|assembly_entry_point|Entry point tag in the assembly view.|
|assembly_default|Default style for mnemonic in the assembly view.|
|patch_patched_less_or_equal|Bytes patched that are less or equal to the size of the original instruction in the patch popup.|
|patch_patched_greater|Bytes patched that are overflowing the size of the original instruction in the patch popup.|
|patch_old_instruction|Original instruction bytes in the patch popup.|
|patch_old_rest|Remaining bytes that are neither patched nor belonging to the old instruction in the patch popup.|
|patch_line_number|Line number in the patch popup.|
|help_command|Key combination in the help popup.|
|help_description|Command description in the help popup.|
|yes|"Yes" in popups with choiches when not selected.|
|yes_selected|"Yes" in popups with choiches when selected.|
|no|"No" in popups with choiches when not selected.|
|no_selected|"No" in popups with choiches when selected.|
|menu_text|Generic text in a menu, is also used for "Cancel" in popups with choiches.|
|menu_text_selected|Generic selected text in a menu, is also used for "Cancel" in popups with choiches when selected.|
|insert_text_status|Status bar in the insert text popup.|
|command_name|Command name in the run popup.|
|command_description|Command description in the run popup.|
|command_selected|Selected command (both name and description) in the run popup.|
|path_dir|Directory in any popup that shows the filesystem.|
|path_file|File in any popup that shows the filesystem.|
|path_selected|Selected file or directory in any popup that shows the filesystem.|
|log_info|Info level for the log.|
|log_debug|Debug level for the log.|
|log_warning|Warning level for the log.|
|log_error|Error level for the log.|
|log_message|Message content for the log.|
|status_bar|Status bar in the bottom of the screen.|
|status_info|Info level in the dot in the status bar.|
|status_debug|Debug level in the dot in the status bar.|
|status_warning|Warning level in the dot in the status bar.|
|status_error|Error level in the dot in the status bar.|
|scrollbar|Status bar on the side of the screen.|
|placeholder|Placeholder in every text input.|

## Key

### KeyEvent

Every setting in this category is a key event object, a key event object has the following fields:
| Field | Description |
|-------|-------------|
|code|The key code of the key event. See [KeyEvent.Code](#keyeventcode).|
|modifiers|A string containing a `\|`-separated list of modifiers. See [KeyEvent.Modifier](#keyeventmodifier).|
|kind|The kind of key event, must be one of `"Press"`, `"Repeat"` or `"Release"`.|
|state|The state of the keyboard, it's a string containing a `\|`-separated list of states. See [KeyEvent.State](#keyeventstate).|

### KeyEvent.Code

Key codes can be in one of the following formats:

- **Named key**: a string containing the name of one of the following keys:

  - `"Backspace"`
  - `"Enter"`
  - `"Left"`
  - `"Right"`
  - `"Up"`
  - `"Down"`
  - `"Home"`
  - `"End"`
  - `"PageUp"`
  - `"PageDown"`
  - `"Tab"`
  - `"BackTab"`
  - `"Delete"`
  - `"Insert"`
  - `"Esc"`
  - `"CapsLock"`
  - `"ScrollLock"`
  - `"NumLock"`
  - `"PrintScreen"`
  - `"Pause"`
  - `"Menu"`
  - `"KeypadBegin"`

- **Function key**: a json object containing the field `"F"` with the number of the function key i.e.

  ```json
  {"F": 1}
  ```

  Numbers from 1 to 24 are valid.

- **Character key**: a json object containing the field `"Char"` with the character of the key i.e.

  ```json
  {"Char": "a"}
  ```

  Both uppercase and lowercase characters are valid, as well as numbers and symbols. Uppercase characters should be used with the SHIFT modifier.

- **Media key**: a json object containing the field `"Media"` with the name of the media key i.e.
  
  ```json
  {"Media": "Play"}
  ```

  Available media keys are:
  
  - `"Play"`
  - `"Pause"`
  - `"PlayPause"`
  - `"Reverse"`
  - `"Stop"`
  - `"FastForward"`
  - `"Rewind"`
  - `"TrackNext"`
  - `"TrackPrevious"`
  - `"Record"`
  - `"LowerVolume"`
  - `"RaiseVolume"`
  - `"MuteVolume"`

- **Modifier key**: a json object containing the field `"Modifier"` with the name of the modifier key i.e.

  ```json
  {"Modifier": "LeftShift"}
  ```

  Available modifier keys are:

  - `"LeftShift"`
  - `"LeftControl"`
  - `"LeftAlt"`
  - `"LeftSuper"`
  - `"LeftHyper"`
  - `"LeftMeta"`
  - `"RightShift"`
  - `"RightControl"`
  - `"RightAlt"`
  - `"RightSuper"`
  - `"RightHyper"`
  - `"RightMeta"`
  - `"IsoLevel3Shift"`
  - `"IsoLevel5Shift"`

### KeyEvent.Modifier

The available modifiers are:

- SHIFT
- CONTROL
- ALT
- SUPER
- HYPER
- META

### KeyEvent.State

The available states are:

- KEYPAD (this means that the event originated from the keypad)
- CAPS_LOCK
- NUM_LOCK

### Key fields

The following key bindings can be customized in the key settings:
| Name | Description |
|------|-------------|
|up|Move the cursor up.|
|down|Move the cursor down.|
|left|Move the cursor left.|
|right|Move the cursor right.|
|next|Move the cursor to the next item (block of bytes in text mode and instruction or section in assembly mode).|
|previous|Move the cursor to the previous item (block of bytes in text mode and instruction or section in assembly mode).|
|page_up|Move the cursor up a page.|
|page_down|Move the cursor down a page.|
|goto_start|Move the cursor to the start of the file.|
|goto_end|Move the cursor to the end of the file.|
|quit|Quit the app.|
|save_and_quit|Save the file and quit the app.|
|save_as|Save the file with a new name.|
|save|Save the file.|
|open|Open a file.|
|help|Open the help popup.|
|log|Open the log popup.|
|run|Open the run popup.|
|find_text|Open the find text popup.|
|find_symbol|Open the find symbol popup.|
|patch_text|Open the patch text popup.|
|patch_assembly|Open the patch assembly popup.|
|jump|Open the jump popup.|
|change_view|Change the view mode.|
|confirm|Confirm the current action.|
|close_popup|Close the current popup.|
|new_line|Insert a new line in multiline text input.|
|clear_log|Clear the log when the log popup is open.|
|undo|Undo the last action.|
|redo|Redo the last action.|

## App

### App fields

The following app settings can be customized in the app settings:
| Name | Type | Description |
|------|------|-------------|
|history_limit|usize|Maximum number of modifications that are stored in the undo/redo history.|
|log_limit|usize|Maximum number of log messages that are stored in the log.|
|theme|Option<String>|The name of the theme to use. The available themes are: `"auto"`, `"dark"`, `"light"`. `"auto"` chooses automatically between `"dark"` and `"light"` based on the background color of the terminal. By default, the theme is `"auto"`.|

## Custom

Custom fields are used to store and configure plugin related settings in a Key-Value format.

### CustomValue

Every setting in this category is a custom value object, a custom value object can be any JSON value apart from an object or an array. It can also be a [Style](#style) object or a [KeyEvent](#keyevent) object.

Example:

```json
{
    "custom_string": "This is a string",
    "custom_integer": 42,
    "custom_bool": true,
    "custom_float": 3.14,
    "custom_style": {
        "fg": "#000000",
        "bg": "#FFFFFF",
        "underline_color": "#FFFFFF",
        "add_modifier": "BOLD",
        "sub_modifier": ""
    },
    "custom_key": {
        "code": {"Char": "a"},
        "modifiers": "CONTROL",
        "kind": "Press",
        "state": ""
    }
}
```
