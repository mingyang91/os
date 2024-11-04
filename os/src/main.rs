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
#![feature(alloc_error_handler)]
#![deny(missing_docs)]
#![no_std]
#![no_main]

use core::{
    arch::asm,
    mem,
    ptr::addr_of,
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};
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

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

use buddy_system_allocator::LockedHeap;
use mm::{allocator::FRAME_ALLOCATOR, Address, AlignSize, RootPageTable, Sv39};

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

    set_boot_hart(hartid);
    init_page_table(hartid);

    if is_boot_hart(hartid) {
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
    } else {
        _hart_start(hartid, device_tree_paddr);
    }
}

unsafe extern "C" fn _hart_start(hartid: usize, stack_top: usize) -> ! {
    asm!(
        "mv sp, {stack_top}",
        "mv a0, {hartid}",
        "mv a1, {stack_top}",
        "call {main}",
        stack_top = in(reg) stack_top,
        hartid = in(reg) hartid,
        main = sym rust_main,
        options(noreturn),
    )
}

/// virtual address of the kernel
const KERNEL_VIRT_BASE: usize = 0xffffffff80000000;
const VIRT_ADDR: Address = Address::new(KERNEL_VIRT_BASE);
const KERNEL_PHYS_BASE: usize = 0x80000000;
const PHY_ADDR: Address = Address::new(KERNEL_PHYS_BASE);
const KERNEL_START: usize = 0x80200000;

#[inline(always)]
fn init_page_table(hartid: usize) {
    unsafe {
        if is_boot_hart(hartid) {
            let _ =
                ROOT_PAGE_TABLE.map(PHY_ADDR, PHY_ADDR, AlignSize::Page1G, mm::KERNEL_PTE_FLAGS);
            let _ =
                ROOT_PAGE_TABLE.map(VIRT_ADDR, PHY_ADDR, AlignSize::Page1G, mm::KERNEL_PTE_FLAGS);
        }
        ROOT_PAGE_TABLE.active(0);
    }
}

#[no_mangle]
#[link_section = ".pte.entry"]
static ROOT_PAGE_TABLE: RootPageTable<Sv39> = RootPageTable::zero();

static BOOT_HART: AtomicUsize = AtomicUsize::new(usize::MAX);

fn set_boot_hart(hartid: usize) {
    let _ = BOOT_HART.compare_exchange(usize::MAX, hartid, Ordering::AcqRel, Ordering::Acquire);
}

fn is_boot_hart(hartid: usize) -> bool {
    BOOT_HART.load(Ordering::Acquire) == hartid
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
    if is_boot_hart(hartid) {
        logging::init();
        print_sections_range();
        init_bss();
        init_heap();
        let BoardInfo {
            smp,
            frequency,
            uart: _uart,
            memory,
            memory_count,
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

        for (i, (start, end)) in memory.iter().take(memory_count).enumerate() {
            info!(
                r"memory region {i:10} [{start:#20x}, {end:#20x})",
                i = i,
                start = start,
                end = end
            );
            let kernel_end = KERNEL_START + 0x4000000;
            FRAME_ALLOCATOR.init(kernel_end, *end - kernel_end);
        }

        for i in 0..smp {
            let frame = FRAME_ALLOCATOR
                .alloc::<Sv39>(0x800000)
                .expect("failed to allocate stack frame");
            if i != hartid {
                let hart_stack = frame.ptr.as_ptr() as usize;
                sbi_rt::hart_start(i, KERNEL_START, hart_stack);
            }
            mem::forget(frame);
        }

        for _ in 0..100_000_000 {
            unsafe {
                asm!("nop");
            }
        }

        sbi::shutdown();
    } else {
        info!("hart {} started at {:#20x}", hartid, dtb_pa);
        loop {
            for _ in 0..10_000_000 {
                unsafe {
                    asm!("nop");
                }
            }
            info!("hart {} running", hartid);
        }
    }
}

struct BoardInfo {
    smp: usize,
    frequency: u64,
    uart: usize,
    memory: [(usize, usize); 8],
    memory_count: usize,
}

impl BoardInfo {
    fn parse(dtb_pa: usize) -> Self {
        use dtb_walker::{Dtb, DtbObj, HeaderError as E, Property, WalkOperation::*};

        let mut ans = Self {
            smp: 0,
            frequency: 0,
            uart: 0,
            memory: [(0, 0); 8],
            memory_count: 0,
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
                } else if name.starts_with(b"memory@") {
                    StepInto
                } else {
                    StepOver
                }
            }
            DtbObj::Property(Property::Reg(mut reg)) => {
                if ctx.last().starts_with(b"uart") || ctx.last().starts_with(b"serial") {
                    ans.uart = reg.next().unwrap().start;
                } else if ctx.last().starts_with(b"memory") {
                    while let Some(r) = reg.next() {
                        if ans.memory_count >= ans.memory.len() {
                            break;
                        }
                        info!("memory region: {:#x} - {:#x}", r.start, r.end);
                        ans.memory[ans.memory_count] = (r.start, r.end);
                        ans.memory_count += 1;
                    }
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
