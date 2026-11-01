//! Autosave debounce: rapid edits collapse into one write once the
//! text has been quiet for `AUTOSAVE_DEBOUNCE`, and the Welcome
//! document (no save target) never writes.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use gpui::{AppContext, Entity, TestAppContext};
use tempfile::tempdir;

use limn_ui::{Autosave, AUTOSAVE_DEBOUNCE};

/// Stand-in for the editor view: owns the autosave state.
struct Editor {
    autosave: Autosave,
}

fn editor(cx: &mut TestAppContext, target: Option<PathBuf>) -> Entity<Editor> {
    cx.new(|_| Editor {
        autosave: Autosave::new(target),
    })
}

fn type_text(cx: &mut TestAppContext, editor: &Entity<Editor>, text: &str) {
    let text = text.to_string();
    editor.update(cx, |editor, cx| editor.autosave.schedule(text, cx));
}

/// Run every task that is ready now, without moving the clock forward
/// (`run_until_parked` would jump to the next pending timer).
fn drain_ready(cx: &TestAppContext) {
    while cx.executor().tick() {}
}

#[gpui::test]
fn rapid_edits_are_written_once_after_the_debounce_window(cx: &mut TestAppContext) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "original\n").unwrap();
    let editor = editor(cx, Some(path.clone()));

    type_text(cx, &editor, "first\n");
    cx.executor().advance_clock(Duration::from_millis(300));
    drain_ready(cx);
    type_text(cx, &editor, "second\n");

    // 600ms after the first edit, but only 300ms after the last one.
    cx.executor().advance_clock(Duration::from_millis(300));
    drain_ready(cx);
    assert_eq!(fs::read_to_string(&path).unwrap(), "original\n");

    cx.executor().advance_clock(AUTOSAVE_DEBOUNCE);
    cx.run_until_parked();
    assert_eq!(fs::read_to_string(&path).unwrap(), "second\n");
}

#[gpui::test]
fn welcome_document_without_a_target_never_writes(cx: &mut TestAppContext) {
    let dir = tempdir().unwrap();
    let editor = editor(cx, None);

    type_text(cx, &editor, "edited welcome\n");
    cx.executor().advance_clock(AUTOSAVE_DEBOUNCE * 2);
    cx.run_until_parked();

    editor.update(cx, |editor, _| assert!(!editor.autosave.is_enabled()));
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
