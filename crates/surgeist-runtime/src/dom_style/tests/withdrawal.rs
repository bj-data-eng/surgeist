//! Actual hosted cleanup, retained root history and finite upper-owner admission.
use super::*;
use crate::{
    DomHostedDestructionQueued, DomHostedRuntimeError, DomHostedTaskProgress, HostedRuntime,
};
use surgeist_dom::{DocumentString, DomHostedError, DomMetaAtomicLimits, DomStyleSheet};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
struct Fixture {
    session: DomSession,
    styles: DomStyleRuntime,
    hosted: HostedRuntime,
    top: crate::DomHostedTopLevel,
    queued: DomHostedDestructionQueued,
    style: NodeId,
    sheet: DomStyleSheet,
}
fn drive(
    session: &mut DomSession,
    edit: DomSourceEdit,
    policy: &mut impl FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
) {
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: policy,
                    url: None
                },
                &cancel()
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
}
impl Fixture {
    fn new(preferred: DocumentString) -> Self {
        let mut dom = dispatch_dom();
        let dom_limits = dom.limits().with_meta_processing_work(1_000_000);
        dom.set_limits(dom_limits).unwrap();
        let mut session = DomSession::new(dom, session_limits(limits())).unwrap();
        let mut hosted = session.hosted_execution();
        let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
        let top = hosted
            .create_top_level(&mut session.dom(), host, &cancel())
            .unwrap();
        let mut styles = session.style_execution();
        styles
            .bind_root(
                &session.dom(),
                "withdrawal-history".into(),
                &mut Some(DomStyleCssomRoot::new(
                    top.document().node(),
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel(),
            )
            .unwrap();
        let capture = session.snapshot();
        let html = capture.node(top.document().node()).unwrap().children()[0];
        let parent = capture.node(html).unwrap().children()[1];
        let mut setup = DomBatch::new();
        let style = setup.create_element_ns(top.document(), Namespace::html(), "style");
        setup.set_attribute(&style, "title", "A");
        setup.set_attribute(&style, "data-probe", "same");
        let text = setup.create_text(top.document(), "a { color: red }");
        setup.insert_before(&text, &style, None);
        let style = session
            .dom()
            .apply(setup, &cancel())
            .unwrap()
            .created(&style)
            .unwrap()
            .unwrap();
        drive(
            &mut session,
            DomSourceEdit::insert_before(style, parent, None),
            &mut policy,
        );
        let sheet = session
            .snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .clone();
        let before = styles
            .sheet_sets(&session.dom(), top.document().node(), &cancel())
            .unwrap();
        let selected = styles
            .select_sheet_set(&session.dom(), &before, &"A".into(), 1, &cancel())
            .unwrap();
        styles
            .change_preferred_sheet_set_name(
                &session.dom(),
                selected.observation(),
                &preferred,
                1,
                &cancel(),
            )
            .unwrap();
        drive(&mut session, DomSourceEdit::detach(style), &mut |_| None);
        let mut context = Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        };
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 1,
            pending_events: 1,
            nested_dispatches: 1,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                style,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, context: &mut Context, event| {
                    context.events.push((style, event.target().unwrap()));
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &cancel()
            )
            .unwrap(),
            DomStyleNetworkingProgress::Complete { .. }
        ));
        assert_eq!(context.events, [(style, style)]);
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &cancel()
            )
            .unwrap(),
            DomStyleNetworkingProgress::Empty
        ));
        let queued = hosted
            .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
            .unwrap();
        let mut reached = false;
        for _ in 0..256 {
            match hosted.run_next(&mut session.dom(), queued.traversable(), &cancel()) {
                Err(DomHostedRuntimeError::Dom(DomHostedError::PendingDocumentObligations)) => {
                    reached = true;
                    break;
                }
                Ok(DomHostedTaskProgress::Pending { .. }) => (),
                other => panic!("authentic idle root must retain original cleanup: {other:?}"),
            }
        }
        assert!(reached);
        Self {
            session,
            styles,
            hosted,
            top,
            queued,
            style,
            sheet,
        }
    }
    fn withdraw(&mut self) -> Result<DomStyleRootWithdrawal, DomStyleRootWithdrawalError> {
        self.styles.withdraw_root_for_hosted_destruction(
            &mut self.session.dom(),
            &self.hosted,
            self.queued.operation(),
            self.top.document().node(),
            &cancel(),
        )
    }
}
fn frame_bytes() -> usize {
    (size_of::<DomStyleRootWithdrawal>() - size_of::<surgeist_dom::DomHostedStyleRootWithdrawal>())
        + size_of::<std::rc::Rc<DomStyleCssomRoot>>()
        + 2 * size_of::<RootResource>()
}

