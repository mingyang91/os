#![no_std]
#![no_main]

#![feature(global_asm)]
#![feature(asm)]
#![feature(llvm_asm)]


use core::panic::PanicInfo;

use crate::sbi::{console_putchar, shutdown};

mod sbi;

global_asm!(include_str!("boot/entry64.asm"));


pub fn console_putstr(s: &str) {
    for ch in s.bytes() {
        console_putchar(ch as usize);
    }
}

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn rust_main() -> ! {
    // this function is the entry point, since the linker looks for a function named `_start` by default
    loop {
        console_putstr("Hello World!\n");
    }
}