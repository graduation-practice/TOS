#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#[macro_use]
pub mod console;
mod lang_items;
mod init;
pub mod arch; 
