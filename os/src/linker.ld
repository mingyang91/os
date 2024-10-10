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
        KEEP(*(.rodata.entry))                 /* Read-only data needed at boot */
        KEEP(*(.data.entry))                   /* Initialized data needed at boot */
    } > DRAM

    /* Boot stack in low physical memory */
    .bss.entry (NOLOAD) : ALIGN(4K) {
        *(.bss.entry)                          /* Uninitialized data for boot */
        BOOT_STACK_TOP = .;
    } > DRAM

    .pte.entry (NOLOAD) : ALIGN(4K) {
        *(.pte.entry)                          /* Uninitialized data for boot */
    } > DRAM

    .text : ALIGN(4K) {
        stext = .;
        *(.text)
        *(.text.*)
        etext = .;
    } > DRAM
    .rodata : ALIGN(4k) {
        srodata = .;
        *(.rodata)
        *(.rodata.*)
        erodata = .;
    } > DRAM
    .data : ALIGN(4k) {
        sdata = .;
        *(.data)
        *(.data.*)
        edata = .;
    } > DRAM
    .bss (NOLOAD) : ALIGN(4k) {
        sbss = .;
        *(.bss.uninit)
        *(.bss)
        *(.bss.*)
        ebss = .;
    } > DRAM
    /DISCARD/ : {
        *(.eh_frame)
    }
}