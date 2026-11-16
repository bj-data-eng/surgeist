#![forbid(unsafe_code)]
//! The imported CSSTree seeds are exactly what their owner's corpus holds, the
//! import record accounts for every corpus case, and each seed passes the
//! shared evaluator through canonical replay.
//!
//! The totals come from the owner's own records (`expected_cases` in
//! `corpus.toml` and `counts.active` in the generation report) and from a
//! recursive walk of its expectation files, not from the importer's reader.
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use surgeist_css_fuzz::SourceTarget;
use surgeist_fuzz::{CaseTarget, OutcomeKind, engine};

fn package() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn corpus() -> PathBuf {
    package().join("../tests/corpus/csstree")
}

fn record() -> toml::Table {
    toml::from_str(&fs::read_to_string(package().join("corpus-import.toml")).unwrap()).unwrap()
}

fn report() -> Value {
    serde_json::from_slice(
        &fs::read(corpus().join("expectations/generation-reports/all.json")).unwrap(),
    )
    .unwrap()
}

/// Every expectation file under the corpus, however deeply nested.
fn expectation_files(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name != "generation-reports")
            {
                expectation_files(&path, found);
            }
        } else if path.extension().is_some_and(|ext| ext == "json") {
            found.push(path);
        }
    }
}

/// Every corpus case by context (the first directory under `expectations/`),
/// found by walking the tree: id to input.
fn walked_cases() -> BTreeMap<String, BTreeMap<String, String>> {
    let root = corpus().join("expectations");
    let mut files = Vec::new();
    expectation_files(&root, &mut files);
    let mut by_context: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for file in files {
        let context = file
            .strip_prefix(&root)
            .unwrap()
            .components()
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned();
        let document: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        for case in document["cases"].as_array().unwrap() {
            let id = case["id"].as_str().unwrap().to_owned();
            let input = case["input"].as_str().unwrap().to_owned();
            assert!(
                by_context
                    .entry(context.clone())
                    .or_default()
                    .insert(id.clone(), input)
                    .is_none(),
                "duplicate corpus id {id}"
            );
        }
    }
    by_context
}

#[test]
fn the_owner_records_agree_on_the_corpus_total_the_walk_finds() {
    let manifest: toml::Table =
        toml::from_str(&fs::read_to_string(corpus().join("corpus.toml")).unwrap()).unwrap();
    let expected = manifest["source"]["expected_cases"].as_integer().unwrap();
    let report = report();
    assert_eq!(report["counts"]["active"].as_i64(), Some(expected));
    let walked: usize = walked_cases().values().map(BTreeMap::len).sum();
    assert_eq!(i64::try_from(walked).unwrap(), expected);
}

#[test]
fn the_import_record_matches_the_owner_corpus_revision_and_licence() {
    let record = record();
    let report = report();
    assert_eq!(
        record["source_revision"].as_str(),
        report["source_revision"].as_str()
    );
    assert_eq!(
        record["manifest_digest"].as_str(),
        report["manifest_digest"].as_str(),
        "the corpus was regenerated; rerun the import tool"
    );
    let licence = package()
        .join("../../..")
        .join(record["license"].as_str().unwrap());
    // The unchanged upstream notice: permission to use, copy and distribute.
    assert!(
        fs::read_to_string(licence)
            .unwrap()
            .contains("Permission is hereby granted, free of charge")
    );
}

#[test]
fn every_seed_is_a_corpus_input_and_every_corpus_case_is_imported_or_accounted() {
    let record = record();
    let directory = package().join(record["seed_directory"].as_str().unwrap());
    let seeds = record["seed"].as_array().unwrap();
    let walked = walked_cases();
    let all: BTreeMap<&String, &String> = walked.values().flatten().collect();

    for seed in seeds {
        let id = seed["corpus_id"].as_str().unwrap();
        let file = seed["file"].as_str().unwrap();
        let expected = all
            .get(&id.to_owned())
            .unwrap_or_else(|| panic!("unknown corpus id {id}"));
        assert_eq!(
            fs::read(directory.join(file)).unwrap(),
            expected.as_bytes(),
            "{file} is not the corpus input of {id}"
        );
    }
    assert_eq!(fs::read_dir(&directory).unwrap().count(), seeds.len());

    // Imported, duplicate and excluded cases add up to the owner's recorded
    // total, read from the owner's files and not from the importer.
    let manifest: toml::Table =
        toml::from_str(&fs::read_to_string(corpus().join("corpus.toml")).unwrap()).unwrap();
    let owner_total =
        usize::try_from(manifest["source"]["expected_cases"].as_integer().unwrap()).unwrap();
    assert_eq!(
        usize::try_from(record["corpus_cases"].as_integer().unwrap()).unwrap(),
        owner_total
    );
    let duplicates = usize::try_from(record["duplicate_inputs"].as_integer().unwrap()).unwrap();
    let excluded = record["excluded"].as_array().unwrap();
    assert_eq!(excluded.len(), 5);
    let mut excluded_total = 0;
    for entry in excluded {
        let context = entry["context"].as_str().unwrap();
        let count = usize::try_from(entry["cases"].as_integer().unwrap()).unwrap();
        assert_eq!(count, walked[context].len(), "{context}");
        assert!(!entry["reason"].as_str().unwrap().is_empty());
        excluded_total += count;
    }
    assert_eq!(seeds.len() + duplicates + excluded_total, owner_total);

    // Every context that is not excluded was imported (apart from duplicates).
    let excluded_contexts: Vec<&str> = excluded
        .iter()
        .map(|entry| entry["context"].as_str().unwrap())
        .collect();
    let included_total: usize = walked
        .iter()
        .filter(|(context, _)| !excluded_contexts.contains(&context.as_str()))
        .map(|(_, cases)| cases.len())
        .sum();
    assert_eq!(seeds.len() + duplicates, included_total);
}

#[test]
fn every_imported_seed_passes_the_shared_evaluator_and_replays_exactly() {
    let record = record();
    let directory = package().join(record["seed_directory"].as_str().unwrap());
    let target = SourceTarget::new();
    let mut evaluated = 0;
    for seed in record["seed"].as_array().unwrap() {
        let bytes = fs::read(directory.join(seed["file"].as_str().unwrap())).unwrap();
        let observed = engine::replay(&target, &bytes, &mut |_| Ok(())).unwrap();
        assert_eq!(
            observed.outcome.kind(),
            OutcomeKind::Evaluated,
            "{}: {:?}",
            seed["file"],
            observed.outcome.detail()
        );
        assert_eq!(observed.canonical, bytes);
        assert_eq!(
            target.encode(&target.decode(&bytes).unwrap()).unwrap(),
            bytes
        );
        evaluated += 1;
    }
    assert_eq!(evaluated, record["seed"].as_array().unwrap().len());
    assert!(evaluated > 0);
}
