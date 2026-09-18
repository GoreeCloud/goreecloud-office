# Features

## Current Development Features

- Canonical Writer-oriented document model.
- Stable document, block, and paragraph-style identities.
- Logical-Unicode paragraph text.
- UTF-8 mutation-boundary validation.
- Paragraph-style creation and assignment.
- Reversible insert-text commands.
- Reversible delete-text commands.
- Reversible paragraph-style commands.
- Bounded undo/redo history.
- Redo invalidation after a new successful command.
- Preservation of command history after failed mutations.
- Native package extension, provisional media-type, and manifest document-type mappings.
- Native package v1 major-version identity.
- Archive-entry path validation rejecting absolute paths, traversal, backslash separators, dot/empty segments, Windows drive prefixes, and NUL characters.
- Writer v1 archive-entry-table validation for duplicate paths, symlinks, STORE/DEFLATE-only compression, required parts, mimetype ordering/storage, per-entry and total uncompressed-size limits, and compression-ratio limits.
- Canonical lowercase UUID and RFC 3339 timestamp validation for decoded v1 records.
- Decoded Writer manifest semantic validation for family/version/type, document identity, required parts, entry point, timestamps, and producer identity.
- Decoded Writer document semantic validation for schema major version, manifest/document identity consistency, and unique paragraph/run object IDs.
- Repository CI for formatting, build, unit tests, and Clippy.

These are Development source capabilities only. They do not constitute a usable Office application or Writer editor.

## Planned Features

Planned work includes persistence, native Office packages, atomic save, autosave and recovery, text shaping/layout, graphics, styles, printing/export, templates, accessibility, Glaze UI shells, Writer editing, Spreadsheet, Presentations, Forms, synchronization, collaboration, and later database capabilities.

## Removed or Deprecated Features

None.