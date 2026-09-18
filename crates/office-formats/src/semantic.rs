//! Dependency-free semantic validation for decoded Office v1 records.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crate::{
    DocumentType, FORMAT_FAMILY, FORMAT_MAJOR_V1, FormatVersion, WRITER_DOCUMENT_PART,
    validate_entry_path,
};

/// Role declared by an Office package manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentRole {
    /// Ordinary editable document.
    Document,
    /// Reusable template.
    Template,
}

/// Returns whether a value is a canonical lowercase UUID accepted by the v1 schemas.
#[must_use]
pub fn is_canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for index in [8_usize, 13, 18, 23] {
        if bytes[index] != b'-' {
            return false;
        }
    }
    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            continue;
        }
        if !matches!(byte, b'0'..=b'9' | b'a'..=b'f') {
            return false;
        }
    }
    matches!(bytes[14], b'1'..=b'5') && matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
}

/// Returns whether a timestamp uses the supported RFC 3339 date-time form.
#[must_use]
pub fn is_rfc3339_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return false;
    }

    let Some(year) = digits(bytes, 0, 4) else {
        return false;
    };
    let Some(month) = digits(bytes, 5, 2) else {
        return false;
    };
    let Some(day) = digits(bytes, 8, 2) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) || day == 0 || day > month_days(year, month) {
        return false;
    }

    let Some(hour) = digits(bytes, 11, 2) else {
        return false;
    };
    let Some(minute) = digits(bytes, 14, 2) else {
        return false;
    };
    let Some(second) = digits(bytes, 17, 2) else {
        return false;
    };
    if hour > 23 || minute > 59 || second > 60 {
        return false;
    }

    let mut index = 19;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == start {
            return false;
        }
    }

    match bytes.get(index) {
        Some(b'Z') => index + 1 == bytes.len(),
        Some(b'+' | b'-') if index + 6 == bytes.len() => {
            bytes.get(index + 3) == Some(&b':')
                && digits(bytes, index + 1, 2).is_some_and(|hour| hour <= 23)
                && digits(bytes, index + 4, 2).is_some_and(|minute| minute <= 59)
        }
        _ => false,
    }
}

fn digits(bytes: &[u8], start: usize, count: usize) -> Option<u32> {
    let slice = bytes.get(start..start.checked_add(count)?)?;
    if !slice.iter().all(u8::is_ascii_digit) {
        return None;
    }
    slice.iter().try_fold(0_u32, |value, byte| {
        value.checked_mul(10)?.checked_add(u32::from(*byte - b'0'))
    })
}

const fn leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn month_days(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Borrowed Writer manifest values after a serialization adapter decodes JSON.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriterManifest<'a> {
    /// Package format family.
    pub format_family: &'a str,
    /// Package format version.
    pub format_version: FormatVersion,
    /// Declared Office document type.
    pub document_type: DocumentType,
    /// Document or template role.
    pub document_role: DocumentRole,
    /// Stable document UUID.
    pub document_id: &'a str,
    /// Manifest-declared required package parts.
    pub required_parts: &'a [&'a str],
    /// Writer document entry point.
    pub document_entry_point: &'a str,
    /// Creation timestamp.
    pub created_at: &'a str,
    /// Last-saved timestamp.
    pub last_saved_at: &'a str,
    /// Producer application name.
    pub producer_name: &'a str,
    /// Producer application version.
    pub producer_version: &'a str,
}

/// Writer manifest semantic validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriterManifestError {
    /// Format family is not the Office family.
    UnsupportedFormatFamily,
    /// Major version is unsupported.
    UnsupportedMajorVersion(u16),
    /// Document type is not Writer.
    WrongDocumentType,
    /// Stable document identifier is invalid.
    InvalidDocumentId,
    /// Required-part list is empty.
    EmptyRequiredParts,
    /// A required part is empty, unsafe, or repeated.
    InvalidRequiredPart,
    /// Writer document entry point is incorrect.
    InvalidDocumentEntryPoint,
    /// Creation timestamp is invalid.
    InvalidCreatedAt,
    /// Last-saved timestamp is invalid.
    InvalidLastSavedAt,
    /// Producer name or version is empty.
    InvalidProducer,
}

