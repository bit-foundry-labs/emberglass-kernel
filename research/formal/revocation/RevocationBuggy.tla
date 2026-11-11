--------------------------- MODULE RevocationBuggy ---------------------------
EXTENDS Naturals

\* Deliberately broken check-then-use protocol. Two independent cores can
\* interleave these actions; TLC should produce a counterexample trace.
VARIABLES phase, checked, committed, lateCommit
vars == <<phase, checked, committed, lateCommit>>

Init ==
    /\ phase = "Active"
    /\ checked = FALSE
    /\ committed = FALSE
    /\ lateCommit = FALSE

CoreAValidate ==
    /\ phase = "Active"
    /\ ~checked
    /\ checked' = TRUE
    /\ UNCHANGED <<phase, committed, lateCommit>>

CoreBRevoke ==
    /\ phase = "Active"
    /\ phase' = "Revoked"
    /\ UNCHANGED <<checked, committed, lateCommit>>

CoreACommit ==
    /\ checked
    /\ ~committed
    /\ committed' = TRUE
    /\ lateCommit' = (phase = "Revoked")
    /\ UNCHANGED <<phase, checked>>

Next == CoreAValidate \/ CoreBRevoke \/ CoreACommit

TypeOK ==
    /\ phase \in {"Active", "Revoked"}
    /\ checked \in BOOLEAN
    /\ committed \in BOOLEAN
    /\ lateCommit \in BOOLEAN

NoLateCommit == ~lateCommit
=============================================================================
