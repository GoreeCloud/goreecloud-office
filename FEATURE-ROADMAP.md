# Feature Roadmap

## Current State

The shared Rust Office Engine foundation is under active Development. The initial document/command foundation was merged as `f1d58c94fff485221cff14dc8683848094bba216`, and repository-state documentation was subsequently reconciled on `main`. The active source line now also includes dependency-free native package identity and archive-entry path-safety primitives. No usable Office editor or runtime is implemented.

## Phase 0 — Foundation

- [x] Select Rust shared core architecture.
- [x] Select native Android, web, and Linux shell strategy.
- [x] Define native Office package family and interoperability direction in governed specifications.
- [x] Select initial text/font and graphics/PDF dependency architecture in governed decisions.
- [x] Select AGPL-3.0-or-later for this primary repository.
- [x] Establish repository Rust workspace and CI.
- [x] Implement the first document and command source foundation.
- [x] Implement the dependency-free native package identity and archive-entry path-safety foundation.
- [x] Port the Writer v1 archive-entry-table/resource-limit validation contract behind a library-neutral Rust boundary.
- [x] Port decoded Writer v1 manifest and content semantic validation without coupling the format layer to a JSON implementation.
- [x] Port decoded metadata, relationships, and compatibility v1 semantic validation behind the same dependency-free boundary.
- [x] Port integrity-record structure, coverage, digest-syntax, and byte-length semantics without claiming cryptographic SHA-256 verification.
- [x] Complete the mandatory repository documentation baseline and verify it on authoritative `main`.
- [ ] Establish truthful Platform Contract manifest state after the current nine-system schema is verified.

## Phase 1 — Office Foundation

- [ ] File open/save and atomic replacement.
- [ ] Persistence and native package serialization.
- [ ] Autosave and continuous recovery journal.
- [ ] Crash and unsaved-document recovery.
- [ ] Shared text and layout engine.
- [ ] Shared graphics and style engines.
- [ ] Accessibility foundation.
- [ ] Printing/export foundation.
- [ ] Template foundation.
- [ ] Glaze UI application shell.
- [ ] Nine-system Integral Platform evaluation and applicable integrations.

## Phase 2 — Writer

- [ ] Rich-text editing and styles.
- [ ] Pages, lists, sections, tables, and images.
- [ ] Headers, footers, and page numbering.
- [ ] Comments.
- [ ] Printing and common import/export.
- [ ] Templates and reliable recovery.
- [ ] Accessibility validation.

## Later Phases

Spreadsheet, Presentations, synchronization/collaboration, Forms, and advanced productivity remain later milestones and must reuse the shared foundation rather than creating incompatible editor cores.

## Release Rule

Roadmap completion does not by itself establish Stable or production readiness. Every claimed state requires verified implementation and applicable acceptance evidence.