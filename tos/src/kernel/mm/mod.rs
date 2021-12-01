pub mod address;
pub mod frame_allocator;
pub mod heap_allocator;
pub mod page_table;
pub mod space;
use crate::kernel::mm::address::VARangeOrd;
use crate::kernel::mm::page_table::kernel_page_table;
use alloc::collections::BTreeMap;
use core::iter::Map;
use riscv::register::satp;
// pub use p:KERNEL_PAGE_TABLE;
use space::KERNEL_SPACE;
use space::{MapArea, MapPermission, MemorySet};
pub fn init_mm() {
    heap_allocator::init_heap();
    // println!("success init heap allocator");
    frame_allocator::init_allocator();
    // println!("success init frame allocator");

    // let area = BTreeMap::<VARangeOrd, MapArea>::new();

    // unsafe {
    //     page_table::KERNEL_PAGE_TABLE.activate();
    // }

    //TODO 下面两句会触发code = 5 的exception
    // println!("create pt!");

    // let frame = frame_allocator::frame_alloc().unwrap();

    // println!("{} ", pt.root.ppn.0);

    // frame_allocator::frame_allocator_test();
    // kernel_remap();
    // KERNEL_SPACE.lock().activate();
    // println!("++++ setup memory!    ++++");

    // unsafe {
    //     // satp::write(0);
    //     asm!("sfence.vma");
    // }
}

pub fn kernel_remap() {
    println!("success");
    extern "C" {
        fn boot_stack(); //定义在src/boot/entry64.asm
        fn boot_stack_top(); //定义在src/boot/entry64.asm
    }
    let mut memory_set = MemorySet::new();

    let mut map_area = MapArea::new(
        (boot_stack as usize).into(),
        (boot_stack_top as usize).into(),
        space::MapType::Linear,
        MapPermission::R,
    );

    // 將启动栈 push 进来
    memory_set.push(map_area, None);
    unsafe {
        memory_set.activate();
    }
    println!("success");
}
