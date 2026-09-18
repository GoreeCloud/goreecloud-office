# GoreeCloud Office — Repository Specifications

## Status

Development. This repository contains the first bounded source foundation for the shared Office Engine; it is not a usable Office or Writer release.

## Repository Responsibility

This repository is the primary shared Office repository for the GoreeCloud Office application, shared Office Engine, initial Writer implementation, common editing infrastructure, and shared productivity architecture.

## Current Implemented Source

- Rust workspace targeting Rust 1.85 or later within the declared toolchain contract.
- Canonical Writer-oriented document model with stable document, block, and paragraph-style identities.
- Logical-Unicode paragraph text with validated UTF-8 mutation boundaries.
- Paragraph-style creation and assignment.
- Reversible insert, delete, and paragraph-style commands.
- Bounded undo/redo history with redo invalidation after new commands.
- Dependency-free native package identity mappings for Writer, Spreadsheet, Presentations, Forms, and Database.
- Native package v1 format-family/version identity and canonical common-part names.
- Archive-entry path-safety validation for absolute paths, traversal, separator ambiguity, dot/empty segments, Windows drive prefixes, and NUL characters.
- Library-neutral Writer v1 entry-table validation for duplicate paths, symbolic links, STORE/DEFLATE compression, required package parts, mimetype placement/storage, 4,096-entry ceiling, 64 MiB per-entry ceiling, 512 MiB total uncompressed ceiling, and 1000:1 compression-ratio ceiling.
- Decoded Writer v1 semantic validation for canonical UUIDs, RFC 3339 timestamps, manifest family/version/type/role/required-part/entry-point/producer fields, document identity consistency, and unique paragraph/run object IDs.
- Source validation for formatting, build, unit tests, and Clippy.

## Accepted Architecture

- Rust shared Office Engine.
- Kotlin and Jetpack Compose Android presentation shell.
- TypeScript/browser-native web shell with the portable Rust core compiled to WebAssembly.
- Rust and GTK 4 Linux desktop shell.
- One canonical document model in the shared core.
- Renderer-neutral layout and graphics boundaries.

## Planned Product Scope

The intended suite includes Office home, Writer, Spreadsheet, Presentations, Forms, shared engines, native portable file packages, import/export, printing, templates, accessibility, local-first editing, recovery, synchronization, collaboration, and future database capabilities. Planned scope is not current capability.

## Current Boundaries

The repository does not yet provide persistence, a concrete ZIP adapter, JSON decoding or JSON Schema evaluation, native Office package serialization, integrity verification, atomic save, autosave, recovery journals, text shaping, bidirectional layout, pagination, rendering, printing, PDF export, document import/export, Glaze UI, platform application shells, synchronization, collaboration, or a production runtime.

## Integral Platform Systems

All nine Integral Platform Systems must be evaluated: Manager, Privacy Shield, Wardveil Security, Everkeep, Glaze UI, Mesh, Identity, Policy, and Observability. No runtime conformance is currently claimed. GoreeCloud Sync remains separately governed.

## Security and Privacy

Current source is local library code with no network service, telemetry, account system, file persistence, or document-content logging. Future runtime behavior must preserve privacy-by-default, fail-safe security boundaries, and explicit authorization.

## Validation and Release Boundary

Passing repository CI establishes only source-level Development evidence for the exact revision tested. It does not establish production readiness, Stable qualification, deployment acceptance, security acceptance, or end-user availability.

## Governing Records

Deeper product vision, architecture decisions, native package-format specifications, import/export compatibility requirements, feature roadmap, user manual, and implementation tasks are maintained in the governed GoreeCloud documentation system and must remain synchronized with this repository where their scope overlaps.