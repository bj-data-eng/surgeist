//! Original execution activation constrains real owner-wide event delivery.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, report};
use super::style_decision::{empty_policy, installed, observe, request_ready};
use super::style_sheet::{bind, ready};
use surgeist_dom::*;

struct FrontierContext {
    styles: DomStyleRuntime,
    parsed: DomParsedHtmlAssociationId,
    decision: Option<DomParsedHtmlStyleInlinePolicyObservation>,
    activated: Option<DomParsedHtmlStylePhase>,
    calls: Vec<&'static str>,
}
impl DomStyleEventContext for FrontierContext {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}
fn dispatcher() -> DomEventDispatcher<FrontierContext> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap()
}
fn options() -> DomEventListenerOptions {
    DomEventListenerOptions {
        capture: false,
        once: false,
    }
}
fn other_document(session: &DomSession, parsed: DocumentId) -> NodeId {
    session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.node != parsed.node())
        .unwrap()
        .node
}

#[test]
fn accepted_original_execution_survives_command_drop_and_blocks_other_document_task() {
    let (mut session, installation) =
        ready("<!doctype html><style>p{color:red}</style>", envelope());
    let parsed = installation.pending_association().unwrap();
    bind(&mut session, installation.document(), Default::default());
    // ready has accepted the genuine InitialEmpty allow decision and returned;
    // no original command loan remains to keep transient Control.active set.
    assert_eq!(
        session.parsed_html_style_prefix(parsed).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let native_document = other_document(&session, installation.document());
    let target = append_style(&mut session, native_document, None);
    let task = session.state.borrow().completions[0].task().id();
    let mut dispatcher = dispatcher();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut FrontierContext, _| {
                context.calls.push("native");
                Ok(())
            }),
            options(),
        )
        .unwrap();
    let mut context = FrontierContext {
        styles: session.style_execution(),
        parsed,
        decision: None,
        activated: None,
        calls: Vec::new(),
    };
    let stamp = session.snapshot().stamp();
    for _ in 0..2 {
        let outcome = DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel(),
        );
        assert!(
            context.calls.is_empty(),
            "activated original work in another Document must prevent callbacks; outcome={outcome:?}"
        );
        assert!(
            !matches!(outcome, Ok(DomStyleNetworkingProgress::Complete { .. })),
            "unfinished original execution cannot deliver or clean up a native task"
        );
        assert_eq!(session.snapshot().stamp(), stamp);
        let state = session.state.borrow();
        assert_eq!(state.completions.len(), 1);
        let retained = state.completions[0].task();
        assert_eq!(retained.id(), task);
        assert!(!retained.running);
        assert_eq!(retained.dispatch_reserved, 0);
        assert!(matches!(retained.delivery, Delivery::NotStarted));
        assert_eq!(
            retained.cursor.action(),
            HtmlStyleNotificationAction::DispatchLoad
        );
    }
}

#[test]
fn listener_activation_retains_dispatch_before_the_next_listener_and_never_repeats_first() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>p{color:red}</style>"));
    let parsed = installation.pending_association().unwrap();
    request_ready(&mut session, parsed, empty_policy(), envelope());
    bind(&mut session, installation.document(), Default::default());
    let decision = observe(&session, parsed, &mut |request| {
        assert!(request.text().is_empty());
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert!(decision.as_ref().unwrap().allowed());
    assert_eq!(
        session.parsed_html_style_prefix(parsed).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let native_document = other_document(&session, installation.document());
    let target = append_style(&mut session, native_document, None);
    let task = session.state.borrow().completions[0].task().id();
    let mut dispatcher = dispatcher();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|dom, _, context: &mut FrontierContext, _| {
                context.calls.push("A");
                let phase = context
                    .styles
                    .accept_parsed_style_inline_policy(
                        dom,
                        context.parsed,
                        &mut context.decision,
                        envelope(),
                        &cancel(),
                    )
                    .unwrap();
                assert_eq!(phase, DomParsedHtmlStylePhase::CreateSheetPending);
                context.activated = Some(phase);
                Ok(())
            }),
            options(),
        )
        .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut FrontierContext, _| {
                context.calls.push("B");
                Ok(())
            }),
            options(),
        )
        .unwrap();
    let mut context = FrontierContext {
        styles: session.style_execution(),
        parsed,
        decision,
        activated: None,
        calls: Vec::new(),
    };
    let outcome = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    );
    assert_eq!(
        context.activated,
        Some(DomParsedHtmlStylePhase::CreateSheetPending),
        "listener A must genuinely accept the original decision"
    );
    assert!(context.decision.is_none());
    assert_eq!(context.calls.iter().filter(|&&name| name == "A").count(), 1);
    assert_eq!(
        context.calls.iter().filter(|&&name| name == "B").count(),
        0,
        "activation inside listener A must gate listener B; outcome={outcome:?}"
    );
    assert!(!matches!(
        outcome,
        Ok(DomStyleNetworkingProgress::Complete { .. })
    ));
    assert_eq!(session.state.borrow().completions.len(), 1);
    assert_eq!(session.state.borrow().completions[0].task().id(), task);
    assert!(matches!(
        session.state.borrow().completions[0].task().delivery,
        Delivery::Dispatching(_)
    ));
    let resumed = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    );
    assert_eq!(
        context.calls,
        vec!["A"],
        "the retained continuation must neither repeat A nor invoke B; outcome={resumed:?}"
    );
    finish_allowed_history(&mut session, parsed);
    let completed = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap();
    assert!(
        matches!(completed, DomStyleNetworkingProgress::Complete { task: actual, .. } if actual == task)
    );
    assert_eq!(context.calls, ["A", "B"]);
}

