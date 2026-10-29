//! Ignoring Cf for character-unit properties stays inside genuine components.
//! These are new complete-basis/profile qualifications, not a historical RED.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Barrier {
    Atomic,
    Disconnected,
}

fn separated(
    system: &mut System,
    source: &Source,
    style: Style,
    barrier: Barrier,
    at: usize,
) -> crate::Result<PreparedText> {
    if matches!(barrier, Barrier::Atomic) {
        assert_eq!(source.boxes().len(), 1);
        assert_eq!(source.boxes()[0].index(), at);
        return prepared(system, source, style);
    }
    assert!(source.boxes().is_empty());
    let end = source.text().len();
    // Two actual original occurrences, connected by the native disconnected
    // relation. Local intervals deliberately use the retained original Source.
    let input = ParagraphInput {
        source: source.clone(),
        base_style: style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: [Range::new(0, at), Range::new(at, end)]
            .into_iter()
            .map(|range| crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: crate::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![crate::ParagraphConnection {
            at,
            kind: crate::ParagraphConnectionKind::Disconnected,
        }],
        controls: vec![],
        replacements: vec![],
        anchor_order: vec![],
        boundary_policies: vec![],
    };
    let analyzed =
        AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word);
    system.prepare(&analyzed, PreparationOptions::default())
}

fn source_with_barrier(text: &str, barrier: Barrier, at: usize) -> Source {
    let mut source = Source::new(text);
    if matches!(barrier, Barrier::Atomic) {
        source.inline_box(
            InlineBox::new(BOX, InlineBoxKind::InFlow, at, Size::new(10.0, 10.0))
                .try_with_letter_spacing(0.0)
                .unwrap(),
        );
    }
    source
}

fn owner_piece(barrier: Barrier, range: Range) -> crate::SpacingPiece {
    crate::SpacingPiece::Source {
        origin: usize::from(matches!(barrier, Barrier::Disconnected)),
        local: range,
        aggregate: range,
    }
}

fn component_signature(line: &FinalizedLine, bytes: &[u8]) -> Vec<(u32, f32, f32, f32, f64, f64)> {
    // Retain the actual visible bases, the mark and any provider repair glyph.
    // Additional zero-advance non-mark glyphs may represent the inserted Cf.
    // No assumption that the baseline lacks a dotted-circle repair is needed.
    raw(line, bytes)
        .into_iter()
        .zip(paint_offsets(line))
        .filter_map(|((_, id, x, y, advance), (inline, cross))| {
            (advance != 0.0 || id == 100).then_some((id, x, y, advance, inline, cross))
        })
        .collect()
}

