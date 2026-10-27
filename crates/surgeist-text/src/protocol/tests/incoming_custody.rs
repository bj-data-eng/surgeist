//! Receiver event custody is independent of visibility and local bidi parity.
use super::*;
use crate::ParagraphControlKind as K;
use crate::protocol::prepared::{ParticipantData, PresentationUnitKind as U};

fn fixture(
    source: Source,
    direction: ParagraphBaseDirection,
    kinds: Vec<(usize, K)>,
) -> PreparedText {
    fixture_with_style(
        source,
        direction,
        kinds,
        own_style("A", HyphenationPlacement::LogicalAfter),
        None,
    )
}
fn fixture_with_style(
    source: Source,
    direction: ParagraphBaseDirection,
    kinds: Vec<(usize, K)>,
    mut style: Style,
    anchor_order: Option<Vec<paragraph::ParagraphAnchor>>,
) -> PreparedText {
    let len = source.text().len();
    style.hyphenate_limit_chars = crate::HyphenateLimitChars {
        total: crate::HyphenationMinimum::Count(0),
        before: crate::HyphenationMinimum::Count(0),
        after: crate::HyphenationMinimum::Count(0),
    };
    let mut anchors: Vec<_> = (0..kinds.len())
        .map(|control| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(control),
            affinity: crate::Affinity::Before,
        })
        .collect();
    anchors.extend(
        (0..source.boxes().len()).map(|atomic| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Atomic(atomic),
            affinity: crate::Affinity::Before,
        }),
    );
    capture_input(
        ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: direction,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![paragraph::ParagraphOrigin {
                source: source.clone(),
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: paragraph::ParagraphOriginKind::Authored,
                text_node: None,
            }],
            connections: vec![],
            boundary_policies: (0..source.boxes().len())
                .flat_map(|atomic| {
                    [
                        paragraph::ParagraphBoundaryDefinition::AtomicBefore(atomic),
                        paragraph::ParagraphBoundaryDefinition::AtomicAfter(atomic),
                    ]
                })
                .map(|definition| paragraph::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            controls: kinds
                .into_iter()
                .map(|(at, kind)| paragraph::ParagraphControl { at, kind })
                .collect(),
            anchor_order: anchor_order.unwrap_or(anchors),
        },
        AHEM,
    )
}

// These fixtures affirmatively allocate each actual Start five pixels and all
// other endpoints zero; demanded facts are answered from that same fixture fact.
fn finish_at_edges(
    p: &PreparedText,
    mut request: LineSelectionRequest,
    direction: ParagraphBaseDirection,
) -> crate::Result<FinalizedLine> {
    request.context.base_direction = direction;
    request.context.line_box_direction = match direction {
        crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
    };
    let edge = |control: &PreparedControl| {
        crate::InlineBoundaryAllocation::try_new(
            if *control.view().kind() == K::InlineBoundaryStart {
                5.0
            } else {
                0.0
            },
        )
        .unwrap()
    };
    for control in &mut request.controls {
        if control.boundary.is_some() {
            control.boundary = Some(edge(&control.occurrence));
        }
    }
    while let Some(control) = p.boundary_context_demand(&request)? {
        request
            .boundary_context
            .push(crate::InlineBoundaryContextAllocation {
                boundary: edge(&control),
                occurrence: control,
            });
    }
    finish(p, request)
}

