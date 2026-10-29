//! Genuine atomic run endpoints, transparency, selected cuts and native frames.
use super::*;
use crate::{
    Affinity, AtomicPlacement, FontBaselineScript, Hit, InlineBox, InlineBoxKind, Layout,
    ParagraphAnchor, ParagraphAnchorItem, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    Point, Rect, Size, SpacingSlotOwner, SpacingSlotTarget,
};

// IDs deliberately sort opposite to semantic X/Y order; source storage is Y/X.
const X: Id = Id::from_u64(70102);
const Y: Id = Id::from_u64(70101);
const OOF: Id = Id::from_u64(70103);

fn physical_size(orientation: LineOrientation) -> Size {
    if orientation == LineOrientation::Horizontal {
        Size::new(10.0, 6.0)
    } else {
        Size::new(6.0, 10.0)
    }
}

fn source(text: &str, orientation: LineOrientation, values: [f64; 4], transparent: bool) -> Source {
    let mut source = Source::identified(text, Id::from_u64(1566030), 1);
    if !text.is_empty() {
        source.span(
            Range::new(source.text().len() - 1, source.text().len()),
            own(values[3]),
        );
    }
    let seam = usize::from(!text.is_empty());
    // Store Y before X, then capture X before Y through genuine anchor order.
    for (id, at, value) in [
        (Y, seam + usize::from(transparent), values[2]),
        (X, seam, values[1]),
    ] {
        let box_ = InlineBox::new(id, InlineBoxKind::InFlow, at, physical_size(orientation));
        source.inline_box(if value == 0.0 {
            // Exercise the constructor's genuine resolved zero default.
            assert_eq!(box_.letter_spacing(), 0.0);
            box_
        } else {
            box_.try_with_letter_spacing(value).unwrap()
        });
    }
    if transparent {
        source.inline_box(
            InlineBox::new(OOF, InlineBoxKind::OutOfFlow, seam, Size::new(30.0, 40.0))
                .try_with_letter_spacing(100.0)
                .unwrap(),
        );
        source.span(Range::new(1, 2), own(100.0));
    }
    source
}

