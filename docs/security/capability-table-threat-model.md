# Capability table threat model and adversarial verification checklist

## Scope and status

This document models threats for the **Rust policy-guard capability table prototype** discussed in issue [#4](https://github.com/bit-foundry-labs/emberglass-kernel/issues/4) and draft PRs [#6](https://github.com/bit-foundry-labs/emberglass-kernel/pull/6) and [#11](https://github.com/bit-foundry-labs/emberglass-kernel/pull/11).

- **Prototype properties (current):** host-testable Rust `no_std`-compatible design assumptions and API-level invariants.
- **Not yet guaranteed:** ring-3 syscall authentication, hardware-enforced isolation, DMA ownership enforcement, production scheduler fairness/liveness guarantees, formal machine-checked proof of implementation correctness.

## Security objectives

1. Only trusted kernel-root authority can mint root capabilities.
2. Caller identity is derived from trusted execution context, never accepted as a user-supplied `SpaceId` claim.
3. Capability handles are treated as untrusted references (not secrets) and must pass protected lookup + generation checks.
4. Delegation must only attenuate rights and narrow limits; no privilege amplification.
5. Revocation must propagate parent-to-child and reject stale generation/ABA reuse.
6. Authorization and use must remain safe under concurrency and multicore scheduling.
7. CPU quota admission checks must not be misrepresented as full fairness/liveness guarantees.
8. Policy server interaction must tolerate denial-of-service pressure, stale epochs, and replay attempts.

## Threat matrix

| Attacker | Ability | Boundary under test | Required check/invariant | Negative test(s) | Remaining assumption |
| --- | --- | --- | --- | --- | --- |
| Unprivileged user task | Calls mint path with crafted identity | Kernel minting boundary | `mint_root` callable only from trusted kernel context; caller identity derived from thread metadata | Attempt root mint from user path; attempt to pass forged `SpaceId`; expect denial | Real kernel thread identity and syscall auth are not implemented in this prototype |
| Unprivileged user task | Replays/guesses handles | Per-process capability lookup | Handle lookup requires (slot, generation, owner space) match in protected table | Random handle fuzz; cross-space handle use; stale-generation handle after slot reuse | Table memory isolation and syscall boundary need hardware-backed kernel/user separation |
| Delegated child task | Tries rights escalation | Delegation attenuation | Child rights ⊆ parent rights; limits can only narrow | Property test: derive random parent/child rights and assert monotonic narrowing; explicit escalation attempts fail | Correctness currently demonstrated by tests, not formal proof of code |
| Revoked descendant | Uses previously valid handle | Revocation propagation | Parent revoke invalidates full subtree | Build delegation tree, revoke parent, assert all descendants denied | Revocation latency across cores depends on synchronization model not yet integrated with kernel scheduler |
| Concurrent malicious tasks | Exploit ABA/stale slot reuse | Generation and slot lifecycle | Generation increments on reuse; stale handle rejected | Interleaving test: free/reallocate same slot, then use old handle; fuzz with concurrent revoke/reuse operations | Requires atomicity/ordering guarantees in final multicore implementation |
| Racing caller | TOCTOU between authorize and use | Authorization-use temporal boundary | Authorization result bound to current object state/version/epoch | Simulate state change between authorize and use; stale epoch/version must fail | Final syscall/object model must expose version/epoch semantics consistently |
| Policy client flooder | Saturates policy server queues | Kernel-policy IPC admission | Bounded queues, fairness policy, timeouts, deterministic fallback | Flood requests until queue bound; verify bounded memory and fallback progress | Queue fairness policy details remain design work; not fully modeled in prototype |
| Replay attacker | Replays old “allow” recommendation | Epoch/freshness boundary | Recommendation accepted only for current epoch/window | Inject delayed/replayed recommendation; expect stale reject and baseline fallback | Trusted time/epoch source and anti-replay persistence are future kernel concerns |
| Adversarial tenant | Requests high CPU quota repeatedly | Admission vs runtime scheduling boundary | Admission feasibility check is distinct from runtime fairness/liveness | Cases where admission passes but scheduler can still starve under bad runtime policy; must be documented and detected | End-to-end fairness requires scheduler implementation and measurements, not just admission math |
| Compromised accelerator client | Submits long-lived DMA-capable work then gets revoked | Device isolation boundary | Revocation must account for in-flight/persistent device work | Simulate revoke during outstanding work; assert model marks unsupported/incomplete guarantee | No production DMA isolation/ownership model implemented yet |
| Overconfident integrator | Assumes Rust typestate implies hardware isolation | Language/runtime trust boundary | Rust safe code and typestate reduce memory-unsafety classes but do not replace privilege separation | Documentation assertion tests/checklist gate: reject claims equating safe Rust with ring-0/ring-3 isolation | Hardware/MMU/IOMMU boundaries are outside this prototype |

## Adversarial verification checklist

### A. Invariant-focused property tests

- [ ] **Root minting authority invariant:** for all untrusted caller contexts, minting root capability fails.
- [ ] **Caller binding invariant:** for all forged/mismatched `SpaceId` inputs, authorization derives identity from trusted context and rejects mismatch.
- [ ] **Handle validation invariant:** for all random handles, access succeeds iff owner + slot + generation all match current table state.
- [ ] **Attenuation invariant:** for all delegations, child rights/limits are a subset (never superset) of parent.
- [ ] **Revocation invariant:** for all parent capabilities with descendants, parent revoke invalidates descendant use.
- [ ] **Generation/ABA invariant:** for all stale handles after slot reuse, access is denied.
- [ ] **Epoch freshness invariant:** for all stale policy epochs/replayed decisions, fallback path is taken.

### B. Adversarial fault injection scenarios

- [ ] Inject malformed handles, max-value IDs, and randomized bit patterns.
- [ ] Inject cross-space confused-deputy attempts where attacker supplies victim `SpaceId`.
- [ ] Inject revoke/use races and reuse/use races under multicore-style interleavings.
- [ ] Inject policy-server queue flooding and delayed responses; verify bounded resource usage and timeout fallback.
- [ ] Inject stale replay packets and out-of-order epochs.
- [ ] Inject quota-saturating admission requests and compare admission outcomes vs measured runtime fairness (documenting any mismatch).
- [ ] Inject revocation during simulated persistent accelerator work and record unsupported guarantees explicitly.

### C. Evidence quality gates

- [ ] Each invariant has at least one deterministic negative test and one randomized/property-style stress test.
- [ ] Each denial path states expected error/fallback behavior (not just “fails”).
- [ ] Test reports distinguish **prototype simulation evidence** from **future kernel/hardware guarantees**.

## Prototype guarantees vs future guarantees

### Achieved in prototype scope (if tests pass)

- API-level handle validation with owner/slot/generation checks.
- Delegation attenuation and subtree revocation semantics at model level.
- Explicit stale/replay rejection rules and deterministic fallback intent.
- `no_std`-compatible Rust design constraints and `unsafe`-avoidance in the prototype crate.

### Deferred to future kernel/hardware work

- Trusted ring-3 caller identity derivation through real syscall/IPC path.
- Enforced kernel/user memory separation and tamper-resistant cap-table storage.
- Multicore synchronization proofs for TOCTOU, revocation propagation latency, and fairness under contention.
- Device isolation (IOMMU/DMA ownership), cancellation, and revocation of in-flight accelerator work.
- End-to-end formal verification claims tied to concrete implementation artifacts.

## References

- seL4 capabilities tutorial: <https://docs.sel4.systems/Tutorials/capabilities.html>
- seL4 IPC tutorial: <https://docs.sel4.systems/Tutorials/ipc>
- seL4 Mixed-Criticality Scheduling (MCS) background: <https://sel4.systems/About/FAQ.html> (MCS kernel variant and temporal isolation context)
- Rust `no_std` target support (x86_64-unknown-none): <https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html>
- Formal-methods reference (primary): seL4 proof/code correspondence overview: <https://sel4.systems/Verification/proofs.html>
