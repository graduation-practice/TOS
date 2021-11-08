use riscv::register::{
    scause::{
        self,
        Trap,
        Exception,
        Interrupt
    },
    sepc,
    stvec,
    sscratch,
    sstatus
};

use crate::{arch::timer::{
    tick,
}, console::print};
use crate::arch::context::TrapFrame;
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
        // 将中断处理总入口设置为 __alltraps
        stvec::write(__trap as usize, stvec::TrapMode::Direct);
        // 设置 sstatus 的 SIE 位
        sstatus::set_sie();
    }
    println!("++++ setup interrupt! ++++");
}
#[no_mangle]
pub fn handle_trap(tf: &mut TrapFrame){

    // 
    match scause::read().cause() {
        // 断点中断
        Trap::Exception(Exception::Breakpoint) => breakpoint(&mut tf.sepc),
        // S态时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => tick(),
        _ => panic!("undefined trap!")
    }
    
    // 触发中断时，硬件会将 sepc 设置为触发中断指令的地址
    // 而中断处理结束，使用 sret 返回时也会跳转到 sepc 处
    // 于是我们又要执行一次那条指令，触发中断，无限循环下去
    // 而我们这里是断点中断，只想这个中断触发一次
    // 因此我们将中断帧内的 sepc 字段设置为触发中断指令下一条指令的地址，即中断结束后跳过这条语句
    // 由于 riscv64 的每条指令都是 32 位，4 字节，因此将地址+ 4 即可
    // 这样在 RESTORE_ALL 时，这个修改后的 sepc 字段就会被 load 到 sepc 寄存器中
    // 使用 sret 返回时就会跳转到 ebreak 的下一条指令了
    tf.sepc += 4;
}


fn breakpoint(sepc: &mut usize) {}
