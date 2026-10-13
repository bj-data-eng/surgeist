use surgeist_html::*;
fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(10000)
        .buffered_utf16_units(10000)
        .token_attributes(1000)
        .auxiliary_entries(10000)
        .managed_nodes(1000)
        .managed_edges(1000)
        .managed_attributes(1000)
        .managed_utf16_units(10000)
        .source_map_entries(10000)
        .source_map_payload_units(10000)
        .diagnostics(1000)
        .diagnostic_payload_units(1000)
        .context_effects(1000)
        .open_elements(100)
        .tree_depth(1000)
        .managed_capacity_bytes(10000000)
        .peak_transient_bytes(10000000)
        .atomic_layout_bytes(10000000)
        .work_credits(10000000)
        .finish()
        .unwrap()
}
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
    let mut documents = Vec::new();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(HtmlParseMetadataDocument::Owned(document)) = record.node_document() {
                    documents.push((record.node().id(), document.id()));
                }
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
        }
    }
    assert!(found);
    let current = documents
        .iter()
        .find(|(node, _)| Some(node) == link_node.as_ref())
        .unwrap()
        .1
        .clone();
    assert_ne!(birth.unwrap(), current);
}
