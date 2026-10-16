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

fn fragment_context(
    mode: HtmlDocumentMode,
    activity: HtmlBrowsingContextPresence,
) -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            mode,
            HtmlScriptingStatus::Enabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_nonce_environment(
            &document,
            HtmlNonceDocumentEnvironment::new(activity, HtmlHeaderDeliveredCsp::Unavailable),
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

// HTML877d interactive1026-1094/1294-1326 and foundations4879.
// This is actual completed original-controller state, not native creation.
#[test]
fn ordinary_open_dialog_keeps_empty_return_value_and_closed_document_history() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><dialog open returnvalue=attribute-only></dialog><template><dialog open></dialog></template>"),
        limits(),
    ).parse().unwrap();
    let mut cursor = report.metadata();
    let (mut dialogs, mut documents) = (0, 0);
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(dialog) = row.dialog() {
                    assert_eq!(dialog.return_value().code_units(), []);
                    assert!(matches!(
                        dialog.disposition(),
                        HtmlParseDialogDisposition::Closed
                    ));
                    dialogs += 1;
                }
                if let Some(state) = row.document_interaction() {
                    assert_eq!(state.document().id(), row.node().id());
                    assert!(matches!(
                        state.disposition(),
                        HtmlParseDialogDisposition::Closed
                    ));
                    documents += 1;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(dialogs, 2);
    assert_eq!(documents, 2); // actual ordinary and associated inert owners
}

#[test]
fn contextual_dialog_missing_activity_retains_the_first_actual_document_action() {
    // This capture supplies no actual BC-Absent proof. Scripting status alone
    // cannot answer fully-active; the existing source-ordered obligation stays
    // unavailable without failing or inventing state for the structural report.
    let context = fragment_context(
        HtmlDocumentMode::NoQuirks,
        HtmlBrowsingContextPresence::Unavailable,
    );
    let report = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<div><dialog open></dialog></div>"),
        context,
        limits(),
    )
    .unwrap()
    .parse()
    .unwrap();
    let mut cursor = report.metadata();
    let mut dialogs = 0;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(dialog) = row.dialog() {
                    assert_eq!(dialog.return_value().code_units(), []);
                    let HtmlParseDialogDisposition::Unavailable(missing) = dialog.disposition()
                    else {
                        panic!("actual captured activity is unavailable");
                    };
                    assert_eq!(missing.subject().id(), row.node().id());
                    assert!(matches!(
                        missing.document(),
                        HtmlParseMetadataDocument::Captured(_)
                    ));
                    assert_eq!(missing.action(), HtmlDialogAction::ReadFullyActive);
                    assert!(missing.cause().is_some());
                    dialogs += 1;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(dialogs, 1);
}

#[test]
fn dialog_free_document_has_its_own_history_and_foreign_dialog_has_no_html_state() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><svg><dialog open/></svg><p>plain</p>"),
        limits(),
    )
    .parse()
    .unwrap();
    let mut cursor = report.metadata();
    let (mut dialogs, mut documents) = (0, 0);
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                dialogs += usize::from(row.dialog().is_some());
                if let Some(state) = row.document_interaction() {
                    assert!(matches!(
                        state.disposition(),
                        HtmlParseDialogDisposition::Closed
                    ));
                    documents += 1;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(dialogs, 0);
    assert_eq!(documents, 1);
}

