use surgeist_html::{bounded::HtmlConstructionResult, *};
use surgeist_html_parser::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::*;

#[test]
fn parsed_document_enters_the_same_adapter_with_original_result_identity() {
    let limits = HtmlParseLimits::builder()
        .source_units(10000)
        .buffered_utf16_units(10000)
        .token_attributes(1000)
        .auxiliary_entries(10000)
        .managed_nodes(1000)
        .managed_edges(2000)
        .managed_attributes(1000)
        .managed_utf16_units(10000)
        .source_map_entries(10000)
        .source_map_payload_units(10000)
        .diagnostics(1000)
        .diagnostic_payload_units(10000)
        .context_effects(1000)
        .open_elements(100)
        .tree_depth(1000)
        .managed_capacity_bytes(10_000_000)
        .peak_transient_bytes(10_000_000)
        .atomic_layout_bytes(10_000_000)
        .work_credits(10_000_000)
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
    let mut pending = vec![id.clone()];
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
