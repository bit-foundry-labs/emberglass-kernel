# ADR-0001: Capability-oriented microkernel and bounded AI policy plane

- **Status:** Proposed; architectural approval and implementation are separate milestones.
- **Created:** 2026-11-08
- **Baseline reviewed:** 2026-11-10
- **Project:** Embers Kernel, Bit Foundry Labs LLC
- **Source:** [Historical AI-progressive research proposal](../notes/AI_PROGRESSIVE_KERNEL_PROPOSAL_2026-11-08.md)
- **Verified implementation baseline:** The `main` branch contains a Rust `no_std` x86-64 kernel that boots under BIOS/QEMU with a pinned Limine bootloader ([M02 #26](https://github.com/bit-foundry-labs/embers-kernel/pull/26)), basic GDT/TSS/IDT exception diagnostics ([M03 #27](https://github.com/bit-foundry-labs/embers-kernel/pull/27)), validated Limine memory-map and monotonic frame accounting ([M04 #28](https://github.com/bit-foundry-labs/embers-kernel/pull/28)), and a bounded symbolic ownership ledger ([M05 #29](https://github.com/bit-foundry-labs/embers-kernel/pull/29)). These are not user-space isolation, an authoritative capability table, or a complete microkernel. The C prototype remains in [historical PR #2](https://github.com/bit-foundry-labs/embers-kernel/pull/2); it is not required by the Rust build.

## The decision being proposed

Develop Embers Kernel toward an **original Rust `no_std`, capability-oriented microkernel**, with:
1. **Privileged minimal mechanisms:** address-space management, thread lifecycle/scheduling, kernel object and capability authorization, interrupts/timers, IPC/notifications, and basic platform bootstrap.
2. **Unprivileged system services:** file system, network stack, most drivers, process/application launch, desktop compositor, resource policy service, agent orchestrator and model inference.
3. **Capability handles:** opaque identifiers refer to kernel-protected object/rights entries. The kernel, not the AI, creates, validates, attenuates, transfers and revokes authority. Cryptographic tokens are not required for local handles.
4. **Deterministic enforcement, advisory intelligence:** AI recommends bounded changes from user space; kernel maintains safe scheduling, memory and permission fallbacks and always enforces privilege and resource limits.
5. **Incremental verification:** extend the integrated Rust boot and exception baseline with small, testable security boundaries. Preserve the C experiment as historical evidence, not an active parallel kernel or migration prerequisite.

“**Asymmetric microkernel**” is a working label for *minimal fast-path mechanisms in privileged space + more sophisticated policies in isolated services*, not a claim of a standardized architecture class.

This proposal does **not** authorize an LLM inside ring 0, an automatic replacement for CPU branch prediction, or unbounded AI control over CPU scheduling.

## Goals and non-goals

**Goals:** original kernel, x86-64/QEMU first; desktop later; safety by explicit authority; low-latency predictable kernel mechanisms; traceable and revocable AI assistance; fault isolation and restartability; ability to develop and understand each layer.

**Non-goals for initial milestones:** running existing Linux binaries unchanged; production hardware drivers; formal verification from day one; speculative hardware neural predictors; trained models as safety-critical kernel dependencies; replacing the Linux kernel codebase.

Emberglass OS is a separate Linux-based environment. Embers Kernel is an independent research kernel, not a Linux customization or a working replacement for Emberglass OS.

## Research informing this direction

- **seL4**: capability-based authorization, IPC, protected kernel objects; formal verification demonstrates the value of a small, explicit trusted computing base, but those proofs do not apply to Embers Kernel. https://docs.sel4.systems/projects/sel4/ and https://docs.sel4.systems/Tutorials/capabilities.html
- **MINIX 3**: isolating drivers in user processes and restarting failures improves system availability, but restartability depends on state recovery and service dependencies. https://wiki.minix3.org/doku.php?id=www:documentation:reliability
- **Redox OS**: Rust-based microkernel-oriented OS and practical challenges of drivers, Unix-like compatibility and building whole services. https://github.com/redox-os/book/blob/master/src/introduction.md
- **Theseus (OSDI 2020)**: Rust, component state ownership and live recovery, but its single-address-space/single-privilege-level design differs substantially from the *hardware-enforced* isolation chosen here. https://www.usenix.org/conference/osdi20/presentation/boos
- **Linux sched_ext**: example of safely pluggable scheduling experiments with fallback, without making learned policies authoritative by default. https://www.kernel.org/doc/html/next/scheduler/sched-ext.html
- **Linux EEVDF**: contemporary Linux fair scheduling moved beyond the older CFS description in the source proposal. https://kernel.org/doc/html/latest/scheduler/sched-eevdf.html
- **Rust `x86_64-unknown-none`**: official `no_std` bare-metal target, kernel code model, no red zone, with careful CPU-feature configuration. https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **Intel SDM Vol. 3**: authoritative exception handling, page translation, privilege and interrupt reference. https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html

## Proposed mechanisms / policy separation

```text
Desktop / apps / user commands / ForgeLoop integration
                         |
AI broker (user-space, optional; constrained by user approvals)
   | bounded recommendations + observable rationale
Resource policy service <-> telemetry & benchmark service
   | typed capability-restricted IPC (not arbitrary kernel writes)
+----------------------------------------------------------+
| Proposed privileged Embers Kernel core                   |
| memory isolation | thread scheduler | IRQs | IPC | caps  |
| deterministic fallbacks | resource quotas | audit hooks |
+----------------------------------------------------------+
   | protected hardware state
CPU / MMU / timers / devices

Unprivileged peers: filesystem | network | drivers | compositor
```

The AI broker should request actions via a policy service; it must never hold unduly broad capabilities to arbitrary kernel objects. All security-sensitive operations require typed operations, validation, permission checks, expiry/quotas, and audit logs.

## Proposed capability semantics

Start with kernel-managed slots holding `{object_ref, rights, generation, provenance}`. Userspace receives opaque handles; a call resolves the handle *in its caller's protected capability space*. Include:
- rights checks for every operation, default-deny;
- delegation with attenuation (never rights amplification);
- revocation and stale-handle protection;
- bounds on handle count, IPC queue lengths and CPU/memory use;
- recovery behavior for timeouts, failed services and process termination;
- eventual device DMA/IOMMU policy for driver isolation.

“Cryptographic capability” is unnecessary for local kernel handles, though it may be relevant later for remote delegation. Do not mistake a bearer string alone for a security boundary.

## AI introduction: staged gates, not promised dates

**Phase 0 — dependable baseline**:
- **Completed in the emulator:** Rust `no_std` boot, GDT/TSS/IDT exception diagnostics, Limine memory-map parsing, and bounded frame accounting. These are milestone-specific QEMU/host results, not real-hardware security tests.
- **Unverified:** Minimum x86-64 CPU feature profile beyond QEMU `-cpu max`, UEFI behavior, and independent fresh-machine reproduction.
- **Not implemented:** Persistent physical-frame ownership, kernel-owned page tables, hardware user/kernel isolation, syscalls, IPC, authenticated capabilities, and preemptive scheduling.
- **Next:** Define memory mapping/ownership invariants and add QEMU fault-path tests before any user-mode or adaptive-policy integration.

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
- Rust reduces memory-unsafe defects in safe code but cannot automatically prove correctness of `unsafe` blocks, assembly, concurrency or hardware interfaces.
- A modular monolith may be simpler for a learning bootstrap or particular performance targets. This is an architectural preference for Embers Kernel's isolation goals, not an industry-wide consensus.

## Architectural alternatives considered

| Option | Benefit | Cost | Disposition |
| --- | --- | --- | --- |
| Historical C bootstrap experiment | Provides a prior boot reference only | Does not implement the Rust kernel or provide a current architecture baseline | Preserve as history; do not integrate into the Rust kernel |
| Rust modular monolith | Memory safety gains without IPC overhead | More privileged driver/service code | Comparison/control |
| **Rust capability microkernel** | Narrow trusted core, explicit rights, user-space recovery | IPC, driver ecosystem, tooling, compatibility complexity | **Preferred research direction** |
| Base product on existing seL4/Redox | Mature components, faster prototyping | Does not satisfy original-kernel authorship goal | Study designs, don't fork |

## Decision review gate

Adopting this ADR means agreeing on **where authority and policy belong**, not claiming the future architecture has been implemented. Reviewers should confirm that the proposed privileged/user-space division, kernel authority rules, failure behavior, alternatives, and testing plan are internally coherent. Implementation evidence must be recorded separately from ADR status.

## Future implementation acceptance criteria

1. A user task runs in its own hardware-enforced address space; prohibited accesses fault without corrupting privileged memory.
2. A bounded, authenticated syscall/IPC protocol passes positive and adversarial tests and has measured p50/p99 overhead.
3. A capability-denied operation is rejected; stale handles, delegation, revocation and in-flight work are tested under declared concurrency assumptions.
4. A failed user-space service can be restarted according to explicit state-recovery contracts.
5. An optional policy service can recommend bounded changes, while invalid, stale or missing recommendations leave deterministic scheduling functional.
6. No telemetry export, network dependency, cloud inference or paid-model call is introduced without explicit approval.

These are **future milestones** and are not implied by accepting the ADR.

## Immediate engineering sequence

1. Confirm independent Ubuntu/WSL reproduction and establish a tested CPU-feature baseline ([#8](https://github.com/bit-foundry-labs/embers-kernel/issues/8), [#9](https://github.com/bit-foundry-labs/embers-kernel/issues/9)).
2. Specify persistent physical-frame provenance and kernel-owned page-table invariants before changing CR3 or granting writable mappings.
3. Establish an explicitly tested supervisor/user boundary and a kernel-controlled syscall/IPC interface.
4. Introduce authority tables only after protected caller identity and the permission boundary exist.
5. Benchmark deterministic scheduling and IPC before experimenting with user-space adaptive recommendations.
