//! DOM3071 split steps 1–9: exact UTF-16, fresh identity and special Range moves.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(nodes: usize, units: usize) -> DomLimits {
    DomLimits::new(
        nodes,
        units,
        512,
        0,
        0,
        64,
        1_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(64)
    .unwrap()
}
fn policy(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, outputs, 4096, 8)
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn fixture(value: DocumentString, attached: bool) -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(limits(128, 65536)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "root");
    let left = batch.create_comment(&document, "left");
    let text = batch.create_text(&document, value);
    let right = batch.create_comment(&document, "right");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&left, &parent, None);
    if attached {
        batch.insert_before(&text, &parent, None);
    }
    batch.insert_before(&right, &parent, None);
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&parent).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
    )
}
fn range(
    dom: &mut Dom,
    doc: DocumentId,
    start: (NodeId, usize),
    end: (NodeId, usize),
) -> DomRangeId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_range(doc);
    batch.set_range_start(&ticket, start.0, start.1);
    batch.set_range_end(&ticket, end.0, end.1);
    apply(dom, batch).created_range(&ticket).unwrap().unwrap()
}
fn points(snapshot: &DomSnapshot, range: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let r = snapshot.range(range).unwrap();
    (
        (r.start().container(), r.start().offset()),
        (r.end().container(), r.end().offset()),
    )
}

#[test]
fn contiguous_text_visit_stops_at_the_first_refusal_in_tree_order() {
    let (mut dom, document, parent, middle) = fixture("middle".into(), true);
    let cancel = cancel();
    let stop = dom.snapshot().node(middle).unwrap().next_sibling().unwrap();
    let mut batch = DomBatch::new();
    let first = batch.create_text(document, "first");
    let last = batch.create_text(document, "last");
    batch.insert_before(&first, parent, Some(middle.into()));
    batch.insert_before(&last, parent, Some(stop.into()));
    dom.apply(batch, &cancel).unwrap();
    let snapshot = dom.snapshot();
    let mut observed = Vec::new();
    let result = snapshot.node(middle).unwrap().visit_contiguous_text_with(
        || Ok(()),
        |text| {
            observed.push(text.clone());
            if observed.len() == 2 {
                return Err(DomError::Cancelled);
            }
            Ok(())
        },
    );
    assert_eq!(result, Err(DomError::Cancelled));
    assert_eq!(observed, [DocumentString::from("first"), "middle".into()]);
    let mut visited = false;
    assert_eq!(
        snapshot.node(middle).unwrap().visit_contiguous_text_with(
            || Err(DomError::Cancelled),
            |_| {
                visited = true;
                Ok(())
            }
        ),
        Err(DomError::Cancelled)
    );
    assert!(!visited);
}

#[test]
fn split_text_retirement_watch_refusal_leaves_the_split_private() {
    let (mut dom, _, parent, text) = fixture("abcd".into(), true);
    let cancel = cancel();
    let retirement_limits = DomNodeRetirementLimits::new(2_000_000, 20_000);
    let mut inbox = dom
        .node_retirement_inbox(retirement_limits, &cancel)
        .unwrap();
    let before = dom.snapshot();
    let mut refused =
        DomSplitTextOutputWatch::new(&inbox, 42, DomNodeRetirementLimits::new(2_000_000, 0));
    {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::split_text(text, 2), policy(4096), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance_split_text_for_runtime(&mut refused, &cancel),
            Err(DomError::NodeRetirementWorkLimit { limit: 0 })
        ));
    }
    assert!(refused.into_watch().is_none());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.pending_source_edit().is_none());
    let mut output = DomSplitTextOutputWatch::new(&inbox, 42, retirement_limits);
    let tail = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::split_text(text, 2), policy(4096), &cancel)
            .unwrap();
        let DomSourceAdvance::Progress(DomSourceProgress::Complete(done)) = lease
            .advance_split_text_for_runtime(&mut output, &cancel)
            .unwrap()
        else {
            panic!("expected the exact split completion");
        };
        let Some(DomSourceOutput::Node(node)) = done.output() else {
            panic!("expected a Text identity");
        };
        node
    };
    let (watched, _watch) = output.into_watch().unwrap();
    assert_eq!(watched, tail);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(parent).unwrap().children()[2], tail);
    assert_eq!(snapshot.node(text).unwrap().data(), Some(&"ab".into()));
    assert_eq!(snapshot.node(tail).unwrap().data(), Some(&"cd".into()));
    let mut batch = DomBatch::new();
    batch.detach(tail);
    batch.destroy(tail);
    apply(&mut dom, batch);
    let notice = inbox.take().unwrap();
    assert_eq!((notice.node(), notice.custody_key()), (tail, 42));
    assert!(inbox.take().is_none());
}