impl fmt::Display for WriterManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnsupportedFormatFamily => "unsupported Office format family",
            Self::UnsupportedMajorVersion(_) => "unsupported Office package major version",
            Self::WrongDocumentType => "Writer manifest must declare the Writer document type",
            Self::InvalidDocumentId => "manifest document ID must be a canonical supported UUID",
            Self::EmptyRequiredParts => "manifest required_parts must not be empty",
            Self::InvalidRequiredPart => "manifest required_parts contains an invalid path",
            Self::InvalidDocumentEntryPoint => {
                "Writer manifest document entry point must be content/document.json"
            }
            Self::InvalidCreatedAt => "manifest created_at is not valid RFC 3339",
            Self::InvalidLastSavedAt => "manifest last_saved_at is not valid RFC 3339",
            Self::InvalidProducer => "manifest producer name and version must not be empty",
        };
        match self {
            Self::UnsupportedMajorVersion(major) => write!(f, "{message}: {major}"),
            _ => f.write_str(message),
        }
    }
}

impl Error for WriterManifestError {}

/// Validates the decoded semantic fields of a Writer v1 manifest.
///
/// # Errors
///
/// Returns `WriterManifestError` when a governed manifest invariant is violated.
pub fn validate_writer_manifest(manifest: &WriterManifest<'_>) -> Result<(), WriterManifestError> {
    if manifest.format_family != FORMAT_FAMILY {
        return Err(WriterManifestError::UnsupportedFormatFamily);
    }
    if !manifest.format_version.has_supported_major() {
        return Err(WriterManifestError::UnsupportedMajorVersion(
            manifest.format_version.major(),
        ));
    }
    if manifest.document_type != DocumentType::Writer {
        return Err(WriterManifestError::WrongDocumentType);
    }
    if !is_canonical_uuid(manifest.document_id) {
        return Err(WriterManifestError::InvalidDocumentId);
    }
    if manifest.required_parts.is_empty() {
        return Err(WriterManifestError::EmptyRequiredParts);
    }

    let mut seen = BTreeSet::new();
    for part in manifest.required_parts.iter().copied() {
        if part.is_empty() || validate_entry_path(part).is_err() || !seen.insert(part) {
            return Err(WriterManifestError::InvalidRequiredPart);
        }
    }

    if manifest.document_entry_point != WRITER_DOCUMENT_PART {
        return Err(WriterManifestError::InvalidDocumentEntryPoint);
    }
    if !is_rfc3339_timestamp(manifest.created_at) {
        return Err(WriterManifestError::InvalidCreatedAt);
    }
    if !is_rfc3339_timestamp(manifest.last_saved_at) {
        return Err(WriterManifestError::InvalidLastSavedAt);
    }
    if manifest.producer_name.is_empty() || manifest.producer_version.is_empty() {
        return Err(WriterManifestError::InvalidProducer);
    }

    Ok(())
}

/// Borrowed Writer v1 text run after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriterRun<'a> {
    /// Stable run UUID.
    pub id: &'a str,
    /// Logical Unicode text.
    pub text: &'a str,
    /// Optional style identifier.
    pub style: Option<&'a str>,
}

/// Borrowed Writer v1 paragraph after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriterParagraph<'a> {
    /// Stable paragraph UUID.
    pub id: &'a str,
    /// Optional paragraph style identifier.
    pub style: Option<&'a str>,
    /// Paragraph text runs.
    pub runs: &'a [WriterRun<'a>],
}

/// Borrowed Writer v1 semantic document after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriterDocument<'a> {
    /// Writer content schema version.
    pub schema_version: FormatVersion,
    /// Stable document UUID.
    pub document_id: &'a str,
    /// Paragraphs in logical document order.
    pub blocks: &'a [WriterParagraph<'a>],
}

/// Writer document semantic validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriterDocumentError {
    /// Content schema major version is unsupported.
    UnsupportedSchemaMajor(u16),
    /// Document identifier is invalid.
    InvalidDocumentId,
    /// Document identifier does not match the manifest.
    DocumentIdMismatch,
    /// A paragraph or run identifier is invalid.
    InvalidObjectId,
    /// A paragraph or run identifier is repeated.
    DuplicateObjectId(String),
}

