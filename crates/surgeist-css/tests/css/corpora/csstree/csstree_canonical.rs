//! Independent structural evidence from upstream CSSTree canonical output.
//!
//! Unlike `csstree/oracle.json`, which is captured from Surgeist's own parser
//! and therefore only characterizes current behavior, the compared text here is
//! CSSTree's `generate` expectation for each upstream-parsed fixture. Surgeist's
//! side is serialized from its public parse result, so a reinterpreted
//! combinator, a dropped declaration or a changed value fails unless the case is
//! listed with a cited reason in `css/support/csstree/canonical_exceptions.rs`.
//! Each listed case also pins the Surgeist form its reason requires, so a
//! different wrong serialization of an excepted case still fails.

use crate::css::support;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use support::csstree::{
    CANONICAL_EXCEPTIONS, CanonicalExceptionKind, canonically_equal,
    compare_csstree_canonical_forms,
};

#[test]
fn upstream_parsed_cases_match_csstree_canonical_output() {
    let summary = compare_csstree_canonical_forms().unwrap_or_else(|failures| {
        panic!(
            "{} canonical comparison failures:\n{}",
            failures.len(),
            failures.join("\n")
        )
    });
    // (context, matching cases, listed exceptions)
    assert_eq!(
        summary.compared(),
        [
            ("atrule", 49, 11),
            ("atrulePrelude", 1, 0),
            ("block", 7, 22),
            ("declaration", 22, 0),
            ("declarationList", 6, 13),
            ("mediaQuery", 41, 2),
            ("rule", 5, 26),
            ("selector", 118, 44),
            ("selectorList", 2, 0),
            ("stylesheet", 14, 34),
            ("value", 33, 18),
        ]
    );
    assert_eq!(
        summary.excluded(),
        [
            ("atrule", "no_retained_syntax", 56),
            ("atrulePrelude", "no_public_projection", 1),
            ("declaration", "no_retained_syntax", 51),
            ("rule", "no_retained_syntax", 2),
            ("selector", "no_retained_syntax", 34),
            ("value", "no_public_projection", 91),
            ("value", "no_retained_syntax", 18),
        ]
    );
    let compared: usize = summary.compared().iter().map(|row| row.1 + row.2).sum();
    let excluded: usize = summary.excluded().iter().map(|row| row.2).sum();
    assert_eq!((compared, excluded, compared + excluded), (468, 253, 721));
    // The 161 `no_retained_syntax` exclusions split as the corpus README
    // states: 142 strict rejections and 19 fully observed unsupported
    // dispositions (outside-profile, vendor or upstream-option cases). The
    // comparison's split and an independent count over the raw expected-class
    // registry must both give that split.
    let split = [("strict_rejected", 142), ("unsupported", 19)];
    assert_eq!(summary.whole_input_rejections(), split);
    assert_eq!(registry_whole_input_rejections_of_parsed_cases(), split);
    assert_eq!(
        summary.exceptions(CanonicalExceptionKind::Deliberate).len(),
        170
    );
    assert!(
        summary
            .exceptions(CanonicalExceptionKind::Defect)
            .is_empty()
    );
    assert_eq!(CANONICAL_EXCEPTIONS.len(), 170);
}

