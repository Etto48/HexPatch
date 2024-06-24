function init(context)
    context.add_command("p2c1", "Plugin 2 Command 1")
    context.add_command("p2c2", "Plugin 2 Command 2")
end

function p2c1(context)
    context.log(1, "Plugin 2 Command 1 called")
end

function p2c2(context)
    context.log(1, "Plugin 2 Command 2 called")
end