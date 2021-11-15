use crate::arch::config::{PAGE_SIZE, PAGE_SIZE_BITS};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PA(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VA(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PPN(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VPN(pub usize);

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
impl From<usize> for PPN {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl From<PPN> for PA {
    fn from(v: PPN) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
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
}

impl PPN {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PA = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
}
