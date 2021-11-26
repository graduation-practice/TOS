#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(step_trait)]
extern crate alloc;
#[macro_use]
pub mod console;
pub mod arch;
mod init;
pub mod kernel;
mod lang_items;
