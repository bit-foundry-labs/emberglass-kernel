# Embers Rust-first x86-64 QEMU kernel. The legacy C prototype is not a build dependency.
RUST_KERNEL := rust-kernel/target/x86_64-unknown-none/release/embers-rust-kernel
RUST_FEATURES ?=

.PHONY: all check rust-check rust-iso rust-run clean
all: rust-check
check: rust-check

rust-check:
	cd rust-kernel && cargo build --release --offline $(RUST_FEATURES)
	python3 scripts/check_rust_elf.py $(RUST_KERNEL)

rust-iso: rust-check
	KERNEL_ELF="$(RUST_KERNEL)" ISO_KERNEL_NAME="embers.elf" BOOT_CONFIG="rust-kernel/limine.conf" ISO_OUT="build/embers-rust.iso" bash scripts/make_iso.sh

rust-run: rust-iso
	qemu-system-x86_64 -cpu max -m 256M -cdrom build/embers-rust.iso -serial stdio -no-reboot -no-shutdown

clean:
	rm -rf build
	cd rust-kernel && cargo clean
