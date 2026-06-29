//! Settings view (Wave 8) — a separate screen for editing the user
//! configuration that ships in [`limn_service::LimnConfig`].
//!
//! Concretely: three text inputs (font family / font size / vault path)
//! and a theme switch, plus a Save button that writes the draft to
//! `~/.config/limn/config.toml` via `LimnConfig::save_to` (ADR-0002:
//! limn-ui never calls `std::fs`).
//!
//! The view holds a `draft` copy of the config that the inputs mutate
//! independently of the live [`crate::AppConfig`] global; only Save
//! commits the draft back into the global, so cancelling out via Esc /
//! "Back to editor" discards any pending changes.
//!
//! Wave 8 scope: this just persists the values. Applying `font` / `theme`
//! to rendering is deliberately deferred — the change still lands on
//! disk and the next launch picks it up, which is enough to demo the
//! settings UI end-to-end (ADR-0010).

use std::path::PathBuf;

use gpui::{
    div, px, App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Subscription, Window,
};
use gpui_component::{
    button::Button,
    input::{Input, InputEvent, InputState},
    label::Label,
    switch::Switch,
    ActiveTheme, Theme as GpuiTheme, ThemeMode,
};

use limn_service::{LimnConfig, Theme as LimnTheme};

use crate::actions::{CloseSettings, SETTINGS_CONTEXT};
use crate::{AppConfig, ColorTheme, ColorThemeGlobal};

/// Editable settings screen backed by a `draft` copy of [`LimnConfig`].
///
/// The three text inputs each own an [`InputState`]; their `Change` events
/// are subscribed in `new` and update `draft` in place. Save serializes
/// the draft via `LimnConfig::save_to` (on the background executor so the
/// main thread does not block on the rename) and, on success, copies the
/// draft into the [`AppConfig`] global so the running session sees it.
pub struct SettingsView {
    focus_handle: FocusHandle,
    /// In-memory edit buffer. Diverges from `AppConfig` between opens and
    /// gets committed back to it (and to disk) on Save.
    draft: LimnConfig,
    font_family_state: Entity<InputState>,
    font_size_state: Entity<InputState>,
    vault_path_state: Entity<InputState>,
    /// Mirrored bool because [`Theme`] is a two-variant enum we can show as
    /// a switch; tracking it here keeps the render path simple.
    theme_dark: bool,
    /// Holds the input-event subscriptions; dropping the view unsubscribes.
    _subscriptions: Vec<Subscription>,
}

impl SettingsView {
    /// Build the settings view from the current [`AppConfig`] global.
    ///
    /// The draft is initialized from the global so opening settings shows
    /// the values the rest of the app is using. Subsequent edits diverge
    /// from the global until Save commits them back.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current = cx.global::<AppConfig>().0.clone();

        let family_init: SharedString = current.font.family.clone().into();
        let size_init: SharedString = current.font.size.to_string().into();
        let vault_init: SharedString = current
            .vault_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
            .into();

        let font_family_state = cx.new(|cx| InputState::new(window, cx).default_value(family_init));
        let font_size_state = cx.new(|cx| InputState::new(window, cx).default_value(size_init));
        let vault_path_state = cx.new(|cx| InputState::new(window, cx).default_value(vault_init));

        let theme_dark = matches!(current.theme, LimnTheme::Dark);

