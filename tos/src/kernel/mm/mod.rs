pub mod address;
pub mod frame_allocator;
pub mod heap_allocator;
pub mod page_table;
pub mod space;
use core::iter::Map;
use space::KERNEL_SPACE;
use space::{MapArea, MapPermission, MemorySet};
pub fn init() {
    heap_allocator::init_heap();
    // println!("success init heap allocator");
    frame_allocator::init();
    // println!("success init frame allocator");

    //TODO 下面两句会触发code = 5 的exception
    // println!("create pt!");
    //let pt = page_table::PageTable::new();
    // let frame = frame_allocator::frame_alloc().unwrap();

    // println!("{} ", pt.root.ppn.0);
    // frame_allocator::frame_allocator_test();
    KERNEL_SPACE.lock().activate();
    // println!("++++ setup memory!    ++++");
}

// pub fn kernel_remap() {
//     extern "C" {
//         fn bootstack(); //定义在src/boot/entry64.asm
//         fn bootstacktop(); //定义在src/boot/entry64.asm
//     }
//     let mut memory_set = MemorySet::new();
//     let mut map_area = MapArea::new(
//         (bootstack as usize).into(),
//         (bootstacktop as usize).into(),
//         space::MapType::Linear,
//         MapPermission::R,
//     );

//     // 將启动栈 push 进来
//     memory_set.push(map_area, None);
//     unsafe {
//         memory_set.activate();
//     }
// }
