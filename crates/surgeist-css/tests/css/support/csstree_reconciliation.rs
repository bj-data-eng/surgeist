//! One harness for the handwritten CSSTree reconciliation files in `tests/csstree/`.
//!
//! Each `*-reconciliation.json` row is an independent, specification-derived
//! prediction for one neutral corpus case. The harness binds a row to the
//! validated neutral inventory, admits the row's own expected class through the
//! class gate, and observes the case through the same registry adapter, public
//! entry point and extractor that oracle capture uses. Diagnostics are therefore
//! projected once, with closed code and action names and with
//! `payload_relation_holds` as the only classifier of a diagnostic's relation to
//! its payload. A caller supplies the predicted observation for each row and,
//! optionally, a typed-meaning check over the public parse result.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use surgeist_css::selector::{CssLanguageRangeKind, CssNamespaceContext};

use super::adapters::{EntryPoint, Extractor as RegistryExtractor};
use super::{
    CORPUS_ROOT, EXPECTED_CLASSES_PATH, corpus_namespace_context, observe_csstree_record,
    raw_extractor, read_neutral_artifact_set, validate_expected_class,
    validate_neutral_artifact_set,
};
use crate::css::support::digest::sha256_hex;

/// The directory a reconciliation file's row `source_path` and
/// `expectation_path` values are relative to.
#[derive(Clone, Copy, Debug)]
pub(crate) enum RowPaths {
    /// Paths begin at the crate root, for example `tests/corpus/...`.
    Crate,
    /// Paths begin at the crate's `tests` directory, for example `corpus/...`.
    Tests,
}

/// What one row predicts the corpus observer records for its case.
#[derive(Debug)]
pub(crate) struct PredictedObservation {
    pub(crate) syntax_count: u64,
    pub(crate) is_clean: bool,
    /// Ordered diagnostics in the oracle's raw diagnostic form. A row that does
    /// not record `payload_relation` still passes each diagnostic through the
    /// classifier, which rejects a diagnostic for which no closed relation holds.
    pub(crate) diagnostics: Vec<Value>,
}

/// The public entry point and extractor every row of one file is observed with.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ObservationRoute {
    pub(crate) entry_point: EntryPoint,
    /// `None` leaves the extractor to the fixture's registry entry; the row's
    /// expected class still has to name that same extractor.
    pub(crate) extractor: Option<RegistryExtractor>,
}

/// A typed-meaning check of one row against the public parse result.
pub(crate) type TypedMeaning<'a> = &'a dyn Fn(&Value) -> Result<(), String>;

pub(crate) struct Reconciliation {
    file: &'static str,
    paths: RowPaths,
    data: Value,
}

impl Reconciliation {
    /// Loads `tests/csstree/<file>`.
    pub(crate) fn load(file: &'static str, paths: RowPaths) -> Self {
        let path = crate_root().join("tests/csstree").join(file);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let data = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        Self { file, paths, data }
    }

    pub(crate) fn data(&self) -> &Value {
        &self.data
    }

