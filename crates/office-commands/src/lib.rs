//! Undoable command framework for the `GoreeCloud` Office Engine.
//!
//! Commands are the mutation boundary for editor actions. Applying a command
//! returns its inverse, allowing deterministic undo/redo without exposing
//! direct mutable access to canonical document state.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use goreecloud_office_document::{BlockId, Document, DocumentError, StyleId};

/// A reversible Office document command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    /// Insert logical Unicode text at a UTF-8 byte boundary.
    InsertText {
        /// Target paragraph.
        block: BlockId,
        /// UTF-8 byte position.
        at: usize,
        /// Text to insert in logical order.
        text: String,
    },
    /// Delete a UTF-8 byte range from a paragraph.
    DeleteText {
        /// Target paragraph.
        block: BlockId,
        /// Inclusive UTF-8 byte start.
        start: usize,
        /// Exclusive UTF-8 byte end.
        end: usize,
    },
    /// Assign a paragraph style.
    SetParagraphStyle {
        /// Target paragraph.
        block: BlockId,
        /// New paragraph style.
        style: StyleId,
    },
}

impl Command {
    /// Applies the command and returns the exact inverse command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::Document`] when the canonical document rejects
    /// the requested mutation.
    pub fn apply(&self, document: &mut Document) -> Result<Self, CommandError> {
        match self {
            Self::InsertText { block, at, text } => {
                document.insert_text(*block, *at, text)?;
                Ok(Self::DeleteText {
                    block: *block,
                    start: *at,
                    end: *at + text.len(),
                })
            }
            Self::DeleteText { block, start, end } => {
                let removed = document.remove_text(*block, *start, *end)?;
                Ok(Self::InsertText {
                    block: *block,
                    at: *start,
                    text: removed,
                })
            }
            Self::SetParagraphStyle { block, style } => {
                let previous = document.set_paragraph_style(*block, *style)?;
                Ok(Self::SetParagraphStyle {
                    block: *block,
                    style: previous,
                })
            }
        }
    }
}

/// Command execution failures.
#[derive(Debug, Eq, PartialEq)]
pub enum CommandError {
    /// The document rejected the requested mutation.
    Document(DocumentError),
    /// No undo command is available.
    NothingToUndo,
    /// No redo command is available.
    NothingToRedo,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document(error) => write!(f, "document command failed: {error}"),
            Self::NothingToUndo => f.write_str("there is no command to undo"),
            Self::NothingToRedo => f.write_str("there is no command to redo"),
        }
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Document(error) => Some(error),
            Self::NothingToUndo | Self::NothingToRedo => None,
        }
    }
}

impl From<DocumentError> for CommandError {
    fn from(value: DocumentError) -> Self {
        Self::Document(value)
    }
}

/// Bounded undo/redo history for one document editing session.
#[derive(Debug)]
pub struct CommandExecutor {
    max_depth: usize,
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

impl CommandExecutor {
    /// Creates a command executor with a bounded history depth.
    #[must_use]
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Returns whether an undo operation is currently available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns whether a redo operation is currently available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Executes a new user command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::Document`] when the document rejects the
    /// command. Failed commands do not change undo or redo history.
    pub fn execute(
        &mut self,
        document: &mut Document,
        command: &Command,
    ) -> Result<(), CommandError> {
        let inverse = command.apply(document)?;
        self.redo_stack.clear();
        self.push_bounded_undo(inverse);
        Ok(())
    }

    /// Undoes the most recent successful command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::NothingToUndo`] when no undo entry exists, or
    /// [`CommandError::Document`] when applying the inverse command fails.
    pub fn undo(&mut self, document: &mut Document) -> Result<(), CommandError> {
        let inverse = self.undo_stack.pop().ok_or(CommandError::NothingToUndo)?;
        match inverse.apply(document) {
            Ok(redo) => {
                self.push_bounded_redo(redo);
                Ok(())
            }
            Err(error) => {
                self.undo_stack.push(inverse);
                Err(error)
            }
        }
    }

    /// Redoes the most recently undone command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::NothingToRedo`] when no redo entry exists, or
    /// [`CommandError::Document`] when replaying the command fails.
    pub fn redo(&mut self, document: &mut Document) -> Result<(), CommandError> {
        let command = self.redo_stack.pop().ok_or(CommandError::NothingToRedo)?;
        match command.apply(document) {
            Ok(inverse) => {
                self.push_bounded_undo(inverse);
                Ok(())
            }
            Err(error) => {
                self.redo_stack.push(command);
                Err(error)
            }
        }
    }

