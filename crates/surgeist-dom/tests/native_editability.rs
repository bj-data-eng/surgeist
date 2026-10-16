//! Canonical Document design-mode facts in HTML editability queries.
//! HTML877d html-editability lines121,151: fresh Document design mode is false;
//! this direct HTML child has no explicit contenteditable host state.
use std::sync::atomic::AtomicBool;

use surgeist_dom::{Dom, DomBatch, DomHtmlCaptureLimits, DomLimits, Namespace};
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

#[test]
fn native_fresh_document_design_mode_disables_implicit_editing_host() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(8, 4096, 16, 0, 0, 8, 65536).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(root).unwrap().parent(), Some(document.node()));
    assert!(
        snapshot
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(8, 64, 16, 4096, 8, 0, 0),
            DomHtmlCaptureLimits::new(8, 10000),
            &cancel,
        )
        .unwrap();
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(root).unwrap(),
                HtmlSemanticQuery::EditingHost,
            )],
            HtmlValueInterpreter::new(1024),
            HtmlSemanticLimits::new(100000, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    assert!(
        matches!(
            report.results()[0].fact(),
            HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
        ),
        "fresh native Document design mode is disabled; got {:?}",
        report.results()[0].fact()
    );
}