#[test]
fn forgotten_unstarted_request_guard_allows_other_document_delivery_but_not_its_mutation() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>p{color:red}</style>"));
    let parsed = installation.pending_association().unwrap();
    request_ready(&mut session, parsed, empty_policy(), envelope());
    let element = session
        .snapshot()
        .nodes()
        .find(|node| session.snapshot().node(*node).unwrap().as_style().is_some())
        .unwrap();
    // The original view's mutex is deliberately leaked. Header classification
    // must not try to acquire it for a never-activated contribution.
    std::mem::forget(session.parsed_html_style_prefix(parsed).unwrap());
    let native_document = other_document(&session, installation.document());
    let target = append_style(&mut session, native_document, None);
    let mut dispatcher = dispatcher();
    dispatcher.add_listener(&mut session.dom(), target, DomEventType::Load,
        DomEventCallback::new(move |dom, _, context: &mut FrontierContext, _| {
            let mut batch = DomBatch::new();
            batch.set_attribute(element, "title", "attempt");
            assert!(matches!(dom.apply(batch, &cancel()), Err(DomError::ParsedAssociationPending { pending, .. }) if pending == parsed));
            context.calls.push("native");
            Ok(())
        }), options()).unwrap();
    let mut context = FrontierContext {
        styles: session.style_execution(),
        parsed,
        decision: None,
        activated: None,
        calls: Vec::new(),
    };
    let output = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap();
    assert!(matches!(
        output,
        DomStyleNetworkingProgress::Complete { .. }
    ));
    assert_eq!(context.calls, ["native"]);
    assert!(
        session
            .dom
            .parsed_html_synchronous_end(parsed)
            .unwrap()
            .is_none()
    );
}

fn finish_allowed_history(session: &mut DomSession, parsed: DomParsedHtmlAssociationId) {
    fn render(session: &mut DomSession, q: &DomParsedHtmlStyleQualification) {
        session
            .style_execution()
            .exclude_parsed_style_script_bookkeeping(&mut session.dom(), q, envelope(), &cancel())
            .unwrap();
        assert!(matches!(
            session
                .style_execution()
                .process_parsed_style_rendering(&mut session.dom(), q, envelope(), &cancel())
                .unwrap(),
            DomParsedHtmlStyleRenderingProgress::Complete(_)
        ));
    }
    let first = session
        .parsed_html_style_prefix(parsed)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    drop(super::style_sheet::create(session, parsed, envelope()).unwrap());
    render(session, &first);
    let next = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), parsed, envelope(), &cancel())
        .unwrap();
    let q = next.qualification().clone();
    drop(next);
    let mut input = observe(session, parsed, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            parsed,
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    drop(super::style_sheet::create(session, parsed, envelope()).unwrap());
    render(session, &q);
    drop(
        session
            .style_execution()
            .complete_parsed_html_synchronous_execution(
                &mut session.dom(),
                parsed,
                envelope(),
                &cancel(),
            )
            .unwrap(),
    );
}

