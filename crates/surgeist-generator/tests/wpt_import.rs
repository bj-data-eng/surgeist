#![cfg(all(feature = "css-corpus", target_os = "macos", target_arch = "aarch64"))]
#![forbid(unsafe_code)]

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
    fs::write(bundle.join("unlisted.js"), b"throw new Error('unlisted');\n")
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
    assert_eq!(fs::read_to_string(corpus.join("corpus.toml")).unwrap(), manifest);

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
