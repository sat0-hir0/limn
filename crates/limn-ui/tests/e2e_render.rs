//! E2E render test for the M1 vertical slice.
//!
//! Boots a headless gpui `TestAppContext`, opens a window backed by
//! `DocumentView`, renders one frame, and asserts the title string the
//! view exposes — the closest we can get to "the user saw 'Hello'" at
//! the M1 testing surface. Driving real keyboard input lands in M2.
//!
//! This test runs without a GPU because gpui's `TestPlatform` swaps
//! the real display for an in-memory stub. If the gpui API moves
//! enough that the test stops compiling, prefer fixing it over
//! falling back to a bin smoke test — the testing-strategy doc
//! explicitly asks for the gpui-side variant of E2E.

use gpui::{AppContext as _, Entity, TestAppContext};
use gpui_component::{Root, WindowExt as _};

use limn_core::block::Block;
use limn_service::LimnConfig;
use limn_ui::actions::{CloseSettings, OpenSettings, TogglePalette};
use limn_ui::{
    AppConfig, AppShell, DocumentView, EditorView, FeatureFlags, ScreenKind, SettingsView,
};

#[gpui::test]
fn document_view_titles_and_blocks_round_trip_through_a_test_window(cx: &mut TestAppContext) {
    let blocks = vec![
        Block::heading(1, "Hello, Limn"),
        Block::paragraph("It works."),
    ];

    let window = cx.add_window(|_, _cx| DocumentView {
        title: "hello.md".into(),
        blocks: blocks.clone(),
    });

    cx.run_until_parked();

    // Read state back from the view to prove the entity made it through
    // the gpui run loop intact (window opened, view created, frame
    // scheduled). Asserting the rendered element tree itself needs
    // public APIs gpui doesn't expose at this rev; the view's own data
    // is the next-best contract.
    window
        .update(cx, |view, _window, _cx| {
            assert_eq!(view.title.as_ref(), "hello.md");
            assert_eq!(view.blocks, blocks);
        })
        .expect("window update should succeed");
}

/// Shared bootstrap for the Wave 8 tests: install the globals and
/// keybindings the editable shell expects (`gpui_component::init`,
/// `FeatureFlags`, `AppConfig`, the action keybindings). The production
/// entry point in `main.rs` does the same; running it here lets us build
/// real `EditorView` / `SettingsView` / `AppShell` entities inside the
/// `TestAppContext`.
fn install_globals(cx: &mut TestAppContext, config: LimnConfig) {
    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_global(FeatureFlags::default());
        cx.set_global(AppConfig(config));
        limn_ui::actions::bind_keys(cx);
    });
}

#[gpui::test]
fn app_shell_screen_swaps_on_open_and_close_settings_actions(cx: &mut TestAppContext) {
    install_globals(cx, LimnConfig::default());

    // Build a real editor (with no backing file — Welcome-style), wrap it
    // in the shell, and wrap the shell in a `Root` — the same layering as
    // `run_editable` in `main.rs`. The editor is focused so the dispatch
    // tree (Input → editor → shell → Root) is populated; without a
    // focused descendant, gpui has no path on which to find the shell's
    // `on_action` handlers, and `render_dialog_layer` panics when called
    // off a window whose root view is not `Root`.
    //
    // We use `update_window` to also obtain a strong `Entity<AppShell>`
    // outside the window's view tree, so the test can read the shell's
    // `screen_kind()` directly between dispatches.
    let mut shell_entity: Option<Entity<AppShell>> = None;
    let window = cx.add_window(|window, cx| {
        let editor = cx.new(|cx| EditorView::new("test.md", None, "hello".to_string(), window, cx));
        editor.update(cx, |view, cx| view.focus(window, cx));
        let shell = cx.new(|cx| AppShell::new(editor, window, cx));
        shell_entity = Some(shell.clone());
        Root::new(shell, window, cx)
    });
    let shell = shell_entity.expect("shell entity captured during window build");

    cx.run_until_parked();

    // Starts on the editor screen.
    cx.update_window(window.into(), |_, _window, cx| {
        assert_eq!(shell.read(cx).screen_kind(), ScreenKind::Editor);
    })
    .expect("read initial screen");

    // `dispatch_action` on the focused window dispatches into the shell's
    // handlers because `AppShell` is on the focus tree (via track_focus).
    cx.dispatch_action(window.into(), OpenSettings);
    cx.run_until_parked();

    cx.update_window(window.into(), |_, _window, cx| {
        assert_eq!(shell.read(cx).screen_kind(), ScreenKind::Settings);
    })
    .expect("read settings screen");

    cx.dispatch_action(window.into(), CloseSettings);
    cx.run_until_parked();

    cx.update_window(window.into(), |_, _window, cx| {
        assert_eq!(shell.read(cx).screen_kind(), ScreenKind::Editor);
    })
    .expect("read editor screen after close");
}

