//! Public outer allocations over real B/B/Cell/B participants. The Cell keeps
//! its genuine hwid packet; no inner A/A boundary becomes an outer slot.

use super::*;

fn origin(mixed: &Mixed, ordinal: usize) -> PreparedOrigin {
    let capture = mixed.processed.captured_origin(ordinal).unwrap();
    let view = mixed
        .prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&capture))
        .unwrap();
    assert_eq!(view.source(), mixed.processed.source());
    assert_eq!(view.local_range(), Range::new(ordinal, ordinal + 1));
    assert_eq!(view.aggregate_range(), Range::new(ordinal, ordinal + 1));
    view.handle()
}

fn originals(mixed: &Mixed, ordinals: &[usize]) -> Vec<SpacingSlotOwner> {
    ordinals
        .iter()
        .map(|&ordinal| SpacingSlotOwner::Source {
            origin: origin(mixed, ordinal),
            local: Range::new(ordinal, ordinal + 1),
            aggregate: Range::new(ordinal, ordinal + 1),
        })
        .collect()
}

fn endpoint(
    mixed: &Mixed,
    unit: &SpacingUnit,
    participant: &PreparedParticipant,
    ordinals: &[usize],
    tracking: f64,
) {
    assert_eq!(
        (unit.letter_spacing_before(), unit.letter_spacing_after()),
        (tracking, tracking)
    );
    assert_eq!(unit.participants().collect::<Vec<_>>(), [participant]);
    assert_eq!(
        unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
        originals(mixed, ordinals)
    );
}

fn complete(selected: &SelectedLine) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 3);
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip([1.0, 2.0, 3.0])
            .map(|(slot, amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

fn exact_slots(mixed: &Mixed, selected: &SelectedLine, participants: &[PreparedParticipant]) {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 3);
    let roots: [&[usize]; 4] = [&[0], &[1], &[2, 3], &[4]];
    let tracking = [4.0, 4.0, 2.0, 4.0];
    for (index, slot) in slots.iter().enumerate() {
        assert_eq!(slot.selected_line(), *selected);
        let SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            panic!("one real exterior character boundary");
        };
        endpoint(
            mixed,
            &visual_before,
            &participants[index],
            roots[index],
            tracking[index],
        );
        endpoint(
            mixed,
            &visual_after,
            &participants[index + 1],
            roots[index + 1],
            tracking[index + 1],
        );
        let mut expected = originals(mixed, roots[index]);
        expected.extend(originals(mixed, roots[index + 1]));
        assert_eq!(
            slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            slot.participants().cloned().collect::<Vec<_>>(),
            [participants[index].clone(), participants[index + 1].clone()]
        );
    }
    // The same issued Cell is the whole endpoint on BOTH exterior boundaries.
    // Its two genuine originals are not issued as an interior fourth slot.
    let SpacingSlotTarget::Boundary {
        visual_after: entering,
        ..
    } = slots[1].target()
    else {
        unreachable!()
    };
    let SpacingSlotTarget::Boundary {
        visual_before: leaving,
        ..
    } = slots[2].target()
    else {
        unreachable!()
    };
    assert_eq!(
        entering.participants().cloned().collect::<Vec<_>>(),
        leaving.participants().cloned().collect::<Vec<_>>()
    );
    assert_eq!(
        entering
            .pieces()
            .map(|piece| piece.owner())
            .collect::<Vec<_>>(),
        originals(mixed, &[2, 3])
    );
    assert_eq!(
        leaving
            .pieces()
            .map(|piece| piece.owner())
            .collect::<Vec<_>>(),
        originals(mixed, &[2, 3])
    );
}

