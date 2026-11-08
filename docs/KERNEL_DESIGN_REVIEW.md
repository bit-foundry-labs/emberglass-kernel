# Kernel design review · 2026-11-08

This is a research and safety review of the Milestone 01 prototype, **not** a claim of production readiness.

## Guiding requirements

1. **Original kernel implementation:** x86-64 system services in our own freestanding code; external bootloader (Limine) permitted.
2. **Verifiable milestones:** every milestone must demonstrate the behavior in QEMU, not merely compile.
3. **Security boundaries:** AI and app code in unprivileged user space only, with explicit capabilities enforced by the kernel.
4. **Predictable behavior:** interrupt handlers, memory allocation, and scheduler never delegate to nondeterministic models.
5. **Keep architectural options open:** starting modular-monolithic avoids premature IPC overhead; microkernel-style isolation is a later ADR.

## Current architecture decisions

| Decision | Rationale | Status |
| --- | --- | --- |
| Limine bootloader + higher-half ELF | Modern portable handoff; 64-bit paging and framebuffer are supported | reasonable, verify in QEMU |
| x86-64 first | Target Windows/WSL QEMU development and commodity CPUs | accepted |
| C11 freestanding kernel | Minimal dependencies, transparent ABI, incremental systems learning | provisional |
| 4 KiB aligned bootstrap allocation | Deliberately tiny allocator for boot; no frees or ownership table | experimental |
| Serial diagnostics first | Debuggable even without display | accepted |
| AI outside kernel | Reduces kernel attack surface and unpredictable privileged execution | mandatory |
| Apache-2.0 original code | Matches repository license | external attributions need audit |

## Findings and planned corrections

### P0 · Boot and failure observability
- [x] Verify `READY` in QEMU with `-cpu max` (GitHub Actions run 42513742505).
- [x] Diagnose framebuffer stall: emulator default `qemu64` raised `#UD` in early drawing; explicit `-cpu max` boots and reaches `READY`. Establish conservative supported CPU baseline next.
- [ ] Record BIOS and UEFI emulator checks separately.
- [ ] Add `-s -S` GDB debug instructions and QEMU `-d int,guest_errors` fault trace instructions.

### P1 · Toolchain and ABI
- [ ] Replace host-environment-dependent compiler assumptions with a pinned freestanding cross-target (e.g., Clang `-target x86_64-unknown-none-elf` or `x86_64-elf-gcc`).
- [ ] Verify SysV x86-64 calling convention (without FP/SIMD), disabled red zone, stack alignment, linker section and PHDR permissions.
- [ ] Prefer a reviewed, pinned official Limine protocol header over hand-transcribed ABI structs; track version and license in a dependency manifest.

### P1 · Memory safety and interrupt handling
- [ ] Implement a GDT, IDT, TSS and double-fault IST before more complex kernel features.
- [ ] Log trap vector, fault error code, RIP and CR2 (page faults) over serial.
- [ ] Treat the physical allocator as boot-only; later track ownership, freeing, zero-on-allocation, and reserved regions.
- [ ] Introduce page table ownership and W^X, supervisor/user separation before ring 3.

### P2 · Engineering methodology
- [ ] Test hostile and malformed boot inputs, overflow, out-of-range framebuffers, allocator exhaustion, absent Limine responses.
- [x] Add QEMU BIOS smoke test to CI with serial readiness assertion. Later add UEFI and intentional divide-by-zero/page-fault tests.
- [ ] Use signed/pinned external release artifacts and an explicit license/notice inventory.
- [ ] Create an architecture decision record comparing modular monolith, hybrid, and microkernel goals (IPC cost, fault isolation, driver complexity, maintainability).

## Research foundation (primary sources)
- Intel 64 and IA-32 Software Developer's Manual, Volume 3 (System Programming Guide): https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
- Limine protocol specification: https://github.com/limine-bootloader/limine-protocol/blob/trunk/PROTOCOL.md
- Limine bootloader usage: https://github.com/limine-bootloader/limine/blob/v12.x/USAGE.md
- OSDev Limine Bare Bones and boot ISO walkthrough: https://wiki.osdev.org/Limine_Bare_Bones
- QEMU system emulator manual: https://www.qemu.org/docs/master/system/invocation.html
- MIT 6.1810 Operating System Engineering (xv6 course, 2026): https://pdos.csail.mit.edu/6.S081/2026/schedule.html

This document is an evolving research-based checklist. A full threat model and comparative kernel-design study have **not** yet been completed.
