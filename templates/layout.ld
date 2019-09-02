MEMORY
{
    FLASH (rx) : ORIGIN = {{addr config.memory.flash.origin}}, LENGTH = {{size config.memory.flash.size}}
    RAM (wx) : ORIGIN = {{addr config.memory.ram.origin}}, LENGTH = {{size config.memory.ram.size}}
}

ENTRY(reset);

SECTIONS
{
    DRONE_STACK_POINTER = DEFINED(_section_size_unknown)
        ? ORIGIN(RAM)
        : ORIGIN(RAM) + LENGTH(RAM) - _bss_section_size - _data_section_size - _heap_section_size;

    .vtable ORIGIN(FLASH) :
    {
        LONG(DRONE_STACK_POINTER);
        KEEP(*(.rodata.VTABLE));
        . = ALIGN(4);
    } > FLASH

    .text :
    {
        *(.text.*);
        . = ALIGN(4);
    } > FLASH

    .rodata :
    {
        *(.rodata.*);
        . = ALIGN(4);
    } > FLASH

    .bss DRONE_STACK_POINTER :
    {
        DRONE_BSS_START = .;
        *(.bss.*);
        . = ALIGN(4);
        DRONE_BSS_END = .;
    } > RAM

    .data DRONE_BSS_END :
    {
        DRONE_DATA_CONST = LOADADDR(.data);
        DRONE_DATA_START = .;
        *(.data.*);
        . = ALIGN(4);
        DRONE_DATA_END = .;
    } > RAM AT > FLASH

    .heap DRONE_DATA_END :
    {
        DRONE_HEAP_START = .;
        . += {{size config.heap.size}};
        . = ALIGN(4);
        DRONE_HEAP_END = .;
    } > RAM

    /DISCARD/ :
    {
        *(.ARM.*)
    }
}
