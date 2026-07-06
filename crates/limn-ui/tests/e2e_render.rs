//! E2E render tests for the vertical slice.
//!
//! Boots a headless gpui `TestAppContext`, opens a window backed by
//! `DocumentView` or `EditorView`, renders one frame, and asserts the
//! state the view exposes — the closest we can get to "the user saw
//! 'Hello'" at the current testing surface. Real keyboard input is not
//! exercised here yet.
//!
//! This test runs without a GPU because gpui's `TestPlatform` swaps
//! the real display for an in-memory stub. If the gpui API moves
//! enough that the test stops compiling, prefer fixing it over
//! falling back to a bin smoke test — the testing-strategy doc
//! explicitly asks for the gpui-side variant of E2E.

use gpui::TestAppContext;

use limn_core::block::Block;
use limn_ui::{DocumentView, EditorView};

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

#[gpui::test]
fn editor_view_seeds_input_state_with_the_document_text(cx: &mut TestAppContext) {
    // gpui_component widgets register keybindings and global state on
    // first use; without this the window never opens.
    cx.update(gpui_component::init);

    let window = cx.add_window(|window, cx| {
        EditorView::new(
            "hello.md".into(),
            "# Hello, Limn\n\nIt works.\n".to_string(),
            window,
            cx,
        )
    });

    cx.run_until_parked();

    window
        .update(cx, |view, _window, cx| {
            assert_eq!(view.title.as_ref(), "hello.md");
            assert_eq!(view.value(cx).as_ref(), "# Hello, Limn\n\nIt works.\n");
        })
        .expect("window update should succeed");
}
