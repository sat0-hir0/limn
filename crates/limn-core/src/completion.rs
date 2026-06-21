//! Completion engine abstractions.
//!
//! Following ARCHITECTURE.md "Completion engine structure", this module
//! separates the candidate Provider ("who produces candidates") from the
//! Policy ("who decides when to show them").
//!
//! M0: traits only. Concrete implementations land in M3 and beyond.

/// A single completion candidate — the smallest unit a provider returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub label: String,
    pub insert_text: String,
}

/// Input context, such as the text immediately preceding the cursor.
/// M0 ships an empty placeholder struct.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub preceding_text: String,
}

/// A candidate provider. Synchronous and zero-latency by contract
/// (asynchronous sources like AI plug in via a separate trait later).
pub trait Provider {
    fn name(&self) -> &str;
    fn provide(&self, ctx: &Context) -> Vec<Candidate>;
}
