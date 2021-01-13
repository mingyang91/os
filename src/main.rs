#![no_std]
#![no_main]


#![feature(global_asm)]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]


mod sbi;
mod lang_items;
mod console;


global_asm!(include_str!("boot/entry64.asm"));


#[no_mangle] // don't mangle the name of this function
pub extern "C" fn rust_main() -> ! {
    reset_handler();
    println!("Hello World!");

    panic!("Shutdown machine!");
}


fn reset_handler() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        (sbss as usize..ebss as usize).for_each(|a| {
            (a as *mut u8).write_volatile(0)
        });
    }

}