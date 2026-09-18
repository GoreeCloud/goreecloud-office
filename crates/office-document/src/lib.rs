use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

pub const WRITER_SCHEMA_MAJOR: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterDocument {
    pub schema_version: SchemaVersion,
    pub document_id: String,
    pub blocks: Vec<WriterBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterBlock {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: WriterBlockType,
    pub runs: Vec<TextRun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriterBlockType {
    Paragraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRun {
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnsupportedSchemaMajor(u32),
    InvalidUuid { field: &'static str, value: String },
    DuplicateId(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaMajor(major) => {
                write!(f, "unsupported Writer schema major version {major}")
            }
            Self::InvalidUuid { field, value } => {
                write!(f, "{field} is not a canonical lowercase UUID: {value}")
            }
            Self::DuplicateId(value) => write!(f, "duplicate document object id: {value}"),
        }
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_canonical_uuid(field: &'static str, value: &str) -> Result<(), ValidationError> {
    let parsed = Uuid::try_parse(value).map_err(|_| ValidationError::InvalidUuid {
        field,
        value: value.to_owned(),
    })?;

    if parsed.hyphenated().to_string() != value {
        return Err(ValidationError::InvalidUuid {
            field,
            value: value.to_owned(),
        });
    }

    Ok(())
}

impl WriterDocument {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version.major != WRITER_SCHEMA_MAJOR {
            return Err(ValidationError::UnsupportedSchemaMajor(
                self.schema_version.major,
            ));
        }

        validate_canonical_uuid("document_id", &self.document_id)?;

        let mut ids = HashSet::new();
        ids.insert(self.document_id.as_str());

        for block in &self.blocks {
            validate_canonical_uuid("block.id", &block.id)?;
            if !ids.insert(block.id.as_str()) {
                return Err(ValidationError::DuplicateId(block.id.clone()));
            }

            for run in &block.runs {
                validate_canonical_uuid("run.id", &run.id)?;
                if !ids.insert(run.id.as_str()) {
                    return Err(ValidationError::DuplicateId(run.id.clone()));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_document() -> WriterDocument {
        WriterDocument {
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
        }
    }

    #[test]
    fn validates_minimal_document() {
        sample_document().validate().unwrap();
    }

    #[test]
    fn rejects_noncanonical_uuid() {
        let mut document = sample_document();
        document.document_id = "11111111111141118111111111111111".into();
        assert!(matches!(
            document.validate(),
            Err(ValidationError::InvalidUuid { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_ids() {
        let mut document = sample_document();
        document.blocks[0].runs[0].id = document.blocks[0].id.clone();
        assert!(matches!(
            document.validate(),
            Err(ValidationError::DuplicateId(_))
        ));
    }

    #[test]
    fn rejects_future_major_version() {
        let mut document = sample_document();
        document.schema_version.major = 2;
        assert_eq!(
            document.validate(),
            Err(ValidationError::UnsupportedSchemaMajor(2))
        );
    }
}
