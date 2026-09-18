//! Native package identity and archive-path safety foundation for the Office Engine.
//!
//! This crate defines dependency-free format identity primitives that can be
//! reused by package readers and writers before ZIP and JSON implementations
//! are selected. It does not parse or write Office packages.

#![forbid(unsafe_code)]

/// Decoded manifest and Writer content semantic validation.
pub mod semantic;

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Canonical format-family value stored by v1 package manifests.
pub const FORMAT_FAMILY: &str = "goreecloud.office";

/// Supported native package major version.
pub const FORMAT_MAJOR_V1: u16 = 1;

/// Canonical package mimetype part.
pub const MIMETYPE_PART: &str = "mimetype";

/// Canonical package manifest part.
pub const MANIFEST_PART: &str = "manifest.json";

/// Canonical portable metadata part.
pub const METADATA_PART: &str = "metadata.json";

/// Canonical relationship graph part.
pub const RELATIONSHIPS_PART: &str = "relationships.json";

/// Canonical compatibility-reporting part.
pub const COMPATIBILITY_PART: &str = "compatibility.json";

/// Canonical integrity-record part.
pub const INTEGRITY_PART: &str = "integrity.json";

/// Native Office document type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DocumentType {
    /// Writer document.
    Writer,
    /// Spreadsheet workbook.
    Spreadsheet,
    /// Presentations deck.
    Presentation,
    /// Forms definition.
    Form,
    /// Future Database document.
    Database,
}

impl DocumentType {
    /// Returns the native file extension without a leading dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Writer => "gcwriter",
            Self::Spreadsheet => "gcsheet",
            Self::Presentation => "gcpresent",
            Self::Form => "gcform",
            Self::Database => "gcdb",
        }
    }

    /// Returns the provisional v1 media type.
    #[must_use]
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Writer => "application/vnd.goreecloud.office.writer+zip",
            Self::Spreadsheet => "application/vnd.goreecloud.office.spreadsheet+zip",
            Self::Presentation => "application/vnd.goreecloud.office.presentation+zip",
            Self::Form => "application/vnd.goreecloud.office.form+zip",
            Self::Database => "application/vnd.goreecloud.office.database+zip",
        }
    }

    /// Returns the manifest document-type value.
    #[must_use]
    pub const fn manifest_value(self) -> &'static str {
        match self {
            Self::Writer => "writer",
            Self::Spreadsheet => "spreadsheet",
            Self::Presentation => "presentation",
            Self::Form => "form",
            Self::Database => "database",
        }
    }

    /// Resolves a native document type from an extension.
    ///
    /// A leading dot is accepted. Matching is intentionally case-sensitive so
    /// canonical package identity remains predictable.
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.strip_prefix('.').unwrap_or(extension) {
            "gcwriter" => Some(Self::Writer),
            "gcsheet" => Some(Self::Spreadsheet),
            "gcpresent" => Some(Self::Presentation),
            "gcform" => Some(Self::Form),
            "gcdb" => Some(Self::Database),
            _ => None,
        }
    }

    /// Resolves a native document type from the provisional media type.
    #[must_use]
    pub fn from_media_type(media_type: &str) -> Option<Self> {
        match media_type {
            "application/vnd.goreecloud.office.writer+zip" => Some(Self::Writer),
            "application/vnd.goreecloud.office.spreadsheet+zip" => Some(Self::Spreadsheet),
            "application/vnd.goreecloud.office.presentation+zip" => Some(Self::Presentation),
            "application/vnd.goreecloud.office.form+zip" => Some(Self::Form),
            "application/vnd.goreecloud.office.database+zip" => Some(Self::Database),
            _ => None,
        }
    }

    /// Resolves a native document type from the manifest value.
    #[must_use]
    pub fn from_manifest_value(value: &str) -> Option<Self> {
        match value {
            "writer" => Some(Self::Writer),
            "spreadsheet" => Some(Self::Spreadsheet),
            "presentation" => Some(Self::Presentation),
            "form" => Some(Self::Form),
            "database" => Some(Self::Database),
            _ => None,
        }
    }
}

