//! Canonical document-domain foundation for the GoreeCloud Office Engine.
//!
//! This crate intentionally has no external dependencies. It establishes the
//! smallest authoritative document model needed to validate command semantics
//! before persistence, layout, rendering, import/export, or collaboration are
//! introduced.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Stable identity for a document.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentId(u64);

impl DocumentId {
    /// Returns the persisted integer representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable identity for a document block.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockId(u64);

impl BlockId {
    /// Returns the persisted integer representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable identity for a paragraph style.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StyleId(u64);

impl StyleId {
    /// Returns the persisted integer representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Portable paragraph-style properties owned by the Office document model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParagraphStyle {
    id: StyleId,
    name: String,
}

impl ParagraphStyle {
    /// Creates a paragraph style.
    #[must_use]
    pub fn new(id: StyleId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Returns the style identity.
    #[must_use]
    pub const fn id(&self) -> StyleId {
        self.id
    }

    /// Returns the user-facing style name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A paragraph block in logical Unicode order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paragraph {
    id: BlockId,
    style: StyleId,
    text: String,
}

impl Paragraph {
    /// Returns the stable block identity.
    #[must_use]
    pub const fn id(&self) -> BlockId {
        self.id
    }

    /// Returns the assigned paragraph-style identity.
    #[must_use]
    pub const fn style(&self) -> StyleId {
        self.style
    }

    /// Returns paragraph text in logical Unicode order.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// A block in the canonical Office document model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Block {
    /// A text paragraph.
    Paragraph(Paragraph),
}

impl Block {
    /// Returns the stable block identity.
    #[must_use]
    pub const fn id(&self) -> BlockId {
        match self {
            Self::Paragraph(paragraph) => paragraph.id,
        }
    }

    /// Returns this block as a paragraph.
    #[must_use]
    pub const fn as_paragraph(&self) -> Option<&Paragraph> {
        match self {
            Self::Paragraph(paragraph) => Some(paragraph),
        }
    }
}

/// Errors raised by canonical document operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentError {
    /// The requested block does not exist.
    BlockNotFound(BlockId),
    /// The requested style does not exist.
    StyleNotFound(StyleId),
    /// A byte index does not fall on a UTF-8 character boundary.
    InvalidUtf8Boundary { block: BlockId, index: usize },
    /// A text range is invalid for the requested block.
    InvalidRange {
        block: BlockId,
        start: usize,
        end: usize,
        len: usize,
    },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockNotFound(id) => write!(f, "block {} was not found", id.get()),
            Self::StyleNotFound(id) => write!(f, "style {} was not found", id.get()),
            Self::InvalidUtf8Boundary { block, index } => write!(
                f,
                "byte index {index} is not a UTF-8 boundary in block {}",
                block.get()
            ),
            Self::InvalidRange {
                block,
                start,
                end,
                len,
            } => write!(
                f,
                "range {start}..{end} is invalid for block {} with byte length {len}",
                block.get()
            ),
        }
    }
}

impl Error for DocumentError {}

/// The canonical in-memory document model for the first Writer foundation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    id: DocumentId,
    revision: u64,
    next_object_id: u64,
    default_paragraph_style: StyleId,
    blocks: Vec<Block>,
    paragraph_styles: BTreeMap<StyleId, ParagraphStyle>,
}

impl Document {
    /// Creates a new Writer-oriented document with one empty paragraph.
    #[must_use]
    pub fn new(document_id: u64) -> Self {
        let default_style = StyleId(1);
        let first_block = BlockId(2);
        let mut paragraph_styles = BTreeMap::new();
        paragraph_styles.insert(default_style, ParagraphStyle::new(default_style, "Normal"));

        Self {
            id: DocumentId(document_id),
            revision: 0,
            next_object_id: 3,
            default_paragraph_style: default_style,
            blocks: vec![Block::Paragraph(Paragraph {
                id: first_block,
                style: default_style,
                text: String::new(),
            })],
            paragraph_styles,
        }
    }

    /// Returns the document identity.
    #[must_use]
    pub const fn id(&self) -> DocumentId {
        self.id
    }

    /// Returns the monotonic in-memory mutation revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns all top-level blocks in logical document order.
    #[must_use]
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Returns the default paragraph-style identity.
    #[must_use]
    pub const fn default_paragraph_style(&self) -> StyleId {
        self.default_paragraph_style
    }

    /// Returns a paragraph style by identity.
    #[must_use]
    pub fn paragraph_style(&self, id: StyleId) -> Option<&ParagraphStyle> {
        self.paragraph_styles.get(&id)
    }

    /// Returns a paragraph by stable block identity.
    #[must_use]
    pub fn paragraph(&self, id: BlockId) -> Option<&Paragraph> {
        self.blocks.iter().find_map(|block| match block {
            Block::Paragraph(paragraph) if paragraph.id == id => Some(paragraph),
            Block::Paragraph(_) => None,
        })
    }

    /// Returns the first paragraph identity.
    #[must_use]
    pub fn first_paragraph_id(&self) -> BlockId {
        self.blocks
            .first()
            .map(Block::id)
            .expect("new Office documents always contain a paragraph")
    }

    /// Creates a named paragraph style and returns its stable identity.
    pub fn create_paragraph_style(&mut self, name: impl Into<String>) -> StyleId {
        let id = StyleId(self.allocate_object_id());
        self.paragraph_styles
            .insert(id, ParagraphStyle::new(id, name));
        self.bump_revision();
        id
    }

