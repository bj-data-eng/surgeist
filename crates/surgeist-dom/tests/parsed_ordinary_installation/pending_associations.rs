//! Actual parser histories retained by the canonical owner, without execution.
use super::*;
use surgeist_html_parser::{
    HtmlParseAssociationOccurrenceKind as Kind, HtmlParsePendingAssociationStep as Step,
};

fn preferred(report: &HtmlParseReport) -> Vec<Vec<u16>> {
    let history = report.history_view();
    let mut rows = report.pending_associations();
    let mut names = Vec::new();
    loop {
        match rows.advance() {
            Step::Occurrence(row) if row.kind() == Kind::ChangePreferred => {
                let mut scratch =
                    vec![surgeist_html_parser::HtmlParseHistoryFrame::default(); history.len()];
                let mut cursor = history
                    .cursor(row.preferred_name().unwrap(), &mut scratch, None)
                    .unwrap();
                let mut name = Vec::new();
                loop {
                    match cursor.advance() {
                        surgeist_html_parser::HtmlParseHistoryStep::Leaf(leaf) => {
                            name.extend_from_slice(leaf.units())
                        }
                        surgeist_html_parser::HtmlParseHistoryStep::Progress => {}
                        surgeist_html_parser::HtmlParseHistoryStep::Complete => break,
                        surgeist_html_parser::HtmlParseHistoryStep::Cancelled => {
                            panic!("uncancelled history")
                        }
                    }
                }
                names.push(name);
            }
            Step::Complete => break,
            _ => {}
        }
    }
    names
}

#[test]
fn pending_association_zero_sheet_history_survives_drop_move_and_supported_node_retirement() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=first><meta http-equiv=default-style content=second><template><style></style></template><p>survivor",
    ));
    let weak = Arc::downgrade(&report);
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let meta = element(&installed, "meta");
    let paragraph = element(&installed, "p");
    let style = element(&installed, "style");
    let expected = vec![
        "first".encode_utf16().collect::<Vec<_>>(),
        "second".encode_utf16().collect(),
    ];
    assert_eq!(preferred(installed.report()), expected);
    drop(installed);
    assert!(weak.upgrade().is_some());
    let mut moved = dom;
    moved
        .set_limits(moved.limits().with_meta_processing_work(100_000))
        .unwrap();
    let mut setup = DomBatch::new();
    let fragment = setup.create_fragment(document);
    let created = moved.apply(setup, &AtomicBool::new(false)).unwrap();
    let fragment = created.created(&fragment).unwrap().unwrap();
    let mut edit = DomBatch::new();
    edit.destroy(fragment);
    // This original Style is in inert Template contents: its actual removal
    // is disconnected-to-disconnected and requires no association update.
    // A connected pending Style's reached update is guarded separately.
    edit.detach(style);
    edit.destroy(style);
    edit.set_attribute(paragraph, "class", "permitted");
    edit.detach(meta);
    edit.destroy(meta);
    moved.apply(edit, &AtomicBool::new(false)).unwrap();
    assert_eq!(moved.pending_parsed_html_associations().count(), 1);
    let pending = moved.pending_parsed_html_associations().next().unwrap();
    assert_eq!(pending.id(), id);
    assert_eq!(pending.documents().len(), 1);
    assert_eq!(pending.documents()[0].document(), document);
    assert_eq!(preferred(pending.report()), expected);
    assert!(moved.snapshot().node(meta).is_err());
    assert!(moved.snapshot().node(style).is_err());
    assert!(moved.snapshot().node(fragment).is_err());
    drop(moved);
    assert!(weak.upgrade().is_none());
}

#[test]
fn pending_association_independent_installs_share_original_history_but_not_destination_identity() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen><p>X",
    ));
    let (mut dom, environment) = setup();
    let first = install(&mut dom, report.clone(), environment.clone());
    let first_id = first.pending_association().unwrap();
    let first_document = first.document();
    let first_usage = dom.usage();
    let second = install(&mut dom, report.clone(), environment);
    let second_id = second.pending_association().unwrap();
    assert_ne!(first_id, second_id);
    assert_ne!(first_document, second.document());
    let original_ids = |report: &HtmlParseReport| {
        let mut rows = report.pending_associations();
        let mut ids = Vec::new();
        loop {
            match rows.advance() {
                Step::Occurrence(row) => ids.push(row.id()),
                Step::Complete => break,
                _ => {}
            }
        }
        ids
    };
    let source_ids = original_ids(&report);
    drop((first, second));
    assert_eq!(dom.pending_parsed_html_associations().count(), 2);
    for pending in dom.pending_parsed_html_associations() {
        assert_eq!(original_ids(pending.report()), source_ids);
    }
    // The same actual producer graph is retained once; only the independent
    // destination's maps/index/metadata grow. No private tariff is an oracle.
    assert_eq!(
        dom.usage().parsed_association_utf16_units(),
        first_usage.parsed_association_utf16_units()
    );
    assert!(dom.usage().parsed_association_bytes() > first_usage.parsed_association_bytes());
}

