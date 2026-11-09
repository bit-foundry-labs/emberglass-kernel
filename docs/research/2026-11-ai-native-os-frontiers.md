# Emberglass Research Frontier 03: AI-native resource management

**Date:** 2026-11-08
**Project:** Bit Foundry Labs / Emberglass original kernel
**Status:** Research brief; proposed experiments; does not modify the kernel implementation.
**Inputs:** [Five-domain source brief](../notes/AI_NATIVE_OS_FRONTIER_BRIEF_2026-11-08.md), [ADR-0001](../adr/0001-capability-microkernel-and-ai-policy.md)

## Executive finding

The frontier is NOT simply "run a neural model inside every kernel fast path." The strongest practical research pattern is **fast deterministic data plane + asynchronous adaptive control plane + kernel-enforced safety contract**.

Hypothesis: A Rust `no_std` capability microkernel can make heterogeneous compute and model sessions **first-class managed resource objects** without placing model execution or nondeterministic arbitration in privileged memory. That lets us offer AI-native OS functionality while preserving an auditable trusted computing base.

This brief validates research threads and challenges overconfident performance claims. The original independent C x86-64/ Limine kernel on PR #2 achieved serial `READY` in BIOS-based QEMU with `-cpu max`; it does **not** yet implement a microkernel, Rust userspace, drivers, NPU support, or AI.

## Proposed conceptual architecture

```text
Apps / compositor / ForgeLoop-like task agents
       | requested inference sessions / resource preferences
       v
Model registry + execution broker (user mode; per-app caps / quotas)
       | maps models to CPU / GPU / NPU / optional neuromorphic device
       v
Resource policy service (user mode)
       | asynchronous proposals only; bounded rate and validity interval
       v
+----------------- RUST KERNEL TRUST BOUNDARY ----------------+
| Task scheduling | page-table isolation | interrupts/timers |
| IPC + shared memory grants | capability tables | quotas    |
| deterministic fallback | validation | audit event stream |
+------------------------------------------------------------+
       | device isolation via driver service / IOMMU later
       v
CPU | DRAM | GPU | NPU | storage / hardware DMA
```

First-class *resource* objects, not privileged model execution:
- `ComputeDevice`: device identity/features, queue ownership and limits.
- `ModelArtifact`: immutable content ID, version, signature/digest and provenance; stored in a user-space model registry.
- `InferenceSession`: caller capability, backend selection, memory/CPU/energy budget, expiry.
- `MemoryGrant`: scoped shared memory region, owner, access rights and revocation state.
- `PolicyProposal`: opaque policy ID, trusted target, typed value, validity interval, source provenance and sequence number.
- `ObservationStream`: opt-in, rate-limited local performance counters, retention and privacy policy.
The kernel should know the capabilities and resource accounting it needs to enforce, not neural network internals.

## Research assessment by domain

### 1. Heterogeneous asynchronous scheduling and zero-copy IPC

**Strong foundation:** Google/Stanford's *ghOSt* (SOSP 2021) delegated Linux scheduling decisions to user-space agents, separating rapidly evolving policy from kernel mechanisms. It is **not** a Microsoft Research "Project Ghost." The ghOSt design suggests a versioned transaction scheme and stale-decision checks. https://research.google/pubs/ghost-fast-and-flexible-user-space-delegation-of-linux-scheduling/

**Research reality:** Scheduling ML workloads across CPU/NPU hardware is an active topic (e.g. PRISM for heterogeneous neuromorphic systems), but this does not establish that routine CPU thread arbitration is faster, fairer or safer when every decision waits on NPU inference. https://arxiv.org/abs/2209.14777

**Emberglass recommendation:** deterministic CPU dispatch remains local and deadline-bounded; external policies recommend *coarse-grained* thread group placements, utilization preferences, resource classes, and accelerator jobs. Never block the fundamental thread scheduler waiting for a user-space AI service or device completion. Model failures must not halt CPU progress.

