//TODO 完善时钟中断 双核
use super::sbi::set_timer;
use crate::arch::config::CLOCK_FREQ;
use core::time::Duration;
use riscv::register::{sie, time};

// 时钟中断计数
pub static mut TICKS: u64 = 0;

const TICKS_PER_SEC: u64 = 100;
const MSEC_PER_SEC: u64 = 1_000;
const USEC_PER_SEC: u64 = 1_000_000;
const NSEC_PER_SEC: u64 = 1_000_000_000;

// 时钟中断间隔
pub const INTERVAL: u64 = CLOCK_FREQ / TICKS_PER_SEC - 1;

// 初始化时钟中断
pub fn init() {
    unsafe {
        TICKS = 0;

        //允许时钟中断
        sie::set_stimer();
    }
    // 设置下一次时钟中断
    set_next_timeout();
}

pub fn tick() {
    set_next_timeout();
    unsafe {
        TICKS += 1;
        if TICKS % TICKS_PER_SEC == 0 {
            println!("{} s", TICKS / TICKS_PER_SEC);
        }
    }
}

#[inline]
pub fn set_next_timeout() {
    set_timer(time::read() + INTERVAL as usize);
}
