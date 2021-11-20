extern crate alloc;
use super::address::{PA, PPN};

use crate::{arch::config::MEMORY_END, console::print};
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;
use spin::Mutex;
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PPN>;
    fn dealloc(&mut self, ppn: PPN);
}

pub struct FrameTracker {
    pub ppn: PPN,
}

impl FrameTracker {
    pub fn new(ppn: PPN) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}
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

    fn alloc(&mut self) -> Option<PPN> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }

    fn dealloc(&mut self, ppn: PPN) {
        let ppn = ppn.0;

        if ppn >= self.current || self.recycled.iter().find(|&v| *v == ppn).is_some() {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }

        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, p: PPN, r: PPN) {
        self.current = p.0;
        self.end = r.0;
    }
}

type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocatorImpl> =
        Mutex::new(FrameAllocatorImpl::new());
}



pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

pub fn frame_dealloc(ppn: PPN) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
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

pub fn init() {
    extern "C" {
        fn ekernel();
    }

    FRAME_ALLOCATOR.lock().init(
        PA::from(ekernel as usize).ceil(),
        PA::from(MEMORY_END).floor(),
    );

    //TODO debug 加了print语句后不触发page fault bug
    println!("init end!");
}