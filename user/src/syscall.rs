
pub const STDOUT: usize = 1;


#[repr(usize)]
enum SYSCALL {
    Write = 64,
    Exit = 93,
    Yield = 124,
    GetTime = 169,
}

fn syscall(id: SYSCALL, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x17}" (id as usize)
            : "memory"
            : "volatile"
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL::Write, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYSCALL::Exit, [xstate as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL::Yield, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL::GetTime, [0, 0, 0])
}