fn flow(mixed: &Mixed, line: &FinalizedLine, starts: [f64; 4], total: f64) {
    assert_eq!(line.feedback().advance, total);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine outer Cell");
    };
    assert_eq!(
        cell.source_cell().scope(),
        mixed.processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().origin.x, starts[2]);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.scope_displacement().get(), 0.0);
    let mut outside = Vec::new();
    let mut inside = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.font_size(), 10.0);
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&mixed.body.face()));
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            inside.extend(run.glyphs().map(|glyph| {
                (
                    glyph.range(),
                    glyph.id(),
                    glyph.pen(),
                    glyph.x(),
                    glyph.y(),
                    glyph.advance(),
                )
            }));
        } else {
            assert!(run.face().is_same_face(&mixed.parent.face()));
            outside.extend(run.glyphs().map(|glyph| {
                assert_eq!((glyph.id(), glyph.advance()), (36, 10.0));
                (glyph.range(), run.origin().x + glyph.pen())
            }));
        }
    }
    assert_eq!(
        inside,
        [
            (Range::new(2, 3), 36, 0.0, 0.0, 0.0, 5.0),
            (Range::new(3, 4), 36, 5.0, 0.0, 0.0, 5.0),
        ]
    );
    assert_eq!(
        outside,
        [
            (Range::new(0, 1), starts[0]),
            (Range::new(1, 2), starts[1]),
            (Range::new(4, 5), starts[3]),
        ]
    );
    let layout = placed_vertical_cell(&mixed.prepared, line.clone());
    assert_eq!(layout.source(), mixed.processed.source());
    let start = Cursor::new(2, Affinity::After);
    let end = Cursor::new(4, Affinity::Before);
    assert_eq!(
        layout.cursor(start).unwrap().rect().origin.y,
        20.0 + starts[2]
    );
    assert_eq!(
        layout.cursor(end).unwrap().rect().origin.y,
        20.0 + starts[3]
    );
    let positions = layout.positions().collect::<Vec<_>>();
    let first = positions
        .iter()
        .find(|position| position.source_cursor() == start)
        .unwrap();
    let last = positions
        .iter()
        .find(|position| position.source_cursor() == end)
        .unwrap();
    let source = layout.position_selection(first, last).unwrap();
    assert_eq!(source.source_text(), &Text::from("AA"));
    assert_eq!(source.plaintext(), &Text::from("AA"));
    assert_eq!(source.source_pieces().len(), 2);
    for (piece, ordinal) in source.source_pieces().iter().zip(2..4) {
        assert_eq!(piece.local_range(), Range::new(ordinal, ordinal + 1));
        assert_eq!(piece.aggregate_range(), Range::new(ordinal, ordinal + 1));
        assert_eq!(piece.origin(), &origin(mixed, ordinal));
    }
}

fn feedback(line: &FinalizedLine, amounts: [f64; 3]) {
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 3);
    assert!(
        rows.windows(2)
            .all(|pair| pair[0].traversal_order < pair[1].traversal_order)
    );
    for ((row, authored), amount) in rows.iter().zip([4.0, 3.0, 3.0]).zip(amounts) {
        assert_eq!(
            row.geometry,
            SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent)
        );
        assert_eq!(row.stretchability, SpacingStretchability::Direct);
        assert_eq!(row.authored_word_spacing, 0.0);
        assert_eq!(row.authored_letter_spacing, authored);
        assert_eq!(row.allocation, amount);
        assert_eq!(row.advance, authored + amount);
        assert!(row.can_expand());
    }
}

