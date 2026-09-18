//! Concrete local reader and validator for GoreeCloud Writer v1 native packages.
//!
//! The adapter keeps third-party ZIP, JSON, JSON Schema, and digest types behind
//! Office-owned APIs. It never extracts archive entries to the filesystem.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io::{Cursor, Read};

use jsonschema::{Draft, options};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::integrity::{
    IntegrityCriticality, IntegrityEntry, IntegrityError, PackageIntegrity,
    validate_package_integrity,
};
use crate::semantic::{
    CompatibilityError, DocumentRole, MetadataError, PackageCompatibility, PackageMetadata,
    PackageRelationship, PackageRelationships, RelationshipKind, RelationshipsError,
    WriterDocument, WriterDocumentError, WriterManifest, WriterManifestError, WriterParagraph,
    WriterRun, validate_package_compatibility, validate_package_metadata,
    validate_package_relationships, validate_writer_document, validate_writer_manifest,
};
use crate::{
    COMPATIBILITY_PART, CompressionMethod, DocumentType, FormatVersion, INTEGRITY_PART,
    MANIFEST_PART, METADATA_PART, MIMETYPE_PART, PackageEntry, PackageLimits,
    PackageStructureError, RELATIONSHIPS_PART, WRITER_DOCUMENT_PART, validate_writer_entry_table,
};

const MANIFEST_SCHEMA: &str =
    include_str!("../test-data/v1/schemas/office-package-manifest-v1.schema.json");
const METADATA_SCHEMA: &str =
    include_str!("../test-data/v1/schemas/office-package-metadata-v1.schema.json");
const RELATIONSHIPS_SCHEMA: &str =
    include_str!("../test-data/v1/schemas/office-package-relationships-v1.schema.json");
const COMPATIBILITY_SCHEMA: &str =
    include_str!("../test-data/v1/schemas/office-package-compatibility-v1.schema.json");
const INTEGRITY_SCHEMA: &str =
    include_str!("../test-data/v1/schemas/office-package-integrity-v1.schema.json");
const WRITER_DOCUMENT_SCHEMA: &str =
    include_str!("../test-data/v1/schemas/office-writer-document-v1.schema.json");

/// Summary returned after a Writer package passes all currently implemented
/// container, schema, semantic, and cryptographic integrity checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWriterPackage {
    document_id: String,
    title: String,
    paragraph_count: usize,
    relationship_count: usize,
    used_features: Vec<String>,
}

impl ValidatedWriterPackage {
    /// Returns the stable document UUID.
    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Returns the portable document title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the number of paragraph blocks in the validated minimal Writer
    /// content model.
    #[must_use]
    pub const fn paragraph_count(&self) -> usize {
        self.paragraph_count
    }

    /// Returns the number of package relationships.
    #[must_use]
    pub const fn relationship_count(&self) -> usize {
        self.relationship_count
    }

    /// Returns feature identifiers declared by the compatibility record.
    #[must_use]
    pub fn used_features(&self) -> &[String] {
        &self.used_features
    }
}

