use surgeist_html::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlInputImageSource,
    HtmlParseFailureReason, HtmlParseLimits, HtmlParseMetadataStep, HtmlParseReport,
    HtmlParseResource, HtmlParseStop, HtmlParseUnsupportedState, HtmlSource,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureError, HtmlCaptureFact, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureTemplateCacheInput, HtmlCapturedIsValue,
    HtmlCapturedNodeKind, HtmlDocumentClass, HtmlFragmentContext, HtmlImageSupport,
    HtmlScriptingStatus, HtmlTemplateDocumentRole,
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

fn fragment_context(support: HtmlImageSupport) -> HtmlFragmentContext {
    fragment_context_scope(support, true)
}
fn fragment_context_scope(support: HtmlImageSupport, creation_scope: bool) -> HtmlFragmentContext {
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
    if support != HtmlImageSupport::Unavailable {
        if creation_scope {
            builder
                .establish_parser_creation_image_support(&document, support)
                .unwrap();
        } else {
            builder
                .establish_document_image_support(&document, support)
                .unwrap();
        }
    }
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

fn parse(literal: &str) -> HtmlParseReport {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap()
}
#[test]
fn filename_is_empty_with_exact_raw_and_current_default_fields_and_real_form_owner() {
    for (attrs, raw) in [
        ("type=file", ""),
        ("type=file value='raw'", "raw"),
        ("value='&#13;&#10;X' type=file checked=false", "\r\nX"),
    ] {
        let literal = format!("<!doctype html><form id=f><input {attrs}>Z</form>");
        let report = parse(&literal);
        assert_eq!(report.image_support(), HtmlImageSupport::Disabled);
        let mut cursor = report.metadata();
        let mut found = false;
        let mut owner = false;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(flags) = record.input_flags() {
                        found = true;
                        assert_eq!(flags.input_type(), HtmlInputType::File);
                        assert_eq!(
                            record.input_raw_value().unwrap().code_units(),
                            DocumentString::from(raw).code_units()
                        );
                        assert_eq!(
                            record.input_file_selection(),
                            Some(HtmlInputFileSelection::Empty)
                        );
                        assert_eq!(
                            flags.input_type().value_mode().readback(
                                record.input_raw_value().unwrap().code_units(),
                                None,
                                record.input_file_selection(),
                            ),
                            Some(&[][..])
                        );
                        assert_eq!(record.input_image_coordinate(), None);
                        assert_eq!(record.input_image_source(), None);
                        assert!(!flags.dirty_value());
                        assert!(!flags.dirty_checkedness());
                        assert!(!flags.indeterminate());
                    }
                    if let Some(form) = record.form_owner() {
                        assert!(form.is_some());
                        owner = true;
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert!(found && owner);
    }
}
#[test]
fn image_initial_coordinates_and_source_are_exact_and_getter_uses_actual_content() {
    for src in [
        None,
        Some(""),
        Some(" \t "),
        Some("relative.png"),
        Some("https://example.invalid/x"),
        Some("data:image/png;base64,AA=="),
        Some("#fragment"),
    ] {
        let source = src.map_or(String::new(), |s| format!("src='{s}'"));
        let report = parse(&format!(
            "<!doctype html><input {source} type=image value='A&#13;B'>Z"
        ));
        let mut cursor = report.metadata();
        let mut found = false;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(flags) = record.input_flags() {
                        found = true;
                        assert_eq!(flags.input_type(), HtmlInputType::Image);
                        let coordinate = record.input_image_coordinate().unwrap();
                        assert_eq!((coordinate.x(), coordinate.y()), (0, 0));
                        assert_eq!(record.input_file_selection(), None);
                        let node = record.node();
                        let HtmlNodeKind::Element(element) = node.kind() else {
                            panic!("actual qualified Input node")
                        };
                        // The external consumer explicitly observes native
                        // content; the metadata lender performs no search.
                        let content = element.attributes().as_slice().iter().find(|a| {
                            a.name().namespace().uri().is_none()
                                && a.name().local_name().code_units() == [118, 97, 108, 117, 101]
                        });
                        assert_eq!(
                            flags.input_type().value_mode().readback(
                                record.input_raw_value().unwrap().code_units(),
                                content.map(|a| a.value().code_units()),
                                record.input_file_selection(),
                            ),
                            Some(&[65, 13, 66][..])
                        );
                        match (src, record.input_image_source().unwrap()) {
                            (None, HtmlInputImageSource::Absent) => {}
                            (Some(source), HtmlInputImageSource::Present(actual)) => assert_eq!(
                                actual.code_units(),
                                DocumentString::from(source).code_units()
                            ),
                            _ => panic!("specified src identity"),
                        }
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert!(found);
    }
}
#[test]
fn fragment_image_policy_is_explicit_conditional_and_capture_qualified() {
    for support in [
        HtmlImageSupport::Unavailable,
        HtmlImageSupport::Disabled,
        HtmlImageSupport::Enabled,
    ] {
        for src in [None, Some(""), Some(" ")] {
            let attrs = src.map_or(String::new(), |s| format!("src='{s}'"));
            let context = fragment_context(support);
            let capture = context.capture().identity().clone();
            let result = HtmlFragmentParseRequest::new(
                HtmlSource::utf8(format!("<input type=image {attrs}>Z")),
                context,
                limits(),
            )
            .unwrap()
            .parse();
            if src == Some(" ") && support != HtmlImageSupport::Disabled {
                let failure = result.unwrap_err();
                assert_eq!(failure.cleanup_error(), None);
                if support == HtmlImageSupport::Unavailable {
                    assert_eq!(
                        failure.reason(),
                        HtmlParseFailureReason::MissingContextFact(HtmlCaptureFact::ImageSupport)
                    );
                } else {
                    assert_eq!(
                        failure.reason(),
                        HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(
                            HtmlParseUnsupportedState::ImageResource
                        ))
                    );
                }
            } else {
                let report = result.unwrap();
                assert_eq!(report.context().unwrap().capture().identity(), &capture);
                assert_eq!(report.image_support(), support);
            }
        }
    }
}
#[test]
fn image_support_is_not_scripting_and_builder_qualification_rejects_foreign_subjects() {
    let mut a = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 100, 100, 1000, 100, 100, 100, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = a
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Enabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    a.establish_document_image_support(&document, HtmlImageSupport::Enabled)
        .unwrap();
    a.establish_parser_creation_image_support(&document, HtmlImageSupport::Disabled)
        .unwrap();
    assert_eq!(
        a.establish_parser_creation_image_support(&document, HtmlImageSupport::Enabled),
        Err(HtmlCaptureError::DocumentFactsAlreadyEstablished)
    );
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 100, 100, 1000, 100, 100, 100, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    assert_eq!(
        b.establish_parser_creation_image_support(&document, HtmlImageSupport::Disabled),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    let capture = a.finish().unwrap();
    let node = capture
        .node(&capture.subject_id(&document).unwrap())
        .unwrap();
    assert!(
        matches!(node.kind(), HtmlCapturedNodeKind::Document(facts) if facts.image_support() == HtmlImageSupport::Enabled && facts.parser_creation_image_support() == HtmlImageSupport::Disabled && facts.scripting() == HtmlScriptingStatus::Enabled)
    );
}
#[test]
fn file_image_exact_work_and_one_short_refusal_keep_source_and_cleanup() {
    let literal = "<!doctype html><form><input type=file value=raw><input src='A&amp;B' type=image value=Q>after</form>";
    let total = parse(literal)
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    let exact = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(literal),
        limits()
            .into_builder()
            .work_credits(total)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap();
    assert_eq!(
        exact
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current(),
        total
    );
    let source = HtmlSource::utf8(literal);
    let identity = source.identity().clone();
    let failure = HtmlDocumentParseRequest::offline(
        source,
        limits()
            .into_builder()
            .work_credits(total - 1)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap_err();
    assert_eq!(failure.source().identity(), &identity);
    assert_eq!(failure.cleanup_error(), None);
    assert!(matches!(
        failure.first_stop(),
        Some(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedEdges,
    ] {
        assert_eq!(
            failure
                .after_cleanup()
                .unwrap()
                .resource(resource)
                .current(),
            0
        );
    }
}
#[test]
fn cancelled_file_image_request_retains_sticky_first_stop_without_partial_output() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<input type=file><input type=image src=Q>"),
        limits(),
    );
    let identity = request.source().identity().clone();
    let cancel = request.cancellation().clone();
    cancel.cancel();
    let failure = request.parse().unwrap_err();
    assert_eq!(failure.first_stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(failure.source().identity(), &identity);
    assert_eq!(failure.cleanup_error(), None);
    let at_stop = failure.at_stop().unwrap().clone();
    cancel.cancel();
    assert_eq!(failure.at_stop(), Some(&at_stop));
}

#[test]
fn file_image_actual_capacity_and_retained_record_boundaries_refuse_atomically() {
    let literal = "<!doctype html><form><input type=file value='raw'><input type=image src='A&amp;B' value=X>tail</form>";
    let report = parse(literal);
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::SourceMapEntries,
    ] {
        let peak = report.usage().resource(resource).high_water();
        assert!(peak > 0);
        let with_limit = |cap| {
            let builder = limits().into_builder();
            match resource {
                HtmlParseResource::ManagedCapacityBytes => builder.managed_capacity_bytes(cap),
                HtmlParseResource::ManagedUtf16Units => builder.managed_utf16_units(cap),
                HtmlParseResource::AuxiliaryEntries => builder.auxiliary_entries(cap),
                HtmlParseResource::SourceMapEntries => builder.source_map_entries(cap),
                _ => unreachable!(),
            }
            .finish()
            .unwrap()
        };
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), with_limit(peak))
            .parse()
            .unwrap();
        let failure =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), with_limit(peak - 1))
                .parse()
                .unwrap_err();
        assert!(
            matches!(failure.first_stop(),Some(HtmlParseStop::LimitExceeded{resource:actual,..}) if actual==resource)
        );
        assert_eq!(failure.cleanup_error(), None);
        for retained in [
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
        ] {
            assert_eq!(
                failure
                    .after_cleanup()
                    .unwrap()
                    .resource(retained)
                    .current(),
                0
            );
        }
    }
}

