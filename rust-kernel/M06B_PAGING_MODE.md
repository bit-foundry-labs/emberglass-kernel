# M06b: Active paging-mode preflight

**Status:** Read-only CPU state preflight. Not kernel-owned paging.

At the single BSP early-boot checkpoint, Embers reads CR0 and CR4 at CPL0.
The safe validator requires CR0.PG = 1, CR4.PAE = 1, and CR4.LA57 = 0.
It emits `PAGING: active four-level mode validated` only after validation;
otherwise it emits a specific `ERROR:` message and halts.

The five dependency-free host tests exercise accepted bit patterns, each
rejected prerequisite, and deterministic multi-error behavior. The QEMU boot
workflow requires the positive serial checkpoint.

## Boundaries

- The x86-64 Limine long-mode handoff is trusted. This increment does not
  inspect EFER.LMA, CPUID capabilities, CR3, actual table hierarchy,
  permissions, memory ownership, or page-table entry content.
- CR0 and CR4 are sampled on the BSP; there is no AP consistency check.
- The checks never modify control registers, page tables, or virtual mappings.
- A successful result is necessary for the current 48-bit canonical-address
  assumptions, but **not sufficient** for a safe transition to owned tables.
- Host tests do not exercise privileged register reads; QEMU CI must provide
  that evidence before merging.

## Reproduce

```sh
make rust-paging-mode-test
make rust-iso
make rust-run
```

Next: independent, persistent frame ownership and safely zeroed writable
page-table frames. Any future CR3 transition requires the separate M06
transition contract and positive plus negative QEMU evidence.