#[test]
fn actual_outer_cell_boundaries_accept_complete_allocations_without_spacing_inner_glyphs() {
    let mixed = mixed();
    assert_eq!(mixed.processed.source().text(), &Text::from("BBAAB"));
    assert!(!mixed.parent.face().is_same_face(&mixed.body.face()));
    let none = mixed
        .prepared
        .select(request(&mixed.prepared))
        .unwrap()
        .finalize(allocations())
        .unwrap();
    geometry(&mixed, &none);
    flow(&mixed, &none, [0.0, 14.0, 27.0, 40.0], 50.0);
    let participants = mixed.prepared.participants().collect::<Vec<_>>();
    let [
        PreparedParticipant::Text(first),
        PreparedParticipant::Text(second),
        PreparedParticipant::Cell(cell),
        PreparedParticipant::Text(last),
    ] = participants.as_slice()
    else {
        panic!("actual B/B/Cell/B prepared inventory");
    };
    assert_eq!(first.view().source_range(), Range::new(0, 1));
    assert_eq!(second.view().source_range(), Range::new(1, 2));
    assert_eq!(last.view().source_range(), Range::new(4, 5));
    assert_eq!(
        cell.view().source_cell().scope(),
        mixed.processed.captured_scope(1).unwrap()
    );
    let mut value = request(&mixed.prepared);
    value.context.spacing = SpacingPolicy::InterCharacter;
    // The genuine whole Cell receives allocations only at its exterior
    // boundaries; its two inner A glyphs remain one outer target unit.
    let selected = mixed
        .prepared
        .select(value)
        .expect("real outer Cell boundaries must admit InterCharacter allocation");
    exact_slots(&mixed, &selected, &participants);
    let natural = selected.finalize(allocations()).unwrap();
    flow(&mixed, &natural, [0.0, 14.0, 27.0, 40.0], 50.0);
    feedback(&natural, [0.0; 3]);
    // Authored boundary means4/3/3 plus complete amounts1/2/3 give5/5/6.
    // Four genuine outer advances10 produce starts0/15/30/46 and total56.
    let expanded = selected
        .finalize(LineAllocations {
            spacing: complete(&selected),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    flow(&mixed, &expanded, [0.0, 15.0, 30.0, 46.0], 56.0);
    feedback(&expanded, [1.0, 2.0, 3.0]);
    let replay = expanded.with_spacing(complete(&selected)).unwrap();
    assert_eq!(replay.selected_line(), selected);
    flow(&mixed, &replay, [0.0, 15.0, 30.0, 46.0], 56.0);
    feedback(&replay, [1.0, 2.0, 3.0]);
    let restored = replay.with_spacing(SpacingAllocations::None).unwrap();
    flow(&mixed, &restored, [0.0, 14.0, 27.0, 40.0], 50.0);
    feedback(&restored, [0.0; 3]);
    flow(&mixed, &expanded, [0.0, 15.0, 30.0, 46.0], 56.0);
}

#[test]
fn ordinary_bb_uses_the_same_real_parent_font_and_existing_allocated_tracking_contract() {
    let mixed = mixed();
    let own = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        letter_spacing: 4.0,
        wrap: Wrap::Word,
        ..Style::default()
    };
    let source = Source::identified("BB", Id::from_u64(784_987_341), 1);
    let extent = Range::new(0, 2);
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: own,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                origins: vec![ParagraphOrigin {
                    source: source.clone(),
                    local: extent,
                    aggregate: extent,
                    kind: ParagraphOriginKind::Authored,
                    text_node: None,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        Wrap::Word,
    );
    assert_eq!(analyzed.combined_cells().len(), 0);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(mixed.parent.bytes().to_vec())
        .unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.cell_views().len(), 0);
    let mut value = request(&prepared);
    value.context.orientation = LineOrientation::Horizontal;
    value.context.spacing = SpacingPolicy::InterCharacter;
    let selected = prepared.select(value).unwrap();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let natural = selected.finalize(allocations()).unwrap();
    assert_eq!(natural.feedback().advance, 24.0);
    let expanded = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: slots[0].clone(),
                amount: 1.0,
            }]),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(expanded.feedback().advance, 25.0);
    let actual = expanded
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), mixed.parent.bytes());
            run.glyphs()
                .map(|glyph| (glyph.id(), run.origin().x + glyph.pen(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, [(36, 0.0, 10.0), (36, 15.0, 10.0)]);
    assert_eq!(
        expanded
            .with_spacing(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        24.0
    );
}
