//! Non-atomic inline boundaries stay transparent to line-edge SPACE trimming.
//! CSS Text3 sections 1.5 and 4.1.2 define the independent trimming expectations.

use super::*;

fn actual_pair(fixture: &Fixture) -> (text::PreparedControl, text::PreparedControl) {
    let start = fixture
        .binding
        .controls
        .iter()
        .find(|row| row.node == 20)
        .unwrap()
        .occurrence
        .clone();
    let end = fixture
        .binding
        .controls
        .iter()
        .find(|row| row.node == 21)
        .unwrap()
        .occurrence
        .clone();
    assert_ne!(start, end);
    assert_eq!(
        start.view().kind(),
        &text::ParagraphControlKind::InlineBoundaryStart
    );
    assert_eq!(
        end.view().kind(),
        &text::ParagraphControlKind::InlineBoundaryEnd
    );
    let a = start.captured_control().expect("genuine captured start");
    let b = end.captured_control().expect("genuine captured end");
    let scope = a.scope();
    assert_eq!(b.scope(), scope);
    assert_eq!(scope.entry_control(), Some(a));
    assert_eq!(scope.exit_control(), Some(b));
    assert_eq!(scope.parent(), Some(scope.domain()));
    assert_ne!(scope, scope.domain());
    assert_eq!(start.prepared_text(), fixture.binding.prepared);
    assert_eq!(end.prepared_text(), fixture.binding.prepared);
    (start, end)
}

