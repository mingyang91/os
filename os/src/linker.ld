OUTPUT_ARCH(riscv)
ENTRY(_start)
MEMORY {
    DRAM : ORIGIN = 0xffffffff80200000, LENGTH = 64M
}
SECTIONS {
    /* Boot section: startup code and data needed before enabling virtual memory */
    .text.entry : ALIGN(4K) {
        _start = .;                            /* Entry point */
        KEEP(*(.text.entry))                   /* Startup code */
        *(.rodata.entry)                       /* Read-only data needed at boot */
        *(.data.entry)                         /* Initialized data needed at boot */
    } > DRAM

    /* Boot stack in low physical memory */
    .bss.entry (NOLOAD) : ALIGN(4K) {
        *(.bss.entry)                          /* Uninitialized data for boot */
        BOOT_STACK_TOP = .;
    } > DRAM

    .pte.entry (NOLOAD) : ALIGN(4K) {
        *(.pte.entry)                          /* Uninitialized data for boot */
    } > DRAM

    .text : {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        etext = .;
    } > DRAM
    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        erodata = .;
    } > DRAM
    .data : {
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        edata = .;
    } > DRAM
    .bss (NOLOAD) : {
        *(.bss.uninit)
        . = ALIGN(8);
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(8);
        ebss = .;
    } > DRAM
    /DISCARD/ : {
        *(.eh_frame)
    }
}