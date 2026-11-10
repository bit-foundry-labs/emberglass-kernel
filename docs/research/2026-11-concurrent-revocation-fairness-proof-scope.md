# Research Review 06: Concurrent revocation, fairness and proof boundaries

**Date:** 2026-11-08
**Status:** Research proposal, not kernel-implemented or formally proved.
**Project:** Embers Kernel, original Rust capability-microkernel research (Bit Foundry Labs)
**Context:** ADR-0001/0002/0003, draft [capability-table PR #11](https://github.com/bit-foundry-labs/embers-kernel/pull/11), independent [threat model PR #13](https://github.com/bit-foundry-labs/embers-kernel/pull/13).

## User-supplied research hypotheses

1. Revocation must defend against TOCTOU between cores, stale CPU/IOMMU/device translations, and stale capability slot reuse; consider a Capability Derivation Tree and hardware cache synchronization.
2. AI-driven scheduling must preserve hard CPU resource quotas and service for other workloads; examine accelerator virtualization, minimum progress guarantees, and priority inversion when CPU tasks wait for accelerator work.
3. Map property classes to testing, finite-state model checking (TLA+/SPIN), and implementation-level formal proofs (Isabelle/HOL, Coq), recording assumptions and tool limits.

These are sound research questions, but **not** justification for claims of instant and absolute cross-device revocation, universal SR-IOV accelerator compute QoS, or mathematical verification by ordinary Rust typing.

## Decision A: separate revocation semantics

Two distinct API contracts are required:

**Logical revocation.** After an explicit linearization point, no new operation can be successfully admitted under the revoked capability or any affected descendant. In-flight uses are tracked, not silently assumed to have stopped.

**Resource quiescence.** Existing operations have exited, been safely cancelled, or their relevant memory access has been fenced/invalidated; only then may the kernel reclaim/reuse physical memory or device mappings. For DMA this may require a completion fence, device-specific cancellation, IOMMU/device TLB invalidation and/or reset. A hung non-preemptible device may prevent reclaiming buffers; fail closed (retain/quarantine pages), never reassign potentially active memory.

Proposed request state machine:

~~~text
Active
  | revoke begins: close admission gate / increment revocation epoch
  v
Revoking (no new admissions; in-flight requests remain known)
  | CPU operations complete or are cancelled
  v
Draining
  | CPU mapping invalidation and device-specific DMA fence/teardown
  v
Quiesced
  | resources safely reclaimed / handle slots may be reused
  v
Revoked

Failure path: device cannot prove quiescence -> Quarantined / no memory reuse.
~~~

The apparent order of operations is illustrative: an implementation must avoid gaps between admissions, tracking in-flight uses, page-table updates, and hardware invalidation. Non-memory capability revocation may finish without any TLB shootdown. Avoid indiscriminate global cache flushes.

**An unconditional lock-free CDT is not recommended for M01.** Begin with a small protected table and an explicit global lock or equivalent serialized transaction model; benchmark its cost before considering per-object locking, epochs/RCU, or lock-free derivation structures. Lock-free does not automatically mean low-latency or easily verifiable.

## Decision B: specify operations' linearization points

- Authorize a request only after deriving the authenticated caller from current thread/IPC context and looking up a protected, generation-tagged handle.
- Commit admission *atomically* with a hold/lease over resource rights, or revalidate the generation and epoch at the exact actuation point under a suitable synchronization mechanism.
- Revoke first prevents new admissions; it does not automatically undo all accepted work.
- Use monotonic, non-reusable generations (retire exhausted slots), and avoid returning a handle that aliases a retired slot.
- A revoke operation must account for all descendants, but physical erasure need not happen in one unpreemptible global critical section; a multi-phase protocol may bound response latency.
- A stale TLB/IOTLB is relevant when a *memory mapping* is revoked, not for every logical capability operation.
- Explicitly model a caller with a valid cached AuthorizedPolicy snapshot racing with revocation. A safe type alone is not proof against this TOCTOU race.

**Draft PR #11 gaps:** Host-testable CapTable, generational handles, and owner checks are useful modeling mechanisms, but caller `SpaceId` is supplied by the test harness; `mint_root` is public at the research library boundary (kernel-only by intent); and revoked memory is not mapped to hardware. PR #17 proposes an alternative explicit policy-binding API, but neither branch implements trusted kernel authorization. There is no atomic validation-to-hardware-application contract. Cross-session policy replay interference was previously reported in the PR. No protection guarantee carries into a real kernel before user/kernel address-space separation and syscall caller authentication exist.

## Decision C: resource fairness is not synonymous with capped AI priority

- A scheduler with valid priority values can still starve a lower-priority task.
- CPU budgets limit maximum runtime, but guaranteed *minimum service* requires capacity/admission analysis, scheduling discipline, task runnable/blocked assumptions, timer preemption and treatment of interrupts/critical sections.
- On one processor, a simplified necessary condition is sum of guaranteed minimum shares <= available CPU time after kernel reserve. For multicore and real-time deadlines this is not sufficient.
- Priority inheritance reduces unbounded lock inversion only under specified rules. If an AI task holds a lock needed by a real-time process, **do not increase its CPU budget without charge**; consider authorized donation/budget inheritance from the blocked task or a ceiling protocol, with bounded chains and deadlock analysis.
- CPU and GPU queue priorities are not automatically coordinated: a GPU that cannot preempt commands cannot deliver hard response-time bounds simply because the CPU boosted a submitter thread.
- AI-driven changes to fixed kernel reservations must never let a policy agent raise its own maximum allocation or decrease protected critical-service reservations.

**Hardware claim correction:** SR-IOV primarily provides virtual functions and often device/IOMMU isolation. It does **not universally imply dedicated fractions of GPU/NPU compute units or guaranteed minimum accelerator execution progress**. Vendor/device-specific mechanisms such as NVIDIA MIG can provide hardware-partitioned compute/memory QoS on *supported* GPUs. Inference hardware must be evaluated separately.

## Decision D: verification strategy and what each proves

| Method | Best target | Proof/evidence scope | Missing |
| --- | --- | --- | --- |
| Rust unit tests + exhaustive finite input domain | Pure validators, bounds, state transitions | Observed checks for enumerated cases | Unenumerated interleavings, missing specification |
| Adversarial/property-based tests | Random handles, generations, delegation trees and quotas | Repeatable counterexamples in concrete Rust program | Full state space, hardware |
| TLA+ / TLC finite-state model | Authorization/revocation/admission/interleavings | Safety invariants and selected liveness properties within specified finite state space and fairness assumptions | Conformance of Rust implementation to model |
| Kani bounded model checking | Rust handle/generation invariants and arithmetic | Bounded implementation-level assertions, tool-supported semantics | General concurrency, hardware execution, unbounded behavior |
| Verus / Coq / Isabelle | Focused functional contracts, abstraction refinement | Scope explicitly stated in formal specification and tool proof | Assumed hardware, kernel assembly, device firmware, unverified drivers |

TLC supports finite-state exploration and temporal liveness checking, but liveness checks need fairness assumptions and suitable temporal specifications. General statements that model checking proves freedom from all deadlocks, infinite interleavings or real hardware races are incorrect.

seL4 proof assumptions are configuration-specific; hardware and concurrency proof scopes must be evaluated precisely before drawing comparisons. These sources are architectural references, not proof of Embers Kernel.

## First model-checking experiment

A tiny, reproducible two-core model with:
- two process spaces (A/B), two generation-tagged handles, an optional child capability;
- actions MintRoot, Delegate, Authorize, CommitUse, BeginRevoke, FinishUse, Reclaim, ReuseSlot;
- a separate InFlight count and revocation state;
- a map-permission state and simulated device job that can complete or hang;
- explicit state transitions for a policy server crash and fallback.

**Safety invariants:** no successful new admission after logical revoke; no rights amplification on delegation; stale generation never authorizes a different object; no reclaim before zero in-flight users and device quiescence; caller bound to protected space; sum of admitted CPU reservations does not exceed capacity.

**Liveness (conditional):** under weak fairness of the service manager and guaranteed completion/cancellation of jobs, revocation eventually quiesces. Without a guaranteed device completion/reset mechanism, no unconditional liveness proof is possible. Keep the quarantine state safe.

**Counterexample to seek:** A authorizes handle H -> B revokes H -> A commits operation from cached validated snapshot. Another counterexample: A/B share a global PolicyGate replay epoch and same-epoch B is rejected after A's successful update.

## When ready to implement

1. Resolve PR #11 code review and GitHub Actions approval; no merge while critical replay/TOCTOU constraints unresolved.
2. Merge or review PR #13's threat model independently.
3. Model-check abstract revoke/admission/commit actions (single lock first), then encode same state transitions in Rust tests.
4. Introduce protected user-space IPC/capability access into real Rust kernel after paging and caller identity exist.
5. Only later design IOMMU/DMA mapping revocation, with a target device, hardware specification and measured fence/reset behavior.

## Primary sources

- seL4 CNode Revoke interface: https://docs.sel4.systems/projects/sel4/api-doc.html
- seL4 multicore big-lock reference and unverified SMP proof status: https://docs.sel4.systems/projects/sel4/frequently-asked-questions and https://github.com/seL4/seL4/blob/master/CAVEATS.md
- seL4 proof hardware, DMA and side-channel assumptions: https://sel4.systems/Verification/assumptions.html
- Linux mmu_notifier secondary TLB invalidation: https://www.kernel.org/doc/html/next/mm/mmu_notifier.html
- Linux DMA completion before unmap: https://www.kernel.org/doc/html/latest/core-api/dma-api-howto.html
- SR-IOV isolation: https://learn.microsoft.com/en-us/windows-hardware/drivers/network/single-root-i-o-virtualization--sr-iov--interface
- NVIDIA MIG supported-device partitioned GPU memory/compute: https://docs.nvidia.com/datacenter/tesla/mig-user-guide/introduction.html
- seL4 MCS temporal budgets: https://docs.sel4.systems/Tutorials/mcs.html
- TLA+ TLC checker for finite models: https://docs.tlapl.us/using:tlc:start
- TLC safety vs temporal liveness properties: https://learning.tlapl.us/intro/tlc-config/
- Kani: https://model-checking.github.io/kani/
