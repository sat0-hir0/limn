//! Integration tests for the read path: write a `.md` to a temp
//! directory, open it through `Vault`, and inspect the resulting
//! `Document`. These are the BDD-flavoured "user scenario" tests the
//! testing strategy doc calls the main act.

use std::fs;

use limn_core::block::BlockKind;
use limn_service::{OpenError, Vault};
use tempfile::tempdir;

#[test]
fn given_a_vault_with_one_md_file_when_opening_then_blocks_are_parsed() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("hello.md"), "# Hello\n\nworld\n").unwrap();

    let document = Vault::new(dir.path()).open_first_md().unwrap();

    assert_eq!(document.path, dir.path().join("hello.md"));
    assert_eq!(document.blocks.len(), 2);
    assert!(matches!(
        document.blocks[0].kind,
        BlockKind::Heading { level: 1 }
    ));
    assert_eq!(document.blocks[0].text, "Hello");
    assert!(matches!(document.blocks[1].kind, BlockKind::Paragraph));
    assert_eq!(document.blocks[1].text, "world");
}

#[test]
fn given_multiple_md_files_when_opening_first_then_alphabetical_winner_returned() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("zzz.md"), "# z\n").unwrap();
    fs::write(dir.path().join("aaa.md"), "# a\n").unwrap();
    fs::write(dir.path().join("mmm.md"), "# m\n").unwrap();

    let document = Vault::new(dir.path()).open_first_md().unwrap();

    assert_eq!(
        document.path.file_name().and_then(|n| n.to_str()),
        Some("aaa.md")
    );
    assert_eq!(document.blocks[0].text, "a");
}

#[test]
fn given_multiple_md_files_when_list_md_files_then_alphabetical_md_only() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("zzz.md"), "# z\n").unwrap();
    fs::write(dir.path().join("aaa.md"), "# a\n").unwrap();
    fs::write(dir.path().join("mmm.md"), "# m\n").unwrap();
    // A non-.md file must be excluded from the listing.
    fs::write(dir.path().join("notes.txt"), "not markdown").unwrap();
    // A subdirectory (even one holding a .md) must not be walked.
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub").join("nested.md"), "# nested\n").unwrap();

    let entries = Vault::new(dir.path()).list_md_files().unwrap();

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["aaa.md", "mmm.md", "zzz.md"]);
    // Paths point at the real files directly under the root.
    assert_eq!(entries[0].path, dir.path().join("aaa.md"));
}

#[test]
fn given_an_empty_directory_when_list_md_files_then_empty_vec() {
    let dir = tempdir().unwrap();

    let entries = Vault::new(dir.path()).list_md_files().unwrap();

    assert!(entries.is_empty());
}

#[test]
fn given_an_empty_directory_when_opening_then_no_markdown_file_error() {
    let dir = tempdir().unwrap();

    let err = Vault::new(dir.path()).open_first_md().unwrap_err();

    match err {
        OpenError::NoMarkdownFile { dir: reported } => {
            assert_eq!(reported, dir.path());
        }
        OpenError::Io(io_err) => panic!("expected NoMarkdownFile, got Io({io_err:?})"),
    }
}

#[test]
fn given_a_non_md_file_when_opening_then_no_markdown_file_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("notes.txt"), "this is not markdown").unwrap();

    let err = Vault::new(dir.path()).open_first_md().unwrap_err();

    assert!(matches!(err, OpenError::NoMarkdownFile { .. }));
}

#[test]
fn given_a_specific_path_when_open_path_then_document_is_returned() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("only.md");
    fs::write(&path, "## Heading 2\n\nbody\n").unwrap();

    let document = Vault::open_path(&path).unwrap();

    assert_eq!(document.path, path);
    assert!(matches!(
        document.blocks[0].kind,
        BlockKind::Heading { level: 2 }
    ));
}

#[test]
fn given_a_missing_path_when_open_path_then_io_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("does-not-exist.md");

    let err = Vault::open_path(&path).unwrap_err();

    assert!(matches!(err, OpenError::Io(_)));
}

#[test]
fn given_raw_text_when_save_raw_then_round_trips_through_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("note.md");
    fs::write(&path, "# Old\n\nold body\n").unwrap();

    let new_text = "# New\n\nedited body with unicode: café 日本語\n";
    Vault::save_raw(&path, new_text).unwrap();

    // Read back the bytes verbatim — autosave must not mangle the text.
    let on_disk = fs::read_to_string(&path).unwrap();
    assert_eq!(on_disk, new_text);

    // And the service's own raw-read path agrees.
    let reread = Vault::open_path_raw(&path).unwrap();
    assert_eq!(reread.text, new_text);
}

#[test]
fn given_a_new_path_when_save_raw_then_file_is_created() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fresh.md");

    Vault::save_raw(&path, "brand new\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "brand new\n");
}

#[test]
fn given_sequential_saves_when_save_raw_then_last_write_wins_and_no_temp_litter() {
    // NOTE: this verifies *sequential* (synchronous, back-to-back) saves
    // only — the last call's contents win and no temp sidecar is left
    // behind. It does not exercise *concurrent* writers racing on the
    // same path; true concurrent last-write-wins is out of scope here.
    let dir = tempdir().unwrap();
    let path = dir.path().join("note.md");

    Vault::save_raw(&path, "first\n").unwrap();
    Vault::save_raw(&path, "second\n").unwrap();
    Vault::save_raw(&path, "third\n").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "third\n");

    // The atomic write should leave no temporary sidecar files behind.
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(".limn-tmp."))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temp files left behind: {leftovers:?}"
    );
}
