/// limn-doc-preprocessor: mdBook preprocessor that validates doc integrity.
///
/// Checks performed:
/// 1. Path mentions  — `crates/…`, `.github/workflows/…`, `docs/…` must exist in repo.
/// 2. Type mentions  — `limn_core::…`, `limn_service::…`, `limn_ui::…` must appear in source.
/// 3. ADR sequence   — `docs/adr/NNNN-*.md` must have no gaps, no duplicates,
///    and every non-special file starts with `NNNN-`.
/// 4. ADR cross-refs — `ADR-NNNN` references inside ADR files must point to real files.
///
/// Controlled via LIMN_DOC_LINT env var:
/// - (unset / "error") → validation failure causes exit(1)
/// - "warn"            → print warning but exit 0 (useful for local iteration)
use std::{
    collections::{HashMap, HashSet},
    env, fs, io,
    path::{Path, PathBuf},
    process,
};

use anyhow::{anyhow, Context, Result};
use mdbook::book::{Book, BookItem};
use regex::Regex;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Lint-mode helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LintMode {
    Error,
    Warn,
}

impl LintMode {
    fn from_env() -> Self {
        match env::var("LIMN_DOC_LINT").as_deref() {
            Ok("warn") => Self::Warn,
            _ => Self::Error,
        }
    }

