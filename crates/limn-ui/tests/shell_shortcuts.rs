//! Shell shortcut dispatch tests.
//!
//! Exercises the keystroke to view table in `limn_ui::shell::SHORTCUTS`
//! through gpui's simulated keystroke path: bind the keys, focus the
//! `Shell` root view inside a headless test window, type the
//! keystroke, and read `Shell::view` back.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{AnyWindowHandle, AppContext as _, Entity, Focusable, TestAppContext, WindowHandle};
use gpui_component::Root;

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

/// Open a window whose root is `gpui_component::Root` wrapping a
/// `Shell` with an editable editor slot — the same shape `main.rs`
/// builds when `LIMN_FEAT_EDIT` is on. `Input` looks up `Root` while
/// rendering, so the editor cannot be tested without it.
fn open_shell_with_editor(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Shell>) {
    // gpui_component widgets register keybindings and global state on
    // first use; without this the editor's input never initializes.
    cx.update(gpui_component::init);
    cx.update(|cx| cx.bind_keys(key_bindings()));

    let slot: Rc<RefCell<Option<Entity<Shell>>>> = Rc::default();
    let captured = slot.clone();
    let window = cx.add_window(move |window, cx| {
        let shell = cx.new(|cx| {
            Shell::with_editor(
                "hello.md".into(),
                vec![Block::heading(1, "Hello, Limn")],
                "# Hello, Limn
"
                .to_string(),
                window,
                cx,
            )
        });
        shell.update(cx, |shell, cx| shell.focus_initial(window, cx));
        *captured.borrow_mut() = Some(shell.clone());
        Root::new(shell, window, cx)
    });

    cx.run_until_parked();
    let shell = slot.borrow_mut().take().expect("shell should be created");
    (window.into(), shell)
}

#[gpui::test]
fn editor_slot_holds_the_editable_view_when_editing_is_enabled(cx: &mut TestAppContext) {
    let (_window, shell) = open_shell_with_editor(cx);

    shell.read_with(cx, |shell, cx| {
        assert_eq!(shell.view, ShellView::Editor);
        let editor = shell.editor.as_ref().expect("editor should be present");
        assert_eq!(
            editor.read(cx).value(cx).as_ref(),
            "# Hello, Limn
"
        );
    });
}

#[gpui::test]
fn shell_shortcuts_still_fire_while_the_editor_has_focus(cx: &mut TestAppContext) {
    let (window, shell) = open_shell_with_editor(cx);

    cx.simulate_keystrokes(window, "ctrl-shift-p");

    shell.read_with(cx, |shell, _cx| assert_eq!(shell.view, ShellView::Palette));
}

#[gpui::test]
fn shell_without_editing_keeps_the_read_only_view(cx: &mut TestAppContext) {
    let window = open_shell(cx);

    window
        .update(cx, |shell, _window, _cx| {
            assert!(shell.editor.is_none());
        })
        .expect("window update should succeed");
}
