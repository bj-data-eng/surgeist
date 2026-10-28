//! Reviewed literal adaptations of the WPT `css/css-syntax` tests that the
//! pinned CSS Syntax 3 edition cites, exercised against this crate.
//!
//! The corpus (`tests/corpus/wpt`) holds the upstream files exactly as the
//! generator imported them and the vectors adapted from them. Every vector is
//! bound to its upstream path, the digest of the raw file, a one-based line and
//! the literal text of that line, and the binding is checked before the vector
//! is run. Each expectation was written from the upstream assertion and the
//! pinned edition, never from this crate's output.
//!
//! A vector claims only the Syntax 3 observable of its upstream case (tokens
//! and their source, rule and declaration units, `An+B` values, the way tokens
//! are written back). What the upstream test asserts through the CSS object
//! model, selector matching, property validity or `var()` substitution is not a
//! Syntax 3 result; it is stated in the vector's `remainder` or kept as a
//! deferred disposition, and is claimed nowhere.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;
use surgeist_css_parser::anb;
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, Declaration, Failed, QualifiedRule, RuleGrammar, UnitKind,
};
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::serialize::Writer;
use surgeist_css_parser::{Stream, SyntaxError, Token};

use common::digest::sha256_hex;
use common::tree::{outline_of, parse};
use common::{assert_tiles, events, lex_all, render};

const REVISION: &str = "ddcca5943fd41232d42149aaa19d9a04c5651b18";
const LICENSE_SHA256: &str = "5fac07febb0e2a97fb0d7b0def149ec08b642e1ba4b9c345283ab1cbd2af6570";

const ACQUISITION: &str = include_str!("corpus/wpt/acquisition.json");
const ANB: &str = include_str!("corpus/wpt/anb.json");
const TOKENS: &str = include_str!("corpus/wpt/tokens.json");
const NOT_WHITESPACE: &str = include_str!("corpus/wpt/not-whitespace.json");
const UNITS: &str = include_str!("corpus/wpt/units.json");
const CONSECUTIVE: &str = include_str!("corpus/wpt/consecutive-tokens.json");
const ROUND_TRIPS: &str = include_str!("corpus/wpt/escaped-identifier-round-trips.json");
const UNCLOSED: &str = include_str!("corpus/wpt/unclosed-constructs.json");
const DEFERRED: &str = include_str!("corpus/wpt/deferred.json");

/// The 23 tests the edition cites outside `css/css-syntax/charset/`.
const CITED: [&str; 23] = [
    "anb-parsing",
    "anb-serialization",
    "at-rule-in-declaration-list",
    "cdc-vs-ident-tokens",
    "charset-is-not-a-rule",
    "custom-property-rule-ambiguity",
    "decimal-points-in-numbers",
    "declarations-trim-whitespace",
    "escaped-eof",
    "ident-three-code-points",
    "inclusive-ranges",
    "input-preprocessing",
    "invalid-nested-rules",
    "non-ascii-codepoints",
    "parse-invalid-input",
    "serialize-consecutive-tokens",
    "serialize-escape-identifiers",
    "trailing-braces",
    "unclosed-constructs",
    "unclosed-url-at-eof",
    "url-whitespace-consumption",
    "var-with-blocks",
    "whitespace",
];

const DISPOSITIONS: [&str; 3] = [
    "adapted-at-step-8a",
    "pending-step-15",
    "computed-value-substitution",
];

