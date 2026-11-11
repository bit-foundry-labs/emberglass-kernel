# M06a: Bounded contiguous frame reservations

**Scope:** Pure Rust, `no_std`-compatible bookkeeping in [`memory.rs`](src/memory.rs). No page-table writes, virtual mappings, frame clearing, DMA fences, or persistent address-space ownership are introduced.

## Why this is needed

A future kernel-owned x86-64 page table needs independently reserved physical frames for the root and intermediate tables. A caller must be able to request an exact number of frames without partially consuming a physical range on failure.

`FramePool::reserve_contiguous(pages)` now returns a `FrameRun` of contiguous physical frame numbers. It rejects zero-length, overflowing, and oversized requests before updating the monotonic pool cursor. The old `claim()` behavior is retained through the same reservation path, avoiding two allocation rules. The normal QEMU boot exercises a two-frame reservation and preserves the same serial markers expected by M04/M05 checks.

## Contract and limitations

- One unique `FramePool`, constructed from the validated Limine memory map, must be the **sole source** of reservations for its physical span. Separately constructed overlapping pools can still reserve duplicate addresses; this API is not global physical ownership.
- An accepted reservation consumes exactly the requested number of 4 KiB pages. Consecutive `address_at(index)` calls enumerate only the run's physical frame numbers and reject indexes outside its length.
- On zero pages, arithmetic overflow, or insufficient capacity, the pool remains unchanged. Existing claims and reservations cannot be returned to it; dropping `FrameRun` does not free anything.
- The immutable `FrameRun` is **metadata**, not a capability, hardware reservation, trusted virtual pointer, or guarantee that the frames are writable, zeroed, or protected from DMA.
- The current `FramePool` is local to early boot; this change **does not** make the accounting state persistent. A later implementation must establish an authoritative lifetime and enforce nonoverlapping physical ranges before constructing tables.

## How to verify

From the repository root in a development environment with the pinned Rust toolchain:

```sh
make rust-memory-test
make rust-ledger-test
make rust-iso
```

The five new host tests cover contiguous/disjoint addresses after ordinary claims, invalid zero-page requests, atomic exhaustion, arithmetic overflow, and permanent consumption after a reservation descriptor is dropped.

M04/M05 CI also runs the Rust build, host memory/ledger tests, and an optional QEMU ownership smoke run. M02/M03 CI provide ordinary READY and exception regressions. **A passing host test is not evidence of safe hardware page-table construction or isolation.**

## What comes next

Before using a reserved frame for page tables, discover a trusted physical-to-virtual mapping, reserve frames in a persistent owner registry, zero page-table memory through a validated writable mapping, and verify CPU paging mode/permissions. Only then attempt a feature-gated QEMU `CR3` transition as described in [the proposed M06 design](../docs/PAGING_TRANSITION_V1.md), which remains under review in PR #44.
