//! Genuine exterior separator allocations remain separate from the retained
//! horizontal Cell's already executed inner word spacing.

use super::*;
use icu_properties::{CodePointMapData, props::VerticalOrientation};
use surgeist_shaper::{
    Direction, Feature, FontRef as ShaperFontRef, ShaperData, ShapingStatus, Tag, UnicodeBuffer,
    WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
};

#[derive(Clone, Copy)]
struct SpaceMetrics {
    advance: f32,
    basis: GlyphBasis,
}

fn exterior_space_metrics(bytes: &[u8]) -> SpaceMetrics {
    assert_eq!(sfnt::u16_at(sfnt::table(bytes, *b"hmtx"), 3 * 4), 1000);
    assert!(sfnt::u16_at(sfnt::table(bytes, *b"vhea"), 34) > 3);
    assert_eq!(sfnt::u16_at(sfnt::table(bytes, *b"vmtx"), 3 * 4), 1200);
    let font = ShaperFontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    // Both actual provider axes are qualified. Blank ink and horizontal hmtx
    // alone cannot stand in for a native upright SPACE's vertical advance.
    for (direction, advance) in [
        (Direction::LeftToRight, (1000, 0)),
        (Direction::TopToBottom, (0, -1200)),
    ] {
        let mut input = UnicodeBuffer::new();
        input.add(' ', 0);
        input.set_direction(direction);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        let output = shaper.shape(input, &[]);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(3, 0)]
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|p| (p.x_advance, p.y_advance))
                .collect::<Vec<_>>(),
            [advance]
        );
    }
    // This is the pinned source-property authority for Mixed, not an observed
    // Text run or a target-output-derived width. Text must use the matching
    // real provider axis, then scale the independently checked 1000/1200 by .01.
    if CodePointMapData::<VerticalOrientation>::new().get32(0x20) == VerticalOrientation::Rotated {
        SpaceMetrics {
            advance: 10.0,
            basis: GlyphBasis::Clockwise,
        }
    } else {
        SpaceMetrics {
            advance: 12.0,
            basis: GlyphBasis::Unrotated,
        }
    }
}

fn partial_cell_width_premises(bytes: &[u8]) {
    let font = ShaperFontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for (request, glyph, advance) in [
        (None, 35, 1000),
        (Some((*b"hwid", WidthFeature::HalfWidth)), 36, 500),
        (Some((*b"twid", WidthFeature::ThirdWidth)), 58, 400),
        (Some((*b"qwid", WidthFeature::QuarterWidth)), 16, 300),
    ] {
        let mut input = UnicodeBuffer::new();
        for (ordinal, ch) in "A A".chars().enumerate() {
            input.add(ch, u32::try_from(ordinal).unwrap());
        }
        input.set_direction(Direction::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        let features = if let Some((tag, feature)) = request {
            input.observe_width_feature(feature);
            vec![Feature::new(Tag::new(&tag), 1, ..)]
        } else {
            vec![]
        };
        let output = shaper.shape(input, &features);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(glyph, 0), (3, 1), (glyph, 2)]
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
                .collect::<Vec<_>>(),
            [(advance, 0, 0, 0), (1000, 0, 0, 0), (advance, 0, 0, 0)]
        );
        if let Some((_, feature)) = request {
            let receipt = output.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            assert_eq!(receipt.input_len(), 3);
            for (ordinal, expected) in [
                WidthInputOutcome::Covered,
                WidthInputOutcome::NoAction,
                WidthInputOutcome::Covered,
            ]
            .into_iter()
            .enumerate()
            {
                assert_eq!(receipt.input(ordinal).unwrap().outcome(), expected);
            }
        } else {
            assert!(output.width_feature_receipt().unwrap().is_none());
        }
    }
}

fn source_owner(mixed: &Mixed, ordinal: usize) -> SpacingSlotOwner {
    let capture = mixed.processed.captured_origin(ordinal).unwrap();
    let origin = mixed
        .prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&capture))
        .unwrap()
        .handle();
    assert_eq!(origin.view().source(), mixed.processed.source());
    SpacingSlotOwner::Source {
        origin,
        local: Range::new(ordinal, ordinal + 1),
        aggregate: Range::new(ordinal, ordinal + 1),
    }
}

