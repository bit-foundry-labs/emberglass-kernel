# Source proposal: AI policy validation and Rust typestate

Captured 2026-11-08. User-supplied architectural argument, preserved as a structured summary rather than an independently verified specification.

## Thesis
A Rust capability-based microkernel could allow user-space AI to recommend CPU, memory and accelerator settings while deterministic kernel mechanisms and protected capabilities implement changes.

## Proposed trust boundaries
- Unprivileged applications request resource allocations.
- An isolated user-space AI Policy Server observes redacted telemetry and recommends policies via typed IPC.
- A kernel-side validation pipeline checks memory mapping authority, CPU fairness envelopes and hardware accelerator isolation via IOMMU/SMMU.
- Only validated proposals may influence hardware allocation, with fallback when the AI service is absent.

## Three proposed safeguards
1. **Security**: Rust ownership + page tables + kernel capabilities + IOMMU/SMMU isolate other processes, their memory and accelerators.
2. **Fairness**: bounded priority weights combined with stride/token-bucket scheduling and alleged minimum guaranteed CPU time.
3. **Stability**: a typestate-style ValidatedPolicy ensures out-of-range proposals do not reach the execution mechanism.

## Illustrative code from source
A ValidatedPriority(u8) limits AI-supplied priority to 10..=250 and defaults to 128 on absence of a recommendation. A CoreScheduler then calls a placeholder execute_hardware_context_switch and prints a message.

## Claims requiring correction
- Clamping priorities does not establish fairness, starvation-freedom or minimum CPU-service guarantees.
- A type's private constructor protects a programming boundary, not remote IPC/hardware security by itself.
- Rust's type system alone does not prove scheduler liveness, fault isolation, or formal mathematical invariants.
- println! is not generally available in a no_std kernel.
- A placeholder method does not implement or prove safe context switching.
- Runtime validation and isolation are not necessarily zero-cost or O(1).
- A secure CPU address space and an IOMMU do not establish blanket immunity to information leaks and device faults.
- Anonymous-looking process IDs and performance counters may still leak sensitive application behavior.
- Safe fallback takes time and may fail to recover a hung non-preemptible accelerator.

See ../adr/0003-ai-policy-validation-and-admission.md for the rigorous interpretation.
