use goreecloud_office_document::{WriterDocument, validate_canonical_uuid};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const WRITER_EXTENSION: &str = ".gcwriter";
pub const WRITER_MEDIA_TYPE: &str = "application/vnd.goreecloud.office.writer+zip";
pub const FORMAT_FAMILY: &str = "goreecloud.office";
pub const FORMAT_MAJOR: u32 = 1;

const MAX_ENTRIES: usize = 2048;
const MAX_ENTRY_SIZE: u64 = 64 * 1024 * 1024;
const MAX_PACKAGE_SIZE: u64 = 128 * 1024 * 1024;
const MAX_COMPRESSION_RATIO: u64 = 1000;

const MANIFEST_PATH: &str = "manifest.json";
const METADATA_PATH: &str = "metadata.json";
const RELATIONSHIPS_PATH: &str = "relationships.json";
const COMPATIBILITY_PATH: &str = "compatibility.json";
const INTEGRITY_PATH: &str = "integrity.json";
const DOCUMENT_PATH: &str = "content/document.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatVersion {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentType {
    Writer,
    Spreadsheet,
    Presentation,
    Form,
    Database,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentRole {
    Document,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub format_family: String,
    pub format_version: FormatVersion,
    pub document_type: DocumentType,
    pub document_role: DocumentRole,
    pub document_id: String,
    pub required_parts: Vec<String>,
    pub entry_points: BTreeMap<String, String>,
    pub created_at: String,
    pub last_saved_at: String,
    pub producer: Producer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub schema_version: u32,
    pub title: String,
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(default)]
    pub contributors: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRelationships {
    pub schema_version: u32,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: RelationshipKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipKind {
    Internal,
    Embedded,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageCompatibility {
    pub schema_version: u32,
    pub used_features: Vec<String>,
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub conversion_notes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageIntegrity {
    pub schema_version: u32,
    pub algorithm: String,
    pub entries: Vec<IntegrityEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityEntry {
    pub path: String,
    pub byte_length: u64,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriterPackageInput {
    pub document: WriterDocument,
    pub metadata: PackageMetadata,
    pub created_at: String,
    pub last_saved_at: String,
    pub producer_version: String,
}

#[derive(Debug)]
pub enum PackageError {
    Invalid(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Zip(zip::result::ZipError),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Json(error) => write!(f, "JSON error: {error}"),
            Self::Zip(error) => write!(f, "ZIP error: {error}"),
        }
    }
}

impl std::error::Error for PackageError {}

impl From<std::io::Error> for PackageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PackageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<zip::result::ZipError> for PackageError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}

pub fn build_writer_package(input: &WriterPackageInput) -> Result<Vec<u8>, PackageError> {
    input
        .document
        .validate()
        .map_err(|error| PackageError::Invalid(error.to_string()))?;

    if input.metadata.schema_version != 1 {
        return Err(PackageError::Invalid(
            "metadata schema_version must be 1".into(),
        ));
    }

    let required_parts = vec![
        MANIFEST_PATH.into(),
        METADATA_PATH.into(),
        RELATIONSHIPS_PATH.into(),
        COMPATIBILITY_PATH.into(),
        DOCUMENT_PATH.into(),
        INTEGRITY_PATH.into(),
    ];

    let mut entry_points = BTreeMap::new();
    entry_points.insert("document".into(), DOCUMENT_PATH.into());

    let manifest = PackageManifest {
        format_family: FORMAT_FAMILY.into(),
        format_version: FormatVersion { major: 1, minor: 0 },
        document_type: DocumentType::Writer,
        document_role: DocumentRole::Document,
        document_id: input.document.document_id.clone(),
        required_parts,
        entry_points,
        created_at: input.created_at.clone(),
        last_saved_at: input.last_saved_at.clone(),
        producer: Producer {
            name: "GoreeCloud Office".into(),
            version: input.producer_version.clone(),
        },
    };

    let relationships = PackageRelationships {
        schema_version: 1,
        relationships: Vec::new(),
    };
    let compatibility = PackageCompatibility {
        schema_version: 1,
        used_features: vec!["writer.paragraphs".into()],
        required_capabilities: vec!["writer.paragraphs".into()],
        conversion_notes: Vec::new(),
    };

    let mut parts = BTreeMap::new();
    parts.insert(
        MANIFEST_PATH.to_string(),
        serde_json::to_vec_pretty(&manifest)?,
    );
    parts.insert(
        METADATA_PATH.to_string(),
        serde_json::to_vec_pretty(&input.metadata)?,
    );
    parts.insert(
        RELATIONSHIPS_PATH.to_string(),
        serde_json::to_vec_pretty(&relationships)?,
    );
    parts.insert(
        COMPATIBILITY_PATH.to_string(),
        serde_json::to_vec_pretty(&compatibility)?,
    );
    parts.insert(
        DOCUMENT_PATH.to_string(),
        serde_json::to_vec_pretty(&input.document)?,
    );

    let integrity = PackageIntegrity {
        schema_version: 1,
        algorithm: "sha256".into(),
        entries: parts
            .iter()
            .map(|(path, bytes)| IntegrityEntry {
                path: path.clone(),
                byte_length: bytes.len() as u64,
                sha256: sha256_hex(bytes),
                media_type: Some("application/json".into()),
                criticality: Some("critical".into()),
            })
            .collect(),
    };

    parts.insert(
        INTEGRITY_PATH.to_string(),
        serde_json::to_vec_pretty(&integrity)?,
    );

    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);

    writer.start_file(
        "mimetype",
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
    )?;
    writer.write_all(WRITER_MEDIA_TYPE.as_bytes())?;

    for (path, bytes) in parts {
        writer.start_file(
            path,
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )?;
        writer.write_all(&bytes)?;
    }

    Ok(writer.finish()?.into_inner())
}

pub fn validate_writer_package(file_name: &str, bytes: &[u8]) -> Result<(), PackageError> {
    if !file_name.ends_with(WRITER_EXTENSION) {
        return Err(PackageError::Invalid(format!(
            "Writer packages must use the {WRITER_EXTENSION} extension"
        )));
    }

    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    if archive.is_empty() || archive.len() > MAX_ENTRIES {
        return Err(PackageError::Invalid("invalid package entry count".into()));
    }

    let mut entries = BTreeMap::<String, Vec<u8>>::new();
    let mut seen = HashSet::new();
    let mut total_size = 0_u64;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let name = file.name().to_owned();

        if !is_safe_package_path(&name) {
            return Err(PackageError::Invalid(format!(
                "unsafe package path: {name}"
            )));
        }
        if !seen.insert(name.clone()) {
            return Err(PackageError::Invalid(format!(
                "duplicate package entry: {name}"
            )));
        }
        if file.is_symlink() || file.is_dir() || !file.is_file() {
            return Err(PackageError::Invalid(format!(
                "unsupported ZIP entry type: {name}"
            )));
        }
        if file.encrypted() {
            return Err(PackageError::Invalid(format!(
                "encrypted ZIP entries are not supported in v1: {name}"
            )));
        }
        if !matches!(
            file.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(PackageError::Invalid(format!(
                "unsupported ZIP compression for {name}"
            )));
        }
        if file.size() > MAX_ENTRY_SIZE {
            return Err(PackageError::Invalid(format!(
                "package entry exceeds size limit: {name}"
            )));
        }

        total_size = total_size
            .checked_add(file.size())
            .ok_or_else(|| PackageError::Invalid("package size overflow".into()))?;
        if total_size > MAX_PACKAGE_SIZE {
            return Err(PackageError::Invalid(
                "package exceeds uncompressed size limit".into(),
            ));
        }

        let compressed = file.compressed_size();
        if compressed > 0 && file.size() / compressed > MAX_COMPRESSION_RATIO {
            return Err(PackageError::Invalid(format!(
                "package entry exceeds compression-ratio limit: {name}"
            )));
        }

        if index == 0 {
            if name != "mimetype" {
                return Err(PackageError::Invalid(
                    "mimetype must be the first ZIP entry".into(),
                ));
            }
            if file.compression() != CompressionMethod::Stored {
                return Err(PackageError::Invalid(
                    "mimetype must be stored without compression".into(),
                ));
            }
        }

        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data)?;
        entries.insert(name, data);
    }

    let mimetype = entries
        .get("mimetype")
        .ok_or_else(|| PackageError::Invalid("missing mimetype".into()))?;
    if mimetype.as_slice() != WRITER_MEDIA_TYPE.as_bytes() {
        return Err(PackageError::Invalid("incorrect Writer mimetype".into()));
    }

    for required in [
        MANIFEST_PATH,
        METADATA_PATH,
        RELATIONSHIPS_PATH,
        COMPATIBILITY_PATH,
        DOCUMENT_PATH,
        INTEGRITY_PATH,
    ] {
        if !entries.contains_key(required) {
            return Err(PackageError::Invalid(format!(
                "missing required package part: {required}"
            )));
        }
    }

    let manifest: PackageManifest = serde_json::from_slice(&entries[MANIFEST_PATH])?;
    if manifest.format_family != FORMAT_FAMILY || manifest.format_version.major != FORMAT_MAJOR {
        return Err(PackageError::Invalid(
            "unsupported GoreeCloud Office package version".into(),
        ));
    }
    if manifest.document_type != DocumentType::Writer {
        return Err(PackageError::Invalid(
            "package manifest is not a Writer document".into(),
        ));
    }
    validate_canonical_uuid("manifest.document_id", &manifest.document_id)
        .map_err(|error| PackageError::Invalid(error.to_string()))?;

    for required in &manifest.required_parts {
        if !entries.contains_key(required) {
            return Err(PackageError::Invalid(format!(
                "manifest required part is missing: {required}"
            )));
        }
    }
    if manifest.entry_points.get("document").map(String::as_str) != Some(DOCUMENT_PATH) {
        return Err(PackageError::Invalid(
            "Writer document entry point must be content/document.json".into(),
        ));
    }

    let metadata: PackageMetadata = serde_json::from_slice(&entries[METADATA_PATH])?;
    if metadata.schema_version != 1 {
        return Err(PackageError::Invalid(
            "unsupported metadata schema version".into(),
        ));
    }
    let relationships: PackageRelationships = serde_json::from_slice(&entries[RELATIONSHIPS_PATH])?;
    if relationships.schema_version != 1 {
        return Err(PackageError::Invalid(
            "unsupported relationships schema version".into(),
        ));
    }
    for relationship in &relationships.relationships {
        validate_canonical_uuid("relationship.id", &relationship.id)
            .map_err(|error| PackageError::Invalid(error.to_string()))?;
    }

    let compatibility: PackageCompatibility = serde_json::from_slice(&entries[COMPATIBILITY_PATH])?;
    if compatibility.schema_version != 1 {
        return Err(PackageError::Invalid(
            "unsupported compatibility schema version".into(),
        ));
    }

    let document: WriterDocument = serde_json::from_slice(&entries[DOCUMENT_PATH])?;
    document
        .validate()
        .map_err(|error| PackageError::Invalid(error.to_string()))?;
    if document.document_id != manifest.document_id {
        return Err(PackageError::Invalid(
            "manifest and Writer document ids do not match".into(),
        ));
    }

    let integrity: PackageIntegrity = serde_json::from_slice(&entries[INTEGRITY_PATH])?;
    validate_integrity(&entries, &integrity)?;

    Ok(())
}

