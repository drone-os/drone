SECTIONS {
    .vectors :
    {
        LONG(STACK_POINTER_CORE0);
        KEEP(*(.vectors.VECTORS0));
        LONG(STACK_POINTER_CORE1);
        KEEP(*(.vectors.VECTORS1));
        . = ALIGN(4);
    } > FLASH_PROGRAM
} INSERT BEFORE .text;
