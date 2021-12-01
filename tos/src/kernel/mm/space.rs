use super::address::{VARange, VPNRange, PA, PPN, VA, VPN};
use super::frame_allocator::{frame_alloc, FrameTracker};
use super::page_table::{PTEFlags, PageTable};
use crate::arch::config::{MEMORY_END, TRAMPOLINE};
use crate::console::print;
use crate::kernel::mm::address::VARangeOrd;
use _core::iter::Map;
use alloc::collections::BTreeMap;
use alloc::vec;
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

impl MapPermission {
    pub fn to_pte(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
}
pub struct MapArea {
    pub vpn_range: VPNRange,
    pub data_frames: BTreeMap<VPN, FrameTracker>,
    pub map_type: MapType,
    pub map_perm: MapPermission,
}

pub struct MemorySet {
    pub page_table: PageTable,
    pub areas: BTreeMap<VARangeOrd, MapArea>,
}

impl MemorySet {
    pub fn new() -> Self {
        // println!("new!");
        Self {
            page_table: PageTable::new(),
            areas: BTreeMap::<VARangeOrd, MapArea>::new(),
        }
    }

    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
    }

    /// 在地址空间插入一段按帧映射的区域，未检查重叠区域
    pub fn insert_framed_area(
        &mut self,
        va_range: VARange,
        map_perm: MapPermission,
        data: Option<&[u8]>,
    ) {
        let mut area = MapArea {
            vpn_range: VARangeOrd(va_range.clone()).vpn_range(),
            data_frames: BTreeMap::new(),
            map_type: MapType::Framed,
            map_perm,
        };
        // println!("{:#x?} {:?}", va_range, map_perm);
        self.page_table
            .map(VARangeOrd(va_range.clone()), &mut area, data);
        self.areas.insert(VARangeOrd(va_range), area);
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

        // println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);

        // println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);

        // println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);

        // println!(
        //     ".bss [{:#x}, {:#x})",
        //     sbss_with_stack as usize, ebss as usize
        // );
        // println!("m");
        let mut memory_set = Self::new();
        // println!("m");
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
        let satp = 8usize << 60 | self.page_table.root.ppn.0;
        println!("active page_table!");
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
        // println!("active page_table!");
    }

    // fn from_elf(elf_data: &[u8]) -> (Self, usize, usize);
}

impl MapArea {
    pub fn new(sva: VA, eva: VA, map_type: MapType, permission: MapPermission) -> Self {
        let svpn: VPN = sva.floor();
        let evpn: VPN = eva.ceil();
        Self {
            vpn_range: { svpn..evpn },
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
        page_table.map_one(vpn, ppn, pte_flags);
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.clone() {
            self.map_one(page_table, vpn);
        }
    }
}
use alloc::sync::Arc;
use spin::Mutex;
lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}
