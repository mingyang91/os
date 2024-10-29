//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![feature(naked_functions)]
#![feature(const_trait_impl)]
#![deny(missing_docs)]
#![no_std]
#![no_main]

use core::{arch::asm, ptr::addr_of};
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod mm;
mod sbi;

#[path = "boards/qemu.rs"]
mod board;

#[allow(dead_code)]
#[link_section = ".bss.entry"]
static mut BOOT_STACK: [u8; 4096] = [0; 4096];

const KERNEL_HEAP_SIZE: usize = 128 * 1024; // 128KiB
#[link_section = ".bss.uninit"]
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
#[global_allocator]
static KERNEL_HEAP: LockedHeap<32> = LockedHeap::empty();

use buddy_system_allocator::LockedHeap;
use mm::{Address, AlignSize, RootPageTable, RV39};

/// 内核入口。
///
/// # Safety
///
/// 裸函数。
// #[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start(hartid: usize, device_tree_paddr: usize) -> ! {
    extern "C" {
        static BOOT_STACK_TOP: usize;
    }

    // MAGIC: don't modify
    asm!(
        "lui sp, %hi({BOOT_STACK_TOP})",
        "slli sp, sp, 32",
        "srli sp, sp, 32",
        BOOT_STACK_TOP = sym BOOT_STACK_TOP,
    );

    // wrong version
    // asm!(
    //     "lui t0, %hi({BOOT_STACK_TOP})",
    //     "slli t0, t0, 32",
    //     "srli t0, t0, 32",
    //     "add sp, sp, t0",
    //     BOOT_STACK_TOP = sym BOOT_STACK_TOP,
    // );

    init_page_table();
    asm!(
        "lui sp, %hi({BOOT_STACK_TOP})",
        "lui t0, %hi({main})",
        "addi t0, t0, %lo({main})",
        "mv a0, {hartid}",
        "mv a1, {device_tree_paddr}",
        "jr t0",
        BOOT_STACK_TOP = sym BOOT_STACK_TOP,
        hartid = in(reg) hartid,
        device_tree_paddr = in(reg) device_tree_paddr,
        main = sym rust_main,
        options(noreturn),
    );
}

/// virtual address of the kernel
const KERNEL_VIRT_BASE: usize = 0xffffffff80000000;
const VIRT_ADDR: Address = Address::new(KERNEL_VIRT_BASE);
const KERNEL_PHYS_BASE: usize = 0x80000000;
const PHY_ADDR: Address = Address::new(KERNEL_PHYS_BASE);

#[inline(always)]
fn init_page_table() {
    unsafe {
        let _ = ROOT_PAGE_TABLE.map(PHY_ADDR, PHY_ADDR, AlignSize::Page1G, mm::KERNEL_PTE_FLAGS);
        let _ = ROOT_PAGE_TABLE.map(VIRT_ADDR, PHY_ADDR, AlignSize::Page1G, mm::KERNEL_PTE_FLAGS);
        ROOT_PAGE_TABLE.active(0);
    }
}

#[no_mangle]
#[link_section = ".pte.entry"]
static mut ROOT_PAGE_TABLE: RootPageTable<RV39> = RootPageTable::zero();

fn init_bss() {
    extern "C" {
        static sbss: u8;
        static ebss: u8;
    }
    unsafe {
        let sbss_addr = &sbss as *const u8 as usize;
        let ebss_addr = &ebss as *const u8 as usize;
        let bss_size = ebss_addr - sbss_addr;
        let bss_ptr = sbss_addr as *mut u8;
        core::slice::from_raw_parts_mut(bss_ptr, bss_size).fill(0);
    }
}

fn init_heap() {
    unsafe {
        KERNEL_HEAP
            .lock()
            .init(&raw const HEAP_SPACE as usize, KERNEL_HEAP_SIZE)
    }
}

macro_rules! addr_of {
    ($symbol:ident) => {
        (&unsafe { $symbol } as *const _) as usize
    };
}