#[test]
fn split_transfers_strictly_later_endpoints_and_equal_parent_boundaries_at_every_utf16_offset() {
    let units = [0x41, 0xd800, 0, 0xdc00, 0x5a];
    for offset in 0..=units.len() {
        let (mut dom, doc, parent, text) =
            fixture(DocumentString::from_utf16(units.to_vec()), true);
        let text_ranges: Vec<_> = (0..=units.len())
            .map(|i| range(&mut dom, doc, (text, i), (text, i)))
            .collect();
        let parent_ranges: Vec<_> = (0..=3)
            .map(|i| range(&mut dom, doc, (parent, i), (parent, i)))
            .collect();
        let spanning = range(&mut dom, doc, (text, 0), (text, units.len()));
        let before = dom.snapshot();
        let original = before.node(parent).unwrap().children().to_vec();
        let mut batch = DomBatch::new();
        let ticket = batch.split_text(text, offset);
        let commit = apply(&mut dom, batch);
        let new = commit.created(&ticket).unwrap().unwrap();
        assert_ne!(new, text);
        let after = dom.snapshot();
        assert_eq!(
            after.node(parent).unwrap().children(),
            [original[0], text, new, original[2]]
        );
        assert_eq!(
            after.node(text).unwrap().data().unwrap().code_units(),
            &units[..offset]
        );
        assert_eq!(
            after.node(new).unwrap().data().unwrap().code_units(),
            &units[offset..]
        );
        assert_eq!(after.node(new).unwrap().owner_document(), Some(doc));
        for (i, id) in text_ranges.iter().enumerate() {
            let expected = if i > offset {
                (new, i - offset)
            } else {
                (text, i)
            };
            assert_eq!(points(&after, *id), (expected, expected));
            assert_eq!(points(&before, *id), ((text, i), (text, i)));
        }
        for (i, id) in parent_ranges.iter().enumerate() {
            let expected = (parent, if i >= 2 { i + 1 } else { i });
            assert_eq!(points(&after, *id), (expected, expected));
        }
        let end = if offset == units.len() {
            (text, offset)
        } else {
            (new, units.len() - offset)
        };
        assert_eq!(points(&after, spanning), ((text, 0), end));
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::Created(new))
        );
        assert_eq!(
            before.node(text).unwrap().data().unwrap().code_units(),
            units
        );
    }
}

#[test]
fn detached_split_clamps_original_ranges_without_transferring_them() {
    for offset in [0, 2, 4] {
        let (mut dom, doc, _, text) = fixture("abcd".into(), false);
        let id = range(&mut dom, doc, (text, 1), (text, 4));
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::split_text(text, offset),
                policy(4096),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
            panic!("detached split completes")
        };
        let Some(DomSourceOutput::Node(new)) = done.output() else {
            panic!("actual new Text output")
        };
        assert_eq!(done.published_node(), Some(new));
        drop(lease);
        let snapshot = dom.snapshot();
        assert_eq!(snapshot.node(new).unwrap().parent(), None);
        assert_eq!(snapshot.node(new).unwrap().root(), new);
        assert_eq!(
            points(&snapshot, id),
            ((text, 1.min(offset)), (text, offset))
        );
        assert_eq!(
            snapshot.node(new).unwrap().data().unwrap().code_units(),
            &"abcd".encode_utf16().collect::<Vec<_>>()[offset..]
        );
        assert!(dom.pending_source_edit().is_none());
    }
}

