//! E2E tests for the command palette: real keystrokes through a
//! headless gpui window, from `Ctrl+Shift+P` to the main view changing.

use std::fs;

use gpui::{AppContext, Entity, Focusable, TestAppContext, VisualTestContext, WindowOptions};
use tempfile::tempdir;

use limn_core::block::Block;
use limn_service::Vault;
use limn_ui::palette::Mode;
use limn_ui::workspace::{self, MainView, Workspace};
use limn_ui::DocumentView;

fn open_workspace(
    cx: &mut TestAppContext,
    vault: Option<Vault>,
) -> (Entity<Workspace>, VisualTestContext) {
    cx.update(workspace::bind_keys);
    let window = cx.update(|cx| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            let document = DocumentView {
                title: "start.md".into(),
                blocks: vec![Block::paragraph("start")],
            };
            let workspace = cx.new(|cx| Workspace::new(document, vault, cx));
            workspace.focus_handle(cx).focus(window, cx);
            workspace
        })
        .unwrap()
    });
    let mut cx = VisualTestContext::from_window(window.into(), cx);
    let workspace = window.root(&mut cx).unwrap();
    (workspace, cx)
}

fn main_title(workspace: &Entity<Workspace>, cx: &VisualTestContext) -> String {
    workspace.read_with(cx, |ws, _| match &ws.main {
        MainView::Document(doc) => doc.title.to_string(),
        MainView::Settings => "<settings>".to_string(),
    })
}

#[gpui::test]
fn ctrl_shift_p_toggles_and_escape_closes(cx: &mut TestAppContext) {
    let (workspace, mut cx) = open_workspace(cx, None);

    cx.simulate_keystrokes("ctrl-shift-p");
    assert!(workspace.read_with(&cx, |ws, _| ws.palette.is_some()));

    cx.simulate_keystrokes("ctrl-shift-p");
    assert!(workspace.read_with(&cx, |ws, _| ws.palette.is_none()));

    cx.simulate_keystrokes("ctrl-shift-p escape");
    assert!(workspace.read_with(&cx, |ws, _| ws.palette.is_none()));
}

#[gpui::test]
fn typing_filters_and_enter_opens_settings(cx: &mut TestAppContext) {
    let (workspace, mut cx) = open_workspace(cx, None);

    cx.simulate_keystrokes("ctrl-shift-p");
    cx.simulate_input("sett");
    let items = workspace.read_with(&cx, |ws, _| ws.palette.as_ref().unwrap().items());
    assert_eq!(items, vec!["Open Settings"]);

    cx.simulate_keystrokes("enter");
    assert_eq!(main_title(&workspace, &cx), "<settings>");
    assert!(workspace.read_with(&cx, |ws, _| ws.palette.is_none()));
}

#[gpui::test]
fn arrow_keys_move_the_selection(cx: &mut TestAppContext) {
    let (workspace, mut cx) = open_workspace(cx, None);

    cx.simulate_keystrokes("ctrl-shift-p down");
    assert_eq!(
        workspace.read_with(&cx, |ws, _| ws.palette.as_ref().unwrap().selected()),
        1
    );
    cx.simulate_keystrokes("enter");
    assert_eq!(main_title(&workspace, &cx), "<settings>");
}

#[gpui::test]
fn open_file_fuzzy_searches_the_vault_and_opens_the_choice(cx: &mut TestAppContext) {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("notes")).unwrap();
    fs::write(dir.path().join("start.md"), "start\n").unwrap();
    fs::write(dir.path().join("notes/todo.md"), "# Todo\n\nbuy milk\n").unwrap();
    let (workspace, mut cx) = open_workspace(cx, Some(Vault::new(dir.path())));

    cx.simulate_keystrokes("ctrl-shift-p");
    cx.simulate_input("open file");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();

    let files = workspace.read_with(&cx, |ws, _| ws.palette.as_ref().unwrap().mode().clone());
    assert!(matches!(files, Mode::Files { files: Some(ref f) } if f.len() == 2));

    cx.simulate_input("todo");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();

    assert_eq!(main_title(&workspace, &cx), "todo.md");
    let blocks = workspace.read_with(&cx, |ws, _| match &ws.main {
        MainView::Document(doc) => doc.blocks.clone(),
        MainView::Settings => Vec::new(),
    });
    assert_eq!(blocks[0].text, "Todo");
}

#[gpui::test]
fn backspace_edits_the_query(cx: &mut TestAppContext) {
    let (workspace, mut cx) = open_workspace(cx, None);

    cx.simulate_keystrokes("ctrl-shift-p");
    cx.simulate_input("setx");
    cx.simulate_keystrokes("backspace");
    let query = workspace.read_with(&cx, |ws, _| {
        ws.palette.as_ref().unwrap().query().to_string()
    });
    assert_eq!(query, "set");
}
