use crate::parser::support::small_limits_builder;
use surgeist_html::{construction::HtmlConstructionResult, construction::HtmlNodeKind};
use surgeist_html_parser::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::*;

#[test]
fn parsed_document_enters_the_same_adapter_with_original_result_identity() {
    let limits = small_limits_builder()
        .managed_edges(2000)
        .diagnostic_payload_units(10000)
        .finish()
        .unwrap();
    let report =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><p>x&amp;y"), limits)
            .parse()
            .unwrap();
    let document = report.document().unwrap().clone();
    let id = document.root().id();
    let source = HtmlConstructionResult::Document(document);
    let revision = HtmlHostFactsRevision::fresh();
    let builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(200, 2000, 200, 100_000, 200, 10, 10, 16_777_216),
        revision.clone(),
    );
    let mut pending = vec![id];
    let mut rows = Vec::new();
    while let Some(subject) = pending.pop() {
        let node = source.node(&subject).unwrap();
        pending.extend(node.children());
        rows.push(HtmlConstructionCaptureNodeFacts::new(
            subject,
            HtmlConstructionCaptureDocument::ResultDocument,
            matches!(node.kind(), HtmlNodeKind::Document(_)).then(|| {
                HtmlConstructionCaptureDocumentFacts::new(
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Unavailable,
                )
            }),
            matches!(node.kind(), HtmlNodeKind::Element(_)).then(|| {
                HtmlConstructionCaptureElementFacts::new(
                    HtmlCapturedIsValue::Unavailable,
                    HtmlCaptureRegistryInput::Unavailable,
                )
            }),
            None,
        ));
    }
    let facts = HtmlConstructionCaptureFacts::new(revision, rows);
    let HtmlConstructionResult::Document(document) = source else {
        panic!()
    };
    let result = HtmlConstructionCaptureAdapter::document(
        builder,
        document,
        facts,
        HtmlConstructionCaptureLimits::new(200, 20_000, 1_000_000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    assert_eq!(result.mapping().root(), &id);
    assert_eq!(
        result.source().document().unwrap().root().id(),
        report.document().unwrap().root().id()
    );
    let captured = result
        .capture()
        .node(&result.mapping().subject(&id, result.capture()).unwrap())
        .unwrap();
    let HtmlCapturedChildren::Complete(children) = captured.children() else {
        panic!()
    };
    assert!(children.count() >= 2);
}
