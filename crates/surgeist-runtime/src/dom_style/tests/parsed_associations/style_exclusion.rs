//! The actual no-engine Session projects only executable-script bookkeeping out.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::envelope;
use super::style_sheet::{bind, create, pending, ready};
use surgeist_dom::*;

fn qualification(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInitialPolicyRequest {
    session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone()
}
fn exclude(
    session: &mut DomSession,
    request: &DomParsedHtmlStyleInitialPolicyRequest,
    limits: DomParsedHtmlAssociationLimits,
) -> Result<DomParsedHtmlStylePhase, DomStyleParsedPrefixError> {
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), request, limits, &cancel())
}

#[test]
fn no_engine_exclusion_preserves_original_sheet_and_reaches_unresolved_render_context_once() {
    let (mut session, installed) = ready(
        "<!doctype html><style title=chosen media=screen>x{color:red}</style>",
        envelope(),
    );
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    bind(&mut session, document, Default::default());
    let original = qualification(&session, id);
    let created = create(&mut session, id, envelope()).unwrap();
    let request = qualification(&session, id);
    assert_eq!(request, original);
    assert_eq!(
        installed.node_for(request.subject()),
        Some(created.completion().sheet().association().element())
    );
    let stamp = session.snapshot().stamp();
    let before = session
        .style_execution()
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(
        exclude(&mut session, &request, envelope()).unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    let view = session.parsed_html_style_prefix(id).unwrap();
    assert_eq!(view.initial_policy_request(), Some(&request));
    assert_eq!(
        view.script_bookkeeping_exclusion(),
        Some(HtmlStyleExecutionProfile::NoExecutableJavaScript)
    );
    assert!(view.inline_policy_request().is_none());
    drop(view);
    // Media is screen and no environment was bound. The excluded predicate
    // cannot keep this phase pending or supply a render-match result.
    assert_eq!(session.snapshot().stamp(), stamp);
    let after = session
        .style_execution()
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.cssom_revision(), before.cssom_revision());
    assert_eq!(after.preferred_name(), before.preferred_name());
    let cssom = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(cssom.sheets(), &[created.completion().sheet().id().clone()]);
    assert!(matches!(
        exclude(&mut session, &request, envelope()),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::ReadRenderBlockingPending
            }
        ))
    ));
    assert!(create(&mut session, id, envelope()).is_err());
    assert_eq!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .revision(),
        cssom.revision()
    );
    let snapshot = session.snapshot();
    let style = snapshot
        .node(created.completion().sheet().association().element())
        .unwrap()
        .as_style()
        .unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    assert!(matches!(
        style.sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    pending(&session, id);
}

#[test]
fn exclusion_output_refusal_and_raised_retry_preserve_the_original_created_sheet() {
    let large = envelope();
    // Request, decision, Runtime result/processing receipt and joined completion/
    // retirement consume six original outputs before this disposition is admitted.
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        6,
    )
    .unwrap();
    let (mut session, installed) = ready("<!doctype html><style title=chosen>x{}</style>", finite);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    bind(&mut session, document, Default::default());
    let created = create(&mut session, id, finite).unwrap();
    let request = qualification(&session, id);
    let stamp = session.snapshot().stamp();
    assert!(matches!(
        exclude(&mut session, &request, finite),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 6
                }
            ))
        ))
    ));
    {
        let view = session.parsed_html_style_prefix(id).unwrap();
        assert_eq!(
            view.phase(),
            DomParsedHtmlStylePhase::ReadScriptBlockingPending
        );
        assert_eq!(view.script_bookkeeping_exclusion(), None);
    }
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        exclude(&mut session, &request, large).unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    assert_eq!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .revision(),
        created.completion().sheet().revision()
    );
    pending(&session, id);
}

#[test]
fn wrong_phase_owner_and_cancellation_cannot_supply_an_exclusion() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    let request = qualification(&session, id);
    assert!(matches!(
        exclude(&mut session, &request, envelope()),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::CreateSheetPending
            }
        ))
    ));
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, envelope()).unwrap());
    let runtime = session.style_execution();
    let (mut foreign, _) = bound_dispatch_session();
    assert!(matches!(
        runtime.exclude_parsed_style_script_bookkeeping(
            &mut foreign.dom(),
            &request,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        runtime.exclude_parsed_style_script_bookkeeping(
            &mut session.dom(),
            &request,
            envelope(),
            &AtomicBool::new(true)
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    assert_eq!(
        exclude(&mut session, &request, envelope()).unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    pending(&session, id);
}

#[test]
fn original_work_exhaustion_survives_hold_drop_then_raise_resumes_the_same_cursor() {
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
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, finite).unwrap());
    let request = qualification(&session, id);
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
    assert!(exclude(&mut session, &request, finite).is_err());
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    assert_eq!(
        exclude(&mut session, &request, large).unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    pending(&session, id);
}

#[test]
fn source_owner_and_forgotten_canonical_loan_keep_the_exact_phase_unconsumed() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, envelope()).unwrap());
    let request = qualification(&session, id);
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
                    url: None
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
    assert!(
        matches!(exclude(&mut session, &request, envelope()), Err(DomStyleParsedPrefixError::Runtime(DomStyleRuntimeError::SourcePending(actual))) if actual==operation)
    );
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
                    url: None
                },
                &cancel(),
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    std::mem::forget(session.parsed_html_style_prefix(id).unwrap());
    assert!(
        matches!(exclude(&mut session, &request, envelope()), Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::Busy { contribution })) if contribution==id)
    );
    pending(&session, id);
}
