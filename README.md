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

The default branch currently contains project documentation and contributor guidance, **not an integrated bootable Rust microkernel**. Experimental implementation work is in open, sometimes stacked pull requests:

| Track | Experimental work | Status |
| --- | --- | --- |
| x86-64 boot reference | A C bootstrap with QEMU serial diagnostics and allocator tests | [PR #2](https://github.com/bit-foundry-labs/embers-kernel/pull/2), not merged |
| Architecture | Proposed capability boundaries, resource budgets, and AI policy separation | [PR #3](https://github.com/bit-foundry-labs/embers-kernel/pull/3), not merged |
| Rust policy research | Host-testable `no_std`-compatible validation and CPU-admission *model* | [PR #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6), not a running kernel |
| Rust boot progression | Experimental Limine/QEMU Rust entry and subsequent exception/memory work | [PR #20](https://github.com/bit-foundry-labs/embers-kernel/pull/20) onward, stacked and under review |

A passing QEMU or host-test check proves only the tested milestone. It does **not** establish hardware isolation, kernel-level capability enforcement, production security, scheduler superiority, or a functioning AI operating system.

## Try the experimental boot reference

**QEMU development environment only. Do not boot this on physical hardware.** On Ubuntu or WSL2 Ubuntu, the C reference is currently available from a review branch:

```bash
git clone https://github.com/bit-foundry-labs/embers-kernel.git
cd embers-kernel
git fetch origin pull/2/head:boot-reference
git switch boot-reference

sudo apt-get update
sudo apt-get install -y build-essential binutils python3 qemu-system-x86 xorriso curl tar
make check
make test
make iso
```

On a headless host, you can inspect serial output with:

```bash
qemu-system-x86_64 -cpu max -m 256M -cdrom build/emberglass.iso \
  -display none -serial stdio -monitor none -no-reboot
```

The `emberglass.iso` output filename is retained by the older bootstrap branch; the project name is **Embers Kernel**. The reference's expected serial output ends in `READY` when the tested boot path succeeds.

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
