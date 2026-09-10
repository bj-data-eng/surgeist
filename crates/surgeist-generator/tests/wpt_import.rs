#![cfg(all(feature = "css-corpus", target_os = "macos", target_arch = "aarch64"))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use surgeist_generator::Sha256Digest;

const WPT_REPOSITORY: &str = "https://github.com/web-platform-tests/wpt.git";
const SYNTHETIC_REVISION: &str = "1111111111111111111111111111111111111111";
const FIXTURE_PATH: &str = "css/css-text/parsing/example-valid.html";
const FIXTURE: &[u8] =
    b"<!doctype html>\n<script>test_valid_value(\"text-transform\", \"none\");</script>\n";
const LICENSE: &[u8] = b"Synthetic test fixture released into the public domain.\n";
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "surgeist-generator-wpt-import-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).expect("create isolated WPT test root");
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove isolated WPT test root");
    }
}

fn invoke(owner: &Path, corpus: &Path, command: &str, source: Option<&Path>) -> Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_surgeist-css-generate"));
    process
        .arg("--owner-root")
        .arg(owner)
        .arg("--corpus-root")
        .arg(corpus)
        .arg(command);
    if let Some(source) = source {
        process.arg("--source-root").arg(source);
    }
    process.output().expect("run packaged CSS corpus CLI")
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "CSS corpus command failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn wpt_cli_imports_only_declared_files_and_checks_without_source_bundle() {
    let root = TestRoot::new();
    let owner = root.0.join("owner");
    let corpus = owner.join("corpus");
    let bundle = root.0.join("bundle");
    fs::create_dir_all(&corpus).expect("create corpus");
    fs::create_dir_all(bundle.join("css/css-text/parsing")).expect("create source bundle");
    fs::write(bundle.join(FIXTURE_PATH), FIXTURE).expect("write declared fixture");
    fs::write(bundle.join("LICENSE.md"), LICENSE).expect("write declared license");
    fs::write(
        bundle.join("unlisted.js"),
        b"throw new Error('unlisted');\n",
    )
    .expect("write unlisted source");

    let fixture_digest = Sha256Digest::from_bytes(FIXTURE);
    let license_digest = Sha256Digest::from_bytes(LICENSE);
    let manifest = format!(
        concat!(
            "schema_version = 1\n\n",
            "[source]\nkind = \"wpt\"\nrepository = \"{}\"\nrevision = \"{}\"\nimport_root = \"source\"\n\n",
            "[[files]]\npath = \"{}\"\nsha256 = \"{}\"\nlicenses = [\"LICENSE.md\"]\n\n",
            "[[licenses]]\npath = \"LICENSE.md\"\nsha256 = \"{}\"\n",
        ),
        WPT_REPOSITORY, SYNTHETIC_REVISION, FIXTURE_PATH, fixture_digest, license_digest,
    );
    fs::write(corpus.join("corpus.toml"), &manifest).expect("write WPT import manifest");

    assert_success(invoke(&owner, &corpus, "import-wpt", Some(&bundle)));

    let imported_fixture = corpus.join("source").join(FIXTURE_PATH);
    assert_eq!(fs::read(&imported_fixture).unwrap(), FIXTURE);
    assert_eq!(fs::read(corpus.join("source/LICENSE.md")).unwrap(), LICENSE);
    assert!(!corpus.join("source/unlisted.js").exists());
    assert_eq!(fs::read(bundle.join(FIXTURE_PATH)).unwrap(), FIXTURE);
    assert_eq!(fs::read(bundle.join("LICENSE.md")).unwrap(), LICENSE);
    assert_eq!(
        fs::read_to_string(corpus.join("corpus.toml")).unwrap(),
        manifest
    );

    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(corpus.join("source/.surgeist-source.json")).expect("read WPT import receipt"),
    )
    .expect("parse WPT import receipt");
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": 1,
            "generator": "surgeist-css-generate",
            "verification": "manifest-file-digests",
            "source": {
                "kind": "wpt",
                "repository": WPT_REPOSITORY,
                "revision": SYNTHETIC_REVISION,
            },
            "manifest_sha256": Sha256Digest::from_bytes(manifest.as_bytes()).to_string(),
            "files": [{
                "path": FIXTURE_PATH,
                "sha256": fixture_digest.to_string(),
                "licenses": ["LICENSE.md"],
            }],
            "licenses": [{
                "path": "LICENSE.md",
                "sha256": license_digest.to_string(),
            }],
        }),
    );

    fs::remove_dir_all(&bundle).expect("remove acquisition bundle before offline verification");
    assert_success(invoke(&owner, &corpus, "check-corpus", None));

    let unsupported_generation = invoke(&owner, &corpus, "generate", None);
    assert_eq!(unsupported_generation.status.code(), Some(1));
    assert!(unsupported_generation.stdout.is_empty());
    assert!(String::from_utf8_lossy(&unsupported_generation.stderr).contains("WPT"));
    assert_eq!(fs::read(&imported_fixture).unwrap(), FIXTURE);

    fs::write(&imported_fixture, b"changed fixture\n").expect("tamper with imported fixture");
    let rejected = invoke(&owner, &corpus, "check-corpus", None);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains(FIXTURE_PATH));
    assert_eq!(fs::read(imported_fixture).unwrap(), b"changed fixture\n");
}

