# M06d: Bounded nonoverlapping frame-range registry prototype

**Status:** Host-tested, no_std-compatible numeric bookkeeping, not kernel-integrated ownership.

The fixed-capacity `FrameRegistry<N>` records page-aligned physical ranges and
nonzero owner identifiers. It rejects overlap across all owners, zero-length
ranges, invalid alignment, arithmetic overflow, and capacity exhaustion.
Validation is performed before inserting an entry; no rejected call mutates
state. Entries remain recorded for the registry's lifetime, with no release
or reclamation operation.

The test harness exercises adjacent success, partial and full overlap,
invalid requests, failure atomicity, zero capacity, and no reuse.

## Security limits

- Registry constructors are independent. Two registries can contain identical
  physical addresses. No globally exclusive singleton is established.
- Input ranges are *not* checked against the actual Limine USABLE memory map.
  Caller identity and owner IDs are not authenticated.
- M06e adds one integrated BSP boot path, but bypasses still exist: direct
  `FramePool` calls, independently constructed registries, and the optional
  M05 ledger smoke path that consumes a pool without consulting the registry.
- This is one-writer metadata. No synchronization, allocator lifetime
  contract, hardware page permissions, zeroing, DMA/IOMMU quiescence, or
  reclaim-after-revocation proof is provided.

Do not use registry membership as permission to dereference a physical
address or switch CR3.

## Reproduce

```sh
make rust-frame-registry-test
cd rust-kernel && rustfmt --check src/frame_registry.rs
```

The next increment must integrate a *single resident authority source* into
the boot allocator, make bypass allocation impossible, and prove reservation
accounting stays consistent on failed requests. Only then construct or zero
page-table frames under the separate M06 paging transition contract.
