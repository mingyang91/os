mod context;

use riscv::register::{
    scause::{
        self,
        Scause,
        Trap,
        Exception
    },
    stvec,
    mtvec::TrapMode,
    stval,
};
use crate::syscall::{syscall, SYSCALL};

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
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(ctx),
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            let id: SYSCALL = SYSCALL::try_from(ctx.x[17]);
            ctx.x[10] = syscall(id, [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        },
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
            panic!("[kernel] Cannot continue!");
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            panic!("[kernel] Cannot continue!");
        },
        _ => fault(ctx, scause, stval),
    }
    ctx
}


fn breakpoint(context: &mut TrapContext) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

fn fault(context: &mut TrapContext, scause: Scause, stval: usize) {
    panic!(
        "Unresolved trap: {:?}\n{:?}\nstval: {:#x}",// {:?}
        scause.cause(),
        context,
        stval
    );
}


pub use context::TrapContext;