use riscv::register::{scause::Scause, sstatus::Sstatus};

#[repr(C)]
pub struct TrapFrame {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

#[repr(C)]
pub struct TaskContextImpl {
    pub ra: usize,
    satp: usize,
    s: [usize; 12],
}
impl TaskContextImpl {
    pub fn set_ra(&mut self, value: usize) -> &mut Self {
        self.ra = value;
        self
    }
}