impl fmt::Display for WriterDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaMajor(major) => {
                write!(f, "unsupported Writer schema major version: {major}")
            }
            Self::InvalidDocumentId => f.write_str("Writer document ID is invalid"),
            Self::DocumentIdMismatch => f.write_str("Writer document ID does not match manifest"),
            Self::InvalidObjectId => f.write_str("Writer object ID is invalid"),
            Self::DuplicateObjectId(id) => write!(f, "Writer object ID is duplicated: {id}"),
        }
    }
}

impl Error for WriterDocumentError {}

/// Validates decoded Writer v1 content semantics against the package manifest.
///
/// # Errors
///
/// Returns `WriterDocumentError` when version or identity invariants are violated.
pub fn validate_writer_document(
    document: &WriterDocument<'_>,
    expected_document_id: &str,
) -> Result<(), WriterDocumentError> {
    if document.schema_version.major() != FORMAT_MAJOR_V1 {
        return Err(WriterDocumentError::UnsupportedSchemaMajor(
            document.schema_version.major(),
        ));
    }
    if !is_canonical_uuid(document.document_id) {
        return Err(WriterDocumentError::InvalidDocumentId);
    }
    if document.document_id != expected_document_id {
        return Err(WriterDocumentError::DocumentIdMismatch);
    }

    let mut seen = BTreeSet::new();
    for block in document.blocks {
        if !is_canonical_uuid(block.id) {
            return Err(WriterDocumentError::InvalidObjectId);
        }
        if !seen.insert(block.id) {
            return Err(WriterDocumentError::DuplicateObjectId(block.id.to_owned()));
        }
        for run in block.runs {
            if !is_canonical_uuid(run.id) {
                return Err(WriterDocumentError::InvalidObjectId);
            }
            if !seen.insert(run.id) {
                return Err(WriterDocumentError::DuplicateObjectId(run.id.to_owned()));
            }
        }
    }

    Ok(())
}

/// Borrowed portable package metadata after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageMetadata<'a> {
    /// Metadata schema version.
    pub schema_version: u16,
    /// Portable document title.
    pub title: &'a str,
    /// Optional language identifier.
    pub language: Option<&'a str>,
}

/// Portable metadata semantic validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataError {
    /// Metadata schema version is unsupported.
    UnsupportedSchemaVersion(u16),
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported metadata schema version: {version}")
            }
        }
    }
}

impl Error for MetadataError {}

/// Validates decoded portable metadata semantics.
///
/// # Errors
///
/// Returns `MetadataError` when the metadata schema version is unsupported.
pub fn validate_package_metadata(metadata: &PackageMetadata<'_>) -> Result<(), MetadataError> {
    if metadata.schema_version != 1 {
        return Err(MetadataError::UnsupportedSchemaVersion(
            metadata.schema_version,
        ));
    }
    Ok(())
}

/// Relationship kind defined by the v1 relationships schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipKind {
    /// Package-internal relationship.
    Internal,
    /// Relationship to an embedded package resource.
    Embedded,
    /// Relationship to an external resource.
    External,
}

/// Borrowed relationship after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageRelationship<'a> {
    /// Stable relationship UUID.
    pub id: &'a str,
    /// Relationship source.
    pub source: &'a str,
    /// Relationship target.
    pub target: &'a str,
    /// Relationship kind.
    pub kind: RelationshipKind,
    /// Whether the relationship is required.
    pub required: Option<bool>,
    /// Optional declared media type.
    pub media_type: Option<&'a str>,
}

/// Borrowed relationships record after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageRelationships<'a> {
    /// Relationships schema version.
    pub schema_version: u16,
    /// Relationship entries.
    pub relationships: &'a [PackageRelationship<'a>],
}

/// Relationships semantic validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipsError {
    /// Relationships schema version is unsupported.
    UnsupportedSchemaVersion(u16),
    /// A relationship ID is not a canonical supported UUID.
    InvalidRelationshipId {
        /// Zero-based relationship index.
        index: usize,
    },
    /// A relationship ID is repeated.
    DuplicateRelationshipId(String),
}

