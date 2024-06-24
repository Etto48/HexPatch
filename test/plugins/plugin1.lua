function init(settings, context)
    context:add_command("p1c1", "Plugin 1 Command 1")
    context:add_command("p1c2", "Plugin 1 Command 2")
end

function p1c1(data, offset, current_instruction, settings, header, context)
    context:log(1, "Plugin 1 Command 1 called")
end

function p1c2(data, offset, current_instruction, settings, header, context)
    context:log(1, "Plugin 1 Command 2 called")
end

function on_open(data, offset, current_instruction, settings, header, context)
    context:log(1, "Plugin 1 on_open called")
end