//! Settings view — a separate, switchable surface for editing user
//! preferences held in [`Config`].
//!
//! The view owns a working copy of the [`Config`]. The controls mutate
//! that copy in place; Save writes it to disk via [`Config::save`].
//! See [`SettingsView`] for the editing surface each control exposes.

use std::path::PathBuf;

use gpui::{
    div, px, rgb, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, Keystroke,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

use limn_service::{Config, Theme};

/// Smallest font size the stepper allows, in points.
pub const MIN_FONT_SIZE: f32 = 8.0;
/// Largest font size the stepper allows, in points.
pub const MAX_FONT_SIZE: f32 = 72.0;

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

/// A text setting the user can type into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextField {
    /// [`Config::font_family`].
    FontFamily,
    /// [`Config::vault_path`]. An empty value means no vault.
    VaultPath,
}

impl TextField {
    /// The other text field, for moving between fields with Tab.
    fn next(self) -> Self {
        match self {
            Self::FontFamily => Self::VaultPath,
            Self::VaultPath => Self::FontFamily,
        }
    }
}

/// Editable settings surface.
///
/// Holds a working copy of [`Config`] that the controls mutate.
/// Editability per item:
///
/// - **Theme** — a clickable control flips
///   [`Theme::Light`]/[`Theme::Dark`] via [`SettingsView::toggle_theme`].
/// - **Font family, vault path** — click the field, then type. Keys
///   append at the end of the value and Backspace removes the last
///   character; there is no cursor movement or selection. Typed text
///   arrives as key presses, so IME composition (e.g. Japanese) is not
///   received; paste such text with Ctrl+V (Cmd+V on macOS). Tab moves
///   to the other field, Enter or Escape stops editing.
/// - **Font size** — `−` / `+` step it by one point within
///   [`MIN_FONT_SIZE`]..=[`MAX_FONT_SIZE`].
/// - **Save** — writes the working copy to the real config path via
///   [`Config::save`] and records the result in [`SettingsView::status`].
pub struct SettingsView {
    /// The working copy edited by the controls and written on Save.
    pub config: Config,
    /// Result of the most recent Save.
    pub status: SaveStatus,
    /// The config as last written by a successful Save, if any. The
    /// shell hands this to the document view on switching back.
    pub last_saved: Option<Config>,
    /// The text field receiving typed keys, if any.
    pub editing: Option<TextField>,
    /// Focus target for key presses, created on first render.
    focus_handle: Option<FocusHandle>,
}

impl SettingsView {
    /// Build a settings view over a working copy of `config`.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            status: SaveStatus::Idle,
            last_saved: None,
            editing: None,
            focus_handle: None,
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

    /// Change the working font size by `delta` points, clamped to
    /// [`MIN_FONT_SIZE`]..=[`MAX_FONT_SIZE`].
    pub fn step_font_size(&mut self, delta: f32) {
        self.config.font_size = (self.config.font_size + delta).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
        self.status = SaveStatus::Idle;
    }

    /// The current text of `field` in the working config.
    #[must_use]
    pub fn field_text(&self, field: TextField) -> String {
        match field {
            TextField::FontFamily => self.config.font_family.clone(),
            TextField::VaultPath => self
                .config
                .vault_path
                .as_ref()
                .map_or_else(String::new, |p| p.to_string_lossy().into_owned()),
        }
    }

    /// Replace the text of `field` in the working config. An empty vault
    /// path clears the vault.
    fn set_field_text(&mut self, field: TextField, text: String) {
        match field {
            TextField::FontFamily => self.config.font_family = text,
            TextField::VaultPath => {
                self.config.vault_path = (!text.is_empty()).then(|| PathBuf::from(text));
            }
        }
        self.status = SaveStatus::Idle;
    }

    /// Append `text` to the field being edited, dropping control
    /// characters (line breaks from a pasted value included). Does
    /// nothing when no field is being edited.
    pub fn insert_text(&mut self, text: &str) {
        let Some(field) = self.editing else { return };
        let mut value = self.field_text(field);
        value.extend(text.chars().filter(|c| !c.is_control()));
        self.set_field_text(field, value);
    }

    /// Remove the last character of the field being edited.
    pub fn backspace(&mut self) {
        let Some(field) = self.editing else { return };
        let mut value = self.field_text(field);
        value.pop();
        self.set_field_text(field, value);
    }

    /// Apply one key press to the field being edited.
    ///
    /// `paste` is called only for the paste shortcut and yields the
    /// clipboard text. Returns whether the key was consumed.
    pub fn handle_keystroke(
        &mut self,
        keystroke: &Keystroke,
        paste: impl FnOnce() -> Option<String>,
    ) -> bool {
        let Some(field) = self.editing else {
            return false;
        };
        let modifiers = &keystroke.modifiers;
        match keystroke.key.as_str() {
            "backspace" => self.backspace(),
            "enter" | "escape" => self.editing = None,
            "tab" => self.editing = Some(field.next()),
            "v" if modifiers.secondary() => {
                if let Some(text) = paste() {
                    self.insert_text(&text);
                }
            }
            _ if modifiers.control || modifiers.platform || modifiers.function => return false,
            _ => match &keystroke.key_char {
                Some(text) => self.insert_text(text),
                None => return false,
            },
        }
        true
    }

