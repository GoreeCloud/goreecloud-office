# GoreeCloud Office

GoreeCloud Office is GoreeCloud's first-party productivity suite and shared editing platform.

This repository is the canonical **Office product-family monorepo**. Writer, Spreadsheet, Presentations, Forms, Database, Office Server, Office Formats, Office Templates, and shared Office Engine infrastructure are maintained here as explicit component boundaries.

## Repository layout

- `apps/writer/`
- `apps/spreadsheet/`
- `apps/presentations/`
- `apps/forms/`
- `apps/database/`
- `services/office-server/`
- `packages/formats/`
- `packages/templates/`
- shared Rust Office Engine crates and existing repository-level infrastructure

Each application or service may retain its own build target, runtime, tests, APIs, compatibility identity, and release artifact while sharing one coordinated source repository.

## Current state

The repository is in **Development**. The implemented foundation includes the Rust workspace, Writer-oriented document-domain primitives, reversible commands with bounded undo/redo, native package identity and validation primitives, decoded Writer/common package semantics, governed Office Native Package v1 schemas and fixture test data, unit tests, and Rust CI.

A usable Office application, production collaboration service, full persistence, package serialization, rendering/layout stack, import/export, Glaze UI client implementations, and Stable qualification are not yet established.

## Architecture direction

- Product-family monorepo for tightly coupled Office applications, services, formats, templates, and shared infrastructure.
- Rust shared Office Engine.
- Kotlin + Jetpack Compose for Android.
- TypeScript/browser-native web presentation with the Rust core compiled to WebAssembly.
- Rust + GTK 4 for Linux desktop.
- Platform-native user interfaces rather than wrapper-first clients.
- AGPL-3.0-or-later licensing for the primary Office repository.

The provider migration is complete: this canonical repository is `GoreeCloud/office`. The former standalone Writer, Spreadsheet, Presentations, Forms, Database, Office Server, Office Formats, and Office Templates repositories are archived migration predecessors; active Office-family development belongs in the component paths defined here.

See `docs/decisions/0001-consolidate-office-product-family.md` for the consolidation decision.

## Repository documentation

Repository-level product and development records remain at the root, including `SPECIFICATIONS.md`, `FEATURES.md`, `CAPABILITIES.md`, `FEATURE-ROADMAP.md`, `BENEFITS.md`, `COMPETITIVE-OBJECTIVES.md`, `BRANDING.md`, `USER-MANUAL.md`, `PRIVACY POLICY.md`, `SECURITY.md`, and `NOTES.md`.

## Validation

The current workflow validates the Rust workspace with formatting, build, tests, and Clippy. Passing CI is Development evidence only and does not establish a Stable release.

## License

AGPL-3.0-or-later. See `LICENSE`.
