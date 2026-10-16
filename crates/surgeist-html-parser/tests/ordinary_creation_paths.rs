#![forbid(unsafe_code)]

use surgeist_html::{DocumentString, HtmlCustomElementState, HtmlNodeId, HtmlNodeKind};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseConstructionOperation, HtmlParseDestination,
    HtmlParseFailureReason, HtmlParseLimits, HtmlParseMetadataStep, HtmlParseReport,
    HtmlParseResource, HtmlParseStop, HtmlSource,
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

fn creations(
    report: &HtmlParseReport,
    local: &str,
) -> Vec<(HtmlNodeId, HtmlCustomElementState, Option<DocumentString>)> {
    let mut cursor = report.metadata();
    let mut rows = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(record) => {
                let node = record.node();
                if matches!(node.kind(), HtmlNodeKind::Element(element)
                    if element.name().local_name() == &DocumentString::from(local))
                    && let Some((state, saved)) = record.creation()
                {
                    rows.push((node.id(), state, saved.cloned()));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    rows
}

#[test]
fn explicit_and_implied_roots_keep_original_is_before_later_html_merge() {
    // BeforeHtml creates the root from its token or as an implied HTML element.
    // A later HTML token merges attributes; it cannot change saved creation is.
    for (source, expected_state, expected_is) in [
        (
            "<!doctype html><html is='A&amp;B'><body><html is=later>",
            HtmlCustomElementState::Undefined,
            Some(DocumentString::from("A&B")),
        ),
        (
            "<!doctype html><body><html is=later>",
            HtmlCustomElementState::Uncustomized,
            None,
        ),
    ] {
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .parse()
            .unwrap();
        let rows = creations(&report, "html");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].1, rows[0].2.clone()),
            (expected_state, expected_is)
        );
    }
}

#[test]
fn empty_saved_is_remains_present_and_reserved_names_are_not_custom_names() {
    // DOM3071 retains a present-empty is independently from absent is. Its
    // no-definition state uses the actual custom-name grammar, not a hyphen test.
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><p is=''></p><x-card is=''></x-card><font-face></font-face>",
        ),
        limits(),
    )
    .parse()
    .unwrap();
    for local in ["p", "x-card"] {
        let rows = creations(&report, local);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, HtmlCustomElementState::Undefined);
        assert_eq!(rows[0].2, Some(DocumentString::from("")));
    }
    let rows = creations(&report, "font-face");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, HtmlCustomElementState::Uncustomized);
    assert_eq!(rows[0].2, None);
}

#[test]
fn formatting_reconstruction_keeps_each_original_creation_value_and_identity() {
    // HTML877d reconstructs the original i token after b closes. Both actual
    // elements keep the token's decoded saved is, including surrounding spaces.
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><p>1<b>2<i is=' A&amp;B '>3</b>4</i>5</p>"),
        limits(),
    )
    .parse()
    .unwrap();
    let rows = creations(&report, "i");
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].0, rows[1].0);
    for (_, state, saved) in &rows {
        assert_eq!(*state, HtmlCustomElementState::Undefined);
        assert_eq!(saved.as_ref(), Some(&DocumentString::from(" A&B ")));
    }
    let associations = report.associations();
    let mut replay = false;
    for index in 0..associations.len() {
        let row = associations.row(index).unwrap();
        if let HtmlParseDestination::Node(node) = row.destination()
            && rows.iter().any(|(id, _, _)| *id == node.id())
            && row.cause().operation() == HtmlParseConstructionOperation::Replay
        {
            let original = row.cause().original_cause().unwrap();
            assert_eq!(original.operation(), HtmlParseConstructionOperation::Token);
            assert_eq!(
                report.source().text(original.source().unwrap()).unwrap(),
                DocumentString::from("<i is=' A&amp;B '>")
            );
            replay = true;
        }
    }
    assert!(replay, "reconstruction retains its original lexical cause");
}