fn captured(
    bytes: &[u8],
    source: &Source,
    base_tracking: f64,
    orientation: LineOrientation,
    direction: ParagraphBaseDirection,
) -> PreparedText {
    let len = source.text().len();
    let order = if source.boxes().len() == 3 {
        vec![1, 2, 0]
    } else {
        vec![1, 0]
    };
    let value = ParagraphInput {
        source: source.clone(),
        base_style: own(base_tracking),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        origins: if len == 0 {
            vec![]
        } else {
            vec![crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: crate::ParagraphOriginKind::Authored,
            }]
        },
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        anchor_order: order
            .into_iter()
            .map(|index| ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(index),
                affinity: Affinity::Before,
            })
            .collect(),
        boundary_policies: (0..source.boxes().len())
            .flat_map(|index| {
                [
                    ParagraphBoundaryDefinition::AtomicBefore(index),
                    ParagraphBoundaryDefinition::AtomicAfter(index),
                ]
            })
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
    };
    let typography = if orientation == LineOrientation::Vertical {
        crate::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft,
        }
    } else {
        crate::AnalysisTypography::HorizontalComposition
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, typography).unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.source(), source);
    assert_eq!(analyzed.typography(), typography);
    assert_eq!(
        analyzed.direction(),
        if direction == ParagraphBaseDirection::LeftToRight {
            crate::Direction::LeftToRight
        } else {
            crate::Direction::RightToLeft
        }
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    if orientation == LineOrientation::Vertical {
        let font = skrifa::FontRef::new(bytes).unwrap();
        for tag in [b"BASE", b"vhea", b"vmtx"] {
            assert!(font.table_data(skrifa::Tag::new(tag)).is_none());
        }
        let face = system.registered_font_faces().next().unwrap().to_owned();
        let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
        for glyph in [35, 36] {
            let metric = instance
                .vertical_glyph_metrics(glyph, system.font_catalogue_identity())
                .unwrap();
            assert_eq!(
                (metric.advance(), metric.origin_x(), metric.origin_y()),
                (10.0, 5.0, 8.0)
            );
        }
        let metric = instance
            .upright_metrics(
                system.font_catalogue_identity(),
                FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            )
            .unwrap();
        assert_eq!((metric.used_over(), metric.used_under()), (5.0, -5.0));
    }
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), source);
    let actual = prepared
        .participants()
        .filter_map(|participant| match participant {
            PreparedParticipant::Atomic(atomic) => Some(atomic.view().original_box().id()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        if source.boxes().len() == 3 {
            vec![X, OOF, Y]
        } else {
            vec![X, Y]
        }
    );
    for view in prepared.atomic_views() {
        let original = view.original_box();
        let expected = source
            .boxes()
            .iter()
            .find(|item| item.id() == original.id())
            .unwrap();
        assert_eq!(original, expected);
        assert_eq!(original.letter_spacing(), expected.letter_spacing());
    }
    prepared
}

fn local_allocation() -> AtomicLineAllocation {
    AtomicLineAllocation::InFlow {
        flow: AtomicFlowAllocation {
            advance: 10.0,
            before_baseline: 2.0,
            after_baseline: 4.0,
            occupied_start: 0.0,
            occupied_end: 10.0,
        },
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -2.0, 10.0, 6.0),
            content: Rect::new(1.0, -1.0, 8.0, 4.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Baseline,
    }
}

fn select_range(
    prepared: &PreparedText,
    from: usize,
    to: usize,
    method: SpacingPolicy,
) -> SelectedLine {
    let mut value = request(prepared);
    value.start = prepared
        .boundary_views()
        .find(|view| view.participant_offset() == from)
        .unwrap()
        .handle();
    let offer = prepared
        .opportunity_views()
        .find(|view| {
            view.boundary().view().participant_offset() == to
                && matches!(
                    view.kind(),
                    PreparedBreakKind::Allowed | PreparedBreakKind::ParagraphEnd
                )
        })
        .unwrap()
        .handle();
    value.end = offer.view().boundary();
    value.participants = prepared.participants().skip(from).take(to - from).collect();
    value.termination = PreparedTermination::Ordinary(offer.clone());
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = if offer.view().kind() == PreparedBreakKind::ParagraphEnd {
        LineEnd::Paragraph
    } else {
        LineEnd::Soft
    };
    let rtl = prepared.analyzed_source().direction() == crate::Direction::RightToLeft;
    value.context.base_direction = if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    value.context.line_box_direction = if rtl {
        crate::LineBoxDirection::RightToLeft
    } else {
        crate::LineBoxDirection::LeftToRight
    };
    value.context.orientation = prepared.orientation();
    value.context.available_inline = None;
    value.context.spacing = method;
    value.atomics = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Atomic(occurrence) => Some(AtomicAllocation {
                occurrence: occurrence.clone(),
                allocation: if occurrence.view().original_box().kind() == InlineBoxKind::OutOfFlow {
                    AtomicLineAllocation::OutOfFlow
                } else {
                    local_allocation()
                },
            }),
            _ => None,
        })
        .collect();
    prepared.select(value).unwrap()
}

fn finish(prepared: &PreparedText, method: SpacingPolicy) -> FinalizedLine {
    let selected = select_range(prepared, 0, prepared.participants().len(), method);
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(line.selected_line(), selected);
    line
}

fn atomic(prepared: &PreparedText, id: Id) -> PreparedAtomic {
    prepared
        .atomic_views()
        .find(|view| view.original_box().id() == id)
        .unwrap()
        .handle()
}

fn positions(line: &FinalizedLine, expected: &[(Id, f64)]) {
    let slots = line.atomic_slots().collect::<Vec<_>>();
    for &(id, inline) in expected {
        let slot = slots
            .iter()
            .find(|slot| slot.occurrence.view().original_box().id() == id)
            .unwrap();
        assert_eq!(slot.allocation, local_allocation());
        assert_eq!(slot.origin, Point::new(inline, line.feedback().baseline));
    }
}

