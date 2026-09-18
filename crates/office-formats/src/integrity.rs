//! Dependency-free semantic validation for Office v1 integrity records.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use crate::{INTEGRITY_PART, PackageEntry, validate_entry_path};

/// Criticality declared for one protected package part.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityCriticality {
    /// Damage to the part prevents normal document interpretation.
    Critical,
    /// Damage may allow partial recovery of the remaining package.
    Noncritical,
}

/// Borrowed integrity entry after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegrityEntry<'a> {
    /// Protected package path.
    pub path: &'a str,
    /// Declared uncompressed byte length.
    pub byte_length: u64,
    /// Lowercase hexadecimal SHA-256 digest.
    pub sha256: &'a str,
    /// Optional recovery criticality.
    pub criticality: Option<IntegrityCriticality>,
    /// Optional declared media type.
    pub media_type: Option<&'a str>,
}

/// Borrowed integrity record after JSON decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageIntegrity<'a> {
    /// Integrity schema version.
    pub schema_version: u16,
    /// Digest algorithm identifier.
    pub algorithm: &'a str,
    /// Protected package entries.
    pub entries: &'a [IntegrityEntry<'a>],
}

/// Integrity-record semantic validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntegrityError {
    /// Integrity schema version is unsupported.
    UnsupportedSchemaVersion(u16),
    /// Digest algorithm is not the v1 sha256 algorithm.
    UnsupportedAlgorithm,
    /// An integrity path is unsafe.
    UnsafePath {
        /// Zero-based integrity-entry index.
        index: usize,
    },
    /// The integrity record attempts to hash itself.
    SelfReference,
    /// An integrity path is repeated.
    DuplicatePath(String),
    /// A digest is not exactly 64 lowercase hexadecimal characters.
    InvalidDigest {
        /// Zero-based integrity-entry index.
        index: usize,
    },
    /// A package file is not covered by the integrity record.
    MissingPath(String),
    /// The integrity record refers to a path that is not in the package.
    ExtraPath(String),
    /// Declared byte length differs from archive entry metadata.
    ByteLengthMismatch {
        /// Package path with the mismatch.
        path: String,
        /// Declared integrity-record length.
        declared: u64,
        /// Archive entry uncompressed length.
        actual: u64,
    },
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported integrity schema version: {version}")
            }
            Self::UnsupportedAlgorithm => f.write_str("integrity algorithm must be sha256"),
            Self::UnsafePath { index } => write!(f, "integrity path at index {index} is unsafe"),
            Self::SelfReference => f.write_str("integrity.json must not hash itself"),
            Self::DuplicatePath(path) => write!(f, "integrity path is duplicated: {path}"),
            Self::InvalidDigest { index } => {
                write!(f, "integrity digest at index {index} is not lowercase SHA-256 hex")
            }
            Self::MissingPath(path) => write!(f, "integrity record is missing package path: {path}"),
            Self::ExtraPath(path) => write!(f, "integrity record contains extra path: {path}"),
            Self::ByteLengthMismatch {
                path,
                declared,
                actual,
            } => write!(
                f,
                "integrity byte length for {path} is {declared}; archive metadata reports {actual}"
            ),
        }
    }
}

impl Error for IntegrityError {}

/// Returns whether a digest uses the lowercase 64-hex representation of SHA-256.
#[must_use]
pub fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

