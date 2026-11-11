#!/usr/bin/env python3
"""Exhaustive finite-state companion, not a TLA+ parser or Rust kernel proof."""
from collections import deque
from dataclasses import dataclass, replace
from typing import FrozenSet

CORES = ("A", "B")

@dataclass(frozen=True)
class Unsafe:
    phase: str = "Active"
    checked: bool = False
    committed: bool = False
    late_commit: bool = False

def unsafe_moves(s):
    if s.phase == "Active" and not s.checked:
        yield "Core A: validate capability", replace(s, checked=True)
    if s.phase == "Active":
        yield "Core B: revoke capability", replace(s, phase="Revoked")
    if s.checked and not s.committed:
        yield "Core A: commit cached authorization", replace(
            s, committed=True, late_commit=(s.phase == "Revoked")
        )

@dataclass(frozen=True)
class Safe:
    phase: str = "Active"
    generation: int = 1
    handle_gen: tuple = (1, 1)
    holds: FrozenSet[str] = frozenset()
    committed: FrozenSet[str] = frozenset()
    device: str = "Idle"
    reclaimed: bool = False
    bad_admission: bool = False
    bad_stale: bool = False
    other_steps: int = 0

def safe_moves(s):
    for i, core in enumerate(CORES):
        if s.phase == "Active" and core not in s.holds and s.handle_gen[i] == s.generation:
            yield "Admit " + core, replace(
                s, holds=s.holds | {core},
                bad_admission=(s.phase != "Active"),
                bad_stale=(s.handle_gen[i] != s.generation))
        if core in s.holds and core not in s.committed:
            yield "Commit " + core, replace(s, committed=s.committed | {core})
        if core in s.holds:
            yield "Finish " + core, replace(s,
                holds=s.holds - {core}, committed=s.committed - {core})
        if s.phase == "Active" and s.generation == 2 and s.handle_gen[i] == 1:
            updated = list(s.handle_gen)
            updated[i] = 2
            yield "Issue fresh " + core, replace(s, handle_gen=tuple(updated))
    if s.phase == "Active":
        yield "Begin revoke", replace(s, phase="Draining")
        if s.device == "Idle":
            yield "Submit device work", replace(s, device="Running")
    if s.device == "Running":
        yield "Device completes", replace(s, device="Idle")
        yield "Device hangs", replace(s, device="Hung")
    if s.device == "Hung":
        yield "Safe device reset (assumed)", replace(s, device="Idle")
        if s.phase == "Draining":
            yield "Quarantine", replace(s, phase="Quarantined")
    if s.phase in {"Draining", "Quarantined"} and not s.holds and s.device == "Idle":
        yield "Quiesce", replace(s, phase="Quiesced")
    if s.phase == "Quiesced":
        yield "Reclaim", replace(s, phase="Reclaimed", reclaimed=True)
    if s.phase == "Reclaimed" and s.generation == 1:
        yield "Reuse slot with generation 2", replace(s,
            phase="Active", generation=2, reclaimed=False)
    if s.other_steps < 2:
        yield "Unrelated tenant progresses", replace(s, other_steps=s.other_steps + 1)

def bfs(initial, moves, invariant=None, limit=250000):
    seen = {initial}
    queue = deque([(initial, [])])
    while queue:
        state, trace = queue.popleft()
        if invariant is not None and not invariant(state):
            return len(seen), trace
        for label, next_state in moves(state):
            if next_state in seen:
                continue
            seen.add(next_state)
            if len(seen) > limit:
                raise RuntimeError("Finite-state exploration exceeded limit")
            queue.append((next_state, trace + [label]))
    return len(seen), None

def safe_invariants(s):
    return (
        s.phase in {"Active", "Draining", "Quiesced", "Quarantined", "Reclaimed"}
        and s.generation in (1, 2)
        and s.committed.issubset(s.holds)
        and not s.bad_admission and not s.bad_stale
        and (not s.reclaimed or (not s.holds and s.device == "Idle"))
        and (s.phase != "Quarantined" or not s.reclaimed)
        and (s.phase != "Reclaimed" or (not s.holds and s.device == "Idle"))
    )

def main():
    buggy_count, bad_trace = bfs(Unsafe(), unsafe_moves, lambda s: not s.late_commit)
    if bad_trace is None:
        raise SystemExit("FAIL: vulnerable check/use model had no counterexample")
    print("EXPECTED COUNTEREXAMPLE (" + str(buggy_count) + " explored states):")
    for step in bad_trace:
        print("  " + step)
    safe_count, bad_trace = bfs(Safe(), safe_moves, safe_invariants)
    if bad_trace is not None:
        raise SystemExit("FAIL: corrected model violates invariant: " + " -> ".join(bad_trace))
    print("PASS: safe reference explorer checked " + str(safe_count) + " reachable states")
    print("Not a TLA+ syntax check or proof about kernel/hardware.")

if __name__ == "__main__":
    main()
