# Embers Kernel

**An open-source, Rust-first microkernel research project for AI-oriented operating systems.**

Embers explores how an operating system can give local AI workloads access to
compute and memory while retaining explicit control over isolation, resource
budgets, and recovery. We are especially interested in desktop systems, where
interactive applications and model workloads compete for shared resources.

**This is early-stage research. Embers is not yet a protected microkernel, a
general-purpose operating system, or a benchmark-proven AI-optimized kernel.**

[Contribute](CONTRIBUTING.md) · [Research roadmap](docs/CONTRIBUTOR_ROADMAP.md) · [Open issues](https://github.com/bit-foundry-labs/embers-kernel/issues) · [Contributor launch milestone](https://github.com/bit-foundry-labs/embers-kernel/issues/24)

## Research focus

Our central question is: **How can an AI operating system share memory and
accelerator resources efficiently while preserving clear authority, safe
revocation, and responsiveness?**

We are investigating:

- **Memory and authority:** hardware-enforced address spaces, capability-based
  access, and explicit rules for sharing or transferring memory between
  isolated processes.
- **Accelerator resources:** whether device queues, memory regions, and compute
  budgets can become explicit, capability-controlled resources. Device-specific
  DMA, IOMMU mappings, completion, and reset behavior need separate analysis.
- **Revocation and recovery:** reject new work after revocation, track operations
  already admitted, and reclaim memory only after safe quiescence. When device
  access cannot be ruled out, quarantine is a candidate fail-safe.
- **Desktop responsiveness:** evaluate interference between local inference and
  interactive workloads, including frame times, input latency, audio deadlines,
  memory bandwidth, and energy where measurable.
- **Adaptive policy under kernel control:** explore isolated user-space services
  that recommend resource decisions. Kernel mechanisms must validate authority
  and limits, and provide a model-independent fallback.
- **Verification and evidence:** define invariants, exercise negative paths,
  model concurrency, and compare adaptive approaches with reproducible
  deterministic baselines. Host models and QEMU tests have different proof
  scopes; neither alone establishes real-device isolation.

These are research directions, not implemented guarantees. Tensor formats,
model runtimes, drivers, and desktop services are intended to run outside the
privileged kernel where practical. Rust helps express ownership in software; it
does not by itself constrain DMA or prove hardware behavior.

Embers is an original Rust `no_std` microkernel project targeting x86-64 and
QEMU first. It is independent of **Emberglass OS**, a Linux-based desktop
research project; the two do not currently share a working kernel.

### Research starting points

We draw on work such as [seL4 verification](https://sel4.systems/Research/publications.html)
and [scheduling contexts](https://www.sel4.systems/Research/pdfs/scheduling-context-capabilities.pdf),
[MIT xv6](https://pdos.csail.mit.edu/6.828/2020/xv6/book-riscv-rev1.pdf),
[Converos](https://www.usenix.org/conference/atc25/presentation/tang),
[XSched](https://www.usenix.org/conference/osdi25/presentation/shen-weihang),
and [Sereno](https://www.usenix.org/conference/osdi26/presentation/xin).
These systems address different questions and environments; their results are
references for our experiments, not results achieved by Embers.

## Where the code actually stands

The default branch **boots a freestanding Rust `no_std` kernel in headless QEMU** and includes early exception handling and physical-frame bookkeeping. It is not yet a protected microkernel.

| Milestone | On `main` | Evidence |
| --- | --- | --- |
| M02: Rust boot | Limine loads the ELF, bounded COM1 diagnostics, `READY` | [#26](https://github.com/bit-foundry-labs/embers-kernel/pull/26) |
| M03: CPU exceptions | Kernel GDT/TSS/IDT; deliberate #UD, #PF and #DF QEMU tests | [#27](https://github.com/bit-foundry-labs/embers-kernel/pull/27) |
| M04: memory map | Validate Limine descriptors; bounded, unique physical-frame accounting | [#28](https://github.com/bit-foundry-labs/embers-kernel/pull/28) |
| M05: frame ledger | Fixed-capacity symbolic owners, cross-owner rejection and **permanent quarantine** | [#29](https://github.com/bit-foundry-labs/embers-kernel/pull/29) |
| M06e: transactional reservations | One BSP boot path couples frame-pool and registry bookkeeping | [#53](https://github.com/bit-foundry-labs/embers-kernel/pull/53) |\n| Paging research | Host-only supervisor leaf-entry encoding; no active mapping change | [#54](https://github.com/bit-foundry-labs/embers-kernel/pull/54) |\n| Proposed research | Capability and AI-aware resource policy architecture | [ADR #3](https://github.com/bit-foundry-labs/embers-kernel/pull/3), [policy model #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6), not kernel features |

**Not implemented:** kernel-owned page tables, hardware-enforced user/kernel isolation, ring-3 processes, syscalls, IPC, authenticated capability enforcement, DMA/IOMMU quiescence or a deterministic multitasking scheduler. Frame ledger owner IDs are internal demonstration labels, not secure process identities.

A passing QEMU or host-test check proves only the tested milestone. It does **not** establish hardware isolation, kernel-level capability enforcement, production security, scheduler superiority, or a functioning AI operating system.

**Next proposed engineering boundary:** [M06 page-table transition contract](docs/PAGING_TRANSITION_V1.md) specifies the required memory mappings, frame reservations, failure behavior, and QEMU evidence before Embers can safely replace Limine's page-table root. It is a review proposal, not a completed milestone.

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

The serial log should contain `READY: Embers Rust no_std entry reached.`. The intentionally halted kernel keeps QEMU running; stop the emulator with Ctrl+C. The measured emulator baseline is documented in [the Rust boot notes](rust-kernel/README.md): `-cpu max` is retained as a control profile and `qemu64` is the current minimum **tested emulator** model. This is not a minimum hardware requirement.

Run the pure-Rust memory and ownership tests independently of QEMU:

```bash
make rust-memory-test
make rust-ledger-test
```

For the four-mode exception smoke test, see [M03 exception instructions](rust-kernel/M03_EXCEPTIONS.md); for allocation and ownership semantics see [M04](rust-kernel/M04_PHYSICAL_MEMORY.md) and [M05](rust-kernel/M05_FRAME_LEDGER.md). The [CI workflows](.github/workflows/) run these against supported development-emulator configurations.

The earlier C prototype remains accessible as [historical PR #2](https://github.com/bit-foundry-labs/embers-kernel/pull/2). It is **not** required to build or execute the Rust kernel.

For the host-only policy experiment, check out [PR #6](https://github.com/bit-foundry-labs/embers-kernel/pull/6) and run `cargo test` in `experiments/rust-policy-guard`. That does not execute a privileged Rust kernel.

## Help build it

We're looking for contributors interested in **Rust systems programming, kernel bring-up, x86-64/QEMU, capability security, IPC, concurrency, scheduling, model checking, and reproducible benchmarks**.

A useful first contribution is a narrow, verifiable improvement, not a sweeping rewrite:

- **First contribution, documentation:** [Reproduce the Rust QEMU boot on a fresh Ubuntu/WSL machine](https://github.com/bit-foundry-labs/embers-kernel/issues/8).
- **First contribution, code review:** [Document the early-boot `unsafe` contracts](https://github.com/bit-foundry-labs/embers-kernel/issues/33).
- **Architecture:** Review the [proposed page-table transition contract](docs/PAGING_TRANSITION_V1.md) and its unresolved assumptions.

For deeper research, see the [capability threat-model task](https://github.com/bit-foundry-labs/embers-kernel/issues/12) and the [contributor-launch milestone](https://github.com/bit-foundry-labs/embers-kernel/issues/24).

Read [the contribution guide](CONTRIBUTING.md), [research roadmap](docs/CONTRIBUTOR_ROADMAP.md), [review standard](docs/CODE_REVIEW_STANDARD.md), and [Code of Conduct](CODE_OF_CONDUCT.md). Comment on an issue before undertaking substantial work.

## License and stewardship

An experimental open-source project stewarded by [Bit Foundry Labs](https://github.com/bit-foundry-labs), licensed under [Apache-2.0](LICENSE). See [governance](docs/GOVERNANCE.md) and [security policy](SECURITY.md). Contributions are welcome; participation does not imply funding, employment, or publication credit.
