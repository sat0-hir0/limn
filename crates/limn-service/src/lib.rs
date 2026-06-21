//! Imperative shell: the side-effectful layer — I/O, indexing, AI calls.
//!
//! - [`vault`]: `.md` file I/O within a folder
//! - [`index`]: link and backlink index
//!
//! Never block the main thread (see ARCHITECTURE.md "Thread model").
//! Asynchronous runtimes like `tokio` will be introduced in M2 and beyond.

pub mod index;
pub mod vault;

pub use vault::{Document, OpenError, Vault};
