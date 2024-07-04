function init(context)
    context.add_header_parser("test")
end

function test(header_context, context)
    if context.data:get(0) == 0x43 and 
        context.data:get(1) == 0x55 and
        context.data:get(2) == 0x53 and
        context.data:get(3) == 0x54 and
        context.data:get(4) == 0x4f and
        context.data:get(5) == 0x4d and
        context.data:get(6) == 0x00 then
        header_context:set_endianness("little")
        if context.data:get(7) == 0x32 then
            header_context:set_architecture("X86_64_X32")
            header_context:set_bitness(32)
        elseif context.data:get(7) == 0x64 then
            header_context:set_architecture("X86_64")
            header_context:set_bitness(64)
        else
            error("Unknown architecture")
        end
        
        entry = context.data:get(8) + context.data:get(9) * 0x100 
            + context.data:get(10) * 0x10000 + context.data:get(11) * 0x1000000
        header_context:set_entry(entry)
        text_start = context.data:get(12) + context.data:get(13) * 0x100 
            + context.data:get(14) * 0x10000 + context.data:get(15) * 0x1000000
        text_size = context.data:get(16) + context.data:get(17) * 0x100 
            + context.data:get(18) * 0x10000 + context.data:get(19) * 0x1000000
        header_context:add_section(".text", text_start, text_start, text_size)
        header_context:add_symbol(entry, "_start")
    end
end
