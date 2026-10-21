//! Original rendering uses retained media/context and publishes real membership.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::envelope;
use super::style_sheet::{bind, create, pending, ready};
use surgeist_dom::*;

fn request(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleQualification {
    session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone()
}
fn exclude(
    session: &mut DomSession,
    qualification: &DomParsedHtmlStyleQualification,
    limits: DomParsedHtmlAssociationLimits,
) {
    assert_eq!(
        session
            .style_execution()
            .exclude_parsed_style_script_bookkeeping(
                &mut session.dom(),
                qualification,
                limits,
                &cancel(),
            )
            .unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
}
fn render(
    session: &mut DomSession,
    qualification: &DomParsedHtmlStyleQualification,
    limits: DomParsedHtmlAssociationLimits,
) -> Result<DomParsedHtmlStyleRenderingProgress, DomStyleParsedPrefixError> {
    session.style_execution().process_parsed_style_rendering(
        &mut session.dom(),
        qualification,
        limits,
        &cancel(),
    )
}

#[test]
fn original_head_media_defaults_publish_membership_once_without_recreating_sheet_or_preference() {
    for media in ["", " media=''", " media='&#9;&#10;&#12;&#13; '"] {
        let text = format!(
            "<!doctype html><head><style title=chosen{media}>x{{}}</style></head><body>final body"
        );
        let (mut session, installed) = ready(&text, envelope());
        let id = installed.pending_association().unwrap();
        let document = installed.document();
        let final_tree = session.snapshot();
        assert!(final_tree.nodes().any(|id| {
            let node = final_tree.node(id).unwrap();
            node.owner_document() == Some(document)
                && node.as_element().is_some_and(|element| {
                    element.name().local_name() == &DocumentString::from("body")
                })
        }));
        bind(&mut session, document, Default::default());
        let qualification = request(&session, id);
        let created = create(&mut session, id, envelope()).unwrap();
        let element = created.completion().sheet().association().element();
        assert_eq!(installed.node_for(qualification.subject()), Some(element));
        exclude(&mut session, &qualification, envelope());
        let names = session
            .style_execution()
            .sheet_sets(&session.dom(), document.node(), &cancel())
            .unwrap();
        let stamp = session.snapshot().stamp();
        let DomParsedHtmlStyleRenderingProgress::Complete(completed) =
            render(&mut session, &qualification, envelope()).unwrap()
        else {
            panic!("unconditional original media must complete")
        };
        assert_eq!(completed.qualification(), &qualification);
        assert_eq!(completed.element(), element);
        assert_eq!(
            completed.phase(),
            DomParsedHtmlStylePhase::RenderingComplete
        );
        assert!(completed.render_blocking_member());
        assert_ne!(session.snapshot().stamp(), stamp);
        assert!(
            completed
                .dom()
                .changes()
                .changes()
                .contains(&DomChange::DocumentRenderBlocking(document))
        );
        let view = session.parsed_html_style_prefix(id).unwrap();
        assert_eq!(view.qualification(), Some(&qualification));
        assert_eq!(view.phase(), DomParsedHtmlStylePhase::RenderingComplete);
        assert_eq!(
            view.script_bookkeeping_exclusion(),
            Some(HtmlStyleExecutionProfile::NoExecutableJavaScript)
        );
        drop(view);
        let after = session
            .style_execution()
            .sheet_sets(&session.dom(), document.node(), &cancel())
            .unwrap();
        assert_eq!(after.revision(), names.revision());
        assert_eq!(after.preferred_name(), names.preferred_name());
        assert_eq!(after.cssom_revision(), names.cssom_revision());
        let cssom = session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap();
        assert_eq!(cssom.sheets(), &[created.completion().sheet().id().clone()]);
        assert_eq!(cssom.revision(), created.completion().sheet().revision());
        let snapshot = session.snapshot();
        let style = snapshot.node(element).unwrap().as_style().unwrap();
        assert!(matches!(
            style.is_render_blocking(),
            Err(DomError::StyleProcessingUnavailable)
        ));
        assert!(matches!(
            style.sheet(),
            Err(DomError::StyleProcessingUnavailable)
        ));
        assert!(render(&mut session, &qualification, envelope()).is_err());
        assert!(create(&mut session, id, envelope()).is_err());
        pending(&session, id);
        // The real returned completion and its retirement remain owned while
        // the unchanged canonical contribution is queried and retries refuse.
        drop(completed);
    }
}

#[test]
fn original_body_presence_completes_without_membership_despite_parser_potential() {
    let (mut session, installed) = ready("<!doctype html><body><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, envelope()).unwrap());
    let qualification = request(&session, id);
    exclude(&mut session, &qualification, envelope());
    let DomParsedHtmlStyleRenderingProgress::Complete(completed) =
        render(&mut session, &qualification, envelope()).unwrap()
    else {
        panic!("known original body context")
    };
    assert!(!completed.render_blocking_member());
    assert!(
        !completed
            .dom()
            .changes()
            .changes()
            .contains(&DomChange::DocumentRenderBlocking(installed.document()))
    );
    assert_eq!(
        completed.phase(),
        DomParsedHtmlStylePhase::RenderingComplete
    );
    pending(&session, id);
}

#[test]
fn nontrivial_original_media_requires_environment_without_advancing_or_publishing() {
    for media in ["screen", "&#160;"] {
        let (mut session, installed) = ready(
            &format!("<!doctype html><head><style media='{media}'>x{{}}</style></head>"),
            envelope(),
        );
        let id = installed.pending_association().unwrap();
        bind(&mut session, installed.document(), Default::default());
        let created = create(&mut session, id, envelope()).unwrap();
        let qualification = request(&session, id);
        exclude(&mut session, &qualification, envelope());
        let stamp = session.snapshot().stamp();
        for _ in 0..2 {
            assert!(matches!(
                render(&mut session, &qualification, envelope()).unwrap(),
                DomParsedHtmlStyleRenderingProgress::Pending(
                    DomParsedHtmlStyleRenderingRequirement::MediaEnvironment
                )
            ));
        }
        assert_eq!(session.snapshot().stamp(), stamp);
        assert_eq!(
            session.parsed_html_style_prefix(id).unwrap().phase(),
            DomParsedHtmlStylePhase::ReadRenderBlockingPending
        );
        assert_eq!(
            session
                .style_execution()
                .snapshot(installed.document().node())
                .unwrap()
                .unwrap()
                .revision(),
            created.completion().sheet().revision()
        );
        pending(&session, id);
    }
}

#[test]
fn rendering_output_refusal_preserves_phase_and_membership_for_raised_retry() {
    let large = envelope();
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        7,
    )
    .unwrap();
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", finite);
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    let created = create(&mut session, id, finite).unwrap();
    let qualification = request(&session, id);
    exclude(&mut session, &qualification, finite);
    let stamp = session.snapshot().stamp();
    assert!(matches!(
        render(&mut session, &qualification, finite),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 7
                }
            ))
        ))
    ));
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    let DomParsedHtmlStyleRenderingProgress::Complete(completed) =
        render(&mut session, &qualification, large).unwrap()
    else {
        panic!("raised original envelope")
    };
    assert!(completed.render_blocking_member());
    assert_eq!(
        session
            .style_execution()
            .snapshot(installed.document().node())
            .unwrap()
            .unwrap()
            .revision(),
        created.completion().sheet().revision()
    );
    pending(&session, id);
}