#[test]
fn incoming_uses_the_full_nested_receiver_level_and_preserves_prefix_controls() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let p = fixture(
            Source::new("AB\u{ad}CDE"),
            direction,
            vec![
                (0, K::RightToLeftEmbedding),
                (0, K::LeftToRightEmbedding),
                (3, K::InlineBoundaryStart),
                (3, K::InlineBoundaryStart),
            ],
        );
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish_at_edges(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
            direction,
        )
        .unwrap();
        let line = finish_at_edges(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                end(&p),
                first.hyphenation_continuation(),
                false,
            ),
            direction,
        )
        .unwrap();
        assert_eq!(line.feedback().advance, 50.0);
        let selected = &line.inner.selected;
        let units = &selected.presentation.units;
        let domain = &selected.presentation.domains[0];
        // LTR: RLE opens1 then LRE opens2; RTL: RLE opens3 then LRE opens4.
        // Latin C has even implicit level2 or4, while the local A stays L0.
        assert_eq!(
            domain.outer,
            if direction == ParagraphBaseDirection::LeftToRight {
                2
            } else {
                4
            }
        );
        assert_eq!((domain.base, units[domain.units.start].level), (0, 0));
        assert_eq!(domain.units, 2..3);
        let super::super::super::prepared::presentation::Attachment::After {
            participant,
            atom: Some(atom),
        } = domain.attachment
        else {
            panic!("actual nested original receiver required")
        };
        assert!(
            matches!(selected.prepared.tables.participants[participant], ParticipantData::Text(unit) if selected.prepared.tables.units[unit].atoms.contains(&atom))
        );
        assert_eq!(
            units
                .iter()
                .filter_map(|u| match u.kind {
                    U::Control { occurrence } => Some(occurrence),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [2, 3]
        );
        assert!(line.run_views().all(|run| run.face().bytes() == AHEM));
    }
}

#[test]
fn incoming_is_inserted_at_real_formatting_and_deleted_receivers() {
    for deleted in [false, true] {
        let p = fixture(
            Source::new(if deleted {
                "AB\u{ad}\u{ad}CDE"
            } else {
                "AB\u{ad}\u{200f}CDE"
            }),
            ParagraphBaseDirection::LeftToRight,
            vec![(3, K::InlineBoundaryStart)],
        );
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish_at_edges(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
            ParagraphBaseDirection::LeftToRight,
        )
        .unwrap();
        let next = if deleted {
            cut(&p, 4, counts(2, 3))
        } else {
            end(&p)
        };
        let line = finish_at_edges(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                next,
                first.hyphenation_continuation(),
                deleted,
            ),
            ParagraphBaseDirection::LeftToRight,
        )
        .unwrap();
        let selected = &line.inner.selected;
        let units = &selected.presentation.units;
        assert!(matches!(units[0].kind, U::Control { occurrence: 0 }));
        assert!(matches!(units[1].kind, U::Replacement { .. }));
        assert_eq!(units[1].source, Range::new(3, 3));
        assert_eq!(units[2].source, Range::new(3, 4));
        assert_eq!(units[2].kind, if deleted { U::Deleted } else { U::Text });
        assert_eq!(
            selected.presentation.domains[0].outer,
            if deleted { 0 } else { 1 }
        );
        assert_eq!(line.feedback().advance, if deleted { 15.0 } else { 45.0 });
        assert_eq!(
            p.analyzed_source().source().text().code_units()[3],
            if deleted { 0xad } else { 0x200f }
        );
    }
}

#[test]
fn incoming_missing_receiver_keeps_precise_refusal() {
    let p = fixture(
        Source::new("AB\u{ad}"),
        ParagraphBaseDirection::LeftToRight,
        vec![(3, K::InlineBoundaryStart), (3, K::InlineBoundaryEnd)],
    );
    let offer = cut(&p, 3, counts(2, 0));
    let first = finish_at_edges(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
        ParagraphBaseDirection::LeftToRight,
    )
    .unwrap();
    let error = finish_at_edges(
        &p,
        selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        ),
        ParagraphBaseDirection::LeftToRight,
    )
    .unwrap_err();
    hyphenation_reason(&error, HyphenationError::MissingReceivingBidiAuthority);
}

