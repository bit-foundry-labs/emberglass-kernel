# ADR-0002: Native inference as capability-scoped user-space execution

- Status: **Proposed, not implemented**
- Date: 2026-11-08
- Builds on: [ADR-0001](0001-capability-microkernel-and-ai-policy.md)
- Source: [MIC/EBC blueprint](../notes/MIC_EBC_BLUEPRINT_2026-11-08.md)

## Decision proposed

Make inference a **native, capability-protected OS service**, not a model-specific kernel fast path.

The original Rust microkernel controls address spaces, thread scheduling, generic resource quotas, kernel handle/rights tables, IPC, protected memory grants, interrupts and later DMA isolation. User-space inference services manage model artifacts, weights, tensor formats, driver-specific queues and model selection. A user-space adaptive policy service proposes restricted updates; the kernel never waits for it to schedule ordinary CPU threads.

~~~text
Application / desktop / ForgeLoop agent
       |
       | session endpoint capability + budget capability
       v
USER SPACE: Inference broker and model registry
       |            |
       |            +-- Model digest, version, weights and formats
       |            +-- CPU/GPU/NPU backend / user-space driver
       |            +-- Async job queue and completion notification
       v
================ KERNEL TRUST BOUNDARY ================
RUST MICROKERNEL:
handles + rights | scheduling contexts | IPC | memory grants
quotas | page maps | interrupts | audit | deterministic fallback
=======================================================
       |
       v
CPU / DRAM / optional IOMMU-isolated GPU/NPU
~~~

## Capability contracts

### ModelInvocationCap (MIC)

Use MIC as an **application-facing service capability** implemented by a kernel-protected IPC endpoint plus a user-space session registry. It authorizes a narrow operation against a particular immutable model version, not direct ownership of model weights inside the kernel. Suggested fields enforced by the broker:

- artifact digest, version and provenance;
- allowed invocation operations (INVOKE, READ_RESULT, CANCEL), separated from administrative or model-mutation rights;
- input and output data permissions, policy on privacy and retention;
- permitted backends, expiry, per-session concurrency limit and invocation ID.

The kernel enforces access to IPC endpoints and shared memory; the user-space broker enforces model identity and semantic access. These distinct assurance levels must be documented.

### ExecutionBudgetCap (EBC)

Prefer a **generic kernel resource-budget object** for inference and non-inference workloads:

- CPU runtime budget and replenishment period, inspired by seL4 MCS;
- RAM: committed and pinned pages, shared-grant bytes;
- IPC: queue depth, outstanding requests, message size/rate;
- wall-clock deadline and cancellation state;
- accelerator-specific queue occupancy and submitted-work limits where supported;
- energy consumption limits only if trustworthy device counters and controls exist.

Do not promise arbitrary tensor-operation or energy caps at cycle-level precision without hardware support. Do not claim EBC alone eliminates all denial-of-service attacks. Some non-preemptible GPU jobs may require disruptive device reset.

### Supporting generic kernel handles

- MemoryGrantCap: buffer region, rights, owner, mapping, revocation generation and device sync state.
- ComputeDeviceCap: allowed backend device operations, query, submit, and highly restricted reset.
- PolicyUpdateCap: authority to propose changes to an explicitly named resource class within a safe range.
- ObservationCap: authorized access to rate-limited, redacted, opt-in telemetry.

The kernel should not need a Tensor, ModelWeights, Tokenizer, NeuralNetwork or model-specific execution type.

## Protocol skeleton (illustrative, not implemented)

~~~rust
struct InvocationRequest {
    session: Handle,
    input: Handle,
    output: Handle,
    budget: Handle,
    deadline_ns: u64,
    request_id: u64,
}

enum InvocationState {
    Queued,
    Admitted,
    Dispatched,
    CancelRequested,
    Completed,
    Failed,
    Revoked,
}

enum ResourceError {
    PermissionDenied,
    Expired,
    StaleGeneration,
    BudgetExceeded,
    UnsupportedBackend,
    DeviceUnavailable,
    QueueFull,
    Cancelled,
}
~~~

**Admission** checks valid sender authorization, model/session identity, shared-memory rights, remaining queue/memory budget, policy epoch and deadline before dispatch. The broker cannot silently expand a caller's rights or resource budget.

**Revocation** atomically prevents new authorizations. Already-running asynchronous device work may not stop immediately; safe memory reclamation requires completion/drain, hardware fence, or validated device reset. Generational IDs prevent stale handle reuse; do not free/reuse DMA-accessible pages before ownership returns.

## Determinism and fallback

