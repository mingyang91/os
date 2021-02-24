mod fs;
mod process;

use fs::*;
use process::*;
use crate::timer::get_time_ms;

#[repr(usize)]
pub enum SYSCALL {
    Write = 64,
    Exit = 93,
    Yield = 124,
    GetTime = 169,
}

impl SYSCALL {
    pub fn try_from(id: usize) -> SYSCALL {
        match id {
            64 => SYSCALL::Write,
            93 => SYSCALL::Exit,
            124 => SYSCALL::Yield,
            169 => SYSCALL::GetTime,
            _ => panic!("Unsupported syscall_id: {}", id),
        }
    }
}


pub fn syscall(id: SYSCALL, args: [usize; 3]) -> isize {
    match id {
        SYSCALL::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL::Exit => sys_exit(args[0] as i32),
        SYSCALL::Yield => sys_yield(),
        SYSCALL::GetTime => get_time_ms() as isize,
    }
}

