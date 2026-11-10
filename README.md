# Embers Kernel

**An open-source, Rust-first microkernel research project for AI-oriented operating systems.**

We're investigating what the kernel should control when applications and AI agents compete for CPU time, memory, and eventually accelerator resources. The aim is a small, predictable enforcement layer: capability-based authority, explicit resource budgets, isolation, and measurable performance.

**This is a research effort, not a working general-purpose operating system or a benchmark-proven AI-optimized kernel.**

[Contribute](CONTRIBUTING.md) · [Research roadmap](docs/CONTRIBUTOR_ROADMAP.md) · [Open issues](https://github.com/bit-foundry-labs/embers-kernel/issues) · [Contributor launch milestone](https://github.com/bit-foundry-labs/embers-kernel/issues/24)

## The engineering idea

**Keep enforcement deterministic; keep intelligence outside the trusted kernel.**

- **Original microkernel:** target Rust `no_std`, x86-64 and reproducible QEMU experiments. This is not a Linux fork.
- **Capabilities, not ambient privilege:** explicit authority for memory, processes, IPC, and eventual compute resources; test revocation and stale-handle behavior.
- **AI-aware without AI in the kernel:** untrusted user-space services may propose scheduling or resource policies, but cannot grant themselves authority.
- **Bounded resources and fair fallback:** investigate CPU budgets, admission, latency, starvation, and safe recovery before attempting accelerator integration.
- **Evidence before claims:** compare proposed adaptations against deterministic baselines; measure IPC costs, tail latency, overhead, and failure behavior.

Embers Kernel is **separate from Emberglass OS**, a Linux-based Rust operating-system environment. They explore related questions but do not share a working kernel.

## Where the code actually stands

This integration introduces the first **bootable Rust `no_std` x86-64 kernel entry** on the default branch. It prints a diagnostic `READY` marker in QEMU, but **does not yet implement user-mode execution, protected page tables, capabilities, IPC, or scheduling**. More advanced boot experiments remain in stacked, draft PRs:

| Track | Experimental work | Status |
| --- | --- | --- |
| Rust `no_std` boot | Limine handoff, minimal UART diagnostics, freestanding x86-64 ELF | M02: reproducible on `main` after this PR merges |
| Architecture | Proposed capability boundaries, resource budgets, and AI policy separation | [PR #3](https://github.com/bit-foundry-labs/embers-kernel/pull/3), not merged |
| Rust policy research | Host-testable `no_std`-compatible validation and CPU-admission *model* | [PR #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6), not a running kernel |
| Rust boot progression | Exception traps and physical frame accounting experiments | [PRs #21–#23](https://github.com/bit-foundry-labs/embers-kernel/pulls), stacked and under review |

A passing QEMU or host-test check proves only the tested milestone. It does **not** establish hardware isolation, kernel-level capability enforcement, production security, scheduler superiority, or a functioning AI operating system.

## Boot Embers in headless QEMU

**Development emulator only. Do not boot on physical hardware.** On Ubuntu or WSL2 Ubuntu, install Rust via [rustup](https://rustup.rs) and then:

```bash
git clone https://github.com/bit-foundry-labs/embers-kernel.git
cd embers-kernel
sudo apt-get update
sudo apt-get install -y make binutils python3 qemu-system-x86 xorriso curl tar
cd rust-kernel
rustup toolchain install 1.90.0 --component clippy,rustfmt --target x86_64-unknown-none
cd ..
make rust-check
make rust-iso
qemu-system-x86_64 -cpu max -m 256M -cdrom build/embers-rust.iso \
  -display none -serial stdio -monitor none -no-reboot
```

The serial log should contain `READY: Embers Rust no_std entry reached.`. The intentionally halted kernel keeps QEMU running; stop the emulator with Ctrl+C. `-cpu max` is only the verified emulator profile, not a minimum hardware requirement. Both CI evidence and unsafe assumptions are described in [the Rust boot notes](rust-kernel/README.md).

The earlier C prototype remains accessible as the separate [PR #2 research reference](https://github.com/bit-foundry-labs/embers-kernel/pull/2). It is **not** required to build or execute the Rust kernel.

For the host-only policy experiment, check out [PR #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6) and run `cargo test` in `experiments/rust-policy-guard`. That does not execute a privileged Rust kernel.

## Help build it

We're looking for contributors interested in **Rust systems programming, kernel bring-up, x86-64/QEMU, capability security, IPC, concurrency, scheduling, model checking, and reproducible benchmarks**.

A useful first contribution is a narrow, verifiable improvement, not a sweeping rewrite:

- [Reproduce the QEMU bootstrap and improve troubleshooting](https://github.com/bit-foundry-labs/embers-kernel/issues/8)
- [Document the minimum supported x86-64 CPU features](https://github.com/bit-foundry-labs/embers-kernel/issues/9)
- [Challenge the capability threat model](https://github.com/bit-foundry-labs/embers-kernel/issues/12)
- [Help prepare a reproducible, contributor-ready Rust boot baseline](https://github.com/bit-foundry-labs/embers-kernel/issues/24)

Read [the contribution guide](CONTRIBUTING.md), [research roadmap](docs/CONTRIBUTOR_ROADMAP.md), [review standard](docs/CODE_REVIEW_STANDARD.md), and [Code of Conduct](CODE_OF_CONDUCT.md). Comment on an issue before undertaking substantial work.

## License and stewardship

An experimental open-source project stewarded by [Bit Foundry Labs](https://github.com/bit-foundry-labs), licensed under [Apache-2.0](LICENSE). See [governance](docs/GOVERNANCE.md) and [security policy](SECURITY.md). Contributions are welcome; participation does not imply funding, employment, or publication credit.
