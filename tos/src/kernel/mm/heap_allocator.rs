use crate::arch::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

//TODO 实现自己的分配器
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
//TODO LockedHeap 无法在String::from()上工作
#[repr(align(4096))]
pub struct HeapSpace(pub [u8; KERNEL_HEAP_SIZE]);
static mut HEAP_SPACE: HeapSpace = HeapSpace([0; KERNEL_HEAP_SIZE]);

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.0.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
