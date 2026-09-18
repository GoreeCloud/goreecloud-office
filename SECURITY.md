# Security

## Current Security Boundary

The current repository contains local Rust library code and GitHub Actions validation. It does not currently expose a network listener, authentication endpoint, document parser, persistent file store, or deployed service.

## Secure Development Requirements

- Unsafe Rust is forbidden in the current foundation crates.
- Inputs and document mutations must be validated at trust and representation boundaries.
- Untrusted document imports and embedded content must be treated as hostile until appropriate Wardveil Security controls exist.
- Secrets, tokens, private keys, passwords, and production credentials must never be committed.
- Dependencies and GitHub Actions must remain reviewed and securely maintained.
- Security-sensitive changes require exact-revision validation and appropriate review.
- Future local temporary files, persistence, synchronization, collaboration, and export paths must use explicit authorization and safe-failure behavior.

## Vulnerability Reporting

Do not publish active secrets, private exploit details, or sensitive user information in public issues. Use an approved private security-reporting channel when one is configured for this repository. Non-sensitive security hardening discussions may use normal repository collaboration workflows.

## Platform Security

Wardveil Security is the applicable GoreeCloud security authority for trust signals, malicious-content handling, risky imports, integrity protection, and security evidence. No Wardveil runtime conformance is currently claimed.

## Release Boundary

Passing CI does not establish security acceptance, production readiness, or Stable status.