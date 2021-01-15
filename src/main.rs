#![no_std]
#![no_main]


#![feature(global_asm)]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod sbi;
mod lang_items;
mod interrupt;


global_asm!(include_str!("boot/entry64.asm"));


#[no_mangle] // don't mangle the name of this function
pub extern "C" fn rust_main() -> ! {
    println!("Hello World!");
    reset_handler();

    interrupt::init();

    unsafe { llvm_asm!("ebreak"::::"volatile"); }

    loop {}
}


fn reset_handler() {
    extern "C" {
        fn sbss() -> usize;
        fn ebss() -> usize;
    }
    unsafe {
        (sbss()..ebss()).for_each(|a| {
            (a as *mut u8).write_volatile(0)
        });
    }

}