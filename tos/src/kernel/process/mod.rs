pub mod process;
pub mod processor;
pub mod thread;
use crate::kernel::process::process::KERNEL_PROCESS;

pub fn init_process() {
    //创建内核进程

    KERNEL_PROCESS.inner.memory_set.activate();

    // let mut kernle_process = process::Process::new_kernel();
    // kernle_process.inner.memory_set.activate();
}
