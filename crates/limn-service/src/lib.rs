//! Imperative shell: the side-effectful layer — I/O, indexing, AI calls.
//!
//! - [`vault`]: `.md` file I/O within a folder
//! - [`index`]: link and backlink index
//! - [`config`]: user configuration (`~/.config/limn/config.toml`)
//!
//! Never block the main thread (see ARCHITECTURE.md "Thread model").
//! Asynchronous runtimes like `tokio` will be introduced in M2 and beyond.

pub mod config;
pub mod index;
pub mod vault;

pub use config::{ConfigError, FontConfig, LimnConfig, Theme};
pub use vault::{Document, OpenError, RawDocument, Vault, VaultEntry};
