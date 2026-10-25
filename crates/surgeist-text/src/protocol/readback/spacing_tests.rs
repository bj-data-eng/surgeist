//! Independent Ahem10 expectations over the actual native phase owners.
use crate::*;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn input(text: &str, mode: WhiteSpace) -> ParagraphInput {
    let source = Source::new(text);
    let length = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            white_space: mode,
            wrap: Wrap::Word,
            ..Style::default()
        },
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: if length == 0 {
            vec![]
        } else {
            vec![ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, length),
                aggregate: Range::new(0, length),
                kind: ParagraphOriginKind::Authored,
            }]
        },
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare(input: ParagraphInput) -> PreparedText {
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn select(prepared: &PreparedText, alternative: Option<PreparedAlternative>) -> SelectedLine {
    let termination = alternative
        .as_ref()
        .map(|row| row.view().opportunity().unwrap())
        .unwrap_or_else(|| {
            prepared
                .opportunity_views()
                .find(|row| row.kind() == PreparedBreakKind::ParagraphEnd)
                .unwrap()
                .handle()
        });
    let end = if alternative.is_some() {
        LineEnd::Soft
    } else {
        LineEnd::Paragraph
    };
    let atomics = prepared
        .atomic_views()
        .map(|atomic| AtomicAllocation {
            occurrence: atomic.handle(),
            allocation: match atomic.original_box().kind() {
                InlineBoxKind::OutOfFlow => AtomicLineAllocation::OutOfFlow,
                InlineBoxKind::InFlow => AtomicLineAllocation::InFlow {
                    flow: AtomicFlowAllocation {
                        advance: 5.0,
                        before_baseline: 8.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 5.0,
                    },
                    box_geometry: AtomicBoxAllocation {
                        border: Rect::new(0.0, -8.0, 5.0, 10.0),
                        content: Rect::new(0.0, -8.0, 5.0, 10.0),
                        baseline: 0.0,
                    },
                    alignment: LineBoxAlignment::Baseline,
                },
            },
        })
        .collect();
    let controls = prepared
        .control_views()
        .map(|row| {
            let occurrence = row.handle();
            ControlAllocation {
                boundary: matches!(
                    occurrence.view().kind(),
                    crate::ParagraphControlKind::InlineBoundaryStart
                        | crate::ParagraphControlKind::InlineBoundaryEnd
                )
                .then_some(crate::InlineBoundaryAllocation::ZERO),
                occurrence,
                strut: None,
            }
        })
        .collect();
    let tabs = if prepared
        .analyzed_source()
        .processed_source()
        .processed_text()
        .code_units()
        .contains(&9)
    {
        TabContext::FontAdvance
    } else {
        TabContext::NoTabs
    };
    prepared
        .select(LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            participants: prepared
                .participants()
                .take(termination.view().boundary().view().participant_offset())
                .collect(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative,
            atomics,
            controls,
            line_strut: None,
            context: LineContext {
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: crate::LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs,

                spacing: SpacingPolicy::Spaces,
            },
        })
        .unwrap()
}

fn allocations(selected: &SelectedLine, amounts: &[f64]) -> SpacingAllocations {
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(slots.len(), amounts.len());
    SpacingAllocations::Complete(
        slots
            .into_iter()
            .zip(amounts)
            .map(|(slot, &amount)| SpacingAllocation { slot, amount })
            .collect(),
    )
}

fn facts(line: &FinalizedLine) -> Vec<(SpacingSlotDisposition, f64)> {
    line.spacing_slots()
        .map(|row| (row.disposition, row.advance))
        .collect()
}

const EDGE: SpacingSlotDisposition = SpacingSlotDisposition::DiscardedAtLineEdge;
const INNER: SpacingSlotDisposition = SpacingSlotDisposition::Retained { trailing: false };
const TRAILING: SpacingSlotDisposition = SpacingSlotDisposition::Retained { trailing: true };

#[test]
fn collapsed_slots_keep_identity_and_each_actual_finalization_after_resources_drop() {
    // Lexical Collapse leaves one representative at each SPACE run; PhaseII
    // discards the outer two. Two X advances + the interior SPACE =30.
    let prepared = prepare(input("  X X  ", WhiteSpace::Collapse));
    let selected = select(&prepared, None);
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(
        slots
            .iter()
            .map(SpacingSlot::source_range)
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(3, 4), Range::new(5, 6)]
    );
    for slot in &slots {
        let SpacingSlotOwner::Unit(unit) = slot.owner() else {
            panic!("authored SPACE has its actual original unit")
        };
        assert_eq!(
            slot.participant(),
            Some(&PreparedParticipant::Text(unit.clone()))
        );
        assert_eq!(unit.view().source_range(), slot.source_range());
        assert_eq!(unit.prepared_text(), prepared);
    }
    let first = selected.finalize(SpacingAllocations::None).unwrap();
    let second = selected
        .finalize(allocations(&selected, &[0.0, 2.0, 0.0]))
        .unwrap();
    assert_eq!(facts(&first), [(EDGE, 0.0), (INNER, 10.0), (EDGE, 0.0)]);
    assert_eq!(facts(&second), [(EDGE, 0.0), (INNER, 12.0), (EDGE, 0.0)]);
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (30.0, 32.0)
    );
    assert_eq!(
        first
            .spacing_slots()
            .map(|row| row.slot)
            .collect::<Vec<_>>(),
        slots
    );
    assert_eq!(
        second
            .spacing_slots()
            .map(|row| row.slot)
            .collect::<Vec<_>>(),
        slots
    );
    let independently_selected = select(&prepared, None);
    let foreign = independently_selected.spacing_slots().next().unwrap();
    assert_ne!(foreign, slots[0]);
    let error = selected
        .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot: foreign,
            amount: 0.0,
        }]))
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::ForeignSelectedLine
        })
    );
    drop(independently_selected);
    drop(selected);
    drop(prepared);
    drop(slots);
    drop(first);
    assert_eq!(facts(&second), [(EDGE, 0.0), (INNER, 12.0), (EDGE, 0.0)]);
    assert_eq!(
        second.spacing_slots().nth(1).unwrap().slot.source_range(),
        Range::new(3, 4)
    );
    assert!(
        second
            .run_views()
            .all(|run| run.effective_instance().unwrap().face().bytes() == AHEM)
    );
    // Logical spacing never rewrites canonical provider advances.
    assert!(
        second
            .run_views()
            .all(|run| run.glyphs().all(|glyph| glyph.advance() == 10.0))
    );
}