/// Failures produced by the concrete Writer package adapter.
#[derive(Debug)]
pub enum WriterPackageReadError {
    /// The supplied file name does not use the canonical Writer extension.
    WrongExtension,
    /// The ZIP container could not be opened or inspected.
    InvalidZip(String),
    /// Compressed file ranges overlap in the ZIP container.
    OverlappingEntries,
    /// ZIP-level encryption is prohibited by the v1 Office package contract.
    EncryptedEntry {
        /// Package path of the encrypted entry.
        path: String,
    },
    /// The archive entry table violates Office package structure rules.
    Structure(PackageStructureError),
    /// An approved entry could not be read within its validated bounds.
    EntryRead {
        /// Package path being read.
        path: String,
        /// Underlying read failure description.
        message: String,
    },
    /// Actual decompressed bytes differ from validated archive metadata.
    EntrySizeMismatch {
        /// Package path with the mismatch.
        path: String,
        /// Size declared by archive metadata.
        declared: u64,
        /// Bytes actually produced by the decoder.
        actual: u64,
    },
    /// The mimetype payload is not the Writer v1 provisional media type.
    MimetypeMismatch,
    /// A required JSON record could not be decoded as UTF-8 JSON.
    InvalidJson {
        /// Package path containing invalid JSON.
        path: &'static str,
        /// Decoder failure description.
        message: String,
    },
    /// An embedded governed JSON Schema could not be compiled.
    InvalidBundledSchema {
        /// Logical schema name.
        schema: &'static str,
        /// Schema failure description.
        message: String,
    },
    /// A decoded JSON record violates its governed schema.
    SchemaViolation {
        /// Package path that failed schema validation.
        path: &'static str,
        /// Validation failure description.
        message: String,
    },
    /// A schema-valid decoded record could not be converted to the Office
    /// semantic transport representation.
    InvalidDecodedRecord {
        /// Package path being converted.
        path: &'static str,
        /// Conversion failure description.
        message: String,
    },
    /// Writer manifest semantics are invalid.
    Manifest(WriterManifestError),
    /// Writer content semantics are invalid.
    Document(WriterDocumentError),
    /// Portable metadata semantics are invalid.
    Metadata(MetadataError),
    /// Relationship semantics are invalid.
    Relationships(RelationshipsError),
    /// Compatibility semantics are invalid.
    Compatibility(CompatibilityError),
    /// Integrity-record semantics are invalid.
    Integrity(IntegrityError),
    /// A protected package part does not match its declared SHA-256 digest.
    DigestMismatch {
        /// Package path whose digest differs.
        path: String,
    },
}

impl fmt::Display for WriterPackageReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongExtension => f.write_str("Writer packages must use the .gcwriter extension"),
            Self::InvalidZip(message) => write!(f, "invalid Writer ZIP container: {message}"),
            Self::OverlappingEntries => {
                f.write_str("Writer ZIP container contains overlapping compressed entry ranges")
            }
            Self::EncryptedEntry { path } => {
                write!(f, "ZIP-level encryption is not permitted for entry {path}")
            }
            Self::Structure(error) => write!(f, "Writer package structure is invalid: {error}"),
            Self::EntryRead { path, message } => {
                write!(f, "failed to read Writer package entry {path}: {message}")
            }
            Self::EntrySizeMismatch {
                path,
                declared,
                actual,
            } => write!(
                f,
                "Writer package entry {path} declared {declared} bytes but produced {actual}"
            ),
            Self::MimetypeMismatch => f.write_str("Writer package mimetype payload is incorrect"),
            Self::InvalidJson { path, message } => {
                write!(f, "invalid JSON in {path}: {message}")
            }
            Self::InvalidBundledSchema { schema, message } => {
                write!(f, "invalid bundled Office schema {schema}: {message}")
            }
            Self::SchemaViolation { path, message } => {
                write!(f, "JSON Schema validation failed for {path}: {message}")
            }
            Self::InvalidDecodedRecord { path, message } => {
                write!(f, "decoded record {path} is not supported: {message}")
            }
            Self::Manifest(error) => write!(f, "Writer manifest validation failed: {error}"),
            Self::Document(error) => write!(f, "Writer content validation failed: {error}"),
            Self::Metadata(error) => write!(f, "Writer metadata validation failed: {error}"),
            Self::Relationships(error) => {
                write!(f, "Writer relationships validation failed: {error}")
            }
            Self::Compatibility(error) => {
                write!(f, "Writer compatibility validation failed: {error}")
            }
            Self::Integrity(error) => write!(f, "Writer integrity validation failed: {error}"),
            Self::DigestMismatch { path } => {
                write!(f, "SHA-256 digest mismatch for Writer package entry {path}")
            }
        }
    }
}

