# M06c: One-time early-boot frame-pool issuance

**Status:** BSP-only one-shot issuance gate; not persistent memory ownership.

`boot_memory::initial_frame_pool()` now consumes a static atomic issuance
permit before inspecting Limine's memory map. A repeated call returns
`BootMapError::AlreadyIssued` instead of recreating a second `FramePool`
overlapping the first. The permit is never reset, even when parsing fails.

Three host tests cover permanent rejection, independent gate state, and
exactly one winner under concurrent attempted issuance. QEMU smoke checks
continue to exercise first issuance and existing frame accounting.

## Scope and limitations

- This only gates calls through `boot_memory::initial_frame_pool()`.
  Pure `memory::choose_pool()` remains available for host tests and internal
  algorithms, and repeated calls to that pure function still create overlapping
  pool descriptors. Never treat those descriptors as globally unique.
- A one-shot initialization gate is **not** persistent ownership for allocated
  frames; it does not record mapping lifetimes, enforce CPU/DMA exclusivity,
  zero frames, or establish page-table authority.
- Existing `FramePool` is a local monotonic allocator; it cannot recycle
  frames. Kernel-wide future allocation must be routed through a single
  protected ownership source rather than independent pools.
- Atomic ordering ensures at most one caller wins the gate, but the surrounding
  Limine handoff remains BSP-only and is not a general SMP initialization API.

## Reproduce

```sh
make rust-boot-pool-gate-test
make rust-iso
make rust-run
```

Next: introduce an authoritative resident frame registry with explicit,
nonoverlapping reservations, bounded capacity, and no untracked bypasses,
before writable page-table construction or `CR3` changes.