#[test]
fn whitespace_only_discard_and_lexical_deletion_have_distinct_slot_inventories() {
    let collapsed = select(&prepare(input("   ", WhiteSpace::Collapse)), None);
    assert_eq!(collapsed.spacing_slots().len(), 1);
    let line = collapsed.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(facts(&line), [(EDGE, 0.0)]);
    assert_eq!(
        (line.feedback().advance, line.feedback().trailing_whitespace),
        (0.0, 0.0)
    );
    let error = collapsed
        .finalize(allocations(&collapsed, &[1.0]))
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::IneligibleSpacingSlot { slot: 0 }
        })
    );
    let discarded = select(&prepare(input("   ", WhiteSpace::Discard)), None);
    assert_eq!(discarded.spacing_slots().len(), 0);
    assert_eq!(
        discarded
            .finalize(SpacingAllocations::None)
            .unwrap()
            .spacing_slots()
            .len(),
        0
    );
}

#[test]
fn retained_zero_advance_preserves_trailing_classification_and_original_owner() {
    let prepared = prepare(input(" X ", WhiteSpace::Preserve));
    let selected = select(&prepared, None);
    let first = selected.finalize(SpacingAllocations::None).unwrap();
    let second = selected
        .finalize(allocations(&selected, &[0.0, -10.0]))
        .unwrap();
    assert_eq!(facts(&first), [(INNER, 10.0), (TRAILING, 10.0)]);
    assert_eq!(facts(&second), [(INNER, 10.0), (TRAILING, 0.0)]);
    assert_eq!(
        (
            first.feedback().advance,
            first.feedback().trailing_whitespace
        ),
        (30.0, 10.0)
    );
    assert_eq!(
        (
            second.feedback().advance,
            second.feedback().trailing_whitespace
        ),
        (20.0, 0.0)
    );
    assert_eq!(
        first.spacing_slots().nth(1).unwrap().slot,
        second.spacing_slots().nth(1).unwrap().slot
    );
}

#[test]
fn nbsp_and_real_retained_font_tab_do_not_acquire_space_slots() {
    let prepared = prepare(input("\u{a0}\tX", WhiteSpace::Preserve));
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [0xa0, 9, 0x58]
    );
    let selected = select(&prepared, None);
    assert_eq!(selected.spacing_slots().len(), 0);
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(line.spacing_slots().len(), 0);
    assert_eq!(line.feedback().advance, 30.0);
}

