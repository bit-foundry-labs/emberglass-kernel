# Security policy

Emberglass is an **experimental research kernel**, not a security-hardened OS. Do not use it for secrets, production workloads, or deployment on physical hardware. Kernel, driver and simulator security guarantees are not established by a successful CI run.

## Report a possible vulnerability
Please avoid posting working exploits, credentials, or sensitive traces in a public GitHub issue.

If the GitHub repository offers **Security → Report a vulnerability**, use that private channel. If private reporting is not enabled, create a public issue containing only a request for a secure reporting contact, with **no sensitive details**. Maintainers should configure private vulnerability reporting before broader recruitment.

## Research priorities
We particularly welcome tests covering capability rights amplification, stale/revoked handles, unsafe DMA lifecycle, kernel privilege transitions, input validation, scheduler starvation and fail-open AI policy paths.

No bounty or formal disclosure service is currently offered.
