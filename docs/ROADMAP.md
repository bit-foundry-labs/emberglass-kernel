# Emberglass Kernel Engineering Roadmap

## Milestone 01 · Bootstrap
- [x] 64-bit freestanding higher-half ELF
- [x] Serial logging and framebuffer primitive
- [x] Conservative bootstrap physical-frame allocation
- [x] Host unit tests and ELF structural checks
- [ ] Confirm QEMU boot with pinned Limine release and serial evidence

## Milestone 02 · CPU faults
- [ ] Install GDT and a full IDT with exception stubs
- [ ] Introduce separate exception and double-fault stacks (TSS / IST)
- [ ] Serial panic output: vector, error code, instruction pointer, registers
- [ ] Test deliberate divide-by-zero and page-fault traps in emulator

## Milestone 03 · Memory
- [ ] Track physical-frame ownership across noncontiguous regions
- [ ] Own page tables; enforce supervisor/user and W^X boundaries
- [ ] Kernel heap with allocation failure tests

## Milestone 04 · Execution
- [ ] Kernel threads, context switch, timer and scheduler
- [ ] Ring-3 user execution and syscall ABI
- [ ] Validated userspace copying and fault isolation

## Milestone 05 · Services
- [ ] IPC and capability handles
- [ ] Virtual filesystem, storage, input and drivers
- [ ] User-space AI broker, with approvals, budgets and evidence

Linux-based Emberglass desktop can evolve independently while the original kernel matures. Real hardware support is a longer-term milestone.
