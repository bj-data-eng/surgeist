use surgeist_html::*;

struct Cap {
    resource: HtmlParseResource,
    set: fn(HtmlParseLimitsBuilder, u64) -> HtmlParseLimitsBuilder,
    get: fn(&HtmlParseLimits) -> u64,
    maximum: u64,
}
fn caps() -> [Cap; 19] {
    [
        Cap {
            resource: HtmlParseResource::SourceUnits,
            set: HtmlParseLimitsBuilder::source_units,
            get: HtmlParseLimits::source_units,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::BufferedUtf16Units,
            set: HtmlParseLimitsBuilder::buffered_utf16_units,
            get: HtmlParseLimits::buffered_utf16_units,
            maximum: (isize::MAX as u64) / 2,
        },
        Cap {
            resource: HtmlParseResource::TokenAttributes,
            set: HtmlParseLimitsBuilder::token_attributes,
            get: HtmlParseLimits::token_attributes,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::AuxiliaryEntries,
            set: HtmlParseLimitsBuilder::auxiliary_entries,
            get: HtmlParseLimits::auxiliary_entries,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::ManagedNodes,
            set: HtmlParseLimitsBuilder::managed_nodes,
            get: HtmlParseLimits::managed_nodes,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::ManagedEdges,
            set: HtmlParseLimitsBuilder::managed_edges,
            get: HtmlParseLimits::managed_edges,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::ManagedAttributes,
            set: HtmlParseLimitsBuilder::managed_attributes,
            get: HtmlParseLimits::managed_attributes,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::ManagedUtf16Units,
            set: HtmlParseLimitsBuilder::managed_utf16_units,
            get: HtmlParseLimits::managed_utf16_units,
            maximum: (isize::MAX as u64) / 2,
        },
        Cap {
            resource: HtmlParseResource::SourceMapEntries,
            set: HtmlParseLimitsBuilder::source_map_entries,
            get: HtmlParseLimits::source_map_entries,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::SourceMapPayloadUnits,
            set: HtmlParseLimitsBuilder::source_map_payload_units,
            get: HtmlParseLimits::source_map_payload_units,
            maximum: (isize::MAX as u64) / 2,
        },
        Cap {
            resource: HtmlParseResource::Diagnostics,
            set: HtmlParseLimitsBuilder::diagnostics,
            get: HtmlParseLimits::diagnostics,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::DiagnosticPayloadUnits,
            set: HtmlParseLimitsBuilder::diagnostic_payload_units,
            get: HtmlParseLimits::diagnostic_payload_units,
            maximum: (isize::MAX as u64) / 2,
        },
        Cap {
            resource: HtmlParseResource::ContextEffects,
            set: HtmlParseLimitsBuilder::context_effects,
            get: HtmlParseLimits::context_effects,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::OpenElements,
            set: HtmlParseLimitsBuilder::open_elements,
            get: HtmlParseLimits::open_elements,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::TreeDepth,
            set: HtmlParseLimitsBuilder::tree_depth,
            get: HtmlParseLimits::tree_depth,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::ManagedCapacityBytes,
            set: HtmlParseLimitsBuilder::managed_capacity_bytes,
            get: HtmlParseLimits::managed_capacity_bytes,
            maximum: usize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::PeakTransientBytes,
            set: HtmlParseLimitsBuilder::peak_transient_bytes,
            get: HtmlParseLimits::peak_transient_bytes,
            maximum: usize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::AtomicLayoutBytes,
            set: HtmlParseLimitsBuilder::atomic_layout_bytes,
            get: HtmlParseLimits::atomic_layout_bytes,
            maximum: isize::MAX as u64,
        },
        Cap {
            resource: HtmlParseResource::WorkCredits,
            set: HtmlParseLimitsBuilder::work_credits,
            get: HtmlParseLimits::work_credits,
            maximum: u64::MAX,
        },
    ]
}
fn limits(value: u64) -> HtmlParseLimits {
    caps()
        .into_iter()
        .fold(HtmlParseLimits::builder(), |builder, cap| {
            (cap.set)(builder, value)
        })
        .finish()
        .unwrap()
}