impl Error for WriterPackageReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Structure(error) => Some(error),
            Self::Manifest(error) => Some(error),
            Self::Document(error) => Some(error),
            Self::Metadata(error) => Some(error),
            Self::Relationships(error) => Some(error),
            Self::Compatibility(error) => Some(error),
            Self::Integrity(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct EntryMetadata {
    path: String,
    compression: CompressionMethod,
    uncompressed_bytes: u64,
    compressed_bytes: u64,
    symbolic_link: bool,
    encrypted: bool,
}

#[derive(Deserialize)]
struct VersionRecord {
    major: u16,
    minor: u16,
}

#[derive(Deserialize)]
struct ProducerRecord {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct ManifestRecord {
    format_family: String,
    format_version: VersionRecord,
    document_type: String,
    document_role: String,
    document_id: String,
    required_parts: Vec<String>,
    entry_points: BTreeMap<String, String>,
    created_at: String,
    last_saved_at: String,
    producer: ProducerRecord,
}

#[derive(Deserialize)]
struct MetadataRecord {
    schema_version: u16,
    title: String,
    language: Option<String>,
}

#[derive(Deserialize)]
struct RelationshipRecord {
    id: String,
    source: String,
    target: String,
    kind: String,
    required: Option<bool>,
    media_type: Option<String>,
}

#[derive(Deserialize)]
struct RelationshipsRecord {
    schema_version: u16,
    relationships: Vec<RelationshipRecord>,
}

#[derive(Deserialize)]
struct CompatibilityRecord {
    schema_version: u16,
    used_features: Vec<String>,
    required_capabilities: Vec<String>,
}

#[derive(Deserialize)]
struct RunRecord {
    id: String,
    text: String,
    style: Option<String>,
}

#[derive(Deserialize)]
struct BlockRecord {
    id: String,
    #[serde(rename = "type")]
    _block_type: String,
    runs: Vec<RunRecord>,
    style: Option<String>,
}

#[derive(Deserialize)]
struct WriterDocumentRecord {
    schema_version: VersionRecord,
    document_id: String,
    blocks: Vec<BlockRecord>,
}

#[derive(Deserialize)]
struct IntegrityEntryRecord {
    path: String,
    byte_length: u64,
    sha256: String,
    criticality: Option<String>,
    media_type: Option<String>,
}

#[derive(Deserialize)]
struct IntegrityRecord {
    schema_version: u16,
    algorithm: String,
    entries: Vec<IntegrityEntryRecord>,
}

/// Opens and validates a local Writer package entirely in memory.
///
/// Validation currently covers the file extension, ZIP container structure,
/// overlapping/encrypted entries, Office path and resource rules, the exact
/// Writer mimetype, all six governed JSON Schemas, existing Office semantic
/// validators, exact integrity coverage/byte lengths, and SHA-256 digests.
/// No package content is extracted to the filesystem and no network access is
/// performed.
///
/// # Errors
///
/// Returns WriterPackageReadError when any implemented v1 package invariant
/// fails.
pub fn validate_writer_package(
    file_name: &str,
    bytes: &[u8],
    limits: PackageLimits,
) -> Result<ValidatedWriterPackage, WriterPackageReadError> {
    validate_writer_extension(file_name)?;

    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| WriterPackageReadError::InvalidZip(error.to_string()))?;
    if archive
        .has_overlapping_files()
        .map_err(|error| WriterPackageReadError::InvalidZip(error.to_string()))?
    {
        return Err(WriterPackageReadError::OverlappingEntries);
    }

    let metadata = collect_entry_metadata(&mut archive)?;
    if let Some(entry) = metadata.iter().find(|entry| entry.encrypted) {
        return Err(WriterPackageReadError::EncryptedEntry {
            path: entry.path.clone(),
        });
    }

    let package_entries = metadata
        .iter()
        .map(|entry| {
            PackageEntry::new(
                &entry.path,
                entry.compression,
                entry.uncompressed_bytes,
                entry.compressed_bytes,
                entry.symbolic_link,
            )
        })
        .collect::<Vec<_>>();
    validate_writer_entry_table(&package_entries, limits)
        .map_err(WriterPackageReadError::Structure)?;

    let payloads = read_payloads(&mut archive, &metadata, limits)?;
    let writer_mimetype = DocumentType::Writer.media_type().as_bytes();
    if payloads.get(MIMETYPE_PART).map(Vec::as_slice) != Some(writer_mimetype) {
        return Err(WriterPackageReadError::MimetypeMismatch);
    }

    let manifest: ManifestRecord = decode_schema_checked(
        MANIFEST_PART,
        "package-manifest-v1",
        MANIFEST_SCHEMA,
        required_payload(&payloads, MANIFEST_PART)?,
    )?;
    validate_manifest_record(&manifest)?;

    let metadata_record: MetadataRecord = decode_schema_checked(
        METADATA_PART,
        "package-metadata-v1",
        METADATA_SCHEMA,
        required_payload(&payloads, METADATA_PART)?,
    )?;
    validate_metadata_record(&metadata_record)?;

    let relationships: RelationshipsRecord = decode_schema_checked(
        RELATIONSHIPS_PART,
        "package-relationships-v1",
        RELATIONSHIPS_SCHEMA,
        required_payload(&payloads, RELATIONSHIPS_PART)?,
    )?;
    validate_relationships_record(&relationships)?;

    let compatibility: CompatibilityRecord = decode_schema_checked(
        COMPATIBILITY_PART,
        "package-compatibility-v1",
        COMPATIBILITY_SCHEMA,
        required_payload(&payloads, COMPATIBILITY_PART)?,
    )?;
    validate_compatibility_record(&compatibility)?;

    let document: WriterDocumentRecord = decode_schema_checked(
        WRITER_DOCUMENT_PART,
        "writer-document-v1",
        WRITER_DOCUMENT_SCHEMA,
        required_payload(&payloads, WRITER_DOCUMENT_PART)?,
    )?;
    validate_document_record(&document, &manifest.document_id)?;

    let integrity: IntegrityRecord = decode_schema_checked(
        INTEGRITY_PART,
        "package-integrity-v1",
        INTEGRITY_SCHEMA,
        required_payload(&payloads, INTEGRITY_PART)?,
    )?;
    validate_integrity_record(&integrity, &package_entries)?;
    verify_integrity_digests(&integrity, &payloads)?;

    Ok(ValidatedWriterPackage {
        document_id: manifest.document_id,
        title: metadata_record.title,
        paragraph_count: document.blocks.len(),
        relationship_count: relationships.relationships.len(),
        used_features: compatibility.used_features,
    })
}

fn validate_writer_extension(file_name: &str) -> Result<(), WriterPackageReadError> {
    let extension = file_name.rsplit_once('.').map(|(_, extension)| extension);
    if extension.and_then(DocumentType::from_extension) != Some(DocumentType::Writer) {
        return Err(WriterPackageReadError::WrongExtension);
    }
    Ok(())
}

fn collect_entry_metadata(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<Vec<EntryMetadata>, WriterPackageReadError> {
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let file = archive
            .by_index_raw(index)
            .map_err(|error| WriterPackageReadError::InvalidZip(error.to_string()))?;
        entries.push(EntryMetadata {
            path: file.name().to_owned(),
            compression: map_compression(file.compression()),
            uncompressed_bytes: file.size(),
            compressed_bytes: file.compressed_size(),
            symbolic_link: file.is_symlink(),
            encrypted: file.encrypted(),
        });
    }
    Ok(entries)
}

fn map_compression(method: zip::CompressionMethod) -> CompressionMethod {
    match method {
        zip::CompressionMethod::Stored => CompressionMethod::Store,
        zip::CompressionMethod::Deflated => CompressionMethod::Deflate,
        zip::CompressionMethod::Unsupported(method) => CompressionMethod::Unsupported(method),
        _ => CompressionMethod::Unsupported(u16::MAX),
    }
}

fn read_payloads(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    entries: &[EntryMetadata],
    limits: PackageLimits,
) -> Result<BTreeMap<String, Vec<u8>>, WriterPackageReadError> {
    let mut payloads = BTreeMap::new();
    for entry in entries {
        let mut file =
            archive
                .by_name(&entry.path)
                .map_err(|error| WriterPackageReadError::EntryRead {
                    path: entry.path.clone(),
                    message: error.to_string(),
                })?;
        let ceiling = entry
            .uncompressed_bytes
            .min(limits.max_entry_uncompressed_bytes())
            .saturating_add(1);
        let capacity = usize::try_from(entry.uncompressed_bytes)
            .expect("validated Office entry limit must fit usize");
        let mut payload = Vec::with_capacity(capacity);
        (&mut file)
            .take(ceiling)
            .read_to_end(&mut payload)
            .map_err(|error| WriterPackageReadError::EntryRead {
                path: entry.path.clone(),
                message: error.to_string(),
            })?;
        let actual = u64::try_from(payload.len()).unwrap_or(u64::MAX);
        if actual != entry.uncompressed_bytes {
            return Err(WriterPackageReadError::EntrySizeMismatch {
                path: entry.path.clone(),
                declared: entry.uncompressed_bytes,
                actual,
            });
        }
        payloads.insert(entry.path.clone(), payload);
    }
    Ok(payloads)
}

fn required_payload<'a>(
    payloads: &'a BTreeMap<String, Vec<u8>>,
    path: &'static str,
) -> Result<&'a [u8], WriterPackageReadError> {
    payloads.get(path).map(Vec::as_slice).ok_or_else(|| {
        WriterPackageReadError::Structure(PackageStructureError::MissingRequiredPart { path })
    })
}

