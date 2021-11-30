pub mod mm;
pub mod process;
pub mod sync;
pub fn init() {
    mm::init_mm();
    process::init_process();
}
