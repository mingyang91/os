#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]

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
pub fn rust_main() -> ! {
    clear_bss();

    println!("[kernel] Hello, world!");

    trap::init();
    batch::init();
    batch::run_next_app();
}


fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
