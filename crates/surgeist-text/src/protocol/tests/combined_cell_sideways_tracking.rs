//! Signed outer tracking/allocation uses the actual sideways line map while
//! the retained AA hwid packet remains physically horizontal and fixed E10.

use super::super::sideways::{
    metric_premises, ordinary_control, place, receipt, reference, select, square_x, square_y,
};
use super::*;

fn prepare(orientation: LineOrientation, tracking: f64, spaces: bool) -> (Fixture, PreparedText) {
    let (mut system, fixture) = fixture_with_profile(
        "AA",
        tracking,
        true,
        true,
        10.0,
        FixtureProfile {
            white_space: WhiteSpace::Preserve,
            uppercase: false,
            typography: AnalysisTypography::HorizontalComposition,
            reference: reference(),
            exterior_spaces: spaces,
        },
    );
    for scope in [0, 1] {
        assert_eq!(
            fixture
                .processed
                .captured_scope(scope)
                .unwrap()
                .writing_axis(),
            Some(SourceWritingAxis::Vertical)
        );
    }
    metric_premises(&system, fixture.parent.bytes());
    metric_premises(&system, fixture.body.bytes());
    assert_eq!(
        fixture.analyzed.typography(),
        AnalysisTypography::HorizontalComposition
    );
    ordinary_control(&mut system, orientation, fixture.parent.bytes());
    if spaces {
        public_packet(fixture.parent.bytes(), " ", None, 3, 1000);
    }
    // The same genuine normal10/hwid5/twid4/qwid3 native operands and complete
    // successful-action receipts were qualified by the shared factory above.
    let prepared = system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .expect("a source-authenticated sideways Cell must admit its ordinary rotated neighbors");
    assert_eq!(
        prepared.analyzed_source().identity(),
        fixture.analyzed.identity()
    );
    receipt(&prepared, &fixture.processed);
    (fixture, prepared)
}

fn owner(fixture: &Fixture, prepared: &PreparedText, ordinal: usize) -> SpacingSlotOwner {
    let position = fixture.processed.captured_origin(ordinal).unwrap();
    let origin = prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&position))
        .unwrap()
        .handle();
    SpacingSlotOwner::Source {
        origin,
        local: Range::new(ordinal, ordinal + 1),
        aggregate: Range::new(ordinal, ordinal + 1),
    }
}

