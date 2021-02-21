#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]
#![feature(inline_const)]

#[macro_use]
mod console;
mod sbi;
mod lang_items;
mod trap;
mod syscall;
mod loader;
mod task;
mod config;


global_asm!(include_str!("boot/entry64.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle] // don't mangle the name of this function
pub fn rust_main() -> ! {
    clear_bss();

    println!("[kernel] Hello, world!");

    trap::init();
    loader::load_apps();
    task::run_first_task();

    panic!("Unreachable in rust_main");
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
