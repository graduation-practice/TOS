use super::address::{PPN, VA, VPN};
use super::frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use riscv::asm::{sfence_vma, sfence_vma_all};
use riscv::register::satp;
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
// pub struct PageTable {
//     entries: [PTE; 512],
// }
pub struct PageTable {
    root: FrameTracker,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    ///create a new page table
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root: frame,
            frames: vec![],
        }
    }
    //TODO 暂时copy 后续优化
    pub fn map(&mut self, vpn: VPN, ppn: PPN, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PTE::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VPN) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PTE::empty();
    }

    fn find_pte_create(&mut self, vpn: VPN) -> Option<&mut PTE> {
        let idxs = vpn.indexes();
        let mut ppn = self.root.ppn;
        let mut result: Option<&mut PTE> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PTE::new(frame.ppn, PTEFlags::V);

                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    /// set satp value 1000 means SV39
    pub fn token(&self) -> usize {
        8usize << 60 | self.root.ppn.0
    }

    unsafe fn set_token(token: usize) {
        llvm_asm!("csrw satp, $0" :: "r"(token) :: "volatile");
    }

    fn active_token() -> usize {
        satp::read().bits()
    }

    fn flush_tlb() {
        unsafe {
            unsafe {
                unsafe {
                    sfence_vma_all();
                }
            };
        }
    }

    pub unsafe fn activate(&self) {
        let old_token = Self::active_token();
        let new_token = self.token();
        println!("switch satp from {:#x} to {:#x}", old_token, new_token);
        if new_token != old_token {
            Self::set_token(new_token);
            // 别忘了刷新 TLB!
            Self::flush_tlb();
        }
    }
}
