function init(context)
    context.add_command("debug", "Open debug popup")
end

function on_open(context)
    context.log(1, "Data loaded: " .. context.data.len .. "B")
end

function on_save(context)
    context.log(1, "Data saved: " .. context.data.len .. "B")
end

function on_edit(new_bytes, context)
    context.log(1, "Data edited: @" .. context.offset)
end

function on_key(key_event, context)
    modifiers = ""
    if key_event.modifiers.shift then
        modifiers = modifiers .. "+Shift"
    end
    if key_event.modifiers.ctrl then
        modifiers = modifiers .. "+Ctrl"
    end
    if key_event.modifiers.alt then
        modifiers = modifiers .. "+Alt"
    end
    if key_event.modifiers.meta then
        modifiers = modifiers .. "+Meta"
    end
    if key_event.modifiers.super then
        modifiers = modifiers .. "+Super"
    end
    if key_event.modifiers.hyper then
        modifiers = modifiers .. "+Hyper"
    end
    context.log(1, "Key event: " .. key_event.code .. modifiers .. "@" .. context.offset)
end

function on_mouse(mouse_event, context)
    context.log(1, "Mouse event: " .. mouse_event.kind .. "@" .. x .. "," .. y)
end

function debug(context)
    context.open_popup("fill_popup")
end

fill_popup_calls = 0
function fill_popup(popup_text, popup_title, context)
    popup_title:set("Debug")
    popup_text:push_line("Debugging information")
    popup_text:push_line("Calls: " .. fill_popup_calls)
    fill_popup_calls = fill_popup_calls + 1
end
