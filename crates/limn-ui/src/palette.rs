//! Command palette shell (Wave 5) with fuzzy file open (Wave 6), gated
//! behind `LIMN_FEAT_PALETTE`.
//!
//! The palette is a modal list opened with Ctrl/Cmd+Shift+P (the
//! [`crate::actions::TogglePalette`] action). It reuses `gpui-component`'s
//! `Dialog` overlay machinery for the modal chrome (centered card, dimmed
//! backdrop, Esc / overlay-click to close) and its `List` widget for the
//! selectable rows. See ADR-0008 for why the Dialog's own `"Dialog"`
//! context is independent of limn's action contexts.
//!
//! The palette has two modes (see [`PaletteMode`]):
//!
//! - **Commands** (Wave 5): a static, non-searchable list of two
//!   commands, "Open File..." and "Open Settings".
//! - **Files** (Wave 6): a fuzzy search over every `.md` file in the
//!   vault. Selecting "Open File..." in Commands mode transitions the
//!   palette into Files mode in place (the dialog stays open), flips the
//!   list to searchable, and focuses the search input.
//!
//! Confirming a file in Files mode swaps the editor's buffer to that file
//! via [`EditorView::open_file`] (ADR-0009). "Open Settings" remains a
//! placeholder log until Wave 8.

use gpui::{
    div, px, App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Subscription, Task, WeakEntity, Window,
};
use gpui_component::{
    label::Label,
    list::{List, ListDelegate, ListEvent, ListItem, ListState},
    ActiveTheme, IndexPath, Selectable, WindowExt as _,
};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

use limn_service::{Vault, VaultEntry};

use crate::EditorView;

/// A command the palette can invoke. Stable identifier matched on at
/// confirm time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
    /// Enter the fuzzy file-search mode (Wave 6).
    OpenFile,
    /// Open the settings view. View transition is Wave 8.
    OpenSettings,
}

/// One row in the palette's Commands mode: a stable [`CommandId`] plus its
/// display title.
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    /// What the row does when confirmed.
    pub id: CommandId,
    /// Human-readable label shown in the list.
    pub title: SharedString,
}

/// The built-in command set shown when the palette opens.
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

