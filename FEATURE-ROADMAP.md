# Feature Roadmap

## Current State

The shared Rust Office Engine foundation is under active Development. Authoritative `main` at `f1d58c94fff485221cff14dc8683848094bba216` contains canonical document state, stable identities, paragraph styles, reversible text/style commands, bounded undo/redo history, the mandatory repository documentation baseline, and Rust CI. The post-merge push validation passed. No usable Office editor or runtime is implemented.

## Phase 0 — Foundation

- [x] Select Rust shared core architecture.
- [x] Select native Android, web, and Linux shell strategy.
- [x] Define native Office package family and interoperability direction in governed specifications.
- [x] Select initial text/font and graphics/PDF dependency architecture in governed decisions.
- [x] Select AGPL-3.0-or-later for this primary repository.
- [x] Establish repository Rust workspace and CI.
- [x] Implement the first document and command source foundation.
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