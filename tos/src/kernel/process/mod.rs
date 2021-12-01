pub mod process;
pub mod processor;
pub mod thread;
use crate::kernel::process::{process::KERNEL_PROCESS, thread::Thread};

pub fn init_process() {
    //创建内核进程

    KERNEL_PROCESS.inner.lock().memory_set.activate();
}