/// Which stage of the palette is showing.
///
/// The palette starts in [`PaletteMode::Commands`] and transitions to
/// [`PaletteMode::Files`] when "Open File..." is confirmed. The two modes
/// drive every `ListDelegate` method (`items_count`, `render_item`,
/// `confirm`, `perform_search`) through a single delegate.
enum PaletteMode {
    /// Static command list (Wave 5). Not searchable.
    Commands { commands: Vec<PaletteCommand> },
    /// Fuzzy `.md` file search (Wave 6). `entries` is the full vault
    /// listing; `matched` holds indices into `entries` in match-ranked
    /// order (best first). With an empty query, `matched` mirrors
    /// `entries` in alphabetical order.
    Files {
        entries: Vec<VaultEntry>,
        matched: Vec<usize>,
    },
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

/// `ListDelegate` for the palette: owns the current [`PaletteMode`] and
/// selection, renders each row, runs the fuzzy search, and confirms a
/// selection.
///
/// The palette uses a single section, so [`IndexPath::row`] is the index
/// into the current mode's row list.
pub struct PaletteDelegate {
    mode: PaletteMode,
    selected_index: Option<IndexPath>,
    /// Weak handle back to the editor whose buffer a file open swaps.
    /// Weak avoids a reference cycle and lets the palette no-op if the
    /// editor is gone.
    editor: WeakEntity<EditorView>,
    /// Reused fuzzy matcher allocation. `Matcher` owns scratch buffers,
    /// so keeping one across searches avoids reallocating per keystroke.
    matcher: Matcher,
}

impl PaletteDelegate {
    fn new(editor: WeakEntity<EditorView>) -> Self {
        Self {
            mode: PaletteMode::Commands {
                commands: builtin_commands(),
            },
            // Start with the first row selected so Enter works without an
            // explicit arrow press.
            selected_index: Some(IndexPath::default()),
            editor,
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// True once the palette has transitioned into fuzzy file search.
    /// Read by [`PaletteView`] after a confirm to decide whether to flip
    /// the list to searchable and focus the search input.
    fn is_files_mode(&self) -> bool {
        matches!(self.mode, PaletteMode::Files { .. })
    }

    /// Populate the vault `.md` listing and switch to Files mode.
    ///
    /// The vault root is the directory holding the currently open file
    /// (ADR-0009: provisional until Wave 7 introduces a configured root).
    /// If the editor has no path (the ephemeral Welcome) or the listing
    /// fails, we fall back to an empty corpus, which renders the list's
    /// empty state — a graceful degradation rather than an error modal.
    fn enter_files_mode(&mut self, cx: &App) {
        let entries = self
            .editor
            .upgrade()
            .and_then(|editor| {
                editor
                    .read(cx)
                    .current_path()
                    .map(std::path::Path::to_path_buf)
            })
            .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
            .and_then(|root| Vault::new(root).list_md_files().ok())
            .unwrap_or_default();

        // Empty query → show everything in alphabetical (listing) order.
        let matched = (0..entries.len()).collect();
        self.mode = PaletteMode::Files { entries, matched };
        self.selected_index = entries_default_selection(&self.mode);
    }

    /// Re-rank the file list against `query` using the fuzzy matcher.
    /// No-op in Commands mode.
    fn run_fuzzy(&mut self, query: &str) {
        let PaletteMode::Files { entries, matched } = &mut self.mode else {
            return;
        };

        if query.is_empty() {
            *matched = (0..entries.len()).collect();
            return;
        }

        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        // Match against each file name, carrying its index so we can map
        // ranked results back to `entries`. `match_list` returns the
        // surviving items already sorted by descending score.
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        let ranked = pattern.match_list(
            names
                .iter()
                .enumerate()
                .map(|(i, &name)| IndexedName { index: i, name }),
            &mut self.matcher,
        );
        *matched = ranked
            .into_iter()
            .map(|(item, _score)| item.index)
            .collect();
    }

    /// Open the file at the given matched-row index in the editor.
    /// Logs and returns on any failure (dead editor, read error).
    fn open_selected_file(&self, row: usize, window: &mut Window, cx: &mut App) {
        let PaletteMode::Files { entries, matched } = &self.mode else {
            return;
        };
        let Some(&entry_ix) = matched.get(row) else {
            return;
        };
        let Some(entry) = entries.get(entry_ix) else {
            return;
        };

        let raw = match Vault::open_path_raw(&entry.path) {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("limn-ui: open file failed: {e}");
                return;
            }
        };

        let Some(editor) = self.editor.upgrade() else {
            eprintln!("limn-ui: open file: editor is gone");
            return;
        };
        editor.update(cx, |editor, cx| editor.open_file(raw, window, cx));
    }
}

/// A `(name, index)` pair that matches as its `name`. Lets the fuzzy
/// `match_list` carry each entry's position so ranked results map back to
/// the `entries` Vec.
#[derive(Clone, Copy)]
struct IndexedName<'a> {
    index: usize,
    name: &'a str,
}

impl AsRef<str> for IndexedName<'_> {
    fn as_ref(&self) -> &str {
        self.name
    }
}

/// First-row selection for a freshly populated mode, or `None` when the
/// mode has no rows.
fn entries_default_selection(mode: &PaletteMode) -> Option<IndexPath> {
    let has_rows = match mode {
        PaletteMode::Commands { commands } => !commands.is_empty(),
        PaletteMode::Files { matched, .. } => !matched.is_empty(),
    };
    has_rows.then(IndexPath::default)
}

impl ListDelegate for PaletteDelegate {
    type Item = PaletteListItem;

