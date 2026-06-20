//! Scanner: walk the repo, classify lines into debt categories.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Categories tracked by the scanner. Keep this list small and meaningful:
/// every entry should have a story for why it matters and a path to zero.
pub const CATEGORIES: &[&str] = &[
    "rust-loc",
    "external-deps",
    "todo-fixme",
    "allow-dead-code",
    "ignored-tests",
    "unsafe-blocks",
    "personal-windows-path",
];

const SCAN_DIRS: &[&str] = &["crates", "docs", ".github", "scripts"];
const SCAN_ROOT_FILES: &[&str] = &[
    "Cargo.toml",
    "README.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "CODE_OF_CONDUCT.md",
    "CHANGELOG.md",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "lefthook.yml",
    "deny.toml",
    ".gitleaks.toml",
];
const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    "reports",
    ".claude",
    ".cursor",
    ".codex",
    ".gemini",
    ".agents",
];

/// Paths whose contents we never scan: the scanner and the debt
/// dashboard talk about the patterns being looked for, which would
/// otherwise trip every detector on themselves. `.gitleaks.toml`
/// likewise embeds the patterns it watches for.
fn is_self_reference(rel: &Path) -> bool {
    let s = rel.to_string_lossy().replace('\\', "/");
    s.starts_with("crates/debt-scan/")
        || s == "docs/debt.md"
        || s.starts_with("docs/debt/")
        || s == ".gitleaks.toml"
}

