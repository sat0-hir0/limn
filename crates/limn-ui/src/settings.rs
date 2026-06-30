//! Settings view (Wave 8) — a separate screen for editing the user
//! configuration that ships in [`limn_service::LimnConfig`].
//!
//! Concretely: three text inputs (font family / font size / vault path)
//! and a theme switch. Every change auto-applies through
//! `LimnConfig::save_to` (ADR-0002: limn-ui never calls `std::fs`).
//!
//! The view holds a `draft` copy of the config that the inputs mutate
//! independently of the live [`crate::AppConfig`] global. Wave 11 makes
//! the apply path auto-fire: theme toggles apply on click, text fields
//! debounce 500 ms then apply, the vault path validates on Enter. There
//! is no Save button.
//!
//! Wave 8 scope: this just persists the values. Applying `font` / `theme`
//! to rendering is deliberately deferred — the change still lands on
//! disk and the next launch picks it up, which is enough to demo the
//! settings UI end-to-end (ADR-0010).

use std::path::PathBuf;
use std::time::Duration;

use gpui::{
    div, px, App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Subscription, Task, Window,
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
use crate::{AppConfig, ColorPalette, ColorTheme, ColorThemeGlobal};

/// How long to wait after the last keystroke in a text input before
/// applying the change to disk and globals. Long enough that continuous
/// typing collapses into one apply, short enough that a pause feels
/// like "it saved" (matches the editor autosave debounce intent).
const SETTINGS_APPLY_DEBOUNCE: Duration = Duration::from_millis(500);

/// Editable settings screen backed by a `draft` copy of [`LimnConfig`].
///
/// The three text inputs each own an [`InputState`]; their `Change` events
/// are subscribed in `new` and update `draft` in place. Wave 11: every
/// applicable change auto-applies — theme toggle commits immediately,
/// text fields debounce 500 ms then apply, the vault path validates on
/// Enter only (VS Code pattern). Each debounced apply spawns a `Task<()>`
/// into one of the `pending_*_apply` slots; replacing the slot drops the
/// previous task and cancels its still-sleeping timer.
pub struct SettingsView {
    focus_handle: FocusHandle,
    /// In-memory edit buffer. Mirrors every keystroke (font fields and the
    /// raw vault-path string); the apply path validates and commits it to
    /// disk + globals.
    draft: LimnConfig,
    font_family_state: Entity<InputState>,
    font_size_state: Entity<InputState>,
    vault_path_state: Entity<InputState>,
    /// Mirrored bool because [`Theme`] is a two-variant enum we can show as
    /// a switch; tracking it here keeps the render path simple.
    theme_dark: bool,
    /// Holds the input-event subscriptions; dropping the view unsubscribes.
    _subscriptions: Vec<Subscription>,
    /// Pending debounced apply for the font-family field. Replacing the
    /// slot cancels the previous timer (same pattern as
    /// [`crate::EditorView::pending_save`]).
    pending_font_family_apply: Task<()>,
    /// Pending debounced apply for the font-size field.
    pending_font_size_apply: Task<()>,
    /// Pending debounced apply for the vault-path field (armed only after
    /// an Enter-press has validated the path).
    pending_vault_path_apply: Task<()>,
    /// Set when the font-size input fails to parse to a non-zero `u16`;
    /// drives a red-border cue on the input.
    font_size_invalid: bool,
    /// Set when the vault-path input failed validation on Enter (path
    /// does not exist on disk); drives a red-border cue.
    vault_path_invalid: bool,
}

impl SettingsView {
    /// Build the settings view from the current [`AppConfig`] global.
    ///
    /// The draft is initialized from the global so opening settings shows
    /// the values the rest of the app is using. Subsequent edits diverge
    /// from the global until the auto-apply path commits them back
    /// (Wave 11: theme on click, text fields on a 500 ms debounce, vault
    /// path on Enter).
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

        // Subscribe to each input's events. Wave 11 wires each subscription
        // to the auto-apply path:
        //   - Font family: on Change → update draft → debounced apply.
        //   - Font size: on Change → parse → invalid sets a red border and
        //     skips the apply; valid clears the border and arms the debounce.
        //   - Vault path: on Change → mirror raw string into draft (no apply
        //     yet). On PressEnter → validate existence; apply only if it
        //     passes (VS Code pattern: last valid value stays in effect when
        //     the user is mid-typing or types a missing path).
        let family_sub = cx.subscribe_in(
            &font_family_state,
            window,
            |this: &mut Self, state, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.font.family = state.read(cx).value().to_string();
                    this.schedule_font_family_apply(window, cx);
                }
            },
        );
        let size_sub = cx.subscribe_in(
            &font_size_state,
            window,
            |this: &mut Self, state, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    let raw = state.read(cx).value();
                    match raw.parse::<u16>() {
                        Ok(n) if n > 0 => {
                            this.draft.font.size = n;
                            this.font_size_invalid = false;
                            this.schedule_font_size_apply(window, cx);
                        }
                        _ => {
                            // Last valid size in `self.draft.font.size`
                            // stays in effect; flag the input for a red
                            // border and skip the apply.
                            this.font_size_invalid = true;
                            cx.notify();
                        }
                    }
                }
            },
        );
        let vault_sub = cx.subscribe_in(
            &vault_path_state,
            window,
            |this: &mut Self, state, event, window, cx| match event {
                InputEvent::Change => {
                    // Mirror the raw string into the draft so the value is
                    // available if/when validation passes. Apply is NOT
                    // scheduled here — invalid mid-typing paths must not
                    // touch disk (VS Code pattern).
                    let raw = state.read(cx).value().to_string();
                    this.draft.vault_path = if raw.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(raw))
                    };
                }
                InputEvent::PressEnter { .. } => {
                    let exists = match &this.draft.vault_path {
                        None => true, // empty path is valid (`None`)
                        Some(p) => p.as_os_str().is_empty() || p.exists(),
                    };
                    if exists {
                        this.vault_path_invalid = false;
                        this.schedule_vault_path_apply(window, cx);
                    } else {
                        this.vault_path_invalid = true;
                        cx.notify();
                    }
                }
                _ => {}
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
            // No edit yet, so nothing pending. A no-op task is the
            // simplest "empty" value for each debounce slot (mirrors
            // `EditorView::pending_save`'s initialization).
            pending_font_family_apply: Task::ready(()),
            pending_font_size_apply: Task::ready(()),
            pending_vault_path_apply: Task::ready(()),
            font_size_invalid: false,
            vault_path_invalid: false,
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
    /// what the auto-apply path is about to commit without depending on
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

    /// Save the draft synchronously to an explicit path and update the
    /// live [`AppConfig`].
    ///
    /// Same effect as [`Self::apply_to_disk_and_globals`], but inline
    /// rather than dispatched to the background executor, and writing to
    /// the path the caller supplies rather than
    /// [`LimnConfig::config_path`]. Used by the integration tests, which
    /// point it at a `tempfile`; not on the production path (the GUI
    /// uses the auto-apply chain to keep the main thread free and to
    /// land on the canonical config path).
    ///
    /// Intentionally not `#[cfg(test)]`: integration tests are a separate
    /// compilation unit and so cannot see test-cfg items. `#[doc(hidden)]`
    /// flags this as not part of the supported public surface.
    #[doc(hidden)]
    pub fn save_to_path(&mut self, path: &std::path::Path, cx: &mut Context<Self>) -> bool {
        // Wave 11: validation gates — these mirror the rules the
        // production auto-apply path enforces (invalid font size or
        // missing vault path is rejected, last valid value wins).
        //
        // Re-validate the live input text so the test path applies the
        // same "input must parse" rule the production Change-event
        // subscription uses. If the text is unparseable (or zero), bail
        // out without touching disk or globals; the last-valid value in
        // `self.draft.font.size` stays in effect.
        let size_text = self.font_size_state.read(cx).value();
        match size_text.parse::<u16>() {
            Ok(n) if n > 0 => self.draft.font.size = n,
            _ => {
                eprintln!("limn-ui: invalid font size, skipping save_to_path");
                return false;
            }
        }

        // Re-pull the remaining inputs into the draft. Family is any
        // string; vault path is validated below before commit.
        self.draft.font.family = self.font_family_state.read(cx).value().to_string();
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

        if let Some(vault) = &self.draft.vault_path {
            if !vault.as_os_str().is_empty() && !vault.exists() {
                eprintln!("limn-ui: vault_path does not exist, skipping save_to_path");
                return false;
            }
        }

        let draft = self.draft.clone();
        match draft.save_to(path) {
            Ok(()) => {
                Self::commit_globals_sync(&draft, cx);
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

    /// Apply `draft` to the three globals — [`AppConfig`],
    /// [`ColorThemeGlobal`], and the gpui-component [`GpuiTheme`] — without
    /// touching a `Window`. Used by the sync test path and as the
    /// non-`Window` half of [`Self::apply_to_disk_and_globals`].
    ///
    /// Font fields are written AFTER `GpuiTheme::change` because `change`
    /// runs `apply_config`, which resets `font_family` / `font_size`; the
    /// override must come last to win. `cx.refresh_windows()` drives the
    /// redraw in lieu of the `Some(window)` refresh `change` would do
    /// internally if a window were in scope.
    fn commit_globals_sync(draft: &LimnConfig, cx: &mut Context<Self>) {
        cx.set_global(AppConfig(draft.clone()));
        // Wave 10-D: sync the Limn-side color theme so EditorView /
        // DocumentView pick up the change next render (ADR-0011).
        cx.set_global(ColorThemeGlobal(ColorTheme::from_config(draft.theme)));
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
    }

    /// Wave 11 production apply path. Snapshots the current `draft`,
    /// resolves the canonical config path, spawns the background write
    /// (ADR-0007 atomic rename), and on `Ok(())` updates the three
    /// globals + the view's own draft mirror.
    ///
    /// Mirrors the old `save()` exactly, but the `Some(window)` variant
    /// of [`GpuiTheme::change`] is used so the running window refreshes
    /// without waiting for the next paint. Failures log to stderr; a
    /// toast surface is deferred to a later wave.
    pub fn apply_to_disk_and_globals(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let draft = self.draft.clone();
        let Some(path) = LimnConfig::config_path() else {
            eprintln!("limn-ui: settings auto-apply skipped: cannot resolve config path");
            return;
        };

        let to_write = draft.clone();
        let write_task = cx.background_spawn(async move { to_write.save_to(&path) });

        cx.spawn_in(window, async move |this, cx| match write_task.await {
            Ok(()) => {
                if let Err(e) = cx.update(|window, cx| {
                    cx.set_global(AppConfig(draft.clone()));
                    cx.set_global(ColorThemeGlobal(ColorTheme::from_config(draft.theme)));
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
                    eprintln!("limn-ui: settings auto-apply: failed to update global: {e}");
                    return;
                }
                if let Err(e) = this.update(cx, |view, cx| {
                    view.draft = draft;
                    cx.notify();
                }) {
                    eprintln!("limn-ui: settings auto-apply: failed to refresh view: {e}");
                }
                eprintln!("limn-ui: settings auto-applied");
            }
            Err(e) => eprintln!("limn-ui: settings auto-apply failed: {e}"),
        })
        .detach();
    }

    /// Arm (or re-arm) the debounced font-family apply. Replacing
    /// `pending_font_family_apply` drops the previous task and cancels
    /// its still-sleeping timer (same pattern as
    /// [`crate::EditorView::schedule_save`]).
    fn schedule_font_family_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_font_family_apply = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(SETTINGS_APPLY_DEBOUNCE)
                .await;
            let _ = this.update_in(cx, |view, window, cx| {
                view.apply_to_disk_and_globals(window, cx);
            });
        });
    }

    /// Arm (or re-arm) the debounced font-size apply.
    fn schedule_font_size_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_font_size_apply = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(SETTINGS_APPLY_DEBOUNCE)
                .await;
            let _ = this.update_in(cx, |view, window, cx| {
                view.apply_to_disk_and_globals(window, cx);
            });
        });
    }

    /// Arm (or re-arm) the debounced vault-path apply. Only called from
    /// the Enter handler, after the path has already passed the
    /// `exists()` check.
    fn schedule_vault_path_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_vault_path_apply = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(SETTINGS_APPLY_DEBOUNCE)
                .await;
            let _ = this.update_in(cx, |view, window, cx| {
                view.apply_to_disk_and_globals(window, cx);
            });
        });
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
                    .child({
                        let mut wrap = div().child(Input::new(&self.font_size_state));
                        if self.font_size_invalid {
                            // Wave 11: red-border cue when the live input
                            // text fails to parse. The draft retains its
                            // last valid value; this is the visual signal
                            // that the typed value did not reach disk.
                            wrap = wrap
                                .border_1()
                                .border_color(ColorPalette::red_500())
                                .rounded(cx.theme().radius);
                        }
                        wrap
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Label::new("Vault path"))
                    .child({
                        let mut wrap = div().child(Input::new(&self.vault_path_state));
                        if self.vault_path_invalid {
                            // Wave 11: red-border cue when the typed path
                            // failed the existence check on Enter. The
                            // draft retains its last valid value.
                            wrap = wrap
                                .border_1()
                                .border_color(ColorPalette::red_500())
                                .rounded(cx.theme().radius);
                        }
                        wrap
                    }),
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
                                |this: &mut Self, checked: &bool, window, cx| {
                                    this.theme_dark = *checked;
                                    this.draft.theme = if *checked {
                                        LimnTheme::Dark
                                    } else {
                                        LimnTheme::Light
                                    };
                                    // Wave 11: theme commits + applies
                                    // immediately on click. No debounce —
                                    // a toggle is a discrete decision, and
                                    // a 500 ms lag would feel wrong here.
                                    this.apply_to_disk_and_globals(window, cx);
                                    cx.notify();
                                },
                            )),
                    ),
            )
            .child(
                div().flex().flex_row().gap_2().mt_4().child(
                    // Wave 11: the Save button is gone. Every change
                    // auto-applies (theme on click, text fields on a
                    // 500 ms debounce, vault path on Enter). The
                    // "Back to editor" affordance is the only button
                    // left; clicking dispatches the same
                    // CloseSettings action that Esc fires, so all
                    // three close routes converge on one handler.
                    Button::new("settings-close")
                        .label("← Back to editor")
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(CloseSettings), cx);
                        }),
                ),
            )
            .child(div().text_xs().opacity(0.5).mt_2().child(
                "Changes apply automatically. Press Esc or click ← Back to return to the editor.",
            ))
    }
}
