//! Recoverable inputs that must produce a report, not a failure.
//!
//! Every expected tree and diagnostic list below is derived by hand from the
//! pinned HTML Standard (revision 877d6146487f, retained under `references/`),
//! never from parser output:
//!
//! * `</>`: 13.2.5.7 End tag open state. U+003E is a `missing-end-tag-name`
//!   parse error; the state switches to the data state and emits no token, so
//!   the document parses as if the three characters were absent.
//! * Adoption agency inside template contents: 13.2.6.4.7 "in body", the
//!   adoption agency algorithm. When the formatting element sits directly under
//!   a template element, that template is the common ancestor. Step 13.6
//!   creates the replacement elements with it as the intended parent, and step
//!   14 inserts the last node at the adjusted insertion location for it, which
//!   13.2.6.1 places inside the template's contents, after their last child.

use crate::parser::support::{fragment_context, limits};
use surgeist_html::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseDiagnosticKind,
    HtmlParseDiagnosticLocationRole, HtmlParseReport, HtmlPrologueDiagnostic, HtmlSource,
    HtmlTokenizerDiagnostic, HtmlTreeDiagnostic,
};

/// Renders the children of `parent` as an indented tree. A template's contents
/// are shown under a `content` line before its ordinary children.
fn render<'a>(
    lookup: &dyn Fn(&HtmlNodeId) -> HtmlNode<'a>,
    parent: HtmlNode<'a>,
    depth: usize,
    out: &mut String,
) {
    let pad = "  ".repeat(depth);
    for id in parent.children() {
        let node = lookup(&id);
        match node.kind() {
            HtmlNodeKind::Element(element) => {
                let name = String::from_utf16(element.name().local_name().code_units()).unwrap();
                out.push_str(&format!("{pad}<{name}>\n"));
                if let Some(contents) = node.template_contents() {
                    out.push_str(&format!("{pad}  content\n"));
                    render(lookup, lookup(&contents), depth + 2, out);
                }
            }
            HtmlNodeKind::Text(text) => {
                let text = String::from_utf16(text.code_units()).unwrap();
                out.push_str(&format!("{pad}\"{text}\"\n"));
            }
            // Initial, "a DOCTYPE token": missing identifiers become empty.
            HtmlNodeKind::Doctype(doctype) => {
                assert_eq!(doctype.public_id(), Some(&DocumentString::from("")));
                assert_eq!(doctype.system_id(), Some(&DocumentString::from("")));
                let name = String::from_utf16(doctype.name().code_units()).unwrap();
                out.push_str(&format!("{pad}<!DOCTYPE {name}>\n"));
            }
            other => panic!("unexpected node kind {other:?}"),
        }
        render(lookup, node, depth + 1, out);
    }
}

fn sources(input: &str) -> [HtmlSource; 2] {
    [
        HtmlSource::utf8(input),
        HtmlSource::utf16(DocumentString::from(input)),
    ]
}

fn parse_document(source: HtmlSource) -> HtmlParseReport {
    HtmlDocumentParseRequest::offline(source, limits())
        .parse()
        .unwrap_or_else(|failure| panic!("document must parse: {:?}", failure.reason()))
}

fn parse_fragment(source: HtmlSource) -> HtmlParseReport {
    HtmlFragmentParseRequest::with_options(
        source,
        fragment_context(),
        limits(),
        HtmlParserScriptingMode::Inert,
        false,
    )
    .unwrap()
    .parse()
    .unwrap_or_else(|failure| panic!("fragment must parse: {:?}", failure.reason()))
}

fn document_tree(report: &HtmlParseReport) -> (HtmlDocumentMode, String) {
    let document = report.document().unwrap();
    let mut out = String::new();
    render(
        &|id| document.node(id).unwrap(),
        document.root(),
        0,
        &mut out,
    );
    (document.mode(), out)
}

fn fragment_tree(report: &HtmlParseReport) -> String {
    let fragment = report.fragment().unwrap();
    let mut out = String::new();
    render(
        &|id| fragment.node(id).unwrap(),
        fragment.root(),
        0,
        &mut out,
    );
    out
}

/// One expected diagnostic: its typed kind and the pinned rule it cites.
type Expected = (HtmlParseDiagnosticKind, &'static str, &'static str);

