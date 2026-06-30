//! Block tree primitives. Currently supports Heading and Paragraph;
//! richer block types (lists, code, tables, images) are not yet
//! implemented.

/// What kind of block a node is. Keep this enum small — every variant
/// pays for itself in the renderer and the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    /// `#` … `######` headings. `level` is 1-based.
    Heading {
        level: u8,
    },
}

/// A single block: kind, text payload, and optional children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub text: String,
    pub children: Vec<Block>,
}

impl Block {
    #[must_use]
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self {
            kind: BlockKind::Paragraph,
            text: text.into(),
            children: Vec::new(),
        }
    }

    /// Heading helper. `level` is clamped to `1..=6` (anything outside
    /// the `CommonMark` range is treated as 6).
    #[must_use]
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        let level = level.clamp(1, 6);
        Self {
            kind: BlockKind::Heading { level },
            text: text.into(),
            children: Vec::new(),
        }
    }
}
