# ADR-0001: Capability-oriented microkernel and bounded AI policy plane

- **Status:** Proposed for adoption, pending implementation benchmarks and explicit architectural review
- **Date:** 2026-11-08
- **Project:** Emberglass Kernel, Bit Foundry Labs
- **Source:** [Original AI-progressive blueprint](../notes/AI_PROGRESSIVE_KERNEL_PROPOSAL_2026-11-08.md)
- **Current implementation baseline:** C11 freestanding x86-64 kernel on feature branch \`feat/kernel-bootstrap-m01\`, PR #2. Its Limine BIOS/QEMU smoke test reached READY with \`-cpu max\`; the original C bootstrap is NOT yet a microkernel.

## The decision being proposed

Develop Emberglass toward an **original Rust \`no_std\`, capability-oriented microkernel**, with:
1. **Privileged minimal mechanisms:** address-space management, thread lifecycle/scheduling, kernel object and capability authorization, interrupts/timers, IPC/notifications, and basic platform bootstrap.
2. **Unprivileged system services:** file system, network stack, most drivers, process/application launch, desktop compositor, resource policy service, agent orchestrator and model inference.
3. **Capability handles:** opaque identifiers refer to kernel-protected object/rights entries. The kernel, not the AI, creates, validates, attenuates, transfers and revokes authority. Cryptographic tokens are not required for local handles.
4. **Deterministic enforcement, advisory intelligence:** AI recommends bounded changes from user space; kernel maintains safe scheduling, memory and permission fallbacks and always enforces privilege and resource limits.
5. **Explicit incremental proof:** keep the existing compiled-and-booted C prototype as a reference, and migrate components only with verified parity; no risky big-bang rewrite.

“**Asymmetric microkernel**” is a working label for *minimal fast-path mechanisms in privileged space + more sophisticated policies in isolated services*, not a claim of a standardized architecture class.

This proposal does **not** authorize an LLM inside ring 0, an automatic replacement for CPU branch prediction, or unbounded AI control over CPU scheduling.

## Goals and non-goals

**Goals:** original kernel, x86-64/QEMU first; desktop later; safety by explicit authority; low-latency predictable kernel mechanisms; traceable and revocable AI assistance; fault isolation and restartability; ability to develop and understand each layer.

**Non-goals for initial milestones:** running existing Linux binaries unchanged; production hardware drivers; formal verification from day one; speculative hardware neural predictors; trained models as safety-critical kernel dependencies; replacing the Linux kernel codebase.

A separate Linux-based Emberglass desktop is an interim shipping route, not the same kernel.

## Research informing this direction

- **seL4**: capability-based authorization, IPC, protected kernel objects; formal verification demonstrates the value of a small, explicit trusted computing base, but those proofs do NOT automatically apply to Emberglass. https://docs.sel4.systems/projects/sel4/ and https://docs.sel4.systems/Tutorials/capabilities.html
- **MINIX 3**: isolating drivers in user processes and restarting failures improves system availability, but restartability depends on state recovery and service dependencies. https://wiki.minix3.org/doku.php?id=www:documentation:reliability
- **Redox OS**: Rust-based microkernel-oriented OS and practical challenges of drivers, Unix-like compatibility and building whole services. https://github.com/redox-os/book/blob/master/src/introduction.md
- **Theseus (OSDI 2020)**: Rust, component state ownership and live recovery, but its single-address-space/single-privilege-level design differs substantially from the *hardware-enforced* isolation chosen here. https://www.usenix.org/conference/osdi20/presentation/boos
- **Linux sched_ext**: example of safely pluggable scheduling experiments with fallback, without making learned policies authoritative by default. https://www.kernel.org/doc/html/next/scheduler/sched-ext.html
- **Linux EEVDF**: contemporary Linux fair scheduling moved beyond the older CFS description in the source proposal. https://kernel.org/doc/html/latest/scheduler/sched-eevdf.html
- **Rust \`x86_64-unknown-none\`**: official \`no_std\` bare-metal target, kernel code model, no red zone, with careful CPU-feature configuration. https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **Intel SDM Vol. 3**: authoritative exception handling, page translation, privilege and interrupt reference. https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html

## Proposed mechanisms / policy separation

\`\`\`text
Desktop / apps / user commands / ForgeLoop integration
                         |
AI broker (user-space, optional; constrained by user approvals)
   | bounded recommendations + observable rationale
Resource policy service <-> telemetry & benchmark service
   | typed capability-restricted IPC (not arbitrary kernel writes)
+----------------------------------------------------------+
| Privileged Emberglass core                               |
| memory isolation | thread scheduler | IRQs | IPC | caps  |
| deterministic fallbacks | resource quotas | audit hooks |
+----------------------------------------------------------+
   | protected hardware state
CPU / MMU / timers / devices

Unprivileged peers: filesystem | network | drivers | compositor
\`\`\`

The AI broker should request actions via a policy service; it must never hold unduly broad capabilities to arbitrary kernel objects. All security-sensitive operations require typed operations, validation, permission checks, expiry/quotas, and audit logs.

## Proposed capability semantics

Start with kernel-managed slots holding \`{object_ref, rights, generation, provenance}\`. Userspace receives opaque handles; a call resolves the handle *in its caller's protected capability space*. Include:
- rights checks for every operation, default-deny;
- delegation with attenuation (never rights amplification);
- revocation and stale-handle protection;
- bounds on handle count, IPC queue lengths and CPU/memory use;
- recovery behavior for timeouts, failed services and process termination;
- eventual device DMA/IOMMU policy for driver isolation.

“Cryptographic capability” is unnecessary for local kernel handles, though it may be relevant later for remote delegation. Do not mistake a bearer string alone for a security boundary.

## AI introduction: staged gates, not promised dates

**Phase 0 — dependable baseline**:
- QEMU BIOS verified; UEFI pending; define minimum CPU features rather than relying solely on \`-cpu max\`.
- Install GDT/IDT/TSS and fault reporting, clarify C/Rust boot ABI.
- Implement frame ownership, page tables, protected user address spaces and kernel fault tests.
- Establish host tests, emulator regression, fuzzing/property tests for IPC/capabilities.

**Phase 1 — offline, user-controlled observation**:
- Opt-in local telemetry, with retention/redaction and off switch.
- User-space analysis produces explainable *suggestions*, not automatic settings.
- Compare against deterministic metrics; assume nonzero privacy, overhead and telemetry-corruption risks.

**Phase 2 — shadow recommendations**:
- Bounded user-space candidate scheduler/memory hints with no live authority.
- Log policy version, input features, proposed settings and actual outcomes.
- Evaluate p50/p99 latency, throughput, interactivity, power, starvation/fairness, fault recovery and reproducibility across workloads.
- Require measurable improvements and no regression before considering deployment.

**Phase 3 — constrained policy authority**:
- Expose typed policy interfaces with hard bounds, validity windows, rate limiting and revocation.
- Example: CPU performance class preference or per-service resource budget, never arbitrary runqueue edits or raw control-register writes.
- On model timeout, crash, invalid proposal or degraded metrics, atomically return to conservative defaults.
- Canary rollout, automatic rollback, audit and manual override.

**Phase 4 — optional research experiments only**:
- Isolate learned page-replacement/prefetching experiments behind benchmarks.
- CPU branch prediction belongs primarily to processor microarchitecture, not a normal operating-system kernel hook.
- No “sub-nanosecond NPU inference” dependency and no hard-RT guarantees from unbounded model code.
- Kernel-resident eBPF-like extensibility, if ever introduced, requires a sound verifier and its own threat model; user-space Wasm is a simpler early sandbox.

## Why not assume the microkernel is automatically “best”?

- Small kernels can reduce privileged attack surface but **do not eliminate** driver, ABI, IPC, logic or supply-chain bugs.
- User-space drivers and IPC impose context-switch/serialization costs; consider grants/shared-memory zero-copy under carefully scoped rights.
- Recovery is not guaranteed simply by restarting a user process: open resources, persistent state and inter-service coordination need protocols.
- Rust reduces memory-unsafe defects in safe code but cannot automatically prove correctness of \`unsafe\` blocks, assembly, concurrency or hardware interfaces.
- A modular monolith may be simpler for a learning bootstrap or particular performance targets. This is an architectural preference *for Emberglass's isolation goals*, not an industry-wide consensus.

## Architectural alternatives considered

| Option | Benefit | Cost | Disposition |
| --- | --- | --- | --- |
| Continue current C modular monolith | Lowest short-term complexity, already QEMU verified | More code remains privileged; weaker driver separation | Retain as baseline, not final target |
| Rust modular monolith | Memory safety gains without IPC overhead | More privileged driver/service code | Comparison/control |
| **Rust capability microkernel** | Narrow trusted core, explicit rights, user-space recovery | IPC, driver ecosystem, tooling, compatibility complexity | **Preferred research direction** |
| Base product on existing seL4/Redox | Mature components, faster prototyping | Does not satisfy original-kernel authorship goal | Study designs, don't fork |

## Acceptance criteria for ADR adoption

1. A tiny Rust \`no_std\` kernel can boot with diagnostics under QEMU without regressing C Milestone 01.
2. A user task runs in a separate address space; its invalid memory access cannot corrupt the kernel.
3. A two-task IPC ping-pong works with measured p50/p99 latency.
4. A capability-denied operation is rejected and auditable; revocation is tested.
5. A crashed user-space service can restart without kernel panic; failures of essential services are reported correctly.
6. An external policy daemon can suggest a bounded change, and invalid/stale recommendations are rejected with reproducible fallback.
7. No networking, telemetry export, cloud inference or paid-model calls occur without explicit opt-in.

**Decision status remains Proposed** until these tradeoffs and implementation details are reviewed. This ADR is not a claim of full implementation.

## Immediately actionable next steps

1. Keep PR #2 bootstrap unchanged until its CI and QEMU baseline are preserved.
2. Open a separate Rust \`no_std\` proof-of-concept branch reusing the existing Limine handoff, not a destructive rewrite.
3. Write the GDT/IDT/TSS + exception diagnostics milestone with QEMU-driven fault tests.
4. Implement the first capability object type and local IPC contract with tests.
5. Benchmark IPC against the existing C baseline before migrating drivers.
