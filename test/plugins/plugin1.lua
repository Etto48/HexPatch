function init(context)
    context.add_command("p1c1", "Plugin 1 Command 1")
    context.add_command("p1c2", "Plugin 1 Command 2")
end

function p1c1(context)
    context.log(1, "Plugin 1 Command 1 called")
end

function p1c2(context)
    context.log(1, "Plugin 1 Command 2 called")
end

function on_open(context)
    context.log(1, "Plugin 1 on_open called")
end