#[test]
fn captured_absent_activity_and_actual_inert_or_auxiliary_documents_stay_distinct() {
    for activity in [
        HtmlBrowsingContextPresence::Absent,
        HtmlBrowsingContextPresence::Present,
    ] {
        let source = if activity == HtmlBrowsingContextPresence::Absent {
            "<dialog open></dialog><div><dialog open></dialog></div><template><dialog open></dialog></template>"
        } else {
            "<template><dialog open></dialog></template>"
        };
        let report = HtmlFragmentParseRequest::new(
            HtmlSource::utf8(source),
            fragment_context(HtmlDocumentMode::NoQuirks, activity),
            limits(),
        )
        .unwrap()
        .parse()
        .unwrap();
        let mut cursor = report.metadata();
        let (mut dialogs, mut documents, mut inert_documents) = (0, 0, 0);
        let mut inert_ids = Vec::new();
        let mut auxiliary_inert = None;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    if let Some(dialog) = row.dialog() {
                        assert!(matches!(
                            dialog.disposition(),
                            HtmlParseDialogDisposition::Closed
                        ));
                        assert!(dialog.return_value().is_empty());
                        dialogs += 1;
                    }
                    if let Some(document) = row.document_interaction() {
                        assert!(matches!(
                            document.disposition(),
                            HtmlParseDialogDisposition::Closed
                        ));
                        assert!(matches!(row.node().kind(), HtmlNodeKind::Document(_)));
                        let (_, inert, cache) = row.document_facts().unwrap();
                        inert_documents += usize::from(inert);
                        if inert {
                            assert!(!inert_ids.contains(&row.node().id()));
                            inert_ids.push(row.node().id());
                        }
                        if !inert {
                            assert_eq!(
                                row.node().id(),
                                report.fragment_parser_document().unwrap().document().id()
                            );
                            auxiliary_inert = Some(
                                cache
                                    .expect("actual auxiliary owner's first inert cache")
                                    .id(),
                            );
                        }
                        documents += 1;
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(
            dialogs,
            if activity == HtmlBrowsingContextPresence::Absent {
                3
            } else {
                1
            }
        );
        // Direct-root Template creation first uses auxiliary Owned0 and its
        // inert owner. Adoption into the output's captured target invokes the
        // template adoption steps and creates a separate target inert cache.
        // Both actual historical inert Documents remain retained, not merged.
        assert_eq!(documents, 3);
        assert_eq!(inert_documents, 2);
        let auxiliary_inert = auxiliary_inert.unwrap();
        assert!(inert_ids.contains(&auxiliary_inert));
        let effects = report.effects().unwrap();
        assert_eq!(effects.len(), 1);
        let HtmlParseEffect::TemplateCache { document, inert } = effects.effect(0).unwrap() else {
            panic!("actual captured target cache creation");
        };
        assert_eq!(
            &document.id(),
            report.context().unwrap().target_document().subject()
        );
        assert!(inert_ids.contains(&inert.id()));
        assert_ne!(inert.id(), auxiliary_inert);
    }
}

#[test]
fn current_dialog_lenders_are_repeated_fixed_reads_without_mutating_usage() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><dialog open></dialog>"),
        limits(),
    )
    .parse()
    .unwrap();
    let before = report.retained_construction_bytes();
    let work = report
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(dialog) = row.dialog() {
                    for _ in 0..3 {
                        assert!(matches!(
                            dialog.disposition(),
                            HtmlParseDialogDisposition::Closed
                        ));
                        assert!(dialog.return_value().is_empty());
                    }
                    assert!(row.document_interaction().is_none());
                }
                if let Some(document) = row.document_interaction() {
                    for _ in 0..3 {
                        assert_eq!(document.document().id(), row.node().id());
                        assert!(matches!(
                            document.disposition(),
                            HtmlParseDialogDisposition::Closed
                        ));
                    }
                    assert!(row.dialog().is_none());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(report.retained_construction_bytes(), before);
    assert_eq!(
        report
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current(),
        work
    );
}

#[test]
fn dialog_owned_records_obey_exact_capacity_work_and_cleanup_boundaries() {
    let make = |limits| {
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><dialog open returnvalue=raw></dialog><template><dialog open></dialog></template>",
            ),
            limits,
        )
    };
    let report = make(limits()).parse().unwrap();
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::WorkCredits,
    ] {
        // The original owner supplies each independent domain's actual high
        // water; equality admits that same full operation, one-short refuses.
        // No private memory formula or logical-usage substitute is inferred.
        let peak = report.usage().resource(resource).high_water();
        let selected = |cap| {
            let builder = limits().into_builder();
            match resource {
                HtmlParseResource::ManagedCapacityBytes => builder.managed_capacity_bytes(cap),
                HtmlParseResource::AuxiliaryEntries => builder.auxiliary_entries(cap),
                HtmlParseResource::WorkCredits => builder.work_credits(cap),
                _ => unreachable!(),
            }
            .finish()
            .unwrap()
        };
        make(selected(peak)).parse().unwrap();
        let failure = make(selected(peak - 1)).parse().unwrap_err();
        assert!(
            matches!(failure.first_stop(), Some(HtmlParseStop::LimitExceeded {resource:actual,..}) if actual == resource)
        );
        assert_eq!(failure.cleanup_error(), None);
        for resource in [
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::AuxiliaryEntries,
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
        // ManagedNodes is the original nonreclaimable issuance counter, and
        // work is monotone. Teardown releases backing without refunding either.
        for resource in [
            HtmlParseResource::ManagedNodes,
            HtmlParseResource::WorkCredits,
        ] {
            assert_eq!(
                failure
                    .after_cleanup()
                    .unwrap()
                    .resource(resource)
                    .current(),
                failure.at_stop().unwrap().resource(resource).current()
            );
        }
    }
    let request = make(limits());
    request.cancellation().cancel();
    let failure = request.parse().unwrap_err();
    assert_eq!(failure.first_stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(failure.cleanup_error(), None);
}
