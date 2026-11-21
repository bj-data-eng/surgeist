use super::*;
use crate::internal::{
    dispatch::DomDispatchCustodyLimits, shadow::DomShadowRootInit, source::DomSourceEdit,
};
use std::{
    cell::{Cell, RefCell},
    sync::atomic::Ordering,
};
use surgeist_html::{construction::HtmlShadowRootMode, values::Namespace};

fn indexed(
    dom: &Dom,
    global: DomHostedGlobalId,
    index: usize,
) -> Result<Option<DomHostedBrowsingContextId>, DomHostedError> {
    dom.window_indexed_child(global, index, 100_000, || Ok(()), &cancel())
}

fn assert_indexed_order(f: &Fixture, expected: &[DomHostedBrowsingContextId]) {
    let global = f.creation.global;
    for (index, context) in expected.iter().enumerate() {
        assert_eq!(
            indexed(&f.dom, global, index),
            Ok(Some(*context)),
            "index {index}"
        );
    }
    assert_eq!(indexed(&f.dom, global, expected.len()), Ok(None));
    assert_eq!(indexed(&f.dom, global, usize::MAX), Ok(None));
}

#[test]
fn window_indexed_child_orders_insertion_groups_and_current_ordinary_tree_ties() {
    let mut f = Fixture::new();
    assert_indexed_order(&f, &[]);
    let (old, old_child) = insert_absent_src_frame(&mut f);
    let parent = f.dom.snapshot().node(old).unwrap().parent().unwrap();
    assert_indexed_order(&f, &[old_child.context]);

    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(f.creation.document, None);
    let subtree = batch.create_element_ns(&closed, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&subtree, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let first = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    let second = batch.create_element_ns(&closed, Namespace::html(), "frame");
    batch.set_attribute(&first, "sandbox", "");
    batch.insert_before(&first, &subtree, None);
    batch.insert_before(&second, &subtree, None);
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let shadow = commit.attached_shadow_root(&shadow).unwrap().unwrap();
    let [subtree, first, second] =
        [&subtree, &first, &second].map(|ticket| commit.created(ticket).unwrap().unwrap());

    let [first_child, second_child] = drive_native_insertion(&mut f, subtree, parent, Some(old));
    let snapshot = f.dom.snapshot();
    for (child, container) in [(first_child, first), (second_child, second)] {
        assert_eq!(
            snapshot
                .hosted_navigable(child.navigable)
                .unwrap()
                .container(),
            Some(container)
        );
    }
    assert_ne!(
        snapshot
            .document(first_child.document)
            .unwrap()
            .metadata()
            .origin(),
        snapshot
            .document(f.creation.document)
            .unwrap()
            .metadata()
            .origin(),
    );
    assert_eq!(snapshot.node(subtree).unwrap().next_sibling(), Some(old));
    assert_eq!(snapshot.node(first).unwrap().next_sibling(), Some(second));
    assert_indexed_order(
        &f,
        &[old_child.context, first_child.context, second_child.context],
    );

    complete_native_structure(
        &mut f,
        DomSourceEdit::move_before(second, subtree, Some(first)),
    );
    let snapshot = f.dom.snapshot();
    assert_eq!(snapshot.node(second).unwrap().next_sibling(), Some(first));
    assert_indexed_order(
        &f,
        &[old_child.context, second_child.context, first_child.context],
    );

    complete_native_structure(&mut f, DomSourceEdit::move_before(second, shadow, None));
    assert_indexed_order(&f, &[old_child.context, first_child.context]);
    complete_native_structure(
        &mut f,
        DomSourceEdit::move_before(second, subtree, Some(first)),
    );
    assert_indexed_order(
        &f,
        &[old_child.context, second_child.context, first_child.context],
    );

    drive_native_removal(&mut f, old);
    let snapshot = f.dom.snapshot();
    let retiring = snapshot.hosted_navigable(old_child.navigable).unwrap();
    assert!(retiring.is_retiring());
    assert_eq!(
        snapshot.global_document(old_child.global),
        Ok(old_child.document)
    );
    assert_eq!(
        indexed(&f.dom, old_child.global, usize::MAX),
        Err(DomHostedError::StaleIdentity)
    );
    assert_indexed_order(&f, &[second_child.context, first_child.context]);
    let [replacement] = drive_native_insertion(&mut f, old, parent, Some(subtree));
    assert_ne!(replacement.context, old_child.context);
    assert_eq!(
        f.dom.snapshot().node(old).unwrap().next_sibling(),
        Some(subtree)
    );
    assert_indexed_order(
        &f,
        &[
            second_child.context,
            first_child.context,
            replacement.context,
        ],
    );

    let foreign = Fixture::new();
    assert_eq!(
        indexed(&f.dom, foreign.creation.global, usize::MAX),
        Err(DomHostedError::ForeignSource)
    );
}

#[test]
fn window_indexed_child_preserves_the_last_reached_caller_step_and_issuer() {
    let mut f = Fixture::new();
    let (_, first) = insert_absent_src_frame(&mut f);
    let (_, second) = insert_absent_src_frame(&mut f);
    let Fixture {
        dom,
        source,
        creation,
    } = f;
    let admissions = Cell::new(0);
    let result = dom.window_indexed_child(
        creation.global,
        1,
        100_000,
        || {
            admissions.set(admissions.get() + 1);
            Ok::<_, DomHostedError>(())
        },
        &cancel(),
    );
    assert_ne!(first.context, second.context);
    assert_eq!(result, Ok(Some(second.context)));
    let required = admissions.get();
    for shortfall in [0, 1] {
        let limit = required - shortfall;
        admissions.set(0);
        let actual = dom.window_indexed_child(
            creation.global,
            1,
            limit,
            || {
                admissions.set(admissions.get() + 1);
                Ok::<_, DomHostedError>(())
            },
            &cancel(),
        );
        assert_eq!(admissions.get(), limit);
        if shortfall == 0 {
            assert_eq!(actual, Ok(Some(second.context)));
        } else {
            assert_eq!(actual, Err(DomHostedError::WorkLimit { limit }));
        }
    }

    admissions.set(0);
    let result = dom.window_indexed_child(
        creation.global,
        1,
        required,
        || {
            admissions.set(admissions.get() + 1);
            if admissions.get() == required {
                Err(WindowReadError::Refused)
            } else {
                Ok(())
            }
        },
        &cancel(),
    );
    assert_eq!(result, Err(WindowReadError::Refused));

    let cancelled = cancel();
    admissions.set(0);
    let result = dom.window_indexed_child(
        creation.global,
        1,
        required,
        || {
            admissions.set(admissions.get() + 1);
            if admissions.get() == required {
                cancelled.store(true, Ordering::Relaxed);
            }
            Ok::<_, DomHostedError>(())
        },
        &cancelled,
    );
    assert_eq!(result, Err(DomHostedError::Dom(DomError::Cancelled)));

    let source = RefCell::new(Some(source));
    admissions.set(0);
    let result = dom.window_indexed_child(
        creation.global,
        1,
        required,
        || {
            admissions.set(admissions.get() + 1);
            if admissions.get() == required {
                drop(source.borrow_mut().take());
            }
            Ok::<_, DomHostedError>(())
        },
        &cancel(),
    );
    assert_eq!(result, Err(DomHostedError::SourceExpired));
}

#[test]
fn window_indexed_child_uses_one_exact_source_scope_through_rank_selection() {
    let mut f = Fixture::new();
    insert_absent_src_frame(&mut f);
    let (_, second) = insert_absent_src_element(&mut f, "iframe");
    let custody =
        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap();
    f.dom
        .set_limits(f.dom.limits().with_dispatch_custody(custody))
        .unwrap();
    let mut edit = DomBatch::new();
    let actor = edit.create_html_document(f.creation.document, None);
    let commit = f.dom.apply(edit, &cancel()).unwrap();
    let actor = commit.document(&actor).unwrap();
    let parent = f
        .dom
        .snapshot()
        .document(actor)
        .unwrap()
        .document_element()
        .unwrap();
    let mut edit = DomBatch::new();
    let dialog = edit.create_element_ns(actor, Namespace::html(), "dialog");
    edit.insert_before(&dialog, parent, None);
    let commit = f.dom.apply(edit, &cancel()).unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let executor = f.dom.bind_source_event_executor(&cancel()).unwrap();
    let event = {
        let mut edit = f
            .dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(dialog),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        match edit.advance(&cancel()).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Event(event)) => Some(event),
            _ => None,
        }
        .expect("actual native dialog event")
    };
    let admissions = Cell::new(0);
    let global = f.creation.global;
    let resolve = |dom: &Dom| {
        dom.window_indexed_child(
            global,
            1,
            100_000,
            || {
                admissions.set(admissions.get() + 1);
                Ok::<_, DomHostedError>(())
            },
            &cancel(),
        )
    };
    assert_eq!(
        resolve(&f.dom),
        Err(DomHostedError::Dom(DomError::IncompleteSourceOperation))
    );
    assert_eq!(admissions.get(), 0);
    let permit = executor.begin_dispatch(&f.dom, &event, &cancel()).unwrap();
    let scope = executor.enter_dispatch(&f.dom, &permit, &cancel()).unwrap();
    let before = f.dom.pending_source_edit().unwrap().spent_work();
    assert_eq!(resolve(&f.dom), Ok(Some(second.context)));
    let spent = f.dom.pending_source_edit().unwrap().spent_work();
    let required = spent - before;
    assert_eq!(required, admissions.get() + 1);
    let stamp = f.dom.snapshot().stamp();
    for shortfall in [0, 1] {
        let allowance = required - shortfall;
        let mut limits = source_limits();
        let spent = f.dom.pending_source_edit().unwrap().spent_work();
        // Resume the original event and leave exactly the intended budget.
        limits.work = spent + allowance + 4096;
        drop(
            f.dom
                .resume_source_edit(event.operation(), limits, &cancel())
                .unwrap(),
        );
        let remaining = limits.work - f.dom.pending_source_edit().unwrap().spent_work();
        assert!(remaining >= allowance);
        for _ in 0..remaining - allowance {
            f.dom.source_event_step(&event, &cancel()).unwrap();
        }
        admissions.set(0);
        let result = resolve(&f.dom);
        if shortfall == 0 {
            assert_eq!(result, Ok(Some(second.context)));
        } else {
            assert_eq!(
                result,
                Err(DomHostedError::Dom(DomError::SourceWorkLimit {
                    limit: limits.work
                }))
            );
        }
        let current = f.dom.pending_source_edit().unwrap();
        assert_eq!(current.operation(), event.operation());
        assert_eq!(current.spent_work(), limits.work);
        assert_eq!(admissions.get() + 1, required);
        assert_eq!(f.dom.snapshot().stamp(), stamp);
    }
    drop(executor);
    assert_eq!(
        resolve(&f.dom),
        Err(DomHostedError::Dom(DomError::InvalidSourceEvent))
    );
    drop(scope);
}
