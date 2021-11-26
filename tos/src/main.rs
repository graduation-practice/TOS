#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![feature(alloc_error_handler)]
global_asm!(include_str!("boot/entry.asm"));
use tos;
use tos::println;
extern crate alloc;
#[macro_use]
extern crate bitflags;
pub use tos::kernel::mm;
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
pub fn rust_main() -> ! {
    //TODO 11.18 晚提交在运行 rust-objdump -all 会有err
    // error: address range table at offset 0x7380 has a premature terminator entry at offset 0x7390

    // println!("work");
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack();
        fn boot_stack_top();
    }
    clear_bss();
    println!("Hello, world!");
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    tos::arch::trap::init();
    tos::arch::timer::init();

    // extern "C" {
    //     fn ekernel();make show
    // }
    // println!(
    //     "free physical memory paddr = [{:#x}, {:#x})",
    //     ekernel as usize - 0x80200000 + 0x80200000,
    //     0x88000000 as u32,
    // );
    tos::kernel::init();
    // tos::kernel::mm::frame_allocator::frame_allocator_test();
    // panic!("end of rust_main");

    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // };
    loop {}
}