    /// Write the working copy to the real user config path and record
    /// the outcome in [`SettingsView::status`].
    ///
    /// A failed write is captured as [`SaveStatus::Error`] rather than
    /// panicking, so the view can display it.
    pub fn save(&mut self) {
        self.status = match self.config.save() {
            Ok(()) => {
                self.last_saved = Some(self.config.clone());
                SaveStatus::Saved
            }
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

    /// The control that flips the working theme.
    fn theme_toggle(&self, accent: gpui::Rgba, cx: &mut Context<Self>) -> impl IntoElement {
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
            }))
    }

    /// The working font size between `−` / `+` steppers.
    fn font_size_control(&self, border: gpui::Rgba, cx: &mut Context<Self>) -> impl IntoElement {
        let stepper = |id: &'static str, label: &'static str, delta: f32| {
            div()
                .id(id)
                .px_2()
                .border_1()
                .border_color(border)
                .cursor_pointer()
                .child(label)
                .on_click(cx.listener(move |view, _event, _window, cx| {
                    view.step_font_size(delta);
                    cx.notify();
                }))
        };
        let down = stepper("font-size-down", "−", -1.0);
        let up = stepper("font-size-up", "+", 1.0);
        div()
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(down)
            .child(format!("{}", self.config.font_size))
            .child(up)
    }

    /// A clickable text field showing the value of `field`, with a caret
    /// and accent border while it is being edited.
    fn text_field(
        &self,
        field: TextField,
        id: &'static str,
        placeholder: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let editing = self.editing == Some(field);
        let value = self.field_text(field);
        let shown = match (value.is_empty(), editing) {
            (true, false) => placeholder.to_string(),
            (_, true) => format!("{value}▏"),
            (false, false) => value,
        };
        let border = if editing {
            rgb(0x0037_6fd6)
        } else {
            rgb(0x0080_8080)
        };
        div()
            .id(id)
            .px_2()
            .py_1()
            .border_1()
            .border_color(border)
            .cursor_text()
            .child(shown)
            .on_click(cx.listener(move |view, _event, window, cx| {
                view.start_editing(field, window, cx);
            }))
    }

    /// The view's focus target for key presses, created on first use.
    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle())
            .clone()
    }

    /// Start editing `field` and take keyboard focus so typed keys reach
    /// it. Clicking a field does this.
    pub fn start_editing(&mut self, field: TextField, window: &mut Window, cx: &mut Context<Self>) {
        self.editing = Some(field);
        let focus_handle = self.focus_handle(cx);
        window.focus(&focus_handle, cx);
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        let (bg, fg) = match self.config.theme {
            Theme::Light => (rgb(0x00fa_f9f6), rgb(0x001a_1a1a)),
            Theme::Dark => (rgb(0x001a_1a1a), rgb(0x00f0_f0f0)),
        };
        let accent = rgb(0x0037_6fd6);
        let muted = rgb(0x0080_8080);

        let status_row = match &self.status {
            SaveStatus::Idle => div(),
            SaveStatus::Saved => div().text_color(accent).child("Saved"),
            SaveStatus::Error(msg) => div()
                .text_color(rgb(0x00d0_4437))
                .child(format!("Save failed: {msg}")),
        };

        div()
            .id("settings")
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, _window, cx| {
                let consumed = view.handle_keystroke(&event.keystroke, || {
                    cx.read_from_clipboard().and_then(|item| item.text())
                });
                if consumed {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
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
            .child(labeled("Theme", muted, self.theme_toggle(accent, cx)))
            .child(labeled(
                "Font family",
                muted,
                self.text_field(TextField::FontFamily, "font-family-field", "(empty)", cx),
            ))
            .child(labeled(
                "Font size",
                muted,
                self.font_size_control(muted, cx),
            ))
            .child(labeled(
                "Vault path",
                muted,
                self.text_field(TextField::VaultPath, "vault-path-field", "(none)", cx),
            ))
            .child(div().text_xs().text_color(muted).child(
                "Click a field and type. Ctrl+V pastes (use it for Japanese text). \
                 Tab switches fields; Enter or Esc finishes.",
            ))
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
                            view.editing = None;
                            view.save();
                            cx.notify();
                        }),
                    ),
            )
            .child(status_row)
    }
}

/// A label-over-control row.
fn labeled(label: &'static str, label_color: gpui::Rgba, control: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(label_color).child(label))
        .child(control)
}
