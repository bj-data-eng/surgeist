use crate::parser::support::limits;
use surgeist_html::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseDestination,
    HtmlParseFailureReason, HtmlParseMetadataStep, HtmlParseRawField, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureFact, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureTemplateCacheInput, HtmlCapturedIsValue, HtmlDocumentClass, HtmlFragmentContext,
    HtmlScriptingStatus, HtmlTemplateDocumentRole,
};
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
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
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
            if matches!(associations.row(index).unwrap().unwrap().destination(), HtmlParseDestination::Raw { node, kind: HtmlParseRawField::Nonce } if node.id() == p.id())
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
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
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
                HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
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
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
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

pub(super) fn captured_template_contents_context() -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let inert = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &inert,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::InertTemplate,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Document(inert.clone()),
        )
        .unwrap();
    builder
        .establish_document_nonce_environment(
            &document,
            HtmlNonceDocumentEnvironment::new(
                HtmlBrowsingContextPresence::Present,
                HtmlHeaderDeliveredCsp::Present,
            ),
        )
        .unwrap();
    let template = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "template"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_parent(&template, &document).unwrap();
    let contents = builder.add_fragment(&inert).unwrap();
    builder
        .establish_template_contents(&template, &contents)
        .unwrap();
    let div = builder
        .add_element(
            &inert,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_parent(&div, &contents).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

#[test]
fn captured_template_contents_host_does_not_connect_nonce_to_browsing_context() {
    let context = captured_template_contents_context();
    let report = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<template shadowrootmode=open><p nonce=A></p></template>"),
        context,
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap()
    .parse()
    .unwrap();
    assert!(
        slots(&report)
            .iter()
            .any(|(name, value)| name == "p" && value == &DocumentString::from("A"))
    );
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record)
                if record.cryptographic_nonce() == Some(&DocumentString::from("A")) =>
            {
                let HtmlNodeKind::Element(element) = record.node().kind() else {
                    panic!("nonce element")
                };
                // A template's contents host is not a ShadowRoot host. The
                // connected outer Document's policy cannot hide this attribute.
                assert_eq!(
                    element.attributes().as_slice()[0].value(),
                    &DocumentString::from("A")
                );
                break;
            }
            HtmlParseMetadataStep::Complete => panic!("p nonce record"),
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            _ => {}
        }
    }
}
