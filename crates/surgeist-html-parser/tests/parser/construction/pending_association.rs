#[allow(unused_imports)]
use crate::parser::support::{ReportDetail as _, SubjectCause as _};
use crate::parser::support::{environment, fragment_context, limits, record, revision};
use surgeist_html::{
    construction::HtmlNodeId, elements::style::HtmlStyleTrigger, vocabulary::KnownHtmlAttribute,
};
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

fn parse(text: &str) -> HtmlParseReport {
    crate::parser::support::offline_all(HtmlSource::utf8(text), limits())
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
fn units(value: &surgeist_html::values::DocumentString) -> Vec<u16> {
    value.code_units().to_vec()
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
        units(rows[meta].preferred_name().unwrap()),
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
            before
                .style()
                .unwrap()
                .attribute(KnownHtmlAttribute::Title)
                .unwrap()
        ),
        [65]
    );
    assert_eq!(
        units(before.style().unwrap().text(0).unwrap().unwrap()),
        "a{content:'&amp;'}".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        units(
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
            surgeist_html::construction::HtmlDocumentMode::NoQuirks
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
    let a = crate::parser::support::offline_all(source.clone(), limits())
        .parse()
        .unwrap();
    let b = crate::parser::support::offline_all(source, limits())
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
    let report = crate::parser::support::offline_all(HtmlSource::utf8("<!doctype html><style title=A>a{}</style><base href='https://example.test/next/'><style title=C>c{}</style>"), limits()).with_base_facts(facts).unwrap().parse().unwrap();
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
                surgeist_html::construction::HtmlDocumentMode::NoQuirks,
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
                surgeist_html::values::QualifiedName::new(
                    surgeist_html::values::Namespace::html(),
                    None,
                    "div",
                ),
                surgeist_html::values::OrderedAttributes::default(),
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
                        surgeist_html::values::QualifiedName::new(
                            surgeist_html::values::Namespace::html(),
                            None,
                            "form",
                        ),
                        surgeist_html::values::OrderedAttributes::default(),
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
            context.clone(), limits(), surgeist_html::parsing::HtmlParserScriptingMode::Inert, true).unwrap().parse();
        let Some(connected) = connection else {
            let failure = result.unwrap_err();
            // The capture never established the form's parent, and the failure
            // names that fact.
            assert_eq!(
                failure.reason(),
                HtmlParseFailureReason::MissingContextFact(HtmlCaptureFact::Parent)
            );
            assert!(matches!(
                failure.first_stop(),
                Some(HtmlParseStop::UnsupportedStandaloneState(_))
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
        surgeist_html::parsing::HtmlParserScriptingMode::Inert,
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

/// One base observation for `https://example.test/next/` against the fallback
/// `https://example.test/old/`.
fn next_base_facts() -> HtmlParseBaseFacts {
    let revision = revision();
    let old = record(&revision, "https://example.test/old/", false, None);
    let next = record(&revision, "https://example.test/next/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    HtmlParseBaseFacts::new(
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
                old,
                encoding,
                revision,
                HtmlBaseParseOutcome::Parsed {
                    record: next,
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap()
}

/// For each pending association occurrence, in order, the first base element with an `href` it
/// records and the base observation it selects.
fn recorded_bases(text: &str) -> Vec<(Option<HtmlNodeId>, Option<usize>)> {
    let report = crate::parser::support::offline_all(HtmlSource::utf8(text), limits())
        .with_base_facts(next_base_facts())
        .unwrap()
        .parse()
        .unwrap();
    occurrences(&report)
        .iter()
        .map(|row| {
            let context = row.context();
            (
                context.first_base().map(|node| node.id()),
                context.base_observation(),
            )
        })
        .collect()
}

#[test]
fn a_moved_subtree_records_the_first_base_with_an_href_that_it_holds_for_every_node() {
    // HTML 877d6146487f section 13.2.6.4.7 (adoption agency algorithm): `</b>` makes the furthest
    // block, the `p`, move as one subtree holding a base without an href, a style and then a
    // base with one. Both the removal steps and the insertion steps run for each node of the
    // subtree once the whole subtree has been detached or attached, so with the subtree attached
    // the first base element with an `href` in tree order is the later base from the first of
    // them on, and every occurrence the insertion records selects it and its observation.
    //
    // The expectation is the search's: each of these occurrences, in order, as recorded when the
    // hook of every base looked for the first base with an `href` through the whole document.
    // The markup before `</b>` moves nothing and fixes the occurrences that precede the move.
    let base =
        "<!doctype html><b>1<p><base><style>a{}</style><base href='https://example.test/next/'>";
    let kept = recorded_bases(base);
    let moved = recorded_bases(&format!("{base}</b>"));
    let shape = |rows: &[(Option<HtmlNodeId>, Option<usize>)]| -> Vec<(bool, Option<usize>)> {
        rows.iter()
            .map(|(first, observation)| (first.is_some(), *observation))
            .collect()
    };
    assert_eq!(shape(&moved[..kept.len()]), shape(&kept));
    let (none, first) = ((false, None), (true, Some(0)));
    let expected = [vec![none; 3], vec![first; 6], vec![none; 2], vec![first; 3]].concat();
    assert_eq!(
        shape(&moved[kept.len()..]),
        expected,
        "during the move {:?}",
        &moved[kept.len()..]
    );
    let firsts: Vec<_> = moved.iter().filter_map(|(first, _)| *first).collect();
    assert!(firsts.iter().all(|first| *first == firsts[0]));
}
