# M06e: BSP transactional frame reservations

`check_boot_memory` initializes a `TransactionalFrameAllocator<N>` that owns both:

- the monotonic `FramePool` selected from Limine's validated map, and
- a fixed-capacity `FrameRegistry<N>` for reservation records.

The first normal boot diagnostic request reserves two pages through this path.
The reservation is written to the registry exactly once, and the same path
feeds optional M05 ownership smoke by lending the allocator's pool. The allocator
is returned to `kernel_entry` and moved into `KernelState`, which is passed to
`run_kernel`. The initial record is checked after this handoff. The kernel then
halts; future execution must continue to retain and pass this state. This is not
a global allocator, a synchronization contract, or hardware ownership.

Production pool selection requires a non-copyable, one-shot permit owned by the
memory module. The permit is consumed before boot-map parsing, including on
failure; no retry is permitted. The unrestricted selector exists in test builds
only, where independent pools deliberately demonstrate the old bypass. The
standalone `boot_pool_gate` host tests do not exercise the production permit.
This restricts safe construction in one kernel image, not arbitrary unsafe code,
CPU or device mappings, or actual physical access.

## Failure atomicity contract

`TransactionalFrameAllocator::reserve` stages both checks before insertion:

1. preview the pool run without advancing its cursor;
2. validate owner/range/overlap/capacity in the registry without insertion;
3. commit pool advancement and then commit the already-validated registry entry.

If the request is rejected for zero pages, arithmetic overflow, insufficient
frames, invalid owner/start, overlap, or registry capacity, both pool
availability and registry count remain unchanged.

## Remaining bypass inventory

- `FramePool::reserve_contiguous` can still be called directly by code that
  does not route through `TransactionalFrameAllocator`.
- `FrameRegistry::reserve` remains directly callable and can be instantiated
  independently from the boot allocator.
- `FrameLedger` (M05 smoke feature) still allocates from a raw `FramePool`
  reference and does not consult registry records.
- The permit limits safe production pool construction in this kernel image; it
  does not make registries globally unique or prevent forged values or unsafe
  physical access. There is no hardware-enforced isolation.
