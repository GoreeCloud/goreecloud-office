# Security

## Reporting

Do not disclose exploitable GoreeCloud security issues publicly before an authorized remediation path exists. Use the approved GoreeCloud security reporting process.

## Current source boundary

The current Office source parses untrusted ZIP/JSON document packages.

Parser rules include:

- Relative safe package paths only.
- No symlink or directory entries in the current v1 profile.
- No encrypted ZIP entries.
- Stored/Deflated compression only.
- Entry-count and uncompressed-size limits.
- Compression-ratio limit.
- Required-part validation.
- Canonical identifier validation.
- SHA-256 protected-part verification.

These controls are defense-in-depth source behavior. They do not establish Wardveil Security acceptance.

## Secrets

Never commit passwords, access tokens, private keys, production credentials, signing secrets, recovery secrets, or secret-bearing environment files.

## Dependency security

Material dependency updates require review, exact-version reconciliation, test execution, and appropriate vulnerability/security validation before release acceptance.

## Document safety

Opening a document must never automatically execute scripts, macros, embedded executables, or remote content. The v1 native Office package contract does not define executable package parts.
