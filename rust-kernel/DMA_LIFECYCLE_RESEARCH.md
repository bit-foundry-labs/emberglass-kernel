# DMA lifecycle research model

Status: host-only simulation; not a DMA driver, Kani proof, IOMMU setup, or
hardware security guarantee. One buffer, one simulated queue and at most one
operation are tracked by a single trusted state owner. No device, CPU alias,
memory barrier, IOTLB, reset implementation, or cache is modeled.

## Contract under investigation

After revocation, new admissions fail. A buffer may become reusable only after
the one previously admitted operation has a matching completion and the modeled
CPU synchronization step is complete. If hardware completion cannot be
established, quarantine is terminal in this model. Sequence and generation
counters reject stale completions and fail closed at overflow.

The completion method assumes a trusted adapter has correctly validated a
device completion and that it means all modeled device access has ended. This
is an assumption, not something a Rust value proves. Real implementations must
define DMA mapping, CPU and I/O translation invalidation, device-specific
fences, memory ordering, cache coherence, aliases, resets, and multiple
concurrent operations before reclaiming physical frames.

The counterexample test demonstrates that revocation alone can coincide with
possible device access. It is an ordinary executable test, not a model-checker
counterexample. Kani harnesses and a reproducible bounded verification run
remain future work. Explore whether reset can establish quiescence using a
specific device specification; no generic reset guarantee is assumed.

References: [VirtIO 1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html),
[Linux DMA API guide](https://docs.kernel.org/core-api/dma-api-howto.html),
[SADA, USENIX Security 2021](https://www.usenix.org/conference/usenixsecurity21/presentation/bai),
and [Kani loop unwinding](https://model-checking.github.io/kani/tutorial-loop-unwinding.html).