fn complete(selected: &SelectedLine, amounts: [f64; 2]) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip(amounts)
            .map(|(slot, amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

fn allocated(selected: &SelectedLine, amounts: [f64; 2]) -> FinalizedLine {
    selected
        .finalize(LineAllocations {
            spacing: complete(selected, amounts),
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

// Expected ordinary rows are (source ordinal, logical painted pen, raw advance).
fn flow(
    fixture: &Fixture,
    prepared: &PreparedText,
    line: &FinalizedLine,
    extent: (f64, f64, f64),
    ordinary: &[(usize, f64, f32)],
) {
    let (total, start, end) = extent;
    assert_eq!(line.feedback().advance, total);
    assert_eq!(line.measurements().fit().advance(), total);
    let orientation = prepared.orientation();
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine retained AA Cell");
    };
    assert_eq!(cell.square().origin.x, start);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(
        cell.source_cell().scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    let mut outer = Vec::new();
    let mut inner = Vec::new();
    let basis = match orientation {
        LineOrientation::SidewaysClockwise => GlyphBasis::Clockwise,
        LineOrientation::SidewaysCounterclockwise => GlyphBasis::Counterclockwise,
        _ => panic!("sideways profile"),
    };
    for run in line.run_views() {
        assert_eq!(run.font_size(), 10.0);
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&fixture.body.face()));
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            inner.extend(run.glyphs().map(|g| {
                (
                    g.range(),
                    g.id(),
                    run.origin().x + g.pen(),
                    g.advance(),
                    g.x(),
                    g.y(),
                )
            }));
        } else {
            assert!(run.face().is_same_face(&fixture.parent.face()));
            assert_eq!(run.glyph_basis(), basis);
            outer.extend(run.glyphs().map(|g| {
                (
                    g.range(),
                    g.id(),
                    run.origin().x + g.pen(),
                    g.advance(),
                    g.x(),
                    g.y(),
                )
            }));
        }
    }
    assert_eq!(
        inner,
        [
            (
                Range::new(fixture.inner.start, fixture.inner.start + 1),
                36,
                0.0,
                5.0,
                0.0,
                0.0
            ),
            (
                Range::new(fixture.inner.start + 1, fixture.inner.end),
                36,
                5.0,
                5.0,
                0.0,
                0.0
            ),
        ]
    );
    assert_eq!(
        outer,
        ordinary
            .iter()
            .map(|&(ordinal, pen, advance)| (
                Range::new(ordinal, ordinal + 1),
                if fixture.processed.source().text().code_units()[ordinal] == 0x20 {
                    3
                } else {
                    36
                },
                pen,
                advance,
                0.0,
                0.0,
            ))
            .collect::<Vec<_>>()
    );
    let layout = place(prepared, line.clone());
    assert_eq!(layout.source(), fixture.processed.source());
    let sign = if orientation == LineOrientation::SidewaysClockwise {
        1.0
    } else {
        -1.0
    };
    let outside_x = Point::new(0.0, sign);
    let outside_y = Point::new(-sign, 0.0);
    let mut physical_outer = Vec::new();
    let mut physical_inner = Vec::new();
    for run in layout.glyph_runs() {
        let frame = run.glyph_frame();
        if run.font().face().is_same_face(&fixture.body.face()) {
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
            assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
            assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
            assert_eq!(
                frame.origin(),
                Point::new(square_x(orientation), square_y(orientation, start) + 8.0)
            );
            physical_inner.extend(
                run.glyphs()
                    .iter()
                    .map(|g| Point::new(frame.origin().x + g.pen(), frame.origin().y)),
            );
        } else {
            assert!(run.font().face().is_same_face(&fixture.parent.face()));
            assert_eq!(run.glyph_basis(), basis);
            assert_eq!(frame.outline_x(), outside_x);
            assert_eq!(frame.outline_y(), outside_y);
            let pen = frame.pen_direction();
            for glyph in run.glyphs() {
                let x = f64::from(glyph.x()) + glyph.pen() * pen.x;
                let y = f64::from(glyph.y()) + glyph.pen() * pen.y;
                physical_outer.push((
                    glyph.range(),
                    Point::new(
                        frame.origin().x + outside_x.x * x + outside_y.x * y,
                        frame.origin().y + outside_x.y * x + outside_y.y * y,
                    ),
                ));
            }
        }
    }
    assert_eq!(
        physical_inner,
        [
            Point::new(square_x(orientation), square_y(orientation, start) + 8.0),
            Point::new(
                square_x(orientation) + 5.0,
                square_y(orientation, start) + 8.0
            ),
        ]
    );
    assert_eq!(
        physical_outer,
        ordinary
            .iter()
            .map(|&(ordinal, pen, _)| (
                Range::new(ordinal, ordinal + 1),
                Point::new(100.0, 20.0 + sign * pen)
            ))
            .collect::<Vec<_>>()
    );
    // The opaque Cell's ending caret follows the executed predecessor-owned
    // flow end, including signed tracking/allocation; the painted square is E10.
    for (cursor, cut) in [
        (Cursor::new(fixture.inner.start, Affinity::After), start),
        (Cursor::new(fixture.inner.end, Affinity::Before), end),
    ] {
        let y = if sign > 0.0 { 20.0 + cut } else { 19.0 - cut };
        assert_eq!(
            layout.cursor(cursor).unwrap().rect(),
            Rect::new(square_x(orientation), y, 10.0, 1.0)
        );
    }
}

fn boundaries(fixture: &Fixture, prepared: &PreparedText, selected: &SelectedLine, tracking: f64) {
    let participants = prepared.participants().collect::<Vec<_>>();
    let [
        PreparedParticipant::Text(_),
        PreparedParticipant::Cell(cell),
        PreparedParticipant::Text(_),
    ] = participants.as_slice()
    else {
        panic!("B/whole Cell/B participant inventory");
    };
    assert_eq!(
        cell.view().source_cell().scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    let roots: [&[usize]; 3] = [&[0], &[1, 2], &[3]];
    let tracking = [0.0, tracking, 0.0];
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    for (index, slot) in slots.iter().enumerate() {
        let SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            panic!("exterior character boundary");
        };
        for (unit, row) in [(&visual_before, index), (&visual_after, index + 1)] {
            assert_eq!(
                (unit.letter_spacing_before(), unit.letter_spacing_after()),
                (tracking[row], tracking[row])
            );
            assert_eq!(
                unit.participants().collect::<Vec<_>>(),
                [&participants[row]]
            );
            assert_eq!(
                unit.pieces().map(|p| p.owner()).collect::<Vec<_>>(),
                roots[row]
                    .iter()
                    .map(|&i| owner(fixture, prepared, i))
                    .collect::<Vec<_>>()
            );
        }
    }
}

fn character_cases(orientation: LineOrientation) {
    for tracking in [2.0, -2.0] {
        let (fixture, prepared) = prepare(orientation, tracking, false);
        let half = tracking / 2.0;
        let none = select(&prepared, SpacingPolicy::None)
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::None,
            })
            .unwrap();
        flow(
            &fixture,
            &prepared,
            &none,
            (30.0 + tracking, 10.0 + half, 20.0 + tracking),
            &[(0, 0.0, 10.0), (3, 20.0 + tracking, 10.0)],
        );
        let selected = select(&prepared, SpacingPolicy::InterCharacter);
        boundaries(&fixture, &prepared, &selected, tracking);
        let expanded = allocated(&selected, [1.0, 3.0]);
        flow(
            &fixture,
            &prepared,
            &expanded,
            (34.0 + tracking, 11.0 + half, 24.0 + tracking),
            &[(0, 0.0, 10.0), (3, 24.0 + tracking, 10.0)],
        );
        let replay = expanded
            .with_spacing(complete(&selected, [2.0, 0.5]))
            .unwrap();
        assert_eq!(replay.selected_line(), selected);
        flow(
            &fixture,
            &prepared,
            &replay,
            (32.5 + tracking, 12.0 + half, 22.5 + tracking),
            &[(0, 0.0, 10.0), (3, 22.5 + tracking, 10.0)],
        );
        let restored = replay.with_spacing(SpacingAllocations::None).unwrap();
        flow(
            &fixture,
            &prepared,
            &restored,
            (30.0 + tracking, 10.0 + half, 20.0 + tracking),
            &[(0, 0.0, 10.0), (3, 20.0 + tracking, 10.0)],
        );
    }
}

