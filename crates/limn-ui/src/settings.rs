//! Settings view — a separate, switchable surface for editing user
//! preferences held in [`Config`].
//!
//! The view owns a working copy of the [`Config`]. The theme control
//! mutates that copy in place; Save writes it to disk via
//! [`Config::save`]. Font family, font size, and vault path render as
//! their current values (see [`SettingsView`] for the editing surface
//! each control exposes).

use gpui::{
    div, px, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window,
};

use limn_service::{Config, Theme};

/// The outcome of the most recent Save, surfaced to the user as a
/// status line under the Save control.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SaveStatus {
    /// No Save has run since the view opened or since the last edit.
    #[default]
    Idle,
    /// The working config was written to disk.
    Saved,
    /// Writing the working config failed; the message describes why.
    Error(SharedString),
}

/// Editable settings surface.
///
/// Holds a working copy of [`Config`] that the controls mutate.
/// Editability per item:
///
/// - **Theme** — editable. A clickable control flips
///   [`Theme::Light`]/[`Theme::Dark`] via [`SettingsView::toggle_theme`].
/// - **Font family, font size, vault path** — display-only. They render
///   their current values. Text entry for these needs a focused input
///   handler that this gpui rev does not expose through a lightweight
///   API, so the view shows them read-only and says so in the form.
/// - **Save** — writes the working copy to the real config path via
///   [`Config::save`] and records the result in [`SettingsView::status`].
pub struct SettingsView {
    /// The working copy edited by the controls and written on Save.
    pub config: Config,
    /// Result of the most recent Save.
    pub status: SaveStatus,
}

impl SettingsView {
    /// Build a settings view over a working copy of `config`.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            status: SaveStatus::Idle,
        }
    }

    /// Flip the working theme between light and dark.
    ///
    /// Editing the config invalidates any prior Save result, so the
    /// status returns to [`SaveStatus::Idle`].
    pub fn toggle_theme(&mut self) {
        self.config.theme = match self.config.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        self.status = SaveStatus::Idle;
    }

    /// Write the working copy to the real user config path and record
    /// the outcome in [`SettingsView::status`].
    ///
    /// A failed write is captured as [`SaveStatus::Error`] rather than
    /// panicking, so the view can display it.
    pub fn save(&mut self) {
        self.status = match self.config.save() {
            Ok(()) => SaveStatus::Saved,
            Err(e) => SaveStatus::Error(e.to_string().into()),
        };
    }

    /// The label shown on the theme control for the working theme.
    fn theme_label(&self) -> &'static str {
        match self.config.theme {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (bg, fg) = match self.config.theme {
            Theme::Light => (rgb(0x00fa_f9f6), rgb(0x001a_1a1a)),
            Theme::Dark => (rgb(0x001a_1a1a), rgb(0x00f0_f0f0)),
        };
        let accent = rgb(0x0037_6fd6);
        let muted = rgb(0x0080_8080);

        let font_size = format!("{}", self.config.font_size);
        let vault = self
            .config
            .vault_path
            .as_ref()
            .map_or_else(|| "(none)".to_string(), |p| p.display().to_string());

        let status_row = match &self.status {
            SaveStatus::Idle => div(),
            SaveStatus::Saved => div().text_color(accent).child("Saved"),
            SaveStatus::Error(msg) => div()
                .text_color(rgb(0x00d0_4437))
                .child(format!("Save failed: {msg}")),
        };

        div()
            .size_full()
            .bg(bg)
            .text_color(fg)
            .font_family(self.config.font_family.clone())
            .text_size(px(self.config.font_size))
            .p_8()
            .flex()
            .flex_col()
            .gap_4()
            .child(div().text_xl().child("Settings"))
            // Theme — editable via the toggle below.
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xs().opacity(0.7).child("Theme"))
                    .child(
                        div()
                            .id("theme-toggle")
                            .px_3()
                            .py_1()
                            .bg(accent)
                            .text_color(rgb(0x00ff_ffff))
                            .cursor_pointer()
                            .child(format!("{} (click to toggle)", self.theme_label()))
                            .on_click(cx.listener(|view, _event, _window, cx| {
                                view.toggle_theme();
                                cx.notify();
                            })),
                    ),
            )
            // Font family — display-only at this surface.
            .child(setting_row(
                "Font family",
                self.config.font_family.clone(),
                muted,
            ))
            // Font size — display-only at this surface.
            .child(setting_row("Font size", font_size, muted))
            // Vault path — display-only at this surface.
            .child(setting_row("Vault path", vault, muted))
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("Font and vault path are read-only here; edit them in config.toml."),
            )
            // Save — writes the working copy to the real config path.
            .child(
                div()
                    .id("save-button")
                    .px_3()
                    .py_1()
                    .bg(fg)
                    .text_color(bg)
                    .cursor_pointer()
                    .child("Save")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _event, _window, cx| {
                            view.save();
                            cx.notify();
                        }),
                    ),
            )
            .child(status_row)
    }
}

/// A label-over-value row for a display-only setting.
fn setting_row(
    label: &'static str,
    value: impl Into<SharedString>,
    label_color: gpui::Rgba,
) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(label_color).child(label))
        .child(div().child(value.into()))
}