struct Fixture {
    _root: TestRoot,
    owner: PathBuf,
    corpus: PathBuf,
    bundle: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = TestRoot::new();
        let owner = root.0.join("owner");
        let corpus = owner.join("corpus");
        let bundle = owner
            .join("tmp/surgeist-sources/wpt")
            .join(SYNTHETIC_REVISION);
        fs::create_dir_all(&corpus).unwrap();
        fs::create_dir_all(bundle.join("css/css-text/parsing")).unwrap();
        fs::write(bundle.join(FIXTURE_PATH), FIXTURE).unwrap();
        fs::write(bundle.join("LICENSE.md"), LICENSE).unwrap();
        fs::write(
            corpus.join("corpus.toml"),
            manifest_for(FIXTURE_PATH, FIXTURE),
        )
        .unwrap();
        Self {
            _root: root,
            owner,
            corpus,
            bundle,
        }
    }

    fn import(&self) -> Output {
        invoke(&self.owner, &self.corpus, "import-wpt", Some(&self.bundle))
    }

    fn check(&self) -> Output {
        invoke(&self.owner, &self.corpus, "check-corpus", None)
    }

    fn reject_without_writes(&self, command: &str) -> String {
        let before = snapshot(&self.owner);
        let output = match command {
            "import-wpt" => self.import(),
            "check-corpus" => self.check(),
            _ => panic!("unrecognized test command"),
        };
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert_eq!(
            snapshot(&self.owner),
            before,
            "failed {command} mutated owned files"
        );
        String::from_utf8(output.stderr).unwrap()
    }
}

fn manifest_for(path: &str, bytes: &[u8]) -> String {
    format!(
        concat!(
            "schema_version = 1\n\n",
            "[source]\nkind = \"wpt\"\nrepository = \"{}\"\nrevision = \"{}\"\nimport_root = \"source\"\n\n",
            "[[files]]\npath = \"{}\"\nsha256 = \"{}\"\nlicenses = [\"LICENSE.md\"]\n\n",
            "[[licenses]]\npath = \"LICENSE.md\"\nsha256 = \"{}\"\n",
        ),
        WPT_REPOSITORY,
        SYNTHETIC_REVISION,
        path,
        Sha256Digest::from_bytes(bytes),
        Sha256Digest::from_bytes(LICENSE),
    )
}

#[derive(Debug, Eq, PartialEq)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        for child in fs::read_dir(path).unwrap() {
            let child = child.unwrap().path();
            let metadata = fs::symlink_metadata(&child).unwrap();
            let entry = if metadata.is_symlink() {
                Entry::Link(fs::read_link(&child).unwrap())
            } else if metadata.is_dir() {
                visit(root, &child, entries);
                Entry::Directory
            } else {
                Entry::File(fs::read(&child).unwrap())
            };
            entries.insert(child.strip_prefix(root).unwrap().to_path_buf(), entry);
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