fn decode_schema_checked<T: DeserializeOwned>(
    path: &'static str,
    schema_name: &'static str,
    schema_text: &str,
    payload: &[u8],
) -> Result<T, WriterPackageReadError> {
    let instance: Value =
        serde_json::from_slice(payload).map_err(|error| WriterPackageReadError::InvalidJson {
            path,
            message: error.to_string(),
        })?;
    let schema: Value = serde_json::from_str(schema_text).map_err(|error| {
        WriterPackageReadError::InvalidBundledSchema {
            schema: schema_name,
            message: error.to_string(),
        }
    })?;
    jsonschema::meta::validate(&schema).map_err(|error| {
        WriterPackageReadError::InvalidBundledSchema {
            schema: schema_name,
            message: error.to_string(),
        }
    })?;
    let validator = options()
        .with_draft(Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| WriterPackageReadError::InvalidBundledSchema {
            schema: schema_name,
            message: error.to_string(),
        })?;
    validator
        .validate(&instance)
        .map_err(|error| WriterPackageReadError::SchemaViolation {
            path,
            message: error.to_string(),
        })?;
    serde_json::from_value(instance).map_err(|error| WriterPackageReadError::InvalidDecodedRecord {
        path,
        message: error.to_string(),
    })
}

fn validate_manifest_record(record: &ManifestRecord) -> Result<(), WriterPackageReadError> {
    let document_type =
        DocumentType::from_manifest_value(&record.document_type).ok_or_else(|| {
            WriterPackageReadError::InvalidDecodedRecord {
                path: MANIFEST_PART,
                message: "unknown document_type".to_owned(),
            }
        })?;
    let document_role = match record.document_role.as_str() {
        "document" => DocumentRole::Document,
        "template" => DocumentRole::Template,
        _ => {
            return Err(WriterPackageReadError::InvalidDecodedRecord {
                path: MANIFEST_PART,
                message: "unknown document_role".to_owned(),
            });
        }
    };
    let required_parts = record
        .required_parts
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let document_entry_point = record
        .entry_points
        .get("document")
        .map(String::as_str)
        .unwrap_or("");
    let manifest = WriterManifest {
        format_family: &record.format_family,
        format_version: FormatVersion::new(
            record.format_version.major,
            record.format_version.minor,
        ),
        document_type,
        document_role,
        document_id: &record.document_id,
        required_parts: &required_parts,
        document_entry_point,
        created_at: &record.created_at,
        last_saved_at: &record.last_saved_at,
        producer_name: &record.producer.name,
        producer_version: &record.producer.version,
    };
    validate_writer_manifest(&manifest).map_err(WriterPackageReadError::Manifest)
}