/// Read the comma-separated `DEBT_SCAN_PERSONAL_NAMES` env var and return
/// the lowercased entries. Empty / unset → no personal-path detection.
///
/// This keeps personally-identifying strings (developer usernames, real
/// names) out of committed code. Each developer can opt in locally by
/// exporting e.g. `DEBT_SCAN_PERSONAL_NAMES=alice,a-bot`.
#[must_use]
pub fn personal_names_from_env() -> Vec<String> {
    std::env::var("DEBT_SCAN_PERSONAL_NAMES")
        .ok()
        .map(|s| {
            s.split(',')
                .map(|p| p.trim().to_lowercase())
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Run the scan rooted at `root` and return a category -> count map.
///
/// Personal-path detection is opt-in via the `DEBT_SCAN_PERSONAL_NAMES`
/// env var; without it the `personal-windows-path` count stays at zero.
///
/// # Errors
///
/// Returns an error if a Cargo manifest can't be read while counting
/// external dependencies.
pub fn scan(root: &Path) -> Result<BTreeMap<String, u64>> {
    let personal_names = personal_names_from_env();
    scan_with_personal_names(root, &personal_names)
}

/// Same as [`scan`] but takes the personal-names list explicitly. Useful
/// for tests; production callers should use [`scan`].
///
/// # Errors
///
/// Returns an error if a Cargo manifest can't be read while counting
/// external dependencies.
pub fn scan_with_personal_names(
    root: &Path,
    personal_names: &[String],
) -> Result<BTreeMap<String, u64>> {
    let mut counts: BTreeMap<String, u64> =
        CATEGORIES.iter().map(|c| ((*c).to_string(), 0)).collect();

    let mut walked_paths = Vec::new();
    for entry in walk(root) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        walked_paths.push(entry.path().to_path_buf());
    }

    for path in &walked_paths {
        let rel = path.strip_prefix(root).unwrap_or(path);
        if is_self_reference(rel) {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };

        if ext == "rs" && rel.starts_with("crates") {
            // Count code lines only inside our own crates, not target/ etc.
            let loc: u64 = text.lines().count() as u64;
            *counts.get_mut("rust-loc").unwrap() += loc;
        }

        for (idx, line) in text.lines().enumerate() {
            scan_line(rel, idx, line, ext, personal_names, &mut counts);
        }
    }

    counts.insert("external-deps".to_string(), count_external_deps(root)?);

    Ok(counts)
}

fn walk(root: &Path) -> impl Iterator<Item = walkdir::Result<DirEntry>> {
    // Two flavours of WalkDir iterator, so collect each as a Vec of
    // boxed dynamic iterators that share the same item type.
    type WalkIter = Box<dyn Iterator<Item = walkdir::Result<DirEntry>>>;

    let mut walkers: Vec<WalkIter> = SCAN_DIRS
        .iter()
        .filter_map(|d| {
            let p = root.join(d);
            if !p.exists() {
                return None;
            }
            let it: WalkIter = Box::new(
                WalkDir::new(p)
                    .follow_links(false)
                    .into_iter()
                    .filter_entry(|e| !is_skipped(e)),
            );
            Some(it)
        })
        .collect();

    for f in SCAN_ROOT_FILES {
        let p = root.join(f);
        if !p.exists() {
            continue;
        }
        let it: WalkIter = Box::new(
            WalkDir::new(p)
                .max_depth(1)
                .into_iter()
                .filter_entry(|e| !is_skipped(e)),
        );
        walkers.push(it);
    }

    walkers.into_iter().flatten()
}

fn is_skipped(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    SKIP_DIRS.iter().any(|d| name == *d)
}

fn scan_line(
    path: &Path,
    _line_no: usize,
    line: &str,
    ext: &str,
    personal_names: &[String],
    counts: &mut BTreeMap<String, u64>,
) {
    let is_rust = ext == "rs";
    let is_md = ext == "md";

    if is_rust || is_md {
        for marker in &["TODO", "FIXME", "XXX"] {
            if line.contains(marker) && !is_marker_in_skill_definition(path, line, marker) {
                *counts.get_mut("todo-fixme").unwrap() += 1;
                break;
            }
        }
    }

    if is_rust {
        if line.contains("allow(dead_code)") {
            *counts.get_mut("allow-dead-code").unwrap() += 1;
        }
        if line.contains("#[ignore") {
            *counts.get_mut("ignored-tests").unwrap() += 1;
        }
        // unsafe_code = "forbid" makes this redundant, but still useful
        // if the lint is ever relaxed.
        if line.trim_start().starts_with("unsafe ")
            || line.contains(" unsafe ")
            || line.contains("unsafe {")
        {
            *counts.get_mut("unsafe-blocks").unwrap() += 1;
        }
    }

    // Personal Windows path — matches both `C:\Users\<name>\` and the
    // forward-slash variant, for every name listed in
    // `DEBT_SCAN_PERSONAL_NAMES`. With no names configured the check is
    // a no-op, which is the right default for a public repo.
    let line_lower = line.to_lowercase();
    for name in personal_names {
        let win = format!("c:\\users\\{name}");
        let unix = format!("c:/users/{name}");
        if line_lower.contains(&win) || line_lower.contains(&unix) {
            *counts.get_mut("personal-windows-path").unwrap() += 1;
            break;
        }
    }
}

fn is_marker_in_skill_definition(path: &Path, _line: &str, _marker: &str) -> bool {
    // Skill docs talk about TODO/FIXME workflows — that's not debt itself.
    path.starts_with(".skillshare") || path.starts_with(".claude/skills")
}

fn count_external_deps(root: &Path) -> Result<u64> {
    // debt-scan is intentionally excluded — it's a dev tool that ships
    // serde / walkdir / clap, but those deps never reach the editor binary.
    let mut total = 0_u64;
    for crate_dir in &["editor-core", "editor-service", "editor-ui"] {
        let manifest = root.join("crates").join(crate_dir).join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest)?;
        let mut in_deps = false;
        for raw in text.lines() {
            let line = raw.trim();
            if line.starts_with('[') {
                in_deps = matches!(
                    line,
                    "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
                );
                continue;
            }
            if !in_deps || line.is_empty() || line.starts_with('#') {
                continue;
            }
            // `foo = ...` style entry.
            if let Some(rhs) = line.split_once('=').map(|(_, v)| v.trim()) {
                // path-only deps are internal to the workspace, not external.
                if rhs.contains("path =") && !rhs.contains("version") {
                    continue;
                }
                total += 1;
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn counts() -> BTreeMap<String, u64> {
        CATEGORIES.iter().map(|c| ((*c).to_string(), 0)).collect()
    }

    const NO_NAMES: &[String] = &[];

    fn names(vs: &[&str]) -> Vec<String> {
        vs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn detects_todo_marker_in_rust() {
        let mut c = counts();
        scan_line(
            Path::new("crates/editor-core/src/lib.rs"),
            0,
            "// TODO: implement parser",
            "rs",
            NO_NAMES,
            &mut c,
        );
        assert_eq!(c["todo-fixme"], 1);
    }

    #[test]
    fn ignores_todo_marker_in_skill_doc() {
        let mut c = counts();
        scan_line(
            Path::new(".skillshare/skills/commit-message/SKILL.md"),
            0,
            "## TODO checklist",
            "md",
            NO_NAMES,
            &mut c,
        );
        assert_eq!(c["todo-fixme"], 0);
    }

    #[test]
    fn detects_allow_dead_code() {
        let mut c = counts();
        scan_line(
            Path::new("crates/editor-core/src/lib.rs"),
            0,
            "    #[allow(dead_code)]",
            "rs",
            NO_NAMES,
            &mut c,
        );
        assert_eq!(c["allow-dead-code"], 1);
    }

    #[test]
    fn detects_ignored_test() {
        let mut c = counts();
        scan_line(
            Path::new("crates/editor-core/tests/roundtrip.rs"),
            0,
            r#"#[ignore = "scaffold"]"#,
            "rs",
            NO_NAMES,
            &mut c,
        );
        assert_eq!(c["ignored-tests"], 1);
    }

    #[test]
    fn detects_personal_path_when_name_configured() {
        let mut c = counts();
        scan_line(
            Path::new("crates/editor-ui/src/main.rs"),
            0,
            r#"let p = "C:\Users\alice\code\editor";"#,
            "rs",
            &names(&["alice"]),
            &mut c,
        );
        assert_eq!(c["personal-windows-path"], 1);
    }

    #[test]
    fn ignores_personal_path_when_no_names_configured() {
        let mut c = counts();
        scan_line(
            Path::new("crates/editor-ui/src/main.rs"),
            0,
            r#"let p = "C:\Users\alice\code\editor";"#,
            "rs",
            NO_NAMES,
            &mut c,
        );
        assert_eq!(c["personal-windows-path"], 0);
    }
}
