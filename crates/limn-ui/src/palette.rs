//! Command palette state.
//!
//! Plain data with no gpui types, so the open / filter / select /
//! confirm flow is unit-testable without a window. The workspace view
//! owns one of these while the palette is open and turns each
//! [`Outcome`] into a side effect.
//!
//! The palette has two modes: the command list, and the file list that
//! the "Open File" command switches to. Adding a command means adding
//! a [`Command`] variant and its row in [`Command::ALL`]; the confirm
//! side is handled by whoever matches on [`Outcome::Run`].

use std::path::PathBuf;

use limn_core::fuzzy;

/// A command the palette can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    OpenFile,
    OpenSettings,
}

impl Command {
    /// Every command, in the order shown for an empty query.
    pub const ALL: &'static [Command] = &[Command::OpenFile, Command::OpenSettings];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Command::OpenFile => "Open File",
            Command::OpenSettings => "Open Settings",
        }
    }
}

/// Which list the palette is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Commands,
    /// `files` is `None` while the vault walk is still running.
    Files {
        files: Option<Vec<PathBuf>>,
    },
}

/// What the owner should do after a confirm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing to do (e.g. confirm on an empty result list).
    None,
    /// A command was chosen. The palette has already switched mode if
    /// the command needs one (Open File); the owner runs the rest.
    Run(Command),
    /// A file was chosen; the path is relative to the vault root.
    OpenFile(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Palette {
    mode: Mode,
    query: String,
    selected: usize,
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl Palette {
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: Mode::Commands,
            query: String::new(),
            selected: 0,
        }
    }

    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }

    #[must_use]
    pub fn placeholder(&self) -> &'static str {
        match self.mode {
            Mode::Commands => "Type a command",
            Mode::Files { .. } => "Search files by name",
        }
    }

    /// The rows to show for the current query, best match first.
    #[must_use]
    pub fn items(&self) -> Vec<String> {
        match &self.mode {
            Mode::Commands => fuzzy::filter(&self.query, Command::ALL, |c| c.label())
                .into_iter()
                .map(|i| Command::ALL[i].label().to_string())
                .collect(),
            Mode::Files { files: Some(files) } => {
                let labels: Vec<String> = files.iter().map(|p| display_path(p)).collect();
                fuzzy::filter(&self.query, &labels, String::as_str)
                    .into_iter()
                    .map(|i| labels[i].clone())
                    .collect()
            }
            Mode::Files { files: None } => Vec::new(),
        }
    }

    pub fn push_str(&mut self, text: &str) {
        self.query.push_str(text);
        self.selected = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.selected = 0;
    }

    pub fn select_next(&mut self) {
        let len = self.items().len();
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    pub fn select_prev(&mut self) {
        let len = self.items().len();
        if len > 0 {
            self.selected = (self.selected + len - 1) % len;
        }
    }

    /// Hand the vault's file list to the palette once the walk ends.
    /// Ignored if the user has left file mode in the meantime.
    pub fn set_files(&mut self, files: Vec<PathBuf>) {
        if let Mode::Files { files: slot } = &mut self.mode {
            *slot = Some(files);
            self.selected = 0;
        }
    }

    /// Confirm the selected row.
    pub fn confirm(&mut self) -> Outcome {
        match &self.mode {
            Mode::Commands => {
                let hits = fuzzy::filter(&self.query, Command::ALL, |c| c.label());
                let Some(&index) = hits.get(self.selected) else {
                    return Outcome::None;
                };
                let command = Command::ALL[index];
                if command == Command::OpenFile {
                    self.mode = Mode::Files { files: None };
                    self.query.clear();
                    self.selected = 0;
                }
                Outcome::Run(command)
            }
            Mode::Files { files: Some(files) } => {
                let labels: Vec<String> = files.iter().map(|p| display_path(p)).collect();
                let hits = fuzzy::filter(&self.query, &labels, String::as_str);
                hits.get(self.selected)
                    .map_or(Outcome::None, |&i| Outcome::OpenFile(files[i].clone()))
            }
            Mode::Files { files: None } => Outcome::None,
        }
    }
}

/// Render a vault-relative path with `/` separators on every platform.
fn display_path(path: &std::path::Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> Vec<PathBuf> {
        vec![
            ["daily", "2026-11-01.md"].iter().collect(),
            ["notes", "todo.md"].iter().collect(),
            "readme.md".into(),
        ]
    }

    #[test]
    fn empty_query_lists_every_command() {
        let palette = Palette::new();
        assert_eq!(palette.items(), vec!["Open File", "Open Settings"]);
    }

    #[test]
    fn typing_filters_commands_incrementally() {
        let mut palette = Palette::new();
        palette.push_str("set");
        assert_eq!(palette.items(), vec!["Open Settings"]);
        palette.backspace();
        palette.backspace();
        palette.backspace();
        assert_eq!(palette.items().len(), 2);
    }

    #[test]
    fn selection_wraps_in_both_directions() {
        let mut palette = Palette::new();
        palette.select_prev();
        assert_eq!(palette.selected(), 1);
        palette.select_next();
        assert_eq!(palette.selected(), 0);
    }

    #[test]
    fn confirming_open_settings_runs_it() {
        let mut palette = Palette::new();
        palette.push_str("settings");
        assert_eq!(palette.confirm(), Outcome::Run(Command::OpenSettings));
    }

    #[test]
    fn confirming_open_file_switches_to_file_mode() {
        let mut palette = Palette::new();
        palette.push_str("open f");
        assert_eq!(palette.confirm(), Outcome::Run(Command::OpenFile));
        assert_eq!(palette.mode(), &Mode::Files { files: None });
        assert_eq!(palette.query(), "");
        // Still loading: nothing to confirm yet.
        assert_eq!(palette.confirm(), Outcome::None);
    }

    #[test]
    fn file_mode_fuzzy_searches_and_opens_the_selection() {
        let mut palette = Palette::new();
        palette.confirm();
        palette.set_files(files());
        assert_eq!(palette.items().len(), 3);

        palette.push_str("todo");
        assert_eq!(palette.items(), vec!["notes/todo.md"]);
        assert_eq!(
            palette.confirm(),
            Outcome::OpenFile(["notes", "todo.md"].iter().collect())
        );
    }

    #[test]
    fn confirm_with_no_match_does_nothing() {
        let mut palette = Palette::new();
        palette.push_str("zzzz");
        assert_eq!(palette.confirm(), Outcome::None);
    }

    #[test]
    fn late_file_list_is_ignored_outside_file_mode() {
        let mut palette = Palette::new();
        palette.set_files(files());
        assert_eq!(palette.mode(), &Mode::Commands);
    }
}
