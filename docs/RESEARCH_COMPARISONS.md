# Research comparisons and Embers hypotheses

These comparisons inform design; they are not claims that Embers implements or
matches the referenced systems.

- [seL4 verification assumptions](https://sel4.systems/Verification/assumptions.html):
  capability authority and proof boundaries motivate explicit revocation and
  device/DMA assumptions. Embers has neither equivalent verification nor
  authenticated capability enforcement.
- [ASTERINAS, USENIX ATC 2025](https://www.usenix.org/conference/atc25/presentation/peng-yuke):
  a Rust framekernel explores a smaller memory-safety trusted computing base.
  Compare its treatment of unsafe boundaries; Embers remains a microkernel
  research project and has no comparable assurance or Linux ABI goal.
- [CONVEROS, USENIX ATC 2025](https://www.usenix.org/conference/atc25/presentation/tang):
  model checking uncovered concurrency defects in Rust OS modules. Investigate
  specification-to-code conformance for revocation before making SMP claims.
- [XSched, OSDI 2025](https://www.usenix.org/conference/osdi25/presentation/shen-weihang):
  heterogeneous accelerator scheduling exposes varied preemption capabilities.
  Treat device quiescence, cancellation, and DMA safety as distinct concerns.

Candidate hypothesis: a user-space adaptive resource policy can be constrained
by deterministic kernel admission and budgets, and a failed policy can fall
back safely. This remains untested. A future experiment needs a deterministic
baseline, failure scenarios, explicit safety invariants, reproducible workloads,
and measured latency, fairness, overhead, and recovery bounds.
