# GoreeCloud Office Specifications

## Authority

This repository copy records source-coupled requirements for the current GoreeCloud Office development line. The broader canonical product specification remains in GoreeCloud Drive at `GoreeCloud/Projects/Project Specification — Office Suite.md`.

## Current source scope

The current foundation is intentionally narrow:

1. Establish a Rust workspace for shared Office logic.
2. Establish an initial Writer semantic document model.
3. Establish the first native Writer package builder/validator.
4. Carry the accepted Office Native Package v1 schemas beside their implementation.
5. Validate source through pinned CI.

## Document model requirements

The first Writer model contains:

- A schema version.
- One canonical document ID.
- Paragraph blocks.
- Text runs.
- Optional style references.

Every document/block/run ID must be a canonical lowercase hyphenated UUID. IDs must be unique within the document object graph.

Canonical text remains logical Unicode text. Shaping, bidi visual reordering, rendering, selection geometry, pagination, and rich style behavior are later engine layers.

## Package requirements

A v1 `.gcwriter` file is a ZIP-compatible package.

The initial implementation requires:

- `mimetype` as the first ZIP entry.
- `mimetype` stored without compression.
- Exact Writer media type: `application/vnd.goreecloud.office.writer+zip`.
- `manifest.json`.
- `metadata.json`.
- `relationships.json`.
- `compatibility.json`.
- `content/document.json`.
- `integrity.json`.
- Stored or Deflated ZIP compression only.
- No encrypted entries.
- No directory or symlink entries.
- Safe relative forward-slash package paths.
- Bounded entry count and uncompressed size.
- SHA-256 integrity coverage for all ordinary package parts except `mimetype` and `integrity.json`.
- Matching manifest/document identifiers.
- Fail-closed rejection of unsupported major versions.

## Security boundary

Office package input is untrusted structured input. Successful parsing does not establish that a document is safe, authorized, privacy-permitted, or suitable for production use.

## Not yet implemented

The current source does not implement the Office UI, rich text editing, layout, shaping, graphics, PDF/print, ODF/OOXML conversion, autosave/recovery, platform clients, Sync, collaboration, or accepted Integral Platform System integrations.