/// Counts, straight from `expected-classes.json` and the neutral expectation
/// files, the upstream-parsed cases whose expected class records a whole-input
/// rejection: empty or zero retained syntax with a `reject_input` diagnostic,
/// as a strict rejection or an unclean fully observed unsupported class.
fn registry_whole_input_rejections_of_parsed_cases() -> Vec<(&'static str, usize)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut outcomes = BTreeMap::new();
    collect_upstream_outcomes(
        &root.join("tests/corpus/csstree/expectations"),
        &mut outcomes,
    );
    let registry: Value = serde_json::from_slice(
        &fs::read(root.join("tests/csstree/expected-classes.json")).expect("read registry"),
    )
    .expect("registry JSON");
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for record in registry["records"].as_array().expect("records") {
        let id = record["id"].as_str().expect("record ID");
        if outcomes.get(id).map(String::as_str) != Some("parsed") {
            continue;
        }
        let class = &record["class"];
        let (kind, observed) = match class["kind"].as_str().expect("class kind") {
            "strict_rejected" => ("strict_rejected", class),
            "unsupported"
                if class["policy"]["kind"] == "full_observation"
                    && class["policy"]["is_clean"] == false =>
            {
                ("unsupported", &class["policy"])
            }
            _ => continue,
        };
        let predicate = &observed["retained_syntax"]["predicate"];
        let retains_nothing = predicate["relation"] == "empty"
            || (predicate["relation"] == "exact" && predicate["value"] == 0);
        let rejects_input = observed["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["action"] == "reject_input");
        if retains_nothing && rejects_input {
            *counts.entry(kind).or_default() += 1;
        }
    }
    counts.into_iter().collect()
}

fn collect_upstream_outcomes(path: &Path, outcomes: &mut BTreeMap<String, String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read expectation directory") {
            collect_upstream_outcomes(&entry.expect("expectation entry").path(), outcomes);
        }
        return;
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("json")
        || path.to_string_lossy().contains("generation-reports")
    {
        return;
    }
    let document: Value =
        serde_json::from_slice(&fs::read(path).expect("read expectation")).expect("JSON");
    for case in document["cases"].as_array().expect("cases") {
        let id = case["id"].as_str().expect("case ID").to_owned();
        let outcome = case["upstream_outcome"]
            .as_str()
            .expect("outcome")
            .to_owned();
        assert!(outcomes.insert(id, outcome).is_none(), "duplicate case ID");
    }
}