fn qualifies_components(barrier: Barrier) {
    use icu_properties::{
        CodePointMapData, CodePointSetData,
        props::{DefaultIgnorableCodePoint, GeneralCategory},
    };
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get('\u{200d}'),
        GeneralCategory::Format
    );
    assert!(CodePointSetData::new::<DefaultIgnorableCodePoint>().contains('\u{200d}'));
    let bytes = crate::shaping_test_fonts::mark_ahem();
    font_premises(&bytes, true);
    let mut system = system_with(&bytes);
    let control_source = source_with_barrier("A\u{301}B", barrier, 1);
    let control = separated(&mut system, &control_source, own(0.0), barrier, 1).unwrap();
    assert_eq!(
        control.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    let control_line = ordinary(&control);
    let control_layout = placed(&control, &control_line);
    assert_eq!(control_layout.source().text(), control_source.text());
    let control_signature = component_signature(&control_line, &bytes);
    for id in [35, 100, 36] {
        assert_eq!(
            control_signature
                .iter()
                .filter(|glyph| glyph.0 == id)
                .count(),
            1
        );
    }
    let control_raw = raw(&control_line, &bytes);
    let control_carets = carets(&control_layout);
    let control_slots: Vec<_> = control_line
        .atomic_slots()
        .map(|slot| (slot.origin, slot.allocation))
        .collect();

    // First establish that this real Cf-free composition supports ordinary
    // zero but its standalone mark cannot supply a complete character basis.
    let error = selection(
        &control,
        SpacingPolicy::InterCharacter,
        Some(control_line.feedback().advance),
    )
    .unwrap_err();
    mark_profile_error(
        &error,
        crate::SpacingStage::Selection,
        crate::SpacingOperation::InterCharacterBasis,
        &[owner_piece(barrier, Range::new(1, 2))],
        std::slice::from_ref(&(1..2)),
        0.0,
        None,
    );

    for at in [1, 2] {
        let source = source_with_barrier("A\u{200d}\u{301}B", barrier, at);
        let prepared = separated(&mut system, &source, own(0.0), barrier, at).unwrap();
        // Inserting Cf on either side cannot reconnect the two components.
        // On the right it belongs to the canonical ZWJ+mark EGC; on the left
        // it belongs to A+ZWJ, leaving the right mark entirely on its own.
        let graphemes = if at == 1 {
            vec![Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
        } else {
            vec![Range::new(0, 2), Range::new(2, 3), Range::new(3, 4)]
        };
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            graphemes
        );
        let natural = ordinary(&prepared);
        let layout = placed(&prepared, &natural);
        assert_eq!(layout.source().text(), source.text());
        assert_eq!(component_signature(&natural, &bytes), control_signature);
        assert_eq!(natural.feedback().advance, control_line.feedback().advance);
        assert_eq!(
            natural.measurements().alignment(),
            control_line.measurements().alignment()
        );
        let slots: Vec<_> = natural
            .atomic_slots()
            .map(|slot| (slot.origin, slot.allocation))
            .collect();
        assert_eq!(slots, control_slots);
        if matches!(barrier, Barrier::Atomic) {
            let atomic = prepared.atomic_views().next().unwrap();
            assert_eq!(atomic.original_box().id(), BOX);
            assert_eq!(atomic.original_box().index(), at);
            assert_eq!(layout.inline_boxes().len(), 1);
        } else {
            assert_eq!(
                prepared
                    .origin_views()
                    .map(|origin| (origin.local_range(), origin.aggregate_range()))
                    .collect::<Vec<_>>(),
                [
                    (Range::new(0, at), Range::new(0, at)),
                    (Range::new(at, 4), Range::new(at, 4))
                ]
            );
            assert!(layout.inline_boxes().is_empty());
        }
        let (mark_pieces, provider_atoms) = if at == 1 {
            (
                vec![
                    owner_piece(barrier, Range::new(1, 2)),
                    owner_piece(barrier, Range::new(2, 3)),
                ],
                std::iter::once(1..3).collect::<Vec<_>>(),
            )
        } else {
            (
                vec![owner_piece(barrier, Range::new(2, 3))],
                std::iter::once(2..3).collect::<Vec<_>>(),
            )
        };
        let old_raw = raw(&natural, &bytes);
        let old_feedback = natural.feedback();
        let old_carets = carets(&layout);
        // Capacity equals the real natural result: even zero extra must not
        // issue an allegedly complete subset that joins across the barrier.
        let error = selection(
            &prepared,
            SpacingPolicy::InterCharacter,
            Some(natural.feedback().advance),
        )
        .unwrap_err();
        mark_profile_error(
            &error,
            crate::SpacingStage::Selection,
            crate::SpacingOperation::InterCharacterBasis,
            &mark_pieces,
            &provider_atoms,
            0.0,
            None,
        );
        let error = separated(&mut system, &source, own(4.0), barrier, at).unwrap_err();
        mark_profile_error(
            &error,
            crate::SpacingStage::Preparation,
            crate::SpacingOperation::AuthoredTracking,
            &mark_pieces,
            &provider_atoms,
            4.0,
            None,
        );
        assert_eq!(raw(&natural, &bytes), old_raw);
        assert_eq!(natural.feedback(), old_feedback);
        assert_eq!(carets(&layout), old_carets);
        assert_eq!(raw(&ordinary(&prepared), &bytes), old_raw);
    }
    assert_eq!(raw(&control_line, &bytes), control_raw);
    assert_eq!(carets(&control_layout), control_carets);
}

#[test]
fn ignored_format_on_either_side_of_an_inflow_atomic_cannot_stitch_character_components() {
    qualifies_components(Barrier::Atomic);
}

#[test]
fn ignored_format_on_either_side_of_a_disconnection_cannot_stitch_original_occurrences() {
    qualifies_components(Barrier::Disconnected);
}
