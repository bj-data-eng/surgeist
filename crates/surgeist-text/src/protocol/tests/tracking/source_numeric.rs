//! Public-path source custody and independently derived numeric qualification.
use super::fixtures::{allocations, select_method};
use super::*;
use crate::{
    Affinity, ErrorDetail, FinalizedFragmentOwner, HyphenationPlacement, Rect, SpacingSlotOwner,
    SpacingSlotTarget, TextNumericError, TextNumericPhase,
};

fn input(source: &Source, style: Style) -> ParagraphInput {
    let extent = Range::new(0, source.text().len());
    ParagraphInput {
        source: source.clone(),
        base_style: style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
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
    }
}

fn register(bytes: &[u8]) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
}

fn prepare_input(system: &mut System, value: ParagraphInput) -> crate::Result<PreparedText> {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)?,
        crate::Wrap::Word,
    );
    system.prepare(&analyzed, PreparationOptions::default())
}

fn place(prepared: &PreparedText, lines: Vec<FinalizedLine>) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0 + 10.0 * index as f64,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}

fn copied(layout: &crate::Layout, source: &Source) {
    assert_eq!(layout.source(), source);
    let positions = layout.positions().collect::<Vec<_>>();
    let selection = layout
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selection.source_text(), source.text());
    assert_eq!(selection.plaintext(), source.text());
}

fn numeric(error: &crate::Error, field: &'static str, phase: TextNumericPhase, value: f64) {
    assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
    assert!(
        matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
            field: actual_field,
            phase: actual_phase,
            category: TextNumericError::NonZeroLost,
            value: actual_value,
        }) if *actual_field == field && *actual_phase == phase && *actual_value == value),
        "{error:?}"
    );
}

fn endpoint_owners(slot: &SpacingSlot) -> (Vec<SpacingSlotOwner>, Vec<SpacingSlotOwner>) {
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slot.target()
    else {
        panic!("InterCharacter must issue actual visual boundary endpoints")
    };
    let before = visual_before
        .pieces()
        .map(|piece| piece.owner())
        .collect::<Vec<_>>();
    let after = visual_after
        .pieces()
        .map(|piece| piece.owner())
        .collect::<Vec<_>>();
    assert_eq!(
        slot.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
        before.iter().chain(&after).cloned().collect::<Vec<_>>()
    );
    (before, after)
}

fn source_owner(prepared: &PreparedText, range: Range) -> SpacingSlotOwner {
    SpacingSlotOwner::Source {
        origin: prepared.origin_views().next().unwrap().handle(),
        local: range,
        aggregate: range,
    }
}

fn indication_style(tracking: f64, placement: HyphenationPlacement) -> Style {
    Style {
        hyphens: crate::Hyphens::Manual,
        locale: Some("und-Latn".into()),
        native_hyphenation_convention: Some(
            crate::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                crate::Text::from("A"),
                placement,
            )
            .unwrap(),
        ),
        ..own(tracking)
    }
}

fn ending_request(
    prepared: &PreparedText,
    offer: PreparedOpportunity,
    chosen: PreparedAlternative,
) -> LineSelectionRequest {
    let mut value = request(prepared);
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::InterCharacter;
    value.context.end = LineEnd::Soft;
    value.end = offer.view().boundary();
    value.participants = prepared
        .participants()
        .take(value.end.view().participant_offset())
        .collect();
    value.termination = PreparedTermination::Ordinary(offer);
    value.alternative = Some(chosen);
    value
}

fn visible_paints(line: &FinalizedLine) -> Vec<(u32, f64)> {
    let mut result = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            if glyph.advance() == 0.0 {
                assert_eq!(glyph.range(), Range::new(2, 3));
                assert!(matches!(glyph.id(), 0 | 3));
                continue;
            }
            assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (0.0, 0.0, 10.0));
            result.push((glyph.id(), run.origin().x + glyph.pen()));
        }
    }
    result
}

