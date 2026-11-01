//! Workspace: the window's root view.
//!
//! Owns what the main area shows (a document or the settings view) and
//! the command palette overlay. `Ctrl+Shift+P` toggles the palette;
//! while it is open, typing filters the list, `up` / `down` move the
//! selection, `enter` confirms and `escape` closes.
//!
//! Key bindings are registered here with [`bind_keys`] until a shared
//! shortcut layer exists; the actions themselves are the stable seam.

use std::path::Path;

use gpui::{
    actions, div, prelude::*, px, rgb, App, Context, FocusHandle, Focusable, KeyBinding,
    KeyContext, KeyDownEvent, SharedString, Window,
};

use limn_core::block::Block;
use limn_service::Vault;

use crate::palette::{Command, Mode, Outcome, Palette};
use crate::{render_block, DocumentView};

actions!(
    workspace,
    [
        TogglePalette,
        PaletteSelectNext,
        PaletteSelectPrev,
        PaletteConfirm,
        PaletteDismiss
    ]
);

/// Register the workspace key bindings. Call once at startup.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-shift-p", TogglePalette, Some("Workspace")),
        KeyBinding::new("down", PaletteSelectNext, Some("Palette")),
        KeyBinding::new("up", PaletteSelectPrev, Some("Palette")),
        KeyBinding::new("enter", PaletteConfirm, Some("Palette")),
        KeyBinding::new("escape", PaletteDismiss, Some("Palette")),
    ]);
}

/// What the main area shows.
pub enum MainView {
    Document(DocumentView),
    /// Placeholder until the settings view lands.
    Settings,
}

pub struct Workspace {
    pub main: MainView,
    pub palette: Option<Palette>,
    /// Root for the palette's file search. `None` when launched without
    /// a path (the Welcome document has no vault).
    pub vault: Option<Vault>,
    /// Last file-open failure, shown under the title until the next open.
    pub error: Option<SharedString>,
    focus_handle: FocusHandle,
}

impl Workspace {
    pub fn new(document: DocumentView, vault: Option<Vault>, cx: &mut Context<Self>) -> Self {
        Self {
            main: MainView::Document(document),
            palette: None,
            vault,
            error: None,
            focus_handle: cx.focus_handle(),
        }
    }

    fn toggle_palette(&mut self, _: &TogglePalette, _: &mut Window, cx: &mut Context<Self>) {
        self.palette = match self.palette {
            Some(_) => None,
            None => Some(Palette::new()),
        };
        cx.notify();
    }

    fn select_next(&mut self, _: &PaletteSelectNext, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(palette) = &mut self.palette {
            palette.select_next();
            cx.notify();
        }
    }

    fn select_prev(&mut self, _: &PaletteSelectPrev, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(palette) = &mut self.palette {
            palette.select_prev();
            cx.notify();
        }
    }

    fn dismiss(&mut self, _: &PaletteDismiss, _: &mut Window, cx: &mut Context<Self>) {
        self.palette = None;
        cx.notify();
    }

