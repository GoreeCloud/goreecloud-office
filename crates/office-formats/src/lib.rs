//! Native package identity and archive-path safety foundation for the Office Engine.
//!
//! This crate defines dependency-free format identity primitives that can be
//! reused by package readers and writers before ZIP and JSON implementations
//! are selected. It does not parse or write Office packages.

#![forbid(unsafe_code)]

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
