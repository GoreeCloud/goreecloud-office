# Historical Drive Feature Roadmap Migration Source — GoreeCloud Office Suite

> **Status:** Historical, non-authoritative migration evidence.  
> **Source:** Former Google Drive roadmap, captured during repository migration on 2026-09-27.  
> **Rule:** Do not synchronize this file with Google Drive. Current feature truth is in `IMPLEMENTED-FEATURES.md`, `PLANNED-FEATURES.md`, and `CHANGELOGS.md`.

---
title: "GoreeCloud Office Suite — Feature Roadmap"
document_type: "Product Feature Roadmap"
status: "Active Development"
version: "v0.14"
classification: "Internal"
created: "2026-09-17"
last_updated: "2026-09-18"
product: "GoreeCloud Office Suite"
implementation_status: "Initial Rust document/command and native-package validation foundations are merged and validated on main; byte-matched copies of the six governed v1 JSON Schemas and canonical minimal .gcwriter fixture are present as repository test data; JSON decoding/JSON Schema evaluation, cryptographic digest verification, usable Office editor/runtime, concrete ZIP/JSON package reader/writer, and production release remain unverified"
primary_repository: "GoreeCloud/goreecloud-office"
---

# GoreeCloud Office Suite — Feature Roadmap

## Current State

`GoreeCloud/goreecloud-office` is the active primary repository for the shared-engine-first Office implementation. PR #1 was squash-merged to authoritative `main` as `f1d58c94fff485221cff14dc8683848094bba216`.

The current verified Rust source implements a bounded Writer-oriented document/command foundation plus dependency-free native-package identity/path-safety and Writer v1 entry-table/resource-limit validation: stable document/block/style identities, logical-Unicode paragraph text, UTF-8 mutation-boundary validation, reversible insert/delete/style commands, bounded undo/redo history, v1 package identity/version constants, extension/media-type/manifest mappings, canonical package-part names, unsafe archive-path rejection, duplicate-path and symlink rejection, STORE/DEFLATE-only entry metadata, required-part and `mimetype` rules, and bounded entry-count/size/compression-ratio checks. PR #5 merged the identity/path-safety slice as `00266d0a2d3a2e640024e776534612b3253f8aaf`; PR #6 merged the structure/resource validation slice as `890e2c33f9246f29e27ce9ddb9f3ede0463c2086`; PR #7 merged decoded Writer manifest/content semantic validation as `3fb4c821ea622b204bbea98d490271371744ab8c`; PR #8 merged decoded metadata/relationships/compatibility record semantics as `c20180b891fbb94bea39ae46e32b34ece0c97afe`; PR #9 merged integrity-record semantics as `77797500a6d900e01e42cea42d986970a41fa033`. PR #10 then imported byte-matched copies of the six governed JSON Schemas and canonical minimal `.gcwriter` fixture as repository test data, merging as `3b37f490144b2fa30626017908a8a06283962253`. PR #10 exact head passed Rust Foundation run `35313505025`, and push-triggered run `35313543218` passed on the authoritative merge commit. The workspace now contains 44 tests.

The mandatory repository documentation baseline and the accepted `AGPL-3.0-or-later` repository recording are present on `main`. There is still no usable Office/Writer application, persistence, native-package Rust serialization, layout/rendering, platform UI, collaboration runtime, deployment, production acceptance, or Stable release.

Seven reserved Office-family repositories remain empty. The canonical presentation repository is `goreecloud-presentations` (plural); it currently contains only a placeholder README and is not an active separated implementation.

The bounded native-file-format foundation remains verified in Drive: six machine-readable v1 schemas, a minimal valid `.gcwriter` fixture, and a reference validator with 13/13 bounded tests passing. Migration of that package model into the Rust repository is now actionable.

## Phase 0 — Foundation Decisions