#[test]
fn actual_before_after_and_both_indications_retain_selected_tracking_and_shy_query_custody() {
    super::fixtures::abc_premises();
    let font = skrifa::FontRef::new(AHEM).unwrap();
    for (ch, id) in [('D', 38), ('E', 39)] {
        assert_eq!(font.charmap().map(ch), Some(GlyphId::new(id)));
        assert_eq!(
            font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(id)),
            Some(1000.0)
        );
    }
    for mode in [
        HyphenationPlacement::LogicalBefore,
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        let before = mode != HyphenationPlacement::LogicalAfter;
        let after = mode != HyphenationPlacement::LogicalBefore;
        let mut system = register(AHEM);
        // Establish ordinary composition and authentic issued SHY topology first.
        let control_source = Source::new("AB\u{ad}CDE");
        let control = prepare_input(
            &mut system,
            input(&control_source, indication_style(0.0, mode)),
        )
        .unwrap();
        assert_eq!(
            visible_paints(&whole(&control)),
            [(35, 0.0), (36, 10.0), (37, 20.0), (38, 30.0), (39, 40.0)]
        );
        let mut source = Source::identified("AB\u{ad}CDE", Id::from_u64(1566060), 1);
        source.span(Range::new(2, 3), indication_style(8.0, mode));
        let prepared =
            prepare_input(&mut system, input(&source, indication_style(4.0, mode))).unwrap();
        let cut = prepared
            .hyphenation_cut_views()
            .find(|cut| cut.boundary().view().source_before().get() == 3)
            .unwrap();
        assert_eq!(
            cut.counts(),
            Some(crate::HyphenationCounts {
                total: 5,
                before: 2,
                after: 3
            })
        );
        let offer = cut
            .opportunity()
            .expect("the actual SHY counts admit this cut");
        let chosen = offer.view().alternatives().next().unwrap().handle();
        assert_eq!(chosen.view().text(), &crate::Text::from("A"));
        assert_eq!(chosen.view().source_range(), Some(Range::new(2, 3)));
        assert_eq!(chosen.view().insertion_anchor().get(), 3);
        let first_selected = prepared
            .select(ending_request(&prepared, offer.clone(), chosen.clone()))
            .unwrap();
        let first_natural = first_selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(
            first_natural.feedback().advance,
            if before { 40.0 } else { 24.0 }
        );
        let first = first_natural
            .with_spacing(allocations(
                &first_selected,
                &vec![2.0; if before { 2 } else { 1 }],
            ))
            .unwrap();
        assert_eq!(
            visible_paints(&first),
            if before {
                vec![(35, 0.0), (36, 16.0), (35, 34.0)]
            } else {
                vec![(35, 0.0), (36, 16.0)]
            }
        );
        assert_eq!(first.feedback().advance, if before { 44.0 } else { 26.0 });
        if before {
            let slot = first_selected.spacing_slots().last().unwrap();
            let (body, indication) = endpoint_owners(&slot);
            assert_eq!(body, [source_owner(&prepared, Range::new(1, 2))]);
            assert_eq!(
                indication,
                [SpacingSlotOwner::Alternative {
                    alternative: chosen.clone(),
                    local: Range::new(0, 1)
                }]
            );
            let SpacingSlotTarget::Boundary {
                visual_before,
                visual_after,
            } = slot.target()
            else {
                unreachable!()
            };
            assert_eq!(visual_before.letter_spacing_after(), 4.0);
            assert_eq!(visual_after.letter_spacing_before(), 8.0);
            assert_eq!(
                first
                    .spacing_slots()
                    .last()
                    .unwrap()
                    .authored_letter_spacing,
                6.0
            );
        }
        let mut receiver = request(&prepared);
        receiver.start = offer.view().boundary();
        receiver.participants = prepared
            .participants()
            .skip(receiver.start.view().participant_offset())
            .collect();
        receiver.incoming = first.hyphenation_continuation();
        receiver.context.start = LineStart::Continuation;
        receiver.context.available_inline = None;
        receiver.context.spacing = SpacingPolicy::InterCharacter;
        let second_selected = prepared.select(receiver).unwrap();
        let second_natural = second_selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(
            second_natural.feedback().advance,
            if after { 54.0 } else { 38.0 }
        );
        let second = second_natural
            .with_spacing(allocations(
                &second_selected,
                &vec![2.0; if after { 3 } else { 2 }],
            ))
            .unwrap();
        assert_eq!(
            visible_paints(&second),
            if after {
                vec![(35, 0.0), (37, 18.0), (38, 34.0), (39, 50.0)]
            } else {
                vec![(37, 0.0), (38, 16.0), (39, 32.0)]
            }
        );
        assert_eq!(second.feedback().advance, if after { 60.0 } else { 42.0 });
        if after {
            let slot = second_selected.spacing_slots().next().unwrap();
            let (indication, body) = endpoint_owners(&slot);
            assert_eq!(
                indication,
                [SpacingSlotOwner::Alternative {
                    alternative: chosen.clone(),
                    local: Range::new(0, 1)
                }]
            );
            assert_eq!(body, [source_owner(&prepared, Range::new(3, 4))]);
            assert_eq!(
                second
                    .spacing_slots()
                    .next()
                    .unwrap()
                    .authored_letter_spacing,
                6.0
            );
        }
        let origin = prepared.origin_views().next().unwrap().handle();
        let layout = place(&prepared, vec![first, second]);
        copied(&layout, &source);
        let query = layout
            .source_range_fragments(&origin, Range::new(2, 3))
            .unwrap();
        let units = query.units().collect::<Vec<_>>();
        assert_eq!(units.len(), usize::from(before) + usize::from(after));
        let mut expected = Vec::new();
        if before {
            expected.push(Rect::new(34.0, 0.0, 10.0, 10.0));
        }
        if after {
            expected.push(Rect::new(0.0, 10.0, 18.0, 10.0));
        }
        assert_eq!(
            units
                .iter()
                .flat_map(|u| u.pieces().map(|p| p.rect()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            expected
        );
        for unit in units {
            assert_eq!(
                unit.alternatives().collect::<Vec<_>>(),
                std::slice::from_ref(&chosen)
            );
            assert_eq!(
                unit.defining_sources()
                    .map(|s| s.aggregate_range())
                    .collect::<Vec<_>>(),
                [Range::new(2, 3)]
            );
        }
    }
}

#[test]
fn expanded_wbr_is_a_generated_endpoint_with_its_control_custody_and_no_authored_space() {
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (0x42, 36), (0x20, 58)],
    );
    font_premises(&bytes, false);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(' '), Some(GlyphId::new(58)));
    assert_eq!(
        font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(58)),
        Some(1000.0)
    );
    let source = Source::identified("AA", Id::from_u64(1566061), 1);
    let capture = |tracking| {
        let mut value = input(&source, own(tracking));
        value.controls.push(crate::ParagraphControl {
            at: 1,
            kind: crate::ParagraphControlKind::Wbr {
                style: Box::new(Style {
                    word_space_transform: crate::WordSpaceTransform::Space,
                    ..own(tracking)
                }),
            },
        });
        value.anchor_order.push(crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        });
        value
    };
    let mut system = register(&bytes);
    let ordinary = prepare_input(&mut system, capture(0.0)).unwrap();
    assert_eq!(whole(&ordinary).feedback().advance, 30.0);
    let prepared = prepare_input(&mut system, capture(4.0)).unwrap();
    let control = prepared.control_views().next().unwrap().handle();
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    assert_eq!(
        endpoint_owners(&slots[0]),
        (
            vec![source_owner(&prepared, Range::new(0, 1))],
            vec![SpacingSlotOwner::Control(control.clone())]
        )
    );
    assert_eq!(
        endpoint_owners(&slots[1]),
        (
            vec![SpacingSlotOwner::Control(control.clone())],
            vec![source_owner(&prepared, Range::new(1, 2))]
        )
    );
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 38.0);
    assert_eq!(
        paint_offsets(&natural),
        [(0.0, 0.0), (14.0, 0.0), (28.0, 0.0)]
    );
    let line = natural
        .with_spacing(allocations(&selected, &[2.0, 6.0]))
        .unwrap();
    assert_eq!(line.feedback().advance, 46.0);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (16.0, 0.0), (36.0, 0.0)]);
    assert_eq!(
        raw(&line, &bytes),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(1, 1), 58, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 35, 0.0, 0.0, 10.0)
        ]
    );
    let mapping = line
        .mapping_views()
        .find(|m| m.kind() == PresentationMappingKind::GeneratedSeparator)
        .unwrap();
    assert_eq!(mapping.source_range(), Range::new(1, 1));
    assert_eq!(mapping.separator_control(), Some(control.clone()));
    assert!(mapping.separator_virtual().is_none());
    let fragment = line
        .fragment_views()
        .find(|f| f.owner() == FinalizedFragmentOwner::Control(control.clone()))
        .unwrap();
    assert_eq!(fragment.bounds(), Some(Rect::new(16.0, 0.0, 20.0, 10.0)));
    let origin = prepared.origin_views().next().unwrap().handle();
    let layout = place(&prepared, vec![line]);
    copied(&layout, &source);
    let query = layout
        .source_range_fragments(&origin, Range::new(0, 2))
        .unwrap();
    assert!(
        query
            .units()
            .all(|u| u.controls().len() == 0 && u.virtual_separators().len() == 0)
    );
    assert_eq!(
        query
            .units()
            .flat_map(|u| u.pieces().map(|p| p.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            Rect::new(0.0, 0.0, 16.0, 10.0),
            Rect::new(36.0, 0.0, 10.0, 10.0)
        ]
    );
}