/// Native package format version.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FormatVersion {
    major: u16,
    minor: u16,
}

impl FormatVersion {
    /// Creates a format version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns v1.0, the initial package baseline.
    #[must_use]
    pub const fn v1() -> Self {
        Self::new(FORMAT_MAJOR_V1, 0)
    }

    /// Returns the major component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Returns whether the version belongs to the supported v1 major line.
    #[must_use]
    pub const fn has_supported_major(self) -> bool {
        self.major == FORMAT_MAJOR_V1
    }
}

/// Reasons an archive entry path is unsafe or non-canonical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackagePathError {
    /// The entry path is empty.
    Empty,
    /// The entry path is absolute.
    Absolute,
    /// The entry path uses a backslash separator.
    BackslashSeparator,
    /// The entry path contains a current-directory segment.
    CurrentDirectorySegment,
    /// The entry path contains a parent-directory traversal segment.
    ParentTraversal,
    /// The entry path contains an empty path segment.
    EmptySegment,
    /// The entry path starts with a Windows-style drive prefix.
    DrivePrefix,
    /// The entry path contains a NUL character.
    Nul,
}

impl fmt::Display for PackagePathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "package entry path is empty",
            Self::Absolute => "absolute package entry paths are not allowed",
            Self::BackslashSeparator => "package entry paths must use forward slashes",
            Self::CurrentDirectorySegment => {
                "package entry paths must not contain current-directory segments"
            }
            Self::ParentTraversal => {
                "package entry paths must not contain parent-directory traversal"
            }
            Self::EmptySegment => "package entry paths must not contain empty segments",
            Self::DrivePrefix => "Windows-style drive prefixes are not allowed",
            Self::Nul => "package entry paths must not contain NUL characters",
        };
        f.write_str(message)
    }
}

impl Error for PackagePathError {}

/// Validates a ZIP entry path before a package reader accesses or extracts it.
///
/// The v1 format requires readers to reject absolute paths and traversal. This
/// helper also enforces one canonical separator and rejects ambiguous dot,
/// empty, Windows-drive, and NUL-containing paths.
///
/// # Errors
///
/// Returns [`PackagePathError`] when the path is unsafe or non-canonical.
pub fn validate_entry_path(path: &str) -> Result<(), PackagePathError> {
    if path.is_empty() {
        return Err(PackagePathError::Empty);
    }
    if path.contains('\0') {
        return Err(PackagePathError::Nul);
    }
    if path.starts_with('/') {
        return Err(PackagePathError::Absolute);
    }
    if path.contains('\\') {
        return Err(PackagePathError::BackslashSeparator);
    }

    let segments = path.split('/');
    if segments.clone().next().is_some_and(is_windows_drive_prefix) {
        return Err(PackagePathError::DrivePrefix);
    }

    for segment in segments {
        match segment {
            "" => return Err(PackagePathError::EmptySegment),
            "." => return Err(PackagePathError::CurrentDirectorySegment),
            ".." => return Err(PackagePathError::ParentTraversal),
            _ => {}
        }
    }

    Ok(())
}

fn is_windows_drive_prefix(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Canonical Writer content entry point.
pub const WRITER_DOCUMENT_PART: &str = "content/document.json";

/// Required package parts for the validated Writer v1 slice.
pub const WRITER_REQUIRED_PARTS: [&str; 7] = [
    MIMETYPE_PART,
    MANIFEST_PART,
    METADATA_PART,
    RELATIONSHIPS_PART,
    COMPATIBILITY_PART,
    INTEGRITY_PART,
    WRITER_DOCUMENT_PART,
];

/// ZIP compression method exposed through a library-neutral package boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompressionMethod {
    /// ZIP method 0 (STORE).
    Store,
    /// ZIP method 8 (DEFLATE).
    Deflate,
    /// Any method not accepted by the v1 package contract.
    Unsupported(u16),
}