    /// Appends an empty paragraph and returns its stable identity.
    pub fn append_paragraph(&mut self, style: StyleId) -> Result<BlockId, DocumentError> {
        self.ensure_style(style)?;
        let id = BlockId(self.allocate_object_id());
        self.blocks.push(Block::Paragraph(Paragraph {
            id,
            style,
            text: String::new(),
        }));
        self.bump_revision();
        Ok(id)
    }

    /// Inserts UTF-8 text at a validated byte boundary.
    pub fn insert_text(
        &mut self,
        block: BlockId,
        at: usize,
        text: &str,
    ) -> Result<(), DocumentError> {
        let paragraph = self.paragraph_mut(block)?;
        if at > paragraph.text.len() {
            return Err(DocumentError::InvalidRange {
                block,
                start: at,
                end: at,
                len: paragraph.text.len(),
            });
        }
        if !paragraph.text.is_char_boundary(at) {
            return Err(DocumentError::InvalidUtf8Boundary { block, index: at });
        }
        paragraph.text.insert_str(at, text);
        self.bump_revision();
        Ok(())
    }

    /// Removes a validated byte range and returns the deleted text.
    pub fn remove_text(
        &mut self,
        block: BlockId,
        start: usize,
        end: usize,
    ) -> Result<String, DocumentError> {
        let paragraph = self.paragraph_mut(block)?;
        let len = paragraph.text.len();
        if start > end || end > len {
            return Err(DocumentError::InvalidRange {
                block,
                start,
                end,
                len,
            });
        }
        if !paragraph.text.is_char_boundary(start) {
            return Err(DocumentError::InvalidUtf8Boundary {
                block,
                index: start,
            });
        }
        if !paragraph.text.is_char_boundary(end) {
            return Err(DocumentError::InvalidUtf8Boundary { block, index: end });
        }

        let removed = paragraph.text[start..end].to_owned();
        paragraph.text.replace_range(start..end, "");
        self.bump_revision();
        Ok(removed)
    }

    /// Assigns a paragraph style and returns the previous style.
    pub fn set_paragraph_style(
        &mut self,
        block: BlockId,
        style: StyleId,
    ) -> Result<StyleId, DocumentError> {
        self.ensure_style(style)?;
        let paragraph = self.paragraph_mut(block)?;
        let previous = paragraph.style;
        paragraph.style = style;
        self.bump_revision();
        Ok(previous)
    }

    fn ensure_style(&self, style: StyleId) -> Result<(), DocumentError> {
        if self.paragraph_styles.contains_key(&style) {
            Ok(())
        } else {
            Err(DocumentError::StyleNotFound(style))
        }
    }

    fn paragraph_mut(&mut self, id: BlockId) -> Result<&mut Paragraph, DocumentError> {
        self.blocks
            .iter_mut()
            .find_map(|block| match block {
                Block::Paragraph(paragraph) if paragraph.id == id => Some(paragraph),
                Block::Paragraph(_) => None,
            })
            .ok_or(DocumentError::BlockNotFound(id))
    }

    fn allocate_object_id(&mut self) -> u64 {
        let id = self.next_object_id;
        self.next_object_id = self
            .next_object_id
            .checked_add(1)
            .expect("Office document object identity space exhausted");
        id
    }

    fn bump_revision(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("Office document revision space exhausted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_document_has_stable_identity_and_default_paragraph() {
        let document = Document::new(42);
        let paragraph = document.first_paragraph_id();

        assert_eq!(document.id().get(), 42);
        assert_eq!(document.revision(), 0);
        assert_eq!(paragraph.get(), 2);
        assert_eq!(document.paragraph(paragraph).unwrap().text(), "");
        assert_eq!(
            document
                .paragraph_style(document.default_paragraph_style())
                .unwrap()
                .name(),
            "Normal"
        );
    }

    #[test]
    fn successful_mutations_advance_revision_without_changing_block_identity() {
        let mut document = Document::new(9);
        let paragraph = document.first_paragraph_id();

        document.insert_text(paragraph, 0, "Hello").unwrap();
        document.insert_text(paragraph, 5, " world").unwrap();

        assert_eq!(document.revision(), 2);
        assert_eq!(document.first_paragraph_id(), paragraph);
        assert_eq!(document.paragraph(paragraph).unwrap().text(), "Hello world");
    }

    #[test]
    fn edits_require_utf8_boundaries() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        document.insert_text(paragraph, 0, "é").unwrap();
        let revision = document.revision();

        let error = document.insert_text(paragraph, 1, "x").unwrap_err();

        assert_eq!(
            error,
            DocumentError::InvalidUtf8Boundary {
                block: paragraph,
                index: 1
            }
        );
        assert_eq!(document.revision(), revision);
        assert_eq!(document.paragraph(paragraph).unwrap().text(), "é");
    }

    #[test]
    fn style_assignment_requires_known_style_and_preserves_ids() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let heading = document.create_paragraph_style("Heading 1");
        let before = document.revision();

        let previous = document.set_paragraph_style(paragraph, heading).unwrap();

        assert_eq!(previous, document.default_paragraph_style());
        assert_eq!(document.paragraph(paragraph).unwrap().style(), heading);
        assert_eq!(document.revision(), before + 1);
    }

    #[test]
    fn appended_paragraphs_have_stable_non_index_identity() {
        let mut document = Document::new(1);
        let first = document.first_paragraph_id();
        let second = document
            .append_paragraph(document.default_paragraph_style())
            .unwrap();

        assert_ne!(first, second);
        assert_eq!(document.blocks().len(), 2);
        assert_eq!(document.blocks()[1].id(), second);
    }
}