#[test]
fn empty_text_split_still_creates_and_returns_one_distinct_empty_text() {
    let (mut dom, _, parent, text) = fixture("".into(), true);
    let before = dom.snapshot();
    let nodes = dom.usage().managed_nodes();
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::split_text(text, 0), policy(4096), &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("empty Text is still split")
    };
    let Some(DomSourceOutput::Node(new)) = done.output() else {
        panic!("actual empty Text result")
    };
    assert_ne!(text, new);
    drop(lease);
    let after = dom.snapshot();
    assert_eq!(after.node(parent).unwrap().children()[1..3], [text, new]);
    assert_eq!(after.node(new).unwrap().data(), Some(&"".into()));
    assert_eq!(after.node(text).unwrap().data(), Some(&"".into()));
    assert_eq!(dom.usage().managed_nodes(), nodes + 1);
    assert_ne!(after.stamp(), before.stamp());
}

#[test]
fn split_output_refusal_and_precanceled_batch_leave_no_new_text_or_range_movement() {
    let (mut dom, doc, parent, text) = fixture("abcd".into(), true);
    let id = range(&mut dom, doc, (text, 1), (text, 4));
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.split_text(text, 2);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::split_text(text, 2), policy(0), &cancel())
            .unwrap();
        assert_eq!(
            lease.advance(&cancel()).unwrap_err(),
            DomError::SourceOutputLimit
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
    }
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(points(&dom.snapshot(), id), ((text, 1), (text, 4)));
    assert_eq!(
        dom.snapshot().node(parent).unwrap().children(),
        before.node(parent).unwrap().children()
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn split_of_batch_local_text_returns_a_real_ticket_used_by_later_commands() {
    let mut dom = Dom::new(limits(128, 65536)).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let parent = batch.create_element(&doc, "root");
    let original = batch.create_text(&doc, "xy");
    batch.insert_before(&original, &parent, None);
    let tail = batch.split_text(&original, 1);
    batch.replace_data(&tail, 0, 1, "z");
    let commit = apply(&mut dom, batch);
    let old = commit.created(&original).unwrap().unwrap();
    let new = commit.created(&tail).unwrap().unwrap();
    let parent = commit.created(&parent).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(parent).unwrap().children(), [old, new]);
    assert_eq!(snapshot.node(old).unwrap().data(), Some(&"x".into()));
    assert_eq!(snapshot.node(new).unwrap().data(), Some(&"z".into()));
}

