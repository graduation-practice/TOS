use crate::kernel::mm::address::VARangeOrd;
use crate::kernel::mm::space::{MapArea, MemorySet};
use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::String,
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use lazy_static::*;
use spin::Mutex;
lazy_static! {
    /// 内核进程，所有内核线程都属于该进程。
    /// 通过此进程来进行内核栈的分配
    pub static ref KERNEL_PROCESS: Arc<Process> = {
        println!("init kernel process");
        Arc::new(Process {
            pid: 0,
            inner: Mutex::new(ProcessInner {
                cwd: String::from("/"),
                memory_set: MemorySet {
                    page_table: crate::kernel::mm::page_table::kernel_page_table(),
                    areas: BTreeMap::<VARangeOrd, MapArea>::new(),
                },
                // fd_table: vec![Some(STDIN.clone()), Some(STDOUT.clone())],
                // parent: Weak::new(),
                // child: Vec::new(),
                // child_exited: Vec::new(),
                // wake_callbacks: Vec::new(),
            }),
        })
    };
}

pub type Pid = usize;

pub struct Process {
    pub pid: Pid,
    /// 可变的部分。如果要更高的细粒度，去掉 ProcessInner 的 Mutex，给里面的
    /// memory_set 等等分别加上
    pub inner: Mutex<ProcessInner>,
}

pub struct ProcessInner {
    /// 当前工作目录
    pub cwd: String,
    /// 进程中的线程公用页表 / 内存映射
    pub memory_set: MemorySet,
    // 文件描述符
    // pub fd_table: Vec<Option<Arc<FileDescriptor>>>,
    // 父进程
    // pub parent: Weak<Process>,
    // 子进程
    // pub child: Vec<Weak<Process>>,
    // 已经退出了的子进程 (进程ID, 弱引用，exit_status)，其中 exit_status
    // 只有低 8 bit 有效
    // pub child_exited: Vec<(Pid, Weak<Process>, i32)>,
    // 回调
    // pub wake_callbacks: Vec<Box<dyn Fn() + Send>>,
}