fn validate_integrity(
    entries: &BTreeMap<String, Vec<u8>>,
    integrity: &PackageIntegrity,
) -> Result<(), PackageError> {
    if integrity.schema_version != 1 || integrity.algorithm != "sha256" {
        return Err(PackageError::Invalid(
            "unsupported package integrity record".into(),
        ));
    }

    let expected: HashSet<&str> = entries
        .keys()
        .map(String::as_str)
        .filter(|path| *path != "mimetype" && *path != INTEGRITY_PATH)
        .collect();

    let mut covered = HashSet::new();
    for record in &integrity.entries {
        if record.path == "mimetype" || record.path == INTEGRITY_PATH {
            return Err(PackageError::Invalid(format!(
                "invalid integrity target: {}",
                record.path
            )));
        }
        if !covered.insert(record.path.as_str()) {
            return Err(PackageError::Invalid(format!(
                "duplicate integrity record: {}",
                record.path
            )));
        }

        let bytes = entries.get(&record.path).ok_or_else(|| {
            PackageError::Invalid(format!(
                "integrity record references missing part: {}",
                record.path
            ))
        })?;

        if record.byte_length != bytes.len() as u64 || record.sha256 != sha256_hex(bytes) {
            return Err(PackageError::Invalid(format!(
                "integrity validation failed for {}",
                record.path
            )));
        }
    }

    if covered != expected {
        return Err(PackageError::Invalid(
            "integrity coverage does not match package content".into(),
        ));
    }

    Ok(())
}

