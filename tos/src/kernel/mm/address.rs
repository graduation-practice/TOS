use super::page_table::PTE;
use crate::arch::config::{KERNEL_MAP_OFFSET, PAGE_SIZE, PAGE_SIZE_BITS};

use core::{fmt::Debug, iter::Step, mem::size_of};

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

///从指针转为虚拟地址
impl<T> From<*const T> for VA {
    fn from(pointer: *const T) -> Self {
        Self(pointer as usize)
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
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { &mut *(self.0 as *mut T) }
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

    pub fn get_array<T>(&self) -> &'static mut [T] {
        assert!(PAGE_SIZE % size_of::<T>() == 0);
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.0 << PAGE_SIZE_BITS) as *mut T,
                PAGE_SIZE / size_of::<T>(),
            )
        }
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

/// 为各种仅包含一个 usize 的类型实现运算操作
/// TODO 把用不到的删掉
macro_rules! implement_usize_operations {
    ($type_name: ty) => {
        /// `+`
        #[allow(unused_unsafe)]
        impl core::ops::Add<usize> for $type_name {
            type Output = Self;

            fn add(self, other: usize) -> Self::Output {
                Self(self.0 + other)
            }
        }
        /// `+=`
        #[allow(unused_unsafe)]
        impl core::ops::AddAssign<usize> for $type_name {
            fn add_assign(&mut self, rhs: usize) {
                unsafe {
                    self.0 += rhs;
                }
            }
        }
        /// `-`
        #[allow(unused_unsafe)]
        impl core::ops::Sub<usize> for $type_name {
            type Output = Self;

            fn sub(self, other: usize) -> Self::Output {
                Self(self.0 - other)
            }
        }
        /// `-`
        impl core::ops::Sub<$type_name> for $type_name {
            type Output = usize;

            fn sub(self, other: $type_name) -> Self::Output {
                self.0 - other.0
            }
        }
        /// `-=`
        #[allow(unused_unsafe)]
        impl core::ops::SubAssign<usize> for $type_name {
            fn sub_assign(&mut self, rhs: usize) {
                self.0 -= rhs;
            }
        }
        /// 和 usize 相互转换
        // #[allow(unused_unsafe)]
        // impl From<usize> for $type_name {
        //     fn from(value: usize) -> Self {
        //         Self(value)
        //     }
        // }
        // /// 和 usize 相互转换
        // impl From<$type_name> for usize {
        //     fn from(value: $type_name) -> Self {
        //         value.0
        //     }
        // }
        /// 是否有效（0 为无效）
        impl $type_name {
            pub fn valid(&self) -> bool {
                self.0 != 0
            }
        }
        /// {} 输出
        impl core::fmt::Display for $type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}(0x{:x})", stringify!($type_name), self.0)
            }
        }
    };
}
implement_usize_operations! {PA}
implement_usize_operations! {VA}
implement_usize_operations! {PPN}
implement_usize_operations! {VPN}

impl Step for VPN {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Step::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(start + count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(start - count)
    }
}

impl Step for PPN {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Step::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(start + count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(start - count)
    }
}
pub type VPNRange = core::ops::Range<VPN>;
pub type VARange = core::ops::Range<VA>;

#[derive(Clone)]
pub struct VARangeOrd(pub VARange);

impl VARangeOrd {
    /// 获取 VPNRange
    pub fn vpn_range(&self) -> VPNRange {
        self.0.start.floor()..self.0.end.ceil()
    }
}

impl Ord for VARangeOrd {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.eq(other) {
            core::cmp::Ordering::Equal
        } else if self.0.start < other.0.start {
            core::cmp::Ordering::Less
        } else {
            core::cmp::Ordering::Greater
        }
    }
}
impl PartialOrd for VARangeOrd {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for VARangeOrd {}
impl PartialEq for VARangeOrd {
    fn eq(&self, other: &Self) -> bool {
        (self.0.start <= other.0.start && other.0.end <= self.0.end)
            || (other.0.start <= self.0.start && self.0.end <= other.0.end)
    }
}

#[macro_export]
macro_rules! round_down {
    ($value: expr, $boundary: expr) => {
        ($value & !($boundary - 1))
    };
}

#[macro_export]
macro_rules! round_up {
    ($value: expr, $boundary: expr) => {
        ($value + $boundary - 1 & !($boundary - 1))
    };
}
