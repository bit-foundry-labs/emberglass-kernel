# Source brief: five frontier ideas for an AI-native Emberglass OS

**Captured:** 2026-11-08
**Origin:** User-supplied architectural exploration in Bit Foundry conversation
**Status:** Hypotheses to critically evaluate, not verified implementation instructions
**Relation:** [Research audit](../research/2026-11-ai-native-os-frontiers.md) and [ADR-0001](../adr/0001-capability-microkernel-and-ai-policy.md)

The source exploration argued that an AI-integrated OS should treat models and heterogeneous accelerators as "first-class" computing resources, rather than simply running AI as ordinary applications, and proposed these five domains.

## 1. Hardware and runtime: heterogeneous asymmetric schedulers

- **NPU-first thread scheduling.** Explore using on-chip NPUs or GPU neural micro-models to predict thread yields, I/O stalls and CPU placements instead of relying entirely on conventional CPU runqueue algorithms.
- **Zero-copy NPU IPC.** Study lock-free shared-memory rings and unified memory for low-overhead communication between policy services, microkernel and accelerators.

## 2. Compiler and verification: dynamic eBPF / Wasm mechanics

- **Neural-to-eBPF compilation.** Compile or distill small models into verifiable eBPF / Wasm policies.
- **Formal safety checks.** Investigate SMT / Coq-family tools to check properties before loading policy updates; the original idea called for proofs against memory errors, infinite loops and deadlocks.

## 3. Cognitive memory management

- **Spatio-temporal learned paging.** Small Markov models or RNNs learn application access signatures to inform eviction and swap decisions.
- **Semantic prefetch.** Infer application intent, such as sequential database scans, to provide memory placement and prefetch hints ahead of page faults.

## 4. Security: adversarial behavior and model sanitization

- **Policy manipulation.** Protect against malicious workloads crafted to receive unfair priority or favorable resource allocation from an adaptive policy.
- **Deterministic sandbox / explainability.** Audit and log model decisions, use bounded execution, and restore safe defaults when recommendations are suspicious or unavailable.

## 5. Neuromorphic and event-driven paradigms

- **Spiking neural network governors.** Explore event-driven low-energy policies on compatible neuromorphic processors.
- **Online reinforcement learning.** Incrementally learn from outcomes like temperature, power or responsiveness, while guarding against forgetting and reward gaming.

## Suggested reading in the source brief

- A work called "Project Ghost (Microsoft Research)" was cited. **Research correction:** the relevant published system is *ghOSt: Fast and Flexible User-Space Delegation of Linux Scheduling* (Google/Stanford, SOSP 2021): https://research.google/pubs/ghost-fast-and-flexible-user-space-delegation-of-linux-scheduling/
- The Linux eBPF runtime and verifiers: https://docs.kernel.org/bpf/verifier.html
- Research on tiny verified/quantized policies, eBPF, learned prefetching and heterogeneous acceleration.

## Preservation note

The above is a faithful topical transcription of the supplied ideas, not a verbatim copy of the entire chat message. The companion research document distinguishes supported work, speculative claims, architectural hazards and experiment plans so that future Emberglass discussions do not confuse the original proposal with adopted design decisions.