#[test]
fn each_cap_is_required_and_named_getters_preserve_all_values() {
    assert_eq!(
        HtmlParseLimits::builder().finish(),
        Err(HtmlParseLimitsError::MissingResource(
            HtmlParseResource::SourceUnits
        ))
    );
    for missing in 0..19 {
        let builder = caps().into_iter().enumerate().fold(
            HtmlParseLimits::builder(),
            |builder, (index, cap)| {
                if index == missing {
                    builder
                } else {
                    (cap.set)(builder, 0)
                }
            },
        );
        assert_eq!(
            builder.finish(),
            Err(HtmlParseLimitsError::MissingResource(
                caps()[missing].resource
            ))
        );
    }
    let value = caps()
        .into_iter()
        .enumerate()
        .fold(HtmlParseLimits::builder(), |builder, (index, cap)| {
            (cap.set)(builder, index as u64 + 1)
        })
        .finish()
        .unwrap();
    for (index, cap) in caps().into_iter().enumerate() {
        assert_eq!((cap.get)(&value), index as u64 + 1);
    }
}
#[test]
fn zero_and_exact_representational_maxima_are_checked_by_each_domain() {
    let zero = limits(0);
    for cap in caps() {
        assert_eq!((cap.get)(&zero), 0);
        let maximum = (cap.set)(zero.into_builder(), cap.maximum)
            .finish()
            .unwrap();
        assert_eq!((cap.get)(&maximum), cap.maximum);
        if let Some(invalid) = cap.maximum.checked_add(1) {
            assert_eq!(
                (cap.set)(zero.into_builder(), invalid).finish(),
                Err(HtmlParseLimitsError::Unrepresentable {
                    resource: cap.resource,
                    provided: invalid,
                    maximum: cap.maximum
                })
            );
        }
    }
}
#[test]
fn multiple_invalid_caps_preserve_existing_resource_precedence() {
    let builder = limits(0)
        .into_builder()
        .buffered_utf16_units(isize::MAX as u64)
        .atomic_layout_bytes(u64::MAX);
    assert_eq!(
        builder.clone().finish(),
        Err(HtmlParseLimitsError::Unrepresentable {
            resource: HtmlParseResource::BufferedUtf16Units,
            provided: isize::MAX as u64,
            maximum: isize::MAX as u64 / 2
        })
    );
    assert_eq!(
        builder.buffered_utf16_units(0).finish(),
        Err(HtmlParseLimitsError::Unrepresentable {
            resource: HtmlParseResource::AtomicLayoutBytes,
            provided: u64::MAX,
            maximum: isize::MAX as u64
        })
    );
}