// ------------------------------------------------------------------ the data

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    path: String,
    sha256: String,
    line: usize,
    case: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnbCase {
    id: String,
    source: Binding,
    input: String,
    expected: String,
    a: Option<i32>,
    b: Option<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenCase {
    id: String,
    source: Binding,
    input: String,
    tokens: Vec<(String, String)>,
    #[serde(default)]
    input_literal: bool,
    errors: Option<Vec<String>>,
    #[allow(dead_code)]
    remainder: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NotWhitespaceCase {
    id: String,
    source: Binding,
    input: String,
    #[allow(dead_code)]
    remainder: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct GrammarSpec {
    /// Declaration names the grammar says are not valid.
    reject_declarations: Vec<String>,
    /// Qualified rules whose prelude starts with one of these are not valid.
    reject_qualified_starting: Vec<String>,
    /// Read the block of a top-level rule as a block's contents.
    blocks_as_contents: bool,
    /// Describe a custom property by its original text, as its value is.
    original_for_custom: bool,
    /// Report only the declaration of this name in a top-level rule's block.
    focus: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitCase {
    id: String,
    source: Binding,
    input: String,
    context: String,
    grammar: GrammarSpec,
    expected: Vec<String>,
    input_literal: bool,
    #[allow(dead_code)]
    remainder: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairCase {
    id: String,
    source: Binding,
    first: String,
    second: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoundTripCase {
    id: String,
    source: Binding,
    input: String,
    tokens: Vec<String>,
    #[allow(dead_code)]
    remainder: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnclosedCase {
    id: String,
    source: Binding,
    closed: String,
    unclosed: String,
    closed_outline: String,
    unclosed_outline: String,
    #[allow(dead_code)]
    remainder: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeferredCase {
    id: String,
    source: Binding,
    disposition: String,
    input: String,
    expectation: String,
    reason: String,
}

// ------------------------------------------------------- the upstream files

#[derive(Deserialize)]
struct Acquisition {
    revision: String,
    files: Vec<AcquiredFile>,
    cited_but_absent_at_revision: Vec<AbsentFile>,
}

#[derive(Deserialize)]
struct AcquiredFile {
    path: String,
    sha256: String,
    kind: String,
    licenses: Vec<String>,
}

#[derive(Deserialize)]
struct AbsentFile {
    path: String,
    http_status: u16,
}

struct SourceText {
    declared_sha256: String,
    text: String,
}

fn acquisition() -> Acquisition {
    serde_json::from_str(ACQUISITION).expect("reviewed WPT acquisition schema")
}

/// The imported upstream files, each read from the corpus and checked against
/// the digest of the acquisition record before anything uses it.
fn archive() -> &'static BTreeMap<String, SourceText> {
    static SOURCES: OnceLock<BTreeMap<String, SourceText>> = OnceLock::new();
    SOURCES.get_or_init(|| {
        let acquisition = acquisition();
        assert_eq!(acquisition.revision, REVISION);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/wpt/source");
        let mut sources = BTreeMap::new();
        for file in acquisition.files {
            let bytes = fs::read(root.join(&file.path))
                .unwrap_or_else(|error| panic!("read imported {}: {error}", file.path));
            assert_eq!(sha256_hex(&bytes), file.sha256, "{} raw digest", file.path);
            if file.kind == "source" {
                assert_eq!(file.licenses, ["LICENSE.md"]);
                let text = String::from_utf8(bytes).expect("imported UTF-8 WPT source");
                let source = SourceText {
                    declared_sha256: file.sha256,
                    text,
                };
                assert!(sources.insert(file.path, source).is_none());
            } else {
                assert_eq!(file.kind, "license");
                assert_eq!(file.path, "LICENSE.md");
                assert_eq!(file.sha256, LICENSE_SHA256);
            }
        }
        sources
    })
}

/// Why `binding` does not name the upstream line it claims to, if it does not.
fn binding_problem(binding: &Binding) -> Option<String> {
    let Some(upstream) = archive().get(&binding.path) else {
        return Some(format!("{} is not an allowlisted file", binding.path));
    };
    if binding.sha256 != upstream.declared_sha256 {
        return Some(format!("{} digest declaration differs", binding.path));
    }
    let Some(index) = binding.line.checked_sub(1) else {
        return Some("upstream lines are one-based".to_owned());
    };
    let Some(line) = upstream.text.split('\n').nth(index) else {
        return Some(format!("{} has no line {}", binding.path, binding.line));
    };
    (line.trim() != binding.case).then(|| {
        format!(
            "{}:{} is {:?}, not {:?}",
            binding.path,
            binding.line,
            line.trim(),
            binding.case
        )
    })
}

fn upstream_of(binding: &Binding) -> &'static SourceText {
    if let Some(problem) = binding_problem(binding) {
        panic!("{problem}");
    }
    &archive()[&binding.path]
}

/// Every vector of every file, as (id, binding) pairs.
fn all_bindings() -> Vec<(String, Binding)> {
    fn take<T>(cases: Vec<T>, f: impl Fn(T) -> (String, Binding)) -> Vec<(String, Binding)> {
        cases.into_iter().map(f).collect()
    }
    let mut all = Vec::new();
    all.extend(take(anb_cases(), |c| (c.id, c.source)));
    all.extend(take(token_cases(), |c| (c.id, c.source)));
    all.extend(take(not_whitespace_cases(), |c| (c.id, c.source)));
    all.extend(take(unit_cases(), |c| (c.id, c.source)));
    all.extend(take(pair_cases(), |c| (c.id, c.source)));
    all.extend(take(round_trip_cases(), |c| (c.id, c.source)));
    all.extend(take(unclosed_cases(), |c| (c.id, c.source)));
    all.extend(take(deferred_cases(), |c| (c.id, c.source)));
    all
}

fn anb_cases() -> Vec<AnbCase> {
    vectors(ANB, "anb-recognition-and-serialization")
}
fn token_cases() -> Vec<TokenCase> {
    vectors(TOKENS, "tokens-and-sources")
}
fn not_whitespace_cases() -> Vec<NotWhitespaceCase> {
    vectors(NOT_WHITESPACE, "code-point-is-not-a-whitespace-token")
}
fn unit_cases() -> Vec<UnitCase> {
    vectors(UNITS, "rule-and-declaration-units")
}
fn pair_cases() -> Vec<PairCase> {
    vectors(CONSECUTIVE, "consecutive-tokens-need-a-comment")
}
fn round_trip_cases() -> Vec<RoundTripCase> {
    vectors(ROUND_TRIPS, "token-serialization-round-trip")
}
fn unclosed_cases() -> Vec<UnclosedCase> {
    vectors(UNCLOSED, "unclosed-groups-share-their-contents")
}
fn deferred_cases() -> Vec<DeferredCase> {
    vectors(DEFERRED, "deferred-dispositions")
}

/// Panics with every failure at once, so one bad vector hides no other.
fn assert_no_failures(what: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} of the {what} vectors fail:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// -------------------------------------------------------- binding and coverage

#[test]
fn the_digest_helper_gives_the_published_sha256_answers() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(&[b'a'; 1000]),
        "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
    );
}

#[test]
fn the_acquisition_record_binds_every_imported_file_by_digest() {
    // `archive` reads each file from the corpus and checks its digest against
    // the record; a license is present and is the upstream text.
    assert_eq!(archive().len(), 22);
    assert!(
        archive()
            .keys()
            .all(|path| path.starts_with("css/css-syntax/") && !path.contains("/charset/"))
    );
    let license =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/wpt/source/LICENSE.md"))
            .expect("imported license");
    assert_eq!(sha256_hex(&license), LICENSE_SHA256);
}

#[test]
fn every_cited_test_is_acquired_or_recorded_as_absent_at_the_pin() {
    let acquisition = acquisition();
    let mut accounted = BTreeSet::new();
    for path in archive().keys() {
        accounted.insert(path.clone());
    }
    for absent in &acquisition.cited_but_absent_at_revision {
        assert_eq!(absent.http_status, 404);
        assert!(accounted.insert(absent.path.clone()), "{}", absent.path);
    }
    let cited: BTreeSet<String> = CITED
        .iter()
        .map(|name| format!("css/css-syntax/{name}.html"))
        .collect();
    assert_eq!(accounted, cited);
    let absent: Vec<&str> = acquisition
        .cited_but_absent_at_revision
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    assert_eq!(absent, ["css/css-syntax/parse-invalid-input.html"]);
}

#[test]
fn every_binding_names_its_upstream_line_and_digest() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for (id, binding) in all_bindings() {
        checked += 1;
        if let Some(problem) = binding_problem(&binding) {
            failures.push(format!("{id}: {problem}"));
        }
    }
    assert!(checked > 300, "{checked} bindings");
    assert_no_failures("binding", failures);
}

#[test]
fn an_input_marked_literal_is_text_of_its_upstream_file() {
    let mut failures = Vec::new();
    let mut literal = 0;
    for case in token_cases() {
        if case.input_literal {
            literal += 1;
            let upstream = upstream_of(&case.source);
            if !upstream.text.contains(&case.input) {
                failures.push(case.id);
            }
        }
    }
    for case in unit_cases() {
        if case.input_literal {
            literal += 1;
            let upstream = upstream_of(&case.source);
            if !upstream.text.contains(&case.input) {
                failures.push(case.id);
            }
        }
    }
    assert!(literal > 30, "{literal} literal inputs");
    assert_no_failures("literal input", failures);
}

#[test]
fn identifiers_are_unique_and_every_acquired_test_has_a_vector() {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for (id, binding) in all_bindings() {
        assert!(ids.insert(id.clone()), "duplicate vector identifier {id}");
        paths.insert(binding.path);
    }
    let acquired: BTreeSet<String> = archive().keys().cloned().collect();
    assert_eq!(paths, acquired);
}

/// Upstream cases that can be counted from the file's text, against the
/// vectors bound to that file (claimed and deferred together).
///
/// Two files cannot be counted this way: `inclusive-ranges.html` and
/// `non-ascii-codepoints.html` build their cases in JavaScript loops over
/// computed code points (start, middle, end and the neighbours of each range),
/// which counting lines does not reproduce; their vectors were generated from
/// the same ranges and are checked one by one against the upstream line they
/// came from.
#[test]
fn the_vectors_account_for_every_countable_upstream_case() {
    let text = |name: &str| &archive()[&format!("css/css-syntax/{name}.html")].text;
    let starting = |name: &str, prefix: &str| {
        text(name)
            .lines()
            .filter(|line| line.trim().starts_with(prefix))
            .count()
    };
    let containing = |name: &str, needle: &str| {
        text(name)
            .lines()
            .filter(|line| line.contains(needle))
            .count()
    };
    // The items of the one-line array that follows `marker`.
    let items = |name: &str, marker: &str| {
        let line = text(name)
            .lines()
            .find(|line| line.contains(marker))
            .unwrap_or_else(|| panic!("{name} has no {marker}"));
        let inside = &line[line.find('[').unwrap() + 1..line.find(']').unwrap()];
        inside.split(',').count()
    };
    // The lines of the `<style id="sheet">` block, one rule each.
    let sheet_lines = |name: &str| {
        text(name)
            .lines()
            .skip_while(|line| !line.contains("<style id=\"sheet\">"))
            .skip(1)
            .take_while(|line| line.trim() != "</style>")
            .count()
    };
    let bound = |name: &str| {
        let path = format!("css/css-syntax/{name}.html");
        all_bindings()
            .into_iter()
            .filter(|(_, binding)| binding.path == path)
            .count()
    };
    let expected = [
        ("anb-parsing", starting("anb-parsing", "testANB(\"")),
        (
            "anb-serialization",
            starting("anb-serialization", "testANB(\""),
        ),
        (
            "at-rule-in-declaration-list",
            starting("at-rule-in-declaration-list", "test("),
        ),
        // The one test, a token view of it and an ident control.
        (
            "cdc-vs-ident-tokens",
            starting("cdc-vs-ident-tokens", "test(") + 2,
        ),
        (
            "charset-is-not-a-rule",
            starting("charset-is-not-a-rule", "test("),
        ),
        (
            "custom-property-rule-ambiguity",
            starting("custom-property-rule-ambiguity", "test("),
        ),
        (
            "decimal-points-in-numbers",
            starting("decimal-points-in-numbers", "test("),
        ),
        // Nine tests and the whole sheet.
        (
            "declarations-trim-whitespace",
            starting("declarations-trim-whitespace", "test(") + 1,
        ),
        // Two vectors per test: the first (hash) test has two token views (a
        // hash that is an id, and one that is not) and no declaration view;
        // each other test has a token view and a declaration view.
        ("escaped-eof", starting("escaped-eof", "test(") * 2),
        (
            "ident-three-code-points",
            containing("ident-three-code-points", "class=\"item\""),
        ),
        (
            "input-preprocessing",
            starting("input-preprocessing", "testParsing(\""),
        ),
        (
            "invalid-nested-rules",
            starting("invalid-nested-rules", "test("),
        ),
        (
            "serialize-consecutive-tokens",
            starting("serialize-consecutive-tokens", "testTokenPairs(\"")
                + starting("serialize-consecutive-tokens", "testComments(\"")
                + 1,
        ),
        (
            "serialize-escape-identifiers",
            sheet_lines("serialize-escape-identifiers"),
        ),
        ("trailing-braces", starting("trailing-braces", "test(")),
        // Four calls, each a closed control and an unclosed experiment.
        (
            "unclosed-constructs",
            containing("unclosed-constructs", "shouldBeValid(\"") / 2,
        ),
        // Four elements, each a token view and a declaration view.
        (
            "unclosed-url-at-eof",
            containing("unclosed-url-at-eof", "<div id=test") * 2,
        ),
        (
            "url-whitespace-consumption",
            containing("url-whitespace-consumption", "assert_equals(roundTripUrl("),
        ),
        ("var-with-blocks", starting("var-with-blocks", "test(")),
        (
            "whitespace",
            items("whitespace", "var whitespace = [")
                + items("whitespace", "var notWhitespace = ["),
        ),
    ];
    let mut failures = Vec::new();
    for (name, upstream) in expected {
        let vectors = bound(name);
        if upstream != vectors {
            failures.push(format!("{name}: {upstream} upstream, {vectors} vectors"));
        }
    }
    assert_no_failures("countable-case", failures);
}

#[test]
fn a_deferred_case_names_a_known_disposition_and_a_reason() {
    let mut seen = BTreeSet::new();
    for case in deferred_cases() {
        assert!(
            DISPOSITIONS.contains(&case.disposition.as_str()),
            "{}: {}",
            case.id,
            case.disposition
        );
        assert!(case.reason.len() > 40, "{}", case.id);
        assert!(!case.input.is_empty() && !case.expectation.is_empty());
        seen.insert(case.disposition);
    }
    assert_eq!(seen, DISPOSITIONS.iter().map(|d| (*d).to_owned()).collect());
}

// ---------------------------------------------------------------------- An+B

#[test]
fn an_plus_b_vectors_parse_to_the_value_upstream_serializes() {
    let mut failures = Vec::new();
    for case in anb_cases() {
        let document = parse(&case.input);
        let mut stream = document.root();
        // The upstream test puts the text in `:nth-child(...)`, so the whole of
        // it must be one `<an+b>`; anything left over is a parse error.
        let value = anb::parse(&mut stream).ok().and_then(|value| {
            stream.discard_whitespace();
            stream.is_empty().then_some(value)
        });
        let message = match (value, case.a, case.b) {
            (None, None, None) if case.expected == "parse error" => continue,
            (Some(value), Some(a), Some(b))
                if (value.a(), value.b()) == (a, b) && value.to_string() == case.expected =>
            {
                continue;
            }
            (Some(value), ..) => format!(
                "{}: {:?} gave A={} B={} written {:?}, expected {:?} (A={:?} B={:?})",
                case.id,
                case.input,
                value.a(),
                value.b(),
                value.to_string(),
                case.expected,
                case.a,
                case.b
            ),
            (None, ..) => format!(
                "{}: {:?} is not an <an+b>, expected {:?}",
                case.id, case.input, case.expected
            ),
        };
        failures.push(message);
    }
    assert_no_failures("An+B", failures);
}

// -------------------------------------------------------------------- tokens

fn token_sources(text: &str) -> Vec<(String, String)> {
    lex_all(text)
        .0
        .into_iter()
        .map(|item| (item.token, text[item.range].to_owned()))
        .collect()
}

#[test]
fn token_vectors_lex_to_the_tokens_and_sources_the_edition_gives() {
    let mut failures = Vec::new();
    for case in token_cases() {
        assert_tiles(&case.input);
        let actual = token_sources(&case.input);
        if actual != case.tokens {
            failures.push(format!(
                "{}: {:?}\n   expected {:?}\n   actual   {:?}",
                case.id, case.input, case.tokens, actual
            ));
        }
        if let Some(errors) = &case.errors {
            let kinds: Vec<&str> = events(&case.input).iter().map(|event| event.kind).collect();
            if kinds != *errors {
                failures.push(format!(
                    "{}: lexical errors {kinds:?}, expected {errors:?}",
                    case.id
                ));
            }
        }
    }
    assert_no_failures("token", failures);
}

#[test]
fn a_code_point_that_is_not_css_whitespace_makes_no_whitespace_token() {
    let mut failures = Vec::new();
    for case in not_whitespace_cases() {
        assert_tiles(&case.input);
        let tokens = token_sources(&case.input);
        if tokens.iter().any(|(token, _)| token == "Whitespace") {
            failures.push(format!("{}: {:?} gave {tokens:?}", case.id, case.input));
        }
    }
    assert_no_failures("not-whitespace", failures);
}

// --------------------------------------------------------------------- units

fn closer(open: Token<'_>) -> Token<'static> {
    match open {
        Token::OpenCurly => Token::CloseCurly,
        Token::OpenSquare => Token::CloseSquare,
        _ => Token::CloseParen,
    }
}

fn write_stream(stream: &mut Stream<'_>, writer: &mut Writer<String>) {
    while let Ok(token) = stream.consume_token() {
        writer.token(token).expect("a String accepts writes");
        if stream.unentered_block().is_some() {
            stream
                .within_block(|inner| {
                    write_stream(inner, writer);
                    Ok::<_, SyntaxError<'_, ()>>(())
                })
                .expect("a block is waiting");
            writer
                .token(closer(token))
                .expect("a String accepts writes");
        }
    }
}

/// What the rest of `stream` is written as (section 9), whitespace collapsed
/// and trimmed.
fn text_of(stream: &Stream<'_>) -> String {
    let mut writer = Writer::new(String::new());
    write_stream(&mut stream.clone(), &mut writer);
    writer.into_inner().trim().to_owned()
}

#[derive(Clone, Copy, Debug)]
struct Rejected;

struct Describe<'s> {
    spec: &'s GrammarSpec,
    depth: usize,
}

impl Describe<'_> {
    fn block(&self, block: Stream<'_>) -> String {
        if !(self.spec.blocks_as_contents && self.depth == 0) {
            return text_of(&block);
        }
        let mut inner = Describe {
            spec: self.spec,
            depth: self.depth + 1,
        };
        let items: Vec<String> = block
            .block_contents(&mut inner)
            .filter_map(returned)
            .filter(|item| {
                self.spec
                    .focus
                    .as_ref()
                    .is_none_or(|name| item.starts_with(&format!("decl {name} ")))
            })
            .collect();
        items.join(" | ")
    }
}

impl<'i> RuleGrammar<'i> for Describe<'_> {
    type Item = String;
    type Error = Rejected;

    fn at_rule(&mut self, rule: AtRule<'i>) -> Result<String, SyntaxError<'i, Rejected>> {
        let block = rule
            .block()
            .map_or_else(String::new, |block| format!(" {{{}}}", self.block(block)));
        Ok(format!(
            "@{} [{}]{block}",
            rule.name(),
            text_of(&rule.prelude())
        ))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i>,
    ) -> Result<String, SyntaxError<'i, Rejected>> {
        let prelude = text_of(&rule.prelude());
        if self
            .spec
            .reject_qualified_starting
            .iter()
            .any(|start| prelude.starts_with(start.as_str()))
        {
            return Err(SyntaxError::custom(rule.start(), Rejected));
        }
        Ok(format!("rule [{prelude}] {{{}}}", self.block(rule.block())))
    }
}

impl<'i> BlockGrammar<'i> for Describe<'_> {
    fn declaration(
        &mut self,
        declaration: Declaration<'i>,
    ) -> Result<String, SyntaxError<'i, Rejected>> {
        if self
            .spec
            .reject_declarations
            .iter()
            .any(|name| name == declaration.name())
        {
            return Err(SyntaxError::custom(declaration.start(), Rejected));
        }
        let mut text = match declaration.original_text() {
            Some(original) if self.spec.original_for_custom => {
                format!("decl {} = {original:?}", declaration.name())
            }
            _ => format!(
                "decl {}: {}",
                declaration.name(),
                text_of(&declaration.value())
            ),
        };
        if declaration.is_important() {
            text.push_str(" !important");
        }
        Ok(text)
    }
}