**Zero-copy caveat:** shared virtual or physical memory is not synonymous with zero latency. Device-specific DMA mapping/IOMMU, cache coherency, ordering barriers, ring synchronization, safety and revocation remain necessary. Study Linux dma-buf and DMA API as an engineering reference, not a codebase to copy. https://docs.kernel.org/driver-api/dma-buf.html and https://www.kernel.org/doc/html/latest/core-api/dma-api-howto.html

**Experiment:** Compare in-kernel baseline scheduling to an external heuristic and an optional predictor, measuring p50/p99 runnable latency, IPC overhead, fairness/starvation, CPU utilization and energy per useful unit of work. Synthetic accelerator delays must be injected to prove nonblocking fallback.

### 2. Verified and dynamically programmable kernel policy

**Evidence:** Linux sched_ext allows BPF scheduling policies to be loaded and removed, while falling back to default scheduling when errors or runnable stalls are detected. https://docs.kernel.org/scheduler/sched-ext.html

**Relevant research:** *eBPFML* (Columbia University, eBPF workshop 2025) proposes extending eBPF with matrix-multiply helpers / CPU matrix engines. This is a research design, **not** proof of broadly production-ready dynamic neural-to-eBPF compilation. https://www.cs.columbia.edu/~kkaffes/papers/empowering-ebpf25.pdf

**New 2026 example:** The AKTS preprint proposes pre-verifying policy libraries and switching by a simple policy index at runtime rather than compiling arbitrary generated code in the scheduling path. Treat numerical results as **author-reported preprint evidence**, not general guarantees. https://arxiv.org/abs/2609.12276

**Formal methods limit:** An eBPF verifier can reason about bounded program execution and memory safety; it does **not automatically prove** that complex system-wide policy cannot deadlock, starve a process, or violate intended fairness. Those require extra specifications, system modeling and separate evidence. See the Linux verifier docs and seL4 proof scope. https://docs.kernel.org/bpf/verifier.html and https://sel4.com/Verification/proofs.html

**Emberglass recommendation:** Start with an *immutable, reviewed set of policy profiles*, typed calls and model-independent validation, not a whole eBPF/Wasm VM in the kernel. Preserve optional pluggability as a later architecture study. A finite verified policy library is easier to audit and roll back than runtime generated bytecode.

**Experiment:** Submit invalid policy IDs, expired versions, out-of-bounds values, rapidly switching policies, and stale sequence numbers; confirm zero privilege escalation, bounded update cost and instant deterministic fallback.

### 3. Cognitive paging, learned caching, and semantic prefetch

**Most persuasive current source:** *Learning-Augmented Heuristics: Simple Yet Smart, Robust and Interpretable Cache Eviction* (USENIX OSDI 2026; Xia et al.) uses asynchronous learned parameter selection with a traditional high-speed cache data plane. The paper reports S4-FIFO improving mean efficiency 26% vs. S3-FIFO over 1,035 evaluation traces. This is a caching benchmark, **not** an established kernel-wide page-replacement improvement. https://www.usenix.org/conference/osdi26/presentation/xia

**Complement:** *Tiered Memory Management Beyond Hotness* (OSDI 2025) shows that access frequency alone isn't necessarily the right metric for performance-critical memory tiering; latency impact and memory-level parallelism matter. https://www.usenix.org/conference/osdi25/presentation/liu

**Correction:** The kernel cannot reliably infer exact application semantic intent from page faults alone. Routines such as Linux `madvise(MADV_SEQUENTIAL)`, `MADV_WILLNEED` and `MADV_POPULATE_READ` show why app-provided hints are often cleaner than guessed intentions. https://man7.org/linux/man-pages/man2/madvise.2.html

**Emberglass recommendation:** Prototype an opt-in `MemoryIntent` user-space API: `SequentialRead`, `RandomAccess`, `LatencySensitive`, `DiscardableCache` plus a bounded prediction hint, protected by memory quotas. The pager decides whether or not to honor the hint. No speculative prefetch that breaches capability or memory isolation.

