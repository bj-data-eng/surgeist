//! The `Diagnostics` cap bounds what a report retains, not whether a parse
//! completes (#1753).
//!
//! The parse-error counts below are derived by hand from the pinned HTML
//! Standard (revision 877d6146487f, retained under `references/`), never from
//! parser output:
//!
//! * `< ` repeated: 13.2.5.6 Tag open state. A U+0020 after U+003C is an
//!   `invalid-first-character-of-tag-name` parse error; the state emits U+003C
//!   as a character token and reconsumes the space in the data state. Each pair
//!   is one parse error, so `N` pairs are `N` tokenizer errors, and the text is
//!   never a tag.
//! * A document with no DOCTYPE: 13.2.6.4.1 "initial", "anything else" is one
//!   parse error (the document is not an iframe srcdoc document).
//! * U+0000 in body content: 13.2.5.1 Data state raises an
//!   `unexpected-null-character` parse error and emits the character, then
//!   13.2.6.4.7 "in body", "A character token that is U+0000 NULL" raises a
//!   second parse error and ignores the token. Each NUL is two errors.
//!
//! Which tree each input produces does not need a hand-written oracle: the
//! tree of a parse that retains every diagnostic is the oracle for a parse that
//! retains fewer, because the cap bounds retention only.

use crate::parser::support::{fragment_context, limits_builder};
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeId, HtmlNodeKind},
    parsing::HtmlParserScriptingMode,
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseDiagnosticKind, HtmlParseLimits,
    HtmlParseReport, HtmlParseResource, HtmlSource,
};

/// Renders every node below `parent`, with its kind, in document order.
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
                let name = String::from_utf16_lossy(element.name().local_name().code_units());
                out.push_str(&format!("{pad}<{name}>\n"));
                if let Some(contents) = node.template_contents() {
                    out.push_str(&format!("{pad}  content\n"));
                    render(lookup, lookup(&contents), depth + 2, out);
                }
            }
            HtmlNodeKind::Text(text) => {
                let text = String::from_utf16_lossy(text.code_units());
                out.push_str(&format!("{pad}{text:?}\n"));
            }
            other => out.push_str(&format!("{pad}{other:?}\n")),
        }
        render(lookup, node, depth + 1, out);
    }
}

fn tree(report: &HtmlParseReport) -> String {
    let mut out = String::new();
    if let Some(document) = report.document() {
        out.push_str(&format!("mode {:?}\n", document.mode()));
        render(
            &|id| document.node(id).unwrap(),
            document.root(),
            0,
            &mut out,
        );
    } else {
        let fragment = report.fragment().unwrap();
        render(
            &|id| fragment.node(id).unwrap(),
            fragment.root(),
            0,
            &mut out,
        );
    }
    out
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Document,
    Fragment,
}