fn capture(
    scripting: HtmlScriptingStatus,
) -> (HtmlCapture, HtmlCaptureDocumentId, HtmlCaptureSubjectId) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(20, 100, 20, 1000, 20, 20, 20),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            scripting,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&element).unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let element = capture.subject_id(&element).unwrap();
    (capture, document, element)
}
#[test]
fn offline_request_retains_exact_source_identity_units_and_disabled_mode_without_admission() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
        0xFEFF, 0xD800, 13, 10, 0xDC00,
    ]));
    let equal_new = HtmlSource::utf16(source.as_utf16().unwrap().clone());
    let request = HtmlDocumentParseRequest::offline(source.clone(), limits(0));
    assert_eq!(request.source().identity(), source.identity());
    assert_ne!(request.source().identity(), equal_new.identity());
    assert_eq!(
        request.source().coordinates(),
        HtmlSourceCoordinates::Utf16CodeUnits
    );
    assert_eq!(
        request.source().as_utf16().unwrap().code_units(),
        &[0xFEFF, 0xD800, 13, 10, 0xDC00]
    );
    assert_eq!(request.limits(), &limits(0));
    assert_eq!(request.scripting_mode(), HtmlParserScriptingMode::Disabled);
    assert!(!request.allow_declarative_shadow_roots());
    assert!(request.capture().is_none());
    assert!(request.document().is_none());
    let utf8 = HtmlDocumentParseRequest::offline(HtmlSource::utf8("\u{FEFF}😀\r\n"), limits(0));
    assert_eq!(utf8.source().as_utf8(), Some("\u{FEFF}😀\r\n"));
    assert_eq!(
        utf8.source().coordinates(),
        HtmlSourceCoordinates::Utf8Bytes
    );
}
#[test]
fn retry_has_fresh_cancellation_while_clones_cancel_the_original_across_threads() {
    let source = HtmlSource::utf8("x");
    let first = HtmlDocumentParseRequest::offline(source.clone(), limits(0));
    let retry = HtmlDocumentParseRequest::offline(source, limits(0));
    let handle = first.cancellation().clone();
    std::thread::spawn(move || handle.cancel()).join().unwrap();
    assert!(first.cancellation().is_cancelled());
    first.cancellation().cancel();
    assert!(first.cancellation().is_cancelled());
    assert!(!retry.cancellation().is_cancelled());
    assert_eq!(first.source().identity(), retry.source().identity());
}
#[test]
fn captured_document_status_and_foreign_selection_are_typed() {
    let (disabled, document, element) = capture(HtmlScriptingStatus::Disabled);
    assert!(matches!(
        disabled.document_id(&element),
        Err(HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Document,
            actual: HtmlCaptureSubjectKind::Element
        })
    ));
    let request = HtmlDocumentParseRequest::captured(
        HtmlSource::utf8("x"),
        disabled.clone(),
        document.clone(),
        limits(0),
    )
    .unwrap();
    assert_eq!(request.capture().unwrap().identity(), disabled.identity());
    assert_eq!(
        request.capture().unwrap().host_facts_revision(),
        disabled.host_facts_revision()
    );
    assert_eq!(request.document(), Some(&document));
    assert_eq!(request.scripting_mode(), HtmlParserScriptingMode::Disabled);
    assert!(!request.allow_declarative_shadow_roots());
    let (foreign, _, _) = capture(HtmlScriptingStatus::Disabled);
    assert_eq!(
        HtmlDocumentParseRequest::captured(HtmlSource::utf8(""), foreign, document, limits(0))
            .unwrap_err(),
        HtmlParseRequestError::Capture(HtmlCaptureError::ForeignCapture)
    );
    for scripting in [
        HtmlScriptingStatus::Enabled,
        HtmlScriptingStatus::Unavailable,
    ] {
        let (capture, document, _) = capture(scripting);
        let expected = if scripting == HtmlScriptingStatus::Enabled {
            HtmlCaptureError::UnsupportedExecution(HtmlParserScriptingMode::Normal)
        } else {
            HtmlCaptureError::MissingFact {
                subject: Some(document.subject().clone()),
                fact: HtmlCaptureFact::ScriptingStatus,
            }
        };
        assert_eq!(
            HtmlDocumentParseRequest::captured(HtmlSource::utf8(""), capture, document, limits(0))
                .unwrap_err(),
            HtmlParseRequestError::Capture(expected)
        );
    }
}
#[test]
fn fragment_mode_matrix_preserves_requested_effective_and_exact_missing_fact() {
    for scripting in [
        HtmlScriptingStatus::Disabled,
        HtmlScriptingStatus::Enabled,
        HtmlScriptingStatus::Unavailable,
    ] {
        let (capture, _, element) = capture(scripting);
        let context = HtmlFragmentContext::element(&capture, &element).unwrap();
        for requested in [
            HtmlParserScriptingMode::Normal,
            HtmlParserScriptingMode::Disabled,
            HtmlParserScriptingMode::Inert,
            HtmlParserScriptingMode::Fragment,
        ] {
            for allow in [false, true] {
                let request = HtmlFragmentParseRequest::with_options(
                    HtmlSource::utf8(""),
                    context.clone(),
                    limits(0),
                    requested,
                    allow,
                );
                let expected = match (scripting, requested) {
                    (_, HtmlParserScriptingMode::Normal | HtmlParserScriptingMode::Disabled) => {
                        Err(HtmlCaptureError::InvalidFragmentScriptingMode(requested))
                    }
                    (HtmlScriptingStatus::Disabled, _) => Ok(HtmlParserScriptingMode::Disabled),
                    (HtmlScriptingStatus::Enabled, HtmlParserScriptingMode::Inert) => {
                        Ok(HtmlParserScriptingMode::Inert)
                    }
                    (HtmlScriptingStatus::Enabled, _) => Err(
                        HtmlCaptureError::UnsupportedExecution(HtmlParserScriptingMode::Fragment),
                    ),
                    (HtmlScriptingStatus::Unavailable, _) => Err(HtmlCaptureError::MissingFact {
                        subject: Some(context.context_document().subject().clone()),
                        fact: HtmlCaptureFact::ScriptingStatus,
                    }),
                };
                match expected {
                    Ok(effective) => {
                        let request = request.unwrap();
                        assert_eq!(request.requested_scripting_mode(), requested);
                        assert_eq!(request.effective_scripting_mode(), effective);
                        assert_eq!(request.allow_declarative_shadow_roots(), allow);
                        assert_eq!(request.context().capture().identity(), capture.identity());
                        assert_eq!(
                            request.context().target_document(),
                            context.target_document()
                        );
                        assert_eq!(
                            request.context().context_document(),
                            context.context_document()
                        );
                    }
                    Err(error) => {
                        assert_eq!(request.unwrap_err(), HtmlParseRequestError::Capture(error))
                    }
                }
            }
        }
        let default = HtmlFragmentParseRequest::new(HtmlSource::utf8(""), context, limits(0));
        match scripting {
            HtmlScriptingStatus::Unavailable => assert!(default.is_err()),
            _ => {
                let default = default.unwrap();
                assert_eq!(
                    default.requested_scripting_mode(),
                    HtmlParserScriptingMode::Inert
                );
                assert!(!default.allow_declarative_shadow_roots());
                assert!(!default.cancellation().is_cancelled());
                assert_eq!(default.limits(), &limits(0));
            }
        }
    }
}