fn word_cases(orientation: LineOrientation) {
    let (fixture, prepared) = prepare(orientation, 2.0, true);
    assert_eq!(fixture.processed.source().text(), &Text::from("B AA B"));
    assert_eq!(fixture.inner, Range::new(2, 4));
    let none = select(&prepared, SpacingPolicy::None)
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    flow(
        &fixture,
        &prepared,
        &none,
        (52.0, 21.0, 32.0),
        &[
            (0, 0.0, 10.0),
            (1, 10.0, 10.0),
            (4, 32.0, 10.0),
            (5, 42.0, 10.0),
        ],
    );
    for policy in [SpacingPolicy::Spaces, SpacingPolicy::InterWord] {
        let selected = select(&prepared, policy);
        let slots = selected.spacing_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 2);
        for (slot, ordinal) in slots.iter().zip([1, 4]) {
            let SpacingSlotTarget::Separator(unit) = slot.target() else {
                panic!("real exterior SPACE");
            };
            assert_eq!(
                slot.pieces().map(|p| p.owner()).collect::<Vec<_>>(),
                [owner(&fixture, &prepared, ordinal)]
            );
            assert_eq!(
                unit.pieces().map(|p| p.owner()).collect::<Vec<_>>(),
                [owner(&fixture, &prepared, ordinal)]
            );
            let participants = unit.participants().collect::<Vec<_>>();
            assert_eq!(participants.len(), 1);
            let PreparedParticipant::Text(text) = participants[0] else {
                panic!("actual ordinary SPACE participant");
            };
            assert_eq!(text.view().source_range(), Range::new(ordinal, ordinal + 1));
        }
        let factor = if policy == SpacingPolicy::InterWord {
            0.5
        } else {
            0.0
        };
        let expanded = allocated(&selected, [1.0, 3.0]);
        flow(
            &fixture,
            &prepared,
            &expanded,
            (56.0, 22.0, 33.0),
            &[
                (0, 0.0, 10.0),
                (1, 10.0 + factor, 10.0),
                (4, 33.0 + 3.0 * factor, 10.0),
                (5, 46.0, 10.0),
            ],
        );
        let replay = expanded
            .with_spacing(complete(&selected, [2.0, 0.5]))
            .unwrap();
        assert_eq!(replay.selected_line(), selected);
        flow(
            &fixture,
            &prepared,
            &replay,
            (54.5, 23.0, 34.0),
            &[
                (0, 0.0, 10.0),
                (1, 10.0 + 2.0 * factor, 10.0),
                (4, 34.0 + 0.5 * factor, 10.0),
                (5, 44.5, 10.0),
            ],
        );
        let restored = replay.with_spacing(SpacingAllocations::None).unwrap();
        flow(
            &fixture,
            &prepared,
            &restored,
            (52.0, 21.0, 32.0),
            &[
                (0, 0.0, 10.0),
                (1, 10.0, 10.0),
                (4, 32.0, 10.0),
                (5, 42.0, 10.0),
            ],
        );
    }
}

#[test]
fn clockwise_tracking_and_exterior_allocations_preserve_horizontal_inner_cells() {
    character_cases(LineOrientation::SidewaysClockwise);
    word_cases(LineOrientation::SidewaysClockwise);
}
#[test]
fn counterclockwise_tracking_and_exterior_allocations_preserve_horizontal_inner_cells() {
    character_cases(LineOrientation::SidewaysCounterclockwise);
    word_cases(LineOrientation::SidewaysCounterclockwise);
}