fn text_positions(line: &FinalizedLine) -> Vec<(Range, f64)> {
    let mut rows = line
        .run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .filter(|glyph| glyph.advance() != 0.0)
                .map(move |glyph| (glyph.range(), origin.x + glyph.pen()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0.start);
    rows
}

fn placed(prepared: &PreparedText, line: FinalizedLine, expected: &[(Id, f64)], total: f64) {
    let orientation = prepared.orientation();
    let horizontal = orientation == LineOrientation::Horizontal;
    let cross = if horizontal { 70.0 } else { 100.0 };
    let placements = expected
        .iter()
        .map(|&(id, inline)| AtomicPlacement {
            occurrence: atomic(prepared, id),
            border: if horizontal {
                Rect::new(20.0 + inline, 68.0, 10.0, 6.0)
            } else {
                Rect::new(96.0, 20.0 + inline, 6.0, 10.0)
            },
            content: if horizontal {
                Rect::new(21.0 + inline, 69.0, 8.0, 4.0)
            } else {
                Rect::new(97.0, 21.0 + inline, 4.0, 8.0)
            },
            baseline: cross,
        })
        .collect::<Vec<_>>();
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: cross,
            },
            orientation,
            atomics: placements.clone(),
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), prepared.analyzed_source().source());
    let expected_bounds = match orientation {
        LineOrientation::Horizontal => Rect::new(20.0, 62.0, total, 12.0),
        LineOrientation::SidewaysClockwise => Rect::new(96.0, 20.0, 12.0, total),
        LineOrientation::Vertical => Rect::new(95.0, 20.0, 10.0, total),
    };
    assert_eq!(layout.lines()[0].bounds(), expected_bounds);
    for placement in placements {
        let id = placement.occurrence.view().original_box().id();
        assert_eq!(
            layout
                .inline_boxes()
                .iter()
                .find(|item| item.id() == id)
                .unwrap()
                .rect(),
            placement.border
        );
        let center = Point::new(
            placement.border.origin.x + placement.border.size.width * 0.5,
            placement.border.origin.y + placement.border.size.height * 0.5,
        );
        assert_eq!(layout.hit(center), Hit::InlineBox(id));
    }
}

#[test]
fn unequal_atomic_run_endpoints_preserve_actual_boxes_and_native_frames() {
    font_premises(AHEM, false);
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
        LineOrientation::Vertical,
    ] {
        let control_source = source("AB", orientation, [0.0; 4], false);
        let control_prepared = captured(
            AHEM,
            &control_source,
            0.0,
            orientation,
            ParagraphBaseDirection::LeftToRight,
        );
        let control = finish(&control_prepared, SpacingPolicy::None);
        let expected_raw = if orientation == LineOrientation::Vertical {
            vec![
                (Range::new(0, 1), 35, -5.0, 8.0, 10.0),
                (Range::new(1, 2), 36, -5.0, 8.0, 10.0),
            ]
        } else {
            vec![
                (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                (Range::new(1, 2), 36, 0.0, 0.0, 10.0),
            ]
        };
        assert_eq!(raw(&control, AHEM), expected_raw);
        assert_eq!(control.feedback().advance, 40.0);
        positions(&control, &[(X, 10.0), (Y, 20.0)]);
        assert_eq!(
            text_positions(&control),
            [(Range::new(0, 1), 0.0), (Range::new(1, 2), 30.0)]
        );
        let default_source = source("AB", orientation, [4.0, 0.0, 0.0, 6.0], false);
        let source = source("AB", orientation, [4.0, 8.0, 12.0, 6.0], false);
        let prepared = captured(
            AHEM,
            &source,
            4.0,
            orientation,
            ParagraphBaseDirection::LeftToRight,
        );
        assert_ne!(atomic(&prepared, X), atomic(&prepared, Y));
        let line = finish(&prepared, SpacingPolicy::InterCharacter);
        assert_eq!(raw(&line, AHEM), expected_raw);
        // Actual two-atomic run contributes exterior means6 and9, no interior.
        assert_eq!(line.feedback().advance, 55.0);
        assert_eq!(line.measurements().alignment().advance(), 55.0);
        positions(&line, &[(X, 16.0), (Y, 26.0)]);
        assert_eq!(
            text_positions(&line),
            [(Range::new(0, 1), 0.0), (Range::new(1, 2), 45.0)]
        );
        let rows = line.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter()
                .map(|row| row.authored_letter_spacing)
                .collect::<Vec<_>>(),
            [6.0, 9.0]
        );
        placed(&prepared, line, &[(X, 16.0), (Y, 26.0)], 55.0);
        // Actual atomic zero means zero, rather than borrowing either neighbor.
        assert!(
            default_source
                .boxes()
                .iter()
                .all(|item| item.letter_spacing() == 0.0)
        );
        let default_prepared = captured(
            AHEM,
            &default_source,
            4.0,
            orientation,
            ParagraphBaseDirection::LeftToRight,
        );
        let default_line = finish(&default_prepared, SpacingPolicy::InterCharacter);
        assert_eq!(raw(&default_line, AHEM), expected_raw);
        assert_eq!(default_line.feedback().advance, 45.0);
        positions(&default_line, &[(X, 12.0), (Y, 22.0)]);
        assert_eq!(
            text_positions(&default_line),
            [(Range::new(0, 1), 0.0), (Range::new(1, 2), 35.0)]
        );
        assert_eq!(
            default_line
                .spacing_slots()
                .map(|row| row.authored_letter_spacing)
                .collect::<Vec<_>>(),
            [2.0, 3.0]
        );
    }
}

