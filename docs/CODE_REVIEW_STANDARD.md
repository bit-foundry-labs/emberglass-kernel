# Emberglass: Source Code Review Standard

This is a practical quality standard for a new systems-programming research project. It adapts Robert C. Martin's *Clean Code: A Handbook of Agile Software Craftsmanship* (2008), particularly chapters 2–5, 7–10, 13, and 17, **without treating Java-specific advice as universally applicable to Rust or freestanding C**.

## Review in this order

1. **Correctness and trust boundaries.** A perfectly formatted permission bug is still a permission bug. Require a stated invariant for authority, resource ownership, revocation and allocation.
2. **Readability.** Use names that explain what a function does. Mutating methods should not look like read-only queries. Prefer descriptive domain names over ambiguous abbreviations or copied boilerplate.
3. **Cohesion.** Prefer functions that do one conceptual job and small modules with focused reasons to change. Do not split an atomic validation/commit operation purely to meet an arbitrary line count.
4. **Error paths.** Rust `Result<T, E>` and explicit C return values are idiomatic here; panics/exceptions in a kernel are not a substitute for deliberate failure handling. Check integer overflow and underflow, allocation failure, expired and revoked authority, malformed IPC, timeouts and device hangs.
5. **Tests.** Demonstrate the bug with a failing regression first when possible; test both expected behavior and adversarial boundaries. Include negative tests; explicitly differentiate simulated behavior, emulator behavior, and hardware guarantees.
6. **Formatting.** For Rust, run rustfmt and Clippy. For C, compile with enabled warnings-as-errors and use host sanitizer tests; apply a consistent formatter only after a baseline format pass is reviewed, to avoid hiding behavior changes in enormous diffs.
7. **Concurrency.** Declare the linearization point for authorization/revocation, invariants about held references, and whether locks/epochs protect operations. Timeouts alone do not establish device quiescence. Use model checking to complement, not replace, concrete Rust kernel tests.
8. **Reproducibility.** Pin critical tooling and document commands, emulator CPU profile, and benchmark environment. Prefer a single reliable entry command to a multi-stage undocumented manual process.

## Concrete checks currently available

| Component | Automated evidence | Scope |
| --- | --- | --- |
| C kernel boot reference (PR #2) | `make check` and `make test` with warnings as errors and host sanitizers; QEMU READY smoke test | Boot path and memory/serial host tests; not production isolation |
| Rust policy experiment (PR #6, PR #11) | `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | Host code; not a real kernel capability table |
| Split-phase revocation research (PR #15) | Python finite-state reference and pinned TLC finite model-checking | Abstract selected safety invariants only |
| New pull requests | Git diff whitespace-error check in `.github/workflows/source-hygiene.yml` | Tracked-file changes, not semantic correctness |

The older C code has inconsistent formatting across files. Do not introduce a blanket `clang-format` check until all existing files have undergone a **separate formatting-only** cleanup with unchanged behavior.

## PR design

- **One concern per PR:** fixing a UART timeout should not silently refactor unrelated memory management.
- **State transitions explicit:** `try_bind`, `validate_and_record`, `begin_revoke`, `quiesce`, and `reclaim` describe different contracts.
- **Human-readable API:** group strongly related permissions and limits in named types; be cautious with large positional-argument lists.
- **Fail closed:** reject unauthorized work without granting replacement authority or silently widening quotas.
- **No gratuitous abstraction:** keep simple code simple. Prefer tested, readable functions over complex traits for features the project does not yet need.
- **Evidence travels with changes:** include a minimal reproducer, expected/actual result, and checks performed.

## Current cleanup work

- [UART timeout and negative tests (PR #16)](https://github.com/bit-foundry-labs/emberglass-kernel/pull/16).
- [Explicit Rust gate binding and named capability specifications (PR #17)](https://github.com/bit-foundry-labs/emberglass-kernel/pull/17).
- The broader [capability-table implementation (PR #11)](https://github.com/bit-foundry-labs/emberglass-kernel/pull/11) needs synchronization and authorization-to-actuation review before becoming kernel code.

**License note:** this is an independently written adaptation of engineering principles from the book; it does not reproduce the text or examples of the work.