impl fmt::Display for RelationshipsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported relationships schema version: {version}")
            }
            Self::InvalidRelationshipId { index } => {
                write!(f, "relationship ID at index {index} is invalid")
            }
            Self::DuplicateRelationshipId(id) => {
                write!(f, "relationship ID is duplicated: {id}")
            }
        }
    }
}

impl Error for RelationshipsError {}

/// Validates decoded package relationships.
///
/// # Errors
///
/// Returns `RelationshipsError` for unsupported schema versions, invalid UUIDs,
/// or duplicate relationship IDs.
pub fn validate_package_relationships(
    record: &PackageRelationships<'_>,
) -> Result<(), RelationshipsError> {
    if record.schema_version != 1 {
        return Err(RelationshipsError::UnsupportedSchemaVersion(
            record.schema_version,
        ));
    }

    let mut seen = BTreeSet::new();
    for (index, relationship) in record.relationships.iter().enumerate() {
        if !is_canonical_uuid(relationship.id) {
            return Err(RelationshipsError::InvalidRelationshipId { index });
        }
        if !seen.insert(relationship.id) {
            return Err(RelationshipsError::DuplicateRelationshipId(
                relationship.id.to_owned(),
            ));
        }
    }
    Ok(())
}

/// Borrowed compatibility record after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageCompatibility<'a> {
    /// Compatibility schema version.
    pub schema_version: u16,
    /// Feature identifiers used by the package.
    pub used_features: &'a [&'a str],
    /// Capabilities required to interpret the package.
    pub required_capabilities: &'a [&'a str],
}

/// Compatibility semantic validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityError {
    /// Compatibility schema version is unsupported.
    UnsupportedSchemaVersion(u16),
    /// A used-feature identifier is empty or repeated.
    InvalidUsedFeatures,
    /// A required-capability identifier is empty or repeated.
    InvalidRequiredCapabilities,
}

impl fmt::Display for CompatibilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnsupportedSchemaVersion(_) => "unsupported compatibility schema version",
            Self::InvalidUsedFeatures => "used_features must contain unique nonempty strings",
            Self::InvalidRequiredCapabilities => {
                "required_capabilities must contain unique nonempty strings"
            }
        };
        match self {
            Self::UnsupportedSchemaVersion(version) => write!(f, "{message}: {version}"),
            _ => f.write_str(message),
        }
    }
}

impl Error for CompatibilityError {}

/// Validates decoded compatibility semantics.
///
/// # Errors
///
/// Returns `CompatibilityError` for unsupported schema versions or invalid
/// feature/capability arrays.
pub fn validate_package_compatibility(
    record: &PackageCompatibility<'_>,
) -> Result<(), CompatibilityError> {
    if record.schema_version != 1 {
        return Err(CompatibilityError::UnsupportedSchemaVersion(
            record.schema_version,
        ));
    }
    if !unique_nonempty(record.used_features) {
        return Err(CompatibilityError::InvalidUsedFeatures);
    }
    if !unique_nonempty(record.required_capabilities) {
        return Err(CompatibilityError::InvalidRequiredCapabilities);
    }
    Ok(())
}

