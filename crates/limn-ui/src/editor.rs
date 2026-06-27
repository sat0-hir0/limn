//! Editable editor view (M2, gated behind `LIMN_FEAT_EDIT`).
//!
//! Wraps `gpui-component`'s `InputState` / `Input` widget, which brings
//! its own text buffer, cursor, range selection, delete, copy/cut/paste,
//! undo/redo, and IME composition (see ADR-0005). The view is seeded
//! with the file's raw UTF-8 text — not a parsed `Vec<Block>` — and
//! holds the `InputState` entity for the editor's lifetime.
//!
//! Autosave / write-back to disk is intentionally out of scope for this
//! wave; this view only proves that input is accepted and reflected on
//! screen.

use gpui::{
    div, px, rgb, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window,
};
use gpui_component::input::{Input, InputState};

/// An editable view of a Markdown file, backed by a `gpui-component`
/// `InputState`.
pub struct EditorView {
    /// File name shown in the header strip (matches `DocumentView`).
    pub title: SharedString,
    /// The text buffer + editing state machine. Multi-line, seeded with
    /// the file's raw text.
    state: Entity<InputState>,
}

impl EditorView {
    /// Build an editor seeded with `text` (the file's raw UTF-8
    /// contents).
    ///
    /// `InputState::new` needs a `Window` because it registers focus /
    /// blur subscriptions and a blink-cursor observer against it. The
    /// builder chain enables multi-line editing and seeds the buffer via
    /// `default_value`.
    pub fn new(
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let text = text.into();
        let state = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .default_value(text)
        });

        Self {
            title: title.into(),
            state,
        }
    }

    /// Focus the editor so keystrokes land in the buffer.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.focus(window, cx));
    }

    /// Current buffer text. Exposed so tests can assert that typed input
    /// reached the buffer (the wave's UAT condition).
    #[must_use]
    pub fn value(&self, cx: &Context<Self>) -> SharedString {
        self.state.read(cx).value()
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x00fa_f9f6);
        let fg = rgb(0x001a_1a1a);

        div()
            .size_full()
            .bg(bg)
            .text_color(fg)
            .p_8()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .opacity(0.5)
                    .child(format!("Limn — {} (editing)", self.title)),
            )
            .child(
                // Let the input fill the remaining space; `flex_1` /
                // `min_h` keeps it from collapsing in the column.
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .child(Input::new(&self.state).h_full()),
            )
    }
}