**Experiment:** Reproducible memory access traces; compare FIFO, Clock, LRU approximations and a learning-augmented version; measure major/minor faults, cache misses, evictions, amplification, p99 response time, CPU overhead and energy, including adversarial input.

### 4. Adversarial workloads and policy integrity

**Threat model:** Untrusted processes can generate crafted I/O and scheduling patterns to exploit a weak priority predictor; even benign AI optimization can starve interactive tasks or overfit to benchmark patterns. The operating system should prevent the workload from gaining additional rights through the policy engine.

**False reassurance:** A model confidence score or prose explanation **does not prove safety**. Confidence is not guaranteed calibrated, and XAI output is not the same as proof of authorization, fairness or bounded latency.

**Countermeasures:** kernel-enforced permission and quota checks; fairness/starvation monitors; normalized and rate-limited telemetry; policy provenance and signing; model version pinning; independent watchdog; audit log; shadow testing; policy switching with timeouts, expiry, and rollback. Agent must lack permission to rewrite its own acceptance criteria.

**For online reward design:** Google DeepMind documents specification gaming: optimizing the literal reward can produce unintended behavior. Never optimize temperature/FPS alone without hard constraints on fairness, task completion and user input responsiveness. https://deepmind.google/blog/specification-gaming-the-flip-side-of-ai-ingenuity/

**Experiment:** Adversarial producer floods scheduling hints, varying file handle activity, workload bursts and crafted telemetry; verify maximum priority and resource budgets are not amplified and watchdog restores defaults.

### 5. Neuromorphic and event-driven OS research

**Genuine research area:** Intel Loihi 2 and Hala Point explore event-driven spiking models with memory/compute co-design. Hardware power claims must be evaluated at the correct *system* boundary: Intel's 2024 Hala Point system is described as having a maximum power of **2,600 W**, despite energy-efficiency benefits for certain workloads. Therefore “spiking models use microwatts” is not a general platform statement. https://newsroom.intel.com/artificial-intelligence/intel-builds-worlds-largest-neuromorphic-system-to-enable-more-sustainable-ai

**Practical limit:** Proprietary/limited-availability neuromorphic hardware and disparate software stacks make hardware-native scheduling unfit as an x86-64 QEMU boot requirement. If a compatible device exists, expose it as an accelerator driver and model-backend service.

**Experiment:** optional user-mode event-driven policy simulator fed by kernel events. Compare event rate, policy reaction time, energy, stability and costs versus deterministic and small conventional ML policies. Defer hardware until access is reliable.

## Feasibility and priority

| Idea | Research maturity | Emberglass priority | Placement |
| --- | --- | --- | --- |
| Capability-bound heterogeneous compute registry | Implementable OS interface | P1 | Kernel resource caps + user-space broker |
| External adaptive scheduler policy | Supported by ghOSt / sched_ext precedents | P1 after IPC | User-space control, kernel validates |
| Secure policy profiles / offline preverification | Strong engineering direction | P1 | Deterministic kernel validators |
| Learned eviction / memory hints | Credible academic research, workload dependent | P2 | User-space analysis and pager |
| Shared-memory accelerator IPC | Hardware-dependent, complex security | P2/P3 | Memory grants + driver service |
| Neural-to-eBPF/JIT | Emerging research, verifier complexity | P3 | Optional sandbox/compiler |
| NPU arbitrating every CPU thread | Unproven and deadline-risky | Research only | Never on mandatory dispatch path |
| Always-on RL training | Security / energy / convergence concerns | Research only | Offline or tightly sandboxed |
| Neuromorphic governor | Hardware-access dependent | Research only | Driver and user service |

## Roadmap aligned to original kernel development