- Kernel thread selection always has a standalone deterministic code path, never depending on external model inference.
- All policy requests are typed, privilege-checked, bounded, versioned, expiring and rate-limited.
- The kernel checks authorization and invariant preservation before applying a policy.
- Invalid, late, conflicting or stalled policy proposals leave a safe baseline in force.
- A policy process failure causes bounded recovery, **not a zero-cycle fallback**.
- Safety must include priority fairness, starvation controls, essential-service progress, memory safety and device fault recovery.
- Kernel handlers should have measured worst-case execution times for defined assumptions. There is no requirement that every kernel operation be O(1).

Deterministic **mechanisms** do not mean every real-world interrupt schedule is repeatable. Use controlled event traces and logs to recreate decisions for tests.

## Device and memory caveats

CPU, GPU and NPU differ in memory mappings, preemption, driver command queues, coherence, device faults and reset abilities. A shared pointer does not establish a uniform virtual address space. "Zero-copy" can reduce copies but still entails mapping, synchronization, ownership transfers, cache coherence and sometimes IOMMU/DMA work.

An NPU SRAM-only camera-to-model path bypassing RAM is **not** universally available. Confidential data isolation also depends on the driver/firmware/device and side channels; no general mathematical guarantee follows just from possession of MIC.

## Security assumptions and limits

Threats: forged handle; stale or revoked session; malicious policy changing own priority; cross-tenant model result access; queue flood; crashed broker; GPU hang; DMA to freed pages; shared-cache timing leakage.

Invariants to test:
1. Capabilities cannot amplify rights; revoked/expired handles authorize no new requests.
2. Cross-tenant inputs and outputs cannot be mapped without explicit grants.
3. RAM/queue/CPU resource budgets cannot be exceeded within the declared enforcement model.
4. AI process failure cannot block default kernel scheduling.
5. Delayed accelerator completion after revocation cannot cause use-after-free or unauthorized remapping.
6. Resource and policy transitions have stable audit identifiers and can be replayed under controlled events.

seL4 formal proofs are configuration- and assumption-specific, not inherited by Emberglass. They do not imply complete elimination of side channels or correct behavior of proprietary GPUs. Rust safe code helps memory safety, but unsafe drivers, assembly and system-wide liveness require independent evidence.

CHERI offers useful pointer-capability protection, but hardware availability, temporal safety and performance vary. It should be studied as a future hardware backend, **not** a requirement for the x86-64 QEMU prototype.

## First experiment: CPU-only simulated broker in Rust

**Purpose:** Verify our security and resource contracts without claiming real NPU support or requiring a trained model.

1. Host-buildable Rust model of handle tables with rights, generations and revocation; use unit/property tests.
2. Two isolated simulated tenants with separate invocation sessions, output memory grants and budgets.
3. Simulated CPU backend and configurable fake accelerator with queue latency, cancellation, timeout and failure injection.
4. Untrusted policy service able to propose only whitelisted resource settings; independent scheduler baseline.
5. Test unauthorized access, overbudget submission, forged handles, expired sessions, stale generations, queue flood, device hang, delayed completion post-revocation and malicious policy update.
6. Measure p50/p95/p99 admission latency, request throughput, max queue memory, false denials, fairness, and worst observed recovery time.
7. Port generic mechanisms into Rust no_std kernel only after interrupts, paging, ring-3 tasks and IPC are available.

**Go/no-go rule:** Do not add dedicated model object types to the kernel unless generic IPC, scheduling-context, memory-grant and capability objects demonstrably fail stated performance/isolation requirements.

## Primary sources

- seL4 MCS CPU scheduling contexts and time budgets: https://docs.sel4.systems/Tutorials/mcs.html
- Zircon process Jobs and policies: https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/docs/concepts/process/jobs.md
- Fuchsia user-space driver framework: https://fuchsia.googlesource.com/fuchsia.git/+/HEAD/docs/concepts/drivers/
- Linux DMA buffer synchronization: https://kernel.org/doc/html/next/driver-api/dma-buf.html
- Microsoft IOMMU GPU isolation: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/iommu-based-gpu-isolation
- Linux DRM GPU reset limitations: https://www.kernel.org/doc/html/latest/gpu/drm-uapi.html
- seL4 proof assumptions and timing channels: https://sel4.systems/Verification/assumptions.html
- CHERI RISC-V protection model: https://riscv.github.io/riscv-cheri/
- Morello performance limitations: https://ctsrd-cheri.github.io/morello-early-performance-results/performance-methodology/morello-microarchitectural-limitations.html
- PICASSO temporal safety research, USENIX Security 2026: https://www.usenix.org/conference/usenixsecurity26/presentation/guelmez

## Status

Architectural research only. It neither implements an AI-enabled kernel nor alters PR #2's verified C boot baseline. Review alongside ADR-0001 before adopting.
