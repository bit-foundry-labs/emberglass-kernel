# M04: Trusted boot map to bounded physical-frame ledger

**Scope:** BSP-only bookkeeping, with no dynamic allocation, unsafe frame
access, page-table edits, DMA handling, or privileged user-space interface.
This work is stacked on M03 exception handling, not yet merged into main.

## Invariants implemented in pure Rust

- All firmware regions are validated, including reserved types, before
  selecting any usable physical span.
- Zero-length, overlapping, and u64-overflowing ranges reject the entire map.
  There is no silent repair of conflicting firmware claims.
- The candidate frame span must be Limine type USABLE, page aligned inward,
  above physical 1 MiB, and contain at least 16 pages.
- Selection always chooses the lowest suitable physical start, regardless of
  source order. Only one span is used in this milestone.
- A mutable, noncloneable FramePool hands out each aligned page once, without
  wraparound or reuse. Exhaustion returns None without modifying state.

## Limine trust boundary

A Limine base-revision-6 memory-map request carries response/entry pointers.
The kernel explicitly assumes these are valid, stable, mapped, aligned,
bootloader-owned pointers throughout early single-core boot. We **bound
entry_count to 128 before walking pointers** and copy the raw region
descriptors into fixed-size Rust stack storage before validation. These bounds
do not authenticate the bootloader or prove its pointer integrity.

Selected frame addresses are reserved only in an ephemeral Rust ledger.
The code does **not** write to those pages, clear them, acquire exclusive
CPU/IOMMU authority, establish mappings, or prove they are free of DMA.
Do not describe M04 as page-table ownership or process isolation.

## Reproduction and evidence

From the repository root on Ubuntu/WSL with the pinned Rust toolchain:

    make rust-memory-test
    make rust-iso
    make rust-run

Host tests in rust-kernel/src/memory.rs cover malformed inputs, address
overflow, reserved-region overlap, alignment, order independence, exhaustion,
and lack of duplicate claims. A separate QEMU job verifies normal Rust READY
plus two distinct consecutive aligned frame addresses reported over COM1.

CI: .github/workflows/rust-memory.yml

## Next review gate

Before enabling dynamic frame allocation in long-lived kernel state, decide
where the single writer lives, what maps are trusted after exiting bootloader
services, how reference counts/ownership are tracked, and how to reserve
new page-table frames before switching CR3. The existing Limine mappings
remain an inherited trusted boot dependency in M04.

Reference: https://github.com/limine-bootloader/limine-protocol
