//! E2E render test for the current read-only vertical slice.
//!
//! Boots a headless gpui `TestAppContext`, opens a window backed by
//! `DocumentView`, renders one frame, and asserts the title string the
//! view exposes — the closest we can get to "the user saw 'Hello'" at
//! the current testing surface. Real keyboard input is not exercised
//! here yet.
//!
//! This test runs without a GPU because gpui's `TestPlatform` swaps
//! the real display for an in-memory stub. If the gpui API moves
//! enough that the test stops compiling, prefer fixing it over
//! falling back to a bin smoke test — the testing-strategy doc
//! explicitly asks for the gpui-side variant of E2E.

use gpui::{AppContext as _, Entity, TestAppContext};
use gpui_component::{Root, Theme, ThemeMode, WindowExt as _};

use limn_core::block::Block;
use limn_service::LimnConfig;
use limn_ui::actions::{CloseSettings, OpenSettings, TogglePalette};
use limn_ui::{
    AppConfig, AppShell, ColorTheme, ColorThemeGlobal, DocumentView, EditorView, FeatureFlags,
    ScreenKind, SettingsView,
};

#[gpui::test]
fn document_view_titles_and_blocks_round_trip_through_a_test_window(cx: &mut TestAppContext) {
    let blocks = vec![
        Block::heading(1, "Hello, Limn"),
        Block::paragraph("It works."),
    ];

    // Route through `install_globals` for consistency with the other
    // Wave 8+ tests. We use an explicit Light config so the installed
    // `ColorThemeGlobal` is `ColorTheme::paper()` ( preserving the
    // original test's intent — `Theme::default()` is `Dark`, which
    // would install `ColorTheme::ink()` instead ).
    let config = LimnConfig {
        theme: limn_service::Theme::Light,
        ..LimnConfig::default()
    };
    install_globals(cx, &config);

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
fn install_globals(cx: &mut TestAppContext, config: &LimnConfig) {
    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_global(FeatureFlags::default());
        cx.set_global(AppConfig(config.clone()));
        cx.set_global(ColorThemeGlobal(ColorTheme::from_config(config.theme)));
        limn_ui::actions::bind_keys(cx);
    });
}

#[gpui::test]
fn app_shell_screen_swaps_on_open_and_close_settings_actions(cx: &mut TestAppContext) {
    install_globals(cx, &LimnConfig::default());

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
fn settings_view_apply_writes_disk_and_updates_app_config_global(cx: &mut TestAppContext) {
    install_globals(cx, &LimnConfig::default());

    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.toml");
    let vault_path = tmp.path().join("notes");
    // Wave 11: `save_to_path` now rejects vault paths that do not exist on
    // disk (matches the production Enter-handler validation). Create the
    // directory so this happy-path test exercises a path that passes
    // validation, not the rejection branch the new tests below cover.
    std::fs::create_dir_all(&vault_path).expect("create vault dir");

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
        cx.set_global(ColorThemeGlobal(ColorTheme::paper()));
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

/// Wave 9 (Gap A): the gpui-component `Theme` global must track the saved
/// config. We install a Light config, prove the global reflects Light,
/// then save a Dark draft through the settings view and prove the global
/// flips to Dark — the dynamic-render path the settings `save_to_path`
/// now drives.
#[gpui::test]
fn theme_global_reflects_config_after_toggle(cx: &mut TestAppContext) {
    let light_config = LimnConfig {
        font: limn_service::FontConfig {
            family: String::new(),
            size: 14,
        },
        theme: limn_service::Theme::Light,
        vault_path: None,
    };
    install_globals(cx, &light_config);

    // Seed the gpui-component Theme global the way `run_editable` does at
    // startup, then assert it reflects the Light config.
    cx.update(|cx| {
        Theme::change(ThemeMode::Light, None, cx);
        assert_eq!(Theme::global(cx).mode, ThemeMode::Light);
    });

    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.toml");

    let window = cx.add_window(SettingsView::new);
    cx.run_until_parked();

    // Flip the draft to Dark and save through the sync test path, which
    // (Wave 9) applies the theme to the global via `Theme::change`.
    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs("", "14", "", true, window, cx);
            let ok = view.save_to_path(&config_path, cx);
            assert!(ok, "save should succeed");
        })
        .expect("update settings view");

    cx.run_until_parked();

    cx.update(|cx| {
        assert_eq!(
            Theme::global(cx).mode,
            ThemeMode::Dark,
            "Theme global must flip to Dark after saving a Dark draft"
        );
    });
}