macro_rules! print_section_range {
    ($section_name:expr, $start:ident, $end:ident) => {
        debug!(
            "[kernel] {} [{:#20x}, {:#20x})",
            $section_name,
            addr_of!($start),
            addr_of!($end)
        );
    };
}

fn print_sections_range() {
    extern "C" {
        static stext: u8; // Start of .text section
        static etext: u8; // End of .text section
        static srodata: u8; // Start of .rodata section
        static erodata: u8; // End of .rodata section
        static sdata: u8; // Start of .data section
        static edata: u8; // End of .data section
        static sbss: u8; // Start of .bss section
        static ebss: u8; // End of .bss section
    }

    print_section_range!(".text   ", stext, etext);
    print_section_range!(".rodata ", srodata, erodata);
    print_section_range!(".data   ", sdata, edata);
    print_section_range!(".bss    ", sbss, ebss);
}

/// the rust entry-point of os
#[no_mangle]
extern "C" fn rust_main(hartid: usize, dtb_pa: usize) -> ! {
    logging::init();
    print_sections_range();
    init_bss();
    init_heap();
    let BoardInfo {
        smp,
        frequency,
        uart: _uart,
    } = BoardInfo::parse(dtb_pa);

    info!(
        r"
  __  __         _  __                    _ 
 |  \/  |       | |/ /                   | |
 | \  / |_   _  | ' / ___ _ __ _ __   ___| |
 | |\/| | | | | |  < / _ \ '__| '_ \ / _ \ |
 | |  | | |_| | | . \  __/ |  | | | |  __/ |
 |_|  |_|\__, | |_|\_\___|_|  |_| |_|\___|_|
          __/ |                             
         |___/                              
================================================
| boot hart id          | {hartid:20} |
| smp                   | {smp:20} |
| timebase frequency    | {frequency:17} Hz |
| dtb physical address  | {dtb_pa:#20x} |
------------------------------------------------"
    );

    sbi::shutdown();
}

struct BoardInfo {
    smp: usize,
    frequency: u64,
    uart: usize,
}

impl BoardInfo {
    fn parse(dtb_pa: usize) -> Self {
        use dtb_walker::{Dtb, DtbObj, HeaderError as E, Property, WalkOperation::*};

        let mut ans = Self {
            smp: 0,
            frequency: 0,
            uart: 0,
        };
        unsafe {
            Dtb::from_raw_parts_filtered(dtb_pa as _, |e| {
                matches!(e, E::Misaligned(4) | E::LastCompVersion(_))
            })
        }
        .unwrap()
        .walk(|ctx, obj| match obj {
            DtbObj::SubNode { name } => {
                if ctx.level() == 0 && (name == b"cpus" || name == b"soc") {
                    StepInto
                } else if ctx.last() == b"cpus" && name.starts_with(b"cpu@") {
                    ans.smp += 1;
                    StepOver
                } else if ctx.last() == b"soc"
                    && (name.starts_with(b"uart") || name.starts_with(b"serial"))
                {
                    StepInto
                } else {
                    StepOver
                }
            }
            DtbObj::Property(Property::Reg(mut reg)) => {
                if ctx.last().starts_with(b"uart") || ctx.last().starts_with(b"serial") {
                    ans.uart = reg.next().unwrap().start;
                }
                StepOut
            }
            DtbObj::Property(Property::General { name, value }) => {
                if ctx.last() == b"cpus" && name.as_bytes() == b"timebase-frequency" {
                    ans.frequency = match *value {
                        [a, b, c, d] => u32::from_be_bytes([a, b, c, d]) as _,
                        [a, b, c, d, e, f, g, h] => u64::from_be_bytes([a, b, c, d, e, f, g, h]),
                        _ => unreachable!(),
                    };
                }
                StepOver
            }
            DtbObj::Property(_) => StepOver,
        });
        ans
    }
}
