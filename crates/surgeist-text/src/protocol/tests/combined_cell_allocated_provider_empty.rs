//! Allocation moves the real provider-empty Cell as one opaque outer unit.
//! Endpoint custody retains its selected empty presentation and original SPACE;
//! no empty glyph, text cluster or guessed source hull stands in for that unit.

use super::*;

fn request(prepared: &PreparedText) -> LineSelectionRequest {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    LineSelectionRequest {
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        participants: prepared.participants().collect(),
        termination: PreparedTermination::Ordinary(segment.end_opportunity()),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: LineContext {
            alignment_insets: LineAlignmentInsets::ZERO,
            formatted_line: FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Vertical,
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing: SpacingPolicy::InterCharacter,
        },
    }
}

fn endpoint(
    unit: SpacingUnit,
    participant: &PreparedParticipant,
    origin: &PreparedOrigin,
    source: Range,
    presentation: (usize, usize),
) {
    assert_eq!(unit.participants().collect::<Vec<_>>(), [participant]);
    assert_eq!(unit.letter_spacing_before(), 0.0);
    assert_eq!(unit.letter_spacing_after(), 0.0);
    let pieces = unit.pieces().collect::<Vec<_>>();
    let [piece] = pieces.as_slice() else {
        panic!("one actual endpoint original piece");
    };
    assert_eq!(
        piece.owner(),
        SpacingSlotOwner::Source {
            origin: origin.clone(),
            local: source,
            aggregate: source
        }
    );
    assert_eq!(
        (
            piece.presentation_range().start(),
            piece.presentation_range().end()
        ),
        presentation
    );
}

#[test]
fn allocated_character_gaps_keep_provider_empty_cell_square_and_authentic_endpoint_custody() {
    let (mut system, processed, analyzed, font) = tracked_empty_cell(0.0);
    assert_eq!(processed.source().text(), &Text::from("B B"));
    let source_cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(source_cell.selection_unit_count(), 1);
    assert_eq!(source_cell.used_unit_count(), 0);
    assert_eq!(source_cell.presentation_code_points().len(), 0);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let participants = prepared.participants().collect::<Vec<_>>();
    let [
        PreparedParticipant::Text(before),
        PreparedParticipant::Cell(cell),
        PreparedParticipant::Text(after),
    ] = participants.as_slice()
    else {
        panic!("real ordinary B, opaque Cell, ordinary B sequence");
    };
    assert_eq!(before.view().source_range(), Range::new(0, 1));
    assert_eq!(after.view().source_range(), Range::new(2, 3));
    assert_eq!(cell.view().extent(), 10.0);
    assert!(!cell.view().has_provider_input());
    assert_eq!(
        cell.view().source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    let origins = (0..3)
        .map(|index| {
            let capture = processed.captured_origin(index).unwrap();
            prepared
                .origin_views()
                .find(|view| view.capture_position().as_ref() == Some(&capture))
                .unwrap()
                .handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(origins[1].view().local_range(), Range::new(1, 2));
    let natural = finalize(&prepared);
    // Existing callable None qualification precedes the non-None guard: fixed
    // E10, no body packet, true font/payloads, source and placed flow/carets.
    tracked_cell_line(&prepared, &natural, &processed, &font, 30.0, 10.0, 20.0);
    let selected = prepared
        .select(request(&prepared))
        .expect("two real exterior boundaries around a provider-empty Cell must be allocatable");
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slots[0].target()
    else {
        panic!("first exterior character boundary");
    };
    endpoint(
        visual_before,
        &participants[0],
        &origins[0],
        Range::new(0, 1),
        (0, 1),
    );
    // Source processing emits the left B, no inner scalar, then the right B.
    // The genuine selected Cell's empty cut is therefore1..1, not a glyph ID.
    endpoint(
        visual_after,
        &participants[1],
        &origins[1],
        Range::new(1, 2),
        (1, 1),
    );
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slots[1].target()
    else {
        panic!("second exterior character boundary");
    };
    endpoint(
        visual_before,
        &participants[1],
        &origins[1],
        Range::new(1, 2),
        (1, 1),
    );
    endpoint(
        visual_after,
        &participants[2],
        &origins[2],
        Range::new(2, 3),
        (1, 2),
    );
    let amounts = slots
        .into_iter()
        .zip([2.0, 3.0])
        .map(|(slot, amount)| SpacingAllocation { slot, amount })
        .collect();
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::Complete(amounts),
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    // Full boundary amounts belong to their preceding visual owner: B12,
    // square12..22 with flow end25, then B25..35. The square stays E10.
    tracked_cell_line(&prepared, &line, &processed, &font, 35.0, 12.0, 25.0);
    let layout = placed_vertical_cell(&prepared, line.clone());
    let error = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceRangeFragments {
            reason: SourceRangeFragmentError::UnsupportedCell { line: 0, cell: 0 },
        })
    ));
    drop(system);
    let replay_amounts = selected
        .spacing_slots()
        .zip([2.0, 3.0])
        .map(|(slot, amount)| SpacingAllocation { slot, amount })
        .collect();
    let replay = line
        .with_spacing(SpacingAllocations::Complete(replay_amounts))
        .unwrap();
    tracked_cell_line(&prepared, &replay, &processed, &font, 35.0, 12.0, 25.0);
}
