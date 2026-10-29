//! A component-local unit can straddle an atomic only after actual L2 order.
//! Ordinary zero is qualified before demanding material or complete spacing.
use super::*;

fn visual_region_error(
    error: &crate::Error,
    stage: crate::SpacingStage,
    operation: crate::SpacingOperation,
    tracking: f64,
) {
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    let Some(crate::ErrorDetail::UnsupportedSpacing {
        stage: actual_stage,
        operation: actual_operation,
        pieces,
        provider_atoms,
        reason,
        contexts,
        constraints,
    }) = error.detail()
    else {
        panic!("expected actual visually interleaved unit custody: {error:?}")
    };
    assert_eq!((*actual_stage, *actual_operation), (stage, operation));
    assert_eq!(*reason, crate::SpacingFailureReason::InterleavedUnit);
    // X9 formats retain their original custody. L2 reverses the level-one
    // mark/atomic/PDF sequence; no source hull and no following B belongs here.
    assert_eq!(
        pieces.as_ref(),
        [
            piece(Range::new(0, 1)),
            piece(Range::new(1, 2)),
            piece(Range::new(3, 4)),
            crate::SpacingPiece::Atomic { occurrence: 0 },
            piece(Range::new(2, 3)),
        ]
    );
    // Genuine provider clusters may cover adjacent zero-format atoms. Their
    // exact ordered atom custody is invariant under that cluster grouping.
    assert_eq!(
        provider_atoms
            .iter()
            .flat_map(|range| range.clone())
            .collect::<Vec<_>>(),
        [0, 1, 3, 2]
    );
    assert!(constraints.is_empty());
    assert_eq!(contexts.len(), 2);
    let text = &contexts[0];
    assert_eq!(
        text.pieces.as_ref(),
        [
            piece(Range::new(0, 1)),
            piece(Range::new(1, 2)),
            piece(Range::new(2, 3))
        ]
    );
    assert_eq!(text.locale, None);
    // Cf is ignored within the real left component. The unit's base is A,
    // whose raw ScriptExtensions is Latin; the mark has not crossed a barrier.
    use icu_properties::{props::Script, script::ScriptWithExtensions};
    assert_eq!(
        ScriptWithExtensions::new()
            .get_script_extensions_val32(u32::from('A'))
            .iter()
            .collect::<Vec<_>>(),
        [Script::Latin]
    );
    assert_eq!(
        text.candidates
            .iter()
            .map(|script| script.iso15924())
            .collect::<Vec<_>>(),
        [*b"Latn"]
    );
    assert_eq!(
        text.resolved_scripts
            .iter()
            .map(|script| script.iso15924())
            .collect::<Vec<_>>(),
        [*b"Latn"]
    );
    assert!(text.writing_systems.is_empty());
    assert_eq!(
        (text.letter_spacing_before, text.letter_spacing_after),
        (tracking, tracking)
    );
    let atomic = &contexts[1];
    assert_eq!(
        atomic.pieces.as_ref(),
        [crate::SpacingPiece::Atomic { occurrence: 0 }]
    );
    assert_eq!(atomic.locale, None);
    assert!(atomic.candidates.is_empty());
    assert!(atomic.resolved_scripts.is_empty());
    assert!(atomic.writing_systems.is_empty());
    assert_eq!(
        (atomic.letter_spacing_before, atomic.letter_spacing_after),
        (0.0, 0.0)
    );
}

