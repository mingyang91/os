#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]

use core::ptr::write_volatile;
use core::mem::zeroed;


#[macro_use]
mod console;
mod sbi;
mod lang_items;
mod trap;
mod batch;
mod syscall;


global_asm!(include_str!("boot/entry64.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn rust_main() -> ! {
    reset_handler();

    println!("[kernel] Hello, world!");

    trap::init();
    batch::init();
    batch::run_next_app();

}


fn reset_handler() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        for ptr in sbss as usize..ebss as usize {
            write_volatile(ptr as *mut u8, zeroed());
        }
    }

}