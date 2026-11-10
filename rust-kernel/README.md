# Embers Rust boot experiment: M02

A standalone original Rust no_std x86-64 entry point, stacked on
[C Limine/QEMU PR #2](https://github.com/bit-foundry-labs/embers-kernel/pull/2).
The C sources and their normal make targets stay available. Do not boot on hardware.

## Reproduce (Ubuntu / WSL2 Ubuntu)

Install GCC, make, binutils, Python 3, curl, tar, QEMU, and xorriso as in the C
reference README. Install rustup from https://rustup.rs.

Run from the repository root:

    cd rust-kernel
    rustup toolchain install 1.90.0 --component clippy,rustfmt --target x86_64-unknown-none
    cd ..
    make rust-check
    make rust-iso
    make rust-run

The ELF inspection is a static structural check, not evidence of execution.
The rust-iso target reuses the checksum-pinned Limine v12.9.6 release,
writing build/embers-rust.iso. To boot without a display:

    qemu-system-x86_64 -cpu max -m 256M -cdrom build/embers-rust.iso -display none -serial stdio -monitor none -no-reboot

Expected serial line:

    READY: Embers Rust no_std entry reached.

The separate GitHub Actions workflow checks rustfmt, Clippy, the ELF layout,
and a headless QEMU boot with preserved serial and exception logs.

## Security boundary and unsafe inventory

- Limine supplies long-mode execution, higher-half mappings and the entry
  stack; this kernel does not independently own or validate these resources.
- The reused C linker script retains the base revision 6 Limine markers.
- Unsafe sites are two privileged UART port-I/O wrappers, one volatile
  bootloader revision read, and cli/hlt for controlled idle. Each assumes
  privileged x86-64 execution and a legacy COM1 UART in QEMU.
- Each UART byte has a fixed polling bound; truncation makes QEMU CI fail.
- Not implemented: GDT/IDT/TSS ownership, exception handlers, page-table
  isolation, ring-3, syscalls, capability tables, safe revocation, DMA
  isolation, scheduling, hardware ownership, or formal assurance.
- The x86_64-unknown-none target disables vector and floating-point registers
  and red zones by default. The temporary QEMU -cpu max choice is NOT a
  minimum CPU baseline; investigate separately under Issue #9.

References:
- https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- https://github.com/limine-bootloader/limine-protocol/blob/trunk/PROTOCOL.md
- https://github.com/bit-foundry-labs/embers-kernel/issues/19