1. **Keep boot evidence:** PR #2 original C x86-64 kernel + QEMU readiness, BIOS only, modern `-cpu max`; do not merge based on ELF compile alone.
2. **Rust `no_std` proof:** boot and basic serial with Limine in a branch, fault reporting (GDT, IDT, TSS), memory ownership, and paging protection.
3. **First capability + IPC:** protected address spaces, revocation, copy vs shared-grant benchmarks and fault isolation.
4. **First-class managed resources:** design and unit-test `ComputeDevice`, `InferenceSession` and `MemoryGrant` APIs *without* implementing an NPU driver.
5. **Deterministic policy engine:** baseline scheduling and resource budgeting; typed proposals, ownership checks, expiry, quotas, watchdog.
6. **Shadow simulation:** reproducible sched/memory trace replay, then optional simple learned recommendations.
7. **Advanced research:** preverified policy switching, learned cache controls, hardware accelerator broker and adversarial workloads. Any hardware-native ML project follows measured evidence.

## Potential research contributions

- **Capability-scoped AI resource orchestration:** Can models be managed as revocable, budgeted resource instances without putting model code into the trusted kernel?
- **Verified policy promotion:** Can automatically proposed policies be selected from preapproved alternatives under measurable invariants, with formally tested liveness/fairness properties?
- **Adaptive cache controls with robustness guarantees:** Does an asynchronous learner improve user-perceived p99 latency over fixed heuristics across unanticipated workloads?
- **Fail-safe heterogeneous optimization:** Is there any benefit to accelerator-assisted *coarse* resource policies after accounting for IPC, dispatch, energy, and synchronization costs?

## Research hygiene / anti-hype checklist

- Record title, venue, author affiliations, publication status and hardware setup for each paper.
- Distinguish peer-reviewed results from preprints and conceptual proposals.
- Track baseline, workload, inference overhead, 95/99th percentile regressions and energy.
- Never claim cache efficiency results automatically translate to page-fault elimination.
- A deterministic fallback alone is insufficient: enforce quotas, memory isolation, bounded queue waits and fairness.
- No claim that seL4 proofs carry over to Emberglass.
- No GPU/NPU/neuromorphic support claim before end-to-end driver integration and emulator/device testing.

## Key primary sources

1. Humphries et al., ghOSt, SOSP 2021, Google/Stanford: https://research.google/pubs/ghost-fast-and-flexible-user-space-delegation-of-linux-scheduling/
2. Linux sched_ext documentation: https://docs.kernel.org/scheduler/sched-ext.html
3. Xia et al., Learning-Augmented Heuristics, OSDI 2026: https://www.usenix.org/conference/osdi26/presentation/xia
4. Liu et al., Tiered Memory Management Beyond Hotness, OSDI 2025: https://www.usenix.org/conference/osdi25/presentation/liu
5. Sodhi et al., eBPFML, Columbia University workshop 2025: https://www.cs.columbia.edu/~kkaffes/papers/empowering-ebpf25.pdf
6. Khodabandehlou & Alizadeh, AKTS, arXiv preprint September 2026: https://arxiv.org/abs/2609.12276
7. Linux verifier reference: https://docs.kernel.org/bpf/verifier.html
8. Linux DMA sharing and fences: https://docs.kernel.org/driver-api/dma-buf.html
9. Linux DMA mapping: https://www.kernel.org/doc/html/latest/core-api/dma-api-howto.html
10. Linux `madvise`: https://man7.org/linux/man-pages/man2/madvise.2.html
11. seL4 proof scope: https://sel4.com/Verification/proofs.html
12. DeepMind specification gaming: https://deepmind.google/blog/specification-gaming-the-flip-side-of-ai-ingenuity/
13. Intel Loihi 2 / Hala Point: https://newsroom.intel.com/artificial-intelligence/intel-builds-worlds-largest-neuromorphic-system-to-enable-more-sustainable-ai
14. Das, PRISM real-time ML workload scheduling on heterogeneous neuromorphic SoC, preprint 2022: https://arxiv.org/abs/2209.14777
