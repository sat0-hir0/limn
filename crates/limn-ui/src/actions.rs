//! gpui actions + keybindings for the editable shell (Wave 4 foundation,
//! extended for view switching in Wave 8).
//!
//! This module is the single registry for the editable shell's keyboard
//! intents. Per ADR-0008, the *action types* live here (limn-ui owns the
//! vocabulary of intents) while the *context* in which each action is
//! dispatched is declared by the view that handles it (e.g. the
//! `"AppShell"` context on [`crate::shell::AppShell`], the `"Settings"`
//! context on [`crate::settings::SettingsView`]). The handlers that give
//! these actions a body live on those views; the action types stay stable
//! across handler changes.
//!
//! Wave 8 adds two intents for the settings view (ADR-0010):
//! [`OpenSettings`] (editor → settings) and [`CloseSettings`] (settings →
//! editor). The palette toggle moved from `EditorView` to `AppShell`,
//! which now owns view switching; the `TogglePalette` *type* is unchanged.
use gpui::{actions, App, KeyBinding};

// namespace "limn".
// - TogglePalette: open/close the command palette (handled by AppShell).
// - OpenSettings:  switch from the editor to the settings view.
// - CloseSettings: switch from the settings view back to the editor.
actions!(limn, [TogglePalette, OpenSettings, CloseSettings]);

/// The key context name declared by [`crate::settings::SettingsView`].
/// Shared between the view (which declares it on its render root) and
/// [`bind_keys`] (which scopes the Esc binding to it) so the two cannot
/// drift apart.
pub const SETTINGS_CONTEXT: &str = "Settings";

/// Register all global keybindings. Called once from the app run closure.
///
/// `TogglePalette` and `OpenSettings` use a `None` context, so they are
/// global: each dispatches as long as some element in the focused view's
/// dispatch tree declares a matching `on_action` handler. `AppShell`
/// tracks its own focus handle on its render root and is an ancestor of
/// whichever screen is focused, so both reach its handlers (see
/// `crate::shell`).
///
/// `CloseSettings` is bound to `escape` **scoped to the `"Settings"`
/// context** so Esc only closes settings when the settings view is the
/// active screen — it must not fire while the editor is focused (where Esc
/// is free) nor steal the palette Dialog's own Esc-to-close. ADR-0010
/// records this 3-route convergence (palette / keybinding / Esc) onto
/// `AppShell::show_settings` / `show_editor`.
pub fn bind_keys(cx: &mut App) {
    // gpui's "secondary" modifier resolves to Cmd on macOS and Ctrl on
    // Windows/Linux. A plain "cmd-" binding would collapse to the platform
    // modifier (Win+Shift+P on Windows, which the shell reserves and never
    // delivers to the app), so use "secondary" for correct cross-platform
    // dispatch from a single binding.
    cx.bind_keys([
        KeyBinding::new("secondary-shift-p", TogglePalette, None),
        // "secondary-," mirrors the VS Code "open settings" convention
        // (Cmd/Ctrl + comma).
        KeyBinding::new("secondary-,", OpenSettings, None),
        // Esc closes settings only while the settings screen is active.
        KeyBinding::new("escape", CloseSettings, Some(SETTINGS_CONTEXT)),
    ]);
}
