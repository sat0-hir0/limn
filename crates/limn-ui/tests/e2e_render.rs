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

use gpui::TestAppContext;

use limn_core::block::Block;
use limn_service::{Config, Theme};
use limn_ui::{ActiveView, DocumentView, SaveStatus, SettingsView, Shell};

#[gpui::test]
fn document_view_titles_and_blocks_round_trip_through_a_test_window(cx: &mut TestAppContext) {
    let blocks = vec![
        Block::heading(1, "Hello, Limn"),
        Block::paragraph("It works."),
    ];

    let window = cx.add_window(|_, _cx| DocumentView {
        title: "hello.md".into(),
        blocks: blocks.clone(),
        config: Config::default(),
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

#[gpui::test]
fn document_view_renders_with_a_dark_theme_config(cx: &mut TestAppContext) {
    let blocks = vec![Block::paragraph("Dark mode.")];
    let config = Config {
        theme: Theme::Dark,
        ..Default::default()
    };

    let window = cx.add_window(|_, _cx| DocumentView {
        title: "dark.md".into(),
        blocks: blocks.clone(),
        config: config.clone(),
    });

    cx.run_until_parked();

    // The dark-theme config survives construction and the run loop
    // renders a frame without panicking (render() branches on
    // `config.theme` to pick colours). gpui exposes no public handle to
    // the resolved pixels at this rev, so the view's own config is the
    // contract we can assert.
    window
        .update(cx, |view, _window, _cx| {
            assert_eq!(view.config.theme, Theme::Dark);
            assert_eq!(view.config, config);
        })
        .expect("window update should succeed");
}

#[gpui::test]
fn settings_view_renders_and_holds_its_config(cx: &mut TestAppContext) {
    let config = Config {
        theme: Theme::Dark,
        font_family: "Fira Code".to_string(),
        font_size: 16.5,
        vault_path: Some(std::path::PathBuf::from("/home/user/notes")),
    };

    let window = cx.add_window(|_, _cx| SettingsView::new(config.clone()));

    cx.run_until_parked();

    // The settings view construction survives the run loop (window
    // opened, view created, one frame rendered without panicking) and
    // the working config made it through intact. gpui exposes no public
    // handle to the resolved pixels at this rev, so the view's own state
    // is the contract we can assert.
    window
        .update(cx, |view, _window, _cx| {
            assert_eq!(view.config, config);
            assert_eq!(view.status, SaveStatus::Idle);
        })
        .expect("window update should succeed");
}

#[test]
fn toggle_theme_flips_between_light_and_dark() {
    let mut view = SettingsView::new(Config::default());
    assert_eq!(view.config.theme, Theme::Light);

    view.toggle_theme();
    assert_eq!(view.config.theme, Theme::Dark);

    view.toggle_theme();
    assert_eq!(view.config.theme, Theme::Light);
}

#[test]
fn edited_working_config_round_trips_to_disk() {
    // `SettingsView::save` writes to the real config path, which tests
    // must not touch. Exercise the same working-config-to-disk path via
    // `save_to`/`load_from` against a tempfile: an edit made through the
    // view's `toggle_theme` serializes and reloads intact.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("config.toml");

    let mut view = SettingsView::new(Config::default());
    view.toggle_theme();
    let edited = view.config.clone();

    edited.save_to(&path).expect("save_to should succeed");
    let loaded = Config::load_from(&path).expect("load_from should succeed");

    assert_eq!(loaded, edited);
    assert_eq!(loaded.theme, Theme::Dark);
}

fn welcome_document_view(config: Config) -> DocumentView {
    DocumentView {
        title: "Welcome".into(),
        blocks: vec![Block::paragraph("Hello.")],
        config,
    }
}

#[gpui::test]
fn shell_switches_between_document_and_settings(cx: &mut TestAppContext) {
    let window = cx.add_window(|_, cx| {
        Shell::new(
            welcome_document_view(Config::default()),
            ActiveView::Document,
            cx,
        )
    });
    cx.run_until_parked();

    window
        .update(cx, |shell, _window, cx| {
            assert_eq!(shell.active, ActiveView::Document);
            shell.show_settings(cx);
            assert_eq!(shell.active, ActiveView::Settings);
            shell.show_document(cx);
            assert_eq!(shell.active, ActiveView::Document);
        })
        .expect("window update should succeed");
    cx.run_until_parked();
}

#[gpui::test]
fn shell_starts_on_settings_when_asked(cx: &mut TestAppContext) {
    let window = cx.add_window(|_, cx| {
        Shell::new(
            welcome_document_view(Config::default()),
            ActiveView::Settings,
            cx,
        )
    });
    cx.run_until_parked();

    window
        .update(cx, |shell, _window, cx| {
            assert_eq!(shell.active, ActiveView::Settings);
            // Settings edits a copy of the document's config.
            assert_eq!(shell.settings.read(cx).config, Config::default());
        })
        .expect("window update should succeed");
}

#[gpui::test]
fn returning_to_the_document_applies_only_a_saved_config(cx: &mut TestAppContext) {
    let window = cx.add_window(|_, cx| {
        Shell::new(
            welcome_document_view(Config::default()),
            ActiveView::Settings,
            cx,
        )
    });
    cx.run_until_parked();

    window
        .update(cx, |shell, _window, cx| {
            // An unsaved edit stays in Settings.
            shell
                .settings
                .update(cx, |settings, _| settings.toggle_theme());
            shell.show_document(cx);
            assert_eq!(shell.document.read(cx).config.theme, Theme::Light);

            // A saved config reaches the document on return. `save()`
            // writes the real user config, which tests must not touch,
            // so record the saved state the way a successful Save does.
            shell.show_settings(cx);
            shell.settings.update(cx, |settings, _| {
                settings.last_saved = Some(settings.config.clone());
            });
            shell.show_document(cx);
            assert_eq!(shell.document.read(cx).config.theme, Theme::Dark);
        })
        .expect("window update should succeed");
    cx.run_until_parked();
}