#[test]
fn exact_upper_frame_refusal_retry_preserves_original_root_names_serial_and_releases_only_its_capability()
 {
    let large = DocumentString::from_utf16(vec![66; 16_384]);
    let mut f = Fixture::new(large.clone());
    let owner = f.styles.original().unwrap();
    let (original_limits, storage, label, serial, names, weak, spent) = {
        let state = owner.borrow();
        let root = &state.roots[0];
        (
            state.limits,
            state.storage(None, &cancel(), state.limits.work).unwrap(),
            root.label.clone(),
            root.serial,
            root.sheet_sets
                .state()
                .unwrap()
                .names()
                .map(|n| n.shared_code_units().clone())
                .collect::<Vec<_>>(),
            std::rc::Rc::downgrade(root.bound().unwrap()),
            state.work.get(),
        )
    };
    let stamp = f.session.snapshot().stamp();
    let mut tight = original_limits;
    tight.storage_bytes = storage + frame_bytes() - 1;
    f.styles
        .set_limits(&f.session.dom(), tight, &cancel())
        .unwrap();
    assert!(matches!(
        f.withdraw(),
        Err(DomStyleRootWithdrawalError::Style(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert!(
        f.session
            .dom()
            .style_cssom_root(f.top.document().node())
            .unwrap()
            .is_some()
    );
    assert!(owner.borrow().work.get() > spent);
    assert!(weak.upgrade().is_some());
    assert_eq!(f.session.snapshot().stamp(), stamp);
    tight.storage_bytes = storage + frame_bytes();
    f.styles
        .set_limits(&f.session.dom(), tight, &cancel())
        .unwrap();
    let output = f.withdraw().unwrap();
    assert!(
        weak.upgrade().is_none(),
        "actual Runtime Rc capability target no longer survives through released history"
    );
    assert_eq!(output.document_sheet_sets().unwrap().preferred(), &large);
    assert_eq!(
        output.document_sheet_sets().unwrap().last_selected(),
        Some(&DocumentString::from("A"))
    );
    {
        let state = owner.borrow();
        let root = &state.roots[0];
        assert_eq!(root.label, label);
        assert_eq!(root.serial, serial);
        for (actual, old) in root.sheet_sets.state().unwrap().names().zip(&names) {
            assert!(Arc::ptr_eq(actual.shared_code_units(), old));
        }
        assert!(
            matches!(&root.resource,RootResource::ReleasedForHostedDestruction { cleanup,at_withdrawal } if *cleanup==output.cleanup_operation() && at_withdrawal==output.withdrawal_cssom_snapshot().revision())
        );
        assert_eq!(
            state.storage(None, &cancel(), state.limits.work).unwrap(),
            storage - size_of::<DomStyleCssomRoot>()
        );
    }
    assert_eq!(f.session.snapshot().stamp(), stamp);
    drop(output);
    let state = owner.borrow();
    assert_eq!(state.roots[0].label, label);
    assert_eq!(state.roots[0].serial, serial);
    assert_eq!(
        state.roots[0].sheet_sets.state().unwrap().preferred(),
        &large
    );
    assert_eq!(
        state.storage(None, &cancel(), state.limits.work).unwrap(),
        storage - size_of::<DomStyleCssomRoot>()
    );
}

#[test]
fn late_original_upper_work_refusal_and_real_borrowed_owners_preserve_same_cleanup_for_raised_retry()
 {
    let mut reference = Fixture::new("B".into());
    let reference_owner = reference.styles.original().unwrap();
    let before = reference_owner.borrow().work.get();
    drop(reference.withdraw().unwrap());
    let cost = reference_owner.borrow().work.get() - before;
    let mut f = Fixture::new("B".into());
    let owner = f.styles.original().unwrap();
    let baseline = f.styles.snapshot(f.top.document().node()).unwrap().unwrap();
    let held = owner.borrow_mut();
    assert!(matches!(
        f.withdraw(),
        Err(DomStyleRootWithdrawalError::Style(
            DomStyleRuntimeError::Binding
        ))
    ));
    drop(held);
    assert!(matches!(
        f.styles.withdraw_root_for_hosted_destruction(
            &mut f.session.dom(),
            &f.hosted,
            f.queued.operation(),
            f.top.document().node(),
            &AtomicBool::new(true)
        ),
        Err(DomStyleRootWithdrawalError::Hosted(
            DomHostedRuntimeError::Cancelled
        ))
    ));
    let maximum = owner.borrow().limits;
    let before = owner.borrow().work.get();
    f.styles
        .set_limits(&f.session.dom(), maximum, &cancel())
        .unwrap();
    let setter = owner.borrow().work.get() - before;
    let spent = owner.borrow().work.get();
    let mut low = maximum;
    low.work = spent + setter + cost - 1;
    f.styles
        .set_limits(&f.session.dom(), low, &cancel())
        .unwrap();
    assert!(matches!(
        f.withdraw(),
        Err(DomStyleRootWithdrawalError::Style(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert!(owner.borrow().work.get() > spent);
    assert!(
        f.session
            .dom()
            .style_cssom_root(f.top.document().node())
            .unwrap()
            .is_some()
    );
    assert_eq!(
        f.styles
            .snapshot(f.top.document().node())
            .unwrap()
            .unwrap()
            .revision(),
        baseline.revision()
    );
    f.styles
        .set_limits(&f.session.dom(), maximum, &cancel())
        .unwrap();
    let output = f.withdraw().unwrap();
    assert_eq!(output.invocation(), f.queued.operation());
    assert_eq!(
        output.withdrawal_cssom_snapshot().revision(),
        baseline.revision()
    );
}

#[test]
fn later_bound_root_meta_writes_use_actual_projection_mapping_without_reactivating_released_history()
 {
    let mut f = Fixture::new("B".into());
    let output = f.withdraw().unwrap();
    let released = f.top.document().node();
    let history = output.document_sheet_sets().unwrap().revision();
    let other = meta::bind_document(&mut f.session, Default::default());
    let style = meta::named_style(&mut f.session, other, "A".into());
    let parent = meta::container(&f.session, other);
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(other, Namespace::html(), "meta");
    batch.set_attribute(&element, "http-equiv", "default-style");
    batch.set_attribute(&element, "content", "Z");
    batch.insert_before(&element, parent, None);
    let result = f
        .styles
        .apply_meta(
            &mut f.session.dom(),
            batch,
            None,
            meta::envelope(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(result.preferences().len(), 1);
    assert_eq!(result.preferences()[0].observation().document(), other);
    assert_eq!(
        result.preferences()[0].observation().preferred_name(),
        &DocumentString::from("Z")
    );
    assert!(
        f.styles
            .disabled(&f.session.dom(), style)
            .unwrap()
            .disabled()
    );
    // The next actual Source projection also skips the unrelated released row.
    let mut setup = DomBatch::new();
    let element = setup.create_element_ns(other, Namespace::html(), "meta");
    setup.set_attribute(&element, "http-equiv", "default-style");
    setup.set_attribute(&element, "content", "A");
    let element = f
        .session
        .dom()
        .apply(setup, &cancel())
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap();
    let progress = f
        .session
        .source_execution()
        .apply(
            &mut f.session.dom(),
            DomSourceEdit::insert_before(element, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: Some(meta::envelope()),
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(progress, DomSourceDriveProgress::MetaSource(_)));
    assert!(
        !f.styles
            .disabled(&f.session.dom(), style)
            .unwrap()
            .disabled()
    );
    let owner = f.styles.original().unwrap();
    let state = owner.borrow();
    assert_eq!(state.roots[0].node, released);
    assert_eq!(state.roots[1].node, other);
    assert_eq!(
        state.roots[0].sheet_sets.state().unwrap().revision(),
        history
    );
    assert!(matches!(
        state.roots[0].resource,
        RootResource::ReleasedForHostedDestruction { .. }
    ));
    drop(state);
    // A genuinely reached writer for A cannot use a historical snapshot as a cap.
    let capture = f.session.snapshot();
    let html = capture.node(released).unwrap().children()[0];
    let body = capture.node(html).unwrap().children()[1];
    let stamp = capture.stamp();
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(released, Namespace::html(), "meta");
    batch.set_attribute(&element, "http-equiv", "default-style");
    batch.set_attribute(&element, "content", "C");
    batch.insert_before(&element, body, None);
    assert!(
        matches!(f.styles.apply_meta(&mut f.session.dom(),batch,None,DomMetaAtomicLimits::new(16_000_000,1_000_000,1_000_000,16).unwrap(),&cancel()),
        Err(DomStyleMetaError::Runtime(DomStyleRuntimeError::RootReleasedForHostedDestruction { root,.. })) if root==released)
    );
    assert_eq!(f.session.snapshot().stamp(), stamp);
    assert!(
        f.session
            .snapshot()
            .node(f.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(
        output
            .withdrawal_cssom_snapshot()
            .sheet(f.sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
}
