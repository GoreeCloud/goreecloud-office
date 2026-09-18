# Capabilities

## Overview

This repository currently provides a bounded Development source foundation for the shared GoreeCloud Office Engine. It does not provide a usable Office application.

## Core Capabilities

- Canonical Writer-oriented in-memory document state.
- Stable document, block, and paragraph-style identities.
- Logical-Unicode paragraph text.
- UTF-8 mutation-boundary validation.
- Paragraph-style creation and assignment.
- Reversible insert, delete, and style commands.
- Bounded undo/redo command history.
- Dependency-free native package identity, archive-entry path safety, and Writer v1 package-structure/resource-limit validation.
- Source validation through formatting, build, unit tests, and Clippy.

## User Capabilities

None yet. No user-facing editor or application shell is implemented.

## Administrative Capabilities

None yet beyond repository development and CI workflows.

## Platform Integrations

- GoreeCloud Manager: not implemented.
- Privacy Shield: not implemented.
- Wardveil Security: not implemented.
- Everkeep: not implemented.
- Glaze UI: not implemented.
- GoreeCloud Mesh: not implemented.
- GoreeCloud Identity: not implemented.
- GoreeCloud Policy: not implemented.
- GoreeCloud Observability: not implemented.
- GoreeCloud Sync: separately governed and not implemented.

## Data and Interoperability

No persistence or import/export capability is implemented. The Rust workspace now contains dependency-free native package identity, extension/media-type mapping, version-major identity, archive-entry path safety, and library-neutral Writer v1 entry-table validation for duplicate paths, symlinks, supported compression methods, required parts, mimetype ordering/storage, size ceilings, and compression-ratio limits. ZIP parsing, JSON parsing/schema validation, integrity validation, and package read/write remain unimplemented. The machine-readable schemas, fixture, and reference validator remain governed project evidence outside the repository until their migration is completed.

## Supported Platforms and Interfaces

The shared source foundation is platform-neutral Rust library code. Android, web, and Linux application shells are planned but not implemented.

## Security and Privacy Capabilities

Current crates forbid unsafe Rust and expose no network, telemetry, persistence, or account runtime. Broader security and privacy capabilities remain pending.

## Resilience, Backup, and Recovery Capabilities

None yet. Autosave, atomic save, recovery journals, restoration, and Everkeep integration remain planned.

## Accessibility Capabilities

None yet at runtime. Accessibility is an architectural requirement for later UI, document structure, and authoring assistance.

## Automation and API Capabilities

GitHub Actions validates source formatting, build, tests, and Clippy. No product API exists.

## Current Limitations

No usable editor, file persistence, native package serialization, layout, rendering, printing, import/export, Glaze UI, application shell, collaboration, synchronization, or production deployment is implemented.

## Capability Validation

Capability claims in this file are limited to source that exists in the repository. Passing CI is Development evidence only and must not be treated as production or Stable acceptance.