fn raw(line: &text::FinalizedLine) -> Vec<(text::Range, u32, f32, f64)> {
    qualify(line);
    line.run_views()
        .flat_map(|run| {
            let x = run.origin().x;
            run.glyphs()
                .map(move |g| (g.range(), g.id(), g.advance(), x + g.pen()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn removed_custody<S: LayoutScalar>(
    fixture: &Fixture,
    batch: &CompletedLayoutBatchOf<u32, S>,
    line: &text::FinalizedLine,
    value: &str,
    space: text::Range,
) {
    let rows = line
        .edge_units()
        .filter(|row| row.source_pieces().any(|actual| actual == space))
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        panic!("one real original SPACE edge row")
    };
    assert_eq!(row.disposition(), text::LineEdgeDisposition::Removed);
    assert_eq!(row.advance(), 0.0);
    let origin = fixture
        .binding
        .origins
        .iter()
        .find(|row| {
            let captured = row.origin.view().aggregate_range();
            captured.start <= space.start && space.end <= captured.end
        })
        .unwrap()
        .origin
        .clone();
    assert!(
        line.fragments().any(|fragment| {
            let view = fragment.view();
            view.origin().as_ref() == Some(&origin) && view.aggregate_range() == Some(space)
        }),
        "removal preserves the actual original fragment custody"
    );
    let [placed] = batch.placed_text() else {
        panic!("one actual placed Text result")
    };
    assert_eq!(
        placed.layout().source().text().code_units(),
        value.encode_utf16().collect::<Vec<_>>()
    );
    let local = text::Range::new(
        origin.view().local_range().start + space.start - origin.view().aggregate_range().start,
        origin.view().local_range().start + space.end - origin.view().aggregate_range().start,
    );
    let query = placed
        .layout()
        .source_range_fragments(&origin, local)
        .unwrap();
    assert_eq!(
        query.units().len(),
        0,
        "actual discarded source grants no Range geometry"
    );
}

#[test]
fn collapsible_terminal_space_before_real_end_border_padding_is_trimmed_in_wide_and_exact_frames() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::new("A ", (0, 2), text::WhiteSpace::Collapse, false, false);
        let (_, end) = actual_pair(&fixture);
        assert_eq!(end.view().source_position().get(), 2);
        let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::from_f64(3.0)).unwrap();
        for width in [100.0, 15.0] {
            let batch = fixture.compute(
                InlineBoundaryEdgeOf::ZERO,
                edge,
                false,
                Some(width),
                false,
                AvailableOf::definite(S::from_f64(width)),
            );
            let lines = finalized_lines(&batch);
            let [line] = lines.as_slice() else {
                panic!("A10 + retained end strip5 fits one genuine finite line")
            };
            let selected = line.selected_line();
            let request = selected.request();
            assert_eq!(request.context.end, text::LineEnd::Paragraph);
            assert_eq!(request.context.available_inline, Some(width));
            assert!(
                matches!(&request.termination, text::PreparedTermination::Ordinary(offer)
                if offer.view().kind() == text::PreparedBreakKind::ParagraphEnd)
            );
            assert_eq!(raw(line), [(text::Range::new(0, 1), 35, 10.0, 0.0)]);
            // Text3 boundary transparency removes the SPACE10, not the real
            // border2/padding3 strip. This is terminal classification in a
            // fully admitted paragraph, not an unbreakable-overflow oracle.
            assert_eq!(
                (
                    line.feedback().advance,
                    line.measurements().fit().advance(),
                    line.measurements().alignment().advance()
                ),
                (15.0, 15.0, 15.0)
            );
            let slot = line
                .control_slots()
                .find(|slot| slot.control() == end)
                .unwrap();
            assert_eq!(slot.inline_bounds(), (10.0, 15.0));
            let strip = output(&batch, 21);
            assert_eq!(strip.location.x, S::from_f64(10.0));
            assert_eq!(strip.size, Size::new(S::from_f64(5.0), S::ZERO));
            assert_eq!(
                (strip.border.right, strip.padding.right),
                (S::from_f64(2.0), S::from_f64(3.0))
            );
            assert_eq!(strip.content_size, Size::ZERO);
            removed_custody(&fixture, &batch, line, "A ", text::Range::new(1, 2));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn collapsible_leading_space_after_real_start_border_padding_is_trimmed() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::new(" A", (0, 2), text::WhiteSpace::Collapse, false, false);
        let (start, _) = actual_pair(&fixture);
        assert_eq!(start.view().source_position().get(), 0);
        let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::from_f64(3.0)).unwrap();
        for width in [100.0, 15.0] {
            let batch = fixture.compute(
                edge,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(width),
                false,
                AvailableOf::definite(S::from_f64(width)),
            );
            let lines = finalized_lines(&batch);
            let [line] = lines.as_slice() else {
                panic!("real start strip5 + A10 fits one finite line")
            };
            assert_eq!(
                line.selected_line().request().context.end,
                text::LineEnd::Paragraph
            );
            assert_eq!(
                line.selected_line().request().context.available_inline,
                Some(width)
            );
            assert_eq!(raw(line), [(text::Range::new(1, 2), 35, 10.0, 5.0)]);
            assert_eq!(
                (
                    line.feedback().advance,
                    line.measurements().fit().advance(),
                    line.measurements().alignment().advance()
                ),
                (15.0, 15.0, 15.0)
            );
            assert_eq!(
                line.control_slots()
                    .find(|slot| slot.control() == start)
                    .unwrap()
                    .inline_bounds(),
                (0.0, 5.0)
            );
            let strip = output(&batch, 20);
            assert_eq!(strip.location.x, S::ZERO);
            assert_eq!(strip.size, Size::new(S::from_f64(5.0), S::ZERO));
            assert_eq!(
                (strip.border.left, strip.padding.left),
                (S::from_f64(2.0), S::from_f64(3.0))
            );
            removed_custody(&fixture, &batch, line, " A", text::Range::new(0, 1));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn a_following_real_sibling_keeps_internal_collapsible_space_and_real_end_strip() {
    fn lane<S: LayoutScalar>() {
        // Genuine <span>A SPACE</span>B. Source capture really closes Inline1
        // before the outside B origin, within its genuine parent Block0.
        let fixture = Fixture::new("A B", (0, 2), text::WhiteSpace::Collapse, false, false);
        let (_, end) = actual_pair(&fixture);
        assert_eq!(fixture.binding.origins.len(), 2);
        let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::from_f64(3.0)).unwrap();
        let batch = fixture.compute(
            InlineBoundaryEdgeOf::ZERO,
            edge,
            false,
            Some(100.0),
            false,
            AvailableOf::definite(S::from_f64(100.0)),
        );
        let lines = finalized_lines(&batch);
        let [line] = lines.as_slice() else {
            panic!("one real internal-space line")
        };
        assert_eq!(
            raw(line),
            [
                (text::Range::new(0, 1), 35, 10.0, 0.0),
                (text::Range::new(1, 2), 3, 10.0, 10.0),
                (text::Range::new(2, 3), 36, 10.0, 25.0),
            ]
        );
        assert_eq!(line.feedback().advance, 35.0); // A10 + SPACE10 + strip5 + B10.
        assert_eq!(line.measurements().fit().advance(), 35.0);
        assert_eq!(
            line.control_slots()
                .find(|slot| slot.control() == end)
                .unwrap()
                .inline_bounds(),
            (20.0, 25.0)
        );
        assert_eq!(output(&batch, 21).location.x, S::from_f64(20.0));
        let row = line
            .edge_units()
            .find(|row| {
                row.source_pieces()
                    .any(|source| source == text::Range::new(1, 2))
            })
            .unwrap();
        assert_ne!(row.disposition(), text::LineEdgeDisposition::Removed);
        assert_eq!(row.advance(), 10.0);
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn preserved_terminal_space_is_not_trimmed_by_the_collapsible_space_rule() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::new("A ", (0, 2), text::WhiteSpace::Preserve, false, false);
        let (_, end) = actual_pair(&fixture);
        let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::from_f64(3.0)).unwrap();
        let batch = fixture.compute(
            InlineBoundaryEdgeOf::ZERO,
            edge,
            false,
            Some(100.0),
            false,
            AvailableOf::definite(S::from_f64(100.0)),
        );
        let lines = finalized_lines(&batch);
        let [line] = lines.as_slice() else {
            panic!("one actual preserved-space line")
        };
        assert_eq!(
            raw(line),
            [
                (text::Range::new(0, 1), 35, 10.0, 0.0),
                (text::Range::new(1, 2), 3, 10.0, 10.0),
            ]
        );
        assert_eq!(line.feedback().advance, 25.0);
        assert_eq!(
            line.control_slots()
                .find(|slot| slot.control() == end)
                .unwrap()
                .inline_bounds(),
            (20.0, 25.0)
        );
        // No asserted fit/alignment hanging classification here: preserving a
        // real SPACE and its raw flow is separate from terminal hanging rules.
        let row = line
            .edge_units()
            .find(|row| {
                row.source_pieces()
                    .any(|source| source == text::Range::new(1, 2))
            })
            .unwrap();
        assert_ne!(row.disposition(), text::LineEdgeDisposition::Removed);
        assert_eq!(row.advance(), 10.0);
    }
    lane::<f32>();
    lane::<f64>();
}
