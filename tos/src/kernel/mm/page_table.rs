use super::address::{VARange, VARangeOrd, PPN, VA, VPN};
use super::frame_allocator::{frame_alloc, frame_dealloc, Frame, FrameTracker};
use super::space::{MapArea, MapPermission, MapType};
use crate::arch::config::{KERNEL_STACK_TOP, MEMORY_END};
use crate::console::print;
use crate::kernel::process::process::KERNEL_PROCESS;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use core::slice::from_raw_parts_mut;
use lazy_static::lazy_static;
use riscv::asm::{sfence_vma, sfence_vma_all};
use riscv::register::satp;
bitflags! {
    pub struct PTEFlags: u8 {
        const E =      0;
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
impl PTEFlags {
    pub fn to_perm(&self) -> MapPermission {
        MapPermission::from_bits(0x3c & self.bits as u8).unwrap()
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
    pub root: FrameTracker,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    ///create a new page table
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        // println!("new page table");
        Self {
            root: frame,
            frames: vec![],
        }
    }

    /// TODO 考虑页面不够的情况
    pub fn map(&mut self, va_range: VARangeOrd, area: &mut MapArea, data: Option<&[u8]>) {
        match area.map_type {
            MapType::Linear => {
                for vpn in va_range.vpn_range() {
                    self.map_one(vpn, vpn.into(), area.map_perm.to_pte());
                }
                // 线性映射的 area 是一段连续的地址，可以直接复制
                if let Some(data) = data {
                    unsafe {
                        from_raw_parts_mut(va_range.0.start.get_mut(), data.len())
                            .copy_from_slice(data);
                    }
                }
            }
            MapType::Framed => {
                match data {
                    // 有数据，且数据长度不为 0
                    Some(data) if data.len() != 0 => {
                        let src_vpn_range = VA::from(data.as_ptr()).floor()
                            ..VA::from(data.as_ptr() as usize + data.len()).ceil();
                        // println!("src_vpn_range {:x?}", src_vpn_range);
                        // println!("vpn {:x?}", va_range.vpn_range());
                        // XXX va_range.start 和 end 可能并非 4k 对齐的，导致多复制了一些数据
                        for (vpn, src_vpn) in va_range.vpn_range().zip(src_vpn_range) {
                            let dst_frame = frame_alloc().unwrap();
                            self.map_one(vpn, dst_frame.ppn, area.map_perm.to_pte());
                            VPN::from(dst_frame.ppn)
                                .get_array()
                                .copy_from_slice(src_vpn.get_array::<usize>());
                            // println!("{:?}", src_vpn.get_array::<usize>());
                            area.data_frames.insert(vpn, dst_frame);
                        }
                    }
                    // 数据长度为 0，说明是 bss 段
                    Some(_) => {
                        for vpn in va_range.vpn_range() {
                            let dst_frame = frame_alloc().unwrap();
                            self.map_one(vpn, dst_frame.ppn, area.map_perm.to_pte());
                            VPN::from(dst_frame.ppn).get_array().fill(0usize);
                            area.data_frames.insert(vpn, dst_frame);
                        }
                    }
                    // 内核栈/用户栈
                    _ => {
                        for vpn in va_range.vpn_range() {
                            let dst_frame = frame_alloc().unwrap();
                            self.map_one(vpn, dst_frame.ppn, area.map_perm.to_pte());
                            area.data_frames.insert(vpn, dst_frame);
                        }
                    }
                }
            } // MapType::Device => {
              //     for vpn in va_range.vpn_range() {
              //         self.map_one(vpn, PPN(vpn.0), area.map_perm);
              //     }
              // }
        }
    }
    //TODO 暂时copy 后续优化
    pub fn map_one(&mut self, vpn: VPN, ppn: PPN, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PTE::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VPN) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PTE::empty();
    }

    // fn find_pte_create(&mut self, vpn: VPN) -> Option<&mut PTE> {
    //     let idxs = vpn.indexes();
    //     let mut ppn = self.root.ppn;
    //     let mut result: Option<&mut PTE> = None;
    //     for i in 0..3 {
    //         let pte = &mut ppn.get_pte_array()[idxs[i]];
    //         if i == 2 {
    //             result = Some(pte);
    //             break;
    //         }
    //         if !pte.is_valid() {
    //             let frame = frame_alloc().unwrap();
    //             *pte = PTE::new(frame.ppn, PTEFlags::V);

    //             self.frames.push(frame);
    //         }
    //         ppn = pte.ppn();
    //     }
    //     result
    // }
    fn find_pte_create(&mut self, vpn: VPN) -> Option<&mut PTE> {
        let idxs = vpn.indexes();
        let mut pte: &mut PTE = &mut VPN::from(self.root.ppn).get_array()[idxs[0]];
        for &idx in &idxs[1..] {
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                VPN::from(frame.ppn).get_array::<PTE>().fill(PTE::empty());
                *pte = PTE::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            pte = &mut VPN::from(pte.ppn()).get_array()[idx];
        }
        Some(pte)
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
        println!("enter active:");
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

lazy_static! {
    /// 请通过内核进程而非此变量来映射内核栈，因为映射涉及到页框的创建和保存
    pub static ref KERNEL_PAGE_TABLE: &'static PageTable =
        unsafe { &*(&KERNEL_PROCESS.inner.lock().memory_set.page_table as *const PageTable) };
}

pub fn kernel_page_table() -> PageTable {
    println!("enter new kernel page table!");
    let frame = frame_alloc().unwrap();
    // use riscv::register::satp;
    //TODO 加print 不触发page fault
    // println!("{}", frame.ppn);
    // println!("{:#x}", satp::read().bits());
    VPN::from(frame.ppn)
        .get_array::<PTEFlags>()
        .fill(PTEFlags::E);

    let mut page_table = PageTable {
        root: frame,
        frames: vec![],
    };
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss_with_stack();
        fn ebss();
        fn ekernel();
    }
    let areas: [(VARange, PTEFlags); 2] = [
        (
            (stext as usize).into()..(etext as usize).into(),
            PTEFlags::R | PTEFlags::X,
        ),
        (
            (srodata as usize).into()..(erodata as usize).into(),
            PTEFlags::R,
        ),
        // (
        //     (sdata as usize).into()..(edata as usize).into(),
        //     PTEFlags::R | PTEFlags::W,
        // ),
        // (
        //     (sbss_with_stack as usize).into()..(ebss as usize).into(),
        //     PTEFlags::R | PTEFlags::W,
        // ),
        // (
        //     (ekernel as usize).into()..MEMORY_END.into(),
        //     PTEFlags::R | PTEFlags::W,
        // ),
    ];

    for area in areas {
        page_table.map(
            VARangeOrd((area.0).clone()),
            &mut MapArea {
                //TODO 精简MapArea
                vpn_range: VARangeOrd((area.0).clone()).vpn_range(),
                data_frames: BTreeMap::new(),
                map_type: MapType::Linear,
                map_perm: area.1.to_perm(),
            },
            None,
        );
    }

    let vpn = VA(KERNEL_STACK_TOP).floor().indexes()[0];
    let pte: &mut PTE = &mut VPN::from(page_table.root.ppn).get_array()[vpn];
    let frame = frame_alloc().unwrap();
    VPN::from(frame.ppn)
        .get_array::<PTEFlags>()
        .fill(PTEFlags::E);
    *pte = PTE::new(frame.ppn, PTEFlags::V);
    page_table.frames.push(frame);
    println!("sucess init kernel page table");
    page_table
}
