//! Finite Source work cuts; budgets place refusals, while DOM3071 supplies oracles.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn policy(work: u64) -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, work, 128, 4096, 4096, 8)
}
fn deny(request: &DomStyleRequest) -> DomStyleInlineCspObservation {
    DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}
struct Fixture {
    dom: Dom,
    style: NodeId,
    text: NodeId,
    range: DomRangeId,
    boundary: DomRangeId,
}
fn fixture() -> Fixture {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            65536,
            128,
            0,
            0,
            64,
            1_000_000,
            DomNameLimits::new(16_777_216, 16384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_ranges(8)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let style = batch.create_element_ns(&doc, Namespace::html(), "style");
    let text = batch.create_text(&doc, "abcd");
    batch.insert_before(&text, &style, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let doc = commit.document(&doc).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, doc.node(), None),
                policy(20_000_000),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("connected Style policy")
        };
        lease
            .accept_inline_csp(&request, &deny(&request), &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let mut batch = DomBatch::new();
    let range = batch.create_range(doc);
    batch.set_range_start(&range, text, 3);
    batch.set_range_end(&range, text, 4);
    let boundary = batch.create_range(doc);
    batch.set_range_start(&boundary, style, 1);
    batch.set_range_end(&boundary, style, 1);
    let commit = dom.apply(batch, &cancel()).unwrap();
    Fixture {
        dom,
        style,
        text,
        range: commit.created_range(&range).unwrap().unwrap(),
        boundary: commit.created_range(&boundary).unwrap().unwrap(),
    }
}
fn points(snapshot: &DomSnapshot, id: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let r = snapshot.range(id).unwrap();
    (
        (r.start().container(), r.start().offset()),
        (r.end().container(), r.end().offset()),
    )
}
fn prefix(f: &Fixture, new: NodeId) {
    let snapshot = f.dom.snapshot();
    assert_eq!(snapshot.node(f.style).unwrap().children(), [f.text, new]);
    assert_eq!(snapshot.node(new).unwrap().data(), Some(&"cd".into()));
    if snapshot.node(f.text).unwrap().data() == Some(&"abcd".into()) {
        assert_eq!(points(&snapshot, f.range), ((f.text, 3), (f.text, 4)));
        assert_eq!(points(&snapshot, f.boundary), ((f.style, 1), (f.style, 1)));
    } else {
        assert_eq!(snapshot.node(f.text).unwrap().data(), Some(&"ab".into()));
        assert_eq!(points(&snapshot, f.range), ((new, 1), (new, 2)));
        assert_eq!(points(&snapshot, f.boundary), ((f.style, 2), (f.style, 2)));
    }
}

#[test]
fn watched_split_keeps_the_exact_continuation_after_output_admission_refuses() {
    let mut f = fixture();
    let cancel = cancel();
    let inbox = f
        .dom
        .node_retirement_inbox(DomNodeRetirementLimits::new(2_000_000, 20_000), &cancel)
        .unwrap();
    let limits = policy(20_000_000);
    let mut output =
        DomSourceNodeOutputWatch::new(&inbox, 19, DomNodeRetirementLimits::new(2_000_000, 0));
    let (operation, new) = {
        let mut lease = f
            .dom
            .begin_source_edit(DomSourceEdit::split_text(f.text, 2), limits, &cancel)
            .unwrap();
        let DomSourceAdvance::Progress(DomSourceProgress::Request(DomSourceRequest::Style(first))) =
            lease
                .advance_watching_node_for_runtime(&mut output, &cancel)
                .unwrap()
        else {
            panic!("expected insertion Style request");
        };
        let new = lease.snapshot().node(f.style).unwrap().children()[1];
        lease
            .accept_inline_csp(&first, &deny(&first), &cancel)
            .unwrap();
        let DomSourceAdvance::Progress(DomSourceProgress::Request(DomSourceRequest::Style(second))) =
            lease
                .advance_watching_node_for_runtime(&mut output, &cancel)
                .unwrap()
        else {
            panic!("expected deletion Style request");
        };
        assert_eq!(
            (first.source().ordinal(), second.source().ordinal()),
            (0, 1)
        );
        lease
            .accept_inline_csp(&second, &deny(&second), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance_watching_node_for_runtime(&mut output, &cancel),
            Err(DomError::NodeRetirementWorkLimit { limit: 0 })
        ));
        (lease.operation(), new)
    };
    assert!(output.into_watch().is_none());
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
    prefix(&f, new);
    let mut lease = f
        .dom
        .resume_source_edit(operation, limits, &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the acknowledged Style receipts must not replay");
    };
    assert_eq!(done.output(), Some(DomSourceOutput::Node(new)));
    drop(lease);
    prefix(&f, new);
}

