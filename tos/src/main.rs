#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![feature(alloc_error_handler)]
global_asm!(include_str!("boot/entry.asm"));
use tos::arch::sbi::shutdown;
use tos::println;
use tos::{self, console::print};
extern crate alloc;
use tos::arch::trap::TrapImpl;
use tos::arch::trap_context::Trap;
#[macro_use]
extern crate bitflags;
use riscv::register::{satp, sstatus};
pub use tos::kernel::{mm, process};
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    unsafe {
        let mut cur = sbss as *mut usize;
        let end = ebss as *mut usize;
        while cur < end {
            core::ptr::write_volatile(cur, core::mem::zeroed());
            cur = cur.offset(1);
        }

        // 测试后面的内存是否能访问
        cur = (crate::tos::arch::config::MEMORY_END as *mut usize).offset(-1);
        *cur = 0x1234_5678;
        assert_eq!(*cur, 0x1234_5678);
    }
}

#[no_mangle]
pub fn rust_main() -> ! {
    use riscv::asm::ebreak;

    unsafe {
        // 允许内核读写用户态内存
        riscv::register::sstatus::set_sum();
        println!("{:?}", sstatus::read().spp());
    }
    // println!("{:#x}", 0x00000 << 10 | 0xCF);
    // println!("{:#x}", 0x40000 << 10 | 0xCF);
    // println!("{:#x}", 0x80000 << 10 | 0xCF);
    // println!("{:#x}", (0xC0000 << 10) | 0xCF);

    // //TODO 11.18 晚提交在运行 rust-objdump -all 会有err
    // // error: address range table at offset 0x7380 has a premature terminator entry at offset 0x7390

    // // println!("work");
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn sbss_with_stack();
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
    println!(".bssstack [{:#x})", sbss_with_stack as usize);
    TrapImpl::init();
    // // tos::arch::timer::init();
    // use riscv::asm::ebreak;
    // unsafe {
    //     ebreak();
    // }

    // extern "C" {
    //     fn ekernel();make show
    // }
    // println!(
    //     "free physical memory paddr = [{:#x}, {:#x})",
    //     ekernel as usize - 0x80200000 + 0x80200000,
    //     0x88000000 as u32,
    // );
    // use alloc::string::String;
    // let mut a = String::new();
    // a.push('c');
    tos::kernel::init_kernel();

    // tos::kernel::mm::frame_allocator::frame_allocator_test();
    // panic!("end of rust_main");

    // unsafe {
    //     *(0xffff_ffc0_8020_0000 as *mut u64) = 42;
    // };

    // use riscv::register::satp;
    // println!("{:#?}", satp::read().ppn());
    loop {}
    // shutdown();
}