#[test]
fn temporary_and_nested_inert_documents_require_actual_creation_environment_attestation() {
    for creation_scope in [false, true] {
        for support in [HtmlImageSupport::Disabled, HtmlImageSupport::Enabled] {
            for src in ["", "X"] {
                let context = fragment_context_scope(support, creation_scope);
                let result = HtmlFragmentParseRequest::new(
                    HtmlSource::utf8(format!(
                        "<template><template><input type=image src='{src}'></template></template>"
                    )),
                    context,
                    limits(),
                )
                .unwrap()
                .parse();
                if src.is_empty() || creation_scope && support == HtmlImageSupport::Disabled {
                    result.unwrap();
                } else {
                    let failure = result.unwrap_err();
                    if creation_scope {
                        assert_eq!(
                            failure.reason(),
                            HtmlParseFailureReason::Stop(
                                HtmlParseStop::UnsupportedStandaloneState(
                                    HtmlParseUnsupportedState::ImageResource
                                )
                            )
                        );
                    } else {
                        assert_eq!(
                            failure.reason(),
                            HtmlParseFailureReason::MissingContextFact(
                                HtmlCaptureFact::ImageSupport
                            )
                        );
                    }
                    assert_eq!(failure.cleanup_error(), None);
                }
            }
        }
    }
}

