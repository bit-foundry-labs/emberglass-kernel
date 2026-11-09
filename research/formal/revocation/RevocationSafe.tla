---------------------------- MODULE RevocationSafe ----------------------------
EXTENDS Naturals, FiniteSets

\* Bounded two-core and one simulated-device abstraction.
\* Trusted caller identity and synchronized cap-table operations
\* are modeled as atomic actions, not implemented or proved here.
Cores == {"A", "B"}
Phases == {"Active", "Draining", "Quiesced", "Quarantined", "Reclaimed"}
DeviceStates == {"Idle", "Running", "Hung"}

VARIABLES phase, generation, handles, holds, committed, device,
          reclaimed, lateAdmission, staleAdmission, unrelatedSteps
vars == <<phase, generation, handles, holds, committed, device,
          reclaimed, lateAdmission, staleAdmission, unrelatedSteps>>

Init ==
    /\ phase = "Active"
    /\ generation = 1
    /\ handles = [c \in Cores |-> 1]
    /\ holds = {}
    /\ committed = {}
    /\ device = "Idle"
    /\ reclaimed = FALSE
    /\ lateAdmission = FALSE
    /\ staleAdmission = FALSE
    /\ unrelatedSteps = 0

\* Admission creates a tracked hold atomically with the cap check.
Admit(c) ==
    /\ phase = "Active"
    /\ c \notin holds
    /\ handles[c] = generation
    /\ holds' = holds \cup {c}
    /\ lateAdmission' = (phase # "Active")
    /\ staleAdmission' = (handles[c] # generation)
    /\ UNCHANGED <<phase, generation, handles, holds, committed, device,
                  reclaimed, unrelatedSteps>>

\* In-flight work may finish under an existing admission hold.
Commit(c) ==
    /\ c \in holds
    /\ c \notin committed
    /\ committed' = committed \cup {c}
    /\ UNCHANGED <<phase, generation, handles, holds, device,
                  reclaimed, lateAdmission, staleAdmission, unrelatedSteps>>

Finish(c) ==
    /\ c \in holds
    /\ holds' = holds \ {c}
    /\ committed' = committed \ {c}
    /\ UNCHANGED <<phase, generation, handles, device,
                  reclaimed, lateAdmission, staleAdmission, unrelatedSteps>>

BeginRevoke ==
    /\ phase = "Active"
    /\ phase' = "Draining"
    /\ UNCHANGED <<generation, handles, holds, committed, device,
                  reclaimed, lateAdmission, staleAdmission, unrelatedSteps>>

SubmitDevice ==
    /\ phase = "Active"
    /\ device = "Idle"
    /\ device' = "Running"
    /\ UNCHANGED <<phase, generation, handles, holds, committed, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

DeviceComplete ==
    /\ device = "Running"
    /\ device' = "Idle"
    /\ UNCHANGED <<phase, generation, handles, holds, committed, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

DeviceHang ==
    /\ device = "Running"
    /\ device' = "Hung"
    /\ UNCHANGED <<phase, generation, handles, holds, committed, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

\* This recovery action assumes a *provably safe* device reset.
\* Real hardware may not offer it; no universal timeout is promised.
ResetDevice ==
    /\ device = "Hung"
    /\ device' = "Idle"
    /\ UNCHANGED <<phase, generation, handles, holds, committed, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

Quarantine ==
    /\ phase = "Draining"
    /\ device = "Hung"
    /\ phase' = "Quarantined"
    /\ UNCHANGED <<generation, handles, holds, committed, device, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

Quiesce ==
    /\ phase \in {"Draining", "Quarantined"}
    /\ holds = {}
    /\ device = "Idle"
    /\ phase' = "Quiesced"
    /\ UNCHANGED <<generation, handles, holds, committed, device, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

Reclaim ==
    /\ phase = "Quiesced"
    /\ phase' = "Reclaimed"
    /\ reclaimed' = TRUE
    /\ UNCHANGED <<generation, handles, holds, committed, device,
                  lateAdmission, staleAdmission, unrelatedSteps>>

Reuse ==
    /\ phase = "Reclaimed"
    /\ generation = 1
    /\ generation' = 2
    /\ phase' = "Active"
    /\ reclaimed' = FALSE
    /\ UNCHANGED <<handles, holds, committed, device,
                  lateAdmission, staleAdmission, unrelatedSteps>>

\* Trusted reissue. An old handle alone is rejected after slot reuse.
IssueFresh(c) ==
    /\ phase = "Active"
    /\ generation = 2
    /\ handles[c] = 1
    /\ handles' = [handles EXCEPT ![c] = 2]
    /\ UNCHANGED <<phase, generation, holds, committed, device, reclaimed,
                  lateAdmission, staleAdmission, unrelatedSteps>>

\* Other work can remain enabled; progress latency is not guaranteed.
UnrelatedProgress ==
    /\ unrelatedSteps < 2
    /\ unrelatedSteps' = unrelatedSteps + 1
    /\ UNCHANGED <<phase, generation, handles, holds, committed, device,
                  reclaimed, lateAdmission, staleAdmission>>

Next ==
    \/ (\E c \in Cores : Admit(c))
    \/ (\E c \in Cores : Commit(c))
    \/ (\E c \in Cores : Finish(c))
    \/ BeginRevoke
    \/ SubmitDevice
    \/ DeviceComplete
    \/ DeviceHang
    \/ ResetDevice
    \/ Quarantine
    \/ Quiesce
    \/ Reclaim
    \/ Reuse
    \/ (\E c \in Cores : IssueFresh(c))
    \/ UnrelatedProgress

TypeOK ==
    /\ phase \in Phases
    /\ generation \in {1, 2}
    /\ handles \in [Cores -> {1, 2}]
    /\ holds \subseteq Cores
    /\ committed \subseteq Cores
    /\ device \in DeviceStates
    /\ reclaimed \in BOOLEAN
    /\ lateAdmission \in BOOLEAN
    /\ staleAdmission \in BOOLEAN
    /\ unrelatedSteps \in 0..2

NoUntrackedCommit == committed \subseteq holds
NoPostRevokeAdmission == ~lateAdmission
NoStaleHandleAdmission == ~staleAdmission
NoEarlyReclaim == reclaimed => (holds = {} /\ device = "Idle")
QuarantineRetainsMemory == (phase = "Quarantined") => ~reclaimed
ReclaimedIsQuiescent == (phase = "Reclaimed") => (holds = {} /\ device = "Idle")
=============================================================================
