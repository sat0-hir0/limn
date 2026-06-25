//! Limn UI bin.
//!
//! M1: boot a gpui application, open a single window, and render each
//! block as a row. With no argument it shows the embedded Welcome
//! document (VS Code's welcome tab equivalent); with a path it loads
//! that `.md` via `limn-service`. Read-only; editing lands in M2.

use std::path::{Path, PathBuf};

use gpui::{px, size, App, AppContext, Bounds, SharedString, WindowBounds, WindowOptions};
use gpui_platform::application;

use limn_core::markdown;
use limn_service::{Document, Vault};
use limn_ui::DocumentView;

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
    let blocks = document.blocks;

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);

        // gpui's `WindowOptions::icon` is honoured on X11 (Linux /
        // FreeBSD) and ignored elsewhere on the pinned revision
        // (see ADR-0005, ADR-0006). Decoding the PNG on macOS /
        // Windows would be wasted work, so we only build the icon on
        // the platforms that consume it.
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        let window_icon: Option<std::sync::Arc<image::RgbaImage>> = {
            const ICON_PNG: &[u8] = include_bytes!("../../../assets/appicons/linux/limn-256.png");
            image::load_from_memory(ICON_PNG)
                .ok()
                .map(|img| std::sync::Arc::new(img.into_rgba8()))
        };
        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        let window_icon: Option<std::sync::Arc<image::RgbaImage>> = None;

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                icon: window_icon,
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