#[test]
fn pending_association_native_meta_crossing_refuses_atomically_but_unrelated_document_is_usable() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen><p>X",
    ));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let head = element(&installed, "head");
    dom.set_limits(dom.limits().with_meta_processing_work(100_000))
        .unwrap();
    let before = dom.snapshot().stamp();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "different");
    batch.insert_before(&meta, head, None);
    assert!(
        matches!(dom.apply(batch, &AtomicBool::new(false)), Err(DomError::ParsedAssociationPending { document: actual, pending }) if actual == document && pending == id)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage(), usage);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let paragraph = batch.create_element(&other, "p");
    batch.insert_before(&paragraph, &other, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        id
    );
}

#[test]
fn pending_association_disabled_and_detached_style_adoption_keep_exact_document_guards() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen>",
    ));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let document = installed.document();
    let id = installed.pending_association().unwrap();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let style = batch.create_element_ns(&other, Namespace::html(), "style");
    let result = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let style = result.created(&style).unwrap().unwrap();
    let before = dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.adopt(style, document);
    assert!(
        matches!(dom.apply(batch, &AtomicBool::new(false)), Err(DomError::ParsedAssociationPending { document: actual, pending }) if actual == document && pending == id)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert_ne!(
        dom.snapshot()
            .node(style)
            .unwrap()
            .owner_document()
            .unwrap(),
        document
    );
}

#[test]
fn pending_association_caps_cancellation_and_limit_reduction_preserve_both_publications() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen><p>X",
    ));
    for (bytes, units) in [(0, 1_000_000), (16_000_000, 0)] {
        let (mut dom, environment) = setup();
        dom.set_limits(dom.limits().with_parsed_associations(bytes, units).unwrap())
            .unwrap();
        let before = dom.snapshot().stamp();
        let usage = dom.usage();
        assert!(matches!(
            dom.install_parsed_html_document(
                before,
                report.clone(),
                environment,
                limits(),
                &AtomicBool::new(false)
            ),
            Err(DomParsedHtmlInstallError::Target(DomError::ResourceLimit {
                resource: DomResource::ParsedAssociationBytes | DomResource::ParsedAssociationUnits,
                ..
            }))
        ));
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.pending_parsed_html_associations().count(), 0);
    }
    let (mut dom, environment) = setup();
    let before = dom.snapshot().stamp();
    assert!(matches!(
        dom.install_parsed_html_document(
            before,
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(true)
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    let installed = install(&mut dom, report, environment);
    let pending = installed.pending_association().unwrap();
    drop(installed);
    let before = dom.snapshot().stamp();
    let usage = dom.usage();
    assert!(matches!(
        dom.set_limits(dom.limits().with_parsed_associations(0, 0).unwrap()),
        Err(DomError::ResourceLimit { .. })
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        pending
    );
}

#[test]
fn pending_association_source_insertion_cannot_cross_pending_document_history() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen><p>X",
    ));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let document = installed.document();
    let head = element(&installed, "head");
    let id = installed.pending_association().unwrap();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let style = batch.create_element_ns(&other, Namespace::html(), "style");
    let created = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let style = created.created(&style).unwrap().unwrap();
    let mut source = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, head, None),
            DomSourceLimits::new(64_000_000, 16_000_000, 10_000_000, 1024, 4096, 4096, 1),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(
        matches!(source.advance(&AtomicBool::new(false)), Err(DomError::ParsedAssociationPending {document: actual, pending}) if actual == document && pending == id)
    );
    drop(source);
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        id
    );
    assert_eq!(
        preferred(
            dom.pending_parsed_html_associations()
                .next()
                .unwrap()
                .report()
        ),
        vec!["chosen".encode_utf16().collect::<Vec<_>>()]
    );
    assert_ne!(
        dom.snapshot()
            .node(style)
            .unwrap()
            .owner_document()
            .unwrap(),
        document
    );
}

