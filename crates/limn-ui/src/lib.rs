//! Limn UI library — the view types the bin (and tests) render with.
//!
//! Splitting this out of `main.rs` lets tests construct `DocumentView`
//! directly via `TestAppContext`.

use gpui::{div, rgb, Context, IntoElement, ParentElement, Render, SharedString, Styled, Window};

use limn_core::block::{Block, BlockKind};

pub mod actions;
pub mod editor;
pub mod feature_flags;
pub mod palette;

pub use editor::EditorView;
pub use feature_flags::FeatureFlags;
pub use palette::PaletteView;

/// Read-only view of a parsed Markdown document. M1's only view; M2
/// will introduce an editing-capable variant.
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
