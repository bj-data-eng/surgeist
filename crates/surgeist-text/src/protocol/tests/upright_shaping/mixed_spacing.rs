//! Authored spacing, InterWord replay and real selected line edges in Mixed.
//! Text3 §§7.1/7.2 and Text4 §4.3/§8.1, with native predecessor-owned cells.
use super::*;
use crate::{Affinity, Cursor, SelectedLine, Selection, SpacingAllocation, SpacingSlotOwner};

type Raw = (Range, GlyphBasis, u32, f64, f64, f64);
fn raw(line: &FinalizedLine, bytes: &[u8]) -> Vec<Raw> {
    packets(line, bytes)
        .into_iter()
        .map(|p| (p.0, p.1, p.2, p.4, p.5, p.6))
        .collect()
}
fn offsets(line: &FinalizedLine, bytes: &[u8]) -> Vec<f64> {
    packets(line, bytes).iter().map(|row| row.3).collect()
}
fn own(letter: f64, word: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        letter_spacing: letter,
        word_spacing: word,
        ..Style::default()
    }
}
fn prepare(system: &mut System, source: &Source, base_style: Style) -> PreparedText {
    let extent = Range::new(0, source.text().len());
    let value = ParagraphInput {
        source: source.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: extent,
            aggregate: extent,
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, AnalysisTypography::Mixed).unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.direction(), Direction::LeftToRight);
    system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}
