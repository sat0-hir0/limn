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
    div, px, rgb, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Task, Window,
};
use gpui_component::input::{Input, InputEvent, InputState};

use limn_service::Vault;

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
