function init(settings, context)
    context:add_command("debug", "Open debug popup")
end

function on_open(data, offset, current_instruction, settings, header, context)
    context:log(1, "Data loaded: " .. data.len .. "B")
end

function on_save(data, offset, current_instruction, settings, header, context)
    context:log(1, "Data saved: " .. data.len .. "B")
end

function on_edit(new_bytes, data, offset, current_instruction, settings, header, context)
    context:log(1, "Data edited: @" .. offset)
end

function on_key(key_event, data, offset, current_instruction, settings, header, context)
    context:log(1, "Key event: " .. key_event.code .. "+" .. key_event.modifiers .. "@" .. offset)
end

function on_mouse(mouse_event, data, offset, current_instruction, settings, header, context)
    context:log(1, "Mouse event: " .. event_kind .. "@" .. x .. "," .. y)
end

function debug(data, offset, current_instruction, settings, header, context)
    context:open_popup("fill_popup")
end

fill_popup_calls = 0
function fill_popup(popup_text, popup_title, data, offset, current_instruction, settings, header, context)
    popup_title:set("Debug")
    popup_text:push_line("Debugging information")
    popup_text:push_line("Calls: " .. fill_popup_calls)
    fill_popup_calls = fill_popup_calls + 1
end
