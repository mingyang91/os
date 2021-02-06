# Makefile

target := riscv64gc-unknown-none-elf
mode := debug
KERNEL_ELF := target/$(target)/$(mode)/os
KERNEL_BIN := ${KERNEL_ELF}.bin
BOOTLOADER := tools/bootloader/rustsbi-k210.bin
K210-SERIALPORT ?= /dev/cu.usbserial-615648CD930

objdump := rust-objdump --arch-name=riscv64
objcopy := rust-objcopy --binary-architecture=riscv64

.PHONY: kernel build clean qemu run env

env:
	cargo install cargo-binutils
	rustup component add llvm-tools-preview rustfmt
	rustup target add $(target)
	pip3 install kflash

kernel:
	cargo build

$(KERNEL_BIN): kernel
	$(objcopy) $(KERNEL_ELF) --strip-all -O binary $@

asm:
	$(objdump) -d $(KERNEL_ELF) | less

build: $(KERNEL_BIN)

clean:
	cargo clean

qemu: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios tools/bootloader/rustsbi-qemu.bin \
		-device loader,file=$(KERNEL_BIN),addr=0x80200000

run: build qemu

wsl: build
	qemu-system-riscv64 \
		-machine virt \
		-m 256M \
		-nographic \
		-bios tools/bootloader/opensbi0.7-qemu.bin \
		-device loader,file=$(KERNEL_BIN),addr=0x80200000

flash:
	@cp $(BOOTLOADER) $(BOOTLOADER).copy
	@dd if=$(KERNEL_BIN) of=$(BOOTLOADER).copy bs=128k seek=1
	@mv $(BOOTLOADER).copy $(KERNEL_BIN)
	@sudo chmod 777 $(K210-SERIALPORT)
	kflash -p $(K210-SERIALPORT) -b 1500000 $(KERNEL_BIN)
	python -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct $(K210-SERIALPORT) 115200

board-run: build flash