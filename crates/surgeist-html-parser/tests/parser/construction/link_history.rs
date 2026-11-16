use crate::parser::support::small_limits as limits;
use surgeist_html::{construction::HtmlNodeKind, values::DocumentString};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseMetadataDocument, HtmlParseMetadataStep, HtmlSource,
};
#[test]
fn actual_completed_parser_publishes_fresh_flags_and_creating_document_identity() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><link rel=stylesheet disabled><link rel=icon>"),
        limits(),
    )
    .parse()
    .unwrap();
    let mut metadata = report.metadata();
    let mut links = 0;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(history) = record.link_history() {
                    links += 1;
                    assert!(!history.flags().explicitly_enabled());
                    assert!(history.created_by_node_document_parser());
                    assert_eq!(record.link_flags(), Some(history.flags()));
                    let Some(Some(HtmlParseMetadataDocument::Owned(document))) =
                        record.link_parser_document()
                    else {
                        panic!("actual parser document")
                    };
                    assert!(matches!(document.kind(), HtmlNodeKind::Document(_)));
                    let HtmlNodeKind::Element(element) = record.node().kind() else {
                        panic!("actual link")
                    };
                    assert_eq!(element.name().local_name(), &DocumentString::from("link"));
                    assert_eq!(element.link_flags(), record.link_flags());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert_eq!(links, 2);
}

#[test]
fn template_link_birth_document_differs_from_its_actual_inert_node_document() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><template><link rel=stylesheet></template>"),
        limits(),
    )
    .parse()
    .unwrap();
    let mut metadata = report.metadata();
    let mut found = false;
    let mut birth = None;
    let mut link_node = None;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(history) = record.link_history() {
                    found = true;
                    assert!(!history.created_by_node_document_parser());
                    let Some(Some(HtmlParseMetadataDocument::Owned(document))) =
                        record.link_parser_document()
                    else {
                        panic!("actual creating document");
                    };
                    birth = Some(document.id());
                    link_node = Some(record.node().id());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert!(found);
    // The document the link is in is the document of its tree, which the report answers by the
    // root of the tree.
    let tree = report.document().unwrap();
    let mut root = link_node.unwrap();
    while let Some(parent) = tree.node(&root).unwrap().parent() {
        root = parent;
    }
    let Some(HtmlParseMetadataDocument::Owned(current)) =
        report.tree_document(&root).unwrap().document()
    else {
        panic!("the link is in a tree of an owned Document");
    };
    let current = current.id();
    assert_ne!(birth.unwrap(), current);
}

/// The record visits the publication phase of a parse of `links` links, each inside `depth`
/// nested `div` elements, makes.
fn publication_visits(depth: usize, links: usize) -> u64 {
    use surgeist_html::accounting::{HtmlAuxiliaryScope, HtmlWork};
    use surgeist_html_parser::HtmlParseAccounting;
    use surgeist_ledger::ProfileMode;
    let source = format!(
        "<!doctype html><body>{}{}",
        "<div>".repeat(depth),
        "<link rel=x href=y>".repeat(links)
    );
    let report = crate::parser::support::offline_all(
        HtmlSource::utf8(source),
        crate::parser::support::limits(),
    )
    .with_accounting(
        HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false }).observing_phases(),
    )
    .parse()
    .unwrap()
    .profile()
    .unwrap()
    .expect("a summary was selected");
    report
        .scopes
        .iter()
        .find(|scope| scope.name == HtmlAuxiliaryScope::ParsePublication.name())
        .unwrap()
        .usage
        .work(HtmlWork::RecordVisit)
}

/// A link's document is read from the tree it is in, so publishing a link must cost the same at
/// whatever depth it is nested: the visits that `links` links add to a document of `depth` nested
/// elements do not depend on `depth`.
#[test]
fn preparing_links_costs_the_same_at_every_depth() {
    let links = 16;
    let added = |depth: usize| publication_visits(depth, links) - publication_visits(depth, 0);
    let shallow = added(8);
    assert!(
        shallow >= links as u64,
        "{links} links visited {shallow} times"
    );
    assert_eq!(added(16), shallow, "depth 16");
    assert_eq!(added(32), shallow, "depth 32");
}

/// The visits `links` links add at a fixed depth grow in proportion to the links, to within the
/// search of the table that holds their documents: at most 2.2 times per doubling.
#[test]
fn preparing_links_grows_linearly_in_their_number() {
    let added = |links: usize| publication_visits(16, links) - publication_visits(16, 0);
    let mut before = added(8);
    for links in [16, 32, 64] {
        let now = added(links);
        assert!(
            now * 10 <= before * 22,
            "{links} links added {now}, from {before}"
        );
        before = now;
    }
}
