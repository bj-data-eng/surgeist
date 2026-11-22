//! Live native cursor and affine filtering against the pinned TreeWalker steps.

use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};
use surgeist_dom::{
    Dom, DomBatch, DomError,
    edit::DomChange,
    event::{DomDispatchCustodyLimits, DomSourceEventOutcome},
    identity::{DocumentId, NodeId},
    limits::{DomLimits, DomNameLimits, DomResource},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest},
    tree::{
        DomTreeWalkerDirection as Direction, DomTreeWalkerFilter as Filter,
        DomTreeWalkerFilterRequest, DomTreeWalkerId, DomTreeWalkerProgress as Progress,
    },
};
use surgeist_html::values::Namespace;

struct Tree {
    dom: Dom,
    document: DocumentId,
    receiving: DocumentId,
    root: NodeId,
    first: NodeId,
    text: NodeId,
    branch: NodeId,
    nested: NodeId,
    last: NodeId,
    outside: NodeId,
}

fn tree(slots: usize, cancel: &AtomicBool) -> Tree {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    let limits = DomLimits::new(64, 4096, 128, 0, 0, 16, 1_000_000, names)
        .unwrap()
        .with_tree_walkers(slots);
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let receiving = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "section");
    let branch = batch.create_element_ns(&document, Namespace::html(), "main");
    let text = batch.create_text(&document, "under section");
    let nested = batch.create_element_ns(&document, Namespace::html(), "p");
    let last = batch.create_element_ns(&document, Namespace::html(), "span");
    let outside = batch.create_element_ns(&receiving, Namespace::html(), "article");
    let outside_text = batch.create_text(&receiving, "outside the root");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&first, &root, None);
    batch.insert_before(&text, &first, None);
    batch.insert_before(&branch, &root, None);
    batch.insert_before(&nested, &branch, None);
    batch.insert_before(&last, &root, None);
    batch.insert_before(&outside_text, &outside, None);
    let commit = dom.apply(batch, cancel).unwrap();
    Tree {
        dom,
        document: commit.document(&document).unwrap(),
        receiving: commit.document(&receiving).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        first: commit.created(&first).unwrap().unwrap(),
        text: commit.created(&text).unwrap().unwrap(),
        branch: commit.created(&branch).unwrap().unwrap(),
        nested: commit.created(&nested).unwrap().unwrap(),
        last: commit.created(&last).unwrap().unwrap(),
        outside: commit.created(&outside).unwrap().unwrap(),
    }
}

type Admission = fn() -> Result<(), DomError>;

fn navigate<'cancel>(
    dom: &Dom,
    id: DomTreeWalkerId,
    direction: Direction,
    cancel: &'cancel AtomicBool,
) -> Progress<'cancel, Admission> {
    let admit: Admission = || Ok(());
    dom.navigate_tree_walker(id, direction, admit, cancel)
        .unwrap()
}

fn filter<'cancel>(
    progress: Progress<'cancel, Admission>,
    candidate: NodeId,
) -> DomTreeWalkerFilterRequest<'cancel, Admission> {
    let Progress::Filter(filter) = progress else {
        panic!("expected a filter: {progress:?}");
    };
    assert_eq!(filter.node(), candidate);
    filter
}

fn accept<A: Fn() -> Result<(), DomError>>(
    dom: &mut Dom,
    progress: Progress<'_, A>,
    candidate: NodeId,
) {
    let Progress::Match(output) = progress else {
        panic!("expected a prepared match: {progress:?}");
    };
    assert_eq!(output.node(), candidate);
    assert_eq!(output.accept(dom), Ok(candidate));
}

