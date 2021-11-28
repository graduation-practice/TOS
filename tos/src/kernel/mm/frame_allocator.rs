extern crate alloc;
use super::address::{PA, PPN, VA};
use crate::arch::config::{KERNEL_MAP_OFFSET, PAGE_SIZE, PAGE_SIZE_BITS};
use crate::{arch::config::MEMORY_SIZE, arch::config::MEMORY_START, console::print};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;
use spin::Mutex;
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<FrameTracker>;
    fn dealloc(&mut self, ft: &Frame);
}
pub struct Frame {
    // TODO 去掉 pub
    pub ppn: PPN,
}

impl Frame {
    pub fn new(ppn: PPN) -> Self {
        // page cleaning
        // let bytes_array = ppn.get_bytes_array();
        // for i in bytes_array {
        //     *i = 0;
        // }
        Self { ppn }
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Frame:PPN={:#x}", self.ppn.0))
    }
}
impl Drop for Frame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}
pub type FrameTracker = Arc<Frame>;

pub struct StackFrameAllocator {
    current: usize, //空闲内存的起始物理页号
    end: usize,     //空闲内存的结束物理页号
    recycled: Vec<usize>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<FrameTracker> {
        if let Some(ppn) = self.recycled.pop() {
            // println!("a");
            Some(Arc::new(Frame::new(ppn.into())))
        } else {
            if self.current == self.end {
                // println!("b");
                None
            } else {
                self.current += 1;
                // println!(
                //     "frame{:#?}",
                //     Some(Arc::new(Frame::new((self.current - 1).into())))

                // );
                // println!("c");
                Some(Arc::new(Frame::new((self.current - 1).into())))
            }
        }
    }

    fn dealloc(&mut self, ft: &Frame) {
        let ppn = ft.ppn.into();

        if ppn >= self.current || self.recycled.iter().find(|&v| *v == ppn).is_some() {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }

        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, c: PPN, e: PPN) {
        self.current = c.0;
        self.end = e.0;
        println!(
            "last {} Physical Frames: [{:#x}, {:#x}]",
            self.end - self.current,
            self.current,
            self.end
        );
    }
}

type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocatorImpl> =
        Mutex::new(FrameAllocatorImpl::new());
}

pub fn frame_alloc() -> Option<FrameTracker> {
    // println!("enter frame_alloc!");
    FRAME_ALLOCATOR.lock().alloc()

    // .map(|ppn| FrameTracker::new(ppn))
}

pub fn frame_dealloc(ft: &Frame) {
    FRAME_ALLOCATOR.lock().dealloc(ft);
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();

    for i in 0..5 {
        let frame = frame_alloc().unwrap();

        println!("{:?}", frame);
        v.push(frame);
        println!("{}", i);
    }

    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }

    drop(v);
    println!("frameallocator_test passed!");
}

/// init frame allocator
pub fn init() {
    extern "C" {
        fn ekernel();
    }

    FRAME_ALLOCATOR.lock().init(
        VA::from(ekernel as usize + KERNEL_MAP_OFFSET).ceil().into(),
        VA::from(ekernel as usize + KERNEL_MAP_OFFSET + MEMORY_SIZE)
            .floor()
            .into(),
    );

    //TODO debug 加了print语句后不触发page fault bug
    println!("init frame allocator end!");
}