/// What the algorithms return for a unit. The crate reports a construct that an
/// algorithm drops ("return nothing") as a failed unit so that a host can say
/// what was dropped; the algorithms return no value for it, so only an invalid
/// rule error, which the algorithms do return, is described here.
fn returned(unit: Result<String, Failed<'_, Rejected>>) -> Option<String> {
    match unit {
        Ok(item) => Some(item),
        Err(failed) if failed.is_invalid_rule_error() => {
            let kind = match failed.kind() {
                UnitKind::AtRule => "at-rule",
                UnitKind::QualifiedRule => "rule",
                UnitKind::Declaration => "declaration",
                _ => "other",
            };
            Some(format!("invalid rule error: {kind}"))
        }
        Err(_) => None,
    }
}

#[test]
fn unit_vectors_give_the_rules_and_declarations_the_algorithms_consume() {
    let mut failures = Vec::new();
    for case in unit_cases() {
        let document = parse(&case.input);
        let mut grammar = Describe {
            spec: &case.grammar,
            depth: 0,
        };
        let actual: Vec<String> = match case.context.as_str() {
            "sheet" => document
                .root()
                .stylesheet_contents(&mut grammar)
                .filter_map(returned)
                .collect(),
            "block" => document
                .root()
                .block_contents(&mut grammar)
                .filter_map(returned)
                .collect(),
            other => panic!("{}: unknown context {other}", case.id),
        };
        if actual != case.expected {
            failures.push(format!(
                "{}: {:?}\n   expected {:?}\n   actual   {:?}",
                case.id, case.input, case.expected, actual
            ));
        }
    }
    assert_no_failures("unit", failures);
}