#[test]
fn all_directions_publish_only_the_prepared_ordinary_match() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        text,
        branch,
        nested,
        last,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, u32::MAX, Filter::None, &cancel)
        .unwrap();
    let initial = dom.tree_walker(id).unwrap();
    assert_eq!(
        (initial.id(), initial.root(), initial.current_node()),
        (id, root, root)
    );
    assert_eq!(
        (initial.filter(), initial.what_to_show()),
        (Filter::None, u32::MAX)
    );
    assert!(matches!(
        navigate(&dom, id, Direction::Parent, &cancel),
        Progress::NoMatch
    ));
    for (direction, node) in [
        (Direction::FirstChild, first),
        (Direction::NextSibling, branch),
        (Direction::LastChild, nested),
        (Direction::Parent, branch),
        (Direction::PreviousNode, text),
        (Direction::NextNode, branch),
        (Direction::NextSibling, last),
        (Direction::PreviousSibling, branch),
    ] {
        let before = dom.tree_walker(id).unwrap();
        let stamp = dom.snapshot().stamp();
        let found = navigate(&dom, id, direction, &cancel);
        assert_eq!(dom.tree_walker(id), Ok(before));
        assert_eq!(dom.snapshot().stamp(), stamp);
        accept(&mut dom, found, node);
        assert_eq!(dom.tree_walker(id).unwrap().current_node(), node);
    }
}

#[test]
fn child_skip_and_document_order_keep_unknown_filter_results_distinct() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        text,
        branch,
        nested,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, u32::MAX, Filter::Callback, &cancel)
        .unwrap();
    let progress = filter(navigate(&dom, id, Direction::FirstChild, &cancel), first)
        .respond(&dom, 65535)
        .unwrap();
    let progress = filter(progress, branch).respond(&dom, 3).unwrap();
    let progress = filter(progress, nested).respond(&dom, 1).unwrap();
    accept(&mut dom, progress, nested);
    dom.set_tree_walker_current(id, root, &cancel).unwrap();
    let progress = filter(navigate(&dom, id, Direction::NextNode, &cancel), first)
        .respond(&dom, 65535)
        .unwrap();
    let progress = filter(progress, text).respond(&dom, 1).unwrap();
    accept(&mut dom, progress, text);
}

#[test]
fn active_filter_refuses_before_mask_and_accepted_candidate_wins_reentry() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        outside,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, 1, Filter::Callback, &cancel)
        .unwrap();
    let pending = filter(navigate(&dom, id, Direction::FirstChild, &cancel), first);
    dom.set_tree_walker_current(id, outside, &cancel).unwrap();
    assert!(matches!(
        navigate(&dom, id, Direction::Parent, &cancel),
        Progress::NoMatch
    ));
    assert!(matches!(
        dom.navigate_tree_walker(id, Direction::FirstChild, || Ok::<_, DomError>(()), &cancel),
        Err(DomError::TreeWalkerFilterActive)
    ));
    let progress = pending.respond(&dom, 1).unwrap();
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), outside);
    accept(&mut dom, progress, first);
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), first);
}

#[test]
fn child_climb_uses_live_current_and_ignores_node_iterator_removal_fixup() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        text,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, u32::MAX, Filter::Callback, &cancel)
        .unwrap();
    let progress = filter(navigate(&dom, id, Direction::FirstChild, &cancel), first)
        .respond(&dom, 3)
        .unwrap();
    let pending = filter(progress, text);
    dom.set_tree_walker_current(id, first, &cancel).unwrap();
    assert!(matches!(pending.respond(&dom, 2), Ok(Progress::NoMatch)));
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), first);
    let mut detach = DomBatch::new();
    detach.detach(first);
    dom.apply(detach, &cancel).unwrap();
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), first);
    assert!(matches!(
        navigate(&dom, id, Direction::Parent, &cancel),
        Progress::NoMatch
    ));
}

