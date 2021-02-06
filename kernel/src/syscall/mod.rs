use core::{slice::from_raw_parts, str::from_utf8};

use crate::batch::run_next_app;

pub const STDOUT: usize = 1;

#[repr(usize)]
pub enum SYSCALL {
    Write = 64,
    Exit = 93,
}

impl SYSCALL {
    pub fn try_from(id: usize) -> SYSCALL {
        match id {
            64 => SYSCALL::Write,
            93 => SYSCALL::Exit,
            _ => panic!("Unsupported syscall_id: {}", id),
        }
    }
}



pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let slice = unsafe { from_raw_parts(buf, len) };
            let content = from_utf8(slice).unwrap();
            print!("{}", content);
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write");
        }
    }
}

pub fn sys_exit(xstate: i32) -> ! {
    println!("[kernel] Application exited with code {}", xstate);
    run_next_app()
}

pub fn syscall(id: SYSCALL, args: [usize; 3]) -> isize {
    match id {
        SYSCALL::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL::Exit => sys_exit(args[0] as i32),
    }
}