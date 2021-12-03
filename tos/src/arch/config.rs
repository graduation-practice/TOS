#[allow(unused)]
//TODO 适配k210
/// 内核使用线性映射的偏移量
pub const KERNEL_MAP_OFFSET: usize = 0xFFFF_FFC0_0000_0000;
/// 用户栈大小
pub const USER_STACK_SIZE: usize = 1 << 13;
/// 每个内核栈的栈顶都为 1 << KERNEL_STACK_SIZE_BITS 的倍数
pub const KERNEL_STACK_ALIGN_BITS: usize = 14;
/// 内核栈大小，最大为 1 << KERNEL_STACK_SIZE_BITS - PAGE_SIZE
pub const KERNEL_STACK_SIZE: usize = 1 << 13;
/// 内核堆大小
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
/// 内存起始地址
pub const MEMORY_START: usize = 0xFFFF_FFC0_8000_0000;
/// 内存大小
pub const MEMORY_SIZE: usize = 0x80_0000;

pub const MEMORY_END: usize = MEMORY_START + MEMORY_SIZE;
/// PAGE_SIZE = 1 << PAGE_SIZE_BITS
pub const PAGE_SIZE_BITS: usize = 12;
/// MMIO 起始地址
pub const MMIO: [(usize, usize); 1] = [(0x10001000, 0x1000)];
/// 时钟频率
pub const CLOCK_FREQ: u64 = 10_000_000;
/// boot cpu id
pub const BOOT_CPU_ID: usize = 0;
pub const PAGE_SIZE: usize = 0x1000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

pub const KERNEL_STACK_TOP: usize = usize::MAX - KERNEL_STACK_ALIGN_SIZE + 1;

/// 内核栈对齐大小
pub const KERNEL_STACK_ALIGN_SIZE: usize = 1 << KERNEL_STACK_ALIGN_BITS;
