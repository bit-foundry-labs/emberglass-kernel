# M05: Fixed-capacity frame ownership ledger

**Integration:** port of the M05 research from [PR #23](https://github.com/bit-foundry-labs/embers-kernel/pull/23) onto the Rust-only `main` history. M04 was integrated by [PR #28](https://github.com/bit-foundry-labs/embers-kernel/pull/28).

M04 validates Limine's physical memory map and selects a USABLE span.
M05 adds a **single-writer, bounded, testable accounting state machine**.
It does not create a user-mode or device-enforced capability.

## State transitions and invariants

Each slot begins Available and can move to Owned(owner) and then
Quarantined. The transitions are strictly one-way.

| Transition | Requirement | Result |
| --- | --- | --- |
| Available -> Owned | nonzero OwnerId, mutable ledger, free slot | unique frame token |
| Owned -> Quarantined | same owner, matching slot and generation | no future admission |
| Quarantined -> Available | **not supported** | no reuse or premature reclamation |

- The ledger accepts 1..=32 frames from M04's monotonic FramePool.
  It checks the source capacity before consuming any physical addresses.
- Each claim chooses the first available slot deterministically.
- FrameToken carries a private ledger identity, slot index, generation, and
  owner. The identity is derived from the first physical frame assigned by
  the one monotonic FramePool. Cross-ledger tokens from that pool are rejected.
  **These are accounting identifiers, not unforgeable capabilities**.
- Multiple independently constructed FramePools that cover overlapping
  physical ranges can generate the same ledger identity. A future authoritative
  physical-memory allocator must prevent overlapping ownership; this model
  does not establish global uniqueness by itself.
- Every address lookup and quarantine transition checks the ledger identity,
  owner, slot bounds, generation and current state.
- Exhaustion, invalid IDs, forged tokens, cross-owner requests, repeat
  quarantine and generation overflow return typed errors without
  silently allocating or recycling a page.
- Neither the ledger nor its tokens access physical memory. There is
  no implicit mapping, zero-fill, or free operation.

M05 takes the deliberately conservative route of **permanent quarantine**:
no CPU mapping audit, device fences, DMA draining, TLB shootdown or IOMMU
quiescence proof is available. It would be unsound to expose a public
`release` or `reclaim` method at this stage.

## What is tested

Run on Ubuntu/WSL with the pinned Rust toolchain:

    make rust-memory-test
    make rust-ledger-test
    make rust-iso RUST_FEATURES="--features boot-ownership-smoke"
    make rust-run RUST_FEATURES="--features boot-ownership-smoke"

Host tests run the pure-Rust modules in a normal userspace test harness;
the privileged kernel module itself remains `no_std` and allocation-free.
Existing M04 negative-path tests run alongside M05 tests.

The optional `boot-ownership-smoke` feature is the only path that runs the
BSP ledger demonstration. Normal boots retain memory-map validation without
executing ownership test scenarios. Under this feature, the QEMU demo creates
an eight-slot ledger and exercises:
- Two distinct owners with distinct physical frame addresses.
- Rejection of an unauthorized cross-owner address lookup.
- Permanent quarantine of a logically invalidated frame.
- Normal Rust READY after bounded checks.

The QEMU test inspects exact serial markers and the reported frame
address, with the earlier M02/M03/M04 workflows providing regressions.

## Safety limitations and roadmap

The ledger is local to the early boot smoke-test, not persistent kernel
runtime state. Limine supplies trusted memory descriptors and page tables.
Owners are symbolic IDs; no per-process caller authentication, page-table
ownership, ring-3 permissions, DMA isolation, multicore synchronization,
or hardware authority is implemented. Preventing a return from a ledger
lookup does **not** prevent memory access through already-existing mappings.

Next steps: define frame reservation and physical-address provenance,
establish kernel-owned page tables with supervisor-only mappings, then
introduce an authenticated caller model before capability-mediated
delegation and carefully evidenced reclamation.

TheAlgorithms/Rust was reviewed for educational algorithm ideas. No
third-party implementation code was incorporated into this change.
