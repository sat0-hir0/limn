//! Limn UI bin.
//!
//! Boots a gpui application and opens a single window.
//!
//! - Default (read-only): renders each parsed block as a row. With no
//!   argument it shows the embedded Welcome document; with a path it
//!   loads that `.md` via `limn-service`.
//! - `LIMN_FEAT_EDIT=1` (M2, hidden flag): opens an editable view backed
//!   by `gpui-component`'s `InputState`, seeded with the file's raw
//!   text. Input, cursor, selection, delete, copy/cut/paste, undo/redo,
//!   and IME come from the component (see ADR-0005). Autosave is a later
//!   wave.

use std::path::{Path, PathBuf};

use gpui::{px, size, App, AppContext, Bounds, SharedString, WindowBounds, WindowOptions};
use gpui_component::Root;
use gpui_component_assets::Assets;
use gpui_platform::application;

use limn_core::markdown;
use limn_service::{Document, LimnConfig, RawDocument, Vault};
use limn_ui::{file_title, AppConfig, AppShell, DocumentView, EditorView, FeatureFlags};

const WELCOME_MD: &str = include_str!("welcome.md");
const WELCOME_TITLE: &str = "Welcome";

fn main() {
    let flags = FeatureFlags::from_env();

    // Load the user config once at startup. `load` never fails — it falls
    // back to defaults and logs the reason (ADR-0007). Wave 7 applies
    // `vault_path` (below) and carries `font` / `theme` in the global;
    // applying those to rendering is Wave 8.
    let config = LimnConfig::load();
    eprintln!(
        "limn-ui: config loaded (theme={:?}, vault_path={:?})",
        config.theme, config.vault_path
    );

    if flags.edit {
        run_editable(flags, config);
    } else {
        run_read_only(flags, config);
    }
}

/// Read-only path (M1 behaviour): parse the document into blocks and
/// render them with [`DocumentView`].
fn run_read_only(flags: FeatureFlags, config: LimnConfig) {
    let document = match std::env::args().nth(1) {
        Some(arg) => load_document(&PathBuf::from(arg)),
        None => Ok(welcome_document()),
    };
    let document = match document {
        Ok(d) => d,
        Err(e) => {
            eprintln!("limn-ui: failed to load document: {e}");
            std::process::exit(1);
        }
    };

    let title = file_title(&document.path);
    let blocks = document.blocks;

    application().run(move |cx: &mut App| {
        cx.set_global(flags.clone());
        cx.set_global(AppConfig(config.clone()));
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
        .expect("opening the main window should succeed"); // limn:allow-panic: window creation failure is unrecoverable at startup
        cx.activate(true);
    });
}

/// Editable path (`LIMN_FEAT_EDIT=1`): seed an [`EditorView`] with the
/// file's raw text. `gpui-component` needs its bundled [`Assets`], a
/// `gpui_component::init`, and a [`Root`] as the window's first view.
fn run_editable(flags: FeatureFlags, config: LimnConfig) {
    // Resolution order for what to open (Wave 7 wires in `vault_path`):
    //
    // 1. A CLI path argument always wins (preserves existing behaviour).
    // 2. Otherwise, if the config has a `vault_path`, open the first `.md`
    //    in that vault — this is Wave 7's user-facing value: launching
    //    with no argument lands you in your configured vault.
    // 3. Otherwise fall back to the embedded Welcome document.
    //
    // A real file on disk (cases 1 and 2) has a save target; Welcome is
    // ephemeral and autosaves to `None`, so edits to it never write a
    // stray file to the working directory.
    let (raw, save_path) = match std::env::args().nth(1) {
        Some(arg) => match load_raw_document(&PathBuf::from(arg)) {
            // Save back to the resolved path (a directory input resolves
            // to the first `.md` inside it), not the raw argv string.
            Ok(d) => {
                let resolved = d.path.clone();
                (Ok(d), Some(resolved))
            }
            Err(e) => (Err(e), None),
        },
        None => match config.vault_path.as_deref() {
            Some(vault_root) => match load_raw_document(vault_root) {
                Ok(d) => {
                    let resolved = d.path.clone();
                    (Ok(d), Some(resolved))
                }
                // An unreadable / empty configured vault is not fatal:
                // log and fall back to Welcome rather than refusing to
                // start (mirrors the load-never-blocks-startup stance).
                Err(e) => {
                    eprintln!(
                        "limn-ui: configured vault_path {} unusable: {e}; showing Welcome",
                        vault_root.display()
                    );
                    (Ok(welcome_raw_document()), None)
                }
            },
            None => (Ok(welcome_raw_document()), None),
        },
    };
    let raw = match raw {
        Ok(d) => d,
        Err(e) => {
            eprintln!("limn-ui: failed to load document: {e}");
            std::process::exit(1);
        }
    };

    let title = file_title(&raw.path);
    let path = save_path;
    let text: SharedString = raw.text.into();

    application().with_assets(Assets).run(move |cx: &mut App| {
        // Required before using any gpui-component feature.
        gpui_component::init(cx);
        cx.set_global(flags);
        cx.set_global(AppConfig(config));
        // Register the editable shell's global keybindings (Wave 4).
        limn_ui::actions::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let editor = cx.new(|cx| {
                    EditorView::new(title.clone(), path.clone(), text.clone(), window, cx)
                });
                // Focus the editor so the first keystroke lands in
                // the buffer without a click.
                editor.update(cx, |view, cx| view.focus(window, cx));
                // Wave 8: the editor is now wrapped in an `AppShell`
                // that owns view switching (editor ↔ settings) and the
                // palette dialog overlay; the `Root` then wraps the
                // shell since the window's first-level view must be a
                // `Root` (ADR-0010).
                let shell = cx.new(|cx| AppShell::new(editor, window, cx));
                cx.new(|cx| Root::new(shell, window, cx))
            })
            .expect("opening the main window should succeed"); // limn:allow-panic: window creation failure is unrecoverable at startup
            cx.update(|cx| cx.activate(true));
        })
        .detach();
    });
}

fn load_document(path: &Path) -> Result<Document, limn_service::OpenError> {
    if path.is_dir() {
        return Vault::new(path).open_first_md();
    }
    Vault::open_path(path)
}

/// Raw-text counterpart of [`load_document`] for the editable path.
/// Directory inputs resolve to the first `.md` file's raw text.
fn load_raw_document(path: &Path) -> Result<RawDocument, limn_service::OpenError> {
    if path.is_dir() {
        let doc = Vault::new(path).open_first_md()?;
        return Vault::open_path_raw(&doc.path);
    }
    Vault::open_path_raw(path)
}

/// The built-in Welcome document shown when limn-ui is launched without
/// a path argument. Bytes are baked into the binary via `include_str!`.
fn welcome_document() -> Document {
    Document {
        path: PathBuf::from(WELCOME_TITLE),
        blocks: markdown::parse(&welcome_body()),
    }
}

/// Raw-text Welcome document for the editable path.
fn welcome_raw_document() -> RawDocument {
    RawDocument {
        path: PathBuf::from(WELCOME_TITLE),
        text: welcome_body(),
    }
}

fn welcome_body() -> String {
    WELCOME_MD.replace("{VERSION}", env!("CARGO_PKG_VERSION"))
}
