SECTIONS {
    .vectors :
    {
        LONG(STACK_POINTER_CORE0);
        KEEP(*(.vectors.VECTORS0));
        . = ALIGN(4);
    } > FLASH_PROGRAM
} INSERT BEFORE .text;