    fn push_bounded_undo(&mut self, command: Command) {
        if self.max_depth == 0 {
            return;
        }
        if self.undo_stack.len() == self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(command);
    }

    fn push_bounded_redo(&mut self, command: Command) {
        if self.max_depth == 0 {
            return;
        }
        if self.redo_stack.len() == self.max_depth {
            self.redo_stack.remove(0);
        }
        self.redo_stack.push(command);
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new(512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(document: &Document, block: BlockId) -> &str {
        document.paragraph(block).unwrap().text()
    }

    #[test]
    fn insert_can_be_undone_and_redone() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let mut commands = CommandExecutor::default();

        commands
            .execute(
                &mut document,
                &Command::InsertText {
                    block: paragraph,
                    at: 0,
                    text: "Hello 👋".to_owned(),
                },
            )
            .unwrap();
        assert_eq!(text(&document, paragraph), "Hello 👋");
        assert!(commands.can_undo());
        assert!(!commands.can_redo());

        commands.undo(&mut document).unwrap();
        assert_eq!(text(&document, paragraph), "");
        assert!(commands.can_redo());

        commands.redo(&mut document).unwrap();
        assert_eq!(text(&document, paragraph), "Hello 👋");
    }

    #[test]
    fn deletion_restores_exact_logical_unicode_text() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let mut commands = CommandExecutor::default();
        document.insert_text(paragraph, 0, "AéB").unwrap();

        commands
            .execute(
                &mut document,
                &Command::DeleteText {
                    block: paragraph,
                    start: 1,
                    end: 3,
                },
            )
            .unwrap();
        assert_eq!(text(&document, paragraph), "AB");

        commands.undo(&mut document).unwrap();
        assert_eq!(text(&document, paragraph), "AéB");
    }

    #[test]
    fn style_change_is_reversible() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let normal = document.default_paragraph_style();
        let heading = document.create_paragraph_style("Heading 1");
        let mut commands = CommandExecutor::default();

        commands
            .execute(
                &mut document,
                &Command::SetParagraphStyle {
                    block: paragraph,
                    style: heading,
                },
            )
            .unwrap();
        assert_eq!(document.paragraph(paragraph).unwrap().style(), heading);

        commands.undo(&mut document).unwrap();
        assert_eq!(document.paragraph(paragraph).unwrap().style(), normal);
    }

    #[test]
    fn a_new_command_discards_redo_history() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let mut commands = CommandExecutor::default();

        commands
            .execute(
                &mut document,
                &Command::InsertText {
                    block: paragraph,
                    at: 0,
                    text: "A".to_owned(),
                },
            )
            .unwrap();
        commands.undo(&mut document).unwrap();
        assert!(commands.can_redo());

        commands
            .execute(
                &mut document,
                &Command::InsertText {
                    block: paragraph,
                    at: 0,
                    text: "B".to_owned(),
                },
            )
            .unwrap();
        assert_eq!(text(&document, paragraph), "B");
        assert!(!commands.can_redo());
    }

    #[test]
    fn failed_command_does_not_pollute_history() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let mut commands = CommandExecutor::default();
        document.insert_text(paragraph, 0, "é").unwrap();

        let error = commands
            .execute(
                &mut document,
                &Command::InsertText {
                    block: paragraph,
                    at: 1,
                    text: "x".to_owned(),
                },
            )
            .unwrap_err();

        assert!(matches!(
            error,
            CommandError::Document(DocumentError::InvalidUtf8Boundary { .. })
        ));
        assert!(!commands.can_undo());
        assert!(!commands.can_redo());
        assert_eq!(text(&document, paragraph), "é");
    }

    #[test]
    fn history_depth_is_bounded() {
        let mut document = Document::new(1);
        let paragraph = document.first_paragraph_id();
        let mut commands = CommandExecutor::new(2);

        for value in ["A", "B", "C"] {
            let at = document.paragraph(paragraph).unwrap().text().len();
            commands
                .execute(
                    &mut document,
                    &Command::InsertText {
                        block: paragraph,
                        at,
                        text: value.to_owned(),
                    },
                )
                .unwrap();
        }
        assert_eq!(text(&document, paragraph), "ABC");

        commands.undo(&mut document).unwrap();
        commands.undo(&mut document).unwrap();
        assert_eq!(text(&document, paragraph), "A");
        assert_eq!(
            commands.undo(&mut document),
            Err(CommandError::NothingToUndo)
        );
    }
}
