# M06a: Limine address provenance preflight

**Status:** Early-boot metadata validation, not kernel-owned paging.
**Scope:** One BSP, x86-64/QEMU with Limine base revision 6, Rust `no_std`.

## Purpose

The first Embers-owned page tables will require a trustworthy account of
which physical addresses back the executable and which physical memory
regions are reachable under Limine's temporary higher-half direct map.
Do not assume an identity map or a hard-coded direct-map offset.

This increment requests two Limine revision-0 features:

- **HHDM** (`0x48dcf1cb8ad2b852, 0x63984e959a98244b`): returns an offset
  for a higher-half direct map. It is not a promise that *every* physical
  address is mapped. Base revision 6 maps only specified memory-map types.
- **Executable address** (`0x71ba76863cc55f63, 0xb2644a48c516a487`):
  returns the loaded executable's physical and virtual base addresses.

[`boot_addresses.rs`](src/boot_addresses.rs) reads and copies these
responses during the Limine handoff while their mappings are live.
[`address_provenance.rs`](src/address_provenance.rs) independently checks
4 KiB alignment, 48-bit canonical higher-half addresses, and checked
physical-plus-offset arithmetic. The normal boot checks a candidate
translation for two uniquely claimed USABLE frames and emits an
`ADDRESS: Limine HHDM and executable bases validated` marker.

## Important trust limits

- The response pointers themselves remain **trusted bootloader pointers**.
  A nonnull pointer does not prove alignment, mapping validity, initialization
  or provenance. Unsafe response reads rely on Limine's handoff contract.
- Numeric address arithmetic never dereferences a translated address,
  zeroes frames, writes page tables or changes `CR3`. A candidate address
  is **not** permission to access memory.
- The frame candidates come from the validated USABLE range selected by
  M04. It is incorrect to apply the HHDM offset to arbitrary addresses
  without checking the memory-map kind and current bootloader mappings.
- These arithmetic checks target the current four-level, 48-bit canonical
  profile. The kernel does not yet read CR4.LA57 or establish a minimum
  CPU feature baseline. The future paging transition must verify the
  actual active mode and its permissions.
- M06a's `FramePool` is still ephemeral bookkeeping. Independently
  constructed overlapping pools can refer to the same physical frames;
  no DMA/IOMMU safety or persistent ownership is established.
- Executable base addresses provide mapping provenance, not proof that
  every ELF segment is protected or that later accesses will be valid.

## Reproduce

With the repository's pinned Rust toolchain and QEMU requirements:

```sh
make rust-provenance-test
make rust-memory-test
make rust-ledger-test
make rust-iso
make rust-run
```

The provenance host tests exercise valid values, incorrect offset/bases,
alignment failure, checked overflow, and the distinction between computing
a candidate address and proving that it is mapped.

M02 QEMU CI checks the `ADDRESS` success marker. M04/M05 CI runs the
host provenance tests and requires all three numeric response fields
to be emitted in the QEMU serial log. The existing M03 fault tests remain
separate regression evidence.

## Next boundary

Before **any** `CR3` switch, verify hardware paging mode, map permissions,
the executable's physical span, safe writable HHDM access for explicitly
reserved table frames, table initialization, active stacks and exception
delivery, and a persistent nonoverlapping frame-ownership source.
See [M06 transition specification](../docs/PAGING_TRANSITION_V1.md).

Reference: [Limine protocol](https://github.com/limine-bootloader/limine-protocol/blob/trunk/PROTOCOL.md),
sections *Memory Layout at Entry*, *HHDM*, and *Executable Address*.
