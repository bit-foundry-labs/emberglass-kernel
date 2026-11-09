## Problem and related issue

Closes #

## Approach

What changed, and which trust boundary or kernel mechanism does it touch?

## Tests and evidence

- [ ] I ran tests on the relevant branch and reported the results.
- [ ] I included reproduction commands and QEMU serial output where applicable.
- [ ] I documented unsupported hardware and unverified claims.
- [ ] I reviewed third-party licensing and dependencies.

## Code-review standards

- [ ] Names reveal intent; public operations disclose mutations and authority changes.
- [ ] Each module/function has a coherent responsibility without speculative abstraction.
- [ ] Error paths and boundary conditions are tested (overflow, exhaustion, stale/revoked handles).
- [ ] Formatting, compiler warnings, and language-appropriate lint checks pass.
- [ ] Tests demonstrate the claimed behavior; a simulated or model-checked property is not described as a proved kernel guarantee.
- [ ] I checked the patch for unnecessary duplication, unused code, whitespace and secrets.

## Safety and limitations

What could fail? What remains unimplemented or unverified? If authority or concurrent state is involved, state the linearization point and resource lifetime assumptions.

**Reference:** [Code review standard](docs/CODE_REVIEW_STANDARD.md).
