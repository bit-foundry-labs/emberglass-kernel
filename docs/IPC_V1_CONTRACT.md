# IPC v1: Proposed contract

**Status:** Proposed and unimplemented. **Tracking:** [Issue #40](https://github.com/bit-foundry-labs/embers-kernel/issues/40).

Embers currently boots Rust `no_std` on x86-64/QEMU and tests early exception and physical-frame bookkeeping. It has **no** protected user processes, syscall boundary, capability enforcement, or IPC. This contract describes future behavior and must not be presented as a current security guarantee.

## Prerequisites

Before implementing IPC, the kernel must own address-space mappings, establish a tested user/supervisor boundary, authenticate the calling thread, and enforce endpoint capability rights through a protected table. A process identifier or handle submitted in a message is not an authenticated identity.

## Initial operations

`try_send(endpoint, source_buffer, length)` attempts a nonblocking copy into a kernel-owned message queue. `try_receive(endpoint, destination_buffer, capacity)` attempts a nonblocking copy of the oldest queued message and returns its length. These names are illustrative, **not** a syscall ABI.

Each endpoint has a finite maximum message size **M** and finite queue capacity **Q**, both greater than zero. Exact values and resource accounting must be selected and tested before implementation. Endpoint creation, ownership and destruction require separate authorization.

## Required observable behavior

1. **Authorization:** Every send and receive revalidates the caller's protected endpoint identity, generation, and SEND or RECEIVE right. Forged, expired and revoked authority must fail.
2. **Isolation:** The kernel validates user buffers against their actual address-space permissions and copies payloads into kernel-owned storage. A successful sender cannot later change queued bytes through an alias.
3. **Boundaries:** Each successful send enqueues exactly one length-delimited message, including zero-length messages. Receive never joins or truncates messages; an insufficient buffer yields `BufferTooSmall(required_length)` and retains the message.
4. **Ordering:** Messages are FIFO per endpoint in the order successful sends commit. Each successful receive removes one message. Concurrent senders are ordered by their serialized commit points. No ordering across different endpoints is promised.
5. **Bounded progress:** Calls do not wait for another process to run. A full queue returns `QueueFull`; an empty queue returns `WouldBlock`; an oversized message returns `MessageTooLarge`.
6. **Error behavior:** Unauthorized operations, invalid buffers, capacity errors, and failed copies do not mutate queue state. An incomplete copy to a receiver may have written part of the caller's destination buffer, but must not dequeue the message. Callers must discard destination contents on error.
7. **Revocation:** Logical revocation closes admission for later operations using that authority. Permission checking and queue-state commit must be synchronized. A send already committed before revocation remains queued for other authorized receivers; revocation does not retroactively erase delivered messages.
8. **Closure and crashes:** A closed endpoint rejects further operations. Pending data may be discarded only after outstanding uses are accounted for and safe cleanup is possible. A receiver crash must not expose queued data to unauthorized callers or leak resources indefinitely.
9. **Accounting:** Both payload storage and temporary copies must obey explicit finite budgets. Queue limits reduce denial-of-service exposure; they do not prove fairness or starvation freedom.

A successful send means **accepted into the endpoint queue**, not delivered to or processed by a receiver. Distinct error categories should include `Unauthorized`, `InvalidHandle`, `EndpointClosed`, `MessageTooLarge`, `QueueFull`, `WouldBlock`, `BufferTooSmall`, and `InvalidUserBuffer`. The implementation may refine their names but must preserve distinguishable outcomes.

## Regression requirements

| Scenario | Expected result |
| --- | --- |
| Valid send then receive | Identical bytes, exact length, one dequeue. |
| Wrong caller, wrong right, stale handle | Denial without another process's data access or queue changes. |
| Oversize payload, full queue, empty queue | Typed failure without unbounded allocation or waiting. |
| Too-small or invalid receive buffer | Message retained; no unauthorized access. |
| Sender modifies its original buffer after success | Queued bytes do not change. |
| Competing senders and receivers | No torn, duplicated, or silently lost committed messages; FIFO by commit order. |
| Revocation races with admission | Commit and revocation obey documented ordering; no new admission after logical revocation. |
| Receiver exits or endpoint closes | No unauthorized reuse or unaccounted queued memory. |

Host-level state-machine tests cannot establish user-mode isolation. Privileged QEMU fault and concurrency tests are required after the necessary memory and syscall mechanisms exist.

## Deferred from v1

Shared memory, zero-copy paths, cross-core optimizations, capability transfer inside messages, blocking semantics, priority donation, device DMA, and model-driven scheduling are excluded.

## Open implementation decisions

Choose M and Q; define endpoint ownership and cleanup, generation exhaustion, memory-pressure policy, synchronization, and safe user-memory copying when mappings change concurrently. The implementation proposal must state where send, receive, and revocation linearize.

**Acceptance of this document is not evidence of implemented IPC, a verified security boundary, or proven progress guarantees.**