#[test]
fn a_source_free_formatting_receiver_precedes_the_first_visible_original() {
    let direction = ParagraphBaseDirection::LeftToRight;
    let p = fixture(
        Source::new("AB\u{ad}CDE"),
        direction,
        vec![
            (3, K::InlineBoundaryStart),
            (3, K::RightToLeftEmbedding),
            (3, K::InlineBoundaryStart),
        ],
    );
    let offer = cut(&p, 3, counts(2, 3));
    let first = finish_at_edges(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
        direction,
    )
    .unwrap();
    let line = finish_at_edges(
        &p,
        selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        ),
        direction,
    )
    .unwrap();
    assert_eq!(line.feedback().advance, 50.0);
    let selected = &line.inner.selected;
    let units = &selected.presentation.units;
    assert!(matches!(units[0].kind, U::Control { occurrence: 0 }));
    assert!(matches!(units[1].kind, U::Replacement { .. }));
    assert!(matches!(units[2].kind, U::Control { occurrence: 1 }));
    assert!(matches!(units[3].kind, U::Control { occurrence: 2 }));
    assert_eq!(units[4].source, Range::new(3, 4));
    let domain = &selected.presentation.domains[0];
    assert_eq!(
        (domain.units.clone(), domain.outer, domain.base),
        (1..2, 0, 0)
    );
    let super::super::super::prepared::presentation::Attachment::After {
        participant,
        atom: None,
    } = domain.attachment
    else {
        panic!("source-free RLE receiver required")
    };
    assert!(
        matches!(selected.prepared.tables.participants[participant],ParticipantData::Control(control) if selected.prepared.tables.controls[control].occurrence==1)
    );
}

