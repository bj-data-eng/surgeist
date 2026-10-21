//! Exact native wait custody and admission at the owning Runtime bank.
use super::*;

fn drive(session: &mut DomSession, context: &mut Context, edit: DomSourceEdit) {
    assert!(matches!(
        context
            .source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &AtomicBool::new(false),
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
}

fn append_import(session: &mut DomSession, context: &mut Context, document: NodeId) -> NodeId {
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "@import url('pending.css'); a { color: red }");
    batch.insert_before(&text, &style, None);
    let commit = session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let parent = session.snapshot().node(document).unwrap().children()[0];
    drive(
        session,
        context,
        DomSourceEdit::insert_before(style, parent, None),
    );
    style
}

fn fixture() -> (DomSession, Context, NodeId, NodeId, NodeId) {
    let (mut session, document) = bound_dispatch_session();
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    let retired = append_import(&mut session, &mut context, document);
    let live = append_import(&mut session, &mut context, document);
    drive(&mut session, &mut context, DomSourceEdit::detach(retired));
    drive(&mut session, &mut context, DomSourceEdit::destroy(retired));
    assert!(matches!(
        session.snapshot().node(retired),
        Err(DomError::StaleNode)
    ));
    assert!(session.snapshot().node(live).is_ok());
    assert_eq!(context.styles.waiting_resources().unwrap(), 2);
    (session, context, document, retired, live)
}

fn pump(
    session: &mut DomSession,
    context: &mut Context,
) -> Result<DomStyleNetworkingProgress, DomStyleRuntimeError> {
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 0,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        context,
        dispatch_limits(),
        &AtomicBool::new(false),
    )
}

#[test]
fn suffix_work_refusal_preserves_exact_waits_and_cumulative_same_owner_retry() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut context, document, retired, live) = fixture();
    let original = context.styles.original().unwrap();
    let (retired_id, live_id, capacity, next_task, root, storage_steps, work) = {
        let state = original.borrow();
        let before = state.work.get();
        state.storage(None, &cancel, limits().work).unwrap();
        (
            state.completions[0].task().id(),
            state.completions[1].task().id(),
            state.completions.capacity(),
            state.next_task,
            std::rc::Rc::downgrade(&state.completions[0].task().owner),
            state.work.get() - before,
            state.work.get(),
        )
    };
    let strong = root.strong_count();
    // set_limits pays its real step + measured current-storage walk first.
    // The pump can then pay entry/probe selection but not its real suffix move.
    let mut tight = limits();
    tight.work = work + 1 + storage_steps + 3;
    context
        .styles
        .set_limits(&session.dom(), tight, &cancel)
        .unwrap();
    let before = original.borrow().work.get();
    let dom_before = session.snapshot();
    let cssom_before = context.styles.snapshot(document).unwrap().unwrap();
    assert!(matches!(
        pump(&mut session, &mut context),
        Err(DomStyleRuntimeError::Admission)
    ));
    {
        let state = original.borrow();
        assert!(state.work.get() > before);
        assert!(state.work.get() > tight.work);
        assert_eq!(state.completions.len(), 2);
        assert_eq!(state.completions[0].task().id(), retired_id);
        assert_eq!(state.completions[1].task().id(), live_id);
        assert_eq!(state.completions.capacity(), capacity);
        assert_eq!(state.next_task, next_task);
        assert!(state.completions.iter().all(|row| matches!(row,
            Completion::Waiting(task) if !task.running && matches!(task.delivery, Delivery::NotStarted)
        )));
    }
    assert_eq!(root.strong_count(), strong);
    context
        .styles
        .set_limits(&session.dom(), limits(), &cancel)
        .unwrap();
    assert!(matches!(
        pump(&mut session, &mut context).unwrap(),
        DomStyleNetworkingProgress::TargetDestroyed {task, target}
            if task == retired_id && target == retired
    ));
    {
        let state = original.borrow();
        assert_eq!(state.completions.len(), 1);
        assert_eq!(state.completions[0].task().id(), live_id);
        assert_eq!(state.completions[0].task().token.target(), live);
        assert_eq!(state.completions.capacity(), capacity);
        assert_eq!(state.next_task, next_task);
    }
    assert_eq!(root.strong_count(), strong - 1);
    assert!(
        root.upgrade().is_some(),
        "live root and wait retain actual capability"
    );
    assert_eq!(session.snapshot().stamp(), dom_before.stamp());
    assert_eq!(
        context
            .styles
            .snapshot(document)
            .unwrap()
            .unwrap()
            .revision(),
        cssom_before.revision()
    );
    assert!(context.events.is_empty());
    assert!(matches!(
        pump(&mut session, &mut context).unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
}

#[test]
fn output_storage_refusal_retains_current_capacity_and_original_row_custody_until_retry() {
    let cancel = AtomicBool::new(false);
    let (mut session, mut context, document, retired, _) = fixture();
    let original = context.styles.original().unwrap();
    let (bytes, id, suffix_id, capacity, root) = {
        let state = original.borrow();
        (
            state.storage(None, &cancel, limits().work).unwrap(),
            state.completions[0].task().id(),
            state.completions[1].task().id(),
            state.completions.capacity(),
            std::rc::Rc::downgrade(&state.completions[0].task().owner),
        )
    };
    let strong = root.strong_count();
    let mut tight = limits();
    tight.storage_bytes = bytes;
    context
        .styles
        .set_limits(&session.dom(), tight, &cancel)
        .unwrap();
    let before = original.borrow().work.get();
    assert!(matches!(
        pump(&mut session, &mut context),
        Err(DomStyleRuntimeError::Admission)
    ));
    {
        let state = original.borrow();
        assert!(state.work.get() > before);
        assert_eq!(state.completions.len(), 2);
        assert_eq!(state.completions[0].task().id(), id);
        assert_eq!(state.completions[1].task().id(), suffix_id);
        assert_eq!(state.completions.capacity(), capacity);
        assert_eq!(state.storage(None, &cancel, limits().work).unwrap(), bytes);
    }
    assert_eq!(root.strong_count(), strong);
    context
        .styles
        .set_limits(&session.dom(), limits(), &cancel)
        .unwrap();
    assert!(matches!(pump(&mut session, &mut context).unwrap(),
        DomStyleNetworkingProgress::TargetDestroyed {task,target} if task==id && target==retired
    ));
    assert_eq!(root.strong_count(), strong - 1);
    assert_eq!(original.borrow().completions.capacity(), capacity);
    assert!(context.styles.snapshot(document).unwrap().is_some());
    assert!(context.events.is_empty());
}
