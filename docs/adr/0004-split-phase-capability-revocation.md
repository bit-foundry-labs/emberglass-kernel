# ADR-0004: Split-phase capability revocation with explicit quiescence

**Date:** 2026-11-08  
**Status:** Proposed; not implemented in the bootable kernel or formally proved  
**Related:** [ADR-0002](0002-native-inference-capability-contract.md), [ADR-0003](0003-ai-policy-validation-and-admission.md), [Research Review 06](../research/2026-11-concurrent-revocation-fairness-proof-scope.md), [TLA+ research task #14](https://github.com/bit-foundry-labs/embers-kernel/issues/14), draft [PR #11](https://github.com/bit-foundry-labs/embers-kernel/pull/11).

## Decision proposal

Evaluate **split-phase capability revocation** as the target protocol for Embers Kernel. Distinguish three contracts, all explicitly scoped:

1. **Close admission:** atomically mark the capability and its derivation scope as no longer authorizing *new* operations at a linearization point, under a protected kernel synchronization mechanism.
2. **Drain previously admitted work:** let tracked operations complete/cancel safely, without permitting those operations to regain revoked authorization or to allocate additional rights *from the revoked capability*.
3. **Quiesce and reclaim:** release physical pages, device mappings, and handle slots only after relevant CPU readers, accelerator/DMA operations and required translation invalidations are finished. Otherwise quarantine affected resources.

The fast admission path should not wait for AI inference or device completion. The slow reclamation path may be asynchronous and device-specific. **No guarantee of zero latency, lock-free progress, constant-time revocation, microsecond drain, or per-context reset is made.**

## Why this is needed

A handle-generation check solves stale handle reuse but **not** use of an already authorized, copied policy snapshot after revocation, nor pending device DMA to reclaimed pages. CPU epoch-based reclamation can protect participating CPU readers, but a PCIe device will not automatically participate in a CPU grace period.

The draft, host-only Rust CapTable ([PR #11](https://github.com/bit-foundry-labs/embers-kernel/pull/11)) models rights, generations, expiry, and eager descendant invalidation. It does not yet model admission-hold tokens, in-flight operation tracking, physical buffer ownership, device fences, multicore ordering, or a safe handle-table/lifetime synchronization protocol. Do not represent the current model as implementing this ADR.

## State machine

~~~text
         Admission may succeed       no new admissions
     +-------------------------+    +----------------------------------+
     |          ACTIVE         | -> | CLOSING / LOGICALLY REVOKED      |
     +-------------------------+    +----------------+-----------------+
                                                     |
                                          active holds complete
                                                     v
                                      +--------------+------------+
                                      |            DRAINING       |
                                      +--------------+------------+
                                                     |
                                CPU readers ended, device fence
                                completed, required TLB/IOTLB and
                                device translation invalidations ACKed
                                                     v
                                      +--------------+------------+
                                      |           QUIESCED        |
                                      +--------------+------------+
                                                     |
                                        safe reclamation/reuse
                                                     v
                                      +--------------+------------+
                                      |           RECLAIMED       |
                                      +---------------------------+

     When a hardware job hangs or device quiescence cannot be established:
                 DRAINING  --->  QUARANTINED (retain pages; no reuse)
     A later proven device-specific reset may allow recovery and quiescence.
~~~

In an implementation, "CLOSING" should already block new admission at its stated linearization point. "DRAINING" is a control state for asynchronous work; its transition order can be combined with logical revoke when needed. A successful logical revoke response is **not** a successful physical reclaim response.

### Admission and revoke race contract

**Proposed simplified reference protocol (NOT production synchronization code):**

~~~text
try_admit(handle, current_caller):
    within one kernel-serialized critical section:
        check caller's protected space, slot generation, rights, limits and lineage
        check state == ACTIVE
        increment tracked active-holds (with overflow handling)
    return AdmissionHold(object_id, generation, scoped_rights)

begin_revoke(handle, authorized_caller):
    within same synchronization order:
        mark derivation scope CLOSING / deny further admission
        increment revocation version or invalidate generation for future lookup
        enqueue asynchronous drain
    return RevokeTicket(version)  # "no new admissions", not "DMA stopped"

finish_use(hold):
    release hold exactly once
    decrement in-flight holds safely
    wake drain worker if last active hold completes

quiesce(ticket):
    wait asynchronously for holds/fences to finish
    perform mapping/device translation invalidations with required ordering
    if all required completion conditions are proven: state = QUIESCED
    else: state = QUARANTINED (no physical memory reassignment)

reclaim(ticket):
    require state == QUIESCED
    release objects, mappings, and generations according to safe-reuse rules
~~~

**Important:** An admission hold does not authorize unlimited new work, create new child capabilities, or dynamically expand memory grants. Any new authority-producing action must perform its own fresh authorization under the current revocation state. Holds are object-bound and have lifetime/accounting rules.

A global serialization lock can be a correct *initial model*; it may briefly delay unrelated callers. Our objective is low and bounded measured interference, not "unaffected" workloads as an absolute hardware property. Test sharded tables and RCU/epochs later if contention warrants the complexity.

## Device-specific quiescence

An IOMMU provides a mapping/isolation mechanism, not a universal rule that cached device transactions stop when an application handle is revoked. Some systems support ATS/PASID and targeted invalidation, but safe teardown may require stopping DMA, invalidating relevant CPU/device/IOMMU caches and waiting for invalidation completion in an architecture-defined order.

- **CPU access:** synchronize all existing uses before releasing pages; TLB shootdowns when page mappings actually change. A revoked logical policy capability with no changed page mapping does not require a TLB flush.
- **DMA accelerator:** wait for a trusted completion fence, proper cancellation or hardware-supported reset; verify no further device writes are possible before reuse. Distinguish driver-signaled failed fence from proof that DMA is physically quiesced.
- **Hung device:** if a guaranteed safe reset/quiescence mechanism is not available, fail closed: quarantine pinned pages and reject new submissions. Account this memory against a bounded quota; report loss of availability.
- **Virtualized devices:** vendor/device-specific partitioning may permit localized reset, but SR-IOV virtual functions, independent queues, PASID, or MIG alone do not universally guarantee that resetting one job or VF has no effect on others.
- **Epochs:** CPU RCU/SRCU grace periods do not imply DMA completion, and an endlessly executing CPU reader can prevent a grace period completing. Do not promise bounded asynchronous reclamation without liveness assumptions.

## Proposed testable properties

**Safety (unconditional within the modeled atomic protocol):**

- S1: A new admission with a revoked generation or non-ACTIVE derivation scope fails.
- S2: Successful admission occurs before logical-revoke linearization, and increments the in-flight count exactly once.
- S3: Reclaimed memory has zero tracked CPU holds and no outstanding device access; mapping invalidation acknowledgments have been observed.
- S4: No descendant can mint/delegate new rights after its ancestor enters logical revoke.
- S5: Recycled handle slots never authorize operations under stale generations.
- S6: Quarantined buffers are not reused or exposed to another security domain.
- S7: Other tenants' independent capability trees need not be revoked by a local revoke.

**Liveness (conditional):**
- L1: With finite in-flight jobs, fair scheduling and a working device completion/cancellation mechanism, a revoke ticket eventually reaches QUIESCED.
- L2: With a hung non-preemptible device and no safe reset, **no unconditional progress guarantee** exists. Quarantine is safe but is not success.
- L3: Bound new queue admissions and quarantined page consumption per tenant to avoid unbounded denial of service.

**Performance hypotheses, to measure rather than promise:** p99 authorization latency during concurrent revoke; p99 unrelated tenant latency; maximum lock hold; memory held in quarantine; time to logical-close, CPU-drain and device-quiescence separately.

## Research sequence

1. Review the finite-state TLA+ experiment in [PR #15](https://github.com/bit-foundry-labs/embers-kernel/pull/15), including its intentionally unsafe counterexample, model assumptions, and reachable-state limits. A successful model check is not proof of the Rust implementation.
2. Write safety invariants and conditional liveness properties, with model checker configs and reproducible traces.
3. In host-tested Rust, add a simulated AdmissionHold / RevokeTicket / Quarantine API, guard state transitions, and negative tests for delayed device completion and revocation while policy snapshot is cached.
4. Only after protected address spaces, trusted syscall caller identity, and kernel IPC exist, port the mechanism to original Rust kernel.
5. Select a *real target accelerator* before any hardware-specific PASID, SMMU, DMA-fence or context-reset claims.

Do **not** attempt a lock-free CDT before demonstrating correctness and measuring the contention of a simple serialized baseline.

## Source audit and references

- Linux RCU, removal vs delayed reclaim: https://docs.kernel.org/next/RCU/whatisRCU.html
- Linux RCU forward-progress limitations: https://docs.kernel.org/RCU/Design/Requirements/Requirements.html
- Linux shared virtual addressing, PASID, ATS, PRI: https://docs.kernel.org/next/arch/x86/sva.html
- Linux IOMMUFD targeted invalidation interface: https://docs.kernel.org/userspace-api/iommufd.html
- Linux mmu_notifier secondary TLB synchronization: https://cdn.kernel.org/doc/html/latest/mm/mmu_notifier.html
- Linux dma-buf fences and reset best efforts: https://docs.kernel.org/driver-api/dma-buf.html
- NVIDIA MIG reset and system interactions: https://docs.nvidia.com/datacenter/tesla/mig-user-guide/getting-started-with-mig.html
- seL4 capability revoke: https://docs.sel4.systems/projects/sel4/api-doc.html

## Status

This is an architecture **proposal**. No user-facing native AI inference, bounded hardware quiescence, kernel-enforced epoch reclamation or proof of noninterference has been implemented by creating this ADR.