    fn items_count(&self, _section: usize, _: &App) -> usize {
        match &self.mode {
            PaletteMode::Commands { commands } => commands.len(),
            PaletteMode::Files { matched, .. } => matched.len(),
        }
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let title: SharedString = match &self.mode {
            PaletteMode::Commands { commands } => commands.get(ix.row)?.title.clone(),
            PaletteMode::Files { entries, matched } => {
                let &entry_ix = matched.get(ix.row)?;
                entries.get(entry_ix)?.name.clone().into()
            }
        };
        let selected = self.selected_index == Some(ix);
        Some(PaletteListItem {
            base: ListItem::new(ix),
            title,
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

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        // The corpus is the directly-listed vault (small), so matching
        // inline is fine — no background task needed. We mutate `matched`
        // and let the `ListState` re-measure on notify.
        self.run_fuzzy(query);
        cx.notify();
        Task::ready(())
    }

    fn confirm(
        &mut self,
        _secondary: bool,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        let row = match self.selected_index {
            Some(ix) => ix.row,
            None => return,
        };

        match &self.mode {
            PaletteMode::Commands { commands } => {
                let Some(command) = commands.get(row) else {
                    return;
                };
                match command.id {
                    CommandId::OpenFile => {
                        // Transition into fuzzy file search in place. Do
                        // NOT close the dialog — `PaletteView`'s
                        // `ListEvent::Confirm` subscription flips the list
                        // to searchable and focuses the input.
                        self.enter_files_mode(cx);
                        cx.notify();
                    }
                    CommandId::OpenSettings => {
                        // Wave 8 turns this into a settings view
                        // transition; placeholder log for now.
                        eprintln!("limn-ui: command selected: OpenSettings");
                        window.close_dialog(cx);
                    }
                }
            }
            PaletteMode::Files { .. } => {
                self.open_selected_file(row, window, cx);
                window.close_dialog(cx);
            }
        }
    }
}

/// The view rendered inside the palette dialog: a bounded-height list.
/// Created fresh on each open (no persisted state across opens).
pub struct PaletteView {
    state: Entity<ListState<PaletteDelegate>>,
    focus_handle: FocusHandle,
    /// Subscription to the list's [`ListEvent`]. Dropping it unsubscribes,
    /// so it must live as long as the view.
    _list_subscription: Subscription,
}

impl PaletteView {
    /// Build a palette view in Commands mode.
    ///
    /// `searchable(false)`: the command set is static and a search input
    /// would add a second Esc consumer competing with the Dialog's
    /// Esc-to-close. Confirming "Open File..." flips this to searchable
    /// (see [`Self::on_list_event`]) when the fuzzy search becomes the
    /// point of interaction.
    ///
    /// `editor` is a weak handle to the editor whose buffer a file open
    /// swaps; it is threaded into the delegate.
    pub fn new(
        editor: WeakEntity<EditorView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state =
            cx.new(|cx| ListState::new(PaletteDelegate::new(editor), window, cx).searchable(false));

        // Subscribe to the list's events so a Commands-mode confirm that
        // switched to Files mode can flip the list to searchable and
        // focus the search input. This lives here (not in the delegate's
        // `confirm`) because flipping `searchable` and focusing are
        // `ListState` operations, which cannot be re-borrowed from inside
        // the delegate method the `ListState` is already running.
        let subscription = cx.subscribe_in(&state, window, Self::on_list_event);

        Self {
            state,
            focus_handle: cx.focus_handle(),
            _list_subscription: subscription,
        }
    }

    /// React to the list's events. On a confirm that left the delegate in
    /// Files mode (the "Open File..." transition), flip the list to
    /// searchable and focus the search input so the user can type
    /// immediately.
    #[expect(
        clippy::unused_self,
        reason = "signature is fixed by gpui's subscribe_in handler contract; \
                  the handler drives the list entity passed in, not self's fields"
    )]
    fn on_list_event(
        &mut self,
        state: &Entity<ListState<PaletteDelegate>>,
        event: &ListEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(event, ListEvent::Confirm(_)) {
            return;
        }

        let entered_files = state.read(cx).delegate().is_files_mode();
        if !entered_files {
            return;
        }

        state.update(cx, |state, cx| {
            state.set_searchable(true, cx);
            state.focus(window, cx);
        });
    }

    /// Focus the underlying `List` so keyboard navigation works as soon
    /// as the palette opens. In Commands mode this focuses the list; once
    /// searchable, `ListState::focus` focuses the search input instead.
    ///
    /// Mirrors `gpui-component`'s `Combobox`, which focuses its list every
    /// time it opens its overlay. `ListState::focus` is the public API for
    /// the same effect.
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