fn assert_diagnostics(report: &HtmlParseReport, expected: &[Expected], what: &str) {
    let diagnostics = report.diagnostics();
    let actual: Vec<_> = (0..diagnostics.len())
        .map(|index| {
            let (record, _) = diagnostics.retained_record(index).unwrap().unwrap();
            (
                record.kind(),
                record.rule().anchor(),
                record.rule().branch(),
            )
        })
        .collect();
    assert_eq!(actual, expected, "{what}");
}

const MISSING_END_TAG_NAME: Expected = (
    HtmlParseDiagnosticKind::Tokenizer(HtmlTokenizerDiagnostic::MissingEndTagName),
    "parse-error-missing-end-tag-name",
    "missing-end-tag-name",
);
const MISSING_DOCTYPE: Expected = (
    HtmlParseDiagnosticKind::Prologue(HtmlPrologueDiagnostic::MissingDoctype),
    "the-initial-insertion-mode",
    "anything-else/missing-doctype",
);
const ACTIVE_FORMATTING_A: Expected = (
    HtmlParseDiagnosticKind::Tree(HtmlTreeDiagnostic::UnexpectedFormatting),
    "parsing-main-inbody",
    "a-start/active-formatting-a",
);
const NOBR_IN_SCOPE: Expected = (
    HtmlParseDiagnosticKind::Tree(HtmlTreeDiagnostic::UnexpectedFormatting),
    "parsing-main-inbody",
    "nobr-start/nobr-in-scope",
);
/// Adoption agency step 4.6: the formatting element is not the current node.
const NOT_CURRENT: Expected = (
    HtmlParseDiagnosticKind::Tree(HtmlTreeDiagnostic::UnexpectedFormatting),
    "parsing-main-inbody",
    "adoption/formatting-element-not-current",
);
/// "In template" end-of-file: a template element is still open.
const TEMPLATE_EOF: Expected = (
    HtmlParseDiagnosticKind::Tree(HtmlTreeDiagnostic::UnexpectedOpenElement),
    "parsing-main-intemplate",
    "template-eof/template-open",
);

#[test]
fn document_missing_end_tag_name_parses_as_if_the_markup_were_absent() {
    // End tag open state, U+003E: error, data state, no token. With a doctype
    // the document is no-quirks; the EOF then completes html, head and body.
    let cases: [(&str, &str); 4] = [
        (
            "<!doctype html></>",
            "<!DOCTYPE html>\n<html>\n  <head>\n  <body>\n",
        ),
        (
            "<!doctype html><p></>",
            "<!DOCTYPE html>\n<html>\n  <head>\n  <body>\n    <p>\n",
        ),
        // The surrounding character tokens append to one text node.
        (
            "<!doctype html><p>a</>b</p>",
            "<!DOCTYPE html>\n<html>\n  <head>\n  <body>\n    <p>\n      \"ab\"\n",
        ),
        (
            "<!doctype html></></><p></>",
            "<!DOCTYPE html>\n<html>\n  <head>\n  <body>\n    <p>\n",
        ),
    ];
    for (input, tree) in cases {
        let errors = input.matches("</>").count();
        for source in sources(input) {
            let report = parse_document(source);
            assert_eq!(
                document_tree(&report),
                (HtmlDocumentMode::NoQuirks, tree.to_owned()),
                "{input:?}"
            );
            assert_diagnostics(&report, &vec![MISSING_END_TAG_NAME; errors], input);
        }
    }
}

#[test]
fn document_missing_end_tag_name_without_a_doctype_reports_the_tokenizer_error_first() {
    // The missing-end-tag-name error is raised while tokenizing, before the
    // end-of-file token reaches the initial insertion mode, which then reports
    // the missing doctype and selects quirks mode.
    for source in sources("</>") {
        let report = parse_document(source);
        assert_eq!(
            document_tree(&report),
            (
                HtmlDocumentMode::Quirks,
                "<html>\n  <head>\n  <body>\n".to_owned()
            )
        );
        assert_diagnostics(&report, &[MISSING_END_TAG_NAME, MISSING_DOCTYPE], "</>");
    }
}

#[test]
fn document_missing_end_tag_name_diagnostic_locates_the_greater_than_sign() {
    // `<!doctype html>` is 15 code units, so `</>` occupies 15..18 and the
    // input frontier of the error is the U+003E at 17..18.
    for (source, utf16) in sources("<!doctype html></>").into_iter().zip([false, true]) {
        let report = parse_document(source);
        let diagnostics = report.diagnostics();
        assert_eq!(diagnostics.len(), 1, "utf16={utf16}");
        let (_, location) = diagnostics.retained_record(0).unwrap().unwrap();
        assert_eq!(
            location.role(),
            HtmlParseDiagnosticLocationRole::InputFrontier
        );
        assert_eq!(
            location.source_range(),
            &report.source().range(17, 18).unwrap(),
            "utf16={utf16}"
        );
    }
}

