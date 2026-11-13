# Page-table ownership: initial research contract

The bootloader currently supplies the active tables. Encoding an entry does not
grant authority over a frame or prove that it is safe to install.

Before a future switch to kernel-owned tables, the kernel must:

- Retain exclusive ownership of table frames for as long as the CPU can walk them.
- Validate physical address width and supported CPU features before constructing entries.
- Preserve the executing code, stack, descriptor tables, exception handlers,
  and required boot data during the transition.
- Specify permissions at every level, not only the leaf. Supervisor mappings
  must not become user accessible through an ancestor.
- Define synchronization, TLB invalidation, and safe reclamation before changing
  mappings or supporting more than one CPU.

The accompanying pure function models only a present 4 KiB supervisor leaf,
with an optional writable bit and a caller-supplied physical-address width.
It does not set NX; both NX support and EFER.NXE need verification before use.
The function does not authenticate frame ownership, walk tables, write physical
memory, switch CR3, or provide isolation. Host tests are encoding evidence only.
