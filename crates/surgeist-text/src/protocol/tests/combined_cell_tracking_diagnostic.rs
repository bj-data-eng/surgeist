//! An ordinary GPOS-constrained tracking error inside a genuine Cell-active
//! preparation. The Cell is elsewhere; it cannot replace the A/B source owners.

use super::*;
use surgeist_shaper::{
    Direction, FontRef as ShaperFontRef, ShaperData, ShapingStatus, UnicodeBuffer, script,
};

fn constrained_parent() -> Vec<u8> {
    let bytes = native_metric_font(&crate::shaping_test_fonts::cross_unit_cursive_ahem());
    let font = ShaperFontRef::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    let mut input = UnicodeBuffer::new();
    input.add('A', 0);
    input.add('B', 1);
    input.set_direction(Direction::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let output = shaper.shape(input, &[]);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        output
            .glyph_infos()
            .iter()
            .map(|glyph| (glyph.glyph_id, glyph.cluster))
            .collect::<Vec<_>>(),
        [(35, 0), (36, 1)]
    );
    // Real A exit(300,500), B entry(100,200): A advance300,
    // B advance900/offset(-100,+300) in the native y-up provider frame.
    assert_eq!(
        output
            .glyph_positions()
            .iter()
            .map(|position| (
                position.x_advance,
                position.y_advance,
                position.x_offset,
                position.y_offset
            ))
            .collect::<Vec<_>>(),
        [(300, 0, 0, 0), (900, 0, -100, 300)]
    );
    bytes
}

fn attempt(parent: Vec<u8>, tracking: f64) -> crate::Result<Mixed> {
    let outside = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        letter_spacing: tracking,
        word_spacing: 0.0,
        locale: Some("en".into()),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let inside = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        letter_spacing: 0.0,
        ..outside.clone()
    };
    prepare_mixed(
        "ABAAB",
        Range::new(2, 4),
        784_991_301,
        parent,
        composition_font(),
        outside,
        inside,
    )
}

fn piece(origin: usize) -> crate::SpacingPiece {
    crate::SpacingPiece::Source {
        origin,
        local: Range::new(origin, origin + 1),
        aggregate: Range::new(origin, origin + 1),
    }
}

#[test]
fn mixed_cell_preparation_keeps_ordinary_cursive_tracking_diagnostic_source_and_locale() {
    let parent = constrained_parent();
    let mixed = attempt(parent.clone(), 0.0).unwrap();
    assert_eq!(mixed.processed.source().text(), &Text::from("ABAAB"));
    assert_eq!(mixed.prepared.orientation(), LineOrientation::Vertical);
    assert!(mixed.prepared.scope_alignment_retention().is_some());
    // None plus zero tracking may omit the character graph. The real Cell is
    // established by its prepared owner, independently of graph demand.
    let prepared_cells = mixed.prepared.cell_views().collect::<Vec<_>>();
    let [prepared_cell] = prepared_cells.as_slice() else {
        panic!("one prepared AA Cell");
    };
    assert!(prepared_cell.has_provider_input());
    assert_eq!(
        prepared_cell.source_cell().scope(),
        mixed.processed.captured_scope(1).unwrap()
    );
    let selected = mixed.prepared.select(request(&mixed.prepared)).unwrap();
    let line = selected.finalize(allocations()).unwrap();
    // AB contributes3+9, the actual AA Cell10, and the independent last B10.
    // Zero outer tracking allows the attachment and retains exactly one Cell.
    assert_eq!(line.feedback().advance, 32.0);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine AA Cell");
    };
    assert_eq!(
        cell.source_cell().scope(),
        mixed.processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().origin.x, 12.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let mut ordinary = Vec::new();
    let mut inner = Vec::new();
    for run in line.run_views() {
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&mixed.body.face()));
            inner.extend(
                run.glyphs()
                    .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
            );
        } else {
            assert!(run.face().is_same_face(&mixed.parent.face()));
            assert_eq!(run.glyph_basis(), GlyphBasis::Clockwise);
            ordinary.extend(run.glyphs().map(|glyph| {
                (
                    glyph.range(),
                    glyph.id(),
                    glyph.x(),
                    glyph.y(),
                    glyph.advance(),
                )
            }));
        }
    }
    assert_eq!(
        ordinary,
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 3.0),
            (Range::new(1, 2), 36, -1.0, -3.0, 9.0),
            (Range::new(4, 5), 36, 0.0, 0.0, 10.0),
        ]
    );
    assert_eq!(
        inner,
        [(Range::new(2, 3), 36, 5.0), (Range::new(3, 4), 36, 5.0)]
    );

    let error = match attempt(parent, 4.0) {
        Err(error) => error,
        Ok(_) => panic!("material ordinary A/B tracking must preserve the actual GPOS constraint"),
    };
    assert_eq!(error.code, ErrorCode::LayoutFailed);
    let Some(ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        reason,
        pieces,
        provider_atoms,
        contexts,
        constraints,
    }) = error.detail()
    else {
        panic!("actual constrained tracking detail: {error:?}");
    };
    assert_eq!(*stage, crate::SpacingStage::Preparation);
    assert_eq!(*operation, crate::SpacingOperation::AuthoredTracking);
    assert_eq!(*reason, crate::SpacingFailureReason::CrossUnitConstraint);
    assert_eq!(pieces.as_ref(), [piece(0), piece(1)]);
    assert_eq!(provider_atoms.as_ref(), [0..1, 1..2]);
    assert_eq!(contexts.len(), 2);
    for (context, origin) in contexts.iter().zip(0..2) {
        assert_eq!(context.pieces.as_ref(), [piece(origin)]);
        assert_eq!(context.locale.as_deref(), Some("en"));
    }
    let [constraint] = constraints.as_ref() else {
        panic!("one genuine cursive constraint");
    };
    assert_eq!(
        constraint.kind,
        crate::SpacingConstraintKind::CursiveAttachment
    );
    assert_eq!(constraint.run, 0);
    assert_eq!(constraint.components.as_ref(), [0..1, 1..2]);
    assert!(constraint.provenance.is_empty());
    // Refusal cannot alter the previously completed zero-tracking output.
    assert_eq!(line.feedback().advance, 32.0);
}
