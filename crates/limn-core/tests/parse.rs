//! Integration tests for the toy Markdown parser. Pairs with the unit
//! tests inside `markdown.rs`; this file holds the slightly larger
//! samples that read more like real `.md` files.

use limn_core::block::{Block, BlockKind};
use limn_core::markdown;

#[test]
fn heading_paragraph_pair() {
    let md = "# Hello\n\nworld\n";
    let blocks = markdown::parse(md);
    assert_eq!(
        blocks,
        vec![Block::heading(1, "Hello"), Block::paragraph("world")],
    );
}

#[test]
fn blank_lines_are_dropped() {
    let md = "\n\nfirst\n\n\nsecond\n\n";
    let blocks = markdown::parse(md);
    assert_eq!(blocks.len(), 2);
    assert!(blocks
        .iter()
        .all(|b| matches!(b.kind, BlockKind::Paragraph)));
}

#[test]
fn nested_headings_keep_their_levels() {
    let md = "# top\n\n## middle\n\n### bottom\n";
    let blocks = markdown::parse(md);
    let levels: Vec<u8> = blocks
        .iter()
        .filter_map(|b| match b.kind {
            BlockKind::Heading { level } => Some(level),
            BlockKind::Paragraph => None,
        })
        .collect();
    assert_eq!(levels, vec![1, 2, 3]);
}

#[test]
fn empty_input_yields_no_blocks() {
    assert!(markdown::parse("").is_empty());
    assert!(markdown::parse("\n\n\n").is_empty());
}

#[test]
fn hash_without_space_is_paragraph() {
    let blocks = markdown::parse("#nope\n");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0].kind, BlockKind::Paragraph));
    assert_eq!(blocks[0].text, "#nope");
}
