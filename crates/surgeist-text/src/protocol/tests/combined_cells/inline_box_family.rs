//! A demanded complete family retains the genuine opaque outer Cell.
use super::*;

fn one_outer_cell_member(
    piece: &FinalizedInlineBoxPiece,
    cell: &PreparedCell,
    owner: &InlineBoxSource,
) {
    assert_eq!(piece.owner(), *owner);
    assert!(piece.start_control().is_none());
    assert!(piece.end_control().is_none());
    assert!(piece.start_edge().is_none());
    assert!(piece.end_edge().is_none());
    let members = piece.members().collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    assert_eq!(
        members[0].participant(),
        Some(&PreparedParticipant::Cell(cell.clone()))
    );
    assert_eq!(members[0].unit_source_range(), Range::new(0, 1));
}

fn cell_payload_once(line: &FinalizedLine, owner: &CapturedScope, font: &[u8]) {
    one_em(line);
    assert_eq!(line.cell_views().len(), 1);
    let cell = line.cell_views().next().unwrap();
    assert_eq!(cell.source_cell().scope(), *owner);
    assert_eq!(
        cell.source_cell().used_code_points().collect::<Vec<_>>(),
        [0x41]
    );
    assert_eq!(cell.square().origin.x, 0.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.level().get(), 0);
    assert_eq!(line.control_slots().count(), 0);
    assert_eq!(line.atomic_slots().count(), 0);
    let runs = line
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.cell().unwrap().source_cell().scope(), *owner);
    assert_eq!(run.face().bytes(), font);
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(
        run.glyphs()
            .map(|glyph| (glyph.id(), glyph.pen(), glyph.advance(), glyph.range()))
            .collect::<Vec<_>>(),
        [(35, 0.0, 10.0, Range::new(0, 1))]
    );
}

fn placed_cell_once(prepared: &PreparedText, line: FinalizedLine, font: &[u8]) -> Layout {
    let layout = placed_vertical_cell(prepared, line);
    assert_eq!(layout.lines().len(), 1);
    // Verified native BASE0/1200 at size10 gives Central6. E10 is centered
    // about actual reference100; the supplied inline origin is20.
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let runs = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.face().bytes(), font);
    assert_eq!(run.range(), Range::new(0, 1));
    assert_eq!(run.orientation(), LineOrientation::Vertical);
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 10.0)]
    );
    // Actual inner hhea8/-2 and normal[0,10] give baseline8, scale1/offset0.
    let frame = run.glyph_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    drop(runs);
    layout
}

#[test]
fn complete_box_family_membership_keeps_the_real_cell_opaque_without_duplicating_its_packet() {
    let (mut system, processed, analyzed, body) = setup("A", WhiteSpace::Preserve);
    glyph_frame_font_premises(&body);
    let scope = processed.captured_scope(1).unwrap();
    let domain = processed.captured_scope(0).unwrap();
    assert_eq!(scope.role(), ScopeRole::Inline);
    assert_eq!(scope.parent(), Some(domain.clone()));
    let owner = InlineBoxSource::captured(scope.clone());
    let analyzed_cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    assert_eq!(analyzed_cell.scope(), scope);
    assert_eq!(analyzed_cell.parent_scope(), Some(domain.clone()));
    assert_eq!(
        (
            analyzed_cell.selection_unit_count(),
            analyzed_cell.used_unit_count()
        ),
        (1, 1)
    );
    let originals = analyzed_cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(originals.len(), 1);
    assert_eq!(originals[0].source(), processed.source());
    assert_eq!(originals[0].aggregate(), Range::new(0, 1));
    assert_eq!(originals[0].local(), Range::new(0, 1));
    assert_eq!(
        originals[0].origin_position(),
        processed.captured_origin(0).unwrap()
    );
    assert_eq!(
        originals[0].paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let captured = prepared.captured_owner().unwrap();
    assert_eq!(captured.domain(), domain);
    assert_eq!(
        captured.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert!(captured.joint_flow().is_none());
    assert_eq!(prepared.source_segments().len(), 1);
    assert_eq!(prepared.cell_views().len(), 1);
    let cell = prepared.cell_views().next().unwrap().handle();
    assert_eq!(cell.view().extent(), 10.0);
    assert_eq!(cell.view().source_cell().scope(), scope);
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [PreparedParticipant::Cell(cell.clone())]
    );
    let prior = finalize(&prepared);
    assert!(prior.inline_box_family().is_none());
    assert!(prior.inline_box_pieces().is_none());
    cell_payload_once(&prior, &scope, &body);
    let prior_layout = placed_cell_once(&prepared, prior.clone(), &body);
    assert_eq!(prior_layout.inline_box_pieces().count(), 0);
    let limits = domain.scope_alignment_limits().unwrap();
    let family = prepared
        .inline_box_family(
            Vec::new(),
            Some(ScopeAlignmentAllowance::new(
                limits.records(),
                limits.payload_bytes(),
            )),
        )
        .unwrap();
    assert_eq!(family.prepared_text(), prepared);
    let mut request = prior.selected_line().request().clone();
    assert!(request.inline_boxes.is_none());
    request.inline_boxes = Some(family.clone());
    let selected = prepared.select(request).unwrap();
    assert_eq!(selected.inline_box_family(), Some(&family));
    let pieces = selected
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    let members = pieces[0].members().collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    assert_eq!(
        members[0].participant(),
        Some(&PreparedParticipant::Cell(cell.clone()))
    );
    assert_eq!(members[0].unit_source_range(), Range::new(0, 1));
    let fragment = pieces[0].fragment();
    assert_eq!(fragment.owner(), owner);
    let root = fragment.parent().unwrap();
    assert_eq!(root.owner(), InlineBoxSource::captured(domain));
    assert!(root.parent().is_none());
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.inline_box_family(), Some(&family));
    cell_payload_once(&line, &scope, &body);
    let pieces = line
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].fragment(), fragment);
    one_outer_cell_member(&pieces[0], &cell, &owner);
    let layout = placed_cell_once(&prepared, line.clone(), &body);
    assert_eq!(layout.source(), processed.source());
    let pieces = layout
        .inline_box_pieces()
        .filter(|piece| piece.fragment().owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].family(), &family);
    assert_eq!(pieces[0].fragment(), fragment);
    assert_eq!(pieces[0].line_index(), 0);
    assert_eq!(pieces[0].anchor(), Point::new(100.0, 20.0));
    assert_eq!(pieces[0].orientation(), LineOrientation::Vertical);
    one_outer_cell_member(&pieces[0].finalized_piece(), &cell, &owner);
    let stats = system.stats();
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(system.stats(), stats);
    assert_eq!(replay.inline_box_family(), Some(&family));
    cell_payload_once(&replay, &scope, &body);
    let replayed = replay
        .inline_box_pieces()
        .unwrap()
        .filter(|piece| piece.owner() == owner)
        .collect::<Vec<_>>();
    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0].fragment(), fragment);
    one_outer_cell_member(&replayed[0], &cell, &owner);
    cell_payload_once(&prior, &scope, &body);
}