#[test]
fn callback_edits_and_cancellation_survive_abandoned_or_refused_operations() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        root,
        first,
        text,
        last,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, u32::MAX, Filter::Callback, &cancel)
        .unwrap();
    let pending = filter(navigate(&dom, id, Direction::NextNode, &cancel), first);
    let mut edit = DomBatch::new();
    edit.detach(text);
    let child = edit.create_element_ns(document, Namespace::html(), "i");
    edit.insert_before(&child, first, None);
    let child = dom
        .apply(edit, &cancel)
        .unwrap()
        .created(&child)
        .unwrap()
        .unwrap();
    let progress = pending.respond(&dom, 3).unwrap();
    let pending = filter(progress, child);
    dom.set_tree_walker_current(id, last, &cancel).unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(pending.respond(&dom, 1), Err(DomError::Cancelled)));
    cancel.store(false, Ordering::Relaxed);
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), last);
    assert!(dom.snapshot().node(child).is_ok());
    let pending = filter(
        navigate(&dom, id, Direction::PreviousSibling, &cancel),
        dom.snapshot()
            .node(last)
            .unwrap()
            .previous_sibling()
            .unwrap(),
    );
    drop(pending);
    assert!(matches!(
        navigate(&dom, id, Direction::PreviousSibling, &cancel),
        Progress::Filter(_)
    ));
}

#[test]
fn output_refusal_and_intervening_publication_preserve_current() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        last,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, u32::MAX, Filter::None, &cancel)
        .unwrap();
    let Progress::Match(unused) = navigate(&dom, id, Direction::FirstChild, &cancel) else {
        panic!("first child");
    };
    assert_eq!(unused.node(), first);
    drop(unused);
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), root);
    let Progress::Match(stale) = navigate(&dom, id, Direction::FirstChild, &cancel) else {
        panic!("first child");
    };
    dom.set_tree_walker_current(id, last, &cancel).unwrap();
    assert_eq!(
        stale.accept(&mut dom),
        Err(DomError::TreeWalkerPublicationUnavailable)
    );
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), last);
}

#[test]
fn destruction_retires_membership_and_reuse_preserves_generation_and_owner() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        receiving,
        root,
        first,
        outside,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, u32::MAX, Filter::None, &cancel)
        .unwrap();
    let keeper = dom
        .create_tree_walker(outside, 0, Filter::None, &cancel)
        .unwrap();
    assert!(matches!(
        dom.create_tree_walker(root, 0, Filter::None, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::TreeWalkers,
            ..
        })
    ));
    dom.set_tree_walker_current(id, outside, &cancel).unwrap();
    let mut adoption = DomBatch::new();
    adoption.adopt(root, receiving);
    dom.apply(adoption, &cancel).unwrap();
    assert_eq!(dom.tree_walker(id).unwrap().root(), root);
    assert_eq!(dom.tree_walker(id).unwrap().current_node(), outside);
    assert!(matches!(
        navigate(&dom, id, Direction::FirstChild, &cancel),
        Progress::Match(_)
    ));
    let Tree {
        dom: mut foreign,
        root: foreign_root,
        ..
    } = tree(1, &cancel);
    let foreign_id = foreign
        .create_tree_walker(foreign_root, 1, Filter::None, &cancel)
        .unwrap();
    assert_eq!(dom.tree_walker(foreign_id), Err(DomError::ForeignOwner));
    let mut destroy = DomBatch::new();
    destroy.destroy(outside);
    dom.apply(destroy, &cancel).unwrap();
    assert_eq!(dom.tree_walker(id), Err(DomError::StaleTreeWalker));
    assert_eq!(dom.tree_walker(keeper), Err(DomError::StaleTreeWalker));
    assert_eq!(dom.usage().tree_walkers(), 0);
    let new = dom
        .create_tree_walker(first, 1, Filter::None, &cancel)
        .unwrap();
    assert_ne!(id, new);
    assert_eq!(dom.tree_walker(id), Err(DomError::StaleTreeWalker));
    dom.retire_tree_walker(new, &cancel).unwrap();
    assert_eq!(dom.usage().tree_walkers(), 0);
}

