//! Shell root view — owns which top-level view the window shows.
//!
//! gpui only dispatches an action along the focused element's path, so
//! keyboard-triggered view switching needs a single owner sitting above
//! every view. `Shell` is that owner: it renders the view named by
//! `self.view`, and the key bindings that switch it are scoped to
//! `Shell`'s own key context rather than any individual view's.

use gpui::{
    actions, div, rgb, App, AppContext, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyBinding, ParentElement, Render, SharedString, Styled,
    Window,
};

use limn_core::block::Block;

use crate::DocumentView;

actions!(limn, [ShowPalette, ShowSettings, ShowEditor]);

/// The top-level view currently shown inside `Shell`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellView {
    Editor,
    Palette,
    Settings,
}

const KEY_CONTEXT: &str = "Shell";

/// One row of the shortcut table: a keystroke and the view it shows.
pub struct Shortcut {
    pub keystroke: &'static str,
    pub view: ShellView,
}

/// The full set of keystrokes `Shell` binds. One row per shortcut; this
/// is the single place that pairs a keystroke with its destination view.
pub const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        keystroke: "ctrl-shift-p",
        view: ShellView::Palette,
    },
    Shortcut {
        keystroke: "ctrl-,",
        view: ShellView::Settings,
    },
    Shortcut {
        keystroke: "escape",
        view: ShellView::Editor,
    },
];

fn binding(shortcut: &Shortcut) -> KeyBinding {
    match shortcut.view {
        // limn:allow-panic: keystroke is a literal from SHORTCUTS, proven parseable by key_bindings_cover_every_shortcut
        ShellView::Editor => KeyBinding::new(shortcut.keystroke, ShowEditor, Some(KEY_CONTEXT)),
        // limn:allow-panic: keystroke is a literal from SHORTCUTS, proven parseable by key_bindings_cover_every_shortcut
        ShellView::Palette => KeyBinding::new(shortcut.keystroke, ShowPalette, Some(KEY_CONTEXT)),
        // limn:allow-panic: keystroke is a literal from SHORTCUTS, proven parseable by key_bindings_cover_every_shortcut
        ShellView::Settings => KeyBinding::new(shortcut.keystroke, ShowSettings, Some(KEY_CONTEXT)),
    }
}

/// Key bindings for every row in [`SHORTCUTS`], ready for `App::bind_keys`.
#[must_use]
pub fn key_bindings() -> Vec<KeyBinding> {
    SHORTCUTS.iter().map(binding).collect()
}

/// The window's root view. Owns the current [`ShellView`] and the one
/// [`DocumentView`] entity the editor view renders.
pub struct Shell {
    pub view: ShellView,
    pub document: Entity<DocumentView>,
    focus_handle: FocusHandle,
}

impl Shell {
    #[must_use]
    pub fn new(title: SharedString, blocks: Vec<Block>, cx: &mut Context<Self>) -> Self {
        Self {
            view: ShellView::Editor,
            document: cx.new(|_| DocumentView { title, blocks }),
            focus_handle: cx.focus_handle(),
        }
    }

    fn show(&mut self, view: ShellView, cx: &mut Context<Self>) {
        self.view = view;
        cx.notify();
    }
}

impl Focusable for Shell {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("shell")
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(|this, _: &ShowPalette, _window, cx| {
                this.show(ShellView::Palette, cx);
            }))
            .on_action(cx.listener(|this, _: &ShowSettings, _window, cx| {
                this.show(ShellView::Settings, cx);
            }))
            .on_action(cx.listener(|this, _: &ShowEditor, _window, cx| {
                this.show(ShellView::Editor, cx);
            }))
            .track_focus(&self.focus_handle)
            .size_full()
            .child(match self.view {
                ShellView::Editor => self.document.clone().into_any_element(),
                ShellView::Palette => placeholder("Command Palette").into_any_element(),
                ShellView::Settings => placeholder("Settings").into_any_element(),
            })
    }
}

/// A placeholder body for a view that has no state or content yet.
fn placeholder(label: &str) -> gpui::Div {
    div()
        .size_full()
        .bg(rgb(0x00fa_f9f6))
        .text_color(rgb(0x0090_9090))
        .p_8()
        .child(label.to_string())
}