#[test]
fn normalization_removes_presentation_and_keeps_structure() {
    let same = canonically_equal;
    // Presentation choices of the two canonical generators are removed.
    assert!(same("[a|='b']", "[a|=\"b\"]"));
    assert!(same("[a=\"b\"i]", "[a=\"b\" i]"));
    assert!(same("s{p:v}", "s { p: v; }"));
    assert!(same("a,b", "a, b"));
    assert!(same("a>b", "a > b"));
    assert!(same("a/*x*/{color:red}", "a{color:red}"));
    assert!(same("(1<2<3)", "(1 < 2 < 3)"));
    assert!(same("color:red!important", "color: red !important;"));
    assert!(same("url(a\\ b.png)", "url(\"a b.png\")"));
    assert!(same("url(/*x*/a)", "url(\"/*x*/a\")"));
    // Structural and value differences remain.
    assert!(!same("a b", "a>b"));
    assert!(!same("a b", "ab"));
    assert!(!same("a :hover", "a:hover"));
    assert!(!same("a .b", "a.b"));
    assert!(!same("a [b]", "a[b]"));
    assert!(!same("a{color:red;width:1px}", "a{color:red}"));
    assert!(!same("color:red", "color:blue"));
    assert!(!same("width:1px", "width:2px"));
    assert!(!same("width:1px", "width:1em"));
    assert!(!same("width:.5px", "width:0.5px"));
    assert!(!same("A", "a"));
    assert!(!same("a{}b{}", "a{}"));
    assert!(!same("@media screen{a{}}", "@media print{a{}}"));
    assert!(!same("\"a b\"", "\"ab\""));
    assert!(!same("[a=b]", "[a=\"b\"]"));
    // A namespace prefix is not a descendant combinator before `|a`.
    assert!(!same("ns |a", "ns|a"));
    assert!(same("[a |= 'b']", "[a|=\"b\"]"));
    // A signed number token is not a `+` operator between two terms.
    assert!(!same("calc(1px + 2px)", "calc(1px+2px)"));
    assert!(!same("(x+ 1)", "(x+1)"));
    assert!(!same("calc(1px+ 2px)", "calc(1px + 2px)"));
    assert!(!same("calc(1px +2px)", "calc(1px + 2px)"));
    assert!(!same("calc(1px +.5px)", "calc(1px + .5px)"));
    assert!(!same("-/**/.5px", "-.5px"));
    // A comment between whitespace runs is still one optional run.
    assert!(!same("calc(1px + /**/ 2px)", "calc(1px+ 2px)"));
    assert!(!same("calc(1px /**/ + 2px)", "calc(1px +/**/2px)"));
    assert!(same("calc(1px + /**/ 2px)", "calc(1px + 2px)"));
    // Syntax 3 token boundaries: `1-.5px` is `1` then `-.5px`, `x1-.5` is
    // `x1-` then `.5`, and `1-5` is `1` then `-5`.
    assert!(!same("1-.5px", "1-/**/.5px"));
    assert!(same("1-.5px", "1/**/-.5px"));
    assert!(!same("1-.5px", "1/**/-/**/.5px"));
    assert!(same("x1-.5", "x1-/**/.5"));
    assert!(!same("x1-.5", "x1/**/-/**/.5"));
    assert!(same("1-5", "1/**/-5"));
    assert!(!same("1e-5px", "1e/**/-5px"));
    assert!(!same("1e", "1/**/e"));
    assert!(!same("1-px", "1/**/-px"));
    // CDC and CDO are their own tokens, not an ident and delimiters.
    assert!(!same(".-->a", ".-- >a"));
    assert!(!same(".-->a", ".--/**/>a"));
    assert!(!same("<!--a", "< ! --a"));
    // A backslash at end of input is a valid escape.
    assert!(!same("a\\", "a/**/\\"));
    // Syntax 3 preprocessing: CR LF is one newline and NUL is U+FFFD.
    assert!(same("a\r\nb", "a\nb"));
    assert!(!same("\\31\r\nx", "\\31\r x"));
    assert!(!same("a\0b", "a/**/\0/**/b"));
    // A hex escape absorbs one following whitespace.
    assert!(!same("\\31 x", "\\31/**/x"));
    // A bad url never equals a url.
    assert!(!same("url(\"a\" x)", "url(\"a\")"));
    assert!(!same("url(a b)", "url(\"a b\")"));
    // Closed and unclosed blocks stay distinct, whatever the last token holds.
    assert!(!same("(a\\)", "(a\\))"));
    assert!(!same("(/*)", "()"));
    assert!(!same("[a=\"]", "[a=\"]\"]"));
    assert!(!same("a{b:c\\}", "a{b:c\\}}"));
    assert!(!same("url(\"a)", "url(\"a)\")"));
    assert!(!same("f(a", "f(a)"));
    assert!(!same("f(a)", "f (a)"));
    // Unterminated strings and urls stay distinct from terminated ones.
    assert!(!same("\"a", "\"a\""));
    assert!(!same("url(a", "url(a)"));
    // An escaped backslash before the quote still terminates the string.
    assert!(same(r#"a{b:"c\\"}"#, r#"a{b:'c\\'}"#));
    // Backslash parity decides termination: an escaped backslash closes, an
    // escaped quote does not, and a lone quote is never closed.
    assert!(!same(r#""a\\""#, r#""a\\"#));
    assert!(!same(r#""a\""#, r#""a""#));
    assert!(!same("\"", "\"\""));
    assert!(!same(r"url(a\)", r"url(a\))"));
    assert!(same("a+b", "a + b"));
    assert!(same("calc(1px - -.5px)", "calc(1px - -.5px)"));
    // A comment still separates the tokens on either side of it.
    assert!(!same("1px/**/2px", "1px2px"));
    assert!(!same("a/**/b", "ab"));
    // `> =` is two delimiters, not the `>=` comparison.
    assert!(!same("(width > = 1px)", "(width>=1px)"));
    assert!(same("(width >= 1px)", "(width>=1px)"));
}
