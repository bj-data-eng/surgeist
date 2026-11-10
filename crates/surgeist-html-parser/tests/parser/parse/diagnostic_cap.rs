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

#[allow(unused_imports)]
use crate::parser::support::{ReportDetail as _, SubjectCause as _};
use crate::parser::support::{fragment_context, limits_builder};
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeId, HtmlNodeKind},
    parsing::HtmlParserScriptingMode,
};
use surgeist_html_parser::{
    HtmlFragmentParseRequest, HtmlParseDiagnosticKind, HtmlParseLimits, HtmlParseMetadataStep,
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
                for attribute in element.attributes().as_slice() {
                    out.push_str(&format!("{pad}  @{attribute:?}\n"));
                }
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
) -> Result<HtmlParseReport, Box<surgeist_html_parser::HtmlParseFailure>> {
    let source = HtmlSource::utf8(input);
    match kind {
        Kind::Document => crate::parser::support::offline_all(source, limits).parse(),
        Kind::Fragment => HtmlFragmentParseRequest::with_options(
            source,
            fragment_context(),
            limits,
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
        .with_detail(surgeist_html_parser::DetailSet::ALL)
        .unwrap()
        .parse(),
    }
    .map_err(Box::new)
}

fn with_cap(cap: u64) -> HtmlParseLimits {
    limits_builder().diagnostics(cap).finish().unwrap()
}

/// What the report exposes of one retained diagnostic.
type Retained = (
    HtmlParseDiagnosticKind,
    &'static str,
    &'static str,
    (usize, usize),
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
                (
                    location.source_range().start(),
                    location.source_range().end(),
                ),
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
        assert_eq!(full.diagnostics().dropped(), 0, "{name}: nothing dropped");
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
            assert_eq!(
                report.diagnostics().dropped(),
                (total - kept.len()) as u64,
                "{name}: cap {cap} counts exactly the diagnostics it did not retain"
            );
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

#[test]
fn a_failure_keeps_the_prefix_and_the_dropped_count_reached_before_the_stop() {
    let input = "< ".repeat(64);
    let complete = parse(Kind::Document, &input, with_cap(3)).unwrap();
    let used = complete
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    // A quarter of the credits ends the parse inside tokenization however the
    // cost of construction and publication is split between the two.
    let cut = limits_builder()
        .diagnostics(3)
        .work_credits(used / 4)
        .finish()
        .unwrap();
    let failure = parse(Kind::Document, &input, cut).unwrap_err();
    assert_eq!(failure.cleanup_error(), None);
    let after = failure.after_cleanup().expect("cleanup evidence");
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::OpenElements,
    ] {
        assert_eq!(after.resource(resource).current(), 0, "{resource:?}");
    }
    // Only the three committed diagnostics stay owned; the dropped ones never
    // held storage.
    assert_eq!(after.resource(HtmlParseResource::Diagnostics).current(), 3);
    assert_eq!(
        after
            .resource(HtmlParseResource::SourceMapEntries)
            .current(),
        3
    );
    let diagnostics = failure.diagnostics().expect("diagnostics were initialized");
    assert_eq!(diagnostics.len(), 3);
    assert!(
        diagnostics.dropped() > 0,
        "errors past the cap were counted"
    );
    assert!(
        diagnostics.dropped() < complete.diagnostics().dropped(),
        "fewer errors were reached than in the complete parse"
    );
}

/// Everything a report publishes beyond the tree and the diagnostics: the
/// attributes of every element, every association row (the source map of each
/// text, attribute and node to the source range and origin it came from) and
/// the control metadata with its form owners.
fn published(report: &HtmlParseReport) -> String {
    let mut out = tree(report);
    let rows = report.associations();
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap().unwrap();
        out.push_str(&format!(
            "row {index}: source {:?} origin {:?} {}..{} {:?}\n",
            row.source().map(|range| (range.start(), range.end())),
            row.origin_kind(),
            row.start(),
            row.end(),
            row.relation()
        ));
    }
    let name = |node: &HtmlNode<'_>| match node.kind() {
        HtmlNodeKind::Element(element) => {
            String::from_utf16_lossy(element.name().local_name().code_units())
        }
        other => format!("{other:?}"),
    };
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => out.push_str(&format!(
                "record {} owner {:?} input {:?} textarea {:?}\n",
                name(&record.node()),
                record
                    .form_owner()
                    .map(|owner| owner.map(|owner| name(&owner))),
                record.input_raw_value(),
                record.textarea_raw_value()
            )),
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    out
}

#[test]
fn a_capped_parse_publishes_the_attributes_associations_and_metadata_of_an_uncapped_one() {
    // Parse errors: the missing doctype, six `< ` pairs, the unknown named
    // reference and the null reference, so every cap below drops some.
    let input = "<form id=f><input type=hidden value=x name=n><label for=n>L &amp; M</label>\
<textarea>A&amp;B</textarea><select><option selected>Q</option></select></form>\
< < < < < < &bogus; &#0;<p id=z class=\"a b\">t &amp; u</p>";
    let full = parse(Kind::Document, input, with_cap(100_000)).unwrap();
    let total = full.diagnostics().len();
    assert!(total >= 9, "{total} parse errors");
    let expected = published(&full);
    assert!(expected.contains("row "), "associations are published");
    assert!(
        expected.contains("record input owner Some(Some(\"form\"))"),
        "{expected}"
    );
    for cap in [0, 1, 4, total as u64 - 1] {
        let capped = parse(Kind::Document, input, with_cap(cap)).unwrap();
        assert_eq!(capped.diagnostics().len(), cap as usize);
        assert_eq!(capped.diagnostics().dropped(), (total as u64) - cap);
        assert_eq!(published(&capped), expected, "cap {cap}");
    }
}
