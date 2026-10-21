use crate::parser::support::{environment, fragment_context, limits, record, revision};
use surgeist_html::{HtmlStyleTrigger, KnownHtmlAttribute};
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

fn parse(text: &str) -> HtmlParseReport {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), limits())
        .parse()
        .unwrap()
}
fn occurrences(report: &HtmlParseReport) -> Vec<HtmlParseAssociationOccurrence<'_>> {
    let mut cursor = report.pending_associations();
    let mut rows = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Occurrence(row) => rows.push(row),
            HtmlParsePendingAssociationStep::Complete => return rows,
            _ => {}
        }
    }
}
fn documents(report: &HtmlParseReport) -> Vec<HtmlParseAssociationDocument<'_>> {
    let mut cursor = report.pending_associations();
    let mut rows = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Document(row) => rows.push(row),
            HtmlParsePendingAssociationStep::Complete => return rows,
            _ => {}
        }
    }
}
fn units(report: &HtmlParseReport, group: HtmlParseOriginGroup<'_>) -> Vec<u16> {
    let history = report.history_view();
    let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
    let mut cursor = history.cursor(group, &mut scratch, None).unwrap();
    let mut units = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseHistoryStep::Leaf(leaf) => units.extend_from_slice(leaf.units()),
            HtmlParseHistoryStep::Progress => {}
            HtmlParseHistoryStep::Complete => return units,
            HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
        }
    }
}
#[test]
fn original_style_meta_style_dependencies_retain_decoded_inputs_without_selecting_a_set() {
    let report = parse(
        "<!doctype html><style title=A>a{content:'&amp;'}</style><meta http-equiv=default-style content='B&amp;'><style title=C>c{}</style>",
    );
    let rows = occurrences(&report);
    let meta = rows
        .iter()
        .position(|row| row.kind() == HtmlParseAssociationOccurrenceKind::ChangePreferred)
        .unwrap();
    assert_eq!(
        units(&report, rows[meta].preferred_name().unwrap()),
        "B&".encode_utf16().collect::<Vec<_>>()
    );
    assert!(rows[meta].requires_processing());
    assert!(rows[meta].cause().is_some());
    let before = rows[..meta]
        .iter()
        .rev()
        .find(|row| {
            matches!(
                row.kind(),
                HtmlParseAssociationOccurrenceKind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
            )
        })
        .unwrap();
    let after = rows[meta + 1..]
        .iter()
        .find(|row| {
            matches!(
                row.kind(),
                HtmlParseAssociationOccurrenceKind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
            )
        })
        .unwrap();
    assert_eq!(rows[meta].prior_document(), Some(before.id()));
    assert_eq!(
        units(
            &report,
            before
                .style()
                .unwrap()
                .attribute(KnownHtmlAttribute::Title)
                .unwrap()
        ),
        [65]
    );
    assert_eq!(
        units(&report, before.style().unwrap().text(0).unwrap()),
        "a{content:'&amp;'}".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        units(
            &report,
            after
                .style()
                .unwrap()
                .attribute(KnownHtmlAttribute::Title)
                .unwrap()
        ),
        [67]
    );
    assert!(after.prior_document().is_some());
    assert!(before.style().unwrap().state().created_by_document_parser());
    assert!(!before.style().unwrap().state().on_parser_stack());
    assert!(documents(&report).iter().all(|row| row.disposition() == HtmlParseAssociationCaptureDisposition::PendingComplete));
    assert!(report.retained_construction_bytes() > 0);
    assert!(report.retained_construction_utf16_units() > 0);
}
#[test]
fn offline_style_context_is_explicitly_unbound_while_original_mode_and_body_are_fixed() {
    use HtmlParseAssociationContextRequirement::{Known, OfflineUnbound};
    use HtmlParseAssociationContextRole::*;
    let report = parse(
        "<!doctype html><head><style media=screen>a{}</style></head><body><style>b{}</style>",
    );
    let rows = occurrences(&report);
    let pops: Vec<_> = rows
        .iter()
        .filter(|row| {
            matches!(
                row.kind(),
                HtmlParseAssociationOccurrenceKind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
            )
        })
        .collect();
    assert_eq!(pops.len(), 2);
    assert_eq!(pops[0].context().body_present(), Some(false));
    assert_eq!(pops[1].context().body_present(), Some(true));
    for row in &pops {
        let context = row.context();
        assert_eq!(
            context.document_mode(),
            surgeist_html::HtmlDocumentMode::NoQuirks
        );
        for role in [
            DocumentUrl,
            FallbackBase,
            Encoding,
            InitialPolicy,
            ExternalEnvironment,
        ] {
            assert_eq!(context.requirement(role), OfflineUnbound);
            let slot = context.slot(role).unwrap();
            assert_eq!(slot.epoch(), &context.input_epoch(role));
            assert_eq!(slot.role(), role);
        }
        assert_eq!(context.requirement(BodyPresence), Known);
        assert_eq!(context.requirement(DocumentEligibility), Known);
    }
    for role in [DocumentUrl, FallbackBase, Encoding, InitialPolicy] {
        assert_eq!(pops[0].context().slot(role), pops[1].context().slot(role));
    }
    assert_ne!(pops[0].context().epoch(), pops[1].context().epoch());
    assert_ne!(
        pops[0].context().slot(ExternalEnvironment),
        pops[1].context().slot(ExternalEnvironment)
    );
    assert!(documents(&report)[0].has_obligations());
}
#[test]
fn ignored_template_and_type_ineligible_styles_have_complete_capture_without_obligations() {
    let report = parse(
        "<!doctype html><style type=text/plain title=A>x</style><template><style title=B>y</style><meta http-equiv=default-style content=C></template>",
    );
    let docs = documents(&report);
    assert_eq!(docs.len(), 2);
    assert!(docs.iter().all(|doc| doc.disposition()
        == HtmlParseAssociationCaptureDisposition::PendingComplete
        && !doc.has_obligations()));
    assert!(
        !occurrences(&report)
            .iter()
            .any(|row| row.kind() == HtmlParseAssociationOccurrenceKind::ChangePreferred)
    );
    assert!(occurrences(&report).iter().any(|row| matches!(
        row.kind(),
        HtmlParseAssociationOccurrenceKind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
    )));
}
#[test]
fn zero_sheet_meta_is_an_obligation_and_independent_parses_have_distinct_occurrence_ids() {
    let source = HtmlSource::utf8("<!doctype html><meta http-equiv=default-style content=chosen>");
    let a = HtmlDocumentParseRequest::offline(source.clone(), limits())
        .parse()
        .unwrap();
    let b = HtmlDocumentParseRequest::offline(source, limits())
        .parse()
        .unwrap();
    let a_rows = occurrences(&a);
    let b_rows = occurrences(&b);
    assert_eq!(a_rows.len(), 1);
    assert_eq!(b_rows.len(), 1);
    assert_ne!(a_rows[0].id(), b_rows[0].id());
    assert_eq!(a_rows[0].id(), occurrences(&a)[0].id());
    assert!(documents(&a)[0].has_obligations());
    assert_eq!(documents(&a)[0].occurrence_count(), 1);
}
#[test]
fn influencing_link_and_unknown_rel_are_precise_unavailable_while_nonstylesheet_links_are_not() {
    for (rel, expected) in [
        (
            "stylesheet",
            HtmlParseAssociationMissingKind::UnsupportedLinkStylesheetHistory,
        ),
        (
            "x-vendor",
            HtmlParseAssociationMissingKind::ExtensionLinkHistory,
        ),
    ] {
        let report = parse(&format!(
            "<!doctype html><link rel='{rel}' href=x><meta http-equiv=default-style content=B>"
        ));
        let docs = documents(&report);
        assert_eq!(
            docs[0].disposition(),
            HtmlParseAssociationCaptureDisposition::Unavailable(expected)
        );
        assert!(docs[0].first_missing().is_some());
    }
    let report = parse(
        "<!doctype html><link rel=icon href=x><template><link rel=stylesheet href=x></template>",
    );
    assert!(documents(&report).iter().all(|doc| doc.disposition()
        == HtmlParseAssociationCaptureDisposition::PendingComplete
        && !doc.has_obligations()));
}
#[test]
fn known_base_encoding_and_two_original_base_epochs_are_not_final_tree_reconstruction() {
    use HtmlParseAssociationContextRole::*;
    let revision = revision();
    let old = record(&revision, "https://example.test/old/", false, None);
    let next = record(&revision, "https://example.test/next/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(old.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "https://example.test/next/".into(),
                old.clone(),
                encoding.clone(),
                revision,
                HtmlBaseParseOutcome::Parsed {
                    record: next.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><style title=A>a{}</style><base href='https://example.test/next/'><style title=C>c{}</style>"), limits()).with_base_facts(facts).unwrap().parse().unwrap();
    let rows = occurrences(&report);
    let pops: Vec<_> = rows
        .iter()
        .filter(|row| {
            matches!(
                row.kind(),
                HtmlParseAssociationOccurrenceKind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
            )
        })
        .collect();
    assert_eq!(pops[0].context().selected_base(), Ok(&old));
    assert_eq!(pops[1].context().selected_base(), Ok(&next));
    assert_ne!(pops[0].context().epoch(), pops[1].context().epoch());
    for pop in pops {
        for role in [DocumentUrl, FallbackBase, Encoding] {
            assert_eq!(
                pop.context().requirement(role),
                HtmlParseAssociationContextRequirement::Known
            );
            assert!(pop.context().slot(role).is_none());
        }
    }
    assert!(rows.iter().any(
        |row| row.kind() == HtmlParseAssociationOccurrenceKind::BaseChanged
            && row.base_changed_input().is_some()
    ));
    // The existing provider remains useful independently of association readback.
    assert!(environment(&old, &encoding).document().encoding().is_some());
}
#[test]
fn captured_deferred_observations_preserve_exact_authority_and_never_issue_offline_slots() {
    use HtmlParseAssociationContextRole::*;
    let context = fragment_context();
    let report = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<style media=screen>x{}</style><style>y{}</style>"),
        context.clone(),
        limits(),
    )
    .unwrap()
    .parse()
    .unwrap();
    let rows = occurrences(&report);
    let captured: Vec<_> = rows
        .iter()
        .filter(|row| matches!(row.document(), HtmlParseMetadataDocument::Captured(_)))
        .collect();
    assert!(!captured.is_empty());
    assert!(captured.len() >= 2);
    for role in [DocumentUrl, FallbackBase, Encoding, InitialPolicy] {
        assert_eq!(
            captured.first().unwrap().context().input_epoch(role),
            captured.last().unwrap().context().input_epoch(role)
        );
    }
    for row in captured {
        let original = row.context();
        assert_eq!(original.document_mode(), context.document_mode());
        for role in [
            DocumentUrl,
            FallbackBase,
            Encoding,
            InitialPolicy,
            ExternalEnvironment,
            BodyPresence,
        ] {
            assert_eq!(
                original.requirement(role),
                HtmlParseAssociationContextRequirement::DeferredCaptured
            );
            let observation = original.captured_observation(role).unwrap();
            assert_eq!(
                observation.context().capture().identity(),
                context.capture().identity()
            );
            assert_eq!(observation.document(), context.target_document());
            assert_eq!(observation.epoch(), &original.input_epoch(role));
            assert_eq!(observation.role(), role);
            assert!(original.slot(role).is_none());
        }
    }
}

#[test]
fn initial_input_lineage_is_distinct_for_independent_documents_and_reports() {
    use HtmlParseAssociationContextRole::InitialPolicy;
    let source = "<!doctype html><style>a{}</style><template><style>b{}</style></template>";
    let first = parse(source);
    let second = parse(source);
    let rows = occurrences(&first);
    let first_document = match rows[0].document() {
        HtmlParseMetadataDocument::Owned(node) => node.id(),
        _ => panic!("source-owned Document"),
    };
    let same = rows.iter().find(|row| matches!(row.document(), HtmlParseMetadataDocument::Owned(node) if node.id() == first_document)).unwrap();
    let other = rows.iter().find(|row| matches!(row.document(), HtmlParseMetadataDocument::Owned(node) if node.id() != first_document)).unwrap();
    assert_ne!(
        same.context().slot(InitialPolicy),
        other.context().slot(InitialPolicy)
    );
    assert_ne!(
        same.context().slot(InitialPolicy),
        occurrences(&second)[0].context().slot(InitialPolicy)
    );
}

#[test]
fn captured_declarative_shadow_style_uses_original_shadow_including_root() {
    for connection in [Some(false), Some(true), None] {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                surgeist_html::HtmlDocumentMode::NoQuirks,
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
                surgeist_html::QualifiedName::new(surgeist_html::Namespace::html(), None, "div"),
                surgeist_html::OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        match connection {
            Some(true) => builder.establish_parent(&div, &document).unwrap(),
            Some(false) => builder.establish_root(&div).unwrap(),
            None => {
                // Nearest ordinary form is known; its parent is deliberately
                // unavailable, so it cannot prove a shadow-including root.
                let form = builder
                    .add_element(
                        &document,
                        surgeist_html::QualifiedName::new(
                            surgeist_html::Namespace::html(),
                            None,
                            "form",
                        ),
                        surgeist_html::OrderedAttributes::default(),
                        HtmlCapturedIsValue::Absent,
                        HtmlCaptureRegistryInput::Null,
                    )
                    .unwrap();
                builder.establish_parent(&div, &form).unwrap();
            }
        }
        builder.establish_no_shadow_root(&div).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
        let result = HtmlFragmentParseRequest::with_options(HtmlSource::utf8(
            "<template shadowrootmode=open><style title=chosen>a { color: red }</style></template>"),
            context.clone(), limits(), surgeist_html::HtmlParserScriptingMode::Inert, true).unwrap().parse();
        let Some(connected) = connection else {
            let failure = result.unwrap_err();
            assert!(matches!(
                failure.reason(),
                HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(_))
            ));
            assert!(failure.cleanup_error().is_none());
            continue;
        };
        let report = result.unwrap();
        let rows = occurrences(&report);
        let captured: Vec<_> = rows
            .iter()
            .filter(|row| {
                matches!(row.document(), HtmlParseMetadataDocument::Captured(_))
                    && matches!(
                        row.kind(),
                        HtmlParseAssociationOccurrenceKind::StyleUpdate(_)
                    )
            })
            .collect();
        assert!(!captured.is_empty());
        for row in captured {
            assert_eq!(row.style().unwrap().connected(), connected);
            assert!(!row.style().unwrap().in_document_tree());
            assert_eq!(row.requires_processing(), connected);
            let original = row
                .context()
                .captured_observation(HtmlParseAssociationContextRole::InitialPolicy)
                .unwrap();
            assert_eq!(
                original.context().capture().identity(),
                context.capture().identity()
            );
        }
        assert!(documents(&report).iter().all(
            |doc| doc.disposition() == HtmlParseAssociationCaptureDisposition::PendingComplete
        ));
        assert_eq!(
            documents(&report).iter().any(|doc| doc.has_obligations()),
            connected
        );
    }
}

#[test]
fn captured_template_contents_host_does_not_connect_pending_shadow_style() {
    let context = super::nonce::captured_template_contents_context();
    let report = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<template shadowrootmode=open><style title=A>x{}</style></template>"),
        context.clone(),
        limits(),
        surgeist_html::HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap()
    .parse()
    .unwrap();
    let rows = occurrences(&report);
    let captured: Vec<_> = rows
        .iter()
        .filter(|row| {
            matches!(row.document(), HtmlParseMetadataDocument::Captured(_))
                && matches!(
                    row.kind(),
                    HtmlParseAssociationOccurrenceKind::StyleUpdate(_)
                )
        })
        .collect();
    assert!(!captured.is_empty());
    for row in captured {
        assert!(!row.style().unwrap().connected());
        assert!(!row.style().unwrap().in_document_tree());
        assert!(!row.requires_processing());
        let HtmlParseMetadataDocument::Captured(document) = row.document() else {
            panic!("captured owner")
        };
        assert_eq!(document.node_document(), context.target_document().clone());
    }
    assert!(documents(&report).iter().all(|doc| !doc.has_obligations()
        && doc.disposition() == HtmlParseAssociationCaptureDisposition::PendingComplete));
}
