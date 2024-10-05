//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![feature(naked_functions)]
#![deny(missing_docs)]
// #![deny(warnings)]
#![no_std]
#![no_main]

use core::{arch::asm, sync::atomic::{AtomicBool, Ordering}};
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;

#[path = "boards/qemu.rs"]
mod board;

const KERNEL_HEAP_SIZE: usize = 128 * 1024; // 128KiB
#[link_section = ".bss.uninit"]
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
#[global_allocator]
static KERNEL_HEAP: LockedHeap<32> = LockedHeap::empty();

use buddy_system_allocator::LockedHeap;

/// 内核入口。
///
/// # Safety
///
/// 裸函数。
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start(hartid: usize, device_tree_paddr: usize) -> ! {
    asm!(
        "la sp, {stack} + {stack_size}",
        "j  {main}",
        stack_size = const KERNEL_HEAP_SIZE,
        stack      =   sym HEAP_SPACE,
        main       =   sym rust_main,
        options(noreturn),
    )
}

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
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE)
    }
}


macro_rules! addr {
    ($a:expr) => {
        &$a as *const _ as usize
    };
}

unsafe fn print_sections_range() {
    extern "C" {
        static stext: u8;    // Start of .text section
        static etext: u8;    // End of .text section
        static srodata: u8;  // Start of .rodata section
        static erodata: u8;  // End of .rodata section
        static sdata: u8;    // Start of .data section
        static edata: u8;    // End of .data section
        static sbss: u8;     // Start of .bss section
        static ebss: u8;     // End of .bss section
    }

    debug!("[kernel] .text   [{:#20x}, {:#20x})", addr!(stext), addr!(etext));
    debug!(
        "[kernel] .rodata [{:#20x}, {:#20x})",
        addr!(srodata), addr!(erodata)
    );
    debug!("[kernel] .data   [{:#20x}, {:#20x})", addr!(sdata), addr!(edata));
    debug!("[kernel] .bss    [{:#20x}, {:#20x})", addr!(sbss), addr!(ebss));
}

/// the rust entry-point of os
extern "C" fn rust_main(hartid: usize, dtb_pa: usize) -> ! {
    init_bss();
    init_heap();
    logging::init();
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

    unsafe { print_sections_range() };
    use crate::board::QEMUExit;
    crate::board::QEMU_EXIT_HANDLE.exit_success(); // CI autotest success
                                                   //crate::board::QEMU_EXIT_HANDLE.exit_failure(); // CI autoest failed
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
