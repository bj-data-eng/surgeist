use surgeist_html::{DocumentString, HtmlDocument, HtmlDocumentMode, HtmlNodeKind};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseFailure, HtmlParseFailureReason, HtmlParseLimits,
    HtmlParseResource, HtmlParseStop, HtmlSource,
};

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(1_000)
        .buffered_utf16_units(100_000)
        .token_attributes(1_000)
        .auxiliary_entries(100_000)
        .managed_nodes(1_000)
        .managed_edges(1_000)
        .managed_attributes(1_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(1_000)
        .diagnostic_payload_units(100_000)
        .context_effects(1_000)
        .open_elements(100)
        .tree_depth(100)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

fn source(literal: &str, utf16: bool) -> HtmlSource {
    if utf16 {
        HtmlSource::utf16(DocumentString::from(literal))
    } else {
        HtmlSource::utf8(literal)
    }
}

fn limit_failure(
    request: HtmlDocumentParseRequest,
    resource: HtmlParseResource,
    limit: u64,
    current: u64,
) -> HtmlParseFailure {
    let original = request.source().clone();
    let cancel = request.cancellation().clone();
    // Err is the public failure owner; it has no successful document/report.
    let failure = request.parse().unwrap_err();
    let expected = HtmlParseStop::LimitExceeded {
        resource,
        limit,
        current,
        requested_delta: 1,
    };
    assert_eq!(failure.reason(), HtmlParseFailureReason::Stop(expected));
    assert_eq!(failure.first_stop(), Some(expected));
    assert_eq!(failure.source().identity(), original.identity());
    assert_eq!(failure.source().coordinates(), original.coordinates());
    assert_eq!(failure.source().as_utf8(), original.as_utf8());
    assert_eq!(failure.source().as_utf16(), original.as_utf16());
    assert_eq!(failure.source_admitted(), Some(true));
    assert_eq!(failure.cleanup_error(), None);
    let at_stop = failure.at_stop().unwrap().clone();
    let after = failure.after_cleanup().unwrap().clone();
    assert_eq!(at_stop.resource(resource).current(), current);
    for reclaimed in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ContextEffects,
        HtmlParseResource::OpenElements,
    ] {
        assert_eq!(after.resource(reclaimed).current(), 0, "{reclaimed:?}");
    }
    // Source and committed diagnostic evidence may remain failure-owned. Work
    // and permanent-node admission are observations, not refundable storage.
    assert_eq!(
        at_stop.resource(HtmlParseResource::WorkCredits),
        after.resource(HtmlParseResource::WorkCredits)
    );
    cancel.cancel();
    assert_eq!(failure.first_stop(), Some(expected));
    assert_eq!(failure.at_stop(), Some(&at_stop));
    assert_eq!(failure.after_cleanup(), Some(&after));
    failure
}

fn empty_outline(document: &HtmlDocument) {
    assert_eq!(document.mode(), HtmlDocumentMode::Quirks);
    let root = document.root();
    assert!(matches!(root.kind(), HtmlNodeKind::Document(_)));
    let roots: Vec<_> = root.children().collect();
    assert_eq!(roots.len(), 1);
    let html = document.node(&roots[0]).unwrap();
    assert!(matches!(html.kind(), HtmlNodeKind::Element(e)
        if e.name().local_name() == &DocumentString::from("html")));
    let children: Vec<_> = html.children().collect();
    assert_eq!(children.len(), 2);
    for (id, name) in children.iter().zip(["head", "body"]) {
        let child = document.node(id).unwrap();
        assert!(matches!(child.kind(), HtmlNodeKind::Element(e)
            if e.name().local_name() == &DocumentString::from(name)));
        assert_eq!(child.children().count(), 0);
    }
}

#[test]
fn empty_source_implied_document_has_exact_four_node_admission() {
    // Required construction is Document, html, head and body, with no doctype
    // or Text node. All four nodes are admitted even though source length is 0.
    for utf16 in [false, true] {
        let exact = HtmlDocumentParseRequest::offline(
            source("", utf16),
            limits().into_builder().managed_nodes(4).finish().unwrap(),
        )
        .parse()
        .unwrap();
        empty_outline(exact.document().unwrap());
        assert_eq!(
            exact
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current(),
            4
        );
        assert_eq!(
            exact
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current(),
            0
        );
        limit_failure(
            HtmlDocumentParseRequest::offline(
                source("", utf16),
                limits().into_builder().managed_nodes(3).finish().unwrap(),
            ),
            HtmlParseResource::ManagedNodes,
            3,
            3,
        );
    }
}

