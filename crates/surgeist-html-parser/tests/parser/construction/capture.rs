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

/// The block that the attribute values of `report` borrow and the capture's retained UTF-16
/// extent, for a parse of one `div` with `attributes` attributes of `units` letters each.
fn captured_attribute_extent(
    attributes: usize,
    units: usize,
) -> (Option<surgeist_html::values::TextBlock>, usize) {
    let mut source = String::from("<!doctype html><html><body><div");
    for index in 0..attributes {
        source.push_str(&format!(" data-a{index}=\"{}\"", "v".repeat(units)));
    }
    source.push_str("></div></body></html>");
    let limits = small_limits_builder()
        .source_units(1 << 20)
        .buffered_utf16_units(1 << 20)
        .managed_utf16_units(1 << 20)
        .managed_attributes(1000)
        .managed_edges(2000)
        .diagnostic_payload_units(10000)
        .finish()
        .unwrap();
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits)
        .parse()
        .unwrap();
    let document = report.document().unwrap().clone();
    let id = document.root().id();
    let source = HtmlConstructionResult::Document(document);
    let mut block = None;
    let revision = HtmlHostFactsRevision::fresh();
    let builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(200, 2000, 2000, 100_000_000, 200, 10, 10, 1 << 30),
        revision.clone(),
    );
    let mut pending = vec![id];
    let mut rows = Vec::new();
    while let Some(subject) = pending.pop() {
        let node = source.node(&subject).unwrap();
        pending.extend(node.children());
        if let HtmlNodeKind::Element(element) = node.kind()
            && let Some(attribute) = element.attributes().iter().next()
        {
            block = attribute.value().borrowed_block().cloned();
        }
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
        HtmlConstructionCaptureLimits::new(200, 20_000, 10_000_000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .finish()
    .unwrap();
    let extent = result
        .capture()
        .retained_storage_usage_with(|| Ok::<(), std::convert::Infallible>(()))
        .unwrap()
        .utf16_extent();
    (block, extent)
}

#[test]
fn the_capture_of_a_parse_counts_its_attribute_values_by_length_and_their_block_once() {
    let (small_block, small) = captured_attribute_extent(8, 4);
    let (large_block, large) = captured_attribute_extent(8, 400);
    let (small_block, large_block) = (
        small_block.expect("the attribute values of the parse borrow its block"),
        large_block.expect("the attribute values of the parse borrow its block"),
    );
    // The documents differ only in the length of the eight values, so the extents of their
    // captures differ by those lengths and by the capacity of the block each parse made.
    assert_eq!(
        large - small,
        8 * (400 - 4) + large_block.capacity() - small_block.capacity()
    );
}
