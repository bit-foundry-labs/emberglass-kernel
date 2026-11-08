# Emberglass Kernel

**Bit Foundry Labs · Independent x86-64 kernel · Milestone 01**

Emberglass Kernel is original freestanding C code for an experimental operating-system kernel. It is **not** a fork of Linux and not yet a complete operating system. [Limine](https://github.com/limine-bootloader/limine) is an external bootloader, not part of our original kernel implementation.

## What's implemented
- Higher-half x86-64 ELF entry and Limine boot requests
- COM1 serial diagnostics and minimal framebuffer splash
- Validation of boot memory map and higher-half direct map presence
- Conservative, aligned 4 KiB physical-frame bump allocator
- Host allocator tests and ELF format checks

**Not yet implemented:** interrupts/IDT, page-table ownership, heap, processes, system calls, filesystem, network, AI, or a desktop. Do **not** boot on real hardware.

## Build and test
On Ubuntu or Windows WSL2:

```sh
sudo apt update
sudo apt install -y gcc make binutils python3 qemu-system-x86 xorriso curl tar
make check
make test
```

`make check` verifies the compiled ELF format, **not** successful boot. `make test` runs the allocator on the host. The ISO script attempts a pinned Limine release with a SHA-256 check; its download and boot integration must still be validated:

```sh
make iso
make run
```

Once QEMU boots successfully, expect `EMBERGLASS KERNEL` and `READY` in serial output. This has **not** yet been verified.

## Architectural boundaries
Firmware → external Limine bootloader → original kernel entry → memory management / drivers / future scheduler → future unprivileged system services.

AI models and agents will **never** execute inside privileged kernel code. Later user-space services can request audited, capability-checked actions across a syscall boundary.

See [roadmap](docs/ROADMAP.md) and [Issue #1](https://github.com/bit-foundry-labs/emberglass-kernel/issues/1).

## Licensing
Original repository code is Apache License 2.0 (see [LICENSE](LICENSE)). Limine, its boot protocol material, and any other third-party dependencies are separate projects with their own licensing obligations. Do not incorporate GPL-covered Linux kernel code without appropriate licensing review.

This is experimental research software, not production-grade or security-hardened software.
