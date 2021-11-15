pub mod address;
pub mod frame_allocator;
pub mod heap_allocator;
pub mod page_table;

pub fn init() {
    frame_allocator::init();
    heap_allocator::init_heap();
}
