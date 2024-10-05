//! SBI call wrappers

use sbi_rt::{NoReason, Shutdown};

/// use sbi call to shutdown the kernel
pub fn shutdown() -> ! {
    sbi_rt::system_reset(Shutdown, NoReason);
    // crate::board::QEMU_EXIT_HANDLE.exit_failure();
    unreachable!()
}
