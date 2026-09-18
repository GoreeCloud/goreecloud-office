# Features

## Current Development-source functionality

The current branch implements only the bounded shared-engine/file-format foundation:

- Rust workspace with `office-document` and `office-formats` crates.
- Initial Writer paragraph/text-run document model.
- Canonical UUID validation.
- Duplicate object-ID rejection.
- Writer schema-major rejection.
- `.gcwriter` package construction.
- `.gcwriter` package validation.
- Required-part validation.
- ZIP path, entry-type, encryption, compression, entry-count, size, and compression-ratio controls.
- Manifest/document ID consistency validation.
- SHA-256 package-part integrity validation.
- Unit tests for document and package foundations.
- Repository-local machine-readable JSON schemas.

## Planned features

See `FEATURE-ROADMAP.md` and the canonical GoreeCloud Office Suite project specification.

Planned features must not be interpreted as current functionality.
