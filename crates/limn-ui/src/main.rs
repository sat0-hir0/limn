//! Limn UI bin.
//!
//! M1: boot a gpui application, open a single window, ask
//! `limn-service` to load `samples/hello.md` (or the CLI argument), and
//! render each block as a row. Read-only; editing lands in M2.

use std::path::{Path, PathBuf};

use gpui::{px, size, App, AppContext, Bounds, SharedString, WindowBounds, WindowOptions};
use gpui_platform::application;

use limn_service::Vault;
use limn_ui::DocumentView;

const FALLBACK_PATH: &str = "samples/hello.md";

fn main() {
    let path = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from(FALLBACK_PATH), PathBuf::from);
    let document = match load_document(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("limn-ui: failed to load {}: {e}", path.display());
            std::process::exit(1);
        }
    };

    let title: SharedString = document
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| "(unnamed)".to_string(), String::from)
        .into();
    let blocks = document.blocks;

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| DocumentView {
                    title: title.clone(),
                    blocks: blocks.clone(),
                })
            },
        )
        .expect("opening the main window should succeed");
        cx.activate(true);
    });
}

fn load_document(path: &Path) -> Result<limn_service::Document, limn_service::OpenError> {
    if path.is_dir() {
        return Vault::new(path).open_first_md();
    }
    Vault::open_path(path)
}