#[test]
fn exact_admission_covers_preparation_and_first_typed_refusal() {
    #[derive(Debug, PartialEq)]
    enum Stop {
        Native(DomError),
        Work,
    }
    impl From<DomError> for Stop {
        fn from(error: DomError) -> Self {
            Self::Native(error)
        }
    }
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        ..
    } = tree(2, &cancel);
    let id = dom
        .create_tree_walker(root, 1, Filter::None, &cancel)
        .unwrap();
    let spent = Cell::new(0);
    let admit = || {
        spent.set(spent.get() + 1);
        Ok::<_, DomError>(())
    };
    let progress = dom
        .navigate_tree_walker(id, Direction::FirstChild, admit, &cancel)
        .unwrap();
    accept(&mut dom, progress, first);
    let required = spent.get();
    assert!(required > 1);
    for (limit, expected) in [(required - 1, Err(Stop::Work)), (required, Ok(first))] {
        dom.set_tree_walker_current(id, root, &cancel).unwrap();
        let before = dom.snapshot().stamp();
        let remaining = Cell::new(limit);
        let admit = || {
            if remaining.get() == 0 {
                return Err(Stop::Work);
            }
            remaining.set(remaining.get() - 1);
            Ok(())
        };
        let Progress::Match(pending) = dom
            .navigate_tree_walker(id, Direction::FirstChild, admit, &cancel)
            .unwrap()
        else {
            panic!("first child");
        };
        assert_eq!(pending.accept(&mut dom), expected);
        assert_eq!(remaining.get(), 0);
        if expected.is_err() {
            assert_eq!(dom.snapshot().stamp(), before);
            assert_eq!(dom.tree_walker(id).unwrap().current_node(), root);
        }
    }
}