// ------------------------------------------------------------ serialization

/// The rendered tokens of `text` that are not whitespace or comments.
fn non_trivia(text: &str) -> Vec<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut out = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        if !lexeme.token().is_trivia() {
            out.push(render(lexeme.token()));
        }
    }
    out
}

fn write_tokens(text: &str, writer: &mut Writer<String>) {
    let mut lexer = Lexer::new(text, &Options::default());
    while let Some(lexeme) = lexer.next_lexeme() {
        writer
            .token(lexeme.token())
            .expect("a String accepts writes");
    }
}

#[test]
fn consecutive_tokens_are_written_with_a_comment_that_keeps_them_apart() {
    let mut failures = Vec::new();
    for case in pair_cases() {
        let first = non_trivia(&case.first);
        let second = non_trivia(&case.second);
        let mut writer = Writer::new(String::new());
        write_tokens(&case.first, &mut writer);
        write_tokens(&case.second, &mut writer);
        let written = writer.into_inner();
        let naive = format!("{}{}", case.first, case.second);
        let mut problems = Vec::new();
        if !written.starts_with(&case.first) {
            problems.push("does not start with the first token");
        }
        if !written.ends_with(&case.second) {
            problems.push("does not end with the second token");
        }
        if written == naive {
            problems.push("has no comment between the tokens");
        }
        let both: Vec<String> = first.iter().chain(&second).cloned().collect();
        if non_trivia(&written) != both {
            problems.push("reads back as other tokens");
        }
        if !problems.is_empty() {
            failures.push(format!(
                "{}: {:?} then {:?} written {written:?}: {}",
                case.id,
                case.first,
                case.second,
                problems.join(", ")
            ));
        }
    }
    assert_no_failures("consecutive-token", failures);
}