impl CompressionMethod {
    /// Converts a ZIP compression-method code into the package abstraction.
    #[must_use]
    pub const fn from_zip_method(method: u16) -> Self {
        match method {
            0 => Self::Store,
            8 => Self::Deflate,
            other => Self::Unsupported(other),
        }
    }

    /// Returns the corresponding ZIP compression-method code.
    #[must_use]
    pub const fn zip_method(self) -> u16 {
        match self {
            Self::Store => 0,
            Self::Deflate => 8,
            Self::Unsupported(method) => method,
        }
    }
}

/// Structural metadata for one package archive entry.
///
/// Future ZIP adapters can populate this type without leaking a specific ZIP
/// library into the Office format contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageEntry<'a> {
    path: &'a str,
    compression: CompressionMethod,
    uncompressed_bytes: u64,
    compressed_bytes: u64,
    symbolic_link: bool,
}

impl<'a> PackageEntry<'a> {
    /// Creates package-entry metadata for structural validation.
    #[must_use]
    pub const fn new(
        path: &'a str,
        compression: CompressionMethod,
        uncompressed_bytes: u64,
        compressed_bytes: u64,
        symbolic_link: bool,
    ) -> Self {
        Self {
            path,
            compression,
            uncompressed_bytes,
            compressed_bytes,
            symbolic_link,
        }
    }

    /// Returns the package-relative entry path.
    #[must_use]
    pub const fn path(&self) -> &'a str {
        self.path
    }

    /// Returns the declared compression method.
    #[must_use]
    pub const fn compression(&self) -> CompressionMethod {
        self.compression
    }

    /// Returns the uncompressed byte length.
    #[must_use]
    pub const fn uncompressed_bytes(&self) -> u64 {
        self.uncompressed_bytes
    }

    /// Returns the compressed byte length.
    #[must_use]
    pub const fn compressed_bytes(&self) -> u64 {
        self.compressed_bytes
    }

    /// Returns whether the archive entry is a symbolic link.
    #[must_use]
    pub const fn is_symbolic_link(&self) -> bool {
        self.symbolic_link
    }
}

/// Resource limits applied before package content is parsed.
///
/// The defaults mirror the validated pre-bootstrap reference package
/// implementation so the Rust migration begins from the same bounded contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageLimits {
    max_entries: usize,
    entry_uncompressed_ceiling: u64,
    package_uncompressed_ceiling: u64,
    compression_ratio_ceiling: u64,
}

impl PackageLimits {
    /// Creates explicit package resource limits.
    #[must_use]
    pub const fn new(
        max_entries: usize,
        max_entry_uncompressed_bytes: u64,
        max_total_uncompressed_bytes: u64,
        max_compression_ratio: u64,
    ) -> Self {
        Self {
            max_entries,
            entry_uncompressed_ceiling: max_entry_uncompressed_bytes,
            package_uncompressed_ceiling: max_total_uncompressed_bytes,
            compression_ratio_ceiling: max_compression_ratio,
        }
    }

    /// Returns the maximum number of archive entries.
    #[must_use]
    pub const fn max_entries(self) -> usize {
        self.max_entries
    }

    /// Returns the maximum uncompressed size of one entry.
    #[must_use]
    pub const fn max_entry_uncompressed_bytes(self) -> u64 {
        self.entry_uncompressed_ceiling
    }

    /// Returns the maximum total uncompressed package size.
    #[must_use]
    pub const fn max_total_uncompressed_bytes(self) -> u64 {
        self.package_uncompressed_ceiling
    }

    /// Returns the maximum accepted uncompressed-to-compressed size ratio.
    #[must_use]
    pub const fn max_compression_ratio(self) -> u64 {
        self.compression_ratio_ceiling
    }
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self::new(4096, 64 * 1024 * 1024, 512 * 1024 * 1024, 1000)
    }
}

