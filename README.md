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

`make check` verifies the compiled ELF format, **not** successful boot. `make test` runs the allocator on the host. The ISO script fetches pinned Limine v12.9.6 with SHA-256 verification. GitHub Actions successfully built this ISO and booted our kernel with QEMU's `-cpu max` on November 8, 2026:

```sh
make iso
make run
```

The `make run` target uses `-cpu max`, because QEMU's conservative default `qemu64` CPU model caused a `#UD` invalid-opcode fault during early graphics. The exact minimum CPU requirements should be established before supporting real hardware.

For a headless WSL or SSH session (serial output only):

```sh
qemu-system-x86_64 -cpu max -m 256M -cdrom build/emberglass.iso \
  -display none -serial stdio -monitor none -no-reboot
```

Expected serial output includes `DISPLAY: done`, `MEMORY:`, and `READY: original freestanding kernel entered successfully.`. This was verified by [GitHub Actions QEMU smoke test](https://github.com/bit-foundry-labs/emberglass-kernel/actions/workflows/qemu-boot.yml). The CPU model is currently a developer convenience rather than a stable production requirement; further runtime tests are necessary.

## Architectural boundaries
Firmware → external Limine bootloader → original kernel entry → memory management / drivers / future scheduler → future unprivileged system services.

AI models and agents will **never** execute inside privileged kernel code. Later user-space services can request audited, capability-checked actions across a syscall boundary.

See [roadmap](docs/ROADMAP.md), [kernel design review](docs/KERNEL_DESIGN_REVIEW.md), and [Issue #1](https://github.com/bit-foundry-labs/emberglass-kernel/issues/1).

## Licensing
Original repository code is Apache License 2.0 (see [LICENSE](LICENSE)). Limine, its boot protocol material, and any other third-party dependencies are separate projects with their own licensing obligations. Do not incorporate GPL-covered Linux kernel code without appropriate licensing review.

This is experimental research software, not production-grade or security-hardened software.