#[test]
fn escaped_names_read_back_as_the_same_tokens_once_written() {
    let mut failures = Vec::new();
    for case in round_trip_cases() {
        let read = non_trivia(&case.input);
        let mut writer = Writer::new(String::new());
        write_tokens(&case.input, &mut writer);
        let written = writer.into_inner();
        let again = non_trivia(&written);
        if read != case.tokens || again != case.tokens {
            failures.push(format!(
                "{}: {:?} read {read:?}, written {written:?} read {again:?}, expected {:?}",
                case.id, case.input, case.tokens
            ));
        }
    }
    assert_no_failures("round-trip", failures);
}

// -------------------------------------------------------------------- groups

#[test]
fn an_unclosed_group_holds_what_the_closed_group_holds() {
    let mut failures = Vec::new();
    for case in unclosed_cases() {
        let closed = outline_of(&case.closed);
        let unclosed = outline_of(&case.unclosed);
        if closed != case.closed_outline || unclosed != case.unclosed_outline {
            failures.push(format!(
                "{}: closed {closed:?} (expected {:?}), unclosed {unclosed:?} (expected {:?})",
                case.id, case.closed_outline, case.unclosed_outline
            ));
        }
        // Ending at the end of the text is not a tokenizer parse error.
        for text in [&case.closed, &case.unclosed] {
            let kinds: Vec<&str> = events(text).iter().map(|event| event.kind).collect();
            if !kinds.is_empty() {
                failures.push(format!("{}: {text:?} reports {kinds:?}", case.id));
            }
        }
    }
    assert_no_failures("unclosed-group", failures);
}
