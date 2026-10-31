//! Actual signed atomic margins preserve raw geometry and legal soft breaks.

use super::*;

struct Expected {
    width: f64,
    start: f64,
    end: f64,
    flow: f64,
    occupied_end: f64,
    border_x: f64,
    atomic_flow: f64,
    atomic_occupied: (f64, f64),
}

fn assert_complete<S: LayoutScalar>(paragraph: &PreparedParagraph, expected: Expected) {
    let batch = compute::<S>(paragraph, expected.width, expected.start, expected.end);
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(
        placed.analyzed_source().identity(),
        paragraph.prepared.analyzed_source().identity()
    );
    assert_eq!(placed.source().text(), &text::Text::from("A"));
    assert_eq!(
        placed.lines().len(),
        1,
        "actual signed CSS margin-box result must fit on its same complete candidate"
    );
    let line = whole_line(&batch);
    assert_eq!(line.prepared_text(), paragraph.prepared);
    let selected = line.selected_line();
    let request = selected.request();
    let segment = paragraph.prepared.source_segments().next().unwrap();
    assert_eq!(request.start, segment.start());
    assert_eq!(request.end, segment.end());
    assert_eq!(
        request.participants,
        paragraph.prepared.participants().collect::<Vec<_>>()
    );
    assert_eq!(request.context.available_inline, Some(expected.width));
    assert!(request.alternative.is_none() && request.incoming.is_none());
    assert_eq!(line.feedback().advance, expected.flow);
    assert_eq!(
        (line.feedback().occupied_start, line.feedback().occupied_end),
        (0.0, expected.occupied_end)
    );
    // Legacy/raw InlineMeasure stays its own observation domain. The new
    // operation-specific fit permission must not rewrite this occupied union.
    let raw = line.measurements().fit();
    assert_eq!(raw.advance(), expected.flow);
    assert_eq!(
        (raw.occupied_start(), raw.occupied_end()),
        (0.0, expected.occupied_end)
    );
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (10.0, 12.0)
    );
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            let instance = run.effective_instance().unwrap();
            let metrics = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(text::Range::new(0, 1), 35, 0.0, 10.0)]);
    let slots = line.atomic_slots().collect::<Vec<_>>();
    let [slot] = slots.as_slice() else {
        panic!("same actual selected line owns its genuine atomic");
    };
    assert_eq!(
        slot.occurrence,
        paragraph.prepared.atomic_views().next().unwrap().handle()
    );
    let text::AtomicLineAllocation::InFlow {
        flow, box_geometry, ..
    } = slot.allocation
    else {
        panic!("in-flow source atomic");
    };
    assert_eq!(slot.origin.x, 10.0);
    assert_eq!(flow.advance, expected.atomic_flow);
    assert_eq!(
        (flow.occupied_start, flow.occupied_end),
        expected.atomic_occupied
    );
    assert_eq!((flow.before_baseline, flow.after_baseline), (10.0, 0.0));
    assert_eq!(
        slot.origin.x + box_geometry.border.origin.x,
        expected.border_x
    );
    assert_eq!(slot.origin.y + box_geometry.border.origin.y, 0.0);
    assert_eq!(box_geometry.border.size, text::Size::new(10.0, 10.0));
    assert_eq!(placed.inline_boxes().len(), 1);
    assert_eq!(
        placed.inline_boxes()[0].rect(),
        text::Rect::new(expected.border_x, 0.0, 10.0, 10.0)
    );
    // This is ORIGINAL-local font metric geometry, not Cluster's actual full
    // line band. Real baseline10 and Ahem8/+2 give2..12 even when flow retracts.
    let range = placed
        .source_range_fragments(&paragraph.origin(1), text::Range::new(0, 1))
        .unwrap();
    assert_eq!(
        range
            .units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [text::Rect::new(0.0, 2.0, 10.0, 10.0)]
    );
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            root.size,
            Size::new(S::from_f64(expected.width), S::from_f64(12.0))
        );
        let atomic = entries
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(
            atomic.location,
            Point::new(S::from_f64(expected.border_x), S::ZERO)
        );
        assert_eq!(atomic.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
        assert_eq!(
            atomic.margin,
            Edges {
                left: S::from_f64(expected.start),
                right: S::from_f64(expected.end),
                ..Edges::ZERO
            }
        );
    }
}

