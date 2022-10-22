SECTIONS {
    .vtable :
    {
        LONG(STACK_POINTER_CORE0);
        KEEP(*(.rodata.VTABLE));
        . = ALIGN(4);
    } > FLASH_PROGRAM
} INSERT BEFORE .text;