#[test]
fn pending_association_native_meta_source_hold_refuses_before_missing_owner_and_preserves_history()
{
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen><p>X",
    ));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let document = installed.document();
    let head = element(&installed, "head");
    let pending = installed.pending_association().unwrap();
    drop(installed);
    dom.set_limits(dom.limits().with_meta_processing_work(100_000))
        .unwrap();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let meta = batch.create_element_ns(&other, Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "later");
    let created = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let other = created.document(&other).unwrap();
    let meta = created.created(&meta).unwrap().unwrap();
    let before = dom.snapshot().stamp();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(meta, head, None),
            DomSourceLimits::new(64_000_000, 16_000_000, 10_000_000, 1024, 4096, 4096, 1),
            &AtomicBool::new(false),
        )
        .unwrap();
    let operation = lease.operation();
    let mut hold = lease
        .begin_meta_runtime(
            DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 128).unwrap(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let error = match hold.prepare(&[], None, &AtomicBool::new(false)) {
        Err(error) => error,
        Ok(_) => panic!("native Meta cannot cross the original parsed association"),
    };
    assert!(
        matches!(error, DomMetaAtomicError::Dom(DomError::ParsedAssociationPending {
        document: actual, pending: actual_pending
    }) if actual == document && actual_pending == pending),
        "actual refusal: {error:?}"
    );
    drop(hold);
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    assert_eq!(dom.snapshot().node(meta).unwrap().parent(), None);
    assert_eq!(
        dom.snapshot().node(meta).unwrap().owner_document().unwrap(),
        other
    );
    let retained = dom.pending_parsed_html_associations().next().unwrap();
    assert_eq!(retained.id(), pending);
    assert_eq!(
        preferred(retained.report()),
        vec!["chosen".encode_utf16().collect::<Vec<_>>()]
    );
}

#[test]
fn pending_association_style_meta_style_order_and_template_documents_remain_original() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><style title=A>a { color: red }</style><meta http-equiv=default-style content=B><style title=C>c { color: blue }</style><template><style title=T>t { color: black }</style></template>",
    ));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let id = installed.pending_association().unwrap();
    let pending = dom.pending_parsed_html_associations().next().unwrap();
    assert_eq!(pending.id(), id);
    assert_eq!(preferred(pending.report()), vec![vec![66]]);
    let mut cursor = pending.report().pending_associations();
    let mut reached = Vec::new();
    loop {
        match cursor.advance() {
            Step::Occurrence(row) if row.requires_processing() => {
                assert!(row.cause().is_some());
                let context = row.context();
                let role = surgeist_html_parser::HtmlParseAssociationContextRole::DocumentUrl;
                assert_eq!(
                    context.requirement(role),
                    surgeist_html_parser::HtmlParseAssociationContextRequirement::OfflineUnbound
                );
                assert!(context.slot(role).is_some());
                assert!(context.captured_context().is_none());
                // The explicit destination URL is not an original parse-time
                // observation and cannot fill this historical binding slot.

                if let Some(prior) = row.prior_document() {
                    assert!(reached.contains(&prior));
                }
                reached.push(row.id());
                if let Some(style) = row.style() {
                    assert!(style.creating_parser_document().is_some());
                    if let Some(node) = pending.node_for(&row.subject().id()) {
                        assert!(
                            installed
                                .snapshot()
                                .node(node)
                                .unwrap()
                                .as_style()
                                .is_some()
                        );
                    }
                }
            }
            Step::Occurrence(row) => {
                reached.push(row.id());
            }
            Step::Complete => break,
            _ => {}
        }
    }
    let style = element(&installed, "style");
    let before = dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.set_style_disabled(style, true);
    assert!(
        matches!(dom.apply(batch, &AtomicBool::new(false)), Err(DomError::ParsedAssociationPending { pending, .. }) if pending == id)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        preferred(
            dom.pending_parsed_html_associations()
                .next()
                .unwrap()
                .report()
        ),
        vec![vec![66]]
    );
}

#[test]
fn pending_association_no_effect_contextual_meta_does_not_create_an_obligation() {
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let report = fragment_report(
        &capture,
        host,
        HtmlSource::utf8("<meta http-equiv=default-style content=chosen><p>X"),
        false,
    );
    let weak = Arc::downgrade(&report);
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let fragment = installed.root();
    assert_eq!(installed.document(), document);
    assert_eq!(installed.pending_association(), None);
    assert!(preferred(installed.report()).is_empty());
    drop(installed);
    drop(capture);
    assert!(weak.upgrade().is_none());
    dom.set_limits(dom.limits().with_meta_processing_work(100_000))
        .unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(fragment);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(dom.snapshot().node(fragment).is_err());
    assert_eq!(dom.pending_parsed_html_associations().count(), 0);
}