#[test]
fn source_free_atomic_run_and_genuine_selected_cuts_suppress_only_actual_outer_endpoints() {
    font_premises(AHEM, false);
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let source = source(
            "",
            LineOrientation::Horizontal,
            [4.0, 8.0, 12.0, 6.0],
            false,
        );
        let prepared = captured(AHEM, &source, 4.0, LineOrientation::Horizontal, direction);
        assert_eq!(prepared.origin_views().len(), 0);
        assert!(prepared.atomic_views().all(|view| view.level().get()
            == if direction == ParagraphBaseDirection::LeftToRight {
                0
            } else {
                1
            }));
        let line = finish(&prepared, SpacingPolicy::InterCharacter);
        assert!(raw(&line, AHEM).is_empty());
        assert!(line.has_in_flow_content());
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(line.spacing_slots().count(), 0);
        positions(
            &line,
            if direction == ParagraphBaseDirection::LeftToRight {
                &[(X, 0.0), (Y, 10.0)]
            } else {
                &[(Y, 0.0), (X, 10.0)]
            },
        );
    }
    let source = source(
        "AB",
        LineOrientation::Horizontal,
        [4.0, 8.0, 12.0, 6.0],
        false,
    );
    let prepared = captured(
        AHEM,
        &source,
        4.0,
        LineOrientation::Horizontal,
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(prepared.participants().len(), 4);
    for (from, to, total, boxes, text) in [
        (0, 2, 26.0, vec![(X, 16.0)], vec![(Range::new(0, 1), 0.0)]),
        (1, 3, 20.0, vec![(X, 0.0), (Y, 10.0)], vec![]),
        (
            1,
            4,
            39.0,
            vec![(X, 0.0), (Y, 10.0)],
            vec![(Range::new(1, 2), 29.0)],
        ),
        (2, 4, 29.0, vec![(Y, 0.0)], vec![(Range::new(1, 2), 19.0)]),
    ] {
        let selected = select_range(&prepared, from, to, SpacingPolicy::InterCharacter);
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(line.selected_line(), selected);
        assert_eq!(line.feedback().advance, total);
        positions(&line, &boxes);
        assert_eq!(text_positions(&line), text);
        assert_eq!(line.atomic_slots().count(), boxes.len());
    }
}

