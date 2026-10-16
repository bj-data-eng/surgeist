#![forbid(unsafe_code)]

use surgeist_html::{DocumentString, HtmlCustomElementState, HtmlNodeId, HtmlNodeKind};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseCompletion, HtmlParseLimits, HtmlParseMetadataStep,
    HtmlParseReport, HtmlSource,
};

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

fn completed_document(source: &str) -> HtmlParseReport {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
        .parse()
        .expect("the actual offline ordinary parse must complete");
    assert!(report.document().is_some());
    assert!(report.fragment().is_none());
    assert_eq!(
        report.document_state().unwrap().completion(),
        HtmlParseCompletion::Complete
    );
    report
}

fn element(report: &HtmlParseReport, local: &str) -> HtmlNodeId {
    let document = report.document().unwrap();
    let mut pending = vec![document.root().id()];
    let mut found = None;
    while let Some(subject) = pending.pop() {
        let node = document.node(&subject).unwrap();
        if matches!(node.kind(), HtmlNodeKind::Element(element)
            if element.name().namespace() == &surgeist_html::Namespace::html()
                && element.name().local_name() == &DocumentString::from(local))
        {
            assert!(found.replace(subject.clone()).is_none());
        }
        pending.extend(node.children());
    }
    found.expect("literal fixture contains the requested HTML element")
}

fn creation(
    report: &HtmlParseReport,
    subject: &HtmlNodeId,
) -> (HtmlCustomElementState, Option<DocumentString>) {
    let mut metadata = report.metadata();
    let mut found = None;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if record.node().id() == *subject
                    && let Some((state, saved_is)) = record.creation()
                {
                    assert!(found.replace((state, saved_is.cloned())).is_none());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    found.expect("completed ordinary element must lend its actual creation state and saved is")
}

#[test]
fn saved_is_preserves_creation_before_later_body_attribute_merge() {
    // DOM3071 create-element's no-definition branch saves the creation is
    // independently. HTML877d's repeated-body recovery merges attributes into
    // the existing body; it does not create that element again or change its is.
    let initial =
        completed_document("<!doctype html><html><head></head><body is=x-control></body></html>");
    let merged = completed_document(
        "<!doctype html><html><head></head><body><body is=x-control></body></html>",
    );
    let initial_body = element(&initial, "body");
    let merged_body = element(&merged, "body");
    for (report, subject) in [(&initial, &initial_body), (&merged, &merged_body)] {
        let node = report.document().unwrap().node(subject).unwrap();
        let HtmlNodeKind::Element(body) = node.kind() else {
            panic!("checked body element");
        };
        assert_eq!(
            body.attributes()
                .iter()
                .find(|attribute| {
                    attribute.name().namespace().uri().is_none()
                        && attribute.name().local_name() == &DocumentString::from("is")
                })
                .map(|attribute| attribute.value()),
            Some(&DocumentString::from("x-control"))
        );
    }
    assert_eq!(
        creation(&initial, &initial_body),
        (
            HtmlCustomElementState::Undefined,
            Some(DocumentString::from("x-control"))
        )
    );
    assert_eq!(
        creation(&merged, &merged_body),
        (HtmlCustomElementState::Uncustomized, None)
    );
}

#[test]
fn ordinary_null_registry_retains_custom_name_state_without_execution() {
    // DOM3071 create-element step6: with no definition, an HTML custom name has
    // Undefined state; an ordinary HTML name with no saved value is Uncustomized.
    // These literal expectations do not invoke the implementation's selector.
    let report = completed_document(
        "<!doctype html><html><head></head><body><x-card></x-card><section></section></body></html>",
    );
    let custom = element(&report, "x-card");
    let ordinary = element(&report, "section");
    assert_eq!(
        creation(&report, &custom),
        (HtmlCustomElementState::Undefined, None)
    );
    assert_eq!(
        creation(&report, &ordinary),
        (HtmlCustomElementState::Uncustomized, None)
    );
}