#[test]
fn pending_association_unsupported_stylesheet_producer_refuses_before_structural_publication() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><link rel=stylesheet href=sheet.css><meta http-equiv=default-style content=chosen>",
    ));
    let (mut dom, environment) = setup();
    let before = dom.snapshot().stamp();
    let usage = dom.usage();
    assert!(matches!(dom.install_parsed_html_document(before, report, environment, limits(), &AtomicBool::new(false)), Err(DomParsedHtmlInstallError::AssociationCaptureUnavailable { reason: surgeist_html_parser::HtmlParseAssociationMissingKind::UnsupportedLinkStylesheetHistory, .. })));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.pending_parsed_html_associations().count(), 0);
}

#[test]
fn pending_association_exact_utf16_preferred_input_survives_drop_without_reinterpretation() {
    let mut input: Vec<u16> = "<!doctype html><meta http-equiv=default-style content='"
        .encode_utf16()
        .collect();
    input.extend([0xd800, 0, 0x0061]);
    input.extend("'><p>X".encode_utf16());
    let report = parse(HtmlSource::utf16(DocumentString::from_utf16(input)));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    // HTML tokenization replaces a source null; the preserved history contains
    // the actual processed attribute input, including the lone surrogate.
    let expected = vec![vec![0xd800, 0xfffd, 0x0061]];
    assert_eq!(preferred(installed.report()), expected);
    let id = installed.pending_association().unwrap();
    drop(installed);
    let pending = dom.pending_parsed_html_associations().next().unwrap();
    assert_eq!(pending.id(), id);
    assert_eq!(preferred(pending.report()), expected);
}

#[test]
fn pending_association_actual_retained_utf16_cap_accepts_exact_and_refuses_one_short() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=default-style content=chosen>",
    ));
    let (mut baseline, environment) = setup();
    let observed = install(&mut baseline, report.clone(), environment.clone());
    let exact = baseline.usage().parsed_association_utf16_units();
    assert!(exact > 0);
    drop((observed, baseline));
    for units in [exact - 1, exact] {
        let (mut dom, _) = setup();
        dom.set_limits(
            dom.limits()
                .with_parsed_associations(64 * 1024 * 1024, units)
                .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot().stamp();
        let usage = dom.usage();
        let result = dom.install_parsed_html_document(
            before,
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(false),
        );
        if units == exact {
            assert!(result.unwrap().pending_association().is_some());
            assert_eq!(dom.usage().parsed_association_utf16_units(), exact);
        } else {
            assert!(
                matches!(result, Err(DomParsedHtmlInstallError::Target(DomError::ResourceLimit { resource: DomResource::ParsedAssociationUnits, limit })) if limit == units)
            );
            assert_eq!(dom.snapshot().stamp(), before);
            assert_eq!(dom.usage(), usage);
            assert_eq!(dom.pending_parsed_html_associations().count(), 0);
        }
    }
}

#[test]
fn pending_association_captured_shadow_style_retains_exact_context_basis_after_packet_drop() {
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let context_basis = capture.basis();
    let report = fragment_report(
        &capture,
        host,
        HtmlSource::utf8(
            "<template shadowrootmode=open><style title=chosen>a { color: red }</style></template>",
        ),
        true,
    );
    let weak = Arc::downgrade(&report);
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let id = installed
        .pending_association()
        .expect("actual connected captured shadow Style requires processing");
    assert_eq!(installed.context_basis(), Some(context_basis));
    let shadow = installed
        .snapshot()
        .node(host)
        .unwrap()
        .shadow_root()
        .unwrap();
    assert_eq!(
        installed.snapshot().node(shadow).unwrap().owner_document(),
        Some(document)
    );
    drop(installed);
    drop(capture);
    let pending = dom.pending_parsed_html_associations().next().unwrap();
    assert_eq!(pending.id(), id);
    assert_eq!(pending.context_basis(), Some(context_basis));
    assert!(
        pending
            .documents()
            .iter()
            .any(|row| row.document() == document)
    );
    let mut rows = pending.report().pending_associations();
    let mut captured = false;
    loop {
        match rows.advance() {
            Step::Occurrence(row) if row.requires_processing() => {
                if let surgeist_html_parser::HtmlParseMetadataDocument::Captured(original) =
                    row.document()
                {
                    assert_eq!(
                        pending.captured_node_for(&original.id()),
                        Some(document.node())
                    );
                    let role = surgeist_html_parser::HtmlParseAssociationContextRole::InitialPolicy;
                    assert!(row.context().captured_observation(role).is_some());
                    captured = true;
                }
            }
            Step::Complete => break,
            _ => {}
        }
    }
    assert!(captured);
    assert!(weak.upgrade().is_some());
}