    fn emit(self, errors: &[String]) {
        if errors.is_empty() {
            return;
        }
        for e in errors {
            eprintln!("[limn-doc-preprocessor] {e}");
        }
        if self == Self::Error {
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();

    // mdBook calls the preprocessor with `supports <renderer>` first.
    if args.len() > 1 && args[1] == "supports" {
        let renderer = args.get(2).map(String::as_str).unwrap_or("");
        if renderer == "html" {
            process::exit(0);
        } else {
            process::exit(1);
        }
    }

    // Normal preprocessor invocation: read JSON from stdin, validate, write back.
    if let Err(e) = run() {
        eprintln!("[limn-doc-preprocessor] fatal: {e:#}");
        process::exit(1);
    }
}

/// What we actually need from the preprocessor context. mdBook keeps
/// adding fields under `config.*` and rejects the full struct when any
/// optional TOML value comes through as `null`, so we extract the two
/// path fields we care about by hand.
struct LiteContext {
    root: PathBuf,
    src: PathBuf,
}

fn parse_preprocessor_input(raw: &str) -> Result<(LiteContext, Book)> {
    let value: Value = serde_json::from_str(raw).context("parsing mdBook preprocessor JSON")?;
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("mdBook preprocessor input must be a JSON array"))?;
    let ctx_json = arr
        .first()
        .ok_or_else(|| anyhow!("missing PreprocessorContext element"))?;
    let book_json = arr.get(1).ok_or_else(|| anyhow!("missing Book element"))?;

    let root = ctx_json
        .get("root")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing root in PreprocessorContext"))?;
    let src = ctx_json
        .get("config")
        .and_then(|c| c.get("book"))
        .and_then(|b| b.get("src"))
        .and_then(Value::as_str)
        .unwrap_or("src");

    let book: Book = serde_json::from_value(book_json.clone()).context("decoding Book")?;

    Ok((
        LiteContext {
            root: PathBuf::from(root),
            src: PathBuf::from(src),
        },
        book,
    ))
}

fn run() -> Result<()> {
    let mut raw = String::new();
    io::Read::read_to_string(&mut io::stdin(), &mut raw).context("reading stdin")?;

    let (ctx, book) = parse_preprocessor_input(&raw)?;

    let mode = LintMode::from_env();
    let errors = validate(&ctx, &book);
    mode.emit(&errors);

    // Return the book unchanged — we are a validation-only preprocessor.
    serde_json::to_writer(io::stdout(), &book).context("writing book JSON to stdout")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Top-level validator
// ---------------------------------------------------------------------------

fn validate(ctx: &LiteContext, book: &Book) -> Vec<String> {
    // Repo root is the parent of the `src` (docs) directory.
    let repo_root = ctx
        .src
        .parent()
        .map(|p| ctx.root.join(p))
        .unwrap_or_else(|| ctx.root.clone());

    let mut errors: Vec<String> = Vec::new();

    // Collect all chapter source paths and content for later checks.
    let chapters = collect_chapters(book);

    errors.extend(check_path_mentions(&chapters, &repo_root));
    errors.extend(check_type_mentions(&chapters, &repo_root));
    errors.extend(check_adr_sequence(&repo_root));
    errors.extend(check_adr_crossrefs(&chapters, &repo_root));

    errors
}

// ---------------------------------------------------------------------------
// Chapter collection helper
// ---------------------------------------------------------------------------

struct Chapter {
    /// Source path relative to the book `src` directory, if available.
    path: Option<PathBuf>,
    content: String,
}

fn collect_chapters(book: &Book) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    for item in book.iter() {
        if let BookItem::Chapter(ch) = item {
            chapters.push(Chapter {
                path: ch.path.clone(),
                content: ch.content.clone(),
            });
        }
    }
    chapters
}

// ---------------------------------------------------------------------------
// Check 1: path mentions
// ---------------------------------------------------------------------------

/// Looks for patterns like `crates/…`, `.github/workflows/…`, `docs/…`,
/// `src/…` in backtick spans and plain text, then verifies they exist.
fn check_path_mentions(chapters: &[Chapter], repo_root: &Path) -> Vec<String> {
    // Backtick span: capture the inner content.
    let re_backtick = Regex::new(r"`([^`]+)`").expect("static regex");
    // Bare path token anchored to a word boundary.
    // Uses [/] instead of \/ and avoids backslash escapes inside [...].
    let re_bare =
        Regex::new(r"\b((?:crates|docs|src)[/][^ \t\r\n>),`]+|[.]github[/][^ \t\r\n>),`]+)")
            .expect("static regex");

    let mut errors = Vec::new();
    for ch in chapters {
        let content = &ch.content;
        let source = ch
            .path
            .as_deref()
            .and_then(|p| p.to_str())
            .unwrap_or("<unknown>");

        // Track byte ranges covered by backtick spans so the bare-path
        // regex does not double-count them.
        let mut backtick_ranges: Vec<std::ops::Range<usize>> = Vec::new();

        for cap in re_backtick.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            backtick_ranges.push(full_match.start()..full_match.end());

            if let Some(m) = cap.get(1) {
                let candidate = m.as_str();
                if looks_like_repo_path(candidate) && !repo_root.join(candidate).exists() {
                    errors.push(format!(
                        "path-mention: `{candidate}` referenced in `{source}` does not exist"
                    ));
                }
            }
        }

        // Bare paths — skip if the match falls inside a backtick span.
        for cap in re_bare.captures_iter(content) {
            let m = cap.get(0).unwrap();
            let in_backtick = backtick_ranges
                .iter()
                .any(|r| m.start() >= r.start && m.end() <= r.end);
            if in_backtick {
                continue;
            }
            if let Some(inner) = cap.get(1) {
                let candidate = inner.as_str();
                if looks_like_repo_path(candidate) && !repo_root.join(candidate).exists() {
                    errors.push(format!(
                        "path-mention: `{candidate}` referenced in `{source}` does not exist"
                    ));
                }
            }
        }
    }
    errors
}

fn looks_like_repo_path(s: &str) -> bool {
    // Must start with a known prefix and contain at least one `/`.
    let prefixes = ["crates/", "docs/", "src/", ".github/"];
    prefixes.iter().any(|p| s.starts_with(p)) && s.contains('/')
}

// ---------------------------------------------------------------------------
// Check 2: type mentions
// ---------------------------------------------------------------------------

/// Verifies that `limn_core::Foo`, `limn_service::Bar`, `limn_ui::Baz`
/// references in docs correspond to an identifier that actually appears in the
/// respective crate's source tree.
fn check_type_mentions(chapters: &[Chapter], repo_root: &Path) -> Vec<String> {
    let re = Regex::new(r"\b(limn_(?:core|service|ui))::([A-Za-z_][A-Za-z0-9_]*)")
        .expect("static regex");

    // Build a map of crate → set of public identifiers appearing in source.
    let source_ids = build_source_id_map(repo_root);

    let mut errors = Vec::new();
    for ch in chapters {
        for cap in re.captures_iter(&ch.content) {
            let crate_slug = &cap[1]; // e.g. "limn_core"
            let ident = &cap[2];
            let crate_name = crate_slug.replace('_', "-");
            if let Some(ids) = source_ids.get(crate_name.as_str()) {
                if !ids.contains(ident) {
                    let source = ch
                        .path
                        .as_deref()
                        .and_then(|p| p.to_str())
                        .unwrap_or("<unknown>");
                    errors.push(format!(
                        "type-mention: `{crate_slug}::{ident}` in `{source}` not found in crate source"
                    ));
                }
            }
            // If we couldn't build an index for that crate, skip (crate may not exist yet).
        }
    }
    errors
}

