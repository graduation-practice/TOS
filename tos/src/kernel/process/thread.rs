use super::process::Process;
use crate::arch::config::{KERNEL_STACK_ALIGN_SIZE, KERNEL_STACK_SIZE, KERNEL_STACK_TOP};
use crate::arch::context::TaskContextImpl;
use crate::arch::context::TrapFrame;
use crate::kernel::mm::address::{VARange, VPNRange, VA, VPN};
use crate::kernel::mm::page_table::{PTEFlags, PTE};
use crate::kernel::process::process::KERNEL_PROCESS;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use lazy_static::*;
use spin::Mutex;
pub struct TidAllocator {
    current: usize,
    recycled: Vec<usize>,
}
#[derive(Debug)]
pub struct Tid(usize);
lazy_static! {
    /// 用于分配 tid
    pub static ref TID_ALLOCATOR: Mutex<TidAllocator> = Mutex::new(TidAllocator::new());
}
impl Drop for Tid {
    fn drop(&mut self) {
        TID_ALLOCATOR.lock().dealloc(self.0);
    }
}
impl TidAllocator {
    pub fn new() -> Self {
        TidAllocator {
            current: 2, // XXX 0 和 1 被调度线程使用
            recycled: Vec::with_capacity(4),
        }
    }

    pub fn alloc(&mut self) -> Tid {
        if let Some(tid) = self.recycled.pop() {
            Tid(tid)
        } else {
            self.current += 1;
            Tid(self.current - 1)
        }
    }

    pub fn dealloc(&mut self, tid: usize) {
        assert!(tid < self.current);
        assert!(
            self.recycled.iter().find(|ptid| **ptid == tid).is_none(),
            "tid {} has been deallocated!",
            tid
        );
        self.recycled.push(tid);
    }
}

pub struct Thread {
    /// 线程 ID
    pub tid: Tid,
    /// 所属的进程
    pub process: Arc<Process>,
    /// 用户栈顶
    pub user_stack_top: VA,
    /// 当线程处于 Ready 状态时，task_cx 指向保存在内核栈中的 TaskContextImpl；
    pub task_cx: &'static TaskContextImpl,
    /// 用 `Mutex` 包装一些可变的变量
    pub inner: Mutex<ThreadInner>,
}

pub struct ThreadInner {
    /// 线程状态
    pub status: ThreadStatus,
}
pub enum ThreadStatus {
    Ready,
    Running,
    Waiting,
    Zombie,
}
const THREAD_PTR_OFFSET: usize = size_of::<usize>();
const TRAP_FRAME_OFFSET: usize = THREAD_PTR_OFFSET + size_of::<TrapFrame>();
pub fn get_kernel_stack_range(tid: usize) -> VARange {
    let kernel_stack_top = KERNEL_STACK_TOP - tid * KERNEL_STACK_ALIGN_SIZE;
    VA(kernel_stack_top - KERNEL_STACK_SIZE)..VA(kernel_stack_top)
}
impl Thread {
    pub fn new_kernel_thread(entry: usize, _args: Option<&[usize]>) -> Arc<Thread> {
        //创建
        let tid = TID_ALLOCATOR.lock().alloc();
        let kernel_stack_range = get_kernel_stack_range(tid.0);
        KERNEL_PROCESS.inner.lock().memory_set.insert_framed_area(
            kernel_stack_range.clone(),
            (PTEFlags::R | PTEFlags::W).to_perm(),
            None,
        );

        // TrapFrame
        let task_cx = (kernel_stack_range.end - TRAP_FRAME_OFFSET).get_mut::<TaskContextImpl>();
        task_cx.set_ra(entry);

        let new_thread = Arc::new(Self {
            tid,
            process: KERNEL_PROCESS.clone(),
            user_stack_top: VA(0), // 内核线程的用户栈顶为 0，表示没有用户栈
            task_cx,
            inner: Mutex::new(ThreadInner {
                status: ThreadStatus::Ready,
            }),
        });
        *(kernel_stack_range.end - THREAD_PTR_OFFSET).get_mut::<usize>() =
            Arc::<Thread>::as_ptr(&new_thread) as usize;

        new_thread
    }
}
