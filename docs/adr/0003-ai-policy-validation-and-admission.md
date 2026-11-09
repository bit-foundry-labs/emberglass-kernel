# ADR-0003: Capability-constrained AI policies, typed validation and CPU admission control

**Status:** Proposed. Architectural research only, pending Rust tests and kernel integration.
**Date:** 2026-11-08
**Inputs:** User-supplied AI Policy Validation blueprint (../notes/AI_POLICY_VALIDATION_BLUEPRINT_2026-11-08.md), ADR-0001 and ADR-0002.

## Decision proposal
Adopt two separate contracts:

1. **Policy authorization and validity:** A kernel-verified caller capability authorizes a narrow policy update for a specific resource, tenant or group. Runtime input is parsed and validated before producing a private ValidatedPolicy value. Reject malformed, stale, unauthorized, out-of-envelope or replayed requests.
2. **Scheduling feasibility and execution:** Reserve minimum CPU service only after checking aggregate requests against available CPU capacity and preserving kernel/critical-service reserve. A runtime scheduler must enforce those reservations; a priority clamp is **not** a proof of fairness.

Keep the AI agent in user space. The kernel always has runnable-thread dispatch independent of the model. Accelerator policies pass through device-service contracts and, where supported, IOMMU mapping.

## Why a priority clamp cannot prove fairness

Consider one runnable task with weight 250 and another with weight 10. If the scheduler always dispatches the first, the second can starve even though both weights satisfy the validator. A type such as ValidatedPriority proves at most that *one integer is in a permitted range*. It does not describe dispatch decisions, resource accounting or adversarial task arrival.

A useful baseline is an explicit CPU-service reservation within an epoch. On a single CPU with epoch length E and reserved kernel work K, minimum CPU grants m_i are feasible only if:

  sum(m_i) <= E - K.

For multiple CPUs and real-time deadlines, this simple condition is only a necessary simplification; partitioning, migration, critical sections, interrupt overhead, blocking, hardware topology and preemption costs matter. Admission feasibility alone is not runtime assurance. Deadlines require a specified scheduling algorithm and schedulability analysis.

seL4 MCS scheduling contexts enforce *upper* budget/period limits; its documentation cautions that they are not lower bounds. Zircon has fair and deadline scheduling disciplines and requires feasible deadline demand.

## Authorization model
The untrusted AI's message contains resource ID, caller identity, proposed weight/budget/placement, expiry epoch and nonce/version. The kernel resolves the caller's opaque capability in a protected table, checks target binding and allowed rights, then validates resource constraints.

Private constructors and typestate eliminate accidental misuse within safe Rust, but not automatically forged IPC byte streams. The protected kernel handle table, caller address-space isolation, parsing, rights checks and proven revocation semantics establish the runtime boundary.

Invalid policy suggestions should be rejected with logged reasons, not silently clamped: silently changing proposed values can conceal model failure and complicate replay/debugging.

## Required invariants to test
- No unprivileged caller can issue policy changes for other tenants or resources.
- No accepted policy exceeds the bounds on its protected capability.
- No stale or expired policy is accepted; replayed epochs are rejected.
- Sum of admitted CPU minimum reservations stays within a declared capacity reserve.
- Policy absence, crash or delay cannot stop ordinary CPU dispatch.
- Per-tenant starvation and p99 latency are measured with a *real* scheduler, not assumed from positive weights.
- Memory and accelerator placement proposals cannot expand DMA or address-space mappings.
- Queue bounds, stale completion safety and explicit model failure handling are preserved.

## Rust implementation strategy
- Start with a host-tested Rust no_std-compatible reference *validator* crate. No external dependencies.
- Introduce private ValidatedPolicy; check capability scope, revision/expiry, weight, quotas and admission feasibility.
- Simulate invalid proposals and multi-tenant overload, return typed errors and baseline fallback.
- Use CI cargo test plus clippy, and optionally Kani bounded model checking for finite-state invariants.
- Later port generic mechanisms into original Rust kernel after IDT, paging, ring-3 isolation, IPC and protected handle tables exist.
- Use Verus where strong functional correctness arguments are valuable. Both Kani and Verus need a correctly stated property and defined assumptions.

## Safety scope
Rust memory safety does not prove system-wide liveness or eliminate side channels. An IOMMU can restrict certain device DMA, not automatically isolate all GPU/firmware/side-channel states. An opaque or pseudonymous process ID is not necessarily anonymous telemetry. CPU dispatch is not universally O(1), and deterministic policy validation does not guarantee identical interleavings under changing interrupts.

## Research references
- seL4 MCS, scheduling contexts are upper bounds: https://docs.sel4.systems/Tutorials/mcs.html
- Zircon scheduler fair/deadline policies and admission feasibility: https://fuchsia.googlesource.com/fuchsia/+/09e728cb2845ccf1344b650aa5518649cf3cea0e/docs/concepts/kernel/kernel_scheduling.md
- Zircon fair scheduler O(log n) algorithm and fairness details: https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/docs/concepts/kernel/fair_scheduler.md
- Linux sched_ext errors and stalls trigger fallback: https://kernel.org/doc/html/latest/scheduler/sched-ext.html
- ghOSt user-space policy delegation, SOSP 2021: https://mast.stanford.edu/pubs/ghost_fast_flexible_user_space_delegation_of_linux_scheduling/
- Asterinas, Rust framekernel with minimized unsafe TCB, USENIX ATC 2025: https://www.usenix.org/conference/atc25/presentation/peng-yuke
- Kani verifier: https://model-checking.github.io/kani/
- Verus project: https://www.microsoft.com/en-us/research/project/practical-system-verification/

**Status note:** This ADR has not been accepted or implemented in the bootable kernel. A host-only experiment is a first step, not a demonstrated kernel guarantee.
