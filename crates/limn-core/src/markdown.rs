//! Markdown ⇄ block-tree (de)serialization.
//!
//! The round-trip property of this module is the project's "lifeline"
//! (see `docs/testing-strategy.md` §4).
//!
//! M1 ships a deliberately small line-oriented parser: `#`/`##`/`###`
//! lines become Headings, every other non-empty line becomes a
//! Paragraph, blank lines are skipped. `serialize()` is still stubbed
//! and will land alongside the real parser in M2.

use crate::block::Block;

/// Parse a Markdown string into a flat list of blocks.
///
/// This is the M1 toy parser. It only understands ATX-style headings
/// (`#` … `######`) and treats every other non-empty line as a
/// paragraph; consecutive non-blank lines do not merge. M2 will
/// replace this with a real CommonMark-aware parser.
#[must_use]
pub fn parse(md: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    for raw in md.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if let Some((level, text)) = parse_atx_heading(line) {
            blocks.push(Block::heading(level, text));
        } else {
            blocks.push(Block::paragraph(line));
        }
    }
    blocks
}

/// Serialize blocks back to Markdown. **Not implemented in M1.**
///
/// # Panics
///
/// Always panics — the implementation lands in M2.
#[must_use]
pub fn serialize(_blocks: &[Block]) -> String {
    unimplemented!("serialize: lands in M2 together with the real parser")
}

/// Returns `(level, text)` if `line` is an ATX heading, else `None`.
///
/// Recognises 1..=6 leading `#` characters followed by a space.
fn parse_atx_heading(line: &str) -> Option<(u8, &str)> {
    let mut level: u8 = 0;
    let bytes = line.as_bytes();
    while level < 6 && bytes.get(level as usize) == Some(&b'#') {
        level += 1;
    }
    if level == 0 {
        return None;
    }
    // Require a space (or end-of-line) after the hashes — otherwise
    // `#word` is just text, not a heading.
    match bytes.get(level as usize) {
        Some(&b' ') => Some((level, line[level as usize + 1..].trim_start())),
        None => Some((level, "")),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockKind;

    /// Round-trip: parse → serialize keeps the input intact (whitespace
    /// normalisation is allowed). Stays ignored until M2 ships
    /// `serialize`.
    #[test]
    #[ignore = "M2: serialize() not yet implemented"]
    fn roundtrip_preserves_input() {
        // intentionally empty
    }

    #[test]
    fn atx_heading_recognises_levels_1_through_6() {
        for n in 1..=6 {
            let md = format!("{} h\n", "#".repeat(n as usize));
            let blocks = parse(&md);
            assert_eq!(
                blocks,
                vec![Block::heading(n, "h")],
                "level {n} should round-trip into a Heading",
            );
        }
    }

    #[test]
    fn seven_hashes_falls_back_to_paragraph() {
        // 7 `#`s isn't a heading; the parser sees it as text.
        let blocks = parse("####### nope\n");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].kind, BlockKind::Paragraph));
    }
}