#[test]
fn malformed_wpt_manifests_fail_before_any_write() {
    let manifest = manifest_for(FIXTURE_PATH, FIXTURE);
    let file_block = format!(
        "[[files]]\npath = \"{FIXTURE_PATH}\"\nsha256 = \"{}\"\nlicenses = [\"LICENSE.md\"]\n\n",
        Sha256Digest::from_bytes(FIXTURE)
    );
    let license_block = format!(
        "[[licenses]]\npath = \"LICENSE.md\"\nsha256 = \"{}\"\n",
        Sha256Digest::from_bytes(LICENSE)
    );
    let cases = [
        ("unknown top field", format!("unexpected = 1\n{manifest}")),
        (
            "unknown source field",
            manifest.replace("[source]", "[source]\nunexpected = 1"),
        ),
        (
            "unknown file field",
            manifest.replace("[[files]]", "[[files]]\nunexpected = 1"),
        ),
        (
            "unknown license field",
            manifest.replace("[[licenses]]", "[[licenses]]\nunexpected = 1"),
        ),
        (
            "schema",
            manifest.replace("schema_version = 1", "schema_version = 2"),
        ),
        (
            "repository",
            manifest.replace(WPT_REPOSITORY, "https://example.com/wpt.git"),
        ),
        ("revision", manifest.replace(SYNTHETIC_REVISION, "1111111")),
        (
            "hash",
            manifest.replace(&Sha256Digest::from_bytes(FIXTURE).to_string(), "abc"),
        ),
        ("missing files", manifest.replace(&file_block, "")),
        (
            "empty files",
            format!("files = []\n{}", manifest.replace(&file_block, "")),
        ),
        (
            "empty licenses",
            format!("licenses = []\n{}", manifest.replace(&license_block, "")),
        ),
        (
            "unbound file",
            manifest.replace("licenses = [\"LICENSE.md\"]", "licenses = []"),
        ),
        (
            "unknown license",
            manifest.replace("licenses = [\"LICENSE.md\"]", "licenses = [\"OTHER.md\"]"),
        ),
        (
            "duplicate license reference",
            manifest.replace(
                "licenses = [\"LICENSE.md\"]",
                "licenses = [\"LICENSE.md\", \"LICENSE.md\"]",
            ),
        ),
        ("duplicate file", format!("{manifest}\n{file_block}")),
        ("duplicate license", format!("{manifest}\n{license_block}")),
        (
            "case alias",
            format!(
                "{manifest}\n{}",
                file_block.replace(FIXTURE_PATH, &FIXTURE_PATH.to_ascii_uppercase())
            ),
        ),
        (
            "directory case alias",
            format!(
                "{manifest}\n{}",
                file_block.replace(FIXTURE_PATH, "CSS/other.html")
            ),
        ),
        (
            "file is directory",
            manifest.replace(FIXTURE_PATH, "LICENSE.md/example.html"),
        ),
        (
            "traversal",
            manifest.replace(FIXTURE_PATH, "../outside.html"),
        ),
        (
            "absolute path",
            manifest.replace(FIXTURE_PATH, "/outside.html"),
        ),
        (
            "reserved source file",
            manifest.replace(FIXTURE_PATH, "css/.git/config"),
        ),
        (
            "reserved receipt",
            manifest.replace(FIXTURE_PATH, ".surgeist-source.json"),
        ),
        (
            "reserved output root",
            manifest.replace("import_root = \"source\"", "import_root = \".git\""),
        ),
        (
            "output root case alias",
            manifest.replace("import_root = \"source\"", "import_root = \"CORPUS.TOML\""),
        ),
        (
            "nested output root",
            manifest.replace(
                "import_root = \"source\"",
                "import_root = \"nested/source\"",
            ),
        ),
    ];
    for (label, invalid) in cases {
        let fixture = Fixture::new();
        fs::write(fixture.corpus.join("corpus.toml"), invalid).unwrap();
        let diagnostic = fixture.reject_without_writes("import-wpt");
        assert!(
            !diagnostic.contains("unknown CSS command"),
            "{label}: {diagnostic}"
        );
        assert!(
            !fixture.corpus.join(".surgeist-generator").exists(),
            "{label}"
        );
    }
}

#[test]
fn wpt_bundle_hashes_and_link_policy_fail_before_any_write() {
    for damaged in [FIXTURE_PATH, "LICENSE.md"] {
        let fixture = Fixture::new();
        fs::write(fixture.bundle.join(damaged), b"different input\n").unwrap();
        assert!(
            fixture
                .reject_without_writes("import-wpt")
                .contains(damaged)
        );
    }
    for link in [false, true] {
        let fixture = Fixture::new();
        let path = fixture.bundle.join(FIXTURE_PATH);
        fs::remove_file(&path).unwrap();
        let external = fixture.owner.join("external.html");
        fs::write(&external, FIXTURE).unwrap();
        if link {
            std::os::unix::fs::symlink(&external, &path).unwrap();
        } else {
            fs::hard_link(&external, &path).unwrap();
        }
        fixture.reject_without_writes("import-wpt");
    }
    let fixture = Fixture::new();
    fs::remove_file(fixture.bundle.join("LICENSE.md")).unwrap();
    fixture.reject_without_writes("import-wpt");
}

#[test]
fn wpt_import_replaces_only_prior_receipt_owned_files_and_is_atomic_on_bad_input() {
    let fixture = Fixture::new();
    assert_success(fixture.import());
    let imported_before = snapshot(&fixture.corpus.join("source"));
    assert_success(fixture.import());
    assert_eq!(snapshot(&fixture.corpus.join("source")), imported_before);

    let new_path = "css/css-text/parsing/replacement.html";
    let new_bytes = b"<!doctype html>\n<!-- replacement synthetic fixture -->\n";
    fs::write(fixture.bundle.join(new_path), new_bytes).unwrap();
    fs::write(
        fixture.corpus.join("corpus.toml"),
        manifest_for(new_path, new_bytes),
    )
    .unwrap();
    fs::write(fixture.bundle.join("LICENSE.md"), b"wrong license\n").unwrap();
    assert!(
        fixture
            .reject_without_writes("import-wpt")
            .contains("LICENSE.md")
    );
    assert_eq!(snapshot(&fixture.corpus.join("source")), imported_before);

    fs::write(fixture.bundle.join("LICENSE.md"), LICENSE).unwrap();
    assert_success(fixture.import());
    assert!(!fixture.corpus.join("source").join(FIXTURE_PATH).exists());
    assert_eq!(
        fs::read(fixture.corpus.join("source").join(new_path)).unwrap(),
        new_bytes
    );
    assert_success(fixture.check());
}

#[test]
fn wpt_import_and_check_reject_unknown_or_tampered_owned_inventory() {
    for alteration in [
        "unknown-file",
        "unknown-directory",
        "license",
        "missing-file",
        "receipt",
    ] {
        let fixture = Fixture::new();
        assert_success(fixture.import());
        let imported = fixture.corpus.join("source");
        match alteration {
            "unknown-file" => fs::write(imported.join("personal.txt"), b"keep me\n").unwrap(),
            "unknown-directory" => fs::create_dir(imported.join("personal")).unwrap(),
            "license" => fs::write(imported.join("LICENSE.md"), b"changed\n").unwrap(),
            "missing-file" => fs::remove_file(imported.join(FIXTURE_PATH)).unwrap(),
            "receipt" => {
                let path = imported.join(".surgeist-source.json");
                let mut bytes = fs::read(&path).unwrap();
                bytes.push(b'\n');
                fs::write(path, bytes).unwrap();
            }
            _ => unreachable!(),
        }
        fixture.reject_without_writes("import-wpt");
        fixture.reject_without_writes("check-corpus");
    }
    let fixture = Fixture::new();
    fs::create_dir(fixture.corpus.join("source")).unwrap();
    fs::write(fixture.corpus.join("source/personal.txt"), b"keep me\n").unwrap();
    fixture.reject_without_writes("import-wpt");
}