        // Subscribe to each input's Change so the draft mirrors what the
        // user typed. The closures hold the same shape as the editor's
        // input subscription (Wave 3 pattern): pull the latest text from
        // the entity in the handler, write it into the draft.
        let family_sub = cx.subscribe_in(
            &font_family_state,
            window,
            |this: &mut Self, state, event, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.font.family = state.read(cx).value().to_string();
                }
            },
        );
        let size_sub = cx.subscribe_in(
            &font_size_state,
            window,
            |this: &mut Self, state, event, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    // Parse eagerly so the draft holds the canonical u16;
                    // empty / invalid input keeps the previous size (and
                    // is reported again at Save time, see [`Self::save`]).
                    if let Ok(size) = state.read(cx).value().parse::<u16>() {
                        this.draft.font.size = size;
                    }
                }
            },
        );
        let vault_sub = cx.subscribe_in(
            &vault_path_state,
            window,
            |this: &mut Self, state, event, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    let raw = state.read(cx).value().to_string();
                    this.draft.vault_path = if raw.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(raw))
                    };
                }
            },
        );

        Self {
            focus_handle: cx.focus_handle(),
            draft: current,
            font_family_state,
            font_size_state,
            vault_path_state,
            theme_dark,
            _subscriptions: vec![family_sub, size_sub, vault_sub],
        }
    }

    /// Focus the view so the `"Settings"` key context is on the dispatch
    /// tree (and Esc → [`CloseSettings`] resolves here).
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        // Re-render so any focus-sensitive styling refreshes.
        cx.notify();
    }

    /// Read the draft back. Exposed so the AppShell-level test can assert
    /// what the Save handler is about to commit without depending on
    /// global state.
    #[must_use]
    pub fn draft(&self) -> &LimnConfig {
        &self.draft
    }

    /// Write the underlying input buffers directly. Tests only — the
    /// production path drives the inputs via keyboard events, which is
    /// out of reach without a real event loop. `InputState::set_value`
    /// deliberately suppresses `Change` events (it sets
    /// `emit_events = false`), so the `InputEvent::Change` subscriptions
    /// installed in [`Self::new`] do **not** fire from this call; the
    /// draft is mirrored from the arguments below so it stays in sync
    /// with the visible buffers without depending on subscription
    /// delivery.
    ///
    /// Intentionally not behind `#[cfg(test)]` so integration tests
    /// (a separate compilation unit) can call it; the `set_test_`
    /// prefix and `#[doc(hidden)]` mark it as a test affordance.
    /// Production code has no reason to call this.
    #[doc(hidden)]
    pub fn set_test_inputs(
        &mut self,
        font_family: &str,
        font_size: &str,
        vault_path: &str,
        theme_dark: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let family: SharedString = font_family.to_string().into();
        let size: SharedString = font_size.to_string().into();
        let vault: SharedString = vault_path.to_string().into();
        self.font_family_state
            .update(cx, |s, cx| s.set_value(family, window, cx));
        self.font_size_state
            .update(cx, |s, cx| s.set_value(size, window, cx));
        self.vault_path_state
            .update(cx, |s, cx| s.set_value(vault, window, cx));
        self.theme_dark = theme_dark;
        // set_value deliberately suppresses Change events (ADR-0009 buffer
        // swap pattern), so the draft would not be refreshed by the
        // subscriptions. Mirror the writes into the draft directly.
        self.draft.font.family = font_family.to_string();
        if let Ok(s) = font_size.parse::<u16>() {
            self.draft.font.size = s;
        }
        self.draft.vault_path = if vault_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(vault_path))
        };
        self.draft.theme = if theme_dark {
            LimnTheme::Dark
        } else {
            LimnTheme::Light
        };
    }

    /// Validate the font size text and re-pull all three text inputs into
    /// the draft, then return the resolved draft.
    ///
    /// Subscriptions handle the common case, but a user can hit Save
    /// without their last keystroke having committed (e.g. focus moves
    /// between the keydown and the click). Pulling each input here is the
    /// simple, race-free way to capture the final state before write.
    fn finalize_draft(&mut self, cx: &mut Context<Self>) -> Option<LimnConfig> {
        self.draft.font.family = self.font_family_state.read(cx).value().to_string();

        let size_text = self.font_size_state.read(cx).value();
        let Ok(size) = size_text.parse::<u16>() else {
            eprintln!("limn-ui: settings save skipped: invalid font size {size_text:?}");
            return None;
        };
        self.draft.font.size = size;

        let vault_text = self.vault_path_state.read(cx).value().to_string();
        self.draft.vault_path = if vault_text.is_empty() {
            None
        } else {
            Some(PathBuf::from(vault_text))
        };

        self.draft.theme = if self.theme_dark {
            LimnTheme::Dark
        } else {
            LimnTheme::Light
        };

        Some(self.draft.clone())
    }

    /// Save the draft to disk and reflect it in the live [`AppConfig`].
    ///
    /// The write goes through `LimnConfig::save_to` on the background
    /// executor (ADR-0007 atomic rename pattern, ADR-0002 keeps `std::fs`
    /// in limn-service). On success we update the global so the rest of
    /// the running session sees the new values; on failure we just log
    /// (UAT-simplified per the design — surfacing a toast is Wave 9+).
    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.finalize_draft(cx) else {
            return;
        };

        // Resolve the canonical path on the main thread (uses `dirs`) so
        // the background task only carries the resolved PathBuf.
        let Some(path) = LimnConfig::config_path() else {
            eprintln!("limn-ui: settings save skipped: cannot resolve config path");
            return;
        };

        let to_write = draft.clone();
        let write_task = cx.background_spawn(async move { to_write.save_to(&path) });

        cx.spawn_in(window, async move |this, cx| match write_task.await {
            Ok(()) => {
                if let Err(e) = cx.update(|window, cx| {
                    cx.set_global(AppConfig(draft.clone()));
                    // Wave 10-D: sync the Limn-side color theme so EditorView /
                    // DocumentView pick up the change next render (ADR-0011).
                    cx.set_global(ColorThemeGlobal(ColorTheme::from_config(draft.theme)));
                    // Wave 9: apply the saved theme/font to gpui-component's
                    // Theme global so the running session re-renders. `window`
                    // is available here, so pass `Some(window)` to fire
                    // `window.refresh` inside `Theme::change`. Font fields are
                    // written AFTER `change` (which runs `apply_config` and
                    // resets them), so the override wins.
                    let mode = match draft.theme {
                        LimnTheme::Dark => ThemeMode::Dark,
                        LimnTheme::Light => ThemeMode::Light,
                    };
                    GpuiTheme::change(mode, Some(window), cx);
                    if !draft.font.family.is_empty() {
                        GpuiTheme::global_mut(cx).font_family = draft.font.family.clone().into();
                    }
                    if draft.font.size > 0 {
                        GpuiTheme::global_mut(cx).font_size = px(f32::from(draft.font.size));
                    }
                }) {
                    eprintln!("limn-ui: settings save: failed to update global: {e}");
                    return;
                }
                if let Err(e) = this.update(cx, |view, cx| {
                    view.draft = draft;
                    cx.notify();
                }) {
                    eprintln!("limn-ui: settings save: failed to refresh view: {e}");
                }
                eprintln!("limn-ui: settings saved");
            }
            Err(e) => eprintln!("limn-ui: settings save failed: {e}"),
        })
        .detach();
    }

    /// Save the draft synchronously to an explicit path and update the
    /// live [`AppConfig`].
    ///
    /// Same effect as [`Self::save`], but inline rather than dispatched
    /// to the background executor, and writing to the path the caller
    /// supplies rather than [`LimnConfig::config_path`]. Used by the
    /// integration test, which points it at a `tempfile`; not on the
    /// production path (the GUI uses `save` to keep the main thread free
    /// and to land on the canonical config path).
    ///
    /// Intentionally not `#[cfg(test)]`: integration tests are a separate
    /// compilation unit and so cannot see test-cfg items. `#[doc(hidden)]`
    /// flags this as not part of the supported public surface.
    #[doc(hidden)]
    pub fn save_to_path(&mut self, path: &std::path::Path, cx: &mut Context<Self>) -> bool {
        let Some(draft) = self.finalize_draft(cx) else {
            return false;
        };
        match draft.save_to(path) {
            Ok(()) => {
                cx.set_global(AppConfig(draft.clone()));
                // Wave 10-D: sync the Limn-side color theme so EditorView /
                // DocumentView pick up the change next render (ADR-0011).
                cx.set_global(ColorThemeGlobal(ColorTheme::from_config(draft.theme)));
                // Wave 9: apply theme/font to the gpui-component Theme global
                // (sync test path). No `&mut Window` is in scope here, so we
                // pass `None` to `Theme::change` and drive the redraw with
                // `cx.refresh_windows()` instead — `Context<Self>` reaches the
                // `App` API via `Deref`, so refresh IS callable. This keeps the
                // path semantically equivalent to the async `save()` above,
                // where `Some(window)` lets `Theme::change` refresh internally.
                //
                // Font fields are written AFTER `Theme::change`: `change` runs
                // `apply_config`, which resets font_family/font_size, so the
                // override must come last to win.
                let mode = match draft.theme {
                    LimnTheme::Dark => ThemeMode::Dark,
                    LimnTheme::Light => ThemeMode::Light,
                };
                GpuiTheme::change(mode, None, cx);
                if !draft.font.family.is_empty() {
                    GpuiTheme::global_mut(cx).font_family = draft.font.family.clone().into();
                }
                if draft.font.size > 0 {
                    GpuiTheme::global_mut(cx).font_size = px(f32::from(draft.font.size));
                }
                cx.refresh_windows();
                self.draft = draft;
                cx.notify();
                true
            }
            Err(e) => {
                eprintln!("limn-ui: settings save_to_path failed: {e}");
                false
            }
        }
    }
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().background;
        let fg = cx.theme().foreground;

        div()
            // Track focus so the `"Settings"` context is on the dispatch
            // tree; `escape` is bound to `CloseSettings` in that context
            // (see `actions::bind_keys`).
            .track_focus(&self.focus_handle)
            .key_context(SETTINGS_CONTEXT)
            .size_full()
            .bg(bg)
            .text_color(fg)
            .p_8()
            .flex()
            .flex_col()
            .gap_4()
            .child(div().text_xs().opacity(0.5).child("Limn — Settings"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Label::new("Font family"))
                    .child(Input::new(&self.font_family_state)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Label::new("Font size"))
                    .child(Input::new(&self.font_size_state)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Label::new("Vault path"))
                    .child(Input::new(&self.vault_path_state)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(Label::new("Dark theme"))
                    .child(
                        Switch::new("settings-theme-switch")
                            .checked(self.theme_dark)
                            .on_click(cx.listener(
                                |this: &mut Self, checked: &bool, _window, cx| {
                                    this.theme_dark = *checked;
                                    this.draft.theme = if *checked {
                                        LimnTheme::Dark
                                    } else {
                                        LimnTheme::Light
                                    };
                                    cx.notify();
                                },
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .mt_4()
                    .child(
                        Button::new("settings-save")
                            .label("Save")
                            .on_click(cx.listener(|this: &mut Self, _, window, cx| {
                                this.save(window, cx);
                            })),
                    )
                    .child(
                        // Mirrors the "Back to editor" affordance the
                        // design calls for. Clicking dispatches the same
                        // CloseSettings action that Esc fires, so all
                        // three close routes converge on one handler.
                        Button::new("settings-close")
                            .label("← Back to editor")
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(CloseSettings), cx);
                            }),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .opacity(0.5)
                    .mt_2()
                    .child("Press Esc or click Back to return to the editor."),
            )
    }
}