#[test]
fn hosted_fragment_retains_distinct_target_and_context_documents_and_source() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(20, 100, 20, 1000, 20, 20, 20),
        HtmlHostFactsRevision::fresh(),
    );
    let normal = builder
        .add_document(
            HtmlDocumentMode::LimitedQuirks,
            HtmlScriptingStatus::Enabled,
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
    let template = builder
        .add_element(
            &normal,
            QualifiedName::new(Namespace::html(), None, "template"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&template).unwrap();
    let fragment = builder.add_fragment(&inert).unwrap();
    builder
        .establish_template_contents(&template, &fragment)
        .unwrap();
    let capture = builder.finish().unwrap();
    let fragment = capture.subject_id(&fragment).unwrap();
    let context = HtmlFragmentContext::hosted_fragment(&capture, &fragment).unwrap();
    assert_ne!(context.target_document(), context.context_document());
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![0xD800, 0xFEFF]));
    let request =
        HtmlFragmentParseRequest::new(source.clone(), context.clone(), limits(0)).unwrap();
    assert_eq!(request.source().identity(), source.identity());
    assert_eq!(
        request.source().as_utf16().unwrap().code_units(),
        &[0xD800, 0xFEFF]
    );
    assert_eq!(request.context().target(), context.target());
    assert_eq!(
        request.context().context_element(),
        context.context_element()
    );
    assert_eq!(
        request.context().target_document(),
        context.target_document()
    );
    assert_eq!(
        request.context().context_document(),
        context.context_document()
    );
    assert_eq!(
        request.context().document_mode(),
        HtmlDocumentMode::LimitedQuirks
    );
    assert_eq!(
        request.effective_scripting_mode(),
        HtmlParserScriptingMode::Inert
    );
    assert_eq!(
        HtmlFragmentParseRequest::with_options(
            source,
            context,
            limits(0),
            HtmlParserScriptingMode::Fragment,
            true
        )
        .unwrap_err(),
        HtmlParseRequestError::Capture(HtmlCaptureError::UnsupportedExecution(
            HtmlParserScriptingMode::Fragment
        ))
    );
}
