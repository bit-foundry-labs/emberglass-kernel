# Protecting Embers Kernel's main branch

This repository is a research kernel. A green check establishes that specific
host or QEMU tests passed; it **does not** establish security or real-hardware
correctness. The rules below defend the shared development baseline.

## Prerequisite

The four jobs must run on **every pull request**, even documentation-only
changes. Required checks skipped by a workflow-level path filter can stay
Pending forever. The workflows deliberately retain filtered `push` triggers
but have *unfiltered* `pull_request` triggers, so all four run before a merge.

Do not configure required checks until the prerequisite CI workflow change is
merged into `main` and a pull request confirms the jobs appear.

## Repository administrator setup

Go to:
https://github.com/bit-foundry-labs/embers-kernel/settings/rules

In **Settings → Rules → Rulesets**, create a **New branch ruleset**:

| Item | Setting |
| --- | --- |
| Name | `Protect main: reviews and kernel CI` |
| Enforcement | **Active** |
| Target | Branch, include default branch (`main` only) |
| Pull requests | **Require a pull request before merging** |
| Merge checks | **Require status checks to pass** |
| Review discussion | **Require conversation resolution** |
| History | **Block force pushes**, **block deletions** |
| Bypass | No general bypass actors; do not add workflow bots as administrators |

Set these exact required **GitHub Actions job names**, selecting the GitHub
Actions source if prompted:

1. `whitespace` — Source hygiene
2. `rust-boot` — freestanding Rust/ELF/BIOS-QEMU smoke boot
3. `exceptions` — normal/#UD/#PF/#DF QEMU runs
4. `memory` — host map/ledger tests plus QEMU accounting checks

If GitHub lists a combined label such as
`Embers M03 exception traps in QEMU / exceptions`, select the
**matching existing check run** rather than typing a different name. A
fresh PR against main can populate the choices.

**Required approvals:** Set **1** only when at least one *other*
authorized contributor can review PRs. GitHub doesn't count your own
approval toward your own PR; setting 1 before another reviewer exists
can block all regular progress. Until a second reviewer is appointed,
enable the PR requirement and required CI with **0** approvals, review
changes manually, then raise to 1 before broad contributor recruitment.
This is a temporary policy compromise, not the final safety target.

Recommended:
- Require branches to be up to date before merging, when practical.
- Dismiss stale approvals if using required approvals.
- Do not enable a merge queue until workflows support `merge_group`.
- Leave "Allow force pushes" and "Allow deletions" **disabled**.
- Keep bypass disabled and include administrators in enforcement.

If the "Rulesets" UI is unavailable for your organization plan, use
**Settings → Branches → Add classic branch protection rule** for `main`.
Apply equivalent pull request, required checks and protections.

## Verify enforcement

1. Open a documentation-only PR. Confirm that the four required jobs
   **run** and appear on the PR (not Pending due to a missing workflow).
2. Confirm an intentionally failing required check blocks merge.
3. Confirm direct pushes to `main` are blocked for an ordinary developer.
4. Confirm review approval is required *if* the approval rule is enabled.
5. Record the observed protection state and an example blocked PR in
   [launch readiness #24](https://github.com/bit-foundry-labs/embers-kernel/issues/24).

**Do not mark #24's branch-protection gate complete based only on this
documentation or green CI. Check the active ruleset in GitHub settings.**