/// Structural package validation failures that can be detected without parsing
/// JSON or reading entry payload bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageStructureError {
    /// The archive has more entries than the configured limit.
    TooManyEntries {
        /// Actual entry count.
        actual: usize,
        /// Configured maximum entry count.
        maximum: usize,
    },
    /// Two archive entries use the same package path.
    DuplicatePath {
        /// Duplicated package path.
        path: String,
    },
    /// An archive path fails canonical path validation.
    UnsafePath {
        /// Rejected package path.
        path: String,
        /// Underlying path-validation reason.
        reason: PackagePathError,
    },
    /// Symbolic-link archive entries are prohibited.
    SymbolicLink {
        /// Rejected package path.
        path: String,
    },
    /// A ZIP compression method is not supported by the v1 contract.
    UnsupportedCompression {
        /// Rejected package path.
        path: String,
        /// ZIP compression-method code.
        method: u16,
    },
    /// One entry exceeds the configured uncompressed-size limit.
    EntryTooLarge {
        /// Rejected package path.
        path: String,
        /// Actual uncompressed byte size.
        actual: u64,
        /// Configured maximum uncompressed byte size.
        maximum: u64,
    },
    /// The package exceeds the configured total uncompressed-size limit.
    PackageTooLarge {
        /// Actual accumulated uncompressed byte size at rejection.
        actual: u64,
        /// Configured maximum total uncompressed byte size.
        maximum: u64,
    },
    /// One entry exceeds the configured compression-ratio limit.
    CompressionRatioTooHigh {
        /// Rejected package path.
        path: String,
        /// Entry uncompressed byte size.
        uncompressed_bytes: u64,
        /// Entry compressed byte size.
        compressed_bytes: u64,
        /// Configured maximum ratio.
        maximum_ratio: u64,
    },
    /// A required Writer package part is absent.
    MissingRequiredPart {
        /// Missing package path.
        path: &'static str,
    },
    /// Mimetype is not the first archive entry.
    MimetypeNotFirst,
    /// Mimetype is compressed instead of stored.
    MimetypeMustBeStored,
}

impl fmt::Display for PackageStructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyEntries { actual, maximum } => {
                write!(f, "package has {actual} entries; maximum is {maximum}")
            }
            Self::DuplicatePath { path } => {
                write!(f, "package contains duplicate entry path: {path}")
            }
            Self::UnsafePath { path, reason } => {
                write!(f, "unsafe package path {path:?}: {reason}")
            }
            Self::SymbolicLink { path } => {
                write!(f, "symbolic-link entries are not permitted: {path}")
            }
            Self::UnsupportedCompression { path, method } => {
                write!(f, "unsupported ZIP compression method {method} for {path}")
            }
            Self::EntryTooLarge {
                path,
                actual,
                maximum,
            } => write!(
                f,
                "entry {path} has {actual} uncompressed bytes; maximum is {maximum}"
            ),
            Self::PackageTooLarge { actual, maximum } => write!(
                f,
                "package has {actual} uncompressed bytes; maximum is {maximum}"
            ),
            Self::CompressionRatioTooHigh {
                path,
                uncompressed_bytes,
                compressed_bytes,
                maximum_ratio,
            } => write!(
                f,
                "entry {path} compression ratio {uncompressed_bytes}:{compressed_bytes} exceeds {maximum_ratio}:1"
            ),
            Self::MissingRequiredPart { path } => {
                write!(f, "required Writer package part is missing: {path}")
            }
            Self::MimetypeNotFirst => f.write_str("mimetype must be the first ZIP entry"),
            Self::MimetypeMustBeStored => {
                f.write_str("mimetype must be stored without compression")
            }
        }
    }
}

impl Error for PackageStructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsafePath { reason, .. } => Some(reason),
            _ => None,
        }
    }
}