fn atomic_prepared(
    indication: &str,
    placement: HyphenationPlacement,
    size: crate::Size,
    kinds: Vec<(usize, K)>,
    anchor_order: Option<Vec<paragraph::ParagraphAnchor>>,
) -> PreparedText {
    let mut source = Source::identified("AB\u{ad}CDE", crate::Id::from_u64(150300), 1);
    source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(91224),
        crate::InlineBoxKind::InFlow,
        3,
        size,
    ));
    fixture_with_style(
        source,
        ParagraphBaseDirection::LeftToRight,
        kinds,
        own_style(indication, placement),
        anchor_order,
    )
}
fn atomic_offer(p: &PreparedText) -> PreparedOpportunity {
    let cut = p
        .hyphenation_cut_views()
        .find(|cut| {
            cut.provenance() == crate::HyphenationProvenance::Source
                && cut.boundary().view().source_before().get() == 3
        })
        .unwrap();
    // A real in-flow atomic ends the preceding lexical word even at zero size.
    assert_eq!(cut.counts(), Some(counts(2, 0)));
    let offer = cut.opportunity().unwrap();
    assert_eq!(offer.view().kind(), PreparedBreakKind::Allowed);
    assert_eq!(offer.view().alternatives().count(), 1);
    assert!(
        matches!(alternative(&offer).view().definition(),crate::PreparedAlternativeDefinition::Source{source,..} if source==Range::new(2,3))
    );
    offer
}
fn with_atomic(p: &PreparedText, mut request: LineSelectionRequest) -> LineSelectionRequest {
    let occurrence = p.atomic_views().next().unwrap().handle();
    let size = occurrence.view().original_box().size();
    let before = if size.height == 0.0 { 0.0 } else { 8.0 };
    assert_eq!(
        request
            .participants
            .iter()
            .filter(|part| matches!(part, PreparedParticipant::Atomic(_)))
            .count(),
        1
    );
    assert!(
        request
            .participants
            .contains(&PreparedParticipant::Atomic(occurrence.clone()))
    );
    request.atomics = vec![AtomicAllocation {
        occurrence,
        allocation: AtomicLineAllocation::InFlow {
            flow: AtomicFlowAllocation {
                advance: size.width,
                before_baseline: before,
                after_baseline: size.height - before,
                occupied_start: 0.0,
                occupied_end: size.width,
            },
            box_geometry: AtomicBoxAllocation {
                border: crate::Rect::new(0.0, -before, size.width, size.height),
                content: crate::Rect::new(0.0, -before, size.width, size.height),
                baseline: 0.0,
            },
            alignment: LineBoxAlignment::Baseline,
        },
    }];
    request
}
fn atomic_pair(
    p: &PreparedText,
    indication: &str,
    placement: HyphenationPlacement,
) -> (FinalizedLine, FinalizedLine) {
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 66, 173, 67, 68, 69]
    );
    assert_eq!(p.atomic_views().count(), 1);
    let view = p.atomic_views().next().unwrap();
    assert_eq!(view.original_box().id(), crate::Id::from_u64(91224));
    assert_eq!(view.original_box().kind(), crate::InlineBoxKind::InFlow);
    assert_eq!(view.source_position().get(), 3);
    let offer = atomic_offer(p);
    let chosen = alternative(&offer);
    assert_eq!(
        chosen.view().text().code_units(),
        Text::from(indication).code_units()
    );
    let first = finish_at_edges(
        p,
        selected_request(p, start(p), offer.clone(), None, true),
        ParagraphBaseDirection::LeftToRight,
    )
    .unwrap();
    let before = placement == HyphenationPlacement::BothSides && !indication.is_empty();
    assert_eq!(first.feedback().advance, if before { 30.0 } else { 20.0 });
    assert_eq!(first.atomic_slots().count(), 0);
    assert_eq!(
        first.selected_line().request().alternative,
        Some(chosen.clone())
    );
    let continuation = first.hyphenation_continuation();
    if indication.is_empty() {
        assert!(continuation.is_none());
    } else {
        assert_eq!(continuation.as_ref().unwrap().alternative(), chosen);
        assert_eq!(
            continuation.as_ref().unwrap().boundary(),
            offer.view().boundary()
        );
    }
    let request = with_atomic(
        p,
        selected_request(
            p,
            offer.view().boundary(),
            end(p),
            continuation.clone(),
            false,
        ),
    );
    assert_eq!(request.incoming, continuation);
    assert_eq!(request.atomics.len(), 1);
    let second = finish_at_edges(p, request, ParagraphBaseDirection::LeftToRight)
        .expect("genuine in-flow atomic must accept the issued After/Both receiving indication");
    assert_eq!(second.prepared_text(), *p);
    assert!(second.hyphenation_continuation().is_none());
    assert!(second.selected_line().request().alternative.is_none());
    assert_eq!(
        second.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    (first, second)
}
fn place_atomic_pair(
    p: &PreparedText,
    first: FinalizedLine,
    second: FinalizedLine,
    x: f64,
) -> crate::Layout {
    let atomic = p.atomic_views().next().unwrap().handle();
    let size = atomic.view().original_box().size();
    let before = if size.height == 0.0 { 0.0 } else { 8.0 };
    let slots = second.atomic_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].occurrence, atomic);
    // Slot origin retains the actual line-local Ahem baseline8; the later
    // LineAnchor maps that frame to the independently supplied baseline18.
    assert_eq!(slots[0].origin, crate::Point::new(x, 8.0));
    assert_eq!(second.selected_line().request().atomics.len(), 1);
    assert_eq!(
        slots[0].allocation,
        second.selected_line().request().atomics[0].allocation
    );
    let rect = crate::Rect::new(x, 18.0 - before, size.width, size.height);
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![
            crate::PlacementEntry::Formatted(LinePlacement {
                line: first,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }),
            crate::PlacementEntry::Formatted(LinePlacement {
                line: second,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 18.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![AtomicPlacement {
                    occurrence: atomic,
                    border: rect,
                    content: rect,
                    baseline: 18.0,
                }],
            }),
        ],
    })
    .unwrap();
    let boxes = layout.inline_boxes();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].id(), crate::Id::from_u64(91224));
    assert_eq!(boxes[0].kind(), crate::InlineBoxKind::InFlow);
    assert_eq!(boxes[0].index(), 3);
    assert_eq!(boxes[0].line(), 1);
    assert_eq!(boxes[0].rect(), rect);
    assert_eq!(
        layout.source().text().code_units(),
        [65, 66, 173, 67, 68, 69]
    );
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(3, 6)]
    );
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.baseline())
            .collect::<Vec<_>>(),
        [8.0, 18.0]
    );
    layout
}
fn simple_atomic_receiver(
    indication: &str,
    placement: HyphenationPlacement,
    size: crate::Size,
    start_edge: bool,
) {
    let p = atomic_prepared(
        indication,
        placement,
        size,
        if start_edge {
            vec![(3, K::InlineBoundaryStart)]
        } else {
            vec![]
        },
        None,
    );
    let (first, second) = atomic_pair(&p, indication, placement);
    let emission = if indication.is_empty() { 0.0 } else { 10.0 };
    let edge = if start_edge { 5.0 } else { 0.0 };
    assert_eq!(
        second.feedback().advance,
        edge + emission + size.width + 30.0
    );
    for line in [&first, &second] {
        assert_eq!(line.feedback().ascent, 8.0);
        assert_eq!(line.feedback().descent, 2.0);
        assert_eq!(line.feedback().line_height, 10.0);
        assert_eq!(line.feedback().baseline, 8.0);
    }
    let mut first_ids = vec![35, 36];
    if placement == HyphenationPlacement::BothSides && !indication.is_empty() {
        first_ids.push(35);
    }
    assert_eq!(
        glyphs(&first, AHEM)
            .iter()
            .map(|g| (g.id, g.advance, g.x, g.y))
            .collect::<Vec<_>>(),
        first_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, 10.0, 10.0 * i as f64, 0.0))
            .collect::<Vec<_>>()
    );
    let mut expected = vec![];
    if !indication.is_empty() {
        expected.push((35, 10.0, edge, 0.0));
    }
    expected.extend([
        (37, 10.0, edge + emission + size.width, 0.0),
        (38, 10.0, edge + emission + size.width + 10.0, 0.0),
        (39, 10.0, edge + emission + size.width + 20.0, 0.0),
    ]);
    assert_eq!(
        glyphs(&second, AHEM)
            .iter()
            .map(|g| (g.id, g.advance, g.x, g.y))
            .collect::<Vec<_>>(),
        expected
    );
    let source_ranges = second
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .filter(|glyph| glyph.advance() != 0.0)
                .map(|glyph| glyph.range())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut ranges = vec![];
    if !indication.is_empty() {
        ranges.push(Range::new(3, 3));
    }
    ranges.extend([Range::new(3, 4), Range::new(4, 5), Range::new(5, 6)]);
    assert_eq!(source_ranges, ranges);
    let actual = second
        .inner
        .selected
        .presentation
        .units
        .iter()
        .find(|unit| matches!(unit.kind, U::Atomic { .. }))
        .unwrap();
    assert_eq!(actual.line_level, 0);
    if !indication.is_empty() {
        let domain = &second.inner.selected.presentation.domains[0];
        assert_eq!((domain.outer, domain.base), (0, 0));
        let super::super::super::prepared::presentation::Attachment::After {
            participant,
            atom: None,
        } = domain.attachment
        else {
            panic!("actual atomic attachment required")
        };
        assert!(matches!(
            second.inner.selected.prepared.tables.participants[participant],
            ParticipantData::Atomic(_)
        ));
    } else {
        assert!(second.inner.selected.presentation.domains.is_empty());
    }
    let first_snapshot = first.clone();
    let second_snapshot = second.clone();
    let layout = place_atomic_pair(&p, first, second, edge + emission);
    assert_eq!(
        layout.lines()[0].bounds(),
        crate::Rect::new(0.0, 0.0, 100.0, 10.0)
    );
    assert_eq!(
        layout.lines()[1].bounds(),
        crate::Rect::new(0.0, 10.0, 100.0, 10.0)
    );
    assert_eq!(first_snapshot.prepared_text(), p);
    assert_eq!(second_snapshot.prepared_text(), p);
}
#[test]
fn incoming_after_inflow_atomic_preserves_real_glyph_allocation_and_placement() {
    simple_atomic_receiver(
        "A",
        HyphenationPlacement::LogicalAfter,
        crate::Size::new(10.0, 10.0),
        false,
    );
}
#[test]
fn incoming_both_sides_inflow_atomic_preserves_one_choice_and_both_emissions() {
    simple_atomic_receiver(
        "A",
        HyphenationPlacement::BothSides,
        crate::Size::new(10.0, 10.0),
        false,
    );
}
#[test]
fn incoming_atomic_empty_indication_keeps_real_zero_extent_and_ending_controls() {
    // Empty used strings keep the selected ending but produce no continuation.
    // These controls execute before the nonempty zero-extent extension witness.
    for placement in [
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        simple_atomic_receiver("", placement, crate::Size::new(10.0, 10.0), false);
        simple_atomic_receiver("", placement, crate::Size::new(0.0, 0.0), false);
    }
}
#[test]
fn incoming_atomic_real_zero_extent_accepts_nonempty_after_and_both_indications() {
    for placement in [
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        simple_atomic_receiver("A", placement, crate::Size::new(0.0, 0.0), false);
    }
}
#[test]
fn incoming_atomic_preserves_positive_prefix_control_before_indication() {
    simple_atomic_receiver(
        "A",
        HyphenationPlacement::LogicalAfter,
        crate::Size::new(10.0, 10.0),
        true,
    );
}

