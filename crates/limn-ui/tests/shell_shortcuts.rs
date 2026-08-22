//! Shell shortcut dispatch tests.
//!
//! Exercises the keystroke to view table in `limn_ui::shell::SHORTCUTS`
//! through gpui's simulated keystroke path: bind the keys, focus the
//! `Shell` root view inside a headless test window, type the
//! keystroke, and read `Shell::view` back.

use gpui::{Focusable, TestAppContext, WindowHandle};

use limn_core::block::Block;
use limn_ui::shell::{key_bindings, Shell, ShellView, SHORTCUTS};

fn open_shell(cx: &mut TestAppContext) -> WindowHandle<Shell> {
    cx.update(|cx| cx.bind_keys(key_bindings()));

    let window = cx.add_window(|window, cx| {
        let shell = Shell::new(
            "hello.md".into(),
            vec![Block::heading(1, "Hello, Limn")],
            cx,
        );
        shell.focus_handle(cx).focus(window, cx);
        shell
    });

    cx.run_until_parked();
    window
}

fn assert_view(window: &WindowHandle<Shell>, cx: &mut TestAppContext, expected: ShellView) {
    window
        .update(cx, |shell, _window, _cx| {
            assert_eq!(shell.view, expected);
        })
        .expect("window update should succeed");
}

#[gpui::test]
fn editor_ctrl_shift_p_shows_the_palette(cx: &mut TestAppContext) {
    let window = open_shell(cx);

    cx.simulate_keystrokes(window.into(), "ctrl-shift-p");

    assert_view(&window, cx, ShellView::Palette);
}

#[gpui::test]
fn editor_ctrl_comma_shows_settings(cx: &mut TestAppContext) {
    let window = open_shell(cx);

    cx.simulate_keystrokes(window.into(), "ctrl-,");

    assert_view(&window, cx, ShellView::Settings);
}

#[gpui::test]
fn palette_escape_returns_to_the_editor(cx: &mut TestAppContext) {
    let window = open_shell(cx);
    cx.simulate_keystrokes(window.into(), "ctrl-shift-p");
    assert_view(&window, cx, ShellView::Palette);

    cx.simulate_keystrokes(window.into(), "escape");

    assert_view(&window, cx, ShellView::Editor);
}

#[gpui::test]
fn settings_escape_returns_to_the_editor(cx: &mut TestAppContext) {
    let window = open_shell(cx);
    cx.simulate_keystrokes(window.into(), "ctrl-,");
    assert_view(&window, cx, ShellView::Settings);

    cx.simulate_keystrokes(window.into(), "escape");

    assert_view(&window, cx, ShellView::Editor);
}

#[gpui::test]
fn settings_ctrl_shift_p_shows_the_palette(cx: &mut TestAppContext) {
    let window = open_shell(cx);
    cx.simulate_keystrokes(window.into(), "ctrl-,");
    assert_view(&window, cx, ShellView::Settings);

    cx.simulate_keystrokes(window.into(), "ctrl-shift-p");

    assert_view(&window, cx, ShellView::Palette);
}

#[gpui::test]
fn palette_ctrl_comma_shows_settings(cx: &mut TestAppContext) {
    let window = open_shell(cx);
    cx.simulate_keystrokes(window.into(), "ctrl-shift-p");
    assert_view(&window, cx, ShellView::Palette);

    cx.simulate_keystrokes(window.into(), "ctrl-,");

    assert_view(&window, cx, ShellView::Settings);
}

#[gpui::test]
fn editor_escape_is_idempotent(cx: &mut TestAppContext) {
    let window = open_shell(cx);

    cx.simulate_keystrokes(window.into(), "escape");

    assert_view(&window, cx, ShellView::Editor);
}

#[gpui::test]
fn document_entity_survives_a_view_round_trip(cx: &mut TestAppContext) {
    let window = open_shell(cx);

    cx.simulate_keystrokes(window.into(), "ctrl-shift-p");
    cx.simulate_keystrokes(window.into(), "escape");

    window
        .update(cx, |shell, _window, cx| {
            assert_eq!(shell.document.read(cx).title.as_ref(), "hello.md");
        })
        .expect("window update should succeed");
}

/// No `TestAppContext` here: this proves every keystroke literal in
/// `SHORTCUTS` parses, since `KeyBinding::new` panics otherwise.
#[test]
fn key_bindings_cover_every_shortcut() {
    assert_eq!(key_bindings().len(), SHORTCUTS.len());
}