fn validate_metadata_record(record: &MetadataRecord) -> Result<(), WriterPackageReadError> {
    let metadata = PackageMetadata {
        schema_version: record.schema_version,
        title: &record.title,
        language: record.language.as_deref(),
    };
    validate_package_metadata(&metadata).map_err(WriterPackageReadError::Metadata)
}

fn validate_relationships_record(
    record: &RelationshipsRecord,
) -> Result<(), WriterPackageReadError> {
    let relationships = record
        .relationships
        .iter()
        .map(|relationship| {
            let kind = match relationship.kind.as_str() {
                "internal" => RelationshipKind::Internal,
                "embedded" => RelationshipKind::Embedded,
                "external" => RelationshipKind::External,
                _ => {
                    return Err(WriterPackageReadError::InvalidDecodedRecord {
                        path: RELATIONSHIPS_PART,
                        message: "unknown relationship kind".to_owned(),
                    });
                }
            };
            Ok(PackageRelationship {
                id: &relationship.id,
                source: &relationship.source,
                target: &relationship.target,
                kind,
                required: relationship.required,
                media_type: relationship.media_type.as_deref(),
            })
        })
        .collect::<Result<Vec<_>, WriterPackageReadError>>()?;
    let decoded = PackageRelationships {
        schema_version: record.schema_version,
        relationships: &relationships,
    };
    validate_package_relationships(&decoded).map_err(WriterPackageReadError::Relationships)
}