/// Validates Writer v1 archive structure using only ZIP entry metadata.
///
/// This is intentionally independent of any ZIP library. A future adapter must
/// inspect the archive, construct [`PackageEntry`] values, and call this function
/// before reading or extracting entry payloads.
///
/// # Errors
///
/// Returns [`PackageStructureError`] when the entry table violates the Writer v1
/// package contract or configured resource limits.
pub fn validate_writer_entry_table(
    entries: &[PackageEntry<'_>],
    limits: PackageLimits,
) -> Result<(), PackageStructureError> {
    if entries.len() > limits.max_entries {
        return Err(PackageStructureError::TooManyEntries {
            actual: entries.len(),
            maximum: limits.max_entries,
        });
    }

    let mut seen = BTreeSet::new();
    let mut total_uncompressed = 0_u64;

    for entry in entries {
        validate_entry_path(entry.path).map_err(|reason| PackageStructureError::UnsafePath {
            path: entry.path.to_owned(),
            reason,
        })?;

        if !seen.insert(entry.path) {
            return Err(PackageStructureError::DuplicatePath {
                path: entry.path.to_owned(),
            });
        }

        if entry.symbolic_link {
            return Err(PackageStructureError::SymbolicLink {
                path: entry.path.to_owned(),
            });
        }

        if let CompressionMethod::Unsupported(method) = entry.compression {
            return Err(PackageStructureError::UnsupportedCompression {
                path: entry.path.to_owned(),
                method,
            });
        }

        if entry.uncompressed_bytes > limits.entry_uncompressed_ceiling {
            return Err(PackageStructureError::EntryTooLarge {
                path: entry.path.to_owned(),
                actual: entry.uncompressed_bytes,
                maximum: limits.entry_uncompressed_ceiling,
            });
        }

        total_uncompressed = total_uncompressed.saturating_add(entry.uncompressed_bytes);
        if total_uncompressed > limits.package_uncompressed_ceiling {
            return Err(PackageStructureError::PackageTooLarge {
                actual: total_uncompressed,
                maximum: limits.package_uncompressed_ceiling,
            });
        }

        if entry.compressed_bytes > 0
            && entry.uncompressed_bytes
                > entry
                    .compressed_bytes
                    .saturating_mul(limits.compression_ratio_ceiling)
        {
            return Err(PackageStructureError::CompressionRatioTooHigh {
                path: entry.path.to_owned(),
                uncompressed_bytes: entry.uncompressed_bytes,
                compressed_bytes: entry.compressed_bytes,
                maximum_ratio: limits.compression_ratio_ceiling,
            });
        }
    }

    for required in WRITER_REQUIRED_PARTS {
        if !seen.iter().any(|path| *path == required) {
            return Err(PackageStructureError::MissingRequiredPart { path: required });
        }
    }

    if entries.first().map(PackageEntry::path) != Some(MIMETYPE_PART) {
        return Err(PackageStructureError::MimetypeNotFirst);
    }

    if entries[0].compression != CompressionMethod::Store {
        return Err(PackageStructureError::MimetypeMustBeStored);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_TYPES: [DocumentType; 5] = [
        DocumentType::Writer,
        DocumentType::Spreadsheet,
        DocumentType::Presentation,
        DocumentType::Form,
        DocumentType::Database,
    ];

    fn valid_writer_entries() -> Vec<PackageEntry<'static>> {
        vec![
            PackageEntry::new(MIMETYPE_PART, CompressionMethod::Store, 44, 44, false),
            PackageEntry::new(MANIFEST_PART, CompressionMethod::Deflate, 551, 300, false),
            PackageEntry::new(METADATA_PART, CompressionMethod::Deflate, 94, 75, false),
            PackageEntry::new(
                RELATIONSHIPS_PART,
                CompressionMethod::Deflate,
                40,
                35,
                false,
            ),
            PackageEntry::new(
                COMPATIBILITY_PART,
                CompressionMethod::Deflate,
                103,
                80,
                false,
            ),
            PackageEntry::new(INTEGRITY_PART, CompressionMethod::Deflate, 778, 400, false),
            PackageEntry::new(
                WRITER_DOCUMENT_PART,
                CompressionMethod::Deflate,
                290,
                180,
                false,
            ),
        ]
    }

    #[test]
    fn compression_method_codes_round_trip() {
        assert_eq!(
            CompressionMethod::from_zip_method(0),
            CompressionMethod::Store
        );
        assert_eq!(
            CompressionMethod::from_zip_method(8),
            CompressionMethod::Deflate
        );
        assert_eq!(
            CompressionMethod::from_zip_method(93),
            CompressionMethod::Unsupported(93)
        );
        assert_eq!(CompressionMethod::Store.zip_method(), 0);
        assert_eq!(CompressionMethod::Deflate.zip_method(), 8);
        assert_eq!(CompressionMethod::Unsupported(93).zip_method(), 93);
    }

    #[test]
    fn writer_reference_entry_shape_is_accepted() {
        assert_eq!(
            validate_writer_entry_table(&valid_writer_entries(), PackageLimits::default()),
            Ok(())
        );
    }

    #[test]
    fn writer_entry_table_rejects_duplicates_and_symlinks() {
        let mut duplicate = valid_writer_entries();
        duplicate.push(PackageEntry::new(
            MANIFEST_PART,
            CompressionMethod::Deflate,
            2,
            2,
            false,
        ));
        assert_eq!(
            validate_writer_entry_table(&duplicate, PackageLimits::default()),
            Err(PackageStructureError::DuplicatePath {
                path: MANIFEST_PART.to_owned()
            })
        );

        let mut symlink = valid_writer_entries();
        symlink[2] = PackageEntry::new(METADATA_PART, CompressionMethod::Store, 0, 0, true);
        assert_eq!(
            validate_writer_entry_table(&symlink, PackageLimits::default()),
            Err(PackageStructureError::SymbolicLink {
                path: METADATA_PART.to_owned()
            })
        );
    }

    #[test]
    fn writer_entry_table_rejects_unsupported_compression() {
        let mut entries = valid_writer_entries();
        entries[2] = PackageEntry::new(
            METADATA_PART,
            CompressionMethod::Unsupported(12),
            94,
            75,
            false,
        );

        assert_eq!(
            validate_writer_entry_table(&entries, PackageLimits::default()),
            Err(PackageStructureError::UnsupportedCompression {
                path: METADATA_PART.to_owned(),
                method: 12
            })
        );
    }

    #[test]
    fn writer_entry_table_enforces_size_limits() {
        let mut entry_too_large = valid_writer_entries();
        entry_too_large[2] =
            PackageEntry::new(METADATA_PART, CompressionMethod::Store, 601, 601, false);
        let limits = PackageLimits::new(32, 600, 10_000, 1000);
        assert_eq!(
            validate_writer_entry_table(&entry_too_large, limits),
            Err(PackageStructureError::EntryTooLarge {
                path: METADATA_PART.to_owned(),
                actual: 601,
                maximum: 600
            })
        );

        let limits = PackageLimits::new(32, 10_000, 1000, 1000);
        assert!(matches!(
            validate_writer_entry_table(&valid_writer_entries(), limits),
            Err(PackageStructureError::PackageTooLarge { .. })
        ));
    }

    #[test]
    fn writer_entry_table_enforces_compression_ratio() {
        let mut entries = valid_writer_entries();
        entries[2] = PackageEntry::new(METADATA_PART, CompressionMethod::Deflate, 1001, 1, false);

        assert_eq!(
            validate_writer_entry_table(&entries, PackageLimits::default()),
            Err(PackageStructureError::CompressionRatioTooHigh {
                path: METADATA_PART.to_owned(),
                uncompressed_bytes: 1001,
                compressed_bytes: 1,
                maximum_ratio: 1000
            })
        );
    }

    #[test]
    fn writer_entry_table_requires_all_parts_and_mimetype_rules() {
        let mut missing = valid_writer_entries();
        missing.retain(|entry| entry.path() != METADATA_PART);
        assert_eq!(
            validate_writer_entry_table(&missing, PackageLimits::default()),
            Err(PackageStructureError::MissingRequiredPart {
                path: METADATA_PART
            })
        );

        let mut not_first = valid_writer_entries();
        not_first.swap(0, 1);
        assert_eq!(
            validate_writer_entry_table(&not_first, PackageLimits::default()),
            Err(PackageStructureError::MimetypeNotFirst)
        );

        let mut compressed_mimetype = valid_writer_entries();
        compressed_mimetype[0] =
            PackageEntry::new(MIMETYPE_PART, CompressionMethod::Deflate, 44, 40, false);
        assert_eq!(
            validate_writer_entry_table(&compressed_mimetype, PackageLimits::default()),
            Err(PackageStructureError::MimetypeMustBeStored)
        );
    }

    #[test]
    fn document_type_identity_round_trips() {
        for document_type in ALL_TYPES {
            assert_eq!(
                DocumentType::from_extension(document_type.extension()),
                Some(document_type)
            );
            assert_eq!(
                DocumentType::from_extension(&format!(".{}", document_type.extension())),
                Some(document_type)
            );
            assert_eq!(
                DocumentType::from_media_type(document_type.media_type()),
                Some(document_type)
            );
            assert_eq!(
                DocumentType::from_manifest_value(document_type.manifest_value()),
                Some(document_type)
            );
        }
    }

    #[test]
    fn unknown_identity_values_are_rejected() {
        assert_eq!(DocumentType::from_extension("zip"), None);
        assert_eq!(DocumentType::from_extension("GCWRITER"), None);
        assert_eq!(DocumentType::from_media_type("application/zip"), None);
        assert_eq!(DocumentType::from_manifest_value("notes"), None);
    }

    #[test]
    fn v1_major_is_supported() {
        assert!(FormatVersion::v1().has_supported_major());
        assert!(FormatVersion::new(1, 99).has_supported_major());
        assert!(!FormatVersion::new(2, 0).has_supported_major());
    }

    #[test]
    fn canonical_package_paths_are_accepted() {
        for path in [
            MIMETYPE_PART,
            MANIFEST_PART,
            METADATA_PART,
            RELATIONSHIPS_PART,
            COMPATIBILITY_PART,
            INTEGRITY_PART,
            "content/document.json",
            "assets/images/example.png",
            "extensions/vendor/data.json",
        ] {
            assert_eq!(validate_entry_path(path), Ok(()), "{path}");
        }
    }

    #[test]
    fn absolute_and_traversal_paths_are_rejected() {
        assert_eq!(
            validate_entry_path("/content/document.json"),
            Err(PackagePathError::Absolute)
        );
        assert_eq!(
            validate_entry_path("../metadata.json"),
            Err(PackagePathError::ParentTraversal)
        );
        assert_eq!(
            validate_entry_path("content/../../metadata.json"),
            Err(PackagePathError::ParentTraversal)
        );
    }

    #[test]
    fn ambiguous_platform_paths_are_rejected() {
        assert_eq!(
            validate_entry_path(r"content\document.json"),
            Err(PackagePathError::BackslashSeparator)
        );
        assert_eq!(
            validate_entry_path("C:/content/document.json"),
            Err(PackagePathError::DrivePrefix)
        );
        assert_eq!(
            validate_entry_path("./manifest.json"),
            Err(PackagePathError::CurrentDirectorySegment)
        );
        assert_eq!(
            validate_entry_path("content//document.json"),
            Err(PackagePathError::EmptySegment)
        );
    }

    #[test]
    fn empty_and_nul_paths_are_rejected() {
        assert_eq!(validate_entry_path(""), Err(PackagePathError::Empty));
        assert_eq!(
            validate_entry_path("content/\0document.json"),
            Err(PackagePathError::Nul)
        );
    }
}
