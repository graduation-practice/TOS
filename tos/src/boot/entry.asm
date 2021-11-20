    .section .text.entry
    .globl _start
_start:
    li t1, 0xffffffc000000000
    la sp, boot_stack_top
.A:
    auipc   t0, %pcrel_hi(rust_main)
    addi    t0, t0, %pcrel_lo(.A)   # 得到物理地址
    add     t0, t0, t1              # 得到虚拟地址
    jr      t0
    // call rust_main

    .section .bss.stack
    .align 12
    .global boot_stack
boot_stack:
    .space 4096 * 4
    .global boot_stack_top
boot_stack_top: