//! Supplementary functional qualification of exact Cell endpoint custody.
//! Expected pieces come from authored scalar occurrences, Unicode ß -> SS,
//! fixed inner whitespace edges and the resulting selected B/A-or-SS/B stream.
//! No slot result supplies its own expected ancestry or presentation positions.

use super::*;

fn original(fixture: &Fixture, prepared: &PreparedText, ordinal: usize) -> PreparedOrigin {
    let capture = fixture.processed.captured_origin(ordinal).unwrap();
    let view = prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&capture))
        .unwrap();
    assert_eq!(view.source(), fixture.processed.source());
    assert_eq!(view.local_range(), Range::new(ordinal, ordinal + 1));
    assert_eq!(view.aggregate_range(), Range::new(ordinal, ordinal + 1));
    view.handle()
}

// Source occurrence ordinal, followed by independently derived selected range.
type Piece = (usize, (usize, usize));

fn expected(
    fixture: &Fixture,
    prepared: &PreparedText,
    rows: &[Piece],
) -> Vec<(SpacingSlotOwner, (usize, usize))> {
    rows.iter()
        .map(|&(ordinal, presentation)| {
            (
                SpacingSlotOwner::Source {
                    origin: original(fixture, prepared, ordinal),
                    local: Range::new(ordinal, ordinal + 1),
                    aggregate: Range::new(ordinal, ordinal + 1),
                },
                presentation,
            )
        })
        .collect()
}

fn pieces(
    values: impl Iterator<Item = SpacingSlotPiece>,
) -> Vec<(SpacingSlotOwner, (usize, usize))> {
    values
        .map(|piece| {
            let range = piece.presentation_range();
            (piece.owner(), (range.start(), range.end()))
        })
        .collect()
}

fn endpoint(
    fixture: &Fixture,
    prepared: &PreparedText,
    unit: &SpacingUnit,
    participant: &PreparedParticipant,
    rows: &[Piece],
) {
    assert_eq!(unit.participants().collect::<Vec<_>>(), [participant]);
    assert_eq!(unit.letter_spacing_before(), 0.0);
    assert_eq!(unit.letter_spacing_after(), 0.0);
    assert_eq!(pieces(unit.pieces()), expected(fixture, prepared, rows));
}

fn native_none(
    fixture: &Fixture,
    prepared: &PreparedText,
    glyphs: &[(Range, u32, f32)],
    scale: f64,
) -> FinalizedLine {
    let line = finalize(prepared);
    assert_eq!(
        line.selected_line().request().context.spacing,
        SpacingPolicy::None
    );
    assert_eq!(line.spacing_slots().len(), 0);
    assert_eq!(line.feedback().advance, 30.0);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one authentic finalized Cell");
    };
    assert_eq!(
        cell.source_cell().scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().origin.x, 10.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let mut inner = Vec::new();
    let mut outside = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&fixture.body.face()));
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            inner.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        } else {
            assert!(run.face().is_same_face(&fixture.parent.face()));
            outside.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        }
    }
    assert_eq!(inner, glyphs);
    assert_eq!(
        outside,
        [
            (Range::new(0, 1), 36, 10.0),
            (
                Range::new(fixture.inner.end, fixture.inner.end + 1),
                36,
                10.0
            ),
        ]
    );
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), fixture.processed.source());
    let mut body_runs = 0;
    for run in layout.glyph_runs() {
        if run.font().face().is_same_face(&fixture.body.face()) {
            body_runs += 1;
            assert_eq!(run.glyph_frame().pen_direction(), Point::new(1.0, 0.0));
            close(run.glyph_frame().outline_x().x, scale);
            assert_eq!(run.glyph_frame().outline_x().y, 0.0);
            assert_eq!(run.glyph_frame().outline_y(), Point::new(0.0, 1.0));
        }
    }
    assert!(body_runs > 0);
    line
}

fn exterior(
    fixture: &Fixture,
    prepared: &PreparedText,
    none: &FinalizedLine,
    cell_rows: &[Piece],
    trailing: Piece,
) {
    let participants = prepared.participants().collect::<Vec<_>>();
    let [
        PreparedParticipant::Text(_),
        PreparedParticipant::Cell(cell),
        PreparedParticipant::Text(_),
    ] = participants.as_slice()
    else {
        panic!("B, one Cell, B are the actual outer participants");
    };
    assert_eq!(
        cell.view().source_cell().scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    assert!(cell.view().has_provider_input());
    let mut request = none.selected_line().request().clone();
    request.context.spacing = SpacingPolicy::InterCharacter;
    let selected = prepared
        .select(request)
        .expect("actual mixed-custody Cell exterior is allocatable");
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2); // Three outer objects, irrespective of inner parts.
    let SpacingSlotTarget::Boundary {
        visual_before: first,
        visual_after: entering,
    } = slots[0].target()
    else {
        panic!("entering exterior character boundary");
    };
    let SpacingSlotTarget::Boundary {
        visual_before: leaving,
        visual_after: last,
    } = slots[1].target()
    else {
        panic!("leaving exterior character boundary");
    };
    let before = [(0, (0, 1))];
    let after = [trailing];
    endpoint(fixture, prepared, &first, &participants[0], &before);
    endpoint(fixture, prepared, &entering, &participants[1], cell_rows);
    endpoint(fixture, prepared, &leaving, &participants[1], cell_rows);
    endpoint(fixture, prepared, &last, &participants[2], &after);
    // The very same issued Cell is each whole endpoint; no interior SPACE/A
    // or S/S boundary and no deduplication of repeated original ancestry.
    assert_eq!(
        entering.participants().collect::<Vec<_>>(),
        leaving.participants().collect::<Vec<_>>()
    );
    let entering_rows = before.iter().chain(cell_rows).copied().collect::<Vec<_>>();
    let leaving_rows = cell_rows.iter().chain(&after).copied().collect::<Vec<_>>();
    assert_eq!(
        pieces(slots[0].pieces()),
        expected(fixture, prepared, &entering_rows)
    );
    assert_eq!(
        pieces(slots[1].pieces()),
        expected(fixture, prepared, &leaving_rows)
    );
    assert_eq!(
        slots[0].participants().cloned().collect::<Vec<_>>(),
        participants[..2]
    );
    assert_eq!(
        slots[1].participants().cloned().collect::<Vec<_>>(),
        participants[1..]
    );
    let amounts = slots
        .into_iter()
        .zip([2.0, 3.0])
        .map(|(slot, amount)| SpacingAllocation { slot, amount })
        .collect();
    let allocated = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::Complete(amounts),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(allocated.feedback().advance, 35.0);
    let cells = allocated.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual allocated Cell");
    };
    assert_eq!(cell.square().origin.x, 12.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    // Feedback keeps the same immutable selected endpoint correspondence.
    assert_eq!(
        pieces(
            allocated
                .selected_line()
                .spacing_slots()
                .next()
                .unwrap()
                .pieces()
        ),
        expected(fixture, prepared, &entering_rows)
    );
    assert_eq!(none.feedback().advance, 30.0);
}

