//! Shell — the window's root view, switching between the document and
//! the Settings view.
//!
//! Per ADR-0005, Settings is a separate view the user moves to and away
//! from, never drawn over the document. The shell owns both views and
//! renders exactly one of them under a thin top bar whose control
//! switches to the other.

use gpui::{
    div, rgb, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window,
};

use limn_service::Theme;

use crate::{DocumentView, SettingsView};

/// Which view the shell is currently showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    /// The read-only document view.
    Document,
    /// The Settings view.
    Settings,
}

/// Root view holding the document and Settings views and showing one.
pub struct Shell {
    /// The view currently shown.
    pub active: ActiveView,
    /// The document view.
    pub document: Entity<DocumentView>,
    /// The Settings view, editing a working copy of the document's config.
    pub settings: Entity<SettingsView>,
}

impl Shell {
    /// Build a shell over `document`, starting on `active`. The Settings
    /// view starts from a copy of the document's config.
    pub fn new(document: DocumentView, active: ActiveView, cx: &mut Context<Self>) -> Self {
        let settings = SettingsView::new(document.config.clone());
        Self {
            active,
            document: cx.new(|_| document),
            settings: cx.new(|_| settings),
        }
    }

    /// Switch to the Settings view.
    pub fn show_settings(&mut self, cx: &mut Context<Self>) {
        self.active = ActiveView::Settings;
        cx.notify();
    }

    /// Switch back to the document view.
    ///
    /// If Settings saved a config, the document picks it up so the saved
    /// theme and font apply on return. Unsaved edits stay in Settings.
    pub fn show_document(&mut self, cx: &mut Context<Self>) {
        if let Some(saved) = self.settings.read(cx).last_saved.clone() {
            self.document
                .update(cx, |document, _| document.config = saved);
        }
        self.active = ActiveView::Document;
        cx.notify();
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (bar_background, bar_text) = match self.document.read(cx).config.theme {
            Theme::Light => (rgb(0x00ec_eae4), rgb(0x001a_1a1a)),
            Theme::Dark => (rgb(0x0026_2626), rgb(0x00f0_f0f0)),
        };

        let (label, body) = match self.active {
            ActiveView::Document => ("Settings", self.document.clone().into_any_element()),
            ActiveView::Settings => (
                "← Back to document",
                self.settings.clone().into_any_element(),
            ),
        };

        let switch = div()
            .id("view-switch")
            .px_3()
            .py_1()
            .cursor_pointer()
            .child(label)
            .on_click(
                cx.listener(|shell, _event, _window, cx| match shell.active {
                    ActiveView::Document => shell.show_settings(cx),
                    ActiveView::Settings => shell.show_document(cx),
                }),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .bg(bar_background)
                    .text_color(bar_text)
                    .text_sm()
                    .child(switch),
            )
            .child(div().flex_1().child(body))
    }
}
