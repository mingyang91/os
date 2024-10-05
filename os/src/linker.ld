OUTPUT_ARCH(riscv)
ENTRY(_start)
MEMORY {
    DRAM : ORIGIN = 0x80200000, LENGTH = 64M
}
SECTIONS {
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