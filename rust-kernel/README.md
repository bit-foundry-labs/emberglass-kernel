# Embers Kernel M02: Rust-only Limine/QEMU entry

This is an original, freestanding Rust no_std x86-64 kernel entry, loaded
directly by **Limine**. Limine is an external bootloader; there is no
intermediate C kernel execution stage.

The former C bootstrap lives in the historical research branch PR #2.
The Rust build reuses a linker script and checksum-pinned Limine ISO
packaging machinery, but neither compiles nor links the C kernel.

## Fresh Ubuntu/WSL reproduction

Install rustup from https://rustup.rs and system prerequisites:

    sudo apt-get update
    sudo apt-get install -y make binutils python3 qemu-system-x86 xorriso curl tar
    cd rust-kernel
    rustup toolchain install 1.90.0 --component clippy,rustfmt --target x86_64-unknown-none
    cd ..
    make rust-check
    make rust-iso

Headless BIOS/QEMU:

    qemu-system-x86_64 -cpu max -m 256M -cdrom build/embers-rust.iso -display none -serial stdio -monitor none -no-reboot

Expected serial marker:

    READY: Embers Rust no_std entry reached.

The kernel intentionally stops with interrupts disabled in a halt loop.
Stop QEMU with Ctrl+C. This M02 boot profile uses "-cpu max" and does
not establish the minimum supported CPU baseline; see issue #9.

The ISO script downloads an explicitly pinned Limine v12.9.6 archive
and verifies its SHA-256 before extraction. No hosted model services
or paid APIs are involved.

## Safety limits

- Limine provides long mode, page tables and a stack. These are
  inherited, not independently controlled or proven by the Rust kernel.
- Privileged UART I/O, bootloader response reads, CLI/HLT require
  narrowly scoped unsafe Rust, with explicit entry assumptions.
- No processes, kernel-owned page tables, ring-3 isolation, IPC,
  capability enforcement, revocation, resource scheduler or DMA safety.
- The ELF script verifies structure; CI's QEMU READY assertion is
  separate execution evidence, not production hardware validation.

Next: GDT/TSS/IDT and fault injection (M03), then memory-map
validation and ownership ledgers (M04/M05).
