//! Public caller-owned Slot observation work, backing and prefix contracts.
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            65536,
            512,
            512,
            512,
            16,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn limits() -> DomSlotQueryLimits {
    DomSlotQueryLimits::new(32, 64).unwrap()
}
fn hosted_source(dom: &mut Dom) -> DomHostedSource {
    dom.register_hosted_source(
        DomHostedLimits::new(
            DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
            16_000_000,
            16_000_000,
            4_000_000,
        ),
        &cancel(),
    )
    .unwrap()
}

struct Fixture {
    dom: Dom,
    slot: NodeId,
    nested: NodeId,
    text: NodeId,
    element: NodeId,
}
fn fallback() -> Fixture {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&host, &d, None);
    let root = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Manual,
            false,
            false,
            false,
        ),
    );
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    let nested = b.create_element_ns(&d, Namespace::html(), "slot");
    let text = b.create_text(&d, "fallback");
    let element = b.create_element_ns(&d, Namespace::html(), "span");
    let comment = b.create_comment(&d, "excluded");
    b.insert_before(&slot, &root, None);
    b.insert_before(&nested, &slot, None);
    b.insert_before(&text, &nested, None);
    b.insert_before(&comment, &nested, None);
    b.insert_before(&element, &nested, None);
    let c = dom.apply(b, &cancel()).unwrap();
    Fixture {
        slot: c.created(&slot).unwrap().unwrap(),
        nested: c.created(&nested).unwrap().unwrap(),
        text: c.created(&text).unwrap().unwrap(),
        element: c.created(&element).unwrap().unwrap(),
        dom,
    }
}
#[derive(Debug)]
enum Failure {
    Native(DomError),
    Refused(Rc<()>),
}
impl From<DomError> for Failure {
    fn from(value: DomError) -> Self {
        Self::Native(value)
    }
}
struct Backing {
    bytes: usize,
    live: Rc<Cell<usize>>,
}
impl Drop for Backing {
    fn drop(&mut self) {
        self.live
            .set(self.live.get().checked_sub(self.bytes).unwrap());
    }
}
fn reserve(bytes: usize, live: &Rc<Cell<usize>>, peak: &Cell<usize>) -> Backing {
    let total = live.get().checked_add(bytes).unwrap();
    live.set(total);
    peak.set(peak.get().max(total));
    Backing {
        bytes,
        live: live.clone(),
    }
}
#[test]
fn fresh_flatten_visits_nested_slottable_fallback_in_order_and_filters_from_admitted_kind() {
    let f = fallback();
    let snapshot = f.dom.snapshot();
    let slot = snapshot.slot(f.slot).unwrap();
    let mut stored = Vec::new();
    slot.visit_assigned_nodes_with(
        || Ok::<_, DomError>(()),
        |n| {
            stored.push(n.id());
            Ok(())
        },
    )
    .unwrap();
    assert!(stored.is_empty());
    let mut nodes = Vec::new();
    let mut elements = Vec::new();
    slot.visit_flattened_slottables_with(
        limits(),
        || Ok::<_, DomError>(()),
        |_| Ok(()),
        |n| {
            nodes.push(n.id());
            if matches!(n.kind(), DomNodeKind::Element(_)) {
                elements.push(n.id());
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(nodes, [f.text, f.element]);
    assert_eq!(elements, [f.element]);
    assert!(!nodes.contains(&f.nested));
    assert_eq!(
        slot.flattened_slottables(limits(), &cancel())
            .unwrap()
            .nodes(),
        nodes
    );
}
#[test]
fn work_and_visitor_refusals_preserve_original_error_and_accepted_prefix() {
    let f = fallback();
    let snapshot = f.dom.snapshot();
    let marker = Rc::new(());
    let mut output = Vec::new();
    let visits = Cell::new(0);
    let failure = snapshot
        .slot(f.slot)
        .unwrap()
        .visit_flattened_slottables_with(
            limits(),
            || Ok::<_, Failure>(()),
            |_| Ok(()),
            |n| {
                visits.set(visits.get() + 1);
                if n.id() == f.element {
                    return Err(Failure::Refused(marker.clone()));
                }
                output.push(n.id());
                Ok(())
            },
        );
    assert!(matches!(failure,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker)));
    assert_eq!(visits.get(), 2);
    assert_eq!(output, [f.text]);
    let steps = Cell::new(0);
    let visits = Cell::new(0);
    let failure = snapshot
        .slot(f.slot)
        .unwrap()
        .visit_flattened_slottables_with(
            limits(),
            || {
                steps.set(steps.get() + 1);
                Err(Failure::Refused(marker.clone()))
            },
            |_| Ok(()),
            |_| {
                visits.set(visits.get() + 1);
                Ok(())
            },
        );
    assert!(matches!(failure,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker)));
    assert_eq!(steps.get(), 1);
    assert_eq!(visits.get(), 0);
}
#[test]
fn scratch_replacements_and_nested_frames_keep_backing_live_and_release_on_refusal() {
    let f = fallback();
    let snapshot = f.dom.snapshot();
    let live = Rc::new(Cell::new(0));
    let peak = Cell::new(0);
    let reservations = Cell::new(0);
    let marker = Rc::new(());
    let failure = snapshot
        .slot(f.slot)
        .unwrap()
        .visit_flattened_slottables_with(
            limits(),
            || Ok::<_, Failure>(()),
            |bytes| {
                reservations.set(reservations.get() + 1);
                Ok(reserve(bytes, &live, &peak))
            },
            |_| {
                assert!(live.get() > 0);
                Err(Failure::Refused(marker.clone()))
            },
        );
    assert!(matches!(failure,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker)));
    assert!(reservations.get() >= 4);
    assert!(peak.get() > 2 * size_of::<NodeId>());
    assert_eq!(live.get(), 0);
    let failed_reservation = Cell::new(0);
    let visits = Cell::new(0);
    let failure = snapshot
        .slot(f.slot)
        .unwrap()
        .visit_flattened_slottables_with(
            limits(),
            || Ok::<_, Failure>(()),
            |bytes| {
                failed_reservation.set(failed_reservation.get() + 1);
                if failed_reservation.get() == 2 {
                    assert!(live.get() > 0);
                    return Err(Failure::Refused(marker.clone()));
                }
                Ok(reserve(bytes, &live, &peak))
            },
            |_| {
                visits.set(visits.get() + 1);
                Ok(())
            },
        );
    assert!(matches!(failure,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker)));
    assert_eq!(failed_reservation.get(), 2);
    assert_eq!(visits.get(), 0);
    assert_eq!(live.get(), 0);
}
#[test]
fn output_exact_and_one_short_preserve_prefix_and_cancellation_cleans_scratch() {
    let f = fallback();
    let snapshot = f.dom.snapshot();
    for (limit, expected) in [
        (2, Ok(())),
        (
            1,
            Err(DomError::ResourceLimit {
                resource: DomResource::SlotQueryOutput,
                limit: 1,
            }),
        ),
    ] {
        let mut output = Vec::new();
        let result = snapshot
            .slot(f.slot)
            .unwrap()
            .visit_flattened_slottables_with(
                DomSlotQueryLimits::new(limit, 64).unwrap(),
                || Ok::<_, DomError>(()),
                |_| Ok(()),
                |n| {
                    output.push(n.id());
                    Ok(())
                },
            );
        assert_eq!(result, expected);
        assert_eq!(
            output,
            if limit == 2 {
                vec![f.text, f.element]
            } else {
                vec![f.text]
            }
        );
    }
    let cancelled = cancel();
    let live = Rc::new(Cell::new(0));
    let peak = Cell::new(0);
    let mut output = Vec::new();
    let result = snapshot
        .slot(f.slot)
        .unwrap()
        .visit_flattened_slottables_with(
            limits(),
            || {
                if cancelled.load(Ordering::Relaxed) {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
            |bytes| Ok(reserve(bytes, &live, &peak)),
            |n| {
                output.push(n.id());
                cancelled.store(true, Ordering::Relaxed);
                Ok(())
            },
        );
    assert_eq!(result, Err(DomError::Cancelled));
    assert_eq!(output, [f.text]);
    assert_eq!(live.get(), 0);
}
#[test]
fn admitted_assignment_command_preserves_duplicate_operands_and_exact_copy_extent() {
    let mut f = fallback();
    let source = hosted_source(&mut f.dom);
    let input = [f.text, f.text, f.element];
    let steps = Cell::new(0);
    let bytes = Cell::new(0);
    let prepared = DomSourceEdit::assign_slot_with(
        f.slot,
        &input,
        || {
            steps.set(steps.get() + 1);
            Ok::<_, DomError>(())
        },
        |extent| {
            bytes.set(extent);
            Ok((
                source.retain_runtime_payload(&mut f.dom, extent, &cancel())?,
                extent,
            ))
        },
    )
    .unwrap();
    assert_eq!(steps.get(), input.len() + 1);
    assert_eq!(bytes.get(), input.len() * size_of::<NodeOperand>());
    drop(prepared);
    let marker = Rc::new(());
    let visits = Cell::new(0);
    let failed = DomSourceEdit::assign_slot_with(
        f.slot,
        &input,
        || {
            visits.set(visits.get() + 1);
            Ok::<_, Failure>(())
        },
        |_| Err::<(DomHostedRuntimeRetention, ()), _>(Failure::Refused(marker.clone())),
    );
    assert!(matches!(failed,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker)));
    assert_eq!(visits.get(), 1);
}

#[test]
fn assigned_slot_uses_fresh_named_and_manual_observations_and_open_root_presentation() {
    for assignment in [HtmlSlotAssignment::Named, HtmlSlotAssignment::Manual] {
        for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
            let mut dom = owner();
            let mut b = DomBatch::new();
            let d = b.create_document();
            let host = b.create_element_ns(&d, Namespace::html(), "div");
            b.insert_before(&host, &d, None);
            let root = b.attach_shadow(
                &host,
                HtmlShadowRootProperties::new(mode, assignment, false, false, false),
            );
            let slot = b.create_element_ns(&d, Namespace::html(), "slot");
            b.insert_before(&slot, &root, None);
            let child = b.create_text(&d, "assigned");
            b.insert_before(&child, &host, None);
            if assignment == HtmlSlotAssignment::Manual {
                b.assign_slot(&slot, vec![(&child).into()]);
            }
            let c = dom.apply(b, &cancel()).unwrap();
            let child = c.created(&child).unwrap().unwrap();
            let slot = c.created(&slot).unwrap().unwrap();
            let snapshot = dom.snapshot();
            let child = snapshot.node(child).unwrap();
            assert_eq!(
                child
                    .find_slot_with(false, || Ok::<_, DomError>(()))
                    .unwrap(),
                Some(slot)
            );
            assert_eq!(
                child.assigned_slot_with(|| Ok::<_, DomError>(())).unwrap(),
                if mode == HtmlShadowRootMode::Open {
                    Some(slot)
                } else {
                    None
                }
            );
            let marker = Rc::new(());
            let steps = Cell::new(0);
            let failed = child.assigned_slot_with(|| {
                steps.set(steps.get() + 1);
                Err(Failure::Refused(marker.clone()))
            });
            assert!(
                matches!(failed,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker))
            );
            assert_eq!(steps.get(), 1);
            let mut stored = Vec::new();
            snapshot
                .slot(slot)
                .unwrap()
                .visit_assigned_nodes_with(
                    || Ok::<_, DomError>(()),
                    |n| {
                        stored.push(n.id());
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(stored, [child.id()]);
        }
    }
}
#[test]
fn stored_history_and_captured_results_survive_fresh_manual_transfer_and_destruction() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let h1 = b.create_element_ns(&d, Namespace::html(), "div");
    let h2 = b.create_element_ns(&d, Namespace::html(), "div");
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&top, &d, None);
    b.insert_before(&h1, &top, None);
    b.insert_before(&h2, &top, None);
    let properties = HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Open,
        HtmlSlotAssignment::Manual,
        false,
        false,
        false,
    );
    let r1 = b.attach_shadow(&h1, properties);
    let r2 = b.attach_shadow(&h2, properties);
    let s1 = b.create_element_ns(&d, Namespace::html(), "slot");
    let s2 = b.create_element_ns(&d, Namespace::html(), "slot");
    b.insert_before(&s1, &r1, None);
    b.insert_before(&s2, &r2, None);
    let child = b.create_text(&d, "stored");
    let fallback = b.create_text(&d, "fallback");
    b.insert_before(&child, &h1, None);
    b.insert_before(&fallback, &s1, None);
    b.assign_slot(&s1, vec![(&child).into()]);
    let c = dom.apply(b, &cancel()).unwrap();
    let s1 = c.created(&s1).unwrap().unwrap();
    let s2 = c.created(&s2).unwrap().unwrap();
    let child = c.created(&child).unwrap().unwrap();
    let fallback = c.created(&fallback).unwrap().unwrap();
    let old = dom.snapshot();
    let historical = old
        .slot(s1)
        .unwrap()
        .flattened_slottables(limits(), &cancel())
        .unwrap();
    let mut b = DomBatch::new();
    b.assign_slot(s2, vec![child.into()]);
    dom.apply(b, &cancel()).unwrap();
    let current = dom.snapshot();
    let mut stored = Vec::new();
    current
        .slot(s1)
        .unwrap()
        .visit_assigned_nodes_with(
            || Ok::<_, DomError>(()),
            |n| {
                stored.push(n.id());
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(stored, [child]);
    let mut fresh = Vec::new();
    current
        .slot(s1)
        .unwrap()
        .visit_flattened_slottables_with(
            limits(),
            || Ok::<_, DomError>(()),
            |_| Ok(()),
            |n| {
                fresh.push(n.id());
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(fresh, [fallback]);
    assert_eq!(historical.nodes(), [child]);
    assert!(historical.source().node(child).is_ok());
    assert_eq!(
        current
            .node(child)
            .unwrap()
            .assigned_slot_with(|| Ok::<_, DomError>(()))
            .unwrap(),
        None
    );
    let wrong = old.node(child).unwrap().as_slot();
    assert!(wrong.is_none());
    let failed = old
        .node(s1)
        .unwrap()
        .assigned_slot_with(|| Ok::<_, Failure>(()));
    assert!(failed.is_ok());
    let failure = old
        .node(c.document(&d).unwrap().node())
        .unwrap()
        .assigned_slot_with(|| Ok::<_, Failure>(()));
    assert!(matches!(
        failure,
        Err(Failure::Native(DomError::WrongNodeKind))
    ));
    let mut destroy = DomBatch::new();
    destroy.detach(child);
    destroy.destroy(child);
    dom.apply(destroy, &cancel()).unwrap();
    assert_eq!(dom.snapshot().node(child).unwrap_err(), DomError::StaleNode);
    assert_eq!(historical.nodes(), [child]);
    assert!(historical.source().node(child).is_ok());
}

#[test]
fn scratch_reference_exact_extent_succeeds_and_one_short_refuses_before_output() {
    // Outer candidate + frame, nested two candidates + frame = five retained
    // references; the Comment contributes none to slottable fallback.
    let f = fallback();
    let snapshot = f.dom.snapshot();
    for (scratch, expected) in [
        (5, Ok(())),
        (
            4,
            Err(DomError::ResourceLimit {
                resource: DomResource::SlotQueryScratch,
                limit: 4,
            }),
        ),
    ] {
        let mut output = Vec::new();
        let result = snapshot
            .slot(f.slot)
            .unwrap()
            .visit_flattened_slottables_with(
                DomSlotQueryLimits::new(2, scratch).unwrap(),
                || Ok::<_, DomError>(()),
                |_| Ok(()),
                |n| {
                    output.push(n.id());
                    Ok(())
                },
            );
        assert_eq!(result, expected);
        assert_eq!(
            output,
            if scratch == 5 {
                vec![f.text, f.element]
            } else {
                vec![]
            }
        );
    }
}

#[test]
fn operand_copy_exact_backing_and_one_short_use_original_reservation_and_release_on_work_refusal() {
    let mut f = fallback();
    let source = hosted_source(&mut f.dom);
    let input = [f.text, f.text, f.element];
    let required = input.len() * size_of::<NodeOperand>();
    let marker = Rc::new(());
    for allowance in [required, required - 1] {
        let live = Rc::new(Cell::new(0));
        let peak = Cell::new(0);
        let steps = Cell::new(0);
        let result = DomSourceEdit::assign_slot_with(
            f.slot,
            &input,
            || {
                steps.set(steps.get() + 1);
                Ok::<_, Failure>(())
            },
            |bytes| {
                assert_eq!(bytes, required);
                if bytes > allowance {
                    return Err(Failure::Refused(marker.clone()));
                }
                Ok((
                    source.retain_runtime_payload(&mut f.dom, bytes, &cancel())?,
                    reserve(bytes, &live, &peak),
                ))
            },
        );
        if allowance == required {
            let command = result.unwrap();
            assert_eq!(live.get(), required);
            assert_eq!(steps.get(), 4);
            drop(command);
        } else {
            assert!(
                matches!(result,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker))
            );
            assert_eq!(steps.get(), 1);
        }
        assert_eq!(live.get(), 0);
    }
    let live = Rc::new(Cell::new(0));
    let peak = Cell::new(0);
    let steps = Cell::new(0);
    let result = DomSourceEdit::assign_slot_with(
        f.slot,
        &input,
        || {
            steps.set(steps.get() + 1);
            if steps.get() == 3 {
                Err(Failure::Refused(marker.clone()))
            } else {
                Ok(())
            }
        },
        |bytes| {
            Ok((
                source.retain_runtime_payload(&mut f.dom, bytes, &cancel())?,
                reserve(bytes, &live, &peak),
            ))
        },
    );
    assert!(matches!(result,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker)));
    assert_eq!(steps.get(), 3);
    assert_eq!(live.get(), 0);
    assert_eq!(input, [f.text, f.text, f.element]);
}

#[test]
fn long_named_lookup_uses_original_work_and_preserves_refusal_during_name_comparison() {
    // One Slot and one named light child keep association/tree shape fixed.
    // The same finite allowance admits a short name, while the long matching
    // name requires inspecting its UTF-16 content through the owner comparator.
    for name in ["x".to_owned(), "x".repeat(1024)] {
        let mut dom = owner();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let host = b.create_element_ns(&d, Namespace::html(), "div");
        b.insert_before(&host, &d, None);
        let root = b.attach_shadow(
            &host,
            HtmlShadowRootProperties::new(
                HtmlShadowRootMode::Open,
                HtmlSlotAssignment::Named,
                false,
                false,
                false,
            ),
        );
        let slot = b.create_element_ns(&d, Namespace::html(), "slot");
        b.set_attribute(&slot, "name", name.clone());
        b.insert_before(&slot, &root, None);
        let child = b.create_element_ns(&d, Namespace::html(), "span");
        b.set_attribute(&child, "slot", name.clone());
        b.insert_before(&child, &host, None);
        let c = dom.apply(b, &cancel()).unwrap();
        let slot = c.created(&slot).unwrap().unwrap();
        let child = c.created(&child).unwrap().unwrap();
        let snapshot = dom.snapshot();
        let child = snapshot.node(child).unwrap();
        let marker = Rc::new(());
        let accepted = Cell::new(0);
        let refused = Cell::new(0);
        let result = child.assigned_slot_with(|| {
            if accepted.get() == 64 {
                refused.set(refused.get() + 1);
                return Err(Failure::Refused(marker.clone()));
            }
            accepted.set(accepted.get() + 1);
            Ok(())
        });
        if name.len() == 1 {
            assert!(matches!(result,Ok(Some(id)) if id==slot));
            assert_eq!(refused.get(), 0);
        } else {
            assert!(
                matches!(result,Err(Failure::Refused(ref original)) if Rc::ptr_eq(original,&marker))
            );
            assert_eq!(accepted.get(), 64);
            assert_eq!(refused.get(), 1);
        }
        let full = child.assigned_slot_with(|| Ok::<_, DomError>(())).unwrap();
        assert_eq!(full, Some(slot));
    }
}

#[test]
fn prepared_assignment_transfers_at_source_installation_and_cancellation_releases_local_credit() {
    for cancelled in [false, true] {
        let mut f = fallback();
        let source = hosted_source(&mut f.dom);
        let input = [f.text, f.text, f.element];
        let live = Rc::new(Cell::new(0));
        let peak = Cell::new(0);
        let prepared = DomSourceEdit::assign_slot_with(
            f.slot,
            &input,
            || Ok::<_, DomError>(()),
            |bytes| {
                Ok((
                    source.retain_runtime_payload(&mut f.dom, bytes, &cancel())?,
                    reserve(bytes, &live, &peak),
                ))
            },
        )
        .unwrap();
        assert_eq!(live.get(), input.len() * size_of::<NodeOperand>());
        let cancellation = AtomicBool::new(cancelled);
        let admission = DomSourceLimits::new(32_000_000, 2_000_000, 10_000_000, 128, 8192, 8192, 4);
        let result = f.dom.begin_source_edit(prepared, admission, &cancellation);
        assert_eq!(
            live.get(),
            0,
            "local prepared credit ends at installation or refusal"
        );
        if cancelled {
            assert!(matches!(result, Err(DomError::Cancelled)));
        } else {
            let mut lease = result.unwrap();
            match lease.advance(&cancellation).unwrap() {
                DomSourceProgress::Complete(done) => {
                    done.commit();
                }
                unexpected => {
                    panic!("Slot assignment requires no external producer: {unexpected:?}")
                }
            }
            drop(lease);
            assert_eq!(
                f.dom.snapshot().slot(f.slot).unwrap().manual_assignments(),
                [f.text, f.element]
            );
        }
    }
}

#[test]
fn prepared_assignment_from_foreign_issuer_refuses_before_source_installation() {
    let mut f = fallback();
    let _own_source = hosted_source(&mut f.dom);
    let mut other = owner();
    let foreign = hosted_source(&mut other);
    let input = [f.text];
    let live = Rc::new(Cell::new(0));
    let peak = Cell::new(0);
    let prepared = DomSourceEdit::assign_slot_with(
        f.slot,
        &input,
        || Ok::<_, DomError>(()),
        |bytes| {
            Ok((
                foreign.retain_runtime_payload(&mut other, bytes, &cancel())?,
                reserve(bytes, &live, &peak),
            ))
        },
    )
    .unwrap();
    let before = f.dom.snapshot().stamp();
    assert!(matches!(
        f.dom.begin_source_edit(
            prepared,
            DomSourceLimits::new(32_000_000, 2_000_000, 10_000_000, 128, 8192, 8192, 4),
            &cancel()
        ),
        Err(DomError::HostedSourceUnavailable)
    ));
    assert_eq!(live.get(), 0);
    assert!(f.dom.pending_source_edit().is_none());
    assert_eq!(f.dom.snapshot().stamp(), before);
}
