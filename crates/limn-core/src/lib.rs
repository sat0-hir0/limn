//! Functional core: pure logic only. No dependencies beyond `std`.
//!
//! Modules:
//! - `block`: minimal block-tree structures
//! - `markdown`: block tree ⇄ Markdown serialization (the lifeline)
//! - `completion`: completion-provider abstractions
//! - `fuzzy`: fuzzy matching for the command palette

pub mod block;
pub mod completion;
pub mod fuzzy;
pub mod markdown;
