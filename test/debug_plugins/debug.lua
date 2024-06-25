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
    context.log(1, "Mouse event: " .. mouse_event.kind .. "@" .. mouse_event.row .. "," .. mouse_event.column)
end

function debug(context)
    context.open_popup("fill_popup")
end

fill_popup_calls = 0
function fill_popup(popup_context, context)
    popup_context.title:set("Debug")
    popup_context.height:set(6)
    popup_context.text:push_line("Debugging information")
    popup_context.text:push_line("Calls: " .. fill_popup_calls)
    popup_context.text:push_line("Popup size: " .. popup_context.width:get() .. "x" .. popup_context.height:get())
    popup_context.text:push_line("Screen size: " .. context.screen_width .. "x" .. context.screen_height)
    fill_popup_calls = fill_popup_calls + 1
end