- [x] Select implementation language/runtime: Rust shared core.
- [x] Select cross-platform client architecture: Kotlin/Jetpack Compose Android, TypeScript/browser-native web with Rust/WASM core, and Rust/GTK 4 Linux.
- [x] Define shared engine and platform-shell authority boundaries.
- [x] Define the authoritative document-model and serialization boundary at the architectural level.
- [x] Define native package-format naming and extension strategy: `.gcwriter`, `.gcsheet`, `.gcpresent`, `.gcform`, and `.gcdb`.
- [x] Define supporting-component dependency-selection criteria.
- [x] Record the Office Engine implementation architecture decision.
- [x] Define the planned Office Native Package Format v1 contract in `GoreeCloud/Data, Schemas, and APIs`.
- [x] Define the planned Office Import and Export Compatibility Matrix v1, including ODF 1.4, OOXML, JSON/CSV, Markdown/text/HTML, and PDF publication targets.
- [x] Select the initial text shaping, Unicode, bidi, OpenType parsing, and font-data foundation through the Office Text, Unicode, and Font ADR; exact dependency versions remain to be pinned when those dependencies are introduced.
- [x] Select the graphics/raster dependency architecture: renderer-neutral Office scene plus Vello CPU software/reference backend; Vello GPU remains an optional future acceleration candidate pending validation.
- [x] Select Krilla as the preferred PDF/print export backend, conditionally accepted pending Office accessibility, complex-script, PDF/A, and PDF/UA validation.
- [x] Select usvg/resvg for static SVG and `image` with an allowlisted codec set for raster image handling.
- [ ] Select or validate the final color-management implementation if the chosen graphics/PDF dependencies do not fully satisfy Office requirements.
- [x] Select `AGPL-3.0-or-later` for the primary `goreecloud-office` repository under the Office licensing decision.
- [x] Apply and verify the accepted `AGPL-3.0-or-later` repository license recording through the root `LICENSE` rights notice, Cargo metadata, and README statement.
- [x] Implement the first machine-readable Office package schema slice and minimal `.gcwriter` fixture in `GoreeCloud/Data, Schemas, and APIs`.
- [x] Implement and validate a temporary dependency-free reference `.gcwriter` builder/validator; 13/13 bounded tests pass.
- [x] Implement and validate the dependency-free Rust native-package identity/version, extension/media-type mapping, canonical-part naming, and archive-entry path-safety foundation.
- [x] Port and validate the Writer v1 archive-entry-table/resource-limit contract behind a library-neutral Rust boundary.
- [x] Port decoded Writer v1 manifest/content semantic validation without coupling the format layer to a JSON implementation.
- [x] Port decoded metadata, relationships, and compatibility v1 semantic validation behind the dependency-free format boundary.
- [x] Port integrity-record structure, exact-coverage, digest-syntax, and byte-length semantics without claiming cryptographic verification.
- [x] Establish and verify the Rust repository baseline, mandatory root documentation, and CI on authoritative `main`.
- [ ] Verify default-branch protection; the current GitHub integration cannot read the classic branch-protection endpoint and no repository rulesets are returned.
- [ ] Establish truthful Platform Contract state after verifying current nine-system manifest schema support.

## Phase 1 — Office Foundation

- [x] Initial canonical document-domain and reversible command/undo-redo source slice.
- [ ] Shared Office Engine beyond the initial document/command slice.
- File open/save.
- Atomic file replacement.
- Undo/redo.
- Autosave.
- Continuous recovery journal.
- Crash and unsaved-document recovery.
- Shared text engine.
- Shared graphics engine.
- Shared style engine.
- Shared command framework.
- Accessibility foundation.
- Printing/export foundation.
- Template foundation.
- Glaze UI application shell.
- Nine-system Integral Platform evaluation.
- GoreeCloud Policy and Observability boundaries.
- Separately governed GoreeCloud Sync boundary.

## Phase 2 — Writer

- Rich-text editing.
- Paragraph and character styles.
- Headings, lists, sections, and pages.
- Tables.
- Images and drawing objects.
- Headers, footers, and page numbering.
- Comments.
- Printing.
- Common import/export.
- Templates.
- Reliable autosave/recovery.
- Accessibility validation.

## Phase 3 — Spreadsheet

- Grid engine.
- Formula engine.
- Dependency graph.
- Formatting.
- Filters.
- Data validation.
- Charts.
- Analysis tools.
- Large-workbook performance.

## Phase 4 — Presentations

- Slide engine.
- Layout/master system.
- Shared drawing tools.
- Shared charts.
- Speaker mode.
- Transitions.
- Animation.
- Presentation export.

## Phase 5 — Synchronization and Collaboration

- GoreeCloud Identity integration.
- GoreeCloud Sync integration.
- Shared workspaces.
- Real-time coauthoring.
- Presence.
- Comments.
- Suggestions.
- Version history.
- Permissions.
- Offline reconciliation.

## Phase 6 — Forms

Build GoreeCloud Forms using the existing shared document, style, permission, collaboration, and data infrastructure.

## Phase 7 — Advanced Productivity

- Database application.
- Advanced publishing.
- Advanced data analysis.
- Automation.
- Document intelligence.
- Expanded cross-application workflows.
- Organization administration.

## Repository Separation Rule

The initial implementation remains in `GoreeCloud/goreecloud-office`.

The existing reserved Office-family repositories must not be populated merely because they exist. Separation requires a verified need such as scale, release independence, security boundaries, ownership boundaries, or another approved architectural reason.

## Release Integrity Rule

Planned work must not be presented as implemented. Stable qualification requires verified implementation, accepted validation evidence, and the current nine-system Integral Platform evaluation.
