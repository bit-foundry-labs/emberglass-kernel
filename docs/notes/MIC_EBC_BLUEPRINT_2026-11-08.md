# Source proposal: Model Invocation and Execution Budget Capabilities

**Captured:** 2026-11-08
**Origin:** User-supplied architecture proposal
**Status:** Historical research input; claims and guarantees require evaluation. This is not an implemented Embers Kernel interface.
**Related review:** [ADR-0002](../adr/0002-native-inference-capability-contract.md)

## Research question

Can an original capability-based operating system make heterogeneous AI execution a native, secure computing primitive, while allowing adaptive resource policies to evolve without compromising deterministic kernel behavior?

**Proposed answer:** Yes, by separating execution mechanism (kernel) from execution policy (user-space AI), with kernel capabilities and bounded asynchronous accelerator execution.

## 1. First-class AI execution capabilities

### Model Invocation Capability (MIC)
Proposed to bind a specific model (read-only weight pages), execution context and selected hardware execution pipeline (NPU core or GPU stream) to an unforgeable authority handle. Intended to prevent unauthorized accelerator access or other models' weight reads.

### Execution Budget Capability (EBC)
Proposed to bound hardware cycles, operations or energy consumed by each inference invocation. Intended to prevent compute monopolization, runaway execution and denial of service.

### Original trust-boundary sketch

```text
USER-SPACE SANDBOX
  Model Invocation Capability (MIC)
  Execution Budget Capability (EBC)
       | secure IPC + enforced budgets
       v
DETERMINISTIC MICROKERNEL
  protected device memory mapping
  authorization, bounded scheduling
  timeout and safe fallback
```

## 2. Adaptive policy separated from deterministic mechanism

- **Kernel:** context switches, virtual memory mappings and interrupt routing, plus deterministic policy validation. Proposal described an O(1), fully verified core.
- **AI policy server:** unprivileged in terms of hardware access; it monitors telemetry and proposes macro-level resource priorities or memory prefetching.
- **Guard:** validate an envelope of allowed parameter values before applying any proposal; revert to static defaults on invalid input, timeout or server crash.

The source proposal uses the expression "zero clock cycles" for fallback. This is a desideratum, **not an achievable guarantee**.

## 3. Heterogeneous resource abstractions

- Treat CPU, GPU, NPU and neuromorphic accelerators as "Execution Vectors."
- Explore shared CPU/device virtual-memory views and direct shared-memory IPC to avoid redundant host copies.
- Explore device-local SRAM and confidential camera-to-model pipelines.

These are **hardware-dependent** aspirations; direct access to NPU SRAM without host RAM or CPU participation is not generally guaranteed.

## 4. Research directions named

- Wasm/eBPF sandboxing for user-space micro-models.
- CHERI and RISC-V / Arm hardware capability extensions.
- Formal proofs of policy validators and capability-envelope enforcement.

## Follow-up possibilities in the source text

1. Rust skeleton validating AI policy proposals.
2. Hardware-level CHERI and RISC-V capability study.
3. Initial accelerator target evaluation.

**Preservation note:** This is a structured transcription of the supplied proposal, not an exact character-for-character export of the original conversation. See the original message in chat for its complete wording and diagrams. The companion ADR intentionally corrects overconfident claims rather than treating this as an established design.