#[test]
fn split_rejects_wrong_kind_foreign_stale_and_out_of_bounds_without_creation() {
    let (mut dom, doc, parent, text) = fixture("abcd".into(), true);
    let (_, _, _, foreign) = fixture("foreign".into(), false);
    let mut batch = DomBatch::new();
    let retired = batch.create_text(doc, "retired");
    let retired = apply(&mut dom, batch).created(&retired).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(retired);
    apply(&mut dom, batch);
    for (node, offset, expected) in [
        (parent, usize::MAX, DomError::WrongNodeKind),
        (text, 5, DomError::IndexSize),
        (foreign, 0, DomError::ForeignOwner),
        (retired, 0, DomError::StaleNode),
    ] {
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.split_text(node, offset);
        assert_eq!(dom.apply(batch, &cancel()).unwrap_err(), expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
}

#[test]
fn split_admits_new_identity_and_original_plus_tail_before_deleting_any_data() {
    for resource in [DomResource::ManagedNodes, DomResource::StringCodeUnits] {
        let (mut dom, _, _, text) = fixture("abcd".into(), true);
        let before = dom.snapshot();
        let usage = dom.usage();
        let nodes = usage.managed_nodes();
        let units = usage.string_code_units();
        let (short_nodes, short_units, exact_nodes, exact_units) =
            if resource == DomResource::ManagedNodes {
                (nodes, 65536, nodes + 1, 65536)
            } else {
                (128, units + 1, 128, units + 2)
            };
        dom.set_limits(limits(short_nodes, short_units)).unwrap();
        let mut batch = DomBatch::new();
        batch.split_text(text, 2);
        assert!(
            matches!(dom.apply(batch, &cancel()), Err(DomError::ResourceLimit { resource: actual, .. }) if actual == resource)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        dom.set_limits(limits(exact_nodes, exact_units)).unwrap();
        let mut batch = DomBatch::new();
        let tail = batch.split_text(text, 2);
        let commit = apply(&mut dom, batch);
        assert!(commit.created(&tail).unwrap().is_some());
        assert_eq!(dom.usage().managed_nodes(), nodes + 1);
        assert_eq!(dom.usage().string_code_units(), units);
    }
}

fn deny(request: &DomStyleRequest) -> DomStyleInlineCspObservation {
    DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}

#[test]
fn split_retains_insertion_before_deletion_through_real_style_refusal_drop_and_cancel() {
    let mut dom = Dom::new(limits(128, 65536)).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let style = batch.create_element_ns(&doc, Namespace::html(), "style");
    let text = batch.create_text(&doc, "abcd");
    batch.insert_before(&text, &style, None);
    let commit = apply(&mut dom, batch);
    let (doc, style, text) = (
        commit.document(&doc).unwrap(),
        commit.created(&style).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
    );
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, doc.node(), None),
                policy(4096),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("real connected Style policy")
        };
        lease
            .accept_inline_csp(&request, &deny(&request), &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let id = range(&mut dom, doc, (text, 3), (text, 4));
    let boundary = range(&mut dom, doc, (style, 1), (style, 1));
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut atomic = DomBatch::new();
    atomic.split_text(text, 2);
    assert_eq!(
        dom.apply(atomic, &cancel()).unwrap_err(),
        DomError::StyleCompositionRequired(
            surgeist_html::elements::style::HtmlStyleUpdateAction::CheckInlineCsp,
        )
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let (operation, first, new, work) = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::split_text(text, 2), policy(4096), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("insertion precedes deletion")
        };
        assert_eq!(
            request.source().text().as_ref(),
            &"abcdcd".encode_utf16().collect::<Vec<_>>()
        );
        let snapshot = lease.snapshot();
        let new = snapshot.node(style).unwrap().children()[1];
        assert_eq!(snapshot.node(text).unwrap().data(), Some(&"abcd".into()));
        assert_eq!(points(&snapshot, id), ((text, 3), (text, 4)));
        assert_eq!(points(&snapshot, boundary), ((style, 1), (style, 1)));
        (lease.operation(), request, new, lease.status().spent_work())
    };
    assert!(dom.pending_source_edit().unwrap().has_published());
    let mut lease = dom
        .resume_source_edit(operation, policy(4096), &cancel())
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(reissued)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("same original request")
    };
    assert_eq!(reissued.source().ordinal(), first.source().ordinal());
    assert!(lease.status().spent_work() >= work);
    lease.request_cancel();
    lease
        .accept_inline_csp(&reissued, &deny(&reissued), &cancel())
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(second)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("deletion has its own real Style hook")
    };
    assert_eq!(
        second.source().text().as_ref(),
        &"abcd".encode_utf16().collect::<Vec<_>>()
    );
    let snapshot = lease.snapshot();
    assert_eq!(points(&snapshot, id), ((new, 1), (new, 2)));
    assert_eq!(points(&snapshot, boundary), ((style, 2), (style, 2)));
    lease
        .accept_inline_csp(&second, &deny(&second), &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("same split finishes mandatory tail")
    };
    assert_eq!(done.output(), Some(DomSourceOutput::Node(new)));
    assert_eq!(done.published_node(), Some(new));
    assert!(done.cancellation_requested());
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.snapshot().node(style).unwrap().children(), [text, new]);
    assert_eq!(before.node(style).unwrap().children(), [text]);
    assert!(
        dom.snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
}
