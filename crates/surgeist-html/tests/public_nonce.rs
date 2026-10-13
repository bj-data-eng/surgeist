use surgeist_html::*;
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

fn slots(report: &HtmlParseReport) -> Vec<(String, DocumentString)> {
    let mut cursor = report.metadata();
    let mut values = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(value) = record.cryptographic_nonce() {
                    let node = record.node();
                    let HtmlNodeKind::Element(element) = node.kind() else {
                        panic!("nonce element")
                    };
                    values.push((
                        element.name().local_name().to_utf8().unwrap(),
                        value.clone(),
                    ));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    values
}
#[test]
fn standalone_nonce_slots_preserve_content_reference_and_absence_in_both_encodings() {
    let literal = "<!doctype html><p nonce=' A&amp;B '></p><x-host></x-host><svg nonce=S><g nonce=G></g></svg><math nonce=M></math>";
    for source in [HtmlSource::utf8(literal), HtmlSource::utf16(literal.into())] {
        let identity = source.identity().clone();
        let report = HtmlDocumentParseRequest::offline(source, limits())
            .parse()
            .unwrap();
        assert_eq!(report.source().identity(), &identity);
        assert_eq!(
            slots(&report),
            vec![
                ("html".into(), "".into()),
                ("head".into(), "".into()),
                ("body".into(), "".into()),
                ("p".into(), " A&B ".into()),
                ("x-host".into(), "".into()),
                ("svg".into(), "S".into()),
                ("g".into(), "G".into()),
                ("math".into(), "M".into())
            ]
        );
        let document = report.document().unwrap();
        let html = document
            .node(&document.root().children().nth(1).unwrap())
            .unwrap();
        let body = document.node(&html.children().nth(1).unwrap()).unwrap();
        let p = document.node(&body.children().next().unwrap()).unwrap();
        let HtmlNodeKind::Element(element) = p.kind() else {
            panic!("p")
        };
        assert_eq!(
            element.attributes().as_slice()[0].value(),
            &DocumentString::from(" A&B ")
        );
        let associations = report.associations();
        let mut found = false;
        for index in 0..associations.len() {
            if matches!(associations.row(index).unwrap().destination(), HtmlParseDestination::Raw { node, kind: HtmlParseRawField::Nonce } if node.id() == p.id())
            {
                found = true;
            }
        }
        assert!(found, "nonce raw is an actual source-qualified field");
    }
}
#[test]
fn later_html_body_merge_installs_nonce_without_recopying_ignored_existing_field() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><html nonce=A><body><html nonce=B><body nonce=C><p nonce=''>",
        ),
        limits(),
    )
    .parse()
    .unwrap();
    assert_eq!(
        slots(&report),
        vec![
            ("html".into(), "A".into()),
            ("head".into(), "".into()),
            ("body".into(), "C".into()),
            ("p".into(), "".into())
        ]
    );
}

fn connected_shadow_context(policy: HtmlHeaderDeliveredCsp) -> HtmlFragmentContext {
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
    builder
        .establish_document_nonce_environment(
            &document,
            HtmlNonceDocumentEnvironment::new(HtmlBrowsingContextPresence::Present, policy),
        )
        .unwrap();
    let host = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_children(&document, std::slice::from_ref(&host))
        .unwrap();
    builder.establish_no_shadow_root(&host).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&host).unwrap()).unwrap()
}
#[test]
fn captured_connected_shadow_header_policy_hides_content_preserving_real_slot() {
    for policy in [
        HtmlHeaderDeliveredCsp::Absent,
        HtmlHeaderDeliveredCsp::Present,
    ] {
        let request = HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8("<template shadowrootmode=open><p nonce=A></p></template>"),
            connected_shadow_context(policy),
            limits(),
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap();
        let report = request.parse().unwrap();
        let values = slots(&report);
        assert!(
            values
                .iter()
                .any(|(name, value)| name == "p" && value == &DocumentString::from("A"))
        );
        let fragment = report.fragment().unwrap();
        let mut cursor = report.metadata();
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(record)
                    if record.cryptographic_nonce() == Some(&DocumentString::from("A")) =>
                {
                    let id = record.node().id();
                    let node = fragment.node(&id).unwrap();
                    let HtmlNodeKind::Element(element) = node.kind() else {
                        panic!("nonce element")
                    };
                    assert_eq!(
                        element.attributes().as_slice()[0].value(),
                        &DocumentString::from(if policy == HtmlHeaderDeliveredCsp::Present {
                            ""
                        } else {
                            "A"
                        })
                    );
                    break;
                }
                HtmlParseMetadataStep::Complete => panic!("p nonce record"),
                _ => {}
            }
        }
    }
}
#[test]
fn connected_script_hiding_runs_with_actual_pending_connection_effects() {
    let request = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<template shadowrootmode=open><script nonce=A></script></template>"),
        connected_shadow_context(HtmlHeaderDeliveredCsp::Present),
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let report = request.parse().unwrap();
    assert!(slots(&report).contains(&("script".into(), "A".into())));
    let mut found = false;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(record)
                if record.cryptographic_nonce() == Some(&DocumentString::from("A")) =>
            {
                let HtmlNodeKind::Element(element) = record.node().kind() else {
                    panic!("script");
                };
                assert_eq!(element.name().local_name(), &DocumentString::from("script"));
                assert_eq!(
                    element
                        .attributes()
                        .get(&Namespace::none(), &DocumentString::from("nonce"))
                        .unwrap()
                        .value()
                        .code_units(),
                    &[]
                );
                found = true;
            }
            HtmlParseMetadataStep::Complete => break,
            _ => {}
        }
    }
    assert!(found);
}
#[test]
fn captured_connected_unknown_header_is_typed_failure_not_false() {
    let request = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<template shadowrootmode=open><p nonce=A></p></template>"),
        connected_shadow_context(HtmlHeaderDeliveredCsp::Unavailable),
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let failure = request.parse().unwrap_err();
    assert_eq!(
        failure.reason(),
        HtmlParseFailureReason::MissingContextFact(HtmlCaptureFact::HeaderDeliveredCsp)
    );
    assert!(failure.after_cleanup().is_some());
}
