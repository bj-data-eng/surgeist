//! What a parse retains and produces for each kind of optional detail.
//!
//! The expectations come from the selection contract: a parse that selects no
//! detail allocates no storage for it, and a parse that selects a kind records
//! that kind. Resource figures are counted from the report's usage, which holds
//! the live value of each resource at publication.

use crate::{
    DetailSet, HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlParseResource,
    HtmlSource,
};

/// Limits that the documents below do not reach.
fn limits() -> HtmlParseLimits {
    crate::parser::tree_builder::tests::limits(10_000)
}

/// A document with every record a kind of detail holds: parse errors, markup of
/// every lexical shape, text the tree builder drops, a `meta` default-style
/// name, a `style` element and a `script` element.
const DOCUMENT: &str = "<p class=a title=\"t\">one<b>two</b></p>\n< \
<!-- c --><table><tr><td>x</table>\r\n\
<style title=A media=screen>a{}</style><meta http-equiv=default-style content=B>\
<script>1</script><img onerror=e>";

fn parse(selected: DetailSet) -> HtmlParseReport {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits())
        .with_detail(selected)
        .expect("the detail feature is enabled in unit tests")
        .parse()
        .expect("the document parses inside its limits")
}

fn live(report: &HtmlParseReport, resource: HtmlParseResource) -> u64 {
    report.usage().resource(resource).current()
}

const DETAIL_RESOURCES: [HtmlParseResource; 4] = [
    HtmlParseResource::SourceMapEntries,
    HtmlParseResource::SourceMapPayloadUnits,
    HtmlParseResource::Diagnostics,
    HtmlParseResource::DiagnosticPayloadUnits,
];

#[test]
fn a_parse_that_selects_no_detail_retains_none_of_it() {
    let report = parse(DetailSet::NONE);
    for resource in DETAIL_RESOURCES {
        let usage = report.usage().resource(resource);
        assert_eq!(
            (usage.current(), usage.high_water()),
            (0, 0),
            "{resource:?} is neither retained nor reached"
        );
    }
    assert!(report.detail().is_none());
}

#[test]
fn a_parse_that_selects_a_kind_retains_what_the_kind_records() {
    let all = parse(DetailSet::ALL);
    assert!(live(&all, HtmlParseResource::SourceMapEntries) > 0);
    assert!(live(&all, HtmlParseResource::Diagnostics) > 0);
    let none = parse(DetailSet::NONE);
    assert!(all.retained_construction_bytes() > none.retained_construction_bytes());
    for kind in [
        DetailSet::SOURCE_MAP,
        DetailSet::LEXICAL,
        DetailSet::HISTORY,
    ] {
        let report = parse(kind);
        assert!(
            live(&report, HtmlParseResource::SourceMapEntries) > 0,
            "{kind:?} records entries"
        );
        assert_eq!(live(&report, HtmlParseResource::Diagnostics), 0, "{kind:?}");
        assert!(
            report.retained_construction_bytes() > none.retained_construction_bytes(),
            "{kind:?}"
        );
    }
    let diagnostics = parse(DetailSet::DIAGNOSTICS);
    assert!(live(&diagnostics, HtmlParseResource::Diagnostics) > 0);
    assert!(
        diagnostics.retained_construction_bytes() > none.retained_construction_bytes(),
        "diagnostics retain their records"
    );
}

#[test]
fn each_kind_produces_exactly_its_own_records() {
    let source_map = parse(DetailSet::SOURCE_MAP);
    let detail = source_map.detail().unwrap();
    let rows = detail.associations().unwrap();
    assert!(!rows.is_empty());
    assert!(detail.lexical().is_none());
    assert!(detail.omissions().is_none());
    assert!(detail.history().is_none());
    assert!(detail.diagnostics().is_none());

    let lexical = parse(DetailSet::LEXICAL);
    let detail = lexical.detail().unwrap();
    assert!(!detail.lexical().unwrap().is_empty());
    assert!(detail.omissions().is_some());
    assert!(detail.history().is_none());
    assert!(detail.diagnostics().is_none());

    let history = parse(DetailSet::HISTORY);
    let detail = history.detail().unwrap();
    assert!(!detail.history().unwrap().is_empty());
    assert!(detail.lexical().is_none());
    assert!(detail.diagnostics().is_none());

    let diagnostics = parse(DetailSet::DIAGNOSTICS);
    let detail = diagnostics.detail().unwrap();
    assert!(!detail.diagnostics().unwrap().is_empty());
    assert!(detail.associations().is_none());
    assert!(detail.history().is_none());
    assert!(detail.lexical().is_none());
    assert_eq!(detail.selected(), DetailSet::DIAGNOSTICS);
}

#[test]
fn a_selection_without_a_graph_kind_has_no_causes_to_read() {
    let report = parse(DetailSet::DIAGNOSTICS);
    let mut cursor = report.pending_associations();
    let mut occurrences = 0;
    loop {
        match cursor.advance() {
            crate::HtmlParsePendingAssociationStep::Occurrence(row) => {
                occurrences += 1;
                assert!(matches!(
                    report.detail().unwrap().cause_of(&row),
                    Err(crate::HtmlParseCauseUnselected)
                ));
            }
            crate::HtmlParsePendingAssociationStep::Complete => break,
            _ => {}
        }
    }
    assert!(occurrences > 0);
    let report = parse(DetailSet::HISTORY);
    let mut cursor = report.pending_associations();
    loop {
        match cursor.advance() {
            crate::HtmlParsePendingAssociationStep::Occurrence(row) => {
                assert!(report.detail().unwrap().cause_of(&row).unwrap().is_some());
            }
            crate::HtmlParsePendingAssociationStep::Complete => break,
            _ => {}
        }
    }
}
