# HexPatch Plugin API

This document describes the API that plugins can use to interact with HexPatch.

## Functions

### Initialization

```lua
function init(settings, context) end
```

### Events

```lua
function on_open(data, offset, current_instruction, settings, header, context) end
```

```lua
function on_save(data, offset, current_instruction, settings, header, context) end
```

```lua
function on_edit(new_bytes, data, offset, current_instruction, settings, header, context) end
```

```lua
function on_key(key_event, data, offset, current_instruction, settings, header, context) end
```

```lua
function on_mouse(mouse_event, data, offset, current_instruction, settings, header, context) end
```

### Commands

```lua
function COMMAND_NAME(data, offset, current_instruction, settings, header, context) end
```

### Popups

```lua
function FILL_POPUP_NAME(popup_text, popup_title, data, offset, current_instruction, settings, header, context) end
```
