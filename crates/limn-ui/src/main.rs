//! Limn UI bin.
//!
//! Boots a gpui application, opens a single window, and renders one of
//! two views. With `--settings` it opens the editable Settings view;
//! otherwise it renders a document — the embedded Welcome document
//! (VS Code's welcome tab equivalent) with no path, or the `.md` at the
//! given path via `limn-service`. The document view is read-only;
//! editing lives in the Settings view.

use std::path::{Path, PathBuf};

use gpui::{px, size, App, AppContext, Bounds, SharedString, WindowBounds, WindowOptions};
use gpui_platform::application;

use limn_core::markdown;
use limn_service::{Config, Document, Vault};
use limn_ui::{DocumentView, SettingsView};

/// CLI flag that opens the Settings view instead of a document.
const SETTINGS_FLAG: &str = "--settings";

const WELCOME_MD: &str = include_str!("welcome.md");
const WELCOME_TITLE: &str = "Welcome";

fn main() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("limn-ui: failed to load config, using defaults: {e}");
            Config::default()
        }
    };

    // `--settings` selects the Settings view; any other first argument
    // is treated as a document path (per ADR-0005: Settings is a
    // separate switchable view opened via a launch flag).
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == SETTINGS_FLAG) {
        run_settings(config);
    } else {
        run_document(config, args.into_iter().next());
    }
}

/// Open the editable Settings view over a working copy of `config`.
fn run_settings(config: Config) {
    application().run(move |cx: &mut App| {
        open_window(cx, |_, cx| cx.new(|_| SettingsView::new(config.clone())));
    });
}

/// Open the read-only document view. With a `path` load that `.md`;
/// without one show the embedded Welcome document.
fn run_document(config: Config, path: Option<String>) {
    let document = match path {
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
    let blocks = document.blocks;

    application().run(move |cx: &mut App| {
        open_window(cx, |_, cx| {
            cx.new(|_| DocumentView {
                title: title.clone(),
                blocks: blocks.clone(),
                config: config.clone(),
            })
        });
    });
}

/// Open the single centred main window, build its root view with
/// `build_view`, and activate the app.
fn open_window<V, F>(cx: &mut App, build_view: F)
where
    V: gpui::Render,
    F: FnOnce(&mut gpui::Window, &mut App) -> gpui::Entity<V>,
{
    let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        },
        build_view,
    )
    .expect("opening the main window should succeed"); // limn:allow-panic: window creation failure is unrecoverable at startup
    cx.activate(true);
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
    }
}
