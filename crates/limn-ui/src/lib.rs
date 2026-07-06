//! Limn UI library — the view types the bin (and tests) render with.
//!
//! Splitting this out of `main.rs` lets tests construct `DocumentView`
//! and `EditorView` directly via `TestAppContext`.

use gpui::{
    div, rgb, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Subscription, Window,
};
use gpui_component::input::{Input, InputEvent, InputState};

use limn_core::block::{Block, BlockKind};

pub mod feature_flags;

pub use feature_flags::FeatureFlags;

/// Read-only view of a parsed Markdown document.
pub struct DocumentView {
    pub title: SharedString,
    pub blocks: Vec<Block>,
}

impl Render for DocumentView {
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
                    .child(format!("Limn — {}", self.title)),
            )
            .children(self.blocks.iter().map(render_block))
    }
}

/// Editable view of a document, backed by `gpui_component`'s
/// `InputState`. Text entry, IME, cursor movement, selection,
/// clipboard, and undo/redo all come from `InputState` — this view
/// only wires the widget up and keeps its title.
pub struct EditorView {
    pub title: SharedString,
    input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl EditorView {
    #[must_use]
    pub fn new(
        title: SharedString,
        initial_text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .soft_wrap(true)
                .default_value(initial_text)
        });

        let subscriptions = vec![cx.subscribe_in(
            &input,
            window,
            |_view, _input, event: &InputEvent, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            },
        )];

        Self {
            title,
            input,
            _subscriptions: subscriptions,
        }
    }

    /// The editor's current text, read straight from `InputState`.
    #[must_use]
    pub fn value(&self, cx: &gpui::App) -> SharedString {
        self.input.read(cx).value()
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
                    .child(format!("Limn — {}", self.title)),
            )
            .child(div().flex_1().child(Input::new(&self.input)))
    }
}

/// Render one block as a `Div`. Public so tests can poke individual
/// blocks if they need to.
#[must_use]
pub fn render_block(block: &Block) -> gpui::Div {
    let text = block.text.clone();
    match block.kind {
        BlockKind::Heading { level } => {
            let inner = match level {
                1 => div().text_2xl(),
                2 => div().text_xl(),
                _ => div().text_lg(),
            };
            div().child(inner.child(text))
        }
        BlockKind::Paragraph => div().child(div().child(text)),
    }
}
