//! Imperative shell: the side-effectful layer — I/O, indexing, AI calls.
//!
//! - [`vault`]: `.md` file I/O within a folder
//! - [`index`]: link and backlink index
//! - [`config`]: user settings persisted as TOML
//!
//! Never block the main thread (see ARCHITECTURE.md "Thread model").

pub mod config;
pub mod index;
pub mod vault;

pub use config::{Config, ConfigError, Theme};
pub use vault::{Document, OpenError, Vault};
