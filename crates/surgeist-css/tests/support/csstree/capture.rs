use super::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub(crate) fn capture_csstree_oracle_from_public_parser() -> Result<(), String> {
    if std::env::var_os("SURGEIST_CSSTREE_WRITE_ORACLE").as_deref()
        != Some(std::ffi::OsStr::new("1"))
    {
        return Err("refusing to write CSS oracle without SURGEIST_CSSTREE_WRITE_ORACLE=1".into());
    }

    let inventory = load_neutral_csstree_inventory().map_err(str::to_owned)?;
    require_expected_classes_digest(&inventory.expected_classes_digest)?;
    let bytes = observe_csstree_oracle(inventory).map_err(|failures| {
        failures
            .iter()
            .map(BaselineFailure::summary)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    require_expected_classes_digest(&inventory.expected_classes_digest)?;
    let replacement = load_csstree_oracle_schema(&bytes)?;
    validate_oracle(
        &bytes,
        &inventory.report_digest,
        &inventory.expected_classes_digest,
        &inventory.cases,
    )?;

    let oracle_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(ORACLE_PATH);
    let existing = read_file(&oracle_path)?;
    let existing = load_csstree_oracle_schema(&existing)?;
    validate_replacement_bindings(&existing, &replacement)?;
    atomic_write_oracle(&oracle_path, &bytes)?;
    require_expected_classes_digest(&inventory.expected_classes_digest)?;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
    read_artifact_set(&root)
        .and_then(validate_artifact_set)
        .map(|_| ())
        .map_err(|error| format!("reloading ValidatedCorpus after oracle capture failed: {error}"))
}

fn require_expected_classes_digest(expected: &str) -> Result<(), String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED_CLASSES_PATH);
    let actual = sha256_hex(&read_file(&path)?);
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "immutable expected-class registry changed during parser/capture command: expected {expected}, found {actual}"
        ))
    }
}

pub(super) fn validate_replacement_bindings(
    existing: &RawOracle,
    replacement: &RawOracle,
) -> Result<(), String> {
    // Neutral source identity is immutable during owner-oracle maintenance.
    // The owner expectation registry may advance: capture has already checked
    // the replacement against that registry and separately checks that its
    // digest stays unchanged throughout the operation.
    if existing.schema_version != replacement.schema_version
        || existing.provider_repository != replacement.provider_repository
        || existing.source_revision != replacement.source_revision
        || existing.source_tree != replacement.source_tree
        || existing.expectation_schema_version != replacement.expectation_schema_version
        || existing.generation_report_sha256 != replacement.generation_report_sha256
    {
        return Err("refusing to replace CSS oracle with drifted metadata bindings".into());
    }
    if existing.records.len() != replacement.records.len() {
        return Err("refusing to replace CSS oracle with a different record census".into());
    }
    for (existing, replacement) in existing.records.iter().zip(&replacement.records) {
        if existing.id != replacement.id
            || existing.path != replacement.path
            || existing.expectation_sha256 != replacement.expectation_sha256
            || existing.source != replacement.source
            || existing.context != replacement.context
            || existing.options != replacement.options
            || existing.input != replacement.input
        {
            return Err(format!(
                "refusing to replace CSS oracle record with drifted neutral binding: {}",
                replacement.id
            ));
        }
    }
    Ok(())
}

fn atomic_write_oracle(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let directory = path
        .parent()
        .ok_or_else(|| format!("CSS oracle has no parent directory: {}", path.display()))?;
    let temporary: PathBuf = directory.join(".oracle.json.surgeist-capture.tmp");
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| {
                format!(
                    "failed to create same-directory CSS oracle temporary file {}: {error}",
                    temporary.display()
                )
            })?;
        file.write_all(bytes)
            .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("failed to sync {}: {error}", temporary.display()))?;
        fs::rename(&temporary, path).map_err(|error| {
            format!(
                "failed to atomically rename {} to {}: {error}",
                temporary.display(),
                path.display()
            )
        })
    })();
    if write_result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}
