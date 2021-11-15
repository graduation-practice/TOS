use super::address::{PPN, VA};
use bitflags::*;
bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PTE {
    pub bits: usize,
}

impl PTE {
    pub fn new(ppn: PPN, flags: PTEFlags) -> Self {
        PTE {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        PTE { bits: 0 }
    }

    pub fn ppn(&self) -> PPN {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [PTE; 512],
}
