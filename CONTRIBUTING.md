# Contributing to Emberglass

The best first contribution is small enough to explain, test, and review.

## Before starting

1. Read [the README](README.md), [project roadmap](docs/CONTRIBUTOR_ROADMAP.md), and [Code of Conduct](CODE_OF_CONDUCT.md).
2. Check whether someone is already working on the issue. Comment on an open issue with your intended approach before beginning a larger change.
3. Prefer one observable behavior per pull request. For security-critical APIs, start with tests and a written threat assumption.
4. Do not claim mathematical guarantees without a precisely scoped proof, benchmark, or test.

The default branch currently has only initial metadata. **PR #2** contains the QEMU/C reference bootstrap, **PR #3** contains proposed design records, and **PR #6** contains a separate Rust policy simulation. Do not assume they are already merged.

## Choose a contribution path

- **Documentation and research:** Fix reproducibility gaps, clarify hardware assumptions, check technical references, improve the contributor experience.
- **Kernel engineering:** Small boot/exception/paging milestones with QEMU output and negative tests.
- **Rust experiments:** Host-testable, no_std-compatible libraries for capabilities, policy validation, resource budgets, simulation and property tests.

If changing APIs, provide the invariant being enforced, the trust boundary, failure behavior and how it is tested.

## Practical workflow

1. Fork the repository, create a feature branch, and reference an issue.
2. Reproduce the baseline relevant to your change. For the C bootstrap, follow README QEMU instructions on PR #2. For the Rust policy experiment, run cargo test from PR #6.
3. Implement one scoped change. Add or update tests and concise documentation.
4. Open a pull request with: problem, approach, limitations, tests run, QEMU log if relevant, and any security impact.
5. Keep communication respectful. Maintainers may request revisions, defer work, or reject incompatible designs.

Do not change unrelated code, disable failing tests, commit model credentials, include proprietary model weights, or add network/API spending as an undocumented dependency.

## Review checklist

- [ ] I linked the relevant issue or ADR.
- [ ] I described the behavior and relevant threat assumptions.
- [ ] I included a reproducible test or explain why testing is blocked.
- [ ] I differentiated simulation from real kernel behavior and hardware-specific guarantees.
- [ ] No secrets, third-party licensing violations, or unexpected downloads.
- [ ] I verified build/test behavior on the specific branch I changed.

## Compatibility, contribution rights, and credit

Apache-2.0 applies to original code contributed to this repository under its license terms. Ensure you have the right to contribute your work; identify third-party material and license requirements. We currently do not require a separate contributor license agreement. Attribution and academic authorship are based on actual contributions and applicable norms, not guaranteed by opening a pull request.

For substantial architecture changes, propose an ADR or discuss the issue before implementation.
