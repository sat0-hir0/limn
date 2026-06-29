//! App shell that owns view switching between the editor and settings (Wave 8).
//!
//! `AppShell` sits between the gpui-component `Root` and the active screen
//! (either [`EditorView`] or [`SettingsView`]). It is the single owner of
//! the "which view is visible" question, so the three routes that swap
//! screens (palette command, `secondary-,` keybinding, `Esc` in Settings)
//! all converge on the same handlers via gpui actions (ADR-0010).
//!
//! Why a shell and not "`EditorView` with a settings flag":
//!
//! - Settings has its own input widgets and Esc semantics that are
//!   incompatible with the editor's. A separate view lets each declare its
//!   own key context (`"Editor"` / `"Settings"`) cleanly.
//! - Action handlers and the dialog overlay live above both screens, so a
//!   global `TogglePalette` dispatches uniformly and the palette can be
//!   opened only from the editor screen — settings doesn't trip on it.
//! - The editor and settings entities are cached on the shell so toggling
//!   between them is just a screen swap; the editor's autosave debounce
//!   timer and the settings draft survive the transition.

use gpui::{
    div, App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{Root, WindowExt as _};

use crate::actions::{CloseSettings, OpenSettings, TogglePalette};
use crate::{EditorView, FeatureFlags, PaletteView, SettingsView};

/// Which screen the shell is currently showing.
///
/// Both variants carry an `Entity` so the shell can render the chosen
/// screen by handing its entity to `div().child(...)`; the entities are
/// also cached on the shell (see [`AppShell`]) so swapping screens does
/// not drop their state.
pub enum Screen {
    Editor(Entity<EditorView>),
    Settings(Entity<SettingsView>),
}

/// Public discriminant for [`Screen`], so test code (and any future
/// inspector) can ask "which screen is showing?" without naming the
/// entities. Wave 8's test relies on this — the `AppShell` screen-switch
/// assertion only needs the variant, not the concrete entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    Editor,
    Settings,
}

/// Top-level view that owns view switching.
///
/// The render root tracks its own focus handle so the `AppShell` is always
/// on the focus tree above whichever screen is active; that is what makes
/// the globally-bound `TogglePalette` and `OpenSettings` actions reach the
/// shell's handlers (ADR-0008).
pub struct AppShell {
    screen: Screen,
    /// The editor entity, cached so that round-tripping
    /// editor → settings → editor restores the *same* editor (preserving
    /// buffer, autosave state, focus). Without this, `CloseSettings` would
    /// have nothing to swap back to.
    editor_cache: Entity<EditorView>,
    /// The settings entity, similarly cached so opening settings repeatedly
    /// keeps the draft fields intact.
    settings_cache: Entity<SettingsView>,
    focus_handle: FocusHandle,
}

impl AppShell {
    /// Build the shell around an already-constructed editor.
    ///
    /// The settings view is built eagerly here (rather than lazily on
    /// first open) so its `_subscriptions` and `InputState`s register
    /// against the same `Window` as the editor; the user pays the small
    /// upfront cost once.
    pub fn new(editor: Entity<EditorView>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = cx.new(|cx| SettingsView::new(window, cx));
        Self {
            screen: Screen::Editor(editor.clone()),
            editor_cache: editor,
            settings_cache: settings,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Which screen the shell is currently showing. Exposed for tests
    /// and for diagnostics; production rendering reads `self.screen`
    /// directly.
    #[must_use]
    pub fn screen_kind(&self) -> ScreenKind {
        match &self.screen {
            Screen::Editor(_) => ScreenKind::Editor,
            Screen::Settings(_) => ScreenKind::Settings,
        }
    }

    /// Show the settings screen, focusing it so its Esc context is active.
    ///
    /// The action is globally bound (no context scope), so it can fire while
    /// the palette dialog is open. In that case the dialog must be closed
    /// here — leaving it on `Root::active_dialogs` would let the palette
    /// re-render layered over the settings screen on the next paint. The
    /// palette confirm route closes the dialog itself before dispatching
    /// `OpenSettings`, so the close call below is a no-op there.
    fn on_open_settings(&mut self, _: &OpenSettings, window: &mut Window, cx: &mut Context<Self>) {
        if window.has_active_dialog(cx) {
            window.close_dialog(cx);
        }
        if matches!(&self.screen, Screen::Settings(_)) {
            return;
        }
        self.screen = Screen::Settings(self.settings_cache.clone());
        self.settings_cache
            .update(cx, |view, cx| view.focus(window, cx));
        cx.notify();
    }

    /// Return from the settings screen to the editor and refocus the
    /// editor so the next keystroke lands in the buffer.
    ///
    /// Guarded against a no-op dispatch (editor → editor) so future side
    /// effects on this handler do not fire on the wrong screen — the Esc
    /// keybinding is already scoped to `"Settings"`, but the action is
    /// reachable from buttons and tests too.
    fn on_close_settings(
        &mut self,
        _: &CloseSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(&self.screen, Screen::Editor(_)) {
            return;
        }
        self.screen = Screen::Editor(self.editor_cache.clone());
        self.editor_cache
            .update(cx, |view, cx| view.focus(window, cx));
        cx.notify();
    }

    /// Handle the [`TogglePalette`] action: open the command palette on
    /// the editor screen, or no-op on the settings screen.
    ///
    /// Gated on `LIMN_FEAT_PALETTE`: when the flag is off the action is a
    /// no-op (the keybinding stays registered for consistency).
    ///
    /// Moved here from `EditorView` (Wave 5) because the dialog overlay
    /// now lives on the shell. The palette is intentionally suppressed on
    /// the settings screen — Wave 8 has no settings-mode commands, and
    /// allowing the dialog to open over the settings UI would compete with
    /// the Esc-to-close binding scoped to the `"Settings"` context.
    fn on_toggle_palette(
        &mut self,
        _: &TogglePalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !cx.global::<FeatureFlags>().palette {
            return;
        }

        // The palette only makes sense on the editor screen for now.
        // Wave 9+ could lift this if a settings-mode command set arrives.
        let Screen::Editor(editor) = &self.screen else {
            return;
        };

        if window.has_active_dialog(cx) {
            window.close_dialog(cx);
            return;
        }

        let editor_weak = editor.downgrade();
        let palette = cx.new(|cx| PaletteView::new(editor_weak, window, cx));
        window.open_dialog(cx, {
            let palette = palette.clone();
            move |dialog, _, _| dialog.title("Command Palette").child(palette.clone())
        });

        palette.update(cx, |palette, cx| palette.focus_list(window, cx));
    }
}

impl Focusable for AppShell {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AppShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let child = match &self.screen {
            Screen::Editor(editor) => editor.clone().into_any_element(),
            Screen::Settings(settings) => settings.clone().into_any_element(),
        };

        div()
            // Place the shell on the focus tree so globally-bound actions
            // (`TogglePalette`, `OpenSettings`) bubble to its handlers; the
            // `"AppShell"` key context is the shell's namespace, distinct
            // from the screens' `"Editor"` / `"Settings"` contexts.
            .track_focus(&self.focus_handle)
            .key_context("AppShell")
            .on_action(cx.listener(Self::on_open_settings))
            .on_action(cx.listener(Self::on_close_settings))
            .on_action(cx.listener(Self::on_toggle_palette))
            .size_full()
            .child(child)
            // The palette overlay used to live on `EditorView`; Wave 8
            // moved it here so the dialog is hosted by the shell rather
            // than the editor screen, and the palette can be opened/closed
            // independently of which screen is active.
            .children(Root::render_dialog_layer(window, cx))
    }
}
