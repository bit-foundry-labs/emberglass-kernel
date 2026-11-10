# Embers Kernel · Contributor research roadmap

This is a research sequence, not a release schedule. Work is tracked in issues and tested pull requests. Design decisions are proposals until accepted.

## Foundation: reproducible execution

**Integration gate:** [Issue #24](https://github.com/bit-foundry-labs/embers-kernel/issues/24) tracks a trustworthy Rust/QEMU baseline, starting with the minimal Rust `no_std` M02 entry and continuing toward tested exception and frame-accounting milestones.
- Reproduce the original C kernel's x86-64 BIOS/QEMU boot and retain serial evidence ([PR #2](https://github.com/bit-foundry-labs/embers-kernel/pull/2)).
- Record minimum supported CPU features (the existing QEMU run uses -cpu max) and add UEFI coverage.
- Install and test GDT, IDT, TSS, exception reporting, and deliberate CPU faults.
- Reproduce the minimal Rust `no_std` boot on `main`; confirm Limine handoff, serial `READY`, pinned toolchain and documented unsafe assumptions. The original [PR #20](https://github.com/bit-foundry-labs/embers-kernel/pull/20) introduced this experimental code.

## Isolation: the real microkernel
- Physical memory ownership, page tables and user/kernel page permissions.
- First separate user-mode task and validated syscall/IPC boundary.
- Capability handle table; revocation, attenuation, stale-handle rejection.
- Deterministic scheduling, CPU time budgets and documented admission assumptions.
- Crash isolation and restartable user-space services, with state-recovery tests.

## Adaptive policy, only after isolation
- Host-tested Rust policy validation experiment ([PR #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6)).
- Model Invocation and Execution Budget interface research ([Issue #5](https://github.com/bit-foundry-labs/embers-kernel/issues/5)).
- Shadow policy vs. deterministic baseline, with p50/p99, starvation, overhead and fault injection.
- Optional accelerator broker, after device isolation and DMA ownership semantics are implemented.

## Read before proposing a large subsystem
- [Architecture ADR PR #3](https://github.com/bit-foundry-labs/embers-kernel/pull/3) contains candidate designs, **not implemented guarantees**.
- [Research issue #4](https://github.com/bit-foundry-labs/embers-kernel/issues/4) defines the initial benchmark questions.

## Definition of a good first issue
An expected result, a bounded number of files, concrete tests, and an explicit limitation. Missing performance data is an invitation to measure, not invent numbers.