fn assert_real_split<S: LayoutScalar>(paragraph: &PreparedParagraph, width: f64, end: f64) {
    let batch = compute::<S>(paragraph, width, 0.0, end);
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(
        placed.lines().len(),
        2,
        "real Allowed before-atomic cut is not one unbreakable fallback group"
    );
    let mut lines: Vec<Option<text::FinalizedLine>> = vec![None, None];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let index = fragment.line_index();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        if let Some(old) = &lines[index] {
            assert_eq!(old, &line);
        } else {
            lines[index] = Some(line);
        }
    }
    let first = lines[0].as_ref().expect("real source A line");
    assert_eq!(first.feedback().advance, 10.0);
    assert_eq!(first.atomic_slots().count(), 0);
    // An atomic-only line has no Original/control source fragment. Its actual
    // placed atomic and native node output establish the separate second line;
    // source-fragment readback above belongs only to the original A line.
    let boxes = placed.inline_boxes();
    let [atomic] = boxes.as_slice() else {
        panic!("one actual placed atomic");
    };
    assert_eq!(atomic.kind(), text::InlineBoxKind::InFlow);
    assert_eq!(atomic.line(), 1);
    assert_eq!(atomic.rect(), text::Rect::new(0.0, 10.0, 10.0, 10.0));
    let output = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 2)
        .unwrap()
        .output();
    assert_eq!(output.location, Point::new(S::ZERO, S::from_f64(10.0)));
    assert_eq!(output.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
    assert_eq!(output.margin.right, S::from_f64(end));
    let selected = first.selected_line();
    let text::PreparedTermination::Ordinary(offer) = &selected.request().termination else {
        panic!("actual ordinary atomic-edge cut");
    };
    assert_eq!(offer.view().kind(), text::PreparedBreakKind::Allowed);
    assert_eq!(offer.view().boundary().view().participant_offset(), 1);
}

fn preservation<S: LayoutScalar>() {
    let paragraph = paragraph();
    actual_source_premises(&paragraph);
    assert_complete::<S>(
        &paragraph,
        Expected {
            width: 20.0,
            start: 0.0,
            end: 0.0,
            flow: 20.0,
            occupied_end: 20.0,
            border_x: 10.0,
            atomic_flow: 10.0,
            atomic_occupied: (0.0, 10.0),
        },
    );
    assert_complete::<S>(
        &paragraph,
        Expected {
            width: 25.0,
            start: 0.0,
            end: 5.0,
            flow: 25.0,
            occupied_end: 25.0,
            border_x: 10.0,
            atomic_flow: 15.0,
            atomic_occupied: (0.0, 15.0),
        },
    );
    assert_real_split::<S>(&paragraph, 19.0, 0.0);
    assert_real_split::<S>(&paragraph, 24.0, 5.0);
}

fn signed_start<S: LayoutScalar>() {
    let paragraph = paragraph();
    actual_source_premises(&paragraph);
    assert_complete::<S>(
        &paragraph,
        Expected {
            width: 15.0,
            start: -5.0,
            end: 0.0,
            flow: 15.0,
            occupied_end: 15.0,
            border_x: 5.0,
            atomic_flow: 5.0,
            atomic_occupied: (-5.0, 5.0),
        },
    );
    assert_complete::<S>(
        &paragraph,
        Expected {
            width: 15.0,
            start: 3.0,
            end: -8.0,
            flow: 15.0,
            occupied_end: 23.0,
            border_x: 13.0,
            atomic_flow: 5.0,
            atomic_occupied: (0.0, 13.0),
        },
    );
}

fn later_retraction<S: LayoutScalar>() {
    let paragraph = paragraph();
    actual_source_premises(&paragraph);
    // A alone overflows5. With NO margin the actual break admits two separate
    // overflow units; the negative result must not be attributed to that
    // fallback coalescing everything. The full negative candidate's flow is
    // genuinely5, although the earlier candidate flow was10 and raw bounds20.
    assert_real_split::<S>(&paragraph, 5.0, 0.0);
    assert_complete::<S>(
        &paragraph,
        Expected {
            width: 5.0,
            start: 0.0,
            end: -15.0,
            flow: 5.0,
            occupied_end: 20.0,
            border_x: 10.0,
            atomic_flow: -5.0,
            atomic_occupied: (-5.0, 10.0),
        },
    );
}

#[test]
fn zero_and_positive_atomic_margins_keep_fit_and_genuine_soft_split_both_scalars() {
    preservation::<f32>();
    preservation::<f64>();
}
#[test]
fn actual_atomic_start_margins_preserve_border_overlap_overhang_and_signed_fit_both_scalars() {
    signed_start::<f32>();
    signed_start::<f64>();
}
#[test]
fn a_later_negative_atomic_margin_retracts_the_same_candidate_past_an_earlier_overflow_both_scalars()
 {
    later_retraction::<f32>();
    later_retraction::<f64>();
}