fn validate_compatibility_record(
    record: &CompatibilityRecord,
) -> Result<(), WriterPackageReadError> {
    let used_features = record
        .used_features
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let required_capabilities = record
        .required_capabilities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let decoded = PackageCompatibility {
        schema_version: record.schema_version,
        used_features: &used_features,
        required_capabilities: &required_capabilities,
    };
    validate_package_compatibility(&decoded).map_err(WriterPackageReadError::Compatibility)
}

fn validate_document_record(
    record: &WriterDocumentRecord,
    expected_document_id: &str,
) -> Result<(), WriterPackageReadError> {
    let run_storage = record
        .blocks
        .iter()
        .map(|block| {
            block
                .runs
                .iter()
                .map(|run| WriterRun {
                    id: &run.id,
                    text: &run.text,
                    style: run.style.as_deref(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let blocks = record
        .blocks
        .iter()
        .zip(run_storage.iter())
        .map(|(block, runs)| WriterParagraph {
            id: &block.id,
            style: block.style.as_deref(),
            runs,
        })
        .collect::<Vec<_>>();
    let decoded = WriterDocument {
        schema_version: FormatVersion::new(
            record.schema_version.major,
            record.schema_version.minor,
        ),
        document_id: &record.document_id,
        blocks: &blocks,
    };
    validate_writer_document(&decoded, expected_document_id)
        .map_err(WriterPackageReadError::Document)
}

fn validate_integrity_record(
    record: &IntegrityRecord,
    package_entries: &[PackageEntry<'_>],
) -> Result<(), WriterPackageReadError> {
    let entries = record
        .entries
        .iter()
        .map(|entry| {
            let criticality = match entry.criticality.as_deref() {
                Some("critical") => Some(IntegrityCriticality::Critical),
                Some("noncritical") => Some(IntegrityCriticality::Noncritical),
                None => None,
                Some(_) => {
                    return Err(WriterPackageReadError::InvalidDecodedRecord {
                        path: INTEGRITY_PART,
                        message: "unknown integrity criticality".to_owned(),
                    });
                }
            };
            Ok(IntegrityEntry {
                path: &entry.path,
                byte_length: entry.byte_length,
                sha256: &entry.sha256,
                criticality,
                media_type: entry.media_type.as_deref(),
            })
        })
        .collect::<Result<Vec<_>, WriterPackageReadError>>()?;
    let decoded = PackageIntegrity {
        schema_version: record.schema_version,
        algorithm: &record.algorithm,
        entries: &entries,
    };
    validate_package_integrity(&decoded, package_entries).map_err(WriterPackageReadError::Integrity)
}

fn verify_integrity_digests(
    record: &IntegrityRecord,
    payloads: &BTreeMap<String, Vec<u8>>,
) -> Result<(), WriterPackageReadError> {
    for entry in &record.entries {
        let Some(payload) = payloads.get(&entry.path) else {
            return Err(WriterPackageReadError::Integrity(
                IntegrityError::ExtraPath(entry.path.clone()),
            ));
        };
        if sha256_hex(payload) != entry.sha256 {
            return Err(WriterPackageReadError::DigestMismatch {
                path: entry.path.clone(),
            });
        }
    }
    Ok(())
}

fn sha256_hex(payload: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(payload);
    let mut encoded = String::with_capacity(64);
    for byte in digest.iter().copied() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    const FIXTURE: &[u8] =
        include_bytes!("../test-data/v1/fixtures/office-writer-v1-valid-minimal.gcwriter");

    #[test]
    fn canonical_minimal_writer_fixture_validates_end_to_end() {
        let validated = validate_writer_package(
            "office-writer-v1-valid-minimal.gcwriter",
            FIXTURE,
            PackageLimits::default(),
        )
        .unwrap();

        assert_eq!(
            validated.document_id(),
            "a1a11111-1111-4111-8111-111111111111"
        );
        assert_eq!(
            validated.title(),
            "GoreeCloud Office format validation fixture"
        );
        assert_eq!(validated.paragraph_count(), 1);
        assert_eq!(validated.relationship_count(), 0);
        assert_eq!(
            validated.used_features(),
            ["writer.paragraph".to_owned(), "writer.text-run".to_owned()]
        );
    }

    #[test]
    fn writer_extension_is_required() {
        assert!(matches!(
            validate_writer_package("fixture.zip", FIXTURE, PackageLimits::default()),
            Err(WriterPackageReadError::WrongExtension)
        ));
    }

    #[test]
    fn valid_json_tampering_is_detected_by_sha256() {
        let tampered = rewrite_fixture(METADATA_PART, |payload| {
            let mut changed = payload.to_vec();
            let index = changed
                .windows(b"GoreeCloud".len())
                .position(|window| window == b"GoreeCloud")
                .unwrap();
            changed[index] = b'B';
            changed
        });

        assert!(matches!(
            validate_writer_package(
                "tampered.gcwriter",
                &tampered,
                PackageLimits::default()
            ),
            Err(WriterPackageReadError::DigestMismatch { path }) if path == METADATA_PART
        ));
    }

    #[test]
    fn malformed_required_json_is_rejected() {
        let malformed = rewrite_fixture(METADATA_PART, |_| b"{".to_vec());
        assert!(matches!(
            validate_writer_package("malformed.gcwriter", &malformed, PackageLimits::default()),
            Err(WriterPackageReadError::InvalidJson {
                path: METADATA_PART,
                ..
            })
        ));
    }

    #[test]
    fn sha256_helper_matches_governed_fixture_digest() {
        assert_eq!(
            sha256_hex(DocumentType::Writer.media_type().as_bytes()),
            "a13045d5225252f157597cf84615d7d38fc0e267c2e2379d86137859f27ebb19"
        );
    }

    fn rewrite_fixture(target: &str, transform: impl FnOnce(&[u8]) -> Vec<u8>) -> Vec<u8> {
        let mut source = ZipArchive::new(Cursor::new(FIXTURE)).unwrap();
        let output = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(output);
        let mut transform = Some(transform);

        for index in 0..source.len() {
            let mut file = source.by_index(index).unwrap();
            let name = file.name().to_owned();
            let method = file.compression();
            let mut payload = Vec::new();
            file.read_to_end(&mut payload).unwrap();
            if name == target {
                payload = transform.take().unwrap()(&payload);
            }
            let options = SimpleFileOptions::default().compression_method(method);
            writer.start_file(name, options).unwrap();
            writer.write_all(&payload).unwrap();
        }

        writer.finish().unwrap().into_inner()
    }
}