#[test]
fn fragment_missing_end_tag_name_parses_as_if_the_markup_were_absent() {
    let cases: [(&str, &str, usize); 5] = [
        ("</>", "", 1),
        ("<p></>", "<p>\n", 1),
        ("a</>b", "\"ab\"\n", 1),
        ("</></>", "", 2),
        ("x</><p>y</>", "\"x\"\n<p>\n  \"y\"\n", 2),
    ];
    for (input, tree, errors) in cases {
        for source in sources(input) {
            let report = parse_fragment(source);
            assert_eq!(fragment_tree(&report), tree, "{input:?}");
            assert_diagnostics(&report, &vec![MISSING_END_TAG_NAME; errors], input);
        }
    }
}

/// Adoption agency runs whose common ancestor is a template element. Each
/// entry is the input, the document tree below html, the fragment tree and the
/// ordered diagnostics common to both (the document adds a missing doctype,
/// and a doctype-free input is quirks mode).
///
/// 1. `<template><a><i><pre><a>`: the second `a` start tag finds `a` after the
///    last marker (parse error) and runs the algorithm. Stack: html, head,
///    template, a, i, pre. Formatting element a is not the current node
///    (parse error); furthest block pre; common ancestor template. The inner
///    loop replaces i by i' (created for intended parent template), appends
///    pre to i' and ends at a. i' goes into the template contents after a. The
///    new a' takes pre's children (none) and is appended to pre, replacing a in
///    the lists; its own run finds no furthest block and pops through a'. The
///    start tag then inserts a'' into pre. At EOF the open template is a
///    parse error.
/// 2. `<template><nobr><b><pre><nobr>`: the same run started by the nobr rule.
/// 3. `<template><a><b><p>x</a>y`: the same run started by an end tag; the text
///    "x" moves with p's children into a', and "y" then joins it.
const TEMPLATE_RUNS: [(&str, &str, &[Expected]); 3] = [
    (
        "<template><a><i><pre><a>",
        "<template>\n  content\n    <a>\n      <i>\n    <i>\n      <pre>\n        <a>\n        <a>\n",
        &[ACTIVE_FORMATTING_A, NOT_CURRENT, TEMPLATE_EOF],
    ),
    (
        "<template><nobr><b><pre><nobr>",
        "<template>\n  content\n    <nobr>\n      <b>\n    <b>\n      <pre>\n        <nobr>\n        <nobr>\n",
        &[NOBR_IN_SCOPE, NOT_CURRENT, TEMPLATE_EOF],
    ),
    (
        "<template><a><b><p>x</a>y",
        "<template>\n  content\n    <a>\n      <b>\n    <b>\n      <p>\n        <a>\n          \"xy\"\n",
        &[NOT_CURRENT, TEMPLATE_EOF],
    ),
];

#[test]
fn document_adoption_agency_in_template_contents_follows_the_standard() {
    for (input, template, diagnostics) in TEMPLATE_RUNS {
        // Without a doctype the initial insertion mode reports it first and
        // selects quirks mode. The template lands in head, as "in head" rules.
        let tree = format!(
            "<html>\n  <head>\n{}  <body>\n",
            template
                .lines()
                .map(|line| format!("    {line}\n"))
                .collect::<String>()
        );
        let mut expected = vec![MISSING_DOCTYPE];
        expected.extend_from_slice(diagnostics);
        for source in sources(input) {
            let report = parse_document(source);
            assert_eq!(
                document_tree(&report),
                (HtmlDocumentMode::Quirks, tree.clone()),
                "{input:?}"
            );
            assert_diagnostics(&report, &expected, input);
        }
    }
}

#[test]
fn fragment_adoption_agency_in_template_contents_follows_the_standard() {
    // The fragment's root html element sits below the template on the stack of
    // open elements, so the template is again the common ancestor.
    for (input, template, diagnostics) in TEMPLATE_RUNS {
        for source in sources(input) {
            let report = parse_fragment(source);
            assert_eq!(fragment_tree(&report), template, "{input:?}");
            assert_diagnostics(&report, diagnostics, input);
        }
    }
}