#[test]
fn empty_source_implied_document_has_exact_three_edge_admission() {
    // Document→html and html→head/body are three actual ordinary edges.
    for utf16 in [false, true] {
        let exact = HtmlDocumentParseRequest::offline(
            source("", utf16),
            limits().into_builder().managed_edges(3).finish().unwrap(),
        )
        .parse()
        .unwrap();
        empty_outline(exact.document().unwrap());
        assert_eq!(
            exact
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current(),
            3
        );
        limit_failure(
            HtmlDocumentParseRequest::offline(
                source("", utf16),
                limits().into_builder().managed_edges(2).finish().unwrap(),
            ),
            HtmlParseResource::ManagedEdges,
            2,
            2,
        );
    }
}

#[test]
fn empty_source_implied_document_has_exact_two_parent_link_depth() {
    // The Document has depth0, html depth1, head/body depth2. TreeDepth counts
    // ancestry links; it does not count the root as an additional link.
    for utf16 in [false, true] {
        let exact = HtmlDocumentParseRequest::offline(
            source("", utf16),
            limits().into_builder().tree_depth(2).finish().unwrap(),
        )
        .parse()
        .unwrap();
        empty_outline(exact.document().unwrap());
        assert_eq!(
            exact
                .usage()
                .resource(HtmlParseResource::TreeDepth)
                .high_water(),
            2
        );
        limit_failure(
            HtmlDocumentParseRequest::offline(
                source("", utf16),
                limits().into_builder().tree_depth(1).finish().unwrap(),
            ),
            HtmlParseResource::TreeDepth,
            1,
            1,
        );
    }
}

#[test]
fn empty_source_zero_construction_caps_refuse_required_implicit_work() {
    for utf16 in [false, true] {
        for resource in [
            HtmlParseResource::ManagedNodes,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::TreeDepth,
        ] {
            let builder = limits().into_builder();
            let caps = match resource {
                HtmlParseResource::ManagedNodes => builder.managed_nodes(0),
                HtmlParseResource::ManagedEdges => builder.managed_edges(0),
                HtmlParseResource::TreeDepth => builder.tree_depth(0),
                _ => unreachable!(),
            }
            .finish()
            .unwrap();
            limit_failure(
                HtmlDocumentParseRequest::offline(source("", utf16), caps),
                resource,
                0,
                0,
            );
        }
    }
}

#[test]
fn ordinary_text_depth_exact_and_one_short_are_independent_of_open_elements() {
    // html/body/div are at most three open elements. Each Text child has four
    // parent links to Document; closing the div cannot reduce retained depth.
    let literal = "<!doctype html><div>X</div><div>Y</div>";
    for utf16 in [false, true] {
        let caps = limits()
            .into_builder()
            .open_elements(3)
            .tree_depth(4)
            .finish()
            .unwrap();
        let report = HtmlDocumentParseRequest::offline(source(literal, utf16), caps)
            .parse()
            .unwrap();
        let usage = report.usage();
        assert_eq!(usage.resource(HtmlParseResource::TreeDepth).high_water(), 4);
        assert_eq!(
            usage.resource(HtmlParseResource::OpenElements).high_water(),
            3
        );
        assert_eq!(usage.resource(HtmlParseResource::OpenElements).current(), 0);
        let document = report.document().unwrap();
        let html_id = document.root().children().nth(1).unwrap();
        let html = document.node(&html_id).unwrap();
        let body_id = html.children().nth(1).unwrap();
        let body = document.node(&body_id).unwrap();
        let divs: Vec<_> = body.children().collect();
        assert_eq!(divs.len(), 2);
        for (id, value) in divs.iter().zip(["X", "Y"]) {
            let div = document.node(id).unwrap();
            assert!(matches!(div.kind(), HtmlNodeKind::Element(e)
                if e.name().local_name() == &DocumentString::from("div")));
            assert_eq!(div.children().count(), 1);
            let text_id = div.children().next().unwrap();
            assert!(
                matches!(document.node(&text_id).unwrap().kind(), HtmlNodeKind::Text(text)
                if text == &DocumentString::from(value))
            );
        }
        let failed = limit_failure(
            HtmlDocumentParseRequest::offline(
                source(literal, utf16),
                limits()
                    .into_builder()
                    .open_elements(3)
                    .tree_depth(3)
                    .finish()
                    .unwrap(),
            ),
            HtmlParseResource::TreeDepth,
            3,
            3,
        );
        assert_eq!(
            failed
                .at_stop()
                .unwrap()
                .resource(HtmlParseResource::OpenElements)
                .high_water(),
            3
        );
    }
}