#[test]
fn invisible_format_and_out_of_flow_occurrence_do_not_split_the_in_flow_atomic_run() {
    font_premises(AHEM, false);
    assert_eq!(
        icu_properties::CodePointMapData::<icu_properties::props::GeneralCategory>::new()
            .get32(0x200b),
        icu_properties::props::GeneralCategory::Format
    );
    let control_source = source("A\u{200b}B", LineOrientation::Horizontal, [0.0; 4], true);
    let control_prepared = captured(
        AHEM,
        &control_source,
        0.0,
        LineOrientation::Horizontal,
        ParagraphBaseDirection::LeftToRight,
    );
    let control = finish(&control_prepared, SpacingPolicy::None);
    let control_raw = raw(&control, AHEM);
    assert_eq!(
        control_raw
            .iter()
            .copied()
            .filter(|glyph| glyph.4 != 0.0)
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
            (Range::new(2, 3), 36, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(control.feedback().advance, 40.0);
    positions(&control, &[(X, 10.0), (Y, 20.0)]);
    let source = source(
        "A\u{200b}B",
        LineOrientation::Horizontal,
        [4.0, 8.0, 12.0, 6.0],
        true,
    );
    let prepared = captured(
        AHEM,
        &source,
        4.0,
        LineOrientation::Horizontal,
        ParagraphBaseDirection::LeftToRight,
    );
    let line = finish(&prepared, SpacingPolicy::InterCharacter);
    assert_eq!(raw(&line, AHEM), control_raw);
    assert_eq!(line.feedback().advance, 55.0);
    positions(&line, &[(X, 16.0), (Y, 26.0)]);
    assert_eq!(
        text_positions(&line),
        [(Range::new(0, 1), 0.0), (Range::new(2, 3), 45.0)]
    );
    let selected = line.selected_line();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    let expected_members = [atomic(&prepared, X), atomic(&prepared, Y)];
    for (index, slot) in slots.iter().enumerate() {
        let SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = slot.target()
        else {
            panic!("real boundary target required")
        };
        let unit = if index == 0 {
            visual_after
        } else {
            visual_before
        };
        assert_eq!(
            (unit.letter_spacing_before(), unit.letter_spacing_after()),
            (8.0, 12.0)
        );
        assert_eq!(
            unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            expected_members
                .iter()
                .cloned()
                .map(SpacingSlotOwner::Atomic)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            unit.participants().cloned().collect::<Vec<_>>(),
            expected_members
                .iter()
                .cloned()
                .map(PreparedParticipant::Atomic)
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(
        line.atomic_slots()
            .find(|slot| slot.occurrence == atomic(&prepared, OOF))
            .unwrap()
            .allocation,
        AtomicLineAllocation::OutOfFlow
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

#[test]
fn genuine_hebrew_rtl_moves_each_atomic_with_its_own_exterior_values_in_clockwise_order() {
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (0x42, 36), (0x5d0, 35), (0x5d1, 36)],
    );
    font_premises(&bytes, false);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('\u{5d0}'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map('\u{5d1}'), Some(GlyphId::new(36)));
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        let control_source = source("\u{5d0}\u{5d1}", orientation, [0.0; 4], false);
        let control_prepared = captured(
            &bytes,
            &control_source,
            0.0,
            orientation,
            ParagraphBaseDirection::RightToLeft,
        );
        assert!(
            control_prepared
                .unit_views()
                .all(|view| view.level().get() == 1)
        );
        assert!(
            control_prepared
                .atomic_views()
                .all(|view| view.level().get() == 1)
        );
        let control = finish(&control_prepared, SpacingPolicy::None);
        let mut control_raw = raw(&control, &bytes);
        control_raw.sort_by_key(|glyph| glyph.0.start);
        assert_eq!(
            control_raw,
            [
                (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                (Range::new(1, 2), 36, 0.0, 0.0, 10.0)
            ]
        );
        assert_eq!(control.feedback().advance, 40.0);
        positions(&control, &[(Y, 10.0), (X, 20.0)]);
        assert_eq!(
            text_positions(&control),
            [(Range::new(0, 1), 30.0), (Range::new(1, 2), 0.0)]
        );
        let source = source("\u{5d0}\u{5d1}", orientation, [4.0, 8.0, 12.0, 6.0], false);
        let prepared = captured(
            &bytes,
            &source,
            4.0,
            orientation,
            ParagraphBaseDirection::RightToLeft,
        );
        let line = finish(&prepared, SpacingPolicy::InterCharacter);
        let mut actual_raw = raw(&line, &bytes);
        actual_raw.sort_by_key(|glyph| glyph.0.start);
        assert_eq!(actual_raw, control_raw);
        assert_eq!(line.feedback().advance, 55.0);
        positions(&line, &[(Y, 19.0), (X, 29.0)]);
        assert_eq!(
            text_positions(&line),
            [(Range::new(0, 1), 45.0), (Range::new(1, 2), 0.0)]
        );
        assert_eq!(
            line.spacing_slots()
                .map(|row| row.authored_letter_spacing)
                .collect::<Vec<_>>(),
            [9.0, 6.0]
        );
        placed(&prepared, line, &[(Y, 19.0), (X, 29.0)], 55.0);
    }
}
