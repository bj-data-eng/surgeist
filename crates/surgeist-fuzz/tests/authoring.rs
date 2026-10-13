//! The real standalone author package is consumed by Cargo and the registry.
#![forbid(unsafe_code)]

use std::{path::PathBuf, process::Command};
use surgeist_fuzz::{TargetId, registry::TargetRegistry};

#[test]
fn standalone_author_workspace_registers_raw_and_structured_binaries_and_fixtures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/subject")
        .canonicalize()
        .unwrap();
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
            "--locked",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let registry = TargetRegistry::from_metadata_json(&root, &output.stdout).unwrap();
    assert_eq!(registry.targets().count(), 2);
    for (id, binary) in [
        ("surgeist-fixture::bytes", "fixture-worker"),
        ("surgeist-fixture::structured", "fixture-structured-worker"),
    ] {
        let target = registry.get(&TargetId::new(id).unwrap()).unwrap();
        assert_eq!(target.descriptor().binary(), binary);
        assert_eq!(target.package(), "surgeist-fixture-fuzz");
        assert!(target.fixtures().iter().any(|path| path.ends_with("seeds")));
        assert!(
            target
                .fixtures()
                .iter()
                .any(|path| path.ends_with("regressions"))
        );
    }
}
