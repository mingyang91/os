mod context;

use riscv::register::{
    scause::{
        Scause,
        Trap,
        Exception
    },
    stvec,
    mtvec::TrapMode,
};
use crate::syscall::{syscall, SYSCALL};
use crate::batch::run_next_app;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(context: &mut TrapContext, scause: Scause, stval: usize) -> &mut TrapContext {
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        Trap::Exception(Exception::UserEnvCall) => {
            context.sepc += 4;
            let id = SYSCALL::try_from(context.x[17]);
            context.x[10] = syscall(id, [context.x[10], context.x[11], context.x[12]]) as usize;
        },
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
            run_next_app();
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            run_next_app();
        },
        _ => fault(context, scause, stval),
    }
    context
}


fn breakpoint(context: &mut TrapContext) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

fn fault(context: &mut TrapContext, scause: Scause, stval: usize) {
    panic!(
        "Unresolved interrupt: {:?}\n{:x?}\nstval: {:x}",
        scause.cause(),
        context,
        stval
    );
}


pub use context::TrapContext;