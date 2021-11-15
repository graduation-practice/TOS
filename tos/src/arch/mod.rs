pub mod config;
pub mod context;
pub mod sbi;
pub mod timer;
pub mod trap;
use riscv::register::{scause::Scause, sstatus::Sstatus};
//TODO 统一底层接口
