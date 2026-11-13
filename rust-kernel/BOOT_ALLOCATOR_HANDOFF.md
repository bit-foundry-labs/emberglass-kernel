# Boot allocator handoff contract: proposal

Status: design only. No allocator is accessible to a future runtime yet.

- One bootstrap owner receives the validated pool and reservation registry
  together. The pool must not be reconstructed for the same physical range.
- The owner moves into the kernel runtime state before that runtime can allocate.
  No copied allocator, independent overlapping pool, or unaudited raw-pool
  access may issue frames.
- Table and transition-stack frames are reserved before use. Each reservation
  remains recorded for as long as a CPU might walk or access those frames.
- Initialization failure leaves no published partially initialized runtime.
  Reserved frames are not recycled merely because a handle or owner is dropped.
- Start with a single BSP and interrupts disabled. Before interrupts or multiple
  CPUs use this state, define synchronization, TLB invalidation, and quiescence.
- Device DMA, IOMMU ownership, reclaim, and real caller authentication require
  separate designs. No isolation or exclusive physical ownership is established.

Acceptance evidence for a future implementation: initialize the owner, move it
into runtime state, allocate another run through that same owner, inspect both
records, and show that a second issuer is rejected. Add failure-atomic tests and
QEMU observations; host tests alone do not establish hardware protection.
