//! Debounced autosave for the open document.
//!
//! The editor calls [`Autosave::schedule`] on every text change. Each
//! call replaces the previous pending save, so the file is written once
//! the text has been quiet for [`AUTOSAVE_DEBOUNCE`]. The write itself
//! goes through [`Vault::save_raw`] on the background executor — the UI
//! never touches `std::fs` (ADR-0002) and never blocks the main thread.

use std::path::PathBuf;
use std::time::Duration;

use gpui::{Context, Task};

use limn_service::Vault;

/// How long the text must stay unchanged before it is written.
pub const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(600);

/// Pending-save state for one open document.
#[derive(Default)]
pub struct Autosave {
    target: Option<PathBuf>,
    pending: Option<Task<()>>,
}

impl Autosave {
    /// `target` is where saves go. `None` (the built-in Welcome
    /// document) disables autosave entirely.
    #[must_use]
    pub fn new(target: Option<PathBuf>) -> Self {
        Self {
            target,
            pending: None,
        }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.target.is_some()
    }

    /// Schedule a save of `text`, cancelling any save still waiting
    /// for its debounce window. No-op when autosave is disabled.
    pub fn schedule<V: 'static>(&mut self, text: String, cx: &mut Context<V>) {
        let Some(path) = self.target.clone() else {
            return;
        };
        // Dropping the previous task cancels it before it writes.
        self.pending = Some(cx.spawn(async move |_, cx| {
            let executor = cx.background_executor().clone();
            executor.timer(AUTOSAVE_DEBOUNCE).await;
            let result = executor
                .spawn(async move { Vault::save_raw(&path, &text).map_err(|e| (path, e)) })
                .await;
            if let Err((path, e)) = result {
                eprintln!("limn-ui: autosave to {} failed: {e}", path.display());
            }
        }));
    }
}