#[test]
fn rendering_checks_actual_session_phase_cancellation_and_cumulative_original_work() {
    let large = envelope();
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        8192,
        large.outputs(),
    )
    .unwrap();
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", finite);
    let id = installed.pending_association().unwrap();
    let qualification = request(&session, id);
    assert!(matches!(
        render(&mut session, &qualification, finite),
        Err(DomStyleParsedPrefixError::ProcessingUnavailable { contribution }) if contribution == id
    ));
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, finite).unwrap());
    exclude(&mut session, &qualification, finite);
    let runtime = session.style_execution();
    let (mut foreign, _) = bound_dispatch_session();
    assert!(matches!(
        runtime.process_parsed_style_rendering(
            &mut foreign.dom(),
            &qualification,
            finite,
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        runtime.process_parsed_style_rendering(
            &mut session.dom(),
            &qualification,
            finite,
            &AtomicBool::new(true)
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    {
        let mut dom = session.dom();
        let hold = dom
            .raw_mut()
            .begin_parsed_html_style(id, finite, &cancel())
            .unwrap();
        loop {
            if let Err(error) = hold.step(&cancel()) {
                assert!(matches!(
                    error,
                    DomError::WorkLimit {
                        resource: DomResource::ParsedAssociationProcessingWork,
                        limit: 8192
                    }
                ));
                break;
            }
        }
    }
    assert!(render(&mut session, &qualification, finite).is_err());
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    assert!(matches!(
        render(&mut session, &qualification, large).unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    pending(&session, id);
}

#[test]
fn source_tail_and_forgotten_canonical_loan_prevent_rendering_publication() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, envelope()).unwrap());
    let qualification = request(&session, id);
    exclude(&mut session, &qualification, envelope());
    let snapshot = session.snapshot();
    let other_document = snapshot
        .nodes()
        .find(|node| {
            *node != installed.document().node()
                && matches!(
                    snapshot.node(*node).unwrap().kind(),
                    DomNodeKind::Document(_)
                )
        })
        .unwrap();
    let target = snapshot.node(other_document).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let native_style = batch.create_element_ns(other_document, Namespace::html(), "style");
    let setup = session.dom().apply(batch, &cancel()).unwrap();
    let native_style = setup.created(&native_style).unwrap().unwrap();
    let mut source = session.source_execution();
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(native_style, target, None),
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap(),
        DomSourceDriveProgress::Pending {
            reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
            ..
        }
    ));
    let operation = session.dom().pending_source_edit().unwrap().operation();
    assert!(matches!(render(&mut session, &qualification, envelope()),
        Err(DomStyleParsedPrefixError::Runtime(DomStyleRuntimeError::SourcePending(actual)))
            if actual == operation));
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                operation,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel(),
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    let stamp = session.snapshot().stamp();
    std::mem::forget(session.parsed_html_style_prefix(id).unwrap());
    assert!(matches!(render(&mut session, &qualification, envelope()),
        Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::Busy {contribution}))
            if contribution == id));
    assert_eq!(session.snapshot().stamp(), stamp);
    pending(&session, id);
}