    fn confirm(&mut self, _: &PaletteConfirm, _: &mut Window, cx: &mut Context<Self>) {
        let Some(palette) = &mut self.palette else {
            return;
        };
        match palette.confirm() {
            Outcome::None => {}
            Outcome::Run(Command::OpenFile) => self.load_file_list(cx),
            Outcome::Run(Command::OpenSettings) => {
                self.palette = None;
                self.main = MainView::Settings;
            }
            Outcome::OpenFile(relative) => {
                self.palette = None;
                self.open_file(&relative, cx);
            }
        }
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(palette) = &mut self.palette else {
            return;
        };
        let keystroke = &event.keystroke;
        let modifiers = &keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }
        if keystroke.key == "backspace" {
            palette.backspace();
        } else if let Some(text) = keystroke.key_char.as_deref() {
            if text.chars().all(|c| !c.is_control()) {
                palette.push_str(text);
            }
        } else {
            return;
        }
        cx.stop_propagation();
        cx.notify();
    }

    /// Walk the vault off the UI thread and hand the result to the
    /// palette. Without a vault the list is simply empty.
    fn load_file_list(&mut self, cx: &mut Context<Self>) {
        let Some(vault) = self.vault.clone() else {
            if let Some(palette) = &mut self.palette {
                palette.set_files(Vec::new());
            }
            return;
        };
        let walk = cx
            .background_executor()
            .spawn(async move { vault.list_md_files().unwrap_or_default() });
        cx.spawn(async move |this, cx| {
            let files = walk.await;
            this.update(cx, |this, cx| {
                if let Some(palette) = &mut this.palette {
                    palette.set_files(files);
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Read and parse `relative` off the UI thread, then show it.
    fn open_file(&mut self, relative: &Path, cx: &mut Context<Self>) {
        let Some(vault) = &self.vault else {
            return;
        };
        let path = vault.root.join(relative);
        let title: SharedString = display_title(relative).into();
        let read = cx
            .background_executor()
            .spawn(async move { Vault::open_path(&path) });
        cx.spawn(async move |this, cx| {
            let result = read.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(document) => {
                        this.error = None;
                        this.main = MainView::Document(DocumentView {
                            title,
                            blocks: document.blocks,
                        });
                    }
                    Err(e) => this.error = Some(format!("failed to open {title}: {e}").into()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_main(&self) -> gpui::Div {
        let (title, blocks): (SharedString, &[Block]) = match &self.main {
            MainView::Document(doc) => (doc.title.clone(), &doc.blocks),
            MainView::Settings => ("Settings".into(), &[]),
        };
        let mut main = div().flex().flex_col().gap_2().child(
            div()
                .text_xs()
                .opacity(0.5)
                .child(format!("Limn — {title}")),
        );
        if let Some(error) = &self.error {
            main = main.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x00b0_3a2e))
                    .child(error.clone()),
            );
        }
        match &self.main {
            MainView::Document(_) => main.children(blocks.iter().map(render_block)),
            MainView::Settings => main
                .child(div().text_2xl().child("Settings"))
                .child(div().opacity(0.6).child("Nothing to configure yet.")),
        }
    }

    fn render_palette(palette: &Palette) -> gpui::Div {
        let items = palette.items();
        let query: SharedString = if palette.query().is_empty() {
            palette.placeholder().into()
        } else {
            palette.query().to_string().into()
        };
        let empty_hint = match palette.mode() {
            Mode::Files { files: None } => Some("Loading files…"),
            _ if items.is_empty() => Some("No matches"),
            _ => None,
        };
        div()
            .absolute()
            .top(px(48.0))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(
                div()
                    .w(px(520.0))
                    .bg(rgb(0x00ff_fefb))
                    .border_1()
                    .border_color(rgb(0x00d8_d4cc))
                    .rounded_md()
                    .shadow_lg()
                    .p_2()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .border_b_1()
                            .border_color(rgb(0x00e8_e4dc))
                            .when(palette.query().is_empty(), |d| d.opacity(0.5))
                            .child(query),
                    )
                    .children(empty_hint.map(|hint| div().px_2().opacity(0.5).child(hint)))
                    .children(items.into_iter().take(50).enumerate().map(|(i, label)| {
                        div()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .when(i == palette.selected(), |d| d.bg(rgb(0x00ec_e7dc)))
                            .child(label)
                    })),
            )
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut context = KeyContext::new_with_defaults();
        context.add("Workspace");
        if self.palette.is_some() {
            context.add("Palette");
        }

        div()
            .id("workspace")
            .key_context(context)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_palette))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_key_down(cx.listener(Self::on_key_down))
            .relative()
            .size_full()
            .bg(rgb(0x00fa_f9f6))
            .text_color(rgb(0x001a_1a1a))
            .p_8()
            .child(self.render_main())
            .children(self.palette.as_ref().map(Self::render_palette))
    }
}

fn display_title(relative: &Path) -> String {
    relative
        .file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| "(unnamed)".to_string(), String::from)
}