#[test]
fn selectedcontent_clones_retain_saved_creation_state_on_distinct_result_subjects() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><i is='A&amp;B'>X</i></option></select>",
        ),
        limits(),
    )
    .parse()
    .unwrap();
    let rows = creations(&report, "i");
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].0, rows[1].0);
    let document = report.document().unwrap();
    let subjects = rows.iter().map(|(id, _, _)| id.clone()).collect::<Vec<_>>();
    let mut parents = Vec::new();
    for (subject, state, saved) in rows {
        assert_eq!(state, HtmlCustomElementState::Undefined);
        assert_eq!(saved, Some(DocumentString::from("A&B")));
        let parent = document.node(&subject).unwrap().parent().unwrap();
        let HtmlNodeKind::Element(element) = document.node(&parent).unwrap().kind() else {
            panic!("literal parent element");
        };
        parents.push(element.name().local_name().to_utf8().unwrap());
    }
    parents.sort();
    assert_eq!(parents, ["option", "selectedcontent"]);
    let associations = report.associations();
    let mut copy = false;
    for index in 0..associations.len() {
        let row = associations.row(index).unwrap();
        if let HtmlParseDestination::Node(node) = row.destination()
            && subjects.contains(&node.id())
            && row.cause().operation() == HtmlParseConstructionOperation::Copy
        {
            let original = row.cause().original_node().unwrap();
            assert!(subjects.contains(&original.id()));
            assert_ne!(node.id(), original.id());
            copy = true;
        }
    }
    assert!(copy, "clone retains its actual original result subject");
}

#[test]
fn utf16_saved_is_survives_reconstruction_and_later_cancellation_of_its_request() {
    let mut units = "<!doctype html><p><i is='"
        .encode_utf16()
        .collect::<Vec<_>>();
    units.extend([0xd800, 0xd83d, 0xde00, 0xdc00]);
    units.extend("'>X</p>Y".encode_utf16());
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(),
    );
    let cancellation = request.cancellation().clone();
    let report = request.parse().unwrap();
    cancellation.cancel();
    let rows = creations(&report, "i");
    assert_eq!(rows.len(), 2);
    for (_, state, saved) in rows {
        assert_eq!(state, HtmlCustomElementState::Undefined);
        assert_eq!(
            saved.unwrap().code_units(),
            &[0xd800, 0xd83d, 0xde00, 0xdc00]
        );
    }
}

#[test]
fn creation_retention_has_exact_storage_and_work_limits_with_terminal_cleanup() {
    let source = "<!doctype html><p><i is='A&amp;B'>X</p>Y";
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
        .parse()
        .unwrap();
    for resource in [
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::WorkCredits,
        HtmlParseResource::SourceMapEntries,
    ] {
        // Measured usage chooses the finite cut, not expected product values.
        // The oracle is typed refusal, original source ownership and reclamation.
        let exact = report.usage().resource(resource).high_water();
        for allowance in [exact, exact - 1] {
            let builder = limits().into_builder();
            let constrained = match resource {
                HtmlParseResource::ManagedUtf16Units => builder.managed_utf16_units(allowance),
                HtmlParseResource::AuxiliaryEntries => builder.auxiliary_entries(allowance),
                HtmlParseResource::WorkCredits => builder.work_credits(allowance),
                HtmlParseResource::SourceMapEntries => builder.source_map_entries(allowance),
                _ => unreachable!("selected resource"),
            }
            .finish()
            .unwrap();
            let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), constrained);
            let identity = request.source().identity().clone();
            let result = request.parse();
            if allowance == exact {
                let complete = result.unwrap();
                assert_eq!(creations(&complete, "i").len(), 2);
            } else {
                let failure = result.unwrap_err();
                assert!(matches!(
                    failure.reason(),
                    HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
                        resource: actual,
                        ..
                    }) if actual == resource
                ));
                assert_eq!(failure.source().identity(), &identity);
                assert_eq!(failure.cleanup_error(), None);
                for released in [
                    HtmlParseResource::ManagedUtf16Units,
                    HtmlParseResource::ManagedEdges,
                    HtmlParseResource::ManagedAttributes,
                    HtmlParseResource::TokenAttributes,
                    HtmlParseResource::OpenElements,
                ] {
                    assert_eq!(
                        failure
                            .after_cleanup()
                            .unwrap()
                            .resource(released)
                            .current(),
                        0
                    );
                }
            }
        }
    }
}