#[test]
fn native_value_modes_use_explicit_content_and_fixed_qualified_metadata_lenders() {
    let cases = [
        (
            "type=text value='A&#13;B'",
            HtmlInputType::Text,
            &[65u16, 66][..],
            Some(&[65u16, 13, 66][..]),
            &[65u16, 66][..],
        ),
        ("type=hidden", HtmlInputType::Hidden, &[], None, &[]),
        (
            "type=hidden value=''",
            HtmlInputType::Hidden,
            &[],
            Some(&[]),
            &[],
        ),
        (
            "type=checkbox",
            HtmlInputType::Checkbox,
            &[],
            None,
            &[111, 110],
        ),
        (
            "type=checkbox value=''",
            HtmlInputType::Checkbox,
            &[],
            Some(&[]),
            &[],
        ),
        (
            "type=file value=Q",
            HtmlInputType::File,
            &[81],
            Some(&[81]),
            &[],
        ),
        ("type=image", HtmlInputType::Image, &[], None, &[]),
        (
            "type=image value=''",
            HtmlInputType::Image,
            &[],
            Some(&[]),
            &[],
        ),
        (
            "type=image value='A&#13;B'",
            HtmlInputType::Image,
            &[65, 13, 66],
            Some(&[65, 13, 66]),
            &[65, 13, 66],
        ),
    ];
    for (attrs, kind, internal, expected_content, expected_value) in cases {
        let report = parse(&format!(
            "<!doctype html><input id=target x:value=unrelated {attrs}>"
        ));
        let mut cursor = report.metadata();
        let mut inputs = 0;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    let Some(flags) = record.input_flags() else {
                        continue;
                    };
                    inputs += 1;
                    assert_eq!(flags.input_type(), kind);
                    let raw = record.input_raw_value().unwrap();
                    assert_eq!(raw.code_units(), internal);
                    let node = record.node();
                    let HtmlNodeKind::Element(element) = node.kind() else {
                        panic!("actual Input metadata subject")
                    };
                    // Application-owned explicit native attribute observation,
                    // separate from every fixed metadata lender and mode rule.
                    let content = element
                        .attributes()
                        .as_slice()
                        .iter()
                        .find(|a| {
                            a.name().namespace().uri().is_none()
                                && a.name().local_name().code_units() == [118, 97, 108, 117, 101]
                        })
                        .map(|a| a.value().code_units());
                    assert_eq!(content, expected_content);
                    let files = record.input_file_selection();
                    assert_eq!(
                        files,
                        (kind == HtmlInputType::File).then_some(HtmlInputFileSelection::Empty)
                    );
                    // Value and Filename consult no content evidence at all.
                    let consulted = match kind.value_mode() {
                        HtmlInputValueMode::Value | HtmlInputValueMode::Filename => None,
                        HtmlInputValueMode::Default | HtmlInputValueMode::DefaultOn => content,
                    };
                    assert_eq!(
                        kind.value_mode()
                            .readback(raw.code_units(), consulted, files),
                        Some(expected_value)
                    );
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(inputs, 1);
    }
}
