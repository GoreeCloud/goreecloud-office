# GoreeCloud Office

GoreeCloud Office is the shared foundation for the GoreeCloud Office Suite. The repository owns the shared Office Engine, canonical working-document model, commands, text/layout infrastructure, graphics and chart foundations, native Office file formats, import/export, printing, templates, accessibility semantics, collaboration foundations, and the Office start-center application.

## Current development state

This repository is in **Experimental / Development** state.

The first bounded source slice implements:

- A Rust workspace for the shared engine foundation.
- An initial Writer semantic document model containing paragraphs and text runs.
- Canonical lowercase UUID validation and duplicate-object-ID rejection.
- A first `.gcwriter` ZIP package builder and validator.
- Required package-part, mimetype, path-safety, encryption, compression, size, document-ID, and SHA-256 integrity checks.
- Machine-readable Office Native Package v1 JSON schemas.
- Automated source validation through GitHub Actions.

This slice does **not** provide a usable Writer editor, Office UI, import/export with ODF or OOXML, rendering, printing, collaboration, synchronization, or production-ready Office capability.

## Architecture

The accepted direction is a shared Rust Office Engine with platform-native presentation shells:

- Android: Kotlin + Jetpack Compose.
- Web: TypeScript/browser-native shell with the Rust core compiled to WebAssembly.
- Linux: Rust + GTK 4.

The shared engine owns canonical document semantics. Platform clients must not create competing authoritative document models.

## Native formats

Planned native file extensions are:

- Writer: `.gcwriter`
- Spreadsheet: `.gcsheet`
- Presentations: `.gcpresent`
- Forms: `.gcform`
- Database: `.gcdb`

Native formats are intended to preserve GoreeCloud-specific capabilities without locking user information into an opaque application database.

## Product boundaries

- Office owns creation and editing.
- GoreeCloud Drive owns optional file storage and remote availability.
- GoreeCloud Documents owns managed records, OCR, classification, archival organization, and document workflows.
- GoreeCloud Sync owns separately governed synchronization.
- Everkeep owns continuity and recoverability.

## License

GoreeCloud Office is licensed under **AGPL-3.0-or-later**. See [LICENSE](LICENSE).

Third-party dependencies remain subject to their own licenses.

## Status integrity

Repository source, tests, documentation, or CI do not establish Stable or production acceptance by themselves. Current capability claims are limited to exact verified source and validation evidence.
