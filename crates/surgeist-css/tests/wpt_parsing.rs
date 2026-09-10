//! Reviewed literal WPT adaptations at the ordinary recovering parser boundary.
//! Source-only CSSOM and execution assertions remain explicitly deferred data.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;
use surgeist_css::{CssDeclarationList, CssPropertyNameRef, CssRule, parse_sheet};

#[path = "support/digest.rs"]
mod digest;

const REVISION: &str = "ddcca5943fd41232d42149aaa19d9a04c5651b18";
const FONT_CASES: &str = include_str!("corpus/wpt/font-face-src-presence.json");
const SELECTOR_CASES: &str = include_str!("corpus/wpt/nesting-selector-presence.json");
const ORDER_CASES: &str = include_str!("corpus/wpt/nesting-declaration-order.json");
const DEFERRED_CASES: &str = include_str!("corpus/wpt/deferred-cssom-and-execution.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vectors<T> {
    schema_version: u8,
    source_revision: String,
    assertion_kind: String,
    cases: Vec<T>,
}

fn vectors<T: serde::de::DeserializeOwned>(text: &str, kind: &str) -> Vec<T> {
    let vectors: Vectors<T> = serde_json::from_str(text).expect("literal WPT vector schema");
    assert_eq!(vectors.schema_version, 1);
    assert_eq!(vectors.source_revision, REVISION);
    assert_eq!(vectors.assertion_kind, kind);
    vectors.cases
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    path: String,
    sha256: String,
    line: usize,
    case: String,
}

// This projection reads only the acquisition fields used by these tests.
// Raw source hashes use the existing shared test helper; check-corpus also
// verifies the complete imported license inventory and canonical receipt.
#[derive(Deserialize)]
struct Acquisition {
    revision: String,
    files: Vec<AcquiredFile>,
}

#[derive(Deserialize)]
struct AcquiredFile {
    path: String,
    sha256: String,
    kind: String,
    licenses: Vec<String>,
}

struct SourceText {
    declared_sha256: String,
    text: String,
}

fn source_archive() -> &'static BTreeMap<String, SourceText> {
    static SOURCES: OnceLock<BTreeMap<String, SourceText>> = OnceLock::new();
    SOURCES.get_or_init(|| {
        let acquisition: Acquisition =
            serde_json::from_str(include_str!("corpus/wpt/acquisition.json"))
                .expect("reviewed WPT acquisition schema");
        assert_eq!(acquisition.revision, REVISION);
        assert_eq!(acquisition.files.len(), 11);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/wpt/source");
        let mut sources = BTreeMap::new();
        for file in acquisition.files {
            if file.kind == "source" {
                assert_eq!(file.licenses, ["LICENSE.md"]);
                let bytes = fs::read(root.join(&file.path))
                    .unwrap_or_else(|error| panic!("read imported {}: {error}", file.path));
                assert_eq!(
                    digest::sha256_hex(&bytes),
                    file.sha256,
                    "{} raw source digest",
                    file.path
                );
                let text = String::from_utf8(bytes).expect("imported UTF-8 WPT source");
                assert!(
                    sources
                        .insert(
                            file.path,
                            SourceText {
                                declared_sha256: file.sha256,
                                text
                            }
                        )
                        .is_none()
                );
            } else {
                assert_eq!(file.kind, "license");
                assert_eq!(file.path, "LICENSE.md");
            }
        }
        assert_eq!(sources.len(), 10);
        sources
    })
}

fn assert_source_binding(binding: &Binding) -> &'static str {
    let source = source_archive()
        .get(&binding.path)
        .expect("allowlisted WPT source path");
    assert_eq!(
        binding.sha256, source.declared_sha256,
        "{} digest declaration",
        binding.path
    );
    let index = binding.line.checked_sub(1).expect("one-based source line");
    let line = source.text.lines().nth(index).expect("bound source line");
    assert_eq!(
        line.trim(),
        binding.case,
        "{}:{} literal source case",
        binding.path,
        binding.line
    );
    &source.text
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FontCase {
    id: String,
    source: Binding,
    src: String,
    expected_src_present: bool,
}

fn assert_font_case_binding(case: &FontCase) {
    assert_source_binding(&case.source);
    let literal = case
        .source
        .case
        .split_once("src:")
        .expect("literal src record")
        .1
        .trim_start()
        .strip_prefix('\'')
        .expect("single-quoted src literal")
        .split_once('\'')
        .expect("closing src quote")
        .0;
    assert_eq!(case.src, literal, "{} authored input binding", case.id);
    let expected = case
        .source
        .case
        .rsplit_once("valid:")
        .expect("literal validity")
        .1
        .trim_start();
    assert!(
        expected.starts_with(if case.expected_src_present {
            "true"
        } else {
            "false"
        }),
        "{} validity binding",
        case.id
    );
}

fn assert_src_presence(suffix: &str, expected_count: usize) {
    let cases: Vec<FontCase> = vectors(FONT_CASES, "recovering-font-face-src-presence");
    let selected: Vec<_> = cases
        .iter()
        .filter(|case| case.source.path.ends_with(suffix))
        .collect();
    assert_eq!(
        selected.len(),
        expected_count,
        "complete upstream table selection"
    );
    let mut failures = Vec::new();
    for case in selected {
        assert_font_case_binding(case);
        let input = format!("@font-face {{ src: {}}}", case.src);
        let report = parse_sheet(&input);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            failures.push(format!(
                "{} {}:{} lost its outer @font-face: {:?}",
                case.id, case.source.path, case.source.line, report
            ));
            continue;
        };
        let present = rule.descriptors().src().is_some();
        if present != case.expected_src_present {
            failures.push(format!(
                "{} {}:{} src {:?}: expected descriptor presence {}, got {}; diagnostics: {:?}",
                case.id,
                case.source.path,
                case.source.line,
                case.src,
                case.expected_src_present,
                present,
                report.diagnostics()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "WPT recovering src-presence mismatches:\n{}",
        failures.join("\n")
    );
}

#[test]
fn wpt_font_face_src_local_presence() {
    assert_src_presence("font-face-src-local.html", 18);
}

#[test]
fn wpt_font_face_src_list_preserves_valid_fallbacks() {
    assert_src_presence("font-face-src-list.html", 17);
}

#[test]
fn wpt_font_face_src_format_presence() {
    assert_src_presence("font-face-src-format.html", 35);
}

#[test]
fn wpt_font_face_src_tech_presence() {
    assert_src_presence("font-face-src-tech.html", 39);
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectorCase {
    id: String,
    source: Binding,
    parent: String,
    selector: String,
    expected_inner_present: bool,
}

fn assert_selector_case_binding(case: &SelectorCase) {
    let text = assert_source_binding(&case.source);
    let function = if case.expected_inner_present {
        "testNestedSelector"
    } else {
        "testInvalidNestingSelector"
    };
    assert!(
        case.source
            .case
            .starts_with(&format!("{function}(\"{}\"", case.selector)),
        "{} selector/acceptance binding",
        case.id
    );
    if case.source.case.contains("parent:") {
        assert!(
            case.source
                .case
                .contains(&format!("parent:\"{}\"", case.parent)),
            "{} explicit parent binding",
            case.id
        );
    } else {
        assert_eq!(case.parent, ".foo");
        assert!(
            text.contains("parent=\".foo\""),
            "upstream helper default parent"
        );
    }
}

#[test]
fn wpt_nested_selector_presence() {
    let cases: Vec<SelectorCase> = vectors(SELECTOR_CASES, "recovering-nested-style-rule-presence");
    assert_eq!(cases.len(), 32);
    let mut failures = Vec::new();
    for case in cases {
        assert_selector_case_binding(&case);
        let input = format!(
            "{} {{ {} {{ color: green; }} }}",
            case.parent, case.selector
        );
        let report = parse_sheet(&input);
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            failures.push(format!("{} lost outer rule: {:?}", case.id, report));
            continue;
        };
        let expected = usize::from(case.expected_inner_present);
        if parent.rules().len() != expected
            || parent
                .rules()
                .iter()
                .any(|rule| !matches!(rule, CssRule::Style(_)))
        {
            failures.push(format!("{} {}:{} selector {:?}: expected {expected} inner style rules, got {:?}; diagnostics: {:?}", case.id, case.source.path, case.source.line, case.selector, parent.rules(), report.diagnostics()));
        }
    }
    assert!(
        failures.is_empty(),
        "WPT nesting mismatches:\n{}",
        failures.join("\n")
    );
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RuleShape {
    Style {
        declarations: Vec<String>,
        rules: Vec<RuleShape>,
    },
    Declarations {
        declarations: Vec<String>,
    },
    Media {
        rules: Vec<RuleShape>,
    },
}

fn property_names(declarations: &CssDeclarationList) -> Vec<String> {
    declarations
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name().to_owned(),
            CssPropertyNameRef::Custom(name) => name.as_str().to_owned(),
            _ => panic!("future property-name kind in current WPT case"),
        })
        .collect()
}

fn rule_shape(rule: &CssRule) -> RuleShape {
    match rule {
        CssRule::Style(rule) => RuleShape::Style {
            declarations: property_names(rule.declarations()),
            rules: rule.rules().iter().map(rule_shape).collect(),
        },
        CssRule::NestedDeclarations(rule) => RuleShape::Declarations {
            declarations: property_names(rule.declarations()),
        },
        CssRule::Media(rule) => RuleShape::Media {
            rules: rule.rules().iter().map(rule_shape).collect(),
        },
        other => panic!("unexpected rule in selected WPT order case: {other:?}"),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrderCase {
    id: String,
    source: Binding,
    input: String,
    expected: RuleShape,
}

fn assert_order_case_binding(case: &OrderCase) {
    let text = assert_source_binding(&case.source);
    let before_label = text
        .lines()
        .take(case.source.line)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        before_label.contains(&format!("s.replaceSync(`{}`)", case.input)),
        "{} exact authored stylesheet binding",
        case.id
    );
}

#[test]
fn wpt_nested_declaration_order() {
    let cases: Vec<OrderCase> = vectors(ORDER_CASES, "authored-nested-rule-declaration-order");
    assert_eq!(cases.len(), 5);
    for case in cases {
        assert_order_case_binding(&case);
        let report = parse_sheet(&case.input);
        let [rule] = report.syntax().rules() else {
            panic!("{} expected one outer rule, got {:?}", case.id, report);
        };
        assert_eq!(
            rule_shape(rule),
            case.expected,
            "{} {}:{}; diagnostics: {:?}",
            case.id,
            case.source.path,
            case.source.line,
            report.diagnostics()
        );
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeferredCase {
    id: String,
    source: Binding,
    disposition: String,
    input: serde_json::Value,
    expectation: serde_json::Value,
    reason: String,
}

#[test]
fn wpt_vectors_bind_declared_sources_and_keep_deferred_assertions_explicit() {
    let font: Vec<FontCase> = vectors(FONT_CASES, "recovering-font-face-src-presence");
    assert_eq!(font.len(), 109);
    assert_eq!(
        font.iter().filter(|case| case.expected_src_present).count(),
        63
    );
    let mut ids = BTreeSet::new();
    for case in font {
        assert!(ids.insert(case.id.clone()), "duplicate active case");
        assert_font_case_binding(&case);
    }
    let selectors: Vec<SelectorCase> =
        vectors(SELECTOR_CASES, "recovering-nested-style-rule-presence");
    assert_eq!(selectors.len(), 32);
    for case in selectors {
        assert!(ids.insert(case.id.clone()), "duplicate active case");
        assert_selector_case_binding(&case);
    }
    let orders: Vec<OrderCase> = vectors(ORDER_CASES, "authored-nested-rule-declaration-order");
    assert_eq!(orders.len(), 5);
    for case in orders {
        assert!(ids.insert(case.id.clone()), "duplicate active case");
        assert_order_case_binding(&case);
    }
    let deferred: Vec<DeferredCase> = vectors(DEFERRED_CASES, "source-only-cssom-and-execution");
    assert_eq!(deferred.len(), 116);
    for case in deferred {
        assert!(ids.insert(case.id), "duplicate source-only case");
        assert_source_binding(&case.source);
        assert!(case.disposition.starts_with("deferred-"));
        assert!(!case.reason.is_empty());
        assert!(!case.input.is_null());
        assert!(!case.expectation.is_null());
    }
}