#[test]
fn active_source_owns_search_work_and_suspended_operations_cannot_escape_it() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        document,
        root,
        first,
        branch,
        ..
    } = tree(3, &cancel);
    let custody =
        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap();
    dom.set_limits(dom.limits().with_dispatch_custody(custody))
        .unwrap();
    let filtered = dom
        .create_tree_walker(root, 1, Filter::Callback, &cancel)
        .unwrap();
    let plain = dom
        .create_tree_walker(root, 1, Filter::None, &cancel)
        .unwrap();
    let mut edit = DomBatch::new();
    let dialog = edit.create_element_ns(document, Namespace::html(), "dialog");
    edit.insert_before(&dialog, root, None);
    let dialog = dom
        .apply(edit, &cancel)
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let collection = dom
        .get_elements_by_tag_name(
            surgeist_dom::tree::DomDescendantQueryRoot::Document(document),
            "*".into(),
            128,
            &cancel,
        )
        .unwrap();
    let collection_limits = surgeist_dom::tree::DomHtmlCollectionLimits::new(100_000, 16, 1024);
    let limits = DomSourceLimits::new(32_000_000, 1_000_000, 1_000_000, 64, 4096, 4096, 4);
    let executor = dom.bind_source_event_executor(&cancel).unwrap();
    let event = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), limits, &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("showing the dialog reaches its Source callback");
        };
        event
    };
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let before = dom.pending_source_edit().unwrap().spent_work();
    let admitted = Cell::new(0);
    let admission = || {
        admitted.set(admitted.get() + 1);
        Ok::<_, DomError>(())
    };
    let Progress::Match(output) = dom
        .navigate_tree_walker(plain, Direction::FirstChild, admission, &cancel)
        .unwrap()
    else {
        panic!("the unfiltered first child reaches output preparation");
    };
    assert_eq!(output.node(), first);
    assert!(admitted.get() > 0);
    // Entry qualifies the native observation once. Each reached traversal
    // step then spends one caller unit and one native Source unit.
    assert_eq!(
        dom.pending_source_edit().unwrap().spent_work() - before,
        admitted.get() + 1
    );
    let collection_work = Cell::new(0);
    let before = dom.pending_source_edit().unwrap().spent_work();
    {
        let read = dom
            .read_html_collection(
                &collection,
                collection_limits,
                || {
                    collection_work.set(collection_work.get() + 1);
                    Ok::<_, DomError>(())
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(read.item(0).unwrap(), Some(root));
    }
    assert!(collection_work.get() > 0);
    assert_eq!(
        dom.pending_source_edit().unwrap().spent_work() - before,
        collection_work.get() + 1
    );

    let pending = filter(
        navigate(&dom, filtered, Direction::FirstChild, &cancel),
        first,
    );
    dom.set_tree_walker_current(filtered, branch, &cancel)
        .unwrap();
    {
        let read = dom
            .read_html_collection(
                &collection,
                collection_limits,
                || Ok::<_, DomError>(()),
                &cancel,
            )
            .unwrap();
        drop(scope);
        assert_eq!(read.item(0), Err(DomError::InvalidSourceEvent));
        let replacement_scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        assert_eq!(read.item(0), Err(DomError::InvalidSourceEvent));
        let current = dom
            .read_html_collection(
                &collection,
                collection_limits,
                || Ok::<_, DomError>(()),
                &cancel,
            )
            .unwrap();
        assert_eq!(current.item(0).unwrap(), Some(root));
        drop(replacement_scope);
    }
    let mut permit = Some(permit);
    let mut outcome = Some(
        executor
            .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(event.operation(), limits, &cancel)
        .unwrap();
    lease
        .accept_source_event(&event, &mut outcome, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(output.accept(&mut dom), Err(DomError::InvalidSourceEvent));
    assert!(matches!(
        pending.respond(&dom, 1),
        Err(DomError::InvalidSourceEvent)
    ));
    assert_eq!(dom.tree_walker(plain).unwrap().current_node(), root);
    assert_eq!(dom.tree_walker(filtered).unwrap().current_node(), branch);
    assert!(matches!(
        navigate(&dom, filtered, Direction::FirstChild, &cancel),
        Progress::Filter(_)
    ));
}

#[test]
fn refused_destruction_preserves_all_memberships_and_retry_retires_only_watchers() {
    let cancel = AtomicBool::new(false);
    let Tree {
        mut dom,
        root,
        first,
        last,
        ..
    } = tree(4, &cancel);
    let moved = dom
        .create_tree_walker(root, 1, Filter::None, &cancel)
        .unwrap();
    let stays = dom
        .create_tree_walker(root, 1, Filter::None, &cancel)
        .unwrap();
    let anchored = dom
        .create_tree_walker(first, 1, Filter::Callback, &cancel)
        .unwrap();
    dom.set_tree_walker_current(moved, first, &cancel).unwrap();
    dom.set_tree_walker_current(stays, first, &cancel).unwrap();
    let before_move = dom.snapshot().stamp();
    dom.set_tree_walker_current(moved, last, &cancel).unwrap();
    assert_eq!(
        dom.changes_since(before_move).unwrap().changes(),
        &[DomChange::TreeWalkerCurrent(moved)]
    );
    let mut detach = DomBatch::new();
    detach.detach(first);
    dom.apply(detach, &cancel).unwrap();
    let before = dom.snapshot().stamp();
    let refused = DomSourceLimits::new(16_000_000, 100_000, 0, 128, 4096, 4096, 1);
    let error = match dom.begin_source_edit(DomSourceEdit::destroy(first), refused, &cancel) {
        Err(error) => error,
        Ok(mut lease) => {
            let error = lease.advance(&cancel).unwrap_err();
            assert!(!lease.status().has_published());
            error
        }
    };
    assert_eq!(error, DomError::SourceWorkLimit { limit: 0 });
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage().tree_walkers(), 3);
    assert_eq!(dom.tree_walker(stays).unwrap().current_node(), first);
    assert_eq!(dom.tree_walker(anchored).unwrap().current_node(), first);
    let admitted = DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 128, 4096, 4096, 1);
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::destroy(first), admitted, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(dom.tree_walker(stays), Err(DomError::StaleTreeWalker));
    assert_eq!(dom.tree_walker(anchored), Err(DomError::StaleTreeWalker));
    assert_eq!(dom.tree_walker(moved).unwrap().current_node(), last);
    assert_eq!(dom.usage().tree_walkers(), 1);
    let changes = dom.changes_since(before).unwrap();
    for retired in [stays, anchored] {
        assert_eq!(
            changes
                .changes()
                .iter()
                .filter(|change| **change == DomChange::TreeWalkerRetired(retired))
                .count(),
            1
        );
    }
}