    pub(crate) fn rows(&self) -> &[Value] {
        self.data["rows"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: rows", self.file))
    }

    /// Observes every selected row through the corpus observer and compares
    /// it with the row's prediction, then applies the typed-meaning check.
    /// Returns the number of selected rows.
    pub(crate) fn reconcile_observations(
        &self,
        route: ObservationRoute,
        select: impl Fn(&Value) -> bool,
        predicted: impl Fn(&Value) -> PredictedObservation,
        typed_meaning: Option<TypedMeaning<'_>>,
    ) -> usize {
        let root = crate_root().join(CORPUS_ROOT);
        let mut inventory = read_neutral_artifact_set(&root)
            .and_then(validate_neutral_artifact_set)
            .unwrap_or_else(|error| panic!("{}: neutral inventory: {error}", self.file));
        let mut seen = 0;
        let mut failures = Vec::new();
        for row in self.rows().iter().filter(|row| select(row)) {
            seen += 1;
            let id = text(row, "id");
            let Some(case) = inventory.cases.iter_mut().find(|case| case.id == id) else {
                failures.push(format!("{id}: no neutral case"));
                continue;
            };
            let mut binding = Vec::new();
            if case.input != text(row, "input") {
                binding.push("input");
            }
            if case.expectation_sha256 != text(row, "expectation_sha256") {
                binding.push("expectation_sha256");
            }
            if effective_options(row)
                .is_some_and(|options| serde_json::to_value(&case.options).unwrap() != *options)
            {
                binding.push("options");
            }
            if !binding.is_empty() {
                failures.push(format!("{id}: neutral binding differs: {binding:?}"));
                continue;
            }
            let expected_class = match serde_json::from_value(row["expected_class"].clone()) {
                Ok(class) => class,
                Err(error) => {
                    failures.push(format!("{id}: expected class schema: {error}"));
                    continue;
                }
            };
            if let Err(error) = validate_expected_class(case, &expected_class) {
                failures.push(format!("{id}: independent class admission failed: {error}"));
                continue;
            }
            case.expected_class = Some(expected_class);
            let record = match observe_csstree_record(case) {
                Ok(record) => serde_json::to_value(record).unwrap(),
                Err(error) => {
                    failures.push(format!(
                        "{id}: real adapter/class admission failed: {}",
                        error.summary()
                    ));
                    continue;
                }
            };
            let prediction = predicted(row);
            let expected = json!({
                "entry_point": route.entry_point,
                "prefix": "",
                "suffix": "",
                "input_byte_length": case.input.len(),
                "outcome": row["expected_class"]["kind"],
                "extractor": route
                    .extractor
                    .map_or_else(|| record["observation"]["extractor"].clone(), |extractor| {
                        serde_json::to_value(raw_extractor(extractor)).unwrap()
                    }),
                "syntax_count": prediction.syntax_count,
                "is_clean": prediction.is_clean,
                "diagnostics": prediction.diagnostics,
            });
            let observed_diagnostics = record["observation"]["diagnostics"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let actual = json!({
                "entry_point": record["probe"]["entry_point"],
                "prefix": record["probe"]["payload"]["prefix"],
                "suffix": record["probe"]["payload"]["suffix"],
                "input_byte_length": record["probe"]["payload"]["input_byte_length"],
                "outcome": record["outcome"]["kind"],
                "extractor": record["observation"]["extractor"],
                "syntax_count": record["observation"]["syntax_count"],
                "is_clean": record["observation"]["is_clean"],
                "diagnostics": comparable_diagnostics(&observed_diagnostics, &expected["diagnostics"]),
            });
            if actual != expected {
                failures.push(format!("{id}\nexpected: {expected}\nactual: {actual}"));
            }
            if let Some(check) = typed_meaning
                && let Err(error) = check(row)
            {
                failures.push(format!("{id}: {error}"));
            }
        }
        assert!(
            failures.is_empty(),
            "{}: {} of {seen} rows differ:\n{}",
            self.file,
            failures.len(),
            failures.join("\n")
        );
        seen
    }

    /// Requires the CSS-owned class registry to hold exactly one record per row,
    /// bound to the row's neutral expectation digest and equal to its class.
    pub(crate) fn assert_registry_classes(&self) {
        let path = crate_root().join(EXPECTED_CLASSES_PATH);
        let registry: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let records = registry["records"].as_array().unwrap();
        let mut failures = Vec::new();
        for row in self.rows() {
            let id = text(row, "id");
            let matching: Vec<_> = records.iter().filter(|record| record["id"] == id).collect();
            let [record] = matching.as_slice() else {
                failures.push(format!("{id}: {} registry records", matching.len()));
                continue;
            };
            assert_eq!(
                record["expectation_sha256"], row["expectation_sha256"],
                "{}: {id}",
                self.file
            );
            if record["class"] != row["expected_class"] {
                failures.push(format!(
                    "{id}\nexpected: {}\nactual: {}",
                    row["expected_class"], record["class"]
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "{}: {} of {} registry classes differ:\n{}",
            self.file,
            failures.len(),
            self.rows().len(),
            failures.join("\n")
        );
    }

    /// Binds each row to its original neutral case: unique identity, recorded
    /// input/options/source/expectation digests, the original case record when
    /// the row retains it, and the case's input, context, active status and
    /// options. Returns the number of bound rows.
    pub(crate) fn assert_original_bindings<'a>(
        &self,
        rows: impl IntoIterator<Item = &'a Value>,
        context: &str,
    ) -> usize {
        let mut ids = BTreeSet::new();
        for row in rows {
            let id = text(row, "id");
            assert!(ids.insert(id), "{}: duplicate original {id}", self.file);
            if row.get("input_sha256").is_some() {
                assert_eq!(
                    sha256_hex(text(row, "input").as_bytes()),
                    text(row, "input_sha256"),
                    "{id}"
                );
            }
            if row.get("options_sha256").is_some() {
                assert_eq!(
                    sha256_hex(&serde_json::to_vec(effective_options(row).unwrap()).unwrap()),
                    text(row, "options_sha256"),
                    "{id}"
                );
            }
            let source = fs::read(self.row_path(row, "source_path")).unwrap();
            assert_eq!(sha256_hex(&source), text(row, "source_sha256"), "{id}");
            let neutral = fs::read(self.row_path(row, "expectation_path")).unwrap();
            assert_eq!(
                sha256_hex(&neutral),
                text(row, "expectation_sha256"),
                "{id}"
            );
            let neutral: Value = serde_json::from_slice(&neutral).unwrap();
            let original: Vec<_> = neutral["cases"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|case| case["id"] == id)
                .collect();
            let [original] = original.as_slice() else {
                panic!("{id}: {} neutral cases", original.len());
            };
            if let Some(retained) = row.get("original_expectation") {
                assert_eq!(*original, retained, "{id}: original expectation");
            }
            assert_eq!(original["input"], row["input"], "{id}: input");
            assert_eq!(original["context"], context, "{id}: context");
            assert_eq!(original["status"], "active", "{id}: status");
            for key in ["context", "status"] {
                if let Some(value) = row.get(key) {
                    assert_eq!(&original[key], value, "{id}: {key}");
                }
            }
            let authored = original
                .get("options")
                .cloned()
                .unwrap_or_else(|| json!({}));
            if row.get("effective_options").is_some() {
                assert_eq!(
                    original.get("options").is_some(),
                    row["options_present"].as_bool().unwrap(),
                    "{id}: options presence"
                );
                assert_eq!(original["options"], row["options"], "{id}: options");
            }
            if let Some(options) = effective_options(row) {
                assert_eq!(&authored, options, "{id}: effective options");
            }
        }
        ids.len()
    }

    /// The JSON file a row's `source_path` or `expectation_path` names.
    pub(crate) fn row_json(&self, row: &Value, key: &str) -> Value {
        serde_json::from_slice(&fs::read(self.row_path(row, key)).unwrap()).unwrap()
    }

    fn row_path(&self, row: &Value, key: &str) -> PathBuf {
        let base = match self.paths {
            RowPaths::Crate => crate_root().to_path_buf(),
            RowPaths::Tests => crate_root().join("tests"),
        };
        base.join(text(row, key))
    }
}

/// The crate's current specification catalog.
pub(crate) fn catalog() -> Value {
    serde_json::from_slice(&fs::read(crate_root().join("specs/catalog.json")).unwrap()).unwrap()
}

/// The current catalog module with the given identifier.
pub(crate) fn catalog_module(id: &Value) -> Value {
    catalog()["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|module| module["id"] == *id)
        .unwrap_or_else(|| panic!("catalog module {id}"))
        .clone()
}

/// Binds each `{id, publication}` normative source to the catalog's current
/// publication of that module. Historical whole-catalog digests remain
/// provenance; unrelated catalog additions do not alter the selected editions.
pub(crate) fn assert_selected_publications(sources: &Value) {
    for source in sources.as_array().unwrap() {
        assert_eq!(
            catalog_module(&source["id"])["publication"],
            source["publication"],
            "{}",
            source["id"]
        );
    }
}

/// The namespace bindings the corpus observer parses selectors and rules with,
/// for typed-meaning checks that parse the same input through the public API.
pub(crate) fn namespace_context() -> CssNamespaceContext {
    corpus_namespace_context()
}

/// The reconciliation spelling of a language range's authored token form.
pub(crate) const fn language_range_token_kind(kind: CssLanguageRangeKind) -> &'static str {
    match kind {
        CssLanguageRangeKind::Identifier => "identifier",
        CssLanguageRangeKind::String => "string",
        _ => "future",
    }
}

pub(crate) fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key]
        .as_str()
        .unwrap_or_else(|| panic!("string {key} in {row}"))
}

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// A row's options as observed: `effective_options` when the row separates
/// them from the authored `options`, otherwise `options`, or `None` when the
/// row records no options.
fn effective_options(row: &Value) -> Option<&Value> {
    row.get("effective_options").or_else(|| row.get("options"))
}

/// Drops `payload_relation` from each observed diagnostic whose prediction
/// does not record one; every other field is compared as observed.
fn comparable_diagnostics(observed: &[Value], predicted: &Value) -> Value {
    let predicted = predicted.as_array().map(Vec::as_slice).unwrap_or_default();
    Value::Array(
        observed
            .iter()
            .enumerate()
            .map(|(index, diagnostic)| {
                let mut diagnostic = diagnostic.clone();
                if predicted
                    .get(index)
                    .is_some_and(|predicted| predicted.get("payload_relation").is_none())
                {
                    diagnostic
                        .as_object_mut()
                        .unwrap()
                        .remove("payload_relation");
                }
                diagnostic
            })
            .collect(),
    )
}