fn targets(mixed: &Mixed, selected: &SelectedLine) {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(
        slots.len(),
        2,
        "only the two exterior SPACEs are allocation targets"
    );
    for (slot, ordinal) in slots.iter().zip([1, 5]) {
        let SpacingSlotTarget::Separator(unit) = slot.target() else {
            panic!("an exterior separator target");
        };
        let participant = mixed.prepared.participants().find(|participant| matches!(participant,
            PreparedParticipant::Text(handle) if handle.view().source_range() == Range::new(ordinal, ordinal + 1))).unwrap();
        assert_eq!(unit.participants().collect::<Vec<_>>(), [&participant]);
        assert_eq!(
            slot.participants().cloned().collect::<Vec<_>>(),
            [participant]
        );
        assert_eq!(
            (unit.letter_spacing_before(), unit.letter_spacing_after()),
            (4.0, 4.0)
        );
        assert_eq!(
            unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            [source_owner(mixed, ordinal)]
        );
        assert_eq!(
            slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            [source_owner(mixed, ordinal)]
        );
    }
}

fn complete(selected: &SelectedLine, amounts: [f64; 2]) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), amounts.len());
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip(amounts)
            .map(|(slot, amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

fn raw(mixed: &Mixed, line: &FinalizedLine) -> Vec<(bool, Range, u32, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            let cell = run.cell().is_some();
            if cell {
                assert!(run.face().is_same_face(&mixed.body.face()));
            } else {
                assert!(run.face().is_same_face(&mixed.parent.face()));
            }
            run.glyphs()
                .map(|glyph| {
                    (
                        cell,
                        glyph.range(),
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn flow(mixed: &Mixed, line: &FinalizedLine, space: SpaceMetrics, amounts: [f64; 2]) {
    let [first, second] = amounts;
    let intrinsic = f64::from(space.advance);
    // B/SP/Cell/SP/B have intrinsic10/S/10/S/10. Actual outer LS4/4/2/4/4
    // contributes boundary means4/3/3/4. Allocations belong only to the SPACEs.
    let cell_start = 17.0 + intrinsic + first;
    let second_space = 30.0 + intrinsic + first;
    let last = 34.0 + 2.0 * intrinsic + first + second;
    assert_eq!(
        line.feedback().advance,
        44.0 + 2.0 * intrinsic + first + second
    );
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one real A SPACE A Cell");
    };
    assert_eq!(cell.square().origin.x, cell_start);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(
        cell.source_cell().scope(),
        mixed.processed.captured_scope(1).unwrap()
    );
    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.font_size(), 10.0);
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&mixed.body.face()));
            inside.extend(run.glyphs().map(|glyph| {
                (
                    glyph.range(),
                    glyph.id(),
                    run.origin().x + glyph.pen(),
                    glyph.advance(),
                )
            }));
        } else {
            assert!(run.face().is_same_face(&mixed.parent.face()));
            outside.extend(run.glyphs().map(|glyph| {
                if matches!(glyph.range().start, 1 | 5) {
                    assert_eq!(run.glyph_basis(), space.basis);
                }
                (
                    glyph.range(),
                    glyph.id(),
                    run.origin().x + glyph.pen(),
                    glyph.advance(),
                )
            }));
        }
    }
    // Normal3x10 plus actual inner WS4 gives W34. The SPACE lacks every
    // width action, so no narrower partial packet may replace this execution.
    assert_eq!(
        inside,
        [
            (Range::new(2, 3), 35, 0.0, 10.0),
            (Range::new(3, 4), 3, 12.0, 10.0),
            (Range::new(4, 5), 35, 24.0, 10.0),
        ]
    );
    assert_eq!(
        outside,
        [
            (Range::new(0, 1), 36, 0.0, 10.0),
            (Range::new(1, 2), 3, 14.0 + first / 2.0, space.advance),
            (
                Range::new(5, 6),
                3,
                second_space + second / 2.0,
                space.advance
            ),
            (Range::new(6, 7), 36, last, 10.0),
        ]
    );
    let layout = placed_vertical_cell(&mixed.prepared, line.clone());
    assert_eq!(layout.source(), mixed.processed.source());
    let packets = layout.glyph_runs();
    let body = packets
        .iter()
        .filter(|run| run.font().face().is_same_face(&mixed.body.face()))
        .collect::<Vec<_>>();
    assert!(!body.is_empty());
    for body in body {
        close(body.glyph_frame().outline_x().x, 5.0 / 17.0);
        assert_eq!(body.glyph_frame().outline_x().y, 0.0);
        assert_eq!(body.glyph_frame().outline_y(), Point::new(0.0, 1.0));
        close(body.glyph_frame().origin().y, 28.0 + cell_start);
    }
}

