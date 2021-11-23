use super::page_table::PTE;
use crate::arch::config::{KERNEL_MAP_OFFSET, PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PA(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VA(pub usize);

#[derive(Copy, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PPN(pub usize);

#[derive(Copy, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VPN(pub usize);
/// VPN -> PPN
impl From<VPN> for PPN {
    fn from(vpn: VPN) -> Self {
        Self(vpn.0 - (KERNEL_MAP_OFFSET >> PAGE_SIZE_BITS))
    }
}

/// PPN -> VPN
impl From<PPN> for VPN {
    fn from(ppn: PPN) -> Self {
        Self(ppn.0 + (KERNEL_MAP_OFFSET >> PAGE_SIZE_BITS))
    }
}

impl From<usize> for PA {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<PA> for usize {
    fn from(v: PA) -> Self {
        v.0
    }
}

impl From<PA> for PPN {
    fn from(v: PA) -> Self {
        assert_eq!(v.page_offset(), 0);

        v.floor()
    }
}

impl From<VA> for VPN {
    fn from(v: VA) -> Self {
        assert_eq!(v.page_offset(), 0);

        v.floor()
    }
}
impl From<usize> for PPN {
    fn from(v: usize) -> Self {
        // println!("usize -> ppn");
        Self(v)
    }
}
impl From<PPN> for usize {
    fn from(v: PPN) -> Self {
        v.0
    }
}

impl From<PPN> for PA {
    fn from(v: PPN) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<VA> for usize {
    fn from(v: VA) -> Self {
        v.0
    }
}

impl From<usize> for VA {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl PA {
    pub fn floor(&self) -> PPN {
        PPN(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PPN {
        PPN((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl VA {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn floor(&self) -> VPN {
        VPN(self.0 / PAGE_SIZE)
    }

    pub fn ceil(&self) -> VPN {
        VPN((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

impl PPN {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PA = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }

    pub fn get_pte_array(&self) -> &'static mut [PTE] {
        let pa: PA = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PA = self.clone().into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

impl VPN {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VPN {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
pub type VPNRange = SimpleRange<VPN>;
