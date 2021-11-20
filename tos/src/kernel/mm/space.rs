use super::address::{VPNRange, PA, PPN, VA, VPN};
use super::frame_allocator::{frame_alloc, FrameTracker};
use super::page_table::{PTEFlags, PageTable};
use crate::arch::config::{MEMORY_END, TRAMPOLINE};
use crate::console::print;
use _core::iter::Map;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use bitflags::*;
use lazy_static::*;
use riscv::register::satp;
#[derive(Clone, Copy)]
pub enum MapType {
    /// 线性映射
    Linear,
    /// 按帧映射
    Framed,
    // 设备
    //Device,
    // 内核栈
    //KernelStack,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VPN, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}


impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
    }

    pub fn insert_framed_area(&mut self, sva: VA, eva: VA, permission: MapPermission) {
        self.push(MapArea::new(sva, eva, MapType::Framed, permission), None);
    }
    // fn map_trampoline(&mut self) {
    //     self.page_table.map(
    //         VA::from(TRAMPOLINE).into(),
    //         PA::from(strampoline as usize).into(),
    //         PTEFlags::R | PTEFlags::X,
    //     );
    // }

    fn new_kernel() -> Self {
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
            // fn strampoline();
        
        }
        println!("it work!");
        
        // memory_set.map_trampoline();
        // map kernel sections
        println!("it work!");
        
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);

        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);

        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);

        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );

        println!("mapping .text section");
        let mut memory_set = Self::new();
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Linear,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );

        println!("mapping .rodata section");

        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Linear,
                MapPermission::R,
            ),
            None,
        );

        println!("mapping .data section");

        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        println!("mapping .bss section");

        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        println!("mapping physical memory");

        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        memory_set
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            llvm_asm!("sfence.vma" :::: "volatile");
        }
    }

    // fn from_elf(elf_data: &[u8]) -> (Self, usize, usize);
}

impl MapArea {
    pub fn new(sva: VA, eva: VA, map_type: MapType, permission: MapPermission) -> Self {
        let svpn: VPN = sva.floor();
        let evpn: VPN = eva.ceil();
        Self {
            vpn_range: VPNRange::new(svpn, evpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm: permission,
        }
    }

    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VPN) {
        let ppn: PPN;
        match self.map_type {
            MapType::Linear => {
                ppn = PPN(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
}
use alloc::sync::Arc;
use spin::Mutex;
lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()) );
}
