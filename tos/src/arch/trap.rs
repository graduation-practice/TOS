use crate::arch::context::TrapFrame;
use crate::{arch::timer::tick, console::print};
use riscv::register::{
    scause::{self, Exception, Interrupt, Scause, Trap},
    sepc, sscratch, sstatus, stval, stvec,
};
global_asm!(include_str!("trap.asm"));
pub fn init() {
    unsafe {
        extern "C" {
            // 中断处理总入口
            fn __trap();
        }
        // 经过上面的分析，由于现在是在内核态
        // 我们要把 sscratch 初始化为 0
        sscratch::write(0);
        // 仍使用 Direct 模式
        // 将中断处理总入口设置为 __trap
        stvec::write(__trap as usize, stvec::TrapMode::Direct);
        // 设置 sstatus 的 SIE 位
        sstatus::set_sie();
    }
    println!("++++ setup interrupt! ++++");
}
//TODO 完善trap与double fault
#[no_mangle]
pub fn handle_trap(tf: &mut TrapFrame) {
    match scause::read().cause() {
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            tick();

            return;
        }
        _ => {}
    }

    unsafe {
        // 开启 SIE（不是 sie 寄存器），全局中断使能，允许内核态被中断打断
        riscv::register::sstatus::set_sie();
    }
    match scause::read().cause() {
        // exception
        Trap::Exception(Exception::Breakpoint) => breakpoint(&mut tf.sepc),
        Trap::Exception(Exception::InstructionPageFault) => page_fault(tf),
        Trap::Exception(Exception::LoadPageFault) => page_fault(tf),
        Trap::Exception(Exception::StorePageFault) => page_fault(tf),
        // Trap::Exception(Exception::StorePageFault) => page_fault(tf),
        _ => {
            println!(
                "Unsupported trap is excepeion {}, code {}, stval = {:?}!",
                scause::read().is_exception(),
                scause::read().code(),
                stval::read()
            );
            page_fault(tf)
        }
    }
    unsafe {
        // 返回时关闭全局中断
        riscv::register::sstatus::clear_sie();
    }
    // tf.sepc += 2;
}

fn breakpoint(sepc: &mut usize) {}
fn page_fault(tf: &mut TrapFrame) {
    // println!("store_fault!");
    // println!("Accessed Address: {:#?}", sepc::read());
    println!(
        "{:?} va = {:#x} instruction = {:#x}",
        scause::read().cause(),
        stval::read(),
        tf.sepc
    );
    panic!("page fault!");
}
