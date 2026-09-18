# Notes

## Current implementation boundary

The initial source foundation is intentionally limited to the Writer semantic model, Office Native Package v1 records, bounded ZIP construction/validation, and test/CI infrastructure.

## Pre-bootstrap evidence

Before the repository received an initial commit, a dependency-free Python reference package builder/validator was validated in an isolated ChatGPT runtime with 13/13 bounded tests. Its temporary staging bundle remains in GoreeCloud/Notes until equivalent authoritative GitHub functionality is merged and verified.

That temporary reference is not the repository source of truth.

## Open engineering work

- Reconcile Cargo.lock from an exact CI-generated dependency graph.
- Bring Rust package-validation tests to parity with the pre-bootstrap reference matrix.
- Begin commands/undo and atomic save/recovery only after this format foundation is accepted.
