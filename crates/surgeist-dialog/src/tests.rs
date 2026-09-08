use super::*;
use std::path::{Path, PathBuf};

fn path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().to_path_buf()
}

#[test]
fn builds_options_fluently() {
    let dialog = FileDialog::new()
        .title("Open")
        .directory("fixtures")
        .file_name("input.txt")
        .filter("Text", ["txt", "md"]);

    assert_eq!(
        dialog.options(),
        &Options {
            title: Some(String::from("Open")),
            directory: Some(path("fixtures")),
            file_name: Some(String::from("input.txt")),
            filters: vec![Filter::new("Text", ["txt", "md"])],
        }
    );
}

#[test]
fn fake_backend_records_calls_and_returns_results() {
    let options = Options::new().title("Pick").filter("Rust", ["rs"]);
    let mut backend = FakeBackend::new();
    backend.push_open_file(Some("fixtures/lib.rs"));
    backend.push_open_files(Some(vec!["fixtures/a.rs", "fixtures/b.rs"]));
    backend.push_open_folder(Some("fixtures"));
    backend.push_open_folders(Some(vec!["fixtures/a", "fixtures/b"]));
    backend.push_save_file(Some("fixtures/out.rs"));

    assert_eq!(
        backend.open_file(&options).unwrap(),
        Some(path("fixtures/lib.rs"))
    );
    assert_eq!(
        backend.open_files(&options).unwrap(),
        Some(vec![path("fixtures/a.rs"), path("fixtures/b.rs")])
    );
    assert_eq!(
        backend.open_folder(&options).unwrap(),
        Some(path("fixtures"))
    );
    assert_eq!(
        backend.open_folders(&options).unwrap(),
        Some(vec![path("fixtures/a"), path("fixtures/b")])
    );
    assert_eq!(
        backend.save_file(&options).unwrap(),
        Some(path("fixtures/out.rs"))
    );
    assert_eq!(
        backend.calls(),
        &[
            Call::OpenFile(options.clone()),
            Call::OpenFiles(options.clone()),
            Call::OpenFolder(options.clone()),
            Call::OpenFolders(options.clone()),
            Call::SaveFile(options),
        ]
    );
}

#[test]
fn file_dialog_runs_against_supplied_backend() {
    let mut backend = FakeBackend::new();
    backend.push_open_file(Some("fixtures/input.rs"));

    let result = FileDialog::new()
        .title("Pick")
        .filter("Rust", ["rs"])
        .open_file_with(&mut backend)
        .unwrap();

    assert_eq!(result, Some(PathBuf::from("fixtures/input.rs")));
    assert_eq!(backend.calls().len(), 1);
}

#[test]
fn rejects_empty_filter_names_and_extensions() {
    let mut backend = FakeBackend::new();
    let name_error = FileDialog::new()
        .filter("", ["rs"])
        .open_file_with(&mut backend)
        .unwrap_err();
    let extension_error = FileDialog::new()
        .filter("Rust", [""])
        .open_file_with(&mut backend)
        .unwrap_err();

    assert_eq!(name_error.code, ErrorCode::InvalidOptions);
    assert_eq!(extension_error.code, ErrorCode::InvalidOptions);
    assert!(backend.calls().is_empty());
}

#[test]
fn fake_backend_defaults_to_cancelled() {
    let mut backend = FakeBackend::new();

    assert_eq!(backend.open_file(&Options::new()).unwrap(), None);
}