#[test]
fn component_local_a_mark_unit_can_straddle_an_inflow_atomic_after_real_bidi_reordering() {
    use icu_properties::{
        CodePointMapData, CodePointSetData,
        props::{BidiClass, DefaultIgnorableCodePoint, GeneralCategory},
    };
    let classes = CodePointMapData::<BidiClass>::new();
    assert_eq!(
        ['A', '\u{202e}', '\u{301}', '\u{202c}', 'B'].map(|ch| classes.get(ch)),
        [
            BidiClass::LeftToRight,
            BidiClass::RightToLeftOverride,
            BidiClass::NonspacingMark,
            BidiClass::PopDirectionalFormat,
            BidiClass::LeftToRight
        ]
    );
    for ch in ['\u{202e}', '\u{202c}'] {
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get(ch),
            GeneralCategory::Format
        );
        assert!(CodePointSetData::new::<DefaultIgnorableCodePoint>().contains(ch));
    }
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let mut source = Source::new("A\u{202e}\u{301}\u{202c}B");
    source.inline_box(
        InlineBox::new(BOX, InlineBoxKind::InFlow, 3, Size::new(10.0, 10.0))
            .try_with_letter_spacing(0.0)
            .unwrap(),
    );
    let mut system = system_with(&bytes);
    let prepared = prepared(&mut system, &source, own(0.0)).unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    // The atomic is at the end of A/RLO/mark, not inside that component.
    // Cf tailoring inside it gives A+mark; the right component is PDF/B.
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [
            Range::new(0, 1),
            Range::new(1, 2),
            Range::new(2, 3),
            Range::new(3, 4),
            Range::new(4, 5)
        ]
    );
    // RLO's explicit level is one. A/B remain level zero; X9 readback retains
    // the preceding level for RLO/PDF. The atomic occurs before the pop.
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| (unit.source_range(), unit.level().get()))
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 0),
            (Range::new(1, 2), 0),
            (Range::new(2, 3), 1),
            (Range::new(3, 4), 1),
            (Range::new(4, 5), 0)
        ]
    );
    let atomic = prepared.atomic_views().next().unwrap();
    assert_eq!(atomic.original_box().index(), 3);
    assert_eq!(atomic.level().get(), 1);

    // Execute every ordinary boundary before asserting that this is a genuine
    // interleave witness. A provider/composition rejection must remain visible.
    let natural = ordinary(&prepared);
    let layout = placed(&prepared, &natural);
    let old_raw = raw(&natural, &bytes);
    let old_feedback = natural.feedback();
    let old_carets = carets(&layout);
    assert_eq!(layout.source().text(), source.text());
    assert_eq!(layout.inline_boxes().len(), 1);
    let cells: Vec<_> = natural
        .run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(move |glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.advance(),
                        origin.x + glyph.pen(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let a: Vec<_> = cells.iter().filter(|cell| cell.1 == 35).copied().collect();
    let mark: Vec<_> = cells.iter().filter(|cell| cell.1 == 100).copied().collect();
    let b: Vec<_> = cells.iter().filter(|cell| cell.1 == 36).copied().collect();
    assert_eq!(a, [(Range::new(0, 1), 35, 10.0, 0.0)]);
    assert_eq!(mark.len(), 1);
    assert_eq!((mark[0].0, mark[0].2), (Range::new(2, 3), 0.0));
    assert_eq!(b.len(), 1);
    assert_eq!((b[0].0, b[0].2), (Range::new(4, 5), 10.0));
    let slot = natural.atomic_slots().next().unwrap();
    assert_eq!(slot.origin.x, 10.0);
    assert_eq!(slot.allocation, flow());
    // Cells, not outline paint: a mark may have a negative raw attachment
    // offset. Any actual repair glyph remains in the retained raw packet.
    assert!(mark[0].3 >= 20.0);
    assert!(b[0].3 >= mark[0].3);
    assert_eq!(natural.feedback().advance, b[0].3 + 10.0);

    let error = selection(
        &prepared,
        SpacingPolicy::InterCharacter,
        Some(natural.feedback().advance),
    )
    .unwrap_err();
    visual_region_error(
        &error,
        crate::SpacingStage::Selection,
        crate::SpacingOperation::InterCharacterBasis,
        0.0,
    );
    let error = self::prepared(&mut system, &source, own(4.0)).unwrap_err();
    visual_region_error(
        &error,
        crate::SpacingStage::Preparation,
        crate::SpacingOperation::AuthoredTracking,
        4.0,
    );
    assert_eq!(raw(&natural, &bytes), old_raw);
    assert_eq!(natural.feedback(), old_feedback);
    assert_eq!(carets(&layout), old_carets);
    assert_eq!(raw(&ordinary(&prepared), &bytes), old_raw);
}