/// Validates decoded v1 integrity-record semantics against package entry metadata.
///
/// This validates record structure, exact package-file coverage, and declared
/// byte lengths. It deliberately does not compute SHA-256 digests.
///
/// # Errors
///
/// Returns `IntegrityError` when the record violates the governed v1 integrity
/// contract.
pub fn validate_package_integrity(
    record: &PackageIntegrity<'_>,
    package_entries: &[PackageEntry<'_>],
) -> Result<(), IntegrityError> {
    if record.schema_version != 1 {
        return Err(IntegrityError::UnsupportedSchemaVersion(
            record.schema_version,
        ));
    }
    if record.algorithm != "sha256" {
        return Err(IntegrityError::UnsupportedAlgorithm);
    }

    let expected = package_entries
        .iter()
        .filter(|entry| entry.path() != INTEGRITY_PART)
        .map(|entry| (entry.path(), entry.uncompressed_bytes()))
        .collect::<BTreeMap<_, _>>();

    let mut covered = BTreeSet::new();
    for (index, entry) in record.entries.iter().enumerate() {
        if validate_entry_path(entry.path).is_err() {
            return Err(IntegrityError::UnsafePath { index });
        }
        if entry.path == INTEGRITY_PART {
            return Err(IntegrityError::SelfReference);
        }
        if !covered.insert(entry.path) {
            return Err(IntegrityError::DuplicatePath(entry.path.to_owned()));
        }
        if !is_sha256_hex(entry.sha256) {
            return Err(IntegrityError::InvalidDigest { index });
        }
        let Some(actual) = expected.get(entry.path).copied() else {
            return Err(IntegrityError::ExtraPath(entry.path.to_owned()));
        };
        if entry.byte_length != actual {
            return Err(IntegrityError::ByteLengthMismatch {
                path: entry.path.to_owned(),
                declared: entry.byte_length,
                actual,
            });
        }
    }

    for path in expected.keys().copied() {
        if !covered.contains(path) {
            return Err(IntegrityError::MissingPath(path.to_owned()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompressionMethod, MANIFEST_PART, MIMETYPE_PART};

    fn package_entries() -> [PackageEntry<'static>; 3] {
        [
            PackageEntry::new(MIMETYPE_PART, CompressionMethod::Store, 44, 44, false),
            PackageEntry::new(MANIFEST_PART, CompressionMethod::Deflate, 551, 300, false),
            PackageEntry::new(INTEGRITY_PART, CompressionMethod::Deflate, 778, 400, false),
        ]
    }

    fn valid_entries() -> [IntegrityEntry<'static>; 2] {
        [
            IntegrityEntry {
                path: MIMETYPE_PART,
                byte_length: 44,
                sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                criticality: Some(IntegrityCriticality::Critical),
                media_type: Some("text/plain"),
            },
            IntegrityEntry {
                path: MANIFEST_PART,
                byte_length: 551,
                sha256: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
                criticality: Some(IntegrityCriticality::Critical),
                media_type: Some("application/json"),
            },
        ]
    }

    #[test]
    fn valid_integrity_record_shape_is_accepted_without_digest_computation() {
        let entries = valid_entries();
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(validate_package_integrity(&record, &package_entries()), Ok(()));
    }

    #[test]
    fn schema_algorithm_and_digest_syntax_are_enforced() {
        let entries = valid_entries();
        let mut record = PackageIntegrity {
            schema_version: 2,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::UnsupportedSchemaVersion(2))
        );

        record.schema_version = 1;
        record.algorithm = "sha512";
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::UnsupportedAlgorithm)
        );

        let mut invalid = entries;
        invalid[0].sha256 =
            "ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        record.algorithm = "sha256";
        record.entries = &invalid;
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::InvalidDigest { index: 0 })
        );
    }

    #[test]
    fn unsafe_self_and_duplicate_paths_are_rejected() {
        let mut entries = valid_entries();
        entries[0].path = "../mimetype";
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::UnsafePath { index: 0 })
        );

        entries = valid_entries();
        entries[0].path = INTEGRITY_PART;
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::SelfReference)
        );

        entries = valid_entries();
        entries[1].path = MIMETYPE_PART;
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::DuplicatePath(MIMETYPE_PART.to_owned()))
        );
    }

    #[test]
    fn integrity_record_requires_exact_package_coverage() {
        let entries = [valid_entries()[0]];
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::MissingPath(MANIFEST_PART.to_owned()))
        );

        let mut extra = valid_entries().to_vec();
        extra.push(IntegrityEntry {
            path: "content/extra.json",
            byte_length: 1,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            criticality: None,
            media_type: None,
        });
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &extra,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::ExtraPath("content/extra.json".to_owned()))
        );
    }

    #[test]
    fn declared_byte_lengths_must_match_archive_metadata() {
        let mut entries = valid_entries();
        entries[1].byte_length = 550;
        let record = PackageIntegrity {
            schema_version: 1,
            algorithm: "sha256",
            entries: &entries,
        };
        assert_eq!(
            validate_package_integrity(&record, &package_entries()),
            Err(IntegrityError::ByteLengthMismatch {
                path: MANIFEST_PART.to_owned(),
                declared: 550,
                actual: 551,
            })
        );
    }

    #[test]
    fn sha256_hex_syntax_is_lowercase_and_exact_length() {
        assert!(is_sha256_hex(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        assert!(!is_sha256_hex(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde"
        ));
        assert!(!is_sha256_hex(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeF"
        ));
    }
}
