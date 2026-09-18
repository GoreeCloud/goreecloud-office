# GoreeCloud Office

GoreeCloud Office is the first-party productivity suite and shared editing platform for GoreeCloud.

The initial engineering objective is **GoreeCloud Office Engine + GoreeCloud Writer**. The shared engine is intended to provide one canonical document model, command framework, text/layout infrastructure, graphics, styles, file formats, import/export, recovery, accessibility, and later collaboration foundations that can be reused by Writer, Spreadsheet, Presentations, Forms, and future Office applications.

## Current Development State

This repository is in **Development**.

The first source foundation currently contains:

- A Rust workspace.
- A canonical Writer-oriented document-domain crate.
- Stable document, block, and paragraph-style identities.
- Logical-Unicode paragraph text.
- UTF-8 mutation-boundary validation.
- Paragraph-style creation and assignment.
- A reversible command framework.
- Bounded undo/redo history.
- Unit-test source for document and command behavior.
- Pull-request CI for formatting, build, tests, and Clippy.

This does **not** yet implement a usable Office application, Writer UI, persistence, native package serialization, autosave/recovery journals, shaping/layout, rendering, PDF export, import/export, collaboration, Glaze UI, Android, web, or Linux application clients.

## Architecture Direction

The accepted architecture is:

- Rust shared Office Engine.
- Kotlin + Jetpack Compose for Android presentation.
- TypeScript/browser-native web presentation with the Rust core compiled to WebAssembly.
- Rust + GTK 4 for Linux desktop presentation.
- Platform-native user interfaces rather than wrapper-first clients.
- AGPL-3.0-or-later for the primary `goreecloud-office` repository.

Authoritative product, architecture, format, dependency, and implementation-task records remain in the governed GoreeCloud Drive hierarchy until repository-local equivalents are added and synchronized.

## Repository Documentation

- [Specifications](SPECIFICATIONS.md)
- [Current Features](FEATURES.md)
- [Capabilities](CAPABILITIES.md)
- [Feature Roadmap](FEATURE-ROADMAP.md)
- [Benefits](BENEFITS.md)
- [Competitive Objectives](COMPETITIVE-OBJECTIVES.md)
- [Branding](BRANDING.md)
- [Pre-release User Manual](USER-MANUAL.md)
- [Privacy Policy](PRIVACY%20POLICY.md)
- [Security](SECURITY.md)
- [Development Notes](NOTES.md)
## Validation

The repository workflow validates:

```text
cargo fmt --all --check
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Passing source CI is Development evidence only. It does not establish Stable qualification, platform conformance, production acceptance, or a supported release.

## License

GoreeCloud Office is licensed under **AGPL-3.0-or-later**. See [LICENSE](LICENSE).
