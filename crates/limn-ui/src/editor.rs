//! Editable editor view (M2, gated behind `LIMN_FEAT_EDIT`).
//!
//! Wraps `gpui-component`'s `InputState` / `Input` widget, which brings
//! its own text buffer, cursor, range selection, delete, copy/cut/paste,
//! undo/redo, and IME composition (see ADR-0005). The view is seeded
//! with the file's raw UTF-8 text — not a parsed `Vec<Block>` — and
//! holds the `InputState` entity for the editor's lifetime.
//!
//! Autosave (Wave 3): every text change emits `InputEvent::Change`; we
//! subscribe to it, debounce a short window so a burst of keystrokes
//! collapses into one write, then hand the raw buffer to
//! `Vault::save_raw`. There is no explicit save button by design
//! (see `docs/design/basic-features.md`). The actual filesystem write
//! lives in `limn-service` so this crate never calls `std::fs` directly
//! (ADR-0002), and the write is serialized off the main thread via the
//! background executor.

use std::path::PathBuf;
use std::time::Duration;

use gpui::{
    div, px, rgb, App, AppContext as _, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, Subscription,
    Task, Window,
};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Root, WindowExt as _};

use crate::actions::TogglePalette;
use crate::{file_title, FeatureFlags, PaletteView};

use limn_service::{RawDocument, Vault};

/// How long to wait after the last keystroke before writing to disk.
/// Long enough that continuous typing does not write on every character,
/// short enough that a pause feels like "it saved".
const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(600);

/// An editable view of a Markdown file, backed by a `gpui-component`
/// `InputState`.
pub struct EditorView {
    /// File name shown in the header strip (matches `DocumentView`).
    pub title: SharedString,
    /// Where the buffer autosaves back to, or `None` for an ephemeral
    /// document (e.g. the embedded Welcome) that has no backing file.
    /// Held so the change handler knows where — and whether — to write.
    /// When `None`, autosave is skipped entirely (see `schedule_save`)
    /// so editing Welcome never litters the working directory.
    path: Option<PathBuf>,
    /// The text buffer + editing state machine. Multi-line, seeded with
    /// the file's raw text.
    state: Entity<InputState>,
    /// Subscription to the input's change events. Dropping it
    /// unsubscribes, so it must live as long as the view.
    _change_subscription: Subscription,
    /// The pending debounced save. Replacing it drops (and therefore
    /// cancels) any in-flight timer, which is what makes the debounce
    /// work: only the most recent change survives the quiet window.
    pending_save: Task<()>,
    /// The view's own focus handle. The render root is `track_focus`d on
    /// it so the `EditorView` is always present on the window's dispatch
    /// (focus) tree. That placement is what lets a globally-dispatched
    /// action (e.g. [`TogglePalette`]) reach this view's `on_action`
    /// handler — see `focus` and `render` for the focus reasoning.
    focus_handle: FocusHandle,
}

impl EditorView {
    /// Build an editor seeded with `text` (the file's raw UTF-8
    /// contents) that autosaves edits back to `path`.
    ///
    /// `path` is `Some` for a real file opened from disk and `None` for
    /// an ephemeral document (the embedded Welcome) that has nowhere to
    /// write back to. With `None`, edits are accepted but never saved.
    ///
    /// `InputState::new` needs a `Window` because it registers focus /
    /// blur subscriptions and a blink-cursor observer against it. The
    /// builder chain enables multi-line editing and seeds the buffer via
    /// `default_value`. We then subscribe to the state's
    /// [`InputEvent::Change`] so every edit schedules a debounced save.
    pub fn new(
        title: impl Into<SharedString>,
        path: Option<PathBuf>,
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

        // `subscribe_in` delivers `(&mut self, &Entity<InputState>,
        // &InputEvent, &mut Window, &mut Context<Self>)` — the pattern
        // the gpui-component input stories use. Seeding the buffer via
        // `default_value` above does not emit `Change`, so this only
        // fires on real user edits.
        let subscription = cx.subscribe_in(&state, window, Self::on_input_event);

        Self {
            title: title.into(),
            path,
            state,
            _change_subscription: subscription,
            // No edit yet, so nothing pending. A no-op task is the
            // simplest "empty" value for the slot.
            pending_save: Task::ready(()),
            focus_handle: cx.focus_handle(),
        }
    }

