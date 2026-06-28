//! Command palette shell (Wave 5, gated behind `LIMN_FEAT_PALETTE`).
//!
//! The palette is a modal list of commands opened with
//! Ctrl/Cmd+Shift+P (the [`crate::actions::TogglePalette`] action). It
//! reuses `gpui-component`'s `Dialog` overlay machinery for the modal
//! chrome (centered card, dimmed backdrop, Esc / overlay-click to close)
//! and its `List` widget for the selectable rows. See ADR-0008 for why
//! the Dialog's own `"Dialog"` context is independent of limn's action
//! contexts.
//!
//! Scope of this wave is deliberately thin: two static commands ("Open
//! File..." and "Open Settings"), keyboard selection, and a confirm that
//! logs a placeholder and closes the modal. The fuzzy file search behind
//! "Open File..." (Wave 6) and the Settings view transition behind "Open
//! Settings" (Wave 8) are out of scope — [`PaletteDelegate::confirm`]
//! stops at an `eprintln!` placeholder for both.
//!
//! `LIMN_FEAT_PALETTE` is layered on top of `LIMN_FEAT_EDIT`: only the
//! editable path builds a `gpui-component` `Root`, and the Dialog overlay
//! needs that `Root`. So the palette can only appear when *both* flags
//! are on (the toggle handler in `editor.rs` guards on `palette`, and the
//! editable path is the only one that registers the keybinding).

use gpui::{
    div, px, App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Window,
};
use gpui_component::{
    label::Label,
    list::{List, ListDelegate, ListItem, ListState},
    ActiveTheme, IndexPath, Selectable, WindowExt as _,
};

/// A command the palette can invoke. Stable identifier matched on at
/// confirm time; later waves (Wave 6 file open, Wave 8 settings) extend
/// the `match` in [`PaletteDelegate::confirm`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
    /// Open a file by fuzzy search. Search UI is Wave 6.
    OpenFile,
    /// Open the settings view. View transition is Wave 8.
    OpenSettings,
}

/// One row in the palette: a stable [`CommandId`] plus its display title.
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    /// What the row does when confirmed.
    pub id: CommandId,
    /// Human-readable label shown in the list.
    pub title: SharedString,
}

/// The built-in command set shown when the palette opens.
///
/// This is the function boundary that Wave 6 will turn into a fuzzy
/// filter (`builtin_commands()` → `filter(query)`); for now it returns
/// the full static list unconditionally.
#[must_use]
pub fn builtin_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand {
            id: CommandId::OpenFile,
            title: "Open File...".into(),
        },
        PaletteCommand {
            id: CommandId::OpenSettings,
            title: "Open Settings".into(),
        },
    ]
}

/// One rendered palette row. Wraps `gpui-component`'s [`ListItem`] so the
/// list's selection styling (highlight on the selected row) is applied
/// for free.
#[derive(IntoElement)]
pub struct PaletteListItem {
    base: ListItem,
    title: SharedString,
    selected: bool,
}

impl Selectable for PaletteListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self.base = self.base.selected(selected);
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl gpui::RenderOnce for PaletteListItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let text_color = if self.selected {
            cx.theme().accent_foreground
        } else {
            cx.theme().foreground
        };

        self.base
            .px_2()
            .py_1()
            .rounded(cx.theme().radius)
            .child(Label::new(self.title).text_color(text_color))
    }
}

/// `ListDelegate` for the palette: owns the command rows and the current
/// selection, renders each row, and confirms a selection.
///
/// The palette uses a single section, so [`IndexPath::row`] is the index
/// into `commands`.
pub struct PaletteDelegate {
    commands: Vec<PaletteCommand>,
    selected_index: Option<IndexPath>,
}

impl PaletteDelegate {
    fn new() -> Self {
        Self {
            commands: builtin_commands(),
            // Start with the first row selected so Enter works without
            // an explicit arrow press.
            selected_index: Some(IndexPath::default()),
        }
    }
}

impl ListDelegate for PaletteDelegate {
    type Item = PaletteListItem;

    fn items_count(&self, _section: usize, _: &App) -> usize {
        self.commands.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let command = self.commands.get(ix.row)?;
        let selected = self.selected_index == Some(ix);
        Some(PaletteListItem {
            base: ListItem::new(ix),
            title: command.title.clone(),
            selected,
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }

    fn confirm(
        &mut self,
        _secondary: bool,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        // Wave 5 stops at a placeholder: log which command was chosen,
        // then dismiss the modal. Wave 6 (file fuzzy open) and Wave 8
        // (settings view) extend this `match` with real behaviour.
        if let Some(command) = self.selected_index.and_then(|ix| self.commands.get(ix.row)) {
            match command.id {
                CommandId::OpenFile | CommandId::OpenSettings => {
                    eprintln!("limn-ui: command selected: {:?}", command.id);
                }
            }
        }

        window.close_dialog(cx);
    }
}

/// The view rendered inside the palette dialog: a bounded-height list of
/// commands. Created fresh on each open (no persisted selection across
/// opens), so it holds only the `ListState` entity.
pub struct PaletteView {
    state: Entity<ListState<PaletteDelegate>>,
    focus_handle: FocusHandle,
}

impl PaletteView {
    /// Build a palette view with the built-in commands.
    ///
    /// `searchable(false)`: the command set is static (two items) and a
    /// search input would add a second Esc consumer that competes with
    /// the Dialog's Esc-to-close. Wave 6 flips this to `searchable(true)`
    /// when the fuzzy file search lands and the input becomes the point
    /// of interaction.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state =
            cx.new(|cx| ListState::new(PaletteDelegate::new(), window, cx).searchable(false));

        Self {
            state,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Focus the underlying `List` so arrow-key navigation and Enter
    /// confirm work as soon as the palette opens.
    ///
    /// GPUI dispatches key events from the focused node upward to the
    /// root, so the `List`'s `"List"` key context (which binds up / down /
    /// enter) only sits on the dispatch path when the `List` itself is
    /// focused. `open_dialog` focuses the *Dialog* node, and the `List` is
    /// a descendant of it — not an ancestor — so without this explicit
    /// focus the list's bindings never see the keys and keyboard selection
    /// is dead (only mouse click and the Dialog's own Esc work).
    ///
    /// This mirrors `gpui-component`'s own `Combobox`, which calls
    /// `self.state.list.focus_handle(cx).focus(window, cx)` every time it
    /// opens its overlay (combobox.rs). `ListState::focus` is the public
    /// API for the same effect (list/list.rs).
    pub fn focus_list(&self, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| state.focus(window, cx));
    }
}

impl Focusable for PaletteView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PaletteView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // `List` is a virtual list, so its scroll parent must have a
        // bounded height — without `max_h` the virtual list cannot lay
        // out and the rows do not appear.
        div()
            .w_full()
            .max_h(px(320.0))
            .child(List::new(&self.state))
    }
}

#[cfg(test)]
mod tests {
    use super::{builtin_commands, CommandId};

    #[test]
    fn builtin_commands_returns_two() {
        assert_eq!(builtin_commands().len(), 2);
    }

    #[test]
    fn builtin_commands_cover_each_id() {
        let ids: Vec<CommandId> = builtin_commands().into_iter().map(|c| c.id).collect();
        assert!(ids.contains(&CommandId::OpenFile));
        assert!(ids.contains(&CommandId::OpenSettings));
    }
}
