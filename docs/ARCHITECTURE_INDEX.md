# Embers Kernel: architecture proposals and research record

**Status:** Research documentation under review. No ADR in this directory should be treated as an implemented mechanism or formal proof merely because it is merged.

**Created:** November 8, 2026. **Baseline reconciled:** November 10, 2026.

## Current implementation and trust boundary

The default branch boots a freestanding Rust `no_std` kernel under pinned Limine and headless x86-64/QEMU. The verified stages are [M02 Rust boot](https://github.com/bit-foundry-labs/embers-kernel/pull/26), [M03 exception diagnostics](https://github.com/bit-foundry-labs/embers-kernel/pull/27), [M04 physical-memory-map validation and frame accounting](https://github.com/bit-foundry-labs/embers-kernel/pull/28), and [M05 a bounded symbolic ownership ledger](https://github.com/bit-foundry-labs/embers-kernel/pull/29). These observations establish neither memory isolation nor capability enforcement.

**Not implemented:** Persistent physical-frame ownership, kernel-owned page tables, ring-3 isolation, syscall/IPC entry, authenticated capability handles, revocation of device memory, preemptive scheduling, AI workloads, or a GPU/NPU broker.

The [historical C experiment](https://github.com/bit-foundry-labs/embers-kernel/pull/2) is not a dependency of the Rust boot. **Emberglass OS is a separate, Linux-based environment.**

## Proposed architecture decisions

| Document | Question under review | Adoption status | Implemented? |
| --- | --- | --- | --- |
| [ADR-0001](adr/0001-capability-microkernel-and-ai-policy.md) | What belongs in the privileged kernel versus isolated services? | Proposed | No; only M02–M05 foundation exists |
| [ADR-0002](adr/0002-native-inference-capability-contract.md) | Could model invocations use user-space IPC services and generic kernel resource caps? | Proposed, long-term | No |
| [ADR-0003](adr/0003-ai-policy-validation-and-admission.md) | How could bounded policy proposals be authorized separately from actual CPU service guarantees? | Proposed, long-term | No; [host simulation PR #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6) is separate |
| [ADR-0004](adr/0004-split-phase-capability-revocation.md) | How should logical invalidation, in-flight work, device quiescence, and reclamation be separated? | Proposed, long-term | No; [finite model PR #15](https://github.com/bit-foundry-labs/embers-kernel/pull/15) is separate |

A proposal may be accepted as a design direction before implementation, but it must retain explicit unverified assumptions. Reference-model tests do not validate privileged Rust code or physical hardware. Device quiescence cannot be inferred from CPU epochs alone.

## Supporting material, not additional accepted decisions

The [AI-aware resource-management research review](research/2026-11-ai-native-os-frontiers.md) compares scheduling, paging, verification, and heterogeneous-hardware hypotheses. The [revocation/fairness/proof-scope review](research/2026-11-concurrent-revocation-fairness-proof-scope.md) distinguishes logical authority checks, CPU scheduling guarantees, and the hardware work required to reclaim memory safely.

The `notes/` directory preserves four dated **historical proposal summaries**. Their terminology and quoted ideas may reflect earlier, speculative brainstorming. They are not specifications, accepted ADRs, or evidence of implementation. Editorial updates should correct their current-status annotations rather than silently reinterpret historical source text.

## Review order

1. Review ADR-0001 for privileged responsibilities, isolation sequence, alternatives, and accepted-versus-proposed boundaries.
2. Review the narrow contract claims in ADRs 0002–0004 for testability and hardware-specific assumptions.
3. Review supporting research references and historical provenance separately from the design decision.
4. Confirm all links resolve to `bit-foundry-labs/embers-kernel`; verify cited publications individually before relying on specific performance claims.
5. Merge only after a human reviewer resolves open design questions. The mandatory GitHub CI checks cover syntax/build/QEMU milestones; they do not review the scientific validity of proposed mechanisms.

For near-term work, see the [contributor roadmap](CONTRIBUTOR_ROADMAP.md) and [launch readiness checklist](https://github.com/bit-foundry-labs/embers-kernel/issues/24).