#[test]
fn unpublished_split_work_cuts_discard_only_private_creation_and_range_changes() {
    let ceiling = {
        let mut f = fixture();
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::split_text(f.text, 2),
                policy(20_000_000),
                &cancel(),
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Request(DomSourceRequest::Style(_))
        ));
        lease.status().spent_work()
    };
    let mut entered_cuts = 0;
    for work in [ceiling / 4, ceiling / 2, ceiling * 3 / 4] {
        let mut f = fixture();
        let before = f.dom.snapshot();
        let usage = f.dom.usage();
        {
            let result = f.dom.begin_source_edit(
                DomSourceEdit::split_text(f.text, 2),
                policy(work),
                &cancel(),
            );
            match result {
                Err(DomError::SourceWorkLimit { .. }) => (),
                Ok(mut lease) => {
                    assert!(matches!(
                        lease.advance(&cancel()),
                        Err(DomError::SourceWorkLimit { .. })
                    ));
                    assert!(!lease.status().has_published());
                    assert!(lease.status().spent_work() <= work);
                    entered_cuts += 1;
                }
                Err(error) => panic!("expected a work refusal, got {error:?}"),
            }
        }
        assert_eq!(f.dom.snapshot().stamp(), before.stamp());
        assert_eq!(f.dom.usage(), usage);
        assert!(f.dom.pending_source_edit().is_none());
        assert_eq!(f.dom.snapshot().node(f.style).unwrap().children(), [f.text]);
        assert_eq!(
            points(&f.dom.snapshot(), f.range),
            ((f.text, 3), (f.text, 4))
        );
        assert_eq!(
            points(&f.dom.snapshot(), f.boundary),
            ((f.style, 1), (f.style, 1))
        );
    }
    assert!(
        entered_cuts > 0,
        "a cut must execute the real unpublished invocation"
    );
}

#[test]
fn published_split_work_cuts_drop_and_resume_one_identity_without_replaying_hooks_or_deletion() {
    let mut f = fixture();
    let before = f.dom.snapshot();
    let nodes = f.dom.usage().managed_nodes();
    let (operation, new, mut work) = {
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::split_text(f.text, 2),
                policy(20_000_000),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("real insertion prefix")
        };
        assert_eq!(request.source().ordinal(), 0);
        assert_eq!(
            request.source().text().as_ref(),
            &"abcdcd".encode_utf16().collect::<Vec<_>>()
        );
        lease
            .accept_inline_csp(&request, &deny(&request), &cancel())
            .unwrap();
        (
            lease.operation(),
            lease.snapshot().node(f.style).unwrap().children()[1],
            lease.status().spent_work(),
        )
    };
    prefix(&f, new);
    let mut delta = 1;
    let mut refusals = 0;
    let mut second_serviced = false;
    let mut done = None;
    // A short fixed doubling sequence places cuts without treating a work count
    // as a semantic oracle. The actual original counter is never reset.
    for _ in 0..20 {
        let ceiling = work + delta;
        let mut outcome = None;
        {
            let resumed = f
                .dom
                .resume_source_edit(operation, policy(ceiling), &cancel());
            match resumed {
                Err(DomError::SourceWorkLimit { .. }) => {
                    refusals += 1;
                }
                Err(error) => panic!("original resume: {error:?}"),
                Ok(mut lease) => {
                    assert_eq!(lease.operation(), operation);
                    loop {
                        match lease.advance(&cancel()) {
                            Err(DomError::SourceWorkLimit { .. }) => {
                                refusals += 1;
                                break;
                            }
                            Err(error) => panic!("original split: {error:?}"),
                            Ok(DomSourceProgress::Request(DomSourceRequest::Style(request))) => {
                                assert!(
                                    !second_serviced,
                                    "a completed deletion hook cannot run twice"
                                );
                                assert_eq!(
                                    request.source().ordinal(),
                                    1,
                                    "accepted insertion hook cannot replay"
                                );
                                assert_eq!(
                                    request.source().text().as_ref(),
                                    &"abcd".encode_utf16().collect::<Vec<_>>()
                                );
                                match lease.accept_inline_csp(&request, &deny(&request), &cancel())
                                {
                                    Ok(()) => second_serviced = true,
                                    Err(DomError::SourceWorkLimit { .. }) => {
                                        refusals += 1;
                                        break;
                                    }
                                    Err(error) => panic!("real policy result: {error:?}"),
                                }
                            }
                            Ok(DomSourceProgress::Complete(value)) => {
                                outcome = Some(value);
                                break;
                            }
                            Ok(other) => panic!("actual split result: {other:?}"),
                        }
                    }
                }
            }
        }
        prefix(&f, new);
        if let Some(value) = outcome {
            done = Some(value);
            break;
        }
        let status = f.dom.pending_source_edit().unwrap();
        assert_eq!(status.operation(), operation);
        assert!(status.has_published());
        assert!(status.spent_work() >= work);
        assert!(status.spent_work() <= ceiling);
        work = status.spent_work();
        delta *= 2;
    }
    let done = done.expect("finite raised allowance finishes the same short split");
    assert!(refusals > 0);
    assert!(second_serviced);
    assert_eq!(done.operation(), operation);
    assert_eq!(done.output(), Some(DomSourceOutput::Node(new)));
    assert_eq!(f.dom.usage().managed_nodes(), nodes + 1);
    assert!(f.dom.pending_source_edit().is_none());
    assert_eq!(
        f.dom.snapshot().node(f.text).unwrap().data(),
        Some(&"ab".into())
    );
    assert_eq!(points(&f.dom.snapshot(), f.range), ((new, 1), (new, 2)));
    let changes = f.dom.changes_since(before.stamp()).unwrap();
    assert_eq!(
        changes
            .changes()
            .iter()
            .filter(|change| **change == DomChange::Created(new))
            .count(),
        1
    );
}
