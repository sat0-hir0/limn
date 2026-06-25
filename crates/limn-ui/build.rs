//! `limn-ui` build script.
//!
//! On Windows targets, embeds `assets/appicons/windows/limn.ico` into
//! `limn-ui.exe` via `winresource` so Explorer, the task bar, and
//! Alt+Tab show the Limn mark instead of a generic placeholder. The
//! workspace-level `unsafe_code = "forbid"` lint does not reach build
//! scripts, so the resource embed runs outside the runtime lint
//! envelope (see ADR-0005, ADR-0006).
//!
//! On non-Windows targets this script is a no-op. We branch on
//! `CARGO_CFG_TARGET_OS` (the *target* OS Cargo is building for), not
//! on `#[cfg(target_os = "windows")]` which would resolve against the
//! *host* OS that compiles the build script itself — that distinction
//! matters once we add cross-compile (Linux host → Windows target).

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    // Re-run if the icon bundle changes; the path is relative to this
    // crate's directory (= `crates/limn-ui/`), so `../../` walks up to
    // the workspace root.
    println!("cargo::rerun-if-changed=../../assets/appicons/windows/limn.ico");

    embed_windows_icon();
}

fn embed_windows_icon() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set by cargo");
    let icon_path = std::path::Path::new(&manifest_dir)
        .join("..")
        .join("..")
        .join("assets")
        .join("appicons")
        .join("windows")
        .join("limn.ico");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon_path.to_str().expect("icon path is valid UTF-8"));
    res.compile()
        .expect("winresource failed to compile icon resource");
}