#[test]
fn wpt_check_binds_the_exact_manifest_bytes_without_writing() {
    let fixture = Fixture::new();
    assert_success(fixture.import());
    let manifest = fixture.corpus.join("corpus.toml");
    fs::write(
        &manifest,
        format!(
            "{}\n# reviewed comment changed\n",
            fs::read_to_string(&manifest).unwrap()
        ),
    )
    .unwrap();
    assert!(
        fixture
            .reject_without_writes("check-corpus")
            .contains("manifest")
    );
    assert_success(fixture.import());
    assert_success(fixture.check());
}

#[test]
fn wpt_receipt_rejects_a_file_binding_to_an_undeclared_license() {
    let fixture = Fixture::new();
    assert_success(fixture.import());
    let receipt = fixture.corpus.join("source/.surgeist-source.json");
    let bytes = fs::read_to_string(&receipt).unwrap();
    let changed = bytes.replace(
        "\"licenses\":[\"LICENSE.md\"]",
        "\"licenses\":[\"MISSING.md\"]",
    );
    assert_ne!(bytes, changed);
    fs::write(receipt, changed).unwrap();
    assert!(
        fixture
            .reject_without_writes("check-corpus")
            .contains("license binding")
    );
    assert!(
        fixture
            .reject_without_writes("import-wpt")
            .contains("license binding")
    );
}

#[test]
fn wpt_receipt_sorts_files_licenses_and_per_file_bindings() {
    let fixture = Fixture::new();
    let other_path = "css/a.html";
    let other_bytes = b"<!doctype html>\n<!-- another synthetic fixture -->\n";
    fs::write(fixture.bundle.join(other_path), other_bytes).unwrap();
    fs::write(fixture.bundle.join("COPYING.md"), LICENSE).unwrap();
    let mut manifest = manifest_for(FIXTURE_PATH, FIXTURE).replace(
        "licenses = [\"LICENSE.md\"]",
        "licenses = [\"LICENSE.md\", \"COPYING.md\"]",
    );
    manifest.push_str(&format!(
        "\n[[licenses]]\npath = \"COPYING.md\"\nsha256 = \"{}\"\n\n[[files]]\npath = \"{other_path}\"\nsha256 = \"{}\"\nlicenses = [\"LICENSE.md\"]\n",
        Sha256Digest::from_bytes(LICENSE), Sha256Digest::from_bytes(other_bytes),
    ));
    fs::write(fixture.corpus.join("corpus.toml"), manifest).unwrap();
    assert_success(fixture.import());
    let bytes = fs::read(fixture.corpus.join("source/.surgeist-source.json")).unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert!(!bytes[..bytes.len() - 1].contains(&b'\n'));
    let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(receipt["files"][0]["path"], other_path);
    assert_eq!(receipt["files"][1]["path"], FIXTURE_PATH);
    assert_eq!(
        receipt["files"][1]["licenses"],
        serde_json::json!(["COPYING.md", "LICENSE.md"])
    );
    assert_eq!(receipt["licenses"][0]["path"], "COPYING.md");
    assert_eq!(receipt["licenses"][1]["path"], "LICENSE.md");
    assert_success(fixture.check());
}

#[test]
fn wpt_import_rejects_a_bundle_overlapping_its_output() {
    let fixture = Fixture::new();
    let output = fixture.corpus.join("source");
    fs::rename(&fixture.bundle, &output).unwrap();
    let before = snapshot(&fixture.owner);
    let result = invoke(&fixture.owner, &fixture.corpus, "import-wpt", Some(&output));
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(snapshot(&fixture.owner), before);
}

#[test]
fn wpt_cli_rejects_invalid_options_before_opening_roots() {
    for arguments in [
        vec!["import-wpt"],
        vec!["import-wpt", "--source-root", "bundle", "--filter", "css"],
        vec!["check-corpus", "--source-root", "bundle"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_surgeist-css-generate"))
            .args([
                "--owner-root",
                "nonexistent-owner",
                "--corpus-root",
                "nonexistent-corpus",
            ])
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(64));
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(!diagnostic.contains("unknown CSS command"));
        assert!(!diagnostic.contains("canonicalize"));
    }
}
