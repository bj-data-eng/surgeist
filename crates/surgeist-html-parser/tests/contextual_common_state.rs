#![forbid(unsafe_code)]
use surgeist_html::*;
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

fn fragment_context() -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

#[test]
fn completed_null_registry_fragment_lends_actual_closed_common_state() {
    // HTML877d element creation sets the common tuple; this original inert
    // parser executes no script or interaction that can transition that tuple.
    // SVG has the element tuple only; HTML has both. No native initializer is
    // a witness for either source-created subject.
    let report = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<p nonce=retained>x</p><svg><g></g></svg>"),
        fragment_context(),
        limits(),
        HtmlParserScriptingMode::Inert,
        false,
    )
    .unwrap()
    .parse()
    .unwrap();
    assert!(report.document_state().is_none());
    let mut found = [false; 2];
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if row.creation().is_none() {
                    continue;
                }
                let HtmlNodeKind::Element(element) = row.node().kind() else {
                    continue;
                };
                let local = element.name().local_name().code_units();
                let (index, scope) = if local == [112] {
                    (0, HtmlCommonScope::Html)
                } else if local == [103] {
                    (1, HtmlCommonScope::Element)
                } else {
                    continue;
                };
                let common = row
                    .common_state()
                    .expect("actual qualified contextual common tuple");
                assert_eq!(common.scope(), scope);
                assert!(common.element().reaction_queue_is_empty());
                assert!(!common.element().click_in_progress());
                assert_eq!(common.html().is_some(), index == 0);
                if let Some(html) = common.html() {
                    assert!(!html.has_previously_focused_element());
                    assert!(!html.has_attached_internals());
                    assert!(html.popover_is_hidden());
                    assert!(!html.has_popover_trigger());
                    assert!(!html.popover_is_hiding());
                    assert!(!html.has_popover_toggle_task_tracker());
                    assert!(!html.has_popover_close_watcher());
                }
                found[index] = true;
            }
            HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(found, [true, true]);
}