fn selected(prepared: &PreparedText, method: SpacingPolicy) -> SelectedLine {
    let mut value = request(prepared);
    value.context.orientation = LineOrientation::Vertical;
    value.context.base_direction = ParagraphBaseDirection::LeftToRight;
    value.context.line_box_direction = LineBoxDirection::LeftToRight;
    value.context.available_inline = None;
    value.context.alignment = crate::Alignment::Start;
    value.context.spacing = method;
    prepared.select(value).unwrap()
}
fn amounts(selected: &SelectedLine, values: &[f64]) -> SpacingAllocations {
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), values.len());
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip(values)
            .map(|(slot, &amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}
fn cells(prepared: &PreparedText, line: FinalizedLine, source: &Source, edges: &[f64]) {
    let total = *edges.last().unwrap();
    let layout = place_upright(prepared.clone(), line);
    assert_eq!(layout.bounds(), Rect::new(93.0, 20.0, 13.0, total));
    for (index, &edge) in edges.iter().enumerate() {
        let affinity = if index + 1 == edges.len() {
            Affinity::Before
        } else {
            Affinity::After
        };
        assert_eq!(
            layout.cursor(Cursor::new(index, affinity)).unwrap().rect(),
            Rect::new(93.0, 20.0 + edge, 13.0, 1.0)
        );
    }
    for (index, edge) in edges.windows(2).enumerate() {
        let geometry = layout
            .selection(Selection::new(
                Cursor::new(index, Affinity::After),
                Cursor::new(index + 1, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(geometry.rects().len(), 1);
        assert_eq!(
            geometry.rects()[0].rect(),
            Rect::new(93.0, 20.0 + edge[0], 13.0, edge[1] - edge[0])
        );
    }
    copy_source(&layout, source);
}
fn separator(selected: &SelectedLine, prepared: &PreparedText, extent: Range) {
    assert_eq!(selected.spacing_slots().len(), 1);
    let slot = selected.spacing_slots().next().unwrap();
    assert!(matches!(
        slot.target(),
        crate::SpacingSlotTarget::Separator(_)
    ));
    assert_eq!(
        slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
        [SpacingSlotOwner::Source {
            origin: prepared.origin_views().next().unwrap().handle(),
            local: extent,
            aggregate: extent,
        }]
    );
}

#[test]
fn mixed_authored_endpoint_means_and_ic_replacements_retain_raw_axes_and_shared_cells() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    assert_eq!(
        native(&bytes, "X", false, surgeist_shaper::script::LATIN, &[]),
        [(274, 0, 10.0, 0.0, 0.0)]
    );
    assert_eq!(
        native(&bytes, "一", true, surgeist_shaper::script::HAN, &[]),
        [(82, 0, 12.0, -6.0, 1.0)]
    );
    let mut system = system_with(&bytes);
    let ordinary_source = Source::new("X一X");
    let ordinary = prepare(&mut system, &ordinary_source, own(0.0, 0.0));
    let control = selected(&ordinary, SpacingPolicy::None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(control.feedback().advance, 32.0);
    let mut source = Source::identified("X一X", crate::Id::from_u64(1570060), 1);
    source.span(Range::new(1, 2), own(8.0, 0.0));
    let prepared = prepare(&mut system, &source, own(4.0, 0.0));
    let basis = selected(&prepared, SpacingPolicy::InterCharacter);
    let slots = basis.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    for (slot, full) in slots.iter().zip([(4.0, 8.0), (8.0, 4.0)]) {
        let crate::SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            panic!("real Mixed boundary")
        };
        assert_eq!(
            (
                visual_before.letter_spacing_after(),
                visual_after.letter_spacing_before()
            ),
            full
        );
    }
    let natural = basis.finalize(SpacingAllocations::None).unwrap();
    // Raw32 plus6+6. Outer4/2 halves are absent, including at the R edges.
    assert_eq!(natural.feedback().advance, 44.0);
    assert_eq!(offsets(&natural, &bytes), [0.0, 16.0, 34.0]);
    assert_eq!(raw(&natural, &bytes), raw(&control, &bytes));
    assert!(
        natural
            .spacing_slots()
            .all(|row| row.authored_letter_spacing == 6.0 && row.authored_word_spacing == 0.0)
    );
    let expanded = natural.with_spacing(amounts(&basis, &[2.0, 4.0])).unwrap();
    assert_eq!(expanded.feedback().advance, 50.0);
    assert_eq!(offsets(&expanded, &bytes), [0.0, 18.0, 40.0]);
    let replaced = expanded.with_spacing(amounts(&basis, &[1.0, 1.0])).unwrap();
    assert_eq!(replaced.feedback().advance, 46.0);
    assert_eq!(offsets(&replaced, &bytes), [0.0, 17.0, 36.0]);
    let restored = replaced.with_spacing(SpacingAllocations::None).unwrap();
    for line in [&expanded, &replaced, &restored] {
        assert_eq!(line.selected_line(), basis);
        assert_eq!(raw(line, &bytes), raw(&natural, &bytes));
    }
    assert_eq!(restored.feedback().advance, 44.0);
    assert_eq!(offsets(&natural, &bytes), [0.0, 16.0, 34.0]);
    cells(&prepared, natural, &source, &[0.0, 16.0, 34.0, 44.0]);
    cells(&prepared, expanded, &source, &[0.0, 18.0, 40.0, 50.0]);
    cells(&prepared, replaced, &source, &[0.0, 17.0, 36.0, 46.0]);
    cells(&prepared, restored, &source, &[0.0, 16.0, 34.0, 44.0]);
}

#[test]
fn mixed_word_and_letter_spacing_add_to_interword_halves_without_changing_provider_packets() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.hmtx().unwrap().h_metrics()[3].advance.get(), 1000);
    assert_eq!(font.vmtx().unwrap().v_metrics()[3].advance.get(), 1200);
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get(' '),
        VerticalOrientation::Rotated
    );
    assert_eq!(
        native(&bytes, " ", false, surgeist_shaper::script::LATIN, &[]),
        [(3, 0, 10.0, 0.0, 0.0)]
    );
    let source = Source::identified("X 一", crate::Id::from_u64(1570061), 1);
    let mut system = system_with(&bytes);
    let ordinary = prepare(&mut system, &source, own(0.0, 0.0));
    let control = selected(&ordinary, SpacingPolicy::None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(control.feedback().advance, 32.0);
    let prepared = prepare(&mut system, &source, own(4.0, 6.0));
    let basis = selected(&prepared, SpacingPolicy::InterWord);
    separator(&basis, &prepared, Range::new(1, 2));
    let natural = basis.finalize(SpacingAllocations::None).unwrap();
    // Raw32 + two tracking4 boundaries + word6; half3 shifts SPACE paint.
    assert_eq!(natural.feedback().advance, 46.0);
    assert_eq!(offsets(&natural, &bytes), [0.0, 17.0, 34.0]);
    assert_eq!(raw(&natural, &bytes), raw(&control, &bytes));
    for (amount, total, paints, edges) in [
        (6.0, 52.0, [0.0, 20.0, 40.0], [0.0, 14.0, 40.0, 52.0]),
        (2.0, 48.0, [0.0, 18.0, 36.0], [0.0, 14.0, 36.0, 48.0]),
    ] {
        let line = natural.with_spacing(amounts(&basis, &[amount])).unwrap();
        assert_eq!(line.selected_line(), basis);
        assert_eq!(line.feedback().advance, total);
        assert_eq!(offsets(&line, &bytes), paints);
        assert_eq!(raw(&line, &bytes), raw(&natural, &bytes));
        let row = line.spacing_slots().next().unwrap();
        assert_eq!(
            row.geometry,
            crate::SpacingSlotGeometry::Independent {
                intrinsic_advance: 10.0
            }
        );
        assert_eq!((row.authored_word_spacing, row.allocation), (6.0, amount));
        assert!(row.can_expand());
        assert_eq!(row.advance, edges[2] - edges[1]);
        let restored = line.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(restored.feedback().advance, 46.0);
        assert_eq!(offsets(&restored, &bytes), [0.0, 17.0, 34.0]);
        assert_eq!(raw(&restored, &bytes), raw(&natural, &bytes));
        cells(&prepared, line, &source, &edges);
    }
    assert_eq!(natural.feedback().advance, 46.0);
    cells(&prepared, natural, &source, &[0.0, 14.0, 34.0, 46.0]);
}

#[test]
fn mixed_real_soft_cut_suppresses_outer_tracking_and_replays_hanging_interword_space() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    let source = Source::identified("X一 X", crate::Id::from_u64(1570062), 1);
    let mut system = system_with(&bytes);
    let prepared = prepare(&mut system, &source, own(4.0, 6.0));
    let whole = selected(&prepared, SpacingPolicy::None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(whole.feedback().advance, 60.0);
    let offer = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().source_after().get() == 3
        })
        .unwrap()
        .handle();
    let mut first = request(&prepared);
    first.context.orientation = LineOrientation::Vertical;
    first.context.base_direction = ParagraphBaseDirection::LeftToRight;
    first.context.line_box_direction = LineBoxDirection::LeftToRight;
    first.context.end = LineEnd::Soft;
    first.context.available_inline = Some(60.0);
    first.context.alignment = crate::Alignment::Center;
    first.context.spacing = SpacingPolicy::InterWord;
    first.end = offer.view().boundary();
    first.participants = prepared
        .participants()
        .take(first.end.view().participant_offset())
        .collect();
    first.termination = PreparedTermination::Ordinary(offer.clone());
    let basis = prepared.select(first).unwrap();
    separator(&basis, &prepared, Range::new(2, 3));
    let natural = basis.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 46.0);
    assert_eq!(
        (
            natural.feedback().hanging_start,
            natural.feedback().hanging_end
        ),
        (0.0, 16.0)
    );
    assert_eq!(natural.measurements().alignment().advance(), 30.0);
    assert_eq!(offsets(&natural, &bytes), [15.0, 29.0, 48.0]);
    let expanded = natural.with_spacing(amounts(&basis, &[6.0])).unwrap();
    assert_eq!(expanded.feedback().advance, 52.0);
    assert_eq!(expanded.feedback().hanging_end, 22.0);
    assert_eq!(expanded.measurements().alignment().advance(), 30.0);
    assert_eq!(offsets(&expanded, &bytes), [15.0, 29.0, 51.0]);
    assert_eq!(raw(&expanded, &bytes), raw(&natural, &bytes));
    let restored = expanded.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.feedback().hanging_end, 16.0);
    assert_eq!(offsets(&restored, &bytes), [15.0, 29.0, 48.0]);
    let mut receiver = request(&prepared);
    receiver.start = offer.view().boundary();
    receiver.participants = prepared
        .participants()
        .skip(receiver.start.view().participant_offset())
        .collect();
    receiver.context.orientation = LineOrientation::Vertical;
    receiver.context.base_direction = ParagraphBaseDirection::LeftToRight;
    receiver.context.line_box_direction = LineBoxDirection::LeftToRight;
    receiver.context.start = LineStart::Continuation;
    receiver.context.available_inline = Some(60.0);
    receiver.context.alignment = crate::Alignment::Center;
    receiver.context.spacing = SpacingPolicy::None;
    let second = prepared
        .select(receiver)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(second.feedback().advance, 10.0);
    assert_eq!(offsets(&second, &bytes), [25.0]);
    // Exactly one4px boundary disappeared at the genuine cut:46+10 vs60.
    assert_eq!(natural.feedback().advance + second.feedback().advance, 56.0);
    for first in [natural, expanded, restored] {
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: prepared.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: [first, second.clone()]
                .into_iter()
                .enumerate()
                .map(|(index, line)| {
                    crate::PlacementEntry::Formatted(LinePlacement {
                        line,
                        anchor: LineAnchor {
                            inline_origin: 20.0,
                            baseline: 100.0 - 20.0 * index as f64,
                        },
                        orientation: LineOrientation::Vertical,
                        atomics: vec![],
                    })
                })
                .collect(),
        })
        .unwrap();
        copy_source(&layout, &source);
    }
}
