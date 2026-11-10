// Host-only test harness. Both modules remain no_std-compatible in the kernel.
#[path = "../src/memory.rs"]
mod memory;
#[path = "../src/frame_ledger.rs"]
mod frame_ledger;