#[test]
fn neutral_controls_are_transparent_while_real_inflow_allocations_bound_spaces() {
    for inflow in [false, true] {
        let mut captured = input(" X ", WhiteSpace::Collapse);
        captured.controls = [
            (0, ParagraphControlKind::InlineBoundaryStart),
            (3, ParagraphControlKind::InlineBoundaryEnd),
        ]
        .map(|(at, kind)| ParagraphControl { at, kind })
        .to_vec();
        captured.anchor_order = vec![ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::Before,
        }];
        if inflow {
            for (index, at) in [0, 3].into_iter().enumerate() {
                captured.source.inline_box(InlineBox::new(
                    Id::from_u64(100 + index as u64),
                    InlineBoxKind::InFlow,
                    at,
                    Size::new(5.0, 10.0),
                ));
                captured.anchor_order.push(ParagraphAnchor {
                    item: ParagraphAnchorItem::Atomic(index),
                    affinity: Affinity::Before,
                });
                captured.boundary_policies.extend(
                    [
                        ParagraphBoundaryDefinition::AtomicBefore(index),
                        ParagraphBoundaryDefinition::AtomicAfter(index),
                    ]
                    .map(|definition| ParagraphBoundaryPolicy {
                        definition,
                        preceding_wrap: Wrap::Word,
                        following_wrap: Wrap::Word,
                        ancestor_wrap: Wrap::Word,
                    }),
                );
            }
        }
        captured.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(1),
            affinity: Affinity::Before,
        });
        let line = select(&prepare(captured), None)
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            facts(&line),
            if inflow {
                vec![(INNER, 10.0), (INNER, 10.0)]
            } else {
                vec![(EDGE, 0.0), (EDGE, 0.0)]
            }
        );
        assert_eq!(line.feedback().advance, if inflow { 40.0 } else { 10.0 });
        assert_eq!(line.feedback().trailing_whitespace, 0.0);
    }
    let mut captured = input("X ", WhiteSpace::Preserve);
    captured.controls.push(ParagraphControl {
        at: 2,
        kind: ParagraphControlKind::InlineBoundaryEnd,
    });
    captured.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::Before,
    });
    let line = select(&prepare(captured), None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(facts(&line), [(TRAILING, 10.0)]);
    assert_eq!(line.feedback().trailing_whitespace, 10.0);
}

#[test]
fn computed_replacement_spaces_keep_actual_alternative_and_empty_source_anchors() {
    let mut captured = input("XXXXX", WhiteSpace::Collapse);
    captured.controls.push(ParagraphControl {
        at: 2,
        kind: ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(captured.base_style.clone()),
        },
    });
    captured.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::Before,
    });
    captured.replacements.push(
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Control {
                control: 0,
                origin: None,
            },
            2,
            vec![Text::from(" X ")],
        )
        .unwrap(),
    );
    let prepared = prepare(captured);
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let selected = select(&prepared, Some(alternative.clone()));
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(slots.len(), 2);
    assert_ne!(slots[0], slots[1]);
    for slot in &slots {
        assert_eq!(
            slot.owner(),
            SpacingSlotOwner::Alternative(alternative.clone())
        );
        assert_eq!(slot.source_range(), Range::new(2, 2));
        assert!(slot.participant().is_none());
    }
    assert_eq!(
        slots
            .iter()
            .map(|slot| {
                let range = slot.presentation_range();
                (range.start(), range.end())
            })
            .collect::<Vec<_>>(),
        [(2, 3), (4, 5)]
    );
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(facts(&line), [(INNER, 10.0), (EDGE, 0.0)]);
    assert_eq!(line.feedback().advance, 40.0);
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [88, 88, 88, 88, 88]
    );
    assert_eq!(
        line.spacing_slots().map(|row| row.slot).collect::<Vec<_>>(),
        slots
    );
}

#[test]
fn preserved_trailing_space_is_unchanged_by_a_genuine_out_of_flow_occurrence() {
    for out_of_flow in [false, true] {
        let mut captured = input("X ", WhiteSpace::Preserve);
        if out_of_flow {
            captured.source.inline_box(InlineBox::new(
                Id::from_u64(200),
                InlineBoxKind::OutOfFlow,
                2,
                Size::new(4.0, 5.0),
            ));
            captured.anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: Affinity::Before,
            });
        }
        let prepared = prepare(captured);
        let selected = select(&prepared, None);
        for (amount, advance, trailing) in [(0.0, 20.0, 10.0), (3.0, 23.0, 13.0)] {
            let line = selected
                .finalize(allocations(&selected, &[amount]))
                .unwrap();
            assert_eq!(facts(&line), [(TRAILING, trailing)]);
            assert_eq!(
                (line.feedback().advance, line.feedback().trailing_whitespace),
                (advance, trailing)
            );
            assert_eq!(
                (line.feedback().baseline, line.feedback().line_height),
                (8.0, 10.0)
            );
            assert!(
                line.run_views()
                    .all(|run| run.glyphs().all(|glyph| glyph.advance() == 10.0))
            );
            if out_of_flow {
                let slot = line.atomic_slots().next().unwrap();
                assert_eq!(
                    slot.occurrence,
                    prepared.atomic_views().next().unwrap().handle()
                );
                assert_eq!(slot.origin, Point::new(advance, 8.0));
                assert_eq!(slot.allocation, AtomicLineAllocation::OutOfFlow);
            } else {
                assert_eq!(line.atomic_slots().len(), 0);
            }
        }
    }
}
