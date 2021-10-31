use riscv::register::{sie,time};
use core::time::Duration;
use super::sbi::set_timer;

//TODO 完善时钟中断
// 时钟中断计数
pub static mut TICKS: usize = 0;

// 时钟中断间隔

const TIME_BASE: u64 = 100_000;

pub fn init() {
    unsafe {
        TICKS = 0;

        //enable S mode timer interrupt
        sie::set_stimer();
    }
    set_next_timeout();
    println!("+++occur+++");
}

#[inline]
fn set_next_timeout() {

    set_timer(time::read() + TIME_BASE as usize);
}

