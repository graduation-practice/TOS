#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
use tos;
use tos::{println};
global_asm!(include_str!("boot/entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn bootstack();
        fn bootstacktop();
    }
    clear_bss();
    println!("Hello, world!");
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(
        "boot_stack [{:#x}, {:#x})",
        bootstack as usize, bootstacktop as usize
    );
    println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    // panic!("Shutdown machine!");
    tos::arch::interrupt::init();
    tos::arch::timer::init();
    // panic!("end of rust_main");
    //
    loop{};
}