#[test]
fn incoming_atomic_uses_actual_original_and_receiving_l1_not_paragraph_or_later_text() {
    use paragraph::{ParagraphAnchor, ParagraphAnchorItem as I};
    // Ltr paragraph0; actual RLO at0 gives the atomic override level1. At the
    // shared cut the atomic PRECEDES LRE, whose later Latin text is level2.
    // Original-only L1 keeps this atomic Other; local A under odd base has2.
    let p = atomic_prepared(
        "A",
        HyphenationPlacement::LogicalAfter,
        crate::Size::new(10.0, 10.0),
        vec![(0, K::RightToLeftOverride), (3, K::LeftToRightEmbedding)],
        Some(vec![
            ParagraphAnchor {
                item: I::Control(0),
                affinity: crate::Affinity::Before,
            },
            ParagraphAnchor {
                item: I::Atomic(0),
                affinity: crate::Affinity::Before,
            },
            ParagraphAnchor {
                item: I::Control(1),
                affinity: crate::Affinity::Before,
            },
        ]),
    );
    assert_eq!(p.atomic_views().next().unwrap().level().get(), 1);
    let (first, second) = atomic_pair(&p, "A", HyphenationPlacement::LogicalAfter);
    assert_eq!(second.feedback().advance, 50.0);
    let selected = &second.inner.selected;
    let units = &selected.presentation.units;
    assert!(matches!(units[0].kind, U::Replacement { .. }));
    assert!(matches!(units[1].kind, U::Atomic { occurrence: 0 }));
    assert!(matches!(units[2].kind, U::Control { occurrence: 1 }));
    assert_eq!(units[1].line_level, 1);
    let c = units
        .iter()
        .find(|unit| unit.kind == U::Text && unit.source == Range::new(3, 4))
        .unwrap();
    assert_eq!(c.line_level, 2);
    let domain = &selected.presentation.domains[0];
    assert_eq!((domain.outer, domain.base), (1, 1));
    assert_eq!(units[domain.units.start].level, 2);
    let super::super::super::prepared::presentation::Attachment::After {
        participant,
        atom: None,
    } = domain.attachment
    else {
        panic!("atomic After authority required")
    };
    assert!(matches!(
        selected.prepared.tables.participants[participant],
        ParticipantData::Atomic(_)
    ));
    // L2 reverses the original outer1 span: CDE(level2), atomic1, whole
    // indication domain1. Local Latin A is not treated as a later-text seed.
    assert_eq!(
        glyphs(&second, AHEM)
            .iter()
            .map(|g| (g.id, g.advance, g.x, g.y))
            .collect::<Vec<_>>(),
        [
            (37, 10.0, 0.0, 0.0),
            (38, 10.0, 10.0, 0.0),
            (39, 10.0, 20.0, 0.0),
            (35, 10.0, 40.0, 0.0)
        ]
    );
    place_atomic_pair(&p, first, second, 30.0);
}
