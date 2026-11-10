# M03: Minimal x86-64 exception foundation

This experiment extends the Rust `no_std` Limine boot now on `main` via [PR #26](https://github.com/bit-foundry-labs/embers-kernel/pull/26). The original work was developed in [PR #21](https://github.com/bit-foundry-labs/embers-kernel/pull/21).
It is not a protected, multi-process or multicore microkernel.

## Privileged mechanism and safety contract

- A single bootstrap CPU installs a kernel-owned GDT with long-mode code/data
  segments, a 64-bit TSS descriptor, and an IDT before any test faults.
- The TSS provides a dedicated 16 KiB, 16-byte-aligned IST1 stack for double
  faults, independent of Limine's inherited boot stack.
- Vectors 6 (#UD), 8 (#DF), 13 (#GP), and 14 (#PF) are present as DPL0 interrupt
  gates. Other vectors remain absent; interrupts stay disabled.
- Nonreturning assembly stubs account for CPU-pushed error codes, realign the
  stack to the SysV x86-64 calling convention, then call one Rust diagnostic
  reporter. Never use these stubs for resumable interrupts or user-mode traps.
- The reporter emits fault vector, saved RIP, error code and CR2 for page faults
  over COM1, then stops the CPU. The serial writer has bounded polling.
- A #DF test intentionally removes the #PF gate before accessing a missing
  virtual address, provoking a failure while delivering another exception.
  This is a deliberate emulator-only fault-injection technique, not a safe
  runtime method.

## Reproduce with BIOS QEMU

From the repository root, using the pinned Rust toolchain and Limine release:

    make rust-iso
    make rust-run

For each negative test, recompile an ISO with the requested Cargo feature:

    make rust-iso RUST_FEATURES="--features fault-ud"
    make rust-iso RUST_FEATURES="--features fault-pf"
    make rust-iso RUST_FEATURES="--features fault-df"

Run QEMU after each build using the command in the parent Rust README. Expected
serial markers: FAULT: #UD, FAULT: #PF, and FAULT: #DF respectively, including
a saved RIP. The page fault also reports CR2. Restore normal build with
make rust-iso (no RUST_FEATURES value).

The M03 GitHub Actions check independently boots all four modes and retains
serial plus QEMU exception traces. A successful static compile alone is not
accepted as verified exception handling.

## Boundaries and next stage

The kernel still inherits Limine page tables and its ordinary execution
stack. No ring-3 transition, TSS RSP0 transition testing, syscall mechanism,
synchronous interrupt resumption, IRQ acknowledgment, physical memory
ownership, SMP coordination or DMA isolation exists. GDT/IDT/TSS table
installation is a prerequisite for those features, not isolation itself.

Before introducing unprivileged tasks, specify paging permission invariants,
fault ownership and a tested kernel/user trap frame ABI. Review the Intel
SDM Vol 3 and MIT xv6 trap-handling material.

References:
- https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
- https://pdos.csail.mit.edu/6.1810/2024/xv6/book-riscv-rev4.pdf
- https://github.com/bit-foundry-labs/embers-kernel/issues/19
