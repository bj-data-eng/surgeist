#![forbid(unsafe_code)]
//! Registration through Cargo metadata, and the real worker binaries over the
//! framed capture-then-evaluate protocol.
use surgeist_fuzz::{TargetId, registry::TargetRegistry};

fn registry() -> TargetRegistry {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let metadata = std::process::Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    TargetRegistry::from_metadata_json(&root, &metadata.stdout).unwrap()
}

fn assert_registered(registry: &TargetRegistry, id: &str, binary: &str, seeds: &str) {
    let target = registry.get(&TargetId::new(id).unwrap()).unwrap();
    assert_eq!(target.package(), "surgeist-css-fuzz");
    assert_eq!(target.descriptor().binary(), binary);
    assert!(target.fixtures().iter().all(|path| path.is_dir()));
    assert!(target.fixtures().iter().any(|path| path.ends_with(seeds)));
}

#[test]
fn the_package_registers_every_target_with_its_worker_and_seed_directory() {
    let registry = registry();
    assert_registered(
        &registry,
        "surgeist-css::byte-decoding",
        "css-bytes-worker",
        "seeds/bytes",
    );
}

#[cfg(not(feature = "coverage"))]
mod process {
    use surgeist_css_fuzz::ByteTarget;
    use surgeist_fuzz::{CaseTarget, OutcomeKind, worker::*};

    fn exchange(binary: &str, canonical: Vec<u8>) {
        use std::io::Write;
        let mut request = Vec::new();
        write_frame(
            &mut request,
            &WorkerRequest::Evaluate {
                canonical: canonical.clone(),
            },
            1 << 20,
        )
        .unwrap();
        let mut child = std::process::Command::new(binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&request).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut reader = std::io::Cursor::new(output.stdout);
        assert!(matches!(
            read_frame::<_, WorkerResponse>(&mut reader, 1 << 20).unwrap(),
            Some(WorkerResponse::Captured { canonical: captured }) if captured == canonical
        ));
        assert!(matches!(
            read_frame::<_, WorkerResponse>(&mut reader, 1 << 20).unwrap(),
            Some(WorkerResponse::Evaluated { outcome, .. }) if outcome.kind() == OutcomeKind::Evaluated
                && outcome.checked_properties().len() >= 2
        ));
        assert!(
            read_frame::<_, WorkerResponse>(&mut reader, 1 << 20)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn actual_worker_binaries_publish_capture_then_a_checked_success() {
        let bytes = ByteTarget::new();
        exchange(
            env!("CARGO_BIN_EXE_css-bytes-worker"),
            bytes.encode(&bytes.literal(2).unwrap()).unwrap(),
        );
    }
}
