//! gpui actions + keybindings for the editable shell (Wave 4 foundation).
//!
//! This module is the single registry for the editable shell's keyboard
//! intents. Per ADR-0008, the *action types* live here (limn-ui owns the
//! vocabulary of intents) while the *context* in which each action is
//! dispatched is declared by the view that handles it (e.g. the
//! `"Editor"` context on [`crate::editor::EditorView`]). Wave 4 only
//! lands the foundation: the action type and its global keybinding. The
//! handler that gives [`TogglePalette`] a body (opening the command
//! palette overlay) arrives in Wave 5; the action type stays stable
//! across that change.
use gpui::{actions, App, KeyBinding};

// namespace "limn". Placeholder palette toggle; Wave 5 gives it a body.
actions!(limn, [TogglePalette]);

/// Register all global keybindings. Called once from the app run closure.
///
/// The binding uses a `None` context, so it is global: the action is
/// dispatched as long as some element in the focused view's dispatch
/// tree declares a matching `on_action` handler. Wave 4 wires exactly
/// one such handler (`EditorView::on_toggle_palette`); the
/// `"Editor"`-context root keeps the editor on the focus chain so the
/// dispatch reaches it (see `crate::editor`).
pub fn bind_keys(cx: &mut App) {
    // gpui's "secondary" modifier resolves to Cmd on macOS and Ctrl on
    // Windows/Linux. A plain "cmd-" binding would collapse to the platform
    // modifier (Win+Shift+P on Windows, which the shell reserves and never
    // delivers to the app), so use "secondary" for correct cross-platform
    // dispatch from a single binding.
    cx.bind_keys([KeyBinding::new("secondary-shift-p", TogglePalette, None)]);
}