/// Walk `crates/<crate>/src/**/*.rs` and collect every `pub …` identifier.
fn build_source_id_map(repo_root: &Path) -> HashMap<&'static str, HashSet<String>> {
    let crates: &[&str] = &["limn-core", "limn-service", "limn-ui"];
    let ident_re = Regex::new(
        r"(?m)^\s*pub\s+(?:struct|enum|trait|fn|type|mod|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)",
    )
    .expect("static regex");

    let mut map: HashMap<&'static str, HashSet<String>> = HashMap::new();
    for &c in crates {
        let src_dir = repo_root.join("crates").join(c).join("src");
        let mut ids = HashSet::new();
        if src_dir.is_dir() {
            walk_rs_files(&src_dir, &mut |content| {
                for cap in ident_re.captures_iter(&content) {
                    ids.insert(cap[1].to_string());
                }
            });
        }
        map.insert(c, ids);
    }
    map
}

fn walk_rs_files(dir: &Path, cb: &mut dyn FnMut(String)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, cb);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                cb(content);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Check 3: ADR sequence
// ---------------------------------------------------------------------------

/// Validates that `docs/adr/NNNN-*.md` files form a contiguous sequence
/// starting at 0001 with no gaps or duplicates, and that every non-special
/// file starts with four digits.
fn check_adr_sequence(repo_root: &Path) -> Vec<String> {
    let adr_dir = repo_root.join("docs").join("adr");
    if !adr_dir.is_dir() {
        // ADR directory not yet created (A agent still working) — skip silently.
        return vec![];
    }

    let special: HashSet<&str> = ["README.md", "template.md"].iter().copied().collect();
    let num_re = Regex::new(r"^(\d{4})-").expect("static regex");

    let mut errors = Vec::new();
    let mut seen: HashMap<u32, String> = HashMap::new();

    let Ok(entries) = fs::read_dir(&adr_dir) else {
        return vec![format!("adr-sequence: cannot read {}", adr_dir.display())];
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        if special.contains(name.as_str()) {
            continue;
        }
        if let Some(cap) = num_re.captures(&name) {
            let n: u32 = cap[1].parse().unwrap_or(0);
            if let Some(prev) = seen.get(&n) {
                errors.push(format!(
                    "adr-sequence: duplicate ADR number {n:04}: `{prev}` and `{name}`"
                ));
            } else {
                seen.insert(n, name.clone());
            }
        } else {
            errors.push(format!(
                "adr-sequence: `{name}` does not start with a four-digit number"
            ));
        }
    }

    // Check for gaps starting at 0001.
    if !seen.is_empty() {
        let max = *seen.keys().max().unwrap();
        for n in 1..=max {
            if !seen.contains_key(&n) {
                errors.push(format!(
                    "adr-sequence: ADR-{n:04} is missing (gap in sequence)"
                ));
            }
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Check 4: ADR cross-references
// ---------------------------------------------------------------------------

/// Finds `ADR-NNNN` mentions in all chapters and verifies the corresponding
/// file exists in `docs/adr/`.
fn check_adr_crossrefs(chapters: &[Chapter], repo_root: &Path) -> Vec<String> {
    let adr_dir = repo_root.join("docs").join("adr");
    if !adr_dir.is_dir() {
        return vec![];
    }

    let re = Regex::new(r"\bADR-(\d{4})\b").expect("static regex");

    // Build set of existing ADR numbers.
    let existing: HashSet<u32> = fs::read_dir(&adr_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let cap = Regex::new(r"^(\d{4})-").ok()?.captures(&name)?;
            cap[1].parse().ok()
        })
        .collect();

    let mut errors = Vec::new();
    for ch in chapters {
        for cap in re.captures_iter(&ch.content) {
            let n: u32 = cap[1].parse().unwrap_or(0);
            if !existing.contains(&n) {
                let source = ch
                    .path
                    .as_deref()
                    .and_then(|p| p.to_str())
                    .unwrap_or("<unknown>");
                errors.push(format!(
                    "adr-crossref: `ADR-{n:04}` in `{source}` has no matching file in docs/adr/"
                ));
            }
        }
    }
    errors
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_chapter(content: &str) -> Chapter {
        Chapter {
            path: Some(PathBuf::from("test.md")),
            content: content.to_string(),
        }
    }

    // --- looks_like_repo_path ---

    #[test]
    fn test_looks_like_repo_path_positive() {
        assert!(looks_like_repo_path("crates/limn-core/src/lib.rs"));
        assert!(looks_like_repo_path("docs/design/basic-features.md"));
        assert!(looks_like_repo_path(".github/workflows/ci.yml"));
    }

    #[test]
    fn test_looks_like_repo_path_negative() {
        assert!(!looks_like_repo_path("some-random-word"));
        assert!(!looks_like_repo_path("https://example.com/path"));
        assert!(!looks_like_repo_path("crates")); // no slash after prefix means no inner path
    }

    // --- check_path_mentions ---

    #[test]
    fn test_path_mention_existing_path() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("docs/design")).unwrap();
        fs::write(root.join("docs/design/basic-features.md"), "").unwrap();

        let chapters = vec![make_chapter("See `docs/design/basic-features.md`.")];
        let errors = check_path_mentions(&chapters, root);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn test_path_mention_missing_path() {
        let tmp = TempDir::new().unwrap();
        let chapters = vec![make_chapter("See `docs/missing/file.md`.")];
        let errors = check_path_mentions(&chapters, tmp.path());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("docs/missing/file.md"));
    }

    // --- check_adr_sequence ---

    #[test]
    fn test_adr_sequence_no_gap() {
        let tmp = TempDir::new().unwrap();
        let adr = tmp.path().join("docs/adr");
        fs::create_dir_all(&adr).unwrap();
        for name in &["0001-adopt-gpui.md", "0002-layers.md", "README.md"] {
            fs::write(adr.join(name), "").unwrap();
        }
        let errors = check_adr_sequence(tmp.path());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn test_adr_sequence_gap_detected() {
        let tmp = TempDir::new().unwrap();
        let adr = tmp.path().join("docs/adr");
        fs::create_dir_all(&adr).unwrap();
        fs::write(adr.join("0001-first.md"), "").unwrap();
        fs::write(adr.join("0003-third.md"), "").unwrap(); // 0002 is missing
        let errors = check_adr_sequence(tmp.path());
        assert!(
            errors.iter().any(|e| e.contains("ADR-0002")),
            "expected gap error: {errors:?}"
        );
    }

    #[test]
    fn test_adr_duplicate_number() {
        let tmp = TempDir::new().unwrap();
        let adr = tmp.path().join("docs/adr");
        fs::create_dir_all(&adr).unwrap();
        fs::write(adr.join("0001-first.md"), "").unwrap();
        fs::write(adr.join("0001-duplicate.md"), "").unwrap();
        let errors = check_adr_sequence(tmp.path());
        assert!(
            errors.iter().any(|e| e.contains("duplicate")),
            "expected duplicate error: {errors:?}"
        );
    }

    // --- check_adr_crossrefs ---

    #[test]
    fn test_adr_crossref_valid() {
        let tmp = TempDir::new().unwrap();
        let adr = tmp.path().join("docs/adr");
        fs::create_dir_all(&adr).unwrap();
        fs::write(adr.join("0001-adopt-gpui.md"), "").unwrap();

        let chapters = vec![make_chapter("See ADR-0001 for rationale.")];
        let errors = check_adr_crossrefs(&chapters, tmp.path());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn test_adr_crossref_missing() {
        let tmp = TempDir::new().unwrap();
        let adr = tmp.path().join("docs/adr");
        fs::create_dir_all(&adr).unwrap();
        // No ADR files present.

        let chapters = vec![make_chapter("See ADR-0042 for details.")];
        let errors = check_adr_crossrefs(&chapters, tmp.path());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("ADR-0042"));
    }
}