#[test]
fn delivered_cleanup_retains_receipt_across_work_refusal_activation_and_real_end() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>p{color:red}</style>"));
    let parsed = installation.pending_association().unwrap();
    request_ready(&mut session, parsed, empty_policy(), envelope());
    bind(&mut session, installation.document(), Default::default());
    let decision = observe(&session, parsed, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    let native_document = other_document(&session, installation.document());
    let target = append_style(&mut session, native_document, None);
    let task = session.state.borrow().completions[0].task().id();
    let mut dispatcher = dispatcher();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|dom, _, context: &mut FrontierContext, _| {
                context.calls.push("delivered");
                // Measure the real unchanged policy-admission walk, then set an
                // exact cumulative cap. The event's independent delivery budget
                // can finish; subsequent Runtime cleanup must genuinely refuse.
                let owner = context.styles.original().unwrap();
                let before = owner.borrow().work.get();
                context.styles.set_limits(dom, limits(), &cancel()).unwrap();
                let cost = owner.borrow().work.get() - before;
                let mut cap = limits();
                cap.work = owner.borrow().work.get().checked_add(cost).unwrap();
                context.styles.set_limits(dom, cap, &cancel()).unwrap();
                assert_eq!(owner.borrow().work.get(), cap.work);
                Ok(())
            }),
            options(),
        )
        .unwrap();
    let mut context = FrontierContext {
        styles: session.style_execution(),
        parsed,
        decision,
        activated: None,
        calls: Vec::new(),
    };
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel()
        ),
        Err(DomStyleRuntimeError::Admission)
    ));
    assert_eq!(context.calls, ["delivered"]);
    assert!(matches!(
        session.state.borrow().completions[0].task().delivery,
        Delivery::Delivered(_)
    ));
    context
        .styles
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    context
        .styles
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            parsed,
            &mut context.decision,
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher, &mut context, dispatch_limits(), &cancel()).unwrap(), DomStyleNetworkingProgress::OriginalSynchronousPending { task: actual, .. } if actual == task)
    );
    assert!(matches!(
        session.state.borrow().completions[0].task().delivery,
        Delivery::Delivered(_)
    ));
    finish_allowed_history(&mut session, parsed);
    let DomStyleNetworkingProgress::Complete {
        task: actual,
        receipt,
    } = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
    else {
        panic!("real end permits cleanup")
    };
    assert_eq!(actual, task);
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(context.calls, ["delivered"]);
}

#[test]
fn activated_forgotten_read_guard_retains_exact_admission_error_without_listener_replay() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>p{color:red}</style>"));
    let parsed = installation.pending_association().unwrap();
    request_ready(&mut session, parsed, empty_policy(), envelope());
    bind(&mut session, installation.document(), Default::default());
    let decision = observe(&session, parsed, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    let native_document = other_document(&session, installation.document());
    let target = append_style(&mut session, native_document, None);
    let mut dispatcher = dispatcher();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|dom, _, context: &mut FrontierContext, _| {
                context.calls.push("A");
                context
                    .styles
                    .accept_parsed_style_inline_policy(
                        dom,
                        context.parsed,
                        &mut context.decision,
                        envelope(),
                        &cancel(),
                    )
                    .unwrap();
                std::mem::forget(dom.raw().parsed_html_style_prefix(context.parsed).unwrap());
                Ok(())
            }),
            options(),
        )
        .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut FrontierContext, _| {
                context.calls.push("B");
                Ok(())
            }),
            options(),
        )
        .unwrap();
    let mut context = FrontierContext {
        styles: session.style_execution(),
        parsed,
        decision,
        activated: None,
        calls: Vec::new(),
    };
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher, &mut context, dispatch_limits(), &cancel()).unwrap(),
        DomStyleNetworkingProgress::Retained { dispatch: DomEventDispatchOutcome::RetainedParsedAdmission(DomError::ParsedAssociationOperationPending { pending }), .. } if pending == parsed)
    );
    assert_eq!(context.calls, ["A"]);
    // This deliberately forgotten actual loan is unrecoverable. Refusal keeps
    // both its custody and the original event's already-advanced continuation.
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher, &mut context, dispatch_limits(), &cancel()),
        Err(DomStyleRuntimeError::Dom(DomError::ParsedAssociationOperationPending { pending })) if pending == parsed)
    );
    assert_eq!(context.calls, ["A"]);
    assert!(matches!(
        session.state.borrow().completions[0].task().delivery,
        Delivery::Dispatching(_)
    ));
}