#[test]
fn visible_and_removed_or_zero_cell_parts_keep_real_cuts_on_both_exterior_boundaries() {
    for (inner, cell_rows, trailing, visible) in [
        (
            " A ",
            vec![(1, (1, 1)), (2, (1, 2)), (3, (2, 2))],
            (4, (2, 3)),
            2,
        ),
        (
            "  A  ",
            vec![
                (1, (1, 1)),
                (2, (1, 1)),
                (3, (1, 2)),
                (4, (2, 2)),
                (5, (2, 2)),
            ],
            (6, (2, 3)),
            3,
        ),
    ] {
        let (mut system, fixture) =
            fixture_with_processing(inner, 0.0, true, true, 10.0, WhiteSpace::Collapse, false);
        assert_eq!(fixture.processed.processed_text(), &Text::from("B A B"));
        let source = fixture.analyzed.combined_cells().next().unwrap();
        assert_eq!(source.used_code_points().collect::<Vec<_>>(), [0x41]);
        let originals = source.source_pieces().collect::<Vec<_>>();
        assert_eq!(originals.iter().filter(|piece| piece.is_used()).count(), 1);
        assert_eq!(
            originals
                .iter()
                .filter(|piece| piece.has_presentation_emission())
                .count(),
            1
        );
        if inner == "  A  " {
            // Collapse retains the first SPACE of each run; the second is a
            // separate step4-zero occurrence. Both later inner edges suppress
            // paint without erasing either actual original.
            assert_eq!(
                originals
                    .iter()
                    .map(|piece| piece.emission_subposition())
                    .collect::<Vec<_>>(),
                [Some(0), None, Some(0), Some(0), None]
            );
        }
        let prepared = system
            .prepare(
                &fixture.analyzed,
                PreparationOptions::default().with_orientation(LineOrientation::Vertical),
            )
            .unwrap();
        let none = native_none(
            &fixture,
            &prepared,
            &[(Range::new(visible, visible + 1), 35, 10.0)],
            1.0,
        );
        exterior(&fixture, &prepared, &none, &cell_rows, trailing);
    }
}

#[test]
fn uppercase_expansion_keeps_repeated_original_ancestry_with_distinct_selected_emissions() {
    let (mut system, fixture) =
        fixture_with_processing("ß", 0.0, true, true, 10.0, WhiteSpace::Preserve, true);
    assert_eq!(fixture.processed.source().text(), &Text::from("BßB"));
    assert_eq!(fixture.inner, Range::new(1, 2)); // Original UTF-16, not ß's UTF-8 bytes.
    let source = fixture.analyzed.combined_cells().next().unwrap();
    assert_eq!(source.selection_unit_count(), 2);
    assert_eq!(source.used_unit_count(), 2);
    assert_eq!(
        source.presentation_code_points().collect::<Vec<_>>(),
        [0x53, 0x53]
    );
    let originals = source.source_pieces().collect::<Vec<_>>();
    assert_eq!(originals.len(), 2);
    assert_eq!(
        originals
            .iter()
            .map(|piece| piece.emission_subposition())
            .collect::<Vec<_>>(),
        [Some(0), Some(1)]
    );
    for piece in originals {
        assert_eq!(piece.local(), Range::new(1, 2));
        assert_eq!(
            piece.origin_position(),
            fixture.processed.captured_origin(1).unwrap()
        );
    }
    use skrifa::MetadataProvider;
    let parsed = skrifa::FontRef::new(fixture.body.bytes()).unwrap();
    let glyph = parsed.charmap().map('S').unwrap().to_u32();
    let prepared = system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    // Exact face S has normal10 per scalar and Complete NoAction for each width
    // plan (qualified by the fixture), so retained SS normal20 compresses to E10.
    let none = native_none(
        &fixture,
        &prepared,
        &[
            (Range::new(1, 2), glyph, 10.0),
            (Range::new(1, 2), glyph, 10.0),
        ],
        0.5,
    );
    exterior(
        &fixture,
        &prepared,
        &none,
        &[(1, (1, 2)), (1, (2, 3))],
        (2, (3, 4)),
    );
}
