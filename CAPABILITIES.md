# Capabilities

## Overview

This repository currently provides an **Experimental source foundation**, not a usable Office application.

## Core Capabilities

- Represent a minimal Writer document using a versioned Rust semantic model.
- Validate canonical document object identifiers and reject duplicates.
- Build a minimal v1 `.gcwriter` package from an in-memory Writer document and package metadata.
- Validate bounded v1 Writer packages and reject unsupported or unsafe package states.
- Verify SHA-256 integrity records for protected package parts.

## Data and Interoperability

The current source includes the GoreeCloud Office Native Package v1 schema set. ODF, OOXML, Markdown, HTML, PDF, CSV, JSON interchange, and other external-format conversions remain planned and are not implemented by this source slice.

## Supported Platforms and Interfaces

The Rust crates are intended to become portable shared-engine components. Platform client builds are not yet implemented or accepted.

## Platform Integrations

### GoreeCloud Manager
Applicable, blocked. No accepted Office-specific integration exists.

### Privacy Shield
Applicable, blocked. No accepted Office-specific integration exists.

### Wardveil Security
Applicable, blocked. No accepted Office-specific integration exists.

### Everkeep
Applicable, blocked. No accepted Office-specific integration exists.

### Glaze UI
Applicable, blocked. No Office graphical client exists in this source slice.

### GoreeCloud Mesh
Applicable, blocked. No accepted Office-specific integration exists.

### GoreeCloud Identity
Applicable, blocked for networked/collaborative identity scope. Core local file parsing does not require an account.

### GoreeCloud Policy
Applicable, blocked. No accepted Office-specific policy integration exists.

### GoreeCloud Observability
Applicable, blocked. CI/test output is not platform Observability acceptance.

## Security and Privacy Capabilities

Package validation currently rejects unsafe relative-path states, encrypted entries, unsupported ZIP entry types, unsupported compression, excessive package sizes, inconsistent identifiers, and failed integrity checks.

These controls are source-level parser hardening only. They do not constitute Wardveil Security or Privacy Shield acceptance.

## Resilience, Backup, and Recovery Capabilities

No application recovery capability is implemented yet.

## Accessibility Capabilities

No user-facing accessibility implementation exists yet.

## Automation and API Capabilities

No Office network API exists.

## Current Limitations

There is no editor, renderer, UI, printing path, external-format converter, collaboration service, Sync integration, or release artifact.

## Capability Validation

Current capability acceptance is limited to exact repository source and passing tests/CI for the evaluated revision. No Stable or production claim is authorized.