fn parse(
    kind: Kind,
    input: &str,
    limits: HtmlParseLimits,
) -> Result<HtmlParseReport, surgeist_html_parser::HtmlParseFailure> {
    let source = HtmlSource::utf8(input);
    match kind {
        Kind::Document => HtmlDocumentParseRequest::offline(source, limits).parse(),
        Kind::Fragment => HtmlFragmentParseRequest::with_options(
            source,
            fragment_context(),
            limits,
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
        .parse(),
    }
}

fn with_cap(cap: u64) -> HtmlParseLimits {
    limits_builder().diagnostics(cap).finish().unwrap()
}

/// What the report exposes of one retained diagnostic.
type Retained = (
    HtmlParseDiagnosticKind,
    &'static str,
    &'static str,
    surgeist_html_parser::HtmlSourceRange,
);

fn retained(report: &HtmlParseReport) -> Vec<Retained> {
    let diagnostics = report.diagnostics();
    (0..diagnostics.len())
        .map(|index| {
            let (record, location) = diagnostics.retained_record(index).unwrap().unwrap();
            (
                record.kind(),
                record.rule().anchor(),
                record.rule().branch(),
                location.source_range().clone(),
            )
        })
        .collect()
}

struct Case {
    name: &'static str,
    kind: Kind,
    input: String,
    /// The number of parse errors the Standard requires for `input`, when it
    /// is derived by hand above; the other inputs rely on the uncapped parse.
    required: Option<usize>,
}

fn cases() -> Vec<Case> {
    const N: usize = 40;
    let pairs = "< ".repeat(N);
    let nuls = "\0".repeat(N);
    vec![
        Case {
            name: "document of lt-space pairs",
            kind: Kind::Document,
            input: pairs.clone(),
            required: Some(N + 1),
        },
        Case {
            name: "fragment of lt-space pairs",
            kind: Kind::Fragment,
            input: pairs,
            required: Some(N),
        },
        Case {
            name: "document of NULs in body",
            kind: Kind::Document,
            input: format!("<p>{nuls}"),
            required: Some(1 + 2 * N),
        },
        Case {
            name: "fragment of NULs in body",
            kind: Kind::Fragment,
            input: nuls.clone(),
            required: Some(2 * N),
        },
        Case {
            name: "document of NULs in a table",
            kind: Kind::Document,
            input: format!("<table>{nuls}"),
            required: None,
        },
        Case {
            name: "document of NULs in foreign content",
            kind: Kind::Document,
            input: format!("<svg>{nuls}"),
            required: None,
        },
        Case {
            name: "document of text in a frameset",
            kind: Kind::Document,
            input: format!("<frameset>{}", "x ".repeat(N)),
            required: None,
        },
        Case {
            name: "document of stray tags after the body",
            kind: Kind::Document,
            input: format!("<!doctype html></body></html>{}", "</p>".repeat(N)),
            required: None,
        },
    ]
}

#[test]
fn a_parse_with_more_errors_than_the_cap_completes_with_the_uncapped_tree() {
    for case in cases() {
        let Case {
            name,
            kind,
            input,
            required,
        } = case;
        let full = parse(kind, &input, with_cap(100_000)).unwrap_or_else(|failure| {
            panic!("{name}: uncapped parse failed {:?}", failure.reason())
        });
        let all = retained(&full);
        if let Some(required) = required {
            assert_eq!(all.len(), required, "{name}: derived error count");
        }
        let total = all.len();
        assert!(total > 8, "{name}: needs more errors than the caps below");
        let expected_tree = tree(&full);
        for cap in [0, 1, 2, 7, total / 2, total - 1, total, total + 1] {
            let report = parse(kind, &input, with_cap(cap as u64)).unwrap_or_else(|failure| {
                panic!(
                    "{name}: cap {cap} must retain a bounded prefix, not stop the parse: {:?}",
                    failure.reason()
                )
            });
            let kept = retained(&report);
            assert_eq!(kept.len(), total.min(cap), "{name}: cap {cap}");
            assert!(
                kept.iter().eq(all.iter().take(cap)),
                "{name}: cap {cap} retains the first diagnostics in parse order"
            );
            assert_eq!(tree(&report), expected_tree, "{name}: cap {cap}");
        }
    }
}

#[test]
fn dropped_diagnostics_come_from_the_tree_builder_as_well_as_the_tokenizer() {
    // Cap one: every NUL error after the first is dropped, and the NULs in
    // body content raise errors from both producers.
    let input = format!("<p>{}", "\0".repeat(16));
    let full = parse(Kind::Document, &input, with_cap(100_000)).unwrap();
    let all = retained(&full);
    assert!(
        all.iter()
            .skip(1)
            .any(|(kind, ..)| matches!(kind, HtmlParseDiagnosticKind::Tree(_))),
        "a dropped diagnostic is a tree-builder diagnostic"
    );
    assert!(
        all.iter()
            .skip(1)
            .any(|(kind, ..)| matches!(kind, HtmlParseDiagnosticKind::Tokenizer(_))),
        "a dropped diagnostic is a tokenizer diagnostic"
    );
    let capped = parse(Kind::Document, &input, with_cap(1)).unwrap();
    assert_eq!(retained(&capped).len(), 1);
    assert_eq!(tree(&capped), tree(&full));
}

#[test]
fn counting_dropped_diagnostics_is_charged_linearly_in_the_dropped_count() {
    // Cap zero drops every diagnostic, so the work is the parse plus the
    // counting. Doubling the input may at most double that, within slack.
    let mut previous: Option<u64> = None;
    for pairs in [64, 128, 256, 512] {
        let input = "< ".repeat(pairs);
        let report = parse(Kind::Document, &input, with_cap(0)).unwrap();
        assert_eq!(report.diagnostics().len(), 0);
        let work = report
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current();
        if let Some(previous) = previous {
            assert!(
                work * 10 <= previous * 22,
                "{pairs} pairs: work {work} grew more than 2.2x from {previous}"
            );
        }
        previous = Some(work);
    }
}

#[test]
fn another_resource_still_stops_a_capped_parse() {
    // The relaxation is the Diagnostics cap alone. A work budget below what
    // the capped parse needs is still terminal.
    let input = "< ".repeat(64);
    let capped = with_cap(0);
    let used = parse(Kind::Document, &input, capped)
        .unwrap()
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    let short = limits_builder()
        .diagnostics(0)
        .work_credits(used - 1)
        .finish()
        .unwrap();
    let failure = parse(Kind::Document, &input, short).unwrap_err();
    assert!(matches!(
        failure.first_stop(),
        Some(surgeist_html_parser::HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    let exact = limits_builder()
        .diagnostics(0)
        .work_credits(used)
        .finish()
        .unwrap();
    parse(Kind::Document, &input, exact).unwrap();
}