fn is_safe_package_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return false;
    }

    Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use goreecloud_office_document::{SchemaVersion, TextRun, WriterBlock, WriterBlockType};

    fn sample_input() -> WriterPackageInput {
        WriterPackageInput {
            document: WriterDocument {
                schema_version: SchemaVersion { major: 1, minor: 0 },
                document_id: "11111111-1111-4111-8111-111111111111".into(),
                blocks: vec![WriterBlock {
                    id: "22222222-2222-4222-8222-222222222222".into(),
                    block_type: WriterBlockType::Paragraph,
                    style: None,
                    runs: vec![TextRun {
                        id: "33333333-3333-4333-8333-333333333333".into(),
                        text: "Hello, GoreeCloud Office.".into(),
                        style: None,
                    }],
                }],
            },
            metadata: PackageMetadata {
                schema_version: 1,
                title: "Minimal Writer fixture".into(),
                language: Some("en".into()),
                subject: None,
                creator: Some("GoreeCloud".into()),
                contributors: Vec::new(),
                keywords: vec!["fixture".into()],
            },
            created_at: "2026-09-17T00:00:00Z".into(),
            last_saved_at: "2026-09-17T00:00:00Z".into(),
            producer_version: "0.1.0".into(),
        }
    }

    #[test]
    fn builds_and_validates_minimal_writer_package() {
        let bytes = build_writer_package(&sample_input()).unwrap();
        validate_writer_package("fixture.gcwriter", &bytes).unwrap();
    }

    #[test]
    fn rejects_wrong_extension() {
        let bytes = build_writer_package(&sample_input()).unwrap();
        assert!(validate_writer_package("fixture.zip", &bytes).is_err());
    }

    #[test]
    fn rejects_tampered_package_bytes() {
        let mut bytes = build_writer_package(&sample_input()).unwrap();
        let needle = b"Hello, GoreeCloud Office.";
        let position = bytes
            .windows(needle.len())
            .position(|window| window == needle);

        if let Some(position) = position {
            bytes[position] = b'J';
            assert!(validate_writer_package("fixture.gcwriter", &bytes).is_err());
        } else {
            // Deflate may hide the source string. A truncated archive is still an
            // intentional corruption case and must fail closed.
            bytes.truncate(bytes.len().saturating_sub(8));
            assert!(validate_writer_package("fixture.gcwriter", &bytes).is_err());
        }
    }

    #[test]
    fn safe_path_policy_rejects_traversal_and_absolute_paths() {
        assert!(!is_safe_package_path("../escape.json"));
        assert!(!is_safe_package_path("/absolute.json"));
        assert!(!is_safe_package_path("content\\document.json"));
        assert!(is_safe_package_path("content/document.json"));
    }
}
