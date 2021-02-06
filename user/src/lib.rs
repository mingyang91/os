#![no_std]
#![feature(llvm_asm)]
#![feature(linkage)]
#![feature(panic_info_message)]


#[macro_use]
pub mod console;
mod syscall;
mod lang_items;


use core::ptr::write_volatile;
use core::mem::zeroed;


#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    unreachable!("Unreachable after sys_exit!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    unsafe {
        for addr in start_bss as usize..end_bss as usize {
            write_volatile(addr as *mut usize, zeroed());
        }
    }

}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}



use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> isize { sys_exit(exit_code) }