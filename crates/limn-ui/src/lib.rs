//! Limn UI library — the view types the bin (and tests) render with.
//!
//! Splitting this out of `main.rs` lets tests construct `DocumentView`
//! directly via `TestAppContext`.

use gpui::{
    div, px, rgb, Context, IntoElement, ParentElement, Render, SharedString, Styled, Window,
};

use limn_core::block::{Block, BlockKind};
use limn_service::{Config, Theme};

pub mod settings;

pub use settings::{SaveStatus, SettingsView};

/// Read-only view of a parsed Markdown document. Currently the only
/// view; an editing-capable variant is not yet provided.
pub struct DocumentView {
    pub title: SharedString,
    pub blocks: Vec<Block>,
    /// User settings driving theme colours and editor font. Read when
    /// the view is constructed and applied on every `render`.
    pub config: Config,
}

impl Render for DocumentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (bg, fg) = match self.config.theme {
            Theme::Light => (rgb(0x00fa_f9f6), rgb(0x001a_1a1a)),
            Theme::Dark => (rgb(0x001a_1a1a), rgb(0x00f0_f0f0)),
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
