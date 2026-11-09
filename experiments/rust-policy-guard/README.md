# Emberglass Rust Policy Guard (experimental)

A small, dependency-free, no_std-compatible Rust reference model for AI policy validation and simplified CPU minimum-reservation admission.

It is **not** a scheduler, kernel, protected capability table, hardware driver or mathematical proof of fairness. Policy validation is bound to a kernel-resolved capability handle plus caller `SpaceId` in this experiment; callers do not supply authority directly. No CPU reservations are delivered or timed here.

Test: `cargo test` in this directory.

## Invariants exercised
- No advisory policy for another tenant
- Reject excessive weight and memory requests instead of silently clamping
- Epoch expiry and replay rejection
- Model absence produces baseline fallback, independent of model inference
- Simplified sum of CPU minimum requests cannot exceed an epoch's available capacity

Remaining risks and limits: host-only caller identity simulation, no hardware-enforced address-space isolation, bounded linear revocation scans (possible DoS pressure at scale), and no formal verification of capability invariants.

Next: scheduler runtime fairness, syscall/IPC binding, fault injection, and formal property harnesses.

Related: [ADR-0003](../../docs/adr/0003-ai-policy-validation-and-admission.md) in the research PR #3.