#[gpui::test]
fn settings_view_save_writes_disk_and_updates_app_config_global(cx: &mut TestAppContext) {
    install_globals(cx, LimnConfig::default());

    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.toml");
    let vault_path = tmp.path().join("notes");

    // Build a real SettingsView in a window so the InputState entities
    // have a Window to register against.
    let window = cx.add_window(SettingsView::new);
    cx.run_until_parked();

    let expected = LimnConfig {
        font: limn_service::FontConfig {
            family: "Fira Code".to_string(),
            size: 18,
        },
        theme: limn_service::Theme::Light,
        vault_path: Some(vault_path.clone()),
    };
    let expected_for_assert = expected.clone();

    // Drive the inputs as a user would (modulo keyboard plumbing) and
    // save through the test-only path that lets us point at a tempfile.
    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs(
                &expected.font.family,
                &expected.font.size.to_string(),
                &expected.vault_path.as_ref().unwrap().to_string_lossy(),
                matches!(expected.theme, limn_service::Theme::Dark),
                window,
                cx,
            );
            let ok = view.save_to_path(&config_path, cx);
            assert!(ok, "save should succeed");
        })
        .expect("update settings view");

    cx.run_until_parked();

    // Assert: the file on disk parses back to `expected`, and the live
    // `AppConfig` global was updated to match.
    let on_disk = LimnConfig::load_from(&config_path);
    assert_eq!(on_disk, expected_for_assert, "on-disk config matches draft");

    cx.update(|cx| {
        let live = &cx.global::<AppConfig>().0;
        assert_eq!(live, &expected_for_assert, "AppConfig global updated");
    });

    // Sanity: the view's own draft equals what was saved.
    window
        .update(cx, |view, _window, _cx| {
            assert_eq!(view.draft(), &expected_for_assert);
        })
        .expect("read draft back");
}

/// `TogglePalette` is bound globally so it can fire while the settings
/// screen is active. The shell intentionally drops the request in that
/// case — settings has no palette-mode commands and the dialog would
/// otherwise compete with the Esc-to-close binding scoped to
/// `"Settings"` (ADR-0010). This test pins the no-op behaviour so a
/// future palette extension cannot regress it accidentally.
#[gpui::test]
fn toggle_palette_is_a_no_op_while_settings_screen_is_active(cx: &mut TestAppContext) {
    // `palette: true` matches the gating in `AppShell::on_toggle_palette`;
    // without the flag the handler exits before the Screen check, which
    // would make this test pass for the wrong reason.
    let flags = FeatureFlags {
        palette: true,
        ..FeatureFlags::default()
    };
    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_global(flags);
        cx.set_global(AppConfig(LimnConfig::default()));
        limn_ui::actions::bind_keys(cx);
    });

    let mut shell_entity: Option<Entity<AppShell>> = None;
    let window = cx.add_window(|window, cx| {
        let editor = cx.new(|cx| EditorView::new("test.md", None, "hello".to_string(), window, cx));
        editor.update(cx, |view, cx| view.focus(window, cx));
        let shell = cx.new(|cx| AppShell::new(editor, window, cx));
        shell_entity = Some(shell.clone());
        Root::new(shell, window, cx)
    });
    let shell = shell_entity.expect("shell entity captured during window build");

    cx.run_until_parked();

    // Move to the settings screen so the Screen::Editor guard is the only
    // thing protecting the no-op.
    cx.dispatch_action(window.into(), OpenSettings);
    cx.run_until_parked();
    cx.update_window(window.into(), |_, _window, cx| {
        assert_eq!(shell.read(cx).screen_kind(), ScreenKind::Settings);
    })
    .expect("settings screen active");

    // Dispatch the palette toggle. The shell must stay on the settings
    // screen — no swap, no dialog opened underneath.
    cx.dispatch_action(window.into(), TogglePalette);
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        assert_eq!(
            shell.read(cx).screen_kind(),
            ScreenKind::Settings,
            "TogglePalette must not change screen on settings"
        );
        assert!(
            !window.has_active_dialog(cx),
            "TogglePalette must not open a dialog on settings"
        );
    })
    .expect("post-toggle assertions");
}
