MAKEFILE_DIR := $(dir $(abspath $(firstword $(MAKEFILE_LIST))))
target := riscv64imac-unknown-none-elf
rustsbi := ../bootloader/rustsbi-qemu.bin
mode := debug
kernel := $(MAKEFILE_DIR)target/$(target)/$(mode)/tos
bin := $(MAKEFILE_DIR)target/$(target)/$(mode)/kernel.bin
objdump := rust-objdump --arch-name=riscv64
objcopy := rust-objcopy --binary-architecture=riscv64
.PHONY: kernel build clean qemu run env
env:
	cargo install cargo-binutils
	rustup component add llvm-tools-preview rustfmt
	rustup target add $(target)
kernel:
	cargo build 
build: kernel
	$(objcopy) $(kernel) --strip-all -O binary $(bin)
asm:
	$(objdump) -d $(kernel) | less

clean:
	cargo clean
qemu: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(rustsbi) \
		-device loader,file=$(bin),addr=0x80200000
run: build qemu

read: build
	rust-objdump -all $(kernel)