/// Wave 9 (Gap B): toggling the palette opens it as a searchable dialog.
/// We can assert the dialog is open; cursor-in-input focus is not
/// verifiable under headless gpui (no real event loop / text system), so
/// this test pins the "opens, and a dialog is active" contract that the
/// `searchable(true)` change preserves.
#[gpui::test]
fn toggle_palette_opens_with_searchable_list(cx: &mut TestAppContext) {
    let flags = FeatureFlags {
        palette: true,
        ..FeatureFlags::default()
    };
    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_global(flags);
        cx.set_global(AppConfig(LimnConfig::default()));
        cx.set_global(ColorThemeGlobal(ColorTheme::paper()));
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
    let _shell = shell_entity.expect("shell entity captured during window build");

    cx.run_until_parked();

    // From the editor screen, toggling the palette must open the dialog.
    cx.dispatch_action(window.into(), TogglePalette);
    cx.run_until_parked();

    cx.update_window(window.into(), |_, window, cx| {
        assert!(
            window.has_active_dialog(cx),
            "TogglePalette must open the palette dialog on the editor screen"
        );
        // Note: the palette list now opens `searchable(true)` (Wave 9), so
        // a search input is present and the list filters commands as the
        // user types. Asserting the cursor actually lands in that input is
        // not possible under headless gpui (no text-input/event loop), so
        // the assertable contract here is "a dialog is open".
    })
    .expect("post-toggle dialog assertion");
}

/// Wave 10-C: startup wires `ColorThemeGlobal` from `LimnConfig.theme`.
/// A Light config must produce the paper theme's semantic roles in the
/// installed global.
#[gpui::test]
fn color_theme_global_reflects_config_at_startup(cx: &mut TestAppContext) {
    let config = LimnConfig {
        theme: limn_service::Theme::Light,
        ..LimnConfig::default()
    };
    install_globals(cx, &config);
    cx.update(|cx| {
        let global = cx.global::<ColorThemeGlobal>();
        assert_eq!(global.0, ColorTheme::paper());
    });
}

/// Wave 10-C: a Dark config must produce the ink theme's semantic roles
/// in the installed global.
#[gpui::test]
fn color_theme_global_dark_config_gives_ink_values(cx: &mut TestAppContext) {
    let config = LimnConfig {
        theme: limn_service::Theme::Dark,
        ..LimnConfig::default()
    };
    install_globals(cx, &config);
    cx.update(|cx| {
        let global = cx.global::<ColorThemeGlobal>();
        assert_eq!(global.0, ColorTheme::ink());
    });
}

/// Wave 10-D drift-detection: Settings save must update BOTH globals
/// together — `ColorThemeGlobal` (which the Limn-side `EditorView` /
/// `DocumentView` read from per ADR-0011) and `gpui_component::Theme`
/// (which the rest of the gpui-component widget tree reads from). If
/// only one moves, the screen ends up in a mismatched half-applied
/// state.
///
/// **What this test verifies:** both globals advance to their
/// intended target on a single `save_to_path` call. The
/// `ColorThemeGlobal` is checked by *value* (`ColorTheme::ink()`),
/// the `gpui_component::Theme` is checked by *mode* (`ThemeMode::Dark`).
///
/// **What this test does NOT verify:** per-field value equality
/// between `ColorTheme` and `gpui_component::Theme`. The two systems
/// own independent palettes by design — gpui-component is upstream
/// and we deliberately do not fork it (see ADR-0011 Alternative A
/// rejection and its Negative-consequences discussion of drift). For
/// example, gpui-component's Dark `background` is `neutral-950`
/// (`#0a0a0a`) while `ColorTheme::ink().surface_app` is `n_850`
/// (`#1a1e22`); enforcing equality would either require forking
/// gpui-component or rewriting its globals, both of which ADR-0011
/// rules out. Deeper palette drift in upstream gpui-component is a
/// visual-UAT concern, not a unit-test one.
#[gpui::test]
fn save_updates_both_color_theme_global_and_gpui_component_theme(cx: &mut TestAppContext) {
    // Start on Light so the Dark draft is a real change.
    let light_config = LimnConfig {
        font: limn_service::FontConfig {
            family: String::new(),
            size: 14,
        },
        theme: limn_service::Theme::Light,
        vault_path: None,
    };
    install_globals(cx, &light_config);

    // Seed the gpui-component Theme global the way `run_editable` does at
    // startup so we have a Light baseline to flip off.
    cx.update(|cx| {
        Theme::change(ThemeMode::Light, None, cx);
        assert_eq!(Theme::global(cx).mode, ThemeMode::Light);
        assert_eq!(cx.global::<ColorThemeGlobal>().0, ColorTheme::paper());
    });

    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.toml");

    let window = cx.add_window(SettingsView::new);
    cx.run_until_parked();

    // Flip to Dark and save.
    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs("", "14", "", true, window, cx);
            let ok = view.save_to_path(&config_path, cx);
            assert!(ok, "save should succeed");
        })
        .expect("update settings view");

    cx.run_until_parked();

    // Both globals must have flipped to Dark / ink. If only one of these
    // assertions fails, save() updated one side but not the other —
    // exactly the drift this test exists to catch.
    cx.update(|cx| {
        assert_eq!(
            cx.global::<ColorThemeGlobal>().0,
            ColorTheme::ink(),
            "ColorThemeGlobal must flip to ink after saving a Dark draft",
        );
        assert_eq!(
            Theme::global(cx).mode,
            ThemeMode::Dark,
            "gpui_component::Theme must flip to Dark after saving a Dark draft",
        );
    });
}

