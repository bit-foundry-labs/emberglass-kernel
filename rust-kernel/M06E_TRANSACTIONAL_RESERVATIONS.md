# M06e: BSP transactional frame reservations

`check_boot_memory` initializes a `TransactionalFrameAllocator<N>` that owns both:

- the monotonic `FramePool` selected from Limine's validated map, and
- a fixed-capacity `FrameRegistry<N>` for reservation records.

The first normal boot diagnostic request reserves two pages through this path.
The reservation is written to the registry exactly once, and the same path
feeds optional M05 ownership smoke by lending the allocator's pool. The allocator
is returned to `kernel_entry`, which checks that the initial record is present.
The kernel then halts; future runtime code must explicitly carry this owner forward.
This is not a global allocator or a hardware ownership guarantee.

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
- No global singleton authority exists across independently created pools or
  registries, and no hardware-enforced isolation is claimed.