fn exercise(policy: SpacingPolicy) {
    let mixed = mixed_with_cell("B A A B", Range::new(2, 5), 4.0, 784_988_741);
    assert_eq!(mixed.processed.source().text(), &Text::from("B A A B"));
    assert_eq!(
        mixed
            .processed
            .captured_scope(0)
            .unwrap()
            .style()
            .unwrap()
            .word_spacing,
        0.0
    );
    assert_eq!(
        mixed
            .processed
            .captured_scope(1)
            .unwrap()
            .style()
            .unwrap()
            .word_spacing,
        4.0
    );
    let space = exterior_space_metrics(mixed.parent.bytes());
    partial_cell_width_premises(mixed.body.bytes());
    let none = mixed
        .prepared
        .select(request(&mixed.prepared))
        .unwrap()
        .finalize(allocations())
        .unwrap();
    flow(&mixed, &none, space, [0.0, 0.0]);
    let original = raw(&mixed, &none);
    let mut value = request(&mixed.prepared);
    value.context.spacing = policy;
    let selected = mixed
        .prepared
        .select(value)
        .expect("genuine exterior separator demand must admit a retained Cell");
    targets(&mixed, &selected);
    let natural = selected.finalize(allocations()).unwrap();
    flow(&mixed, &natural, space, [0.0, 0.0]);
    assert_eq!(raw(&mixed, &natural), original);
    let expanded = selected
        .finalize(LineAllocations {
            spacing: complete(&selected, [1.0, 3.0]),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    flow(&mixed, &expanded, space, [1.0, 3.0]);
    assert_eq!(
        expanded.feedback().advance,
        natural.feedback().advance + 4.0
    );
    assert_eq!(raw(&mixed, &expanded), original);
    for (row, amount) in expanded.spacing_slots().zip([1.0, 3.0]) {
        assert_eq!(
            row.geometry,
            SpacingSlotGeometry::Independent {
                intrinsic_advance: f64::from(space.advance)
            }
        );
        assert_eq!(row.authored_word_spacing, 0.0);
        assert_eq!(row.allocation, amount);
        assert!(row.can_expand());
    }
    let replay = expanded
        .with_spacing(complete(&selected, [2.0, 0.5]))
        .unwrap();
    assert_eq!(replay.selected_line(), selected);
    flow(&mixed, &replay, space, [2.0, 0.5]);
    assert_eq!(raw(&mixed, &replay), original);
    let restored = replay.with_spacing(SpacingAllocations::None).unwrap();
    flow(&mixed, &restored, space, [0.0, 0.0]);
    assert_eq!(raw(&mixed, &restored), original);
    flow(&mixed, &expanded, space, [1.0, 3.0]);
}

#[test]
fn spaces_allocates_only_real_exterior_separators_around_the_word_spaced_cell() {
    exercise(SpacingPolicy::Spaces);
}

#[test]
fn inter_word_allocates_only_real_exterior_separators_around_the_word_spaced_cell() {
    exercise(SpacingPolicy::InterWord);
}
