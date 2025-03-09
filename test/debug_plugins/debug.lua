mouse_ctl = false
function init(context)
    context.add_command("debug", "Open debug popup")
    context.add_command("mousectl", "Toggle mouse controlled UI")
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
    local modifiers = ""
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
    if key_event.code == "Esc" then
        pcall(function()context.close_popup("fill_popup")end)
    end

    if context.get_popup() == "fill_popup" and key_event.code == "Enter" then
        last_time_enter_pressed = context.get_instant_now()
    end
end

function on_mouse(mouse_event, context)
    local location = "nil"
    if mouse_event.location ~= nil then
        location = mouse_event.location.info.type
        if mouse_ctl
            and mouse_event.kind == "Down(Left)"
        then
            if mouse_event.location.info.type == "HexView" 
            then
                context.set_selected_pane("hex")
            elseif mouse_event.location.info.type == "TextView"
                or mouse_event.location.info.type == "AssemblyView" 
            then
                context.set_selected_pane("view")
            end
            if mouse_event.location.info.file_address ~= nil 
            then
                context.jump_to(mouse_event.location.info.file_address)
            end
        end
    end
    context.log(1, "Mouse event: " .. mouse_event.kind .. " on " .. location .. " @ " .. mouse_event.row .. "," .. mouse_event.column)
end

function on_focus(context)
    context.log(1, "Focus gained")
end

function on_blur(context)
    context.log(1, "Focus lost")
end

function on_paste(text, context)
    context.log(1, "Text pasted: " .. text)
end

function on_resize(width, height, context)
    context.log(1, "Resized: " .. width .. "x" .. height .. " from " .. context.screen_width .. "x" .. context.screen_height)
end

function debug(context)
    context.open_popup("fill_popup")
end

function mousectl(context)
    mouse_ctl = not mouse_ctl
end

last_time_enter_pressed = nil
function fill_popup(popup_context, context)
    popup_context.title:set("Debug")
    popup_context.height:set(7)
    popup_context.text:push_line("Debugging information")
    popup_context.text:set_style({fg="green"})
    popup_context.text:set_alignment("left")
    popup_context.text:push_line("Time from last enter: ")
    popup_context.text:reset_style()
    if last_time_enter_pressed == nil then
        popup_context.text:push_span("Never")
    else
        popup_context.text:push_span(last_time_enter_pressed:elapsed())
    end
    popup_context.text:set_style({fg="green"})
    popup_context.text:push_line("Popup size: ")
    popup_context.text:reset_style()
    popup_context.text:push_span(popup_context.width:get() .. "x" .. popup_context.height:get())
    popup_context.text:set_style({fg="green"})
    popup_context.text:push_line("Screen size: ")
    popup_context.text:reset_style()
    popup_context.text:push_span(context.screen_width .. "x" .. context.screen_height)
    popup_context.text:reset_alignment()
    if last_time_enter_pressed == nil or last_time_enter_pressed:elapsed() > 1 then
        popup_context.text:push_line("Press Enter!")
    else
        popup_context.text:push_line("Enter Pressed!")
    end
end