/// Wave 10-D inverse drift test: Light → Dark → Light round-trips both
/// globals. Catches a `save()` that handles one direction (e.g. only
/// flipping to Dark) but leaves stale Dark values when the user goes
/// back to Light.
///
/// **What this test verifies:** both globals flip together in BOTH
/// directions of a round-trip — neither one lags or sticks.
///
/// **What this test does NOT verify:** per-field value equality
/// across the two systems (see
/// `save_updates_both_color_theme_global_and_gpui_component_theme`
/// for the rationale — gpui-component owns its palette upstream and
/// we do not fork it).
#[gpui::test]
fn save_round_trips_both_globals_light_to_dark_to_light(cx: &mut TestAppContext) {
    let light_config = LimnConfig {
        font: limn_service::FontConfig {
            family: String::new(),
            size: 14,
        },
        theme: limn_service::Theme::Light,
        vault_path: None,
    };
    install_globals(cx, &light_config);

    cx.update(|cx| {
        Theme::change(ThemeMode::Light, None, cx);
    });

    let tmp = tempfile::tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.toml");

    let window = cx.add_window(SettingsView::new);
    cx.run_until_parked();

    // Light → Dark.
    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs("", "14", "", true, window, cx);
            assert!(
                view.save_to_path(&config_path, cx),
                "Dark save should succeed"
            );
        })
        .expect("update settings view (Dark)");
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(cx.global::<ColorThemeGlobal>().0, ColorTheme::ink());
        assert_eq!(Theme::global(cx).mode, ThemeMode::Dark);
    });

    // Dark → Light.
    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs("", "14", "", false, window, cx);
            assert!(
                view.save_to_path(&config_path, cx),
                "Light save should succeed"
            );
        })
        .expect("update settings view (Light)");
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(
            cx.global::<ColorThemeGlobal>().0,
            ColorTheme::paper(),
            "ColorThemeGlobal must flip back to paper on Dark→Light",
        );
        assert_eq!(
            Theme::global(cx).mode,
            ThemeMode::Light,
            "gpui_component::Theme must flip back to Light on Dark→Light",
        );
    });
}

/// Wave 11: invalid font size text must be rejected — neither disk nor
/// the live `AppConfig` global moves, and the previous valid value (14)
/// stays in effect. Mirrors the auto-apply path's "last valid value
/// wins" rule (VS Code pattern) on the sync test surface.
#[gpui::test]
fn settings_apply_skips_invalid_font_size_and_preserves_last_valid(cx: &mut TestAppContext) {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("config.toml");
    let initial = LimnConfig {
        font: limn_service::FontConfig {
            family: String::new(),
            size: 14,
        },
        theme: limn_service::Theme::Light,
        vault_path: None,
    };
    install_globals(cx, &initial);

    let window = cx.add_window(SettingsView::new);
    cx.run_until_parked();

    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs("", "abc", "", false, window, cx);
            let ok = view.save_to_path(&config_path, cx);
            assert!(!ok, "save_to_path must return false on invalid font size");
        })
        .expect("update settings view (invalid font size)");

    cx.run_until_parked();

    cx.update(|cx| {
        let cfg = &cx.global::<AppConfig>().0;
        assert_eq!(
            cfg.font.size, 14,
            "AppConfig.font.size must be unchanged when invalid input was rejected",
        );
    });
}

/// Wave 11: a vault path that does not exist on disk must be rejected
/// (matches the production Enter-handler validation in
/// `SettingsView::new`). The previous valid value (`None`) stays in
/// effect and `AppConfig` is untouched.
#[gpui::test]
fn settings_apply_skips_nonexistent_vault_path(cx: &mut TestAppContext) {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("config.toml");
    let initial = LimnConfig {
        font: limn_service::FontConfig {
            family: String::new(),
            size: 14,
        },
        theme: limn_service::Theme::Light,
        vault_path: None,
    };
    install_globals(cx, &initial);

    let window = cx.add_window(SettingsView::new);
    cx.run_until_parked();

    window
        .update(cx, |view, window, cx| {
            view.set_test_inputs(
                "",
                "14",
                "C:/nonexistent/path/that/should/not/exist/12345",
                false,
                window,
                cx,
            );
            let ok = view.save_to_path(&config_path, cx);
            assert!(
                !ok,
                "save_to_path must return false on nonexistent vault path",
            );
        })
        .expect("update settings view (nonexistent vault)");

    cx.run_until_parked();

    cx.update(|cx| {
        let cfg = &cx.global::<AppConfig>().0;
        assert!(
            cfg.vault_path.is_none(),
            "AppConfig.vault_path must remain None when invalid input was rejected",
        );
    });
}