    /// Focus the editor so keystrokes land in the buffer.
    ///
    /// We focus the `InputState`, not `self.focus_handle`, on purpose.
    /// Text input must reach the component's `EntityInputHandler` (cursor,
    /// selection, IME), which only happens when the `InputState` itself
    /// holds focus. `self.focus_handle` is *not* focused here; instead the
    /// render root is `track_focus`d on it (see `render`), so the
    /// `EditorView` is an ancestor of the focused `InputState` on the
    /// dispatch tree. That ancestry is enough for a globally-dispatched
    /// action to bubble to this view's `on_action` handler.
    ///
    /// This is the deliberate resolution of the focus double-management
    /// concern the architect flagged: rather than juggling who "owns"
    /// focus, we keep a single focused handle (the `InputState`) and rely
    /// on the focus *chain* — the tracked `EditorView` root above it — to
    /// receive actions. No real device was available to confirm dispatch
    /// behaviour in this session, so we chose the option that keeps the
    /// `EditorView` on the focus chain unconditionally rather than one
    /// that depends on a `None`-context action finding a non-ancestor
    /// handler.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.focus(window, cx));
    }

    /// Switch the editor to a different file in place (Wave 6).
    ///
    /// Rather than tearing down and rebuilding the window's view tree,
    /// we keep the same [`InputState`] entity and swap its buffer. This
    /// is the "buffer swap" approach (ADR-0009): the editor, its focus
    /// handle, and the change subscription all survive the switch.
    ///
    /// The ordering is load-bearing and guards against autosaving the
    /// *new* text back to the *old* path:
    ///
    /// 1. **Flush the old file synchronously**, then drop the debounce
    ///    timer. Any edit made just before the switch must reach the
    ///    file it belongs to, and the still-sleeping debounced save
    ///    (which captured the *old* `path` and old text) must be
    ///    cancelled so it cannot fire after we repoint `self.path`.
    /// 2. **Repoint `path` / `title` to the new file** before touching
    ///    the buffer, so the field the change handler reads is already
    ///    correct.
    /// 3. **Swap the buffer via [`InputState::set_value`]**, which sets
    ///    `emit_events = false` around the replace (state.rs) and so
    ///    does *not* emit [`InputEvent::Change`]. That is the core of
    ///    the safety: seeding the new text never schedules a save, so
    ///    there is no window in which the new text could be written to
    ///    the old path.
    /// 4. **Re-focus the editor** so typing lands in the freshly loaded
    ///    buffer without a click.
    pub fn open_file(&mut self, raw: RawDocument, window: &mut Window, cx: &mut Context<Self>) {
        // 1. Flush pending edits to the OLD path, then cancel the timer.
        //    Dropping `pending_save` cooperatively cancels the debounce,
        //    but cannot interrupt a background `save_raw` already mid-write
        //    (its blocking body has no await point). That synchronous flush
        //    and the in-flight background save can thus both target the OLD
        //    path at once. They write the *same* (old) buffer, and each now
        //    writes its own per-write temp file before an atomic rename (see
        //    `Vault::save_raw`), so the OLD path always ends up with one
        //    writer's complete contents — never an empty or torn file.
        self.flush_pending_save(cx);
        self.pending_save = Task::ready(());

        // 2. Repoint to the NEW file before the buffer changes.
        self.path = Some(raw.path.clone());
        self.title = file_title(&raw.path);

        // 3. Swap the buffer. `set_value` does not emit `Change`, so no
        //    save is scheduled against either path here.
        self.state
            .update(cx, |state, cx| state.set_value(raw.text, window, cx));

        // 4. Focus so the first keystroke lands in the new buffer.
        self.focus(window, cx);
    }

    /// Write the current buffer back to the current `path` synchronously,
    /// if there is one.
    ///
    /// Called from [`open_file`] before switching away: the debounced
    /// autosave may still be sleeping, so we force the most recent edits
    /// to disk *now* while `self.path` still points at the file they
    /// belong to. A `None` path (the ephemeral Welcome document) has no
    /// save target, so this is a no-op there — matching `schedule_save`.
    ///
    /// This runs on the main thread, unlike the debounced autosave which
    /// is dispatched to the background executor. ADR-0009 accepts this
    /// as a deliberate, narrowly-scoped exception to ARCHITECTURE.md's
    /// "never block the main thread" rule: a file switch is a discrete
    /// user action (not a hot path), vault files are small Markdown
    /// notes, and doing the flush inline is what makes the
    /// flush-before-repoint ordering airtight without an async round-trip.
    ///
    /// [`open_file`]: EditorView::open_file
    fn flush_pending_save(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else {
            return;
        };
        let text = self.state.read(cx).value().to_string();
        match Vault::save_raw(&path, &text) {
            Ok(()) => eprintln!("limn-ui: flushed on switch"),
            Err(e) => eprintln!("limn-ui: flush on switch failed: {e}"),
        }
    }

    /// Handle the [`TogglePalette`] action: open the command palette
    /// modal, or close it if one is already open.
    ///
    /// Gated on `LIMN_FEAT_PALETTE`: when the flag is off the action is a
    /// no-op (the keybinding is still registered, but the editable shell
    /// otherwise behaves as before). The flag is layered on
    /// `LIMN_FEAT_EDIT` — this handler only exists on the editable path,
    /// which is the only one that builds a `gpui-component` `Root`, and a
    /// `Root` is required for the Dialog overlay the palette renders into.
    ///
    /// A fresh [`PaletteView`] is created on every open so no selection or
    /// query state survives across opens; the `EditorView` deliberately
    /// holds no palette field.
    #[expect(
        clippy::unused_self,
        reason = "signature is fixed by gpui's on_action listener contract; \
                  the handler reads globals, the window, and the current \
                  entity via cx, not self's fields"
    )]
    fn on_toggle_palette(
        &mut self,
        _: &TogglePalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !cx.global::<FeatureFlags>().palette {
            return;
        }

        if window.has_active_dialog(cx) {
            window.close_dialog(cx);
            return;
        }

        // Hand the palette a weak handle back to this editor so its
        // "Open File" confirm can swap the buffer (Wave 6). Weak avoids a
        // reference cycle (editor → dialog → palette → editor) and lets
        // the palette degrade gracefully if the editor is gone.
        let editor = cx.entity().downgrade();
        let palette = cx.new(|cx| PaletteView::new(editor, window, cx));
        window.open_dialog(cx, {
            let palette = palette.clone();
            move |dialog, _, _| {
                // No title / footer / close button: the palette is a bare
                // command list. `keyboard(true)` (the default) keeps the
                // Dialog's Esc-to-close. The list's own Esc/Enter live in
                // the `"List"` context; see ADR-0008 for why these are
                // independent of limn's action contexts.
                dialog.title("Command Palette").child(palette.clone())
            }
        });

        // `open_dialog` focuses the Dialog node, not the List inside it.
        // GPUI dispatches keys from the focused node upward, so the List's
        // `"List"` context (up / down / enter) is only on the dispatch
        // path when the List itself is focused. Without this the palette
        // opens but keyboard selection is dead. Mirrors `gpui-component`'s
        // `Combobox`, which focuses its list every time the overlay opens.
        palette.update(cx, |palette, cx| palette.focus_list(window, cx));
    }

    /// Current buffer text. Exposed so tests can assert that typed input
    /// reached the buffer (the wave's UAT condition).
    #[must_use]
    pub fn value(&self, cx: &Context<Self>) -> SharedString {
        self.state.read(cx).value()
    }

    /// The file the editor is currently backed by, or `None` for an
    /// ephemeral document (the embedded Welcome).
    ///
    /// The palette uses this to derive the vault root for its "Open File"
    /// listing: the directory holding the open file (Wave 6). A `None`
    /// path means there is no vault to list, so the palette shows an empty
    /// file search.
    #[must_use]
    pub fn current_path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }

    /// React to the input widget's events. Only `Change` matters here:
    /// it (re)arms the debounced autosave.
    fn on_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            let text = state.read(cx).value().to_string();
            self.schedule_save(text, cx);
        }
    }

    /// Arm (or re-arm) the debounced write. Each call replaces
    /// `pending_save`; dropping the previous `Task` cancels its still
    /// -sleeping timer, so rapid edits collapse into a single write that
    /// fires `AUTOSAVE_DEBOUNCE` after the *last* keystroke.
    ///
    /// An ephemeral document (`path == None`, e.g. the embedded Welcome)
    /// has nowhere to write back to, so we return early and never touch
    /// the filesystem — this is what keeps editing Welcome from creating
    /// a stray `Welcome` file in the working directory.
    fn schedule_save(&mut self, text: String, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else {
            return;
        };

        self.pending_save = cx.spawn(async move |_view, cx| {
            cx.background_executor().timer(AUTOSAVE_DEBOUNCE).await;

            // The write itself is blocking I/O; keep it off the main
            // thread by running it on the background executor.
            let result = cx
                .background_spawn(async move { Vault::save_raw(&path, &text) })
                .await;

            // Observability surface for the headless UAT: a log line per
            // outcome. (No GUI is available to eyeball the file in this
            // session — the save_raw round-trip test is the hard proof;
            // these lines prove the *trigger* fired end to end when run
            // interactively.)
            match result {
                Ok(()) => eprintln!("limn-ui: autosaved"),
                Err(e) => eprintln!("limn-ui: autosave failed: {e}"),
            }
        });
    }
}

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x00fa_f9f6);
        let fg = rgb(0x001a_1a1a);

        div()
            // Put the `EditorView` on the window's focus/dispatch tree so
            // its `on_action` handler is reachable. `key_context("Editor")`
            // names the context after the view (ADR-0008: context name =
            // view name; the gpui-component `Root`'s own context is not
            // used for limn actions). `on_action` binds the action type to
            // this view's handler via `cx.listener`.
            .track_focus(&self.focus_handle)
            .key_context("Editor")
            .on_action(cx.listener(Self::on_toggle_palette))
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
            // Render the gpui-component Dialog overlay layer. The
            // component's `Root` does NOT draw active dialogs itself
            // (`Root::render` omits the dialog layer), so without this the
            // palette's `open_dialog` would mutate state but paint
            // nothing. `render_dialog_layer` returns `None` when no dialog
            // is open, so this is inert until the palette is toggled.
            .children(Root::render_dialog_layer(window, cx))
    }
}