fn unique_nonempty(values: &[&str]) -> bool {
    let mut seen = BTreeSet::new();
    values
        .iter()
        .copied()
        .all(|value| !value.is_empty() && seen.insert(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MANIFEST_PART, MIMETYPE_PART, WRITER_REQUIRED_PARTS};

    fn valid_manifest<'a>(parts: &'a [&'a str]) -> WriterManifest<'a> {
        WriterManifest {
            format_family: FORMAT_FAMILY,
            format_version: FormatVersion::v1(),
            document_type: DocumentType::Writer,
            document_role: DocumentRole::Document,
            document_id: "a1a11111-1111-4111-8111-111111111111",
            required_parts: parts,
            document_entry_point: WRITER_DOCUMENT_PART,
            created_at: "2026-09-17T00:00:00Z",
            last_saved_at: "2026-09-17T00:00:00Z",
            producer_name: "GoreeCloud Office reference package builder",
            producer_version: "0.1.0-prebootstrap",
        }
    }

    #[test]
    fn metadata_fixture_semantics_validate() {
        let metadata = PackageMetadata {
            schema_version: 1,
            title: "GoreeCloud Office format validation fixture",
            language: Some("en-US"),
        };
        assert_eq!(validate_package_metadata(&metadata), Ok(()));

        let unsupported = PackageMetadata {
            schema_version: 2,
            ..metadata
        };
        assert_eq!(
            validate_package_metadata(&unsupported),
            Err(MetadataError::UnsupportedSchemaVersion(2))
        );
    }

    #[test]
    fn relationships_validate_uuid_uniqueness() {
        let relationships = [
            PackageRelationship {
                id: "d4d44444-4444-4444-8444-444444444444",
                source: "content/document.json",
                target: "assets/image.png",
                kind: RelationshipKind::Embedded,
                required: Some(false),
                media_type: Some("image/png"),
            },
            PackageRelationship {
                id: "e5e55555-5555-4555-8555-555555555555",
                source: "content/document.json",
                target: "https://example.invalid/",
                kind: RelationshipKind::External,
                required: Some(false),
                media_type: None,
            },
        ];
        let record = PackageRelationships {
            schema_version: 1,
            relationships: &relationships,
        };
        assert_eq!(validate_package_relationships(&record), Ok(()));

        let duplicate = [relationships[0], relationships[0]];
        let duplicate_record = PackageRelationships {
            schema_version: 1,
            relationships: &duplicate,
        };
        assert!(matches!(
            validate_package_relationships(&duplicate_record),
            Err(RelationshipsError::DuplicateRelationshipId(_))
        ));
    }

    #[test]
    fn relationships_reject_invalid_uuid_and_schema_version() {
        let relationships = [PackageRelationship {
            id: "not-a-uuid",
            source: "",
            target: "",
            kind: RelationshipKind::Internal,
            required: None,
            media_type: None,
        }];
        let record = PackageRelationships {
            schema_version: 1,
            relationships: &relationships,
        };
        assert_eq!(
            validate_package_relationships(&record),
            Err(RelationshipsError::InvalidRelationshipId { index: 0 })
        );

        let empty: [PackageRelationship<'_>; 0] = [];
        let unsupported = PackageRelationships {
            schema_version: 2,
            relationships: &empty,
        };
        assert_eq!(
            validate_package_relationships(&unsupported),
            Err(RelationshipsError::UnsupportedSchemaVersion(2))
        );
    }

    #[test]
    fn compatibility_fixture_and_uniqueness_rules_validate() {
        let used_features = ["writer.paragraph", "writer.text-run"];
        let required_capabilities: [&str; 0] = [];
        let record = PackageCompatibility {
            schema_version: 1,
            used_features: &used_features,
            required_capabilities: &required_capabilities,
        };
        assert_eq!(validate_package_compatibility(&record), Ok(()));

        let duplicate = ["writer.paragraph", "writer.paragraph"];
        let invalid = PackageCompatibility {
            schema_version: 1,
            used_features: &duplicate,
            required_capabilities: &required_capabilities,
        };
        assert_eq!(
            validate_package_compatibility(&invalid),
            Err(CompatibilityError::InvalidUsedFeatures)
        );

        let empty_capability = [""];
        let invalid = PackageCompatibility {
            schema_version: 1,
            used_features: &used_features,
            required_capabilities: &empty_capability,
        };
        assert_eq!(
            validate_package_compatibility(&invalid),
            Err(CompatibilityError::InvalidRequiredCapabilities)
        );
    }

    #[test]
    fn canonical_uuid_rules_match_v1_schema() {
        assert!(is_canonical_uuid("a1a11111-1111-4111-8111-111111111111"));
        assert!(!is_canonical_uuid("A1A11111-1111-4111-8111-111111111111"));
        assert!(!is_canonical_uuid("a1a11111-1111-7111-8111-111111111111"));
        assert!(!is_canonical_uuid("a1a11111-1111-4111-7111-111111111111"));
    }

    #[test]
    fn timestamp_rules_accept_fixture_fraction_and_offset() {
        assert!(is_rfc3339_timestamp("2026-09-17T00:00:00Z"));
        assert!(is_rfc3339_timestamp("2026-09-17T12:34:56.123-04:00"));
        assert!(!is_rfc3339_timestamp("2025-02-29T12:00:00Z"));
        assert!(!is_rfc3339_timestamp("2026-09-17 00:00:00Z"));
    }

    #[test]
    fn writer_manifest_fixture_semantics_validate() {
        let parts = WRITER_REQUIRED_PARTS;
        assert_eq!(validate_writer_manifest(&valid_manifest(&parts)), Ok(()));
    }

    #[test]
    fn writer_manifest_rejects_identity_required_part_and_entry_point_errors() {
        let parts = WRITER_REQUIRED_PARTS;
        let mut manifest = valid_manifest(&parts);
        manifest.format_version = FormatVersion::new(2, 0);
        assert_eq!(
            validate_writer_manifest(&manifest),
            Err(WriterManifestError::UnsupportedMajorVersion(2))
        );

        let duplicate = [MIMETYPE_PART, MANIFEST_PART, MANIFEST_PART];
        assert_eq!(
            validate_writer_manifest(&valid_manifest(&duplicate)),
            Err(WriterManifestError::InvalidRequiredPart)
        );

        let unsafe_parts = [MIMETYPE_PART, "../manifest.json"];
        assert_eq!(
            validate_writer_manifest(&valid_manifest(&unsafe_parts)),
            Err(WriterManifestError::InvalidRequiredPart)
        );

        manifest = valid_manifest(&parts);
        manifest.document_entry_point = "content/other.json";
        assert_eq!(
            validate_writer_manifest(&manifest),
            Err(WriterManifestError::InvalidDocumentEntryPoint)
        );
    }

    #[test]
    fn writer_manifest_rejects_bad_timestamp_and_producer() {
        let parts = WRITER_REQUIRED_PARTS;
        let mut manifest = valid_manifest(&parts);
        manifest.created_at = "2026-02-30T00:00:00Z";
        assert_eq!(
            validate_writer_manifest(&manifest),
            Err(WriterManifestError::InvalidCreatedAt)
        );

        manifest = valid_manifest(&parts);
        manifest.producer_name = "";
        assert_eq!(
            validate_writer_manifest(&manifest),
            Err(WriterManifestError::InvalidProducer)
        );
    }

    #[test]
    fn writer_document_fixture_semantics_validate() {
        let runs = [WriterRun {
            id: "c3c33333-3333-4333-8333-333333333333",
            text: "Hello from GoreeCloud Writer.",
            style: None,
        }];
        let blocks = [WriterParagraph {
            id: "b2b22222-2222-4222-8222-222222222222",
            style: None,
            runs: &runs,
        }];
        let document = WriterDocument {
            schema_version: FormatVersion::v1(),
            document_id: "a1a11111-1111-4111-8111-111111111111",
            blocks: &blocks,
        };
        assert_eq!(
            validate_writer_document(&document, "a1a11111-1111-4111-8111-111111111111"),
            Ok(())
        );
    }

    #[test]
    fn writer_document_rejects_mismatch_and_duplicate_ids() {
        let runs = [WriterRun {
            id: "b2b22222-2222-4222-8222-222222222222",
            text: "duplicate",
            style: None,
        }];
        let blocks = [WriterParagraph {
            id: "b2b22222-2222-4222-8222-222222222222",
            style: None,
            runs: &runs,
        }];
        let document = WriterDocument {
            schema_version: FormatVersion::v1(),
            document_id: "a1a11111-1111-4111-8111-111111111111",
            blocks: &blocks,
        };
        assert_eq!(
            validate_writer_document(&document, "d4d44444-4444-4444-8444-444444444444"),
            Err(WriterDocumentError::DocumentIdMismatch)
        );
        assert_eq!(
            validate_writer_document(&document, "a1a11111-1111-4111-8111-111111111111"),
            Err(WriterDocumentError::DuplicateObjectId(
                "b2b22222-2222-4222-8222-222222222222".to_owned()
            ))
        );
    }
}
