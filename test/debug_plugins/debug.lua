function on_open(data, context, header)
    context:log(1, "Data loaded: " .. data.len .. "B")
end

function on_save(data, context, header)
    context:log(1, "Data saved: " .. data.len .. "B")
end

function on_edit(data, offset, new_bytes, current_instruction, context, header)
    context:log(1, "Data edited: @" .. offset)
end

function on_key(key_event, data, offset, current_instruction, context, header)
    context:log(1, "Key event: " .. key_event.code .. "+" .. key_event.modifiers .. "@" .. offset)
end

function on_mouse(event_kind, x, y, context, header)
    context:log(1, "Mouse event: " .. event_kind .. "@" .. x .. "," .. y)
end