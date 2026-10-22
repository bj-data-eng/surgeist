//! Reviewed literal WPT adaptations at the ordinary recovering parser boundary.
//! Nested selectorText and src tech() serialization cases replay through the
//! crate's serializers; other source-only CSSOM and execution assertions
//! remain explicitly deferred data.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;

use surgeist_css::{
    CssDeclarationList, CssNamespaceContext, CssPropertyNameRef, CssRule, CssStyleSelectorContext,
    parse_sheet,
};

use crate::css::support::digest;

const REVISION: &str = "ddcca5943fd41232d42149aaa19d9a04c5651b18";
const FONT_CASES: &str = include_str!("../../corpus/wpt/font-face-src-presence.json");
const PROFILE_EXPECTATIONS: &str = include_str!("../../corpus/wpt/profile-expectations.json");
const SELECTOR_CASES: &str = include_str!("../../corpus/wpt/nesting-selector-presence.json");
const ORDER_CASES: &str = include_str!("../../corpus/wpt/nesting-declaration-order.json");
const DEFERRED_CASES: &str = include_str!("../../corpus/wpt/deferred-cssom-and-execution.json");

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

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
            serde_json::from_str(include_str!("../../corpus/wpt/acquisition.json"))
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileExpectations {
    schema_version: u8,
    source_revision: String,
    profile_id: String,
    publication_cutoff_utc: String,
    assertion_kind: String,
    standards: Vec<ProfileStandard>,
    csswg_issue: String,
    rationale: String,
    cases: Vec<ProfileFontCase>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileStandard {
    module_id: String,
    publication_url: String,
    publication_sha256: String,
    sections: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileFontCase {
    id: String,
    source: Binding,
    src: String,
    upstream_expected_src_present: bool,
    selected_profile_expected_src_present: bool,
    reason: String,
}

fn selected_font_expectations(cases: &[FontCase]) -> BTreeMap<String, bool> {
    let profile: ProfileExpectations =
        serde_json::from_str(PROFILE_EXPECTATIONS).expect("selected-profile expectation schema");
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../../specs/catalog.json"))
            .expect("selected standards catalog");
    assert_eq!(profile.schema_version, 1);
    assert_eq!(profile.source_revision, REVISION);
    assert_eq!(
        Some(profile.profile_id.as_str()),
        catalog["profile_id"].as_str()
    );
    assert_eq!(
        Some(profile.publication_cutoff_utc.as_str()),
        catalog["publication_cutoff_utc"].as_str()
    );
    assert_eq!(profile.assertion_kind, "recovering-font-face-src-presence");
    assert_eq!(
        profile.csswg_issue,
        "https://github.com/w3c/csswg-drafts/issues/13692"
    );
    assert!(!profile.rationale.trim().is_empty());
    assert_eq!(profile.standards.len(), 2);
    let mut standard_ids = BTreeSet::new();
    for standard in profile.standards {
        assert!(standard_ids.insert(standard.module_id.clone()));
        let sections: &[&str] = match standard.module_id.as_str() {
            "css-fonts-4" => &[
                "font-family-name-syntax",
                "font-face-src-parsing",
                "local-font-fallback",
            ],
            "css-values-4" => &["custom-idents", "component-multipliers"],
            other => panic!("unreviewed profile expectation standard: {other}"),
        };
        assert_eq!(
            standard
                .sections
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            sections
        );
        let module = catalog["modules"]
            .as_array()
            .expect("catalog modules")
            .iter()
            .find(|module| module["id"].as_str() == Some(standard.module_id.as_str()))
            .expect("profile standard in selected catalog");
        assert_eq!(
            Some(standard.publication_url.as_str()),
            module["publication"]["url"].as_str()
        );
        assert_eq!(
            Some(standard.publication_sha256.as_str()),
            module["publication"]["sha256"].as_str()
        );
    }

    // This is a closed set of independently reviewed source disagreements,
    // not a mechanism for suppressing ordinary parser failures.
    let mut reviewed = BTreeMap::from([
        ("font-src-local-011", ("local(default A)", 26)),
        ("font-src-local-012", ("local(inherit A)", 27)),
        ("font-src-local-013", ("local(revert A)", 28)),
        ("font-src-local-014", ("local(unset A)", 29)),
    ]);
    assert_eq!(profile.cases.len(), reviewed.len());
    let active: BTreeMap<_, _> = cases.iter().map(|case| (case.id.as_str(), case)).collect();
    assert_eq!(active.len(), cases.len(), "unique active font case IDs");
    let mut selected = BTreeMap::new();
    for expectation in profile.cases {
        let (src, line) = reviewed
            .remove(expectation.id.as_str())
            .expect("unique, reviewed profile case ID");
        assert_eq!(expectation.src, src);
        assert_eq!(expectation.source.line, line);
        assert_eq!(
            expectation.source.path,
            "css/css-fonts/parsing/font-face-src-local.html"
        );
        let case = active
            .get(expectation.id.as_str())
            .expect("profile case binds an active font case");
        assert_font_case_binding(case);
        assert_eq!(expectation.source, case.source);
        assert_eq!(expectation.src, case.src);
        assert_eq!(
            expectation.upstream_expected_src_present,
            case.expected_src_present
        );
        assert!(expectation.upstream_expected_src_present);
        assert!(!expectation.selected_profile_expected_src_present);
        assert!(!expectation.reason.trim().is_empty());
        assert!(
            selected
                .insert(
                    expectation.id,
                    expectation.selected_profile_expected_src_present
                )
                .is_none()
        );
    }
    assert!(reviewed.is_empty(), "all four profile cases are bound");
    selected
}

fn expected_src_presence(case: &FontCase, selected: &BTreeMap<String, bool>) -> bool {
    selected
        .get(&case.id)
        .copied()
        .unwrap_or(case.expected_src_present)
}

fn assert_src_presence(suffix: &str, expected_count: usize) {
    let cases: Vec<FontCase> = vectors(FONT_CASES, "recovering-font-face-src-presence");
    let profile = selected_font_expectations(&cases);
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
        let present = ordinary_face!(rule.descriptors(), Src).is_some();
        let expected = expected_src_presence(case, &profile);
        if present != expected {
            failures.push(format!(
                "{} {}:{} src {:?}: selected profile expects descriptor presence {} (upstream {}), got {}; diagnostics: {:?}",
                case.id,
                case.source.path,
                case.source.line,
                case.src,
                expected,
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
    let profile = selected_font_expectations(&font);
    assert_eq!(
        font.iter()
            .filter(|case| expected_src_presence(case, &profile))
            .count(),
        59
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
    let mut dispositions = BTreeMap::<&str, usize>::new();
    for case in &deferred {
        assert!(ids.insert(case.id.clone()), "duplicate source-only case");
        assert_source_binding(&case.source);
        let disposition = match case.disposition.as_str() {
            NESTED_SELECTOR_TEXT | FONT_SRC_TECH_SET => case.disposition.as_str(),
            other => {
                assert!(other.starts_with("deferred-"), "{} disposition", case.id);
                "deferred"
            }
        };
        *dispositions.entry(disposition).or_default() += 1;
        assert!(!case.reason.is_empty());
        assert!(!case.input.is_null());
        assert!(!case.expectation.is_null());
    }
    // The replayed cases run in `wpt_nested_selector_text_*` and
    // `wpt_font_face_src_tech_*`; every other source-only case stays deferred.
    assert_eq!(
        dispositions,
        BTreeMap::from([
            ("deferred", 84),
            (FONT_SRC_TECH_SET, 1),
            (NESTED_SELECTOR_TEXT, 31),
        ])
    );
}

const NESTED_SELECTOR_TEXT: &str = "replayed-nested-selector-text";
const FONT_SRC_TECH_SET: &str = "replayed-font-src-tech-keyword-set";

fn replayed(disposition: &str) -> Vec<DeferredCase> {
    vectors::<DeferredCase>(DEFERRED_CASES, "source-only-cssom-and-execution")
        .into_iter()
        .filter(|case| case.disposition == disposition)
        .collect()
}

fn text_field<'a>(value: &'a serde_json::Value, field: &str, id: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("{id}: string field {field}"))
}

// WPT `testNestedSelector(sel, {expected=sel, parent=".foo"})` inserts
// `${parent} { ${sel} { color: green; }}` and reads the inner rule's
// selectorText. Nesting 1 (WD 2026-01-22) §6: "When serializing a relative
// selector in a nested style rule, the selector must be absolutized, with the
// implied nesting selector inserted"; §3.1 leaves a selector that already
// contains `&` unchanged.
// https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#cssom
#[test]
fn wpt_nested_selector_text_absolutizes_relative_selectors() {
    let cases = replayed(NESTED_SELECTOR_TEXT);
    // The rows include `.class&`, `[attr]&`, `#id&`, `:hover&` and
    // `:is(div)&`: CSSOM "serialize a selector" emits a compound's simple
    // selectors, including `&`, in authored order.
    assert_eq!(cases.len(), 31);
    let mut failures = Vec::new();
    for case in cases {
        assert_source_binding(&case.source);
        let parent = text_field(&case.input, "parent", &case.id);
        let selector = text_field(&case.input, "selector", &case.id);
        let expected = text_field(&case.expectation, "selector_text", &case.id);
        let literal = &case.source.case;
        assert!(
            literal.starts_with(&format!("testNestedSelector(\"{selector}\"")),
            "{} selector binding",
            case.id
        );
        if literal.contains("expected:") {
            assert!(
                literal.contains(&format!("expected:\"{expected}\"")),
                "{} expected binding",
                case.id
            );
        } else {
            assert_eq!(expected, selector, "{} default expectation", case.id);
        }
        if literal.contains("parent:") {
            assert!(
                literal.contains(&format!("parent:\"{parent}\"")),
                "{} parent binding",
                case.id
            );
        } else {
            assert_eq!(parent, ".foo", "{} default parent", case.id);
        }
        let input = format!("{parent} {{ {selector} {{ color: green; }}}}");
        let report = parse_sheet(&input);
        let [CssRule::Style(outer)] = report.syntax().rules() else {
            failures.push(format!("{} lost its outer rule: {report:?}", case.id));
            continue;
        };
        let [CssRule::Style(inner)] = outer.rules() else {
            failures.push(format!("{} lost its inner rule: {report:?}", case.id));
            continue;
        };
        match inner.selectors().serialize_cssom(
            &CssNamespaceContext::default(),
            CssStyleSelectorContext::Nested,
        ) {
            Ok(actual) if actual == expected => {}
            actual => failures.push(format!(
                "{} {}:{} {input:?}: expected selectorText {expected:?}, got {actual:?}",
                case.id, case.source.path, case.source.line
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "WPT nested selectorText mismatches:\n{}",
        failures.join("\n")
    );
}

/// WPT `check_same_tech`: with no `tech(` in the authored source the
/// serialization has none; otherwise the `tech(...)` lists (greedy
/// `/tech\((.+)\)/`, split on `/,\s*/`) agree once sorted, the authored list
/// lowercased after sorting.
fn same_tech(serialized: &str, specified: &str) -> Result<(), String> {
    fn techs(text: &str) -> Option<Vec<&str>> {
        let start = text.find("tech(")? + "tech(".len();
        let end = text.rfind(')')?;
        let mut list: Vec<&str> = text
            .get(start..end)?
            .split(',')
            .map(str::trim_start)
            .collect();
        list.sort_unstable();
        Some(list)
    }
    if !specified.contains("tech(") {
        return if serialized.contains("tech(") {
            Err("expected no tech() function".to_owned())
        } else {
            Ok(())
        };
    }
    let specified = techs(specified).expect("authored tech() list").join(", ");
    let serialized = techs(serialized)
        .ok_or("serialization lost its tech() list")?
        .join(", ");
    if serialized == specified.to_ascii_lowercase() {
        Ok(())
    } else {
        Err(format!(
            "tech() lists differ: {serialized:?} vs {specified:?}"
        ))
    }
}

// WPT css/css-fonts/parsing/font-face-src-tech.html compares the specified
// `src` with its getPropertyValue() serialization for each accepted row not
// marked dontcomparetech. CSSOM (editor's draft 2026-08-31, the selected
// planning edition, "serialize a CSS component value") serializes keywords
// ASCII-lowercased, and Fonts 4 §4.3.1 defines the `<font-tech>` keywords.
// https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
#[test]
fn wpt_font_face_src_tech_keyword_sets_survive_serialization() {
    let mut cases = replayed(FONT_SRC_TECH_SET);
    assert_eq!(cases.len(), 1);
    let case = cases.remove(0);
    let text = assert_source_binding(&case.source);
    assert!(text.contains("if (t.valid && !t.dontcomparetech)"));
    let table = case.source.path.as_str();
    let font: Vec<FontCase> = vectors(FONT_CASES, "recovering-font-face-src-presence");
    let profile = selected_font_expectations(&font);
    let rows: Vec<_> = font
        .iter()
        .filter(|row| row.source.path == table)
        .filter(|row| expected_src_presence(row, &profile))
        .filter(|row| !row.source.case.contains("dontcomparetech: true"))
        .collect();
    // Twelve accepted rows of the upstream table compare their technologies.
    assert_eq!(rows.len(), 12);
    let mut failures = Vec::new();
    for row in rows {
        assert_font_case_binding(row);
        let input = format!("@font-face {{ src: {}}}", row.src);
        let report = parse_sheet(&input);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            failures.push(format!("{} lost its @font-face: {report:?}", row.id));
            continue;
        };
        let Some(src) = ordinary_face!(rule.descriptors(), Src) else {
            failures.push(format!("{} lost its src descriptor: {report:?}", row.id));
            continue;
        };
        let result = src
            .serialize_specified()
            .map_err(|error| format!("{error:?}"))
            .and_then(|serialized| {
                same_tech(&serialized, &row.src).map_err(|why| format!("{serialized:?}: {why}"))
            });
        if let Err(why) = result {
            failures.push(format!("{} {:?}: {why}", row.id, row.src));
        }
    }
    assert!(
        failures.is_empty(),
        "WPT src tech() serialization mismatches:\n{}",
        failures.join("\n")
    );
}
