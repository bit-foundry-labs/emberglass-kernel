# EMBERGLASS
### An original Rust-first operating-system research project

**Kernel engineering, capability security, and adaptive computing.**

Emberglass is an open-source experiment from **Bit Foundry Labs**. We want to understand what an operating system could look like if applications and AI agents could request computing resources intelligently, while the kernel retained final authority over memory, execution time, and hardware access.

The long-term direction is an **original Rust no_std capability-based microkernel**, not a Linux fork. AI models and agents belong in isolated user-space services; the privileged kernel must never depend on a model to make forward progress.

**This is early-stage research, not a usable general-purpose OS.** No production security, hardware-driver, AI-scheduler, or formal-verification claims are made.

## What exists today?

| Track | What exists | Where |
| --- | --- | --- |
| Boot reference | An experimental x86-64 **C bootstrap** with serial diagnostics and an allocator; its CI reached READY under BIOS QEMU using an explicit CPU model | [PR #2](https://github.com/bit-foundry-labs/emberglass-kernel/pull/2) |
| Architecture | Proposed capability-based Rust design, inference permissions, resource budgeting and research bibliography | [PR #3](https://github.com/bit-foundry-labs/emberglass-kernel/pull/3) |
| Rust research | Host-testable no_std-compatible policy guard and CPU admission *model*, not a running Rust kernel | [PR #6](https://github.com/bit-foundry-labs/emberglass-kernel/pull/6) |

**Repository state:** the default branch currently holds only initial project metadata. The work above is under review, not merged into main. Start from the referenced pull requests rather than assuming that main can boot.

## Research questions

- Can explicit capability boundaries prevent an untrusted AI policy service from acquiring additional CPU, memory or accelerator authority?
- Can adaptive scheduling hints improve interactive latency without starving low-priority applications?
- How should model invocations, compute budgets, and memory grants be represented without making a neural-network runtime part of the trusted kernel?
- What is the measured cost of isolation, IPC, capability checks, and safe fallback?

An interesting experiment that fails to outperform its deterministic baseline is still useful research.

## Try the QEMU bootstrap

**Development VM only. Do not boot this on physical hardware.** These commands target Ubuntu or WSL2 Ubuntu:

~~~bash
git clone https://github.com/bit-foundry-labs/emberglass-kernel.git
cd emberglass-kernel
git fetch origin pull/2/head:kernel-bootstrap-m01
git switch kernel-bootstrap-m01

sudo apt-get update
sudo apt-get install -y build-essential binutils python3 qemu-system-x86 xorriso curl tar

make check
make test
make iso
# On a host with a working graphical display:
make run
~~~

The bootstrap's Makefile uses QEMU with -cpu max to avoid a CPU-feature mismatch observed in earlier emulator testing. For a headless boot, after make iso:

~~~bash
qemu-system-x86_64 -cpu max -m 256M -cdrom build/emberglass.iso -display none -serial stdio -monitor none -no-reboot
~~~

Expected serial output ends with READY: original freestanding kernel entered successfully. This is a boot milestone, not a functional desktop.

## Try the Rust policy experiment

~~~bash
git fetch origin pull/6/head:rust-policy-guard
git switch rust-policy-guard
cd experiments/rust-policy-guard
cargo test
~~~

This library models policy validation only; it is not a scheduler and does not implement a hardware security boundary.

## Contribute

Good contributions include reproducible QEMU tests, CPU exception diagnostics, Rust bare-metal bring-up, protocol reviews, clear documentation and rigorous critiques of our safety claims.

Read [CONTRIBUTING.md](CONTRIBUTING.md), [the contributor roadmap](docs/CONTRIBUTOR_ROADMAP.md), and [our conduct guidelines](CODE_OF_CONDUCT.md). Check [GitHub issues](https://github.com/bit-foundry-labs/emberglass-kernel/issues) for a small, scoped first task, and discuss architectural changes before implementing large subsystems.

We welcome research that challenges our hypotheses. We prefer a measured counterexample to a grand untested claim.

## Project stewardship and licensing

The project is maintained under Bit Foundry Labs. Original repository content is available under [Apache License 2.0](LICENSE); third-party dependencies carry their own licenses. See [GOVERNANCE.md](docs/GOVERNANCE.md) and [SECURITY.md](SECURITY.md).

Research collaboration and code contributions are welcome; participation does not create employment, funding, or publication guarantees.