fn phrase_analysis(source: &Source, tracking: f64) -> AnalyzedSource {
    use crate::SourceProcessingEvent as E;
    let processed = std::sync::Arc::new(
        crate::process_contextual_source(crate::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            envelope: crate::SourceProcessingEnvelope::Complete,
            origins: input(source, own(0.0)).origins,
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![crate::SourceProcessingScope::Block {
                topology: crate::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: Style {
                    locale: Some("ja".into()),
                    word_space_transform: crate::WordSpaceTransform::SpaceAutoPhrase,
                    ..own(tracking)
                },
                used_direction: ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![crate::SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    )
}

#[test]
fn actual_phrase_separator_remains_a_virtual_endpoint_and_out_of_original_source_queries() {
    // Retained BudouX0.9.3 README: 今日は▁天気です。. The real captured
    // operation produces this seam; this test does not inject a phrase cut.
    let text = "今日は天気です。";
    let mut mappings = text
        .chars()
        .map(|ch| (u32::from(ch), 35))
        .collect::<Vec<_>>();
    mappings.extend([(0x42, 36), (0x20, 58)]);
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[82, 82], &mappings);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    for ch in text.chars() {
        assert_eq!(font.charmap().map(ch), Some(GlyphId::new(35)));
    }
    assert_eq!(font.charmap().map(' '), Some(GlyphId::new(58)));
    for glyph in [35, 58] {
        assert_eq!(
            font.glyph_metrics(FontSize::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(glyph)),
            Some(1000.0)
        );
    }
    let source = Source::identified(text, Id::from_u64(1566062), 1);
    let mut system = register(&bytes);
    let ordinary = system
        .prepare(
            &phrase_analysis(&source, 0.0),
            PreparationOptions::default(),
        )
        .unwrap();
    assert_eq!(ordinary.virtual_separators().count(), 1);
    assert_eq!(whole(&ordinary).feedback().advance, 90.0);
    let prepared = system
        .prepare(
            &phrase_analysis(&source, 4.0),
            PreparationOptions::default(),
        )
        .unwrap();
    assert!(prepared.captured_paragraph().is_some());
    let virtuals = prepared.virtual_separators().collect::<Vec<_>>();
    assert_eq!(virtuals.len(), 1);
    let owner = virtuals[0].clone();
    assert_eq!(owner.view().source_position().get(), 3);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 8);
    assert_eq!(
        endpoint_owners(&slots[2]),
        (
            vec![source_owner(&prepared, Range::new(2, 3))],
            vec![SpacingSlotOwner::VirtualSeparator(owner.clone())]
        )
    );
    assert_eq!(
        endpoint_owners(&slots[3]),
        (
            vec![SpacingSlotOwner::VirtualSeparator(owner.clone())],
            vec![source_owner(&prepared, Range::new(3, 4))]
        )
    );
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 122.0);
    assert_eq!(
        paint_offsets(&natural)
            .iter()
            .map(|p| p.0)
            .collect::<Vec<_>>(),
        [0.0, 14.0, 28.0, 42.0, 56.0, 70.0, 84.0, 98.0, 112.0]
    );
    let line = natural
        .with_spacing(allocations(
            &selected,
            &[0.0, 0.0, 2.0, 6.0, 0.0, 0.0, 0.0, 0.0],
        ))
        .unwrap();
    assert_eq!(line.feedback().advance, 130.0);
    assert_eq!(
        paint_offsets(&line).iter().map(|p| p.0).collect::<Vec<_>>(),
        [0.0, 14.0, 28.0, 44.0, 64.0, 78.0, 92.0, 106.0, 120.0]
    );
    assert_eq!(raw(&line, &bytes), raw(&natural, &bytes));
    let mapping = line
        .mapping_views()
        .find(|m| m.kind() == PresentationMappingKind::GeneratedSeparator)
        .unwrap();
    assert_eq!(mapping.source_range(), Range::new(3, 3));
    assert_eq!(mapping.separator_virtual(), Some(owner.clone()));
    assert!(mapping.separator_control().is_none());
    let fragment = line
        .fragment_views()
        .find(|f| f.owner() == FinalizedFragmentOwner::VirtualSeparator(owner.clone()))
        .unwrap();
    assert_eq!(fragment.bounds(), Some(Rect::new(44.0, 0.0, 20.0, 10.0)));
    assert!(fragment.origin().is_none());
    assert!(fragment.aggregate_range().is_none());
    let origin = prepared.origin_views().next().unwrap().handle();
    let layout = place(&prepared, vec![line]);
    copied(&layout, &source);
    let query = layout
        .source_range_fragments(&origin, Range::new(0, 8))
        .unwrap();
    assert_eq!(query.units().len(), 8);
    assert!(query.units().all(|u| u.virtual_separators().len() == 0));
}

#[test]
fn exact_opposite_subnormal_and_extreme_values_cancel_without_materializing_private_halves() {
    font_premises(AHEM, false);
    let mut system = register(AHEM);
    // Binary64 min-subnormal is2^-1074; MAX is(2-2^-52)*2^1023.
    // In both cases (x + -x)/2 is exactly zero, including negative x.
    for magnitude in [f64::from_bits(1), f64::from_bits(3), f64::MAX] {
        for left in [magnitude, -magnitude] {
            let mut source = Source::identified("AB", Id::from_u64(1566063), 1);
            source.span(Range::new(1, 2), own(-left));
            let prepared = prepare_input(&mut system, input(&source, own(left))).unwrap();
            let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
            let slot = selected.spacing_slots().next().unwrap();
            let SpacingSlotTarget::Boundary {
                visual_before,
                visual_after,
            } = slot.target()
            else {
                unreachable!()
            };
            assert_eq!(visual_before.letter_spacing_after(), left);
            assert_eq!(visual_after.letter_spacing_before(), -left);
            let line = selected.finalize(SpacingAllocations::None).unwrap();
            assert_eq!(
                line.spacing_slots().next().unwrap().authored_letter_spacing,
                0.0
            );
            assert_eq!(line.feedback().advance, 20.0);
            assert_eq!(paint_offsets(&line), [(0.0, 0.0), (10.0, 0.0)]);
            assert_eq!(
                raw(&line, AHEM),
                [
                    (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                    (Range::new(1, 2), 36, 0.0, 0.0, 10.0)
                ]
            );
        }
    }
}

#[test]
fn a_representable_subnormal_mean_reaches_actual_cell_loss_while_an_unrepresentable_mean_is_refused()
 {
    font_premises(AHEM, false);
    let mut system = register(AHEM);
    let source = Source::new("AB");
    let ordinary = prepare_input(&mut system, input(&source, own(0.0))).unwrap();
    let prior = whole(&ordinary);
    assert_eq!(prior.feedback().advance, 20.0);
    let least = f64::from_bits(1);
    for sign in [1.0, -1.0] {
        let amount = sign * least;
        // Equal endpoint values average to that same representable2^-1074;
        // separate halves would round to zero unnecessarily. The actual10px
        // cell cannot move by this amount, so reject that executed addition.
        let error = prepare_input(&mut system, input(&source, own(amount))).unwrap_err();
        numeric(
            &error,
            "character cell advance",
            TextNumericPhase::Shaping,
            amount,
        );
        let mut one_sided = source.clone();
        one_sided.span(Range::new(1, 2), own(0.0));
        // The true midpoint2^-1075 is not representable and rounds to zero.
        let error = prepare_input(&mut system, input(&one_sided, own(amount))).unwrap_err();
        numeric(
            &error,
            "character boundary mean",
            TextNumericPhase::Shaping,
            0.0,
        );
    }
    // Unequal subnormal pairs have independently representable means too.
    // (1+3)/2 =2 and (-1+3)/2 =1 in units of2^-1074; neither requires
    // materializing a private half. A midpoint exactly1.5 rounds to even2.
    for (left, right, mean) in [(1, 3, 2), (-1, 3, 1), (1, 2, 2)] {
        let unit_value = |units: i32| {
            let magnitude = f64::from_bits(u64::from(units.unsigned_abs()));
            if units < 0 { -magnitude } else { magnitude }
        };
        let mut varied = source.clone();
        varied.span(Range::new(1, 2), own(unit_value(right)));
        let error = prepare_input(&mut system, input(&varied, own(unit_value(left)))).unwrap_err();
        numeric(
            &error,
            "character cell advance",
            TextNumericPhase::Shaping,
            unit_value(mean),
        );
    }
    assert_eq!(prior.feedback().advance, 20.0);
    assert_eq!(paint_offsets(&prior), [(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(whole(&ordinary).feedback().advance, 20.0);
}

#[test]
fn selected_before_indication_loses_a_mean_only_during_retained_finalization() {
    super::fixtures::abc_premises();
    let font = skrifa::FontRef::new(AHEM).unwrap();
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for (scalar, id) in [('D', 38), ('E', 39)] {
        assert_eq!(font.charmap().map(scalar), Some(GlyphId::new(id)));
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
    }
    // A real complete provider receipt establishes residual SHY before any
    // authored geometry is tested; a zero-width glyph alone is not the oracle.
    let face = surgeist_shaper::FontRef::new(AHEM).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut native = surgeist_shaper::UnicodeBuffer::new();
    for (ordinal, scalar) in "AB\u{ad}CDE".chars().enumerate() {
        native.add(scalar, u32::try_from(ordinal).unwrap());
    }
    native.set_direction(surgeist_shaper::Direction::LeftToRight);
    native.set_script(surgeist_shaper::script::LATIN);
    let native = shaper.shape(native, &[]);
    assert_eq!(
        native.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    let receipt = native.default_ignorable_receipt().unwrap();
    assert_eq!(receipt.residual_input_positions(), [2]);
    assert!(receipt.deleted_input_positions().is_empty());

    let mode = HyphenationPlacement::LogicalBefore;
    let least = f64::from_bits(1);
    let mut system = register(AHEM);
    // First qualify the same public Before path with exact zero authored means.
    for tracking in [0.0, least] {
        let mut source = Source::identified("AB\u{ad}CDE", Id::from_u64(1566066), 1);
        source.span(Range::new(2, 3), indication_style(tracking, mode));
        let prepared =
            prepare_input(&mut system, input(&source, indication_style(0.0, mode))).unwrap();
        assert_eq!(prepared.analyzed_source().source(), &source);
        assert_eq!(
            prepared
                .unit_views()
                .find(|unit| unit.source_range() == Range::new(2, 3))
                .unwrap()
                .advance(),
            0.0
        );
        // Original receipt exclusion leaves only zero/zero body boundaries, so
        // preparation and a complete original line succeed even for SHY min.
        let prior = whole(&prepared);
        assert_eq!(prior.feedback().advance, 50.0);
        assert_eq!(
            visible_paints(&prior),
            [(35, 0.0), (36, 10.0), (37, 20.0), (38, 30.0), (39, 40.0)]
        );
        copied(&place(&prepared, vec![prior.clone()]), &source);
        let cut = prepared
            .hyphenation_cut_views()
            .find(|cut| cut.boundary().view().source_before().get() == 3)
            .unwrap();
        assert_eq!(
            cut.counts(),
            Some(crate::HyphenationCounts {
                total: 5,
                before: 2,
                after: 3,
            })
        );
        let offer = cut.opportunity().unwrap();
        let chosen = offer.view().alternatives().next().unwrap().handle();
        assert_eq!(chosen.view().text(), &crate::Text::from("A"));
        assert_eq!(chosen.view().source_range(), Some(Range::new(2, 3)));
        assert_eq!(chosen.view().insertion_anchor().get(), 3);
        let selected = prepared
            .select(ending_request(&prepared, offer.clone(), chosen.clone()))
            .unwrap();
        assert_eq!(selected.spacing_slots().len(), 2);
        let slot = selected.spacing_slots().last().unwrap();
        let (body, indication) = endpoint_owners(&slot);
        assert_eq!(body, [source_owner(&prepared, Range::new(1, 2))]);
        assert_eq!(
            indication,
            [SpacingSlotOwner::Alternative {
                alternative: chosen.clone(),
                local: Range::new(0, 1),
            }]
        );
        let SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            unreachable!()
        };
        assert_eq!(visual_before.letter_spacing_after(), 0.0);
        assert_eq!(visual_after.letter_spacing_before(), tracking);
        if tracking == 0.0 {
            let first = selected.finalize(SpacingAllocations::None).unwrap();
            assert_eq!(first.feedback().advance, 30.0);
            assert_eq!(visible_paints(&first), [(35, 0.0), (36, 10.0), (35, 20.0)]);
            let mut receiver = request(&prepared);
            receiver.start = offer.view().boundary();
            receiver.participants = prepared
                .participants()
                .skip(receiver.start.view().participant_offset())
                .collect();
            receiver.incoming = first.hyphenation_continuation();
            receiver.context.start = LineStart::Continuation;
            receiver.context.available_inline = None;
            receiver.context.spacing = SpacingPolicy::InterCharacter;
            let second = prepared
                .select(receiver)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            assert_eq!(second.feedback().advance, 30.0);
            let layout = place(&prepared, vec![first, second]);
            copied(&layout, &source);
            let origin = prepared.origin_views().next().unwrap().handle();
            let query = layout
                .source_range_fragments(&origin, Range::new(2, 3))
                .unwrap();
            let units = query.units().collect::<Vec<_>>();
            assert_eq!(units.len(), 1);
            assert_eq!(units[0].alternatives().collect::<Vec<_>>(), [chosen]);
            assert_eq!(
                units[0]
                    .defining_sources()
                    .map(|source| source.aggregate_range())
                    .collect::<Vec<_>>(),
                [Range::new(2, 3)]
            );
            assert_eq!(
                units[0]
                    .pieces()
                    .map(|piece| piece.rect())
                    .collect::<Vec<_>>(),
                [Rect::new(20.0, 0.0, 10.0, 10.0)]
            );
        } else {
            // Binary64 min is2^-1074. The selected B0/Amin midpoint is2^-1075,
            // exactly halfway to min and rounds to even zero: the error belongs
            // to this retained operation, after preparation and selection.
            let error = selected.finalize(SpacingAllocations::None).unwrap_err();
            numeric(
                &error,
                "character boundary mean",
                TextNumericPhase::Finalization,
                0.0,
            );
            assert_eq!(prior.feedback().advance, 50.0);
            assert_eq!(raw(&whole(&prepared), AHEM), raw(&prior, AHEM));
            assert_eq!(prepared.analyzed_source().source(), &source);
        }
    }
}

#[test]
fn same_sign_extreme_means_avoid_overflow_and_refuse_the_actual_following_ten_pixel_loss() {
    font_premises(AHEM, false);
    let mut system = register(AHEM);
    let source = Source::new("AB");
    // Each equal pair has an exactly representable mean equal to its endpoint;
    // directly summing either pair would overflow. The next raw Ahem advance
    // is10px and is below half an ULP at either resulting pen, so it must fail.
    for magnitude in [f64::from_bits(0x7fe0_0000_0000_0000), f64::MAX] {
        for tracking in [magnitude, -magnitude] {
            let prepared = prepare_input(&mut system, input(&source, own(tracking))).unwrap();
            let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
            assert_eq!(selected.spacing_slots().len(), 1);
            let error = selected.finalize(SpacingAllocations::None).unwrap_err();
            numeric(
                &error,
                "edge text end",
                TextNumericPhase::Finalization,
                10.0,
            );
        }
    }
}

#[test]
fn lost_actual_allocation_preserves_the_previous_line_and_allows_a_later_valid_retry() {
    font_premises(AHEM, false);
    let source = Source::identified("AB", Id::from_u64(1566064), 1);
    let prepared = prepare(
        AHEM,
        &source,
        own(0.0),
        &[Range::new(0, 1), Range::new(1, 2)],
    );
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let prior = selected.finalize(SpacingAllocations::None).unwrap();
    let least = f64::from_bits(1);
    let error = prior
        .with_spacing(allocations(&selected, &[least]))
        .unwrap_err();
    numeric(
        &error,
        "character cell advance",
        TextNumericPhase::Finalization,
        least,
    );
    assert_eq!(prior.selected_line(), selected);
    assert_eq!(prior.feedback().advance, 20.0);
    assert_eq!(prior.spacing_slots().next().unwrap().allocation, 0.0);
    assert_eq!(paint_offsets(&prior), [(0.0, 0.0), (10.0, 0.0)]);
    let retry = prior.with_spacing(allocations(&selected, &[2.0])).unwrap();
    assert_eq!(retry.selected_line(), selected);
    assert_eq!(retry.feedback().advance, 22.0);
    assert_eq!(paint_offsets(&retry), [(0.0, 0.0), (12.0, 0.0)]);
    assert_eq!(raw(&retry, AHEM), raw(&prior, AHEM));
    assert_eq!(prior.feedback().advance, 20.0);
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn a_real_tracking_displacement_lost_at_a_finite_physical_anchor_preserves_the_prior_placement() {
    font_premises(AHEM, false);
    let source = Source::identified("AB", Id::from_u64(1566065), 1);
    let units = [Range::new(0, 1), Range::new(1, 2)];
    let prepared = prepare(AHEM, &source, own(0.0625), &units);
    let line = whole(&prepared);
    assert_eq!(line.feedback().advance, 20.0625);
    assert_eq!(paint_offsets(&line), [(0.0, 0.0), (10.0625, 0.0)]);
    let placement = |anchor| PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: anchor,
                baseline: 70.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    };
    let retained = crate::Layout::from_finalized_lines(placement(20.0)).unwrap();
    let old_glyphs = retained.glyph_runs();
    let old_bounds = retained.bounds();
    let ordinary = prepare(AHEM, &source, own(0.0), &units);
    let ordinary_line = whole(&ordinary);
    // 2^50 has exponent bits1023+50. On its positive side the ULP is
    // 2^(50-52)=0.25; at -2^50+10 it is0.125. Tracking2^-4 is a quarter
    // positive ULP, or the negative endpoint's exact halfway case. That
    // endpoint has even grid index10/0.125=80, so ties-to-even leaves it
    // unchanged too. Raw10px and the real1px caret width survive both grids.
    let magnitude = f64::from_bits(0x4310_0000_0000_0000);
    for anchor in [magnitude, -magnitude] {
        assert_ne!(anchor + 10.0, anchor);
        assert_ne!(anchor + 1.0, anchor);
        assert_eq!(anchor + 10.0, anchor + 10.0625);
        let ordinary_placed = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: ordinary.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line: ordinary_line.clone(),
                anchor: LineAnchor {
                    inline_origin: anchor,
                    baseline: 70.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .expect("the ordinary raw geometry is admitted at this same physical anchor");
        assert_eq!(
            ordinary_placed
                .cursor(crate::Cursor::new(1, Affinity::After))
                .unwrap()
                .rect(),
            Rect::new(anchor + 10.0, 62.0, 1.0, 10.0)
        );
        let error = crate::Layout::from_finalized_lines(placement(anchor)).unwrap_err();
        // The shared diagnostic names the actual word or character displacement.
        numeric(
            &error,
            "placed spacing displacement",
            TextNumericPhase::Placement,
            0.0625,
        );
        assert_eq!(retained.glyph_runs(), old_glyphs);
        assert_eq!(retained.bounds(), old_bounds);
        assert_eq!(retained.source(), &source);
        assert_eq!(line.feedback().advance, 20.0625);
    }
    assert_eq!(
        crate::Layout::from_finalized_lines(placement(20.0))
            .unwrap()
            .bounds(),
        old_bounds
    );
}
