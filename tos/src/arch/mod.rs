pub mod timer;
pub mod sbi;
pub mod interrupt;
pub mod context;
pub mod config;
use riscv::register::{
    sstatus::Sstatus,
    scause::Scause,
};
