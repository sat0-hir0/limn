//! Limn UI bin.
//!
//! Boots a gpui application, opens a single window, and shows an
//! editable view of the document. With no argument it shows the
//! embedded Welcome document (VS Code's welcome tab equivalent); with
//! a path it loads that `.md` via `limn-service`.

use std::path::{Path, PathBuf};

use gpui::{px, size, App, AppContext, Bounds, SharedString, WindowBounds, WindowOptions};
use gpui_component::Root;
use gpui_platform::application;

use limn_core::markdown;
use limn_service::{Document, Vault};
use limn_ui::{DocumentView, EditorView, FeatureFlags};

const WELCOME_MD: &str = include_str!("welcome.md");
const WELCOME_TITLE: &str = "Welcome";

fn main() {
    let document = match std::env::args().nth(1) {
        Some(arg) => {
            let path = PathBuf::from(arg);
            match load_document(&path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("limn-ui: failed to load {}: {e}", path.display());
                    std::process::exit(1);
                }
            }
        }
        None => welcome_document(),
    };

    let title: SharedString = document
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| "(unnamed)".to_string(), String::from)
        .into();
    let text = document.text;
    let blocks = document.blocks;
    let flags = FeatureFlags::from_env();

    application().run(move |cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                if flags.edit {
                    let view =
                        cx.new(|cx| EditorView::new(title.clone(), text.clone(), window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                } else {
                    let view = cx.new(|_cx| DocumentView {
                        title: title.clone(),
                        blocks: blocks.clone(),
                    });
                    cx.new(|cx| Root::new(view, window, cx))
                }
            },
        )
        .expect("opening the main window should succeed"); // limn:allow-panic: window creation failure is unrecoverable at startup
        cx.activate(true);
    });
}

fn load_document(path: &Path) -> Result<Document, limn_service::OpenError> {
    if path.is_dir() {
        return Vault::new(path).open_first_md();
    }
    Vault::open_path(path)
}

/// The built-in Welcome document shown when limn-ui is launched
/// without a path argument. Bytes are baked into the binary via
/// `include_str!`, so the app has no runtime dependency on the repo
/// layout.
fn welcome_document() -> Document {
    let body = WELCOME_MD.replace("{VERSION}", env!("CARGO_PKG_VERSION"));
    Document {
        path: PathBuf::from(WELCOME_TITLE),
        blocks: markdown::parse(&body),
        text: body,
    }
}
