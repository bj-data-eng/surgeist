//! Current clockwise atomic alignment, custody, bidi and empty dispositions.
use super::*;
use crate::{Error, ErrorCode, ErrorDetail, TextNumericError, TextNumericPhase};

fn value(raw: &str, rtl: bool, kinds: &[InlineBoxKind], affinity: Affinity) -> ParagraphInput {
    let mut src = Source::identified(raw, Id::from_u64(1_556_100), 1);
    for kind in kinds {
        src.inline_box(InlineBox::new(
            if *kind == InlineBoxKind::InFlow {
                INFLOW
            } else {
                POSITIONED
            },
            *kind,
            usize::from(!raw.is_empty()),
            if *kind == InlineBoxKind::InFlow {
                Size::new(6.0, 14.0)
            } else {
                Size::new(60.0, 140.0)
            },
        ));
    }
    let mut result = input(src, own_style(), rtl);
    if raw.is_empty() {
        result.origins.clear();
    }
    result.anchor_order = (0..kinds.len())
        .map(|index| ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(index),
            affinity,
        })
        .collect();
    result.boundary_policies = (0..kinds.len())
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
        .collect();
    result
}

fn finalized(prepared: &PreparedText, alignment: LineBoxAlignment) -> FinalizedLine {
    let mut selected = selected_request(prepared, start(prepared), terminal(prepared));
    selected.atomics = prepared
        .atomic_views()
        .map(|view| {
            let mut local = allocation();
            if let AtomicLineAllocation::InFlow {
                alignment: actual, ..
            } = &mut local
            {
                *actual = alignment;
            }
            AtomicAllocation {
                occurrence: view.handle(),
                allocation: if view.original_box().kind() == InlineBoxKind::InFlow {
                    local
                } else {
                    AtomicLineAllocation::OutOfFlow
                },
            }
        })
        .collect();
    prepared
        .select(selected)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn out_of_flow(occurrence: PreparedAtomic) -> AtomicPlacement {
    AtomicPlacement {
        occurrence,
        border: Rect::new(300.0, 500.0, 60.0, 140.0),
        content: Rect::new(305.0, 507.0, 48.0, 126.0),
        baseline: 321.0,
    }
}

fn placement(
    prepared: &PreparedText,
    line: &FinalizedLine,
    atomics: Vec<AtomicPlacement>,
) -> PlacementRequest {
    PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: CW,
            atomics,
        })],
    }
}

fn protocol(error: Error, reason: ProtocolError) {
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol { reason })
    );
}

fn physical_pair(prepared: &PreparedText, left: f64, cross: f64) -> Vec<AtomicPlacement> {
    prepared
        .atomic_views()
        .map(|view| {
            if view.original_box().kind() == InlineBoxKind::OutOfFlow {
                out_of_flow(view.handle())
            } else {
                AtomicPlacement {
                    occurrence: view.handle(),
                    border: Rect::new(left, 30.0, 6.0, 14.0),
                    content: Rect::new(left + 1.0, 32.0, 4.0, 10.0),
                    baseline: cross,
                }
            }
        })
        .collect()
}

#[test]
fn clockwise_baseline_top_bottom_keep_physical_box_and_actual_cross_reference() {
    let mut system = system_with(AHEM.to_vec());
    let prepared = prepare(&mut system, mixed_input(), CW);
    for (alignment, height, slot_cross, left, box_reference) in [
        (LineBoxAlignment::Baseline, 12.0, 8.0, 96.0, 100.0),
        (LineBoxAlignment::Top, 10.0, 2.0, 102.0, 106.0),
        (LineBoxAlignment::Bottom, 10.0, 6.0, 98.0, 102.0),
    ] {
        let line = finalized(&prepared, alignment);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (24.0, 8.0, height)
        );
        let slots = line.atomic_slots().collect::<Vec<_>>();
        let inflow = slots
            .iter()
            .find(|slot| slot.occurrence.view().original_box().kind() == InlineBoxKind::InFlow)
            .unwrap();
        assert_eq!(inflow.origin, Point::new(10.0, slot_cross));
        let AtomicLineAllocation::InFlow {
            alignment: actual, ..
        } = inflow.allocation
        else {
            panic!("actual in-flow slot required")
        };
        assert_eq!(actual, alignment);
        let positioned = slots
            .iter()
            .find(|slot| slot.occurrence.view().original_box().kind() == InlineBoxKind::OutOfFlow)
            .unwrap();
        assert_eq!(positioned.allocation, AtomicLineAllocation::OutOfFlow);
        assert_eq!(positioned.origin, Point::new(10.0, 8.0));
        let layout = Layout::from_finalized_lines(placement(
            &prepared,
            &line,
            physical_pair(&prepared, left, box_reference),
        ))
        .unwrap();
        assert_eq!(
            layout.lines()[0].bounds(),
            Rect::new(108.0 - height, 20.0, height, 24.0)
        );
        let boxes = layout.inline_boxes();
        assert_eq!(
            boxes.iter().find(|b| b.id() == INFLOW).unwrap().rect(),
            Rect::new(left, 30.0, 6.0, 14.0)
        );
        assert_eq!(
            layout.hit(Point::new(left + 2.0, 35.0)),
            Hit::InlineBox(INFLOW)
        );
        assert_eq!(text_hit(&layout, 103.0, 22.0), 0);
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::After))
                .unwrap()
                .rect(),
            Rect::new(108.0 - height, 20.0, height, 1.0)
        );
        for run in layout.run_views() {
            assert_eq!(run.glyph_basis(), crate::GlyphBasis::Clockwise);
        }
    }
}

#[test]
fn clockwise_atomic_placement_preserves_missing_duplicate_foreign_and_numeric_rejection() {
    let mut system = system_with(AHEM.to_vec());
    let prepared = prepare(&mut system, mixed_input(), CW);
    let line = finalized(&prepared, LineBoxAlignment::Baseline);
    let actual = physical_pair(&prepared, 96.0, 100.0);
    let inflow = actual
        .iter()
        .find(|p| p.occurrence.view().original_box().kind() == InlineBoxKind::InFlow)
        .unwrap()
        .clone();
    let positioned = actual
        .iter()
        .find(|p| p.occurrence.view().original_box().kind() == InlineBoxKind::OutOfFlow)
        .unwrap()
        .clone();
    // Placement diagnostics identify the issued preparation row, not the
    // original box-storage index. This input orders positioned1 before inflow0.
    let inflow_ordinal = prepared
        .atomic_views()
        .position(|view| view.handle() == inflow.occurrence)
        .unwrap();
    let positioned_ordinal = prepared
        .atomic_views()
        .position(|view| view.handle() == positioned.occurrence)
        .unwrap();
    assert_eq!((inflow_ordinal, positioned_ordinal), (1, 0));
    let old = Layout::from_finalized_lines(placement(&prepared, &line, actual.clone())).unwrap();
    for (remaining, missing) in [
        (vec![positioned.clone()], inflow_ordinal),
        (vec![inflow.clone()], positioned_ordinal),
    ] {
        protocol(
            Layout::from_finalized_lines(placement(&prepared, &line, remaining)).unwrap_err(),
            ProtocolError::MissingAtomic {
                occurrence: missing,
            },
        );
    }
    for (duplicates, occurrence) in [
        (
            vec![inflow.clone(), inflow.clone(), positioned.clone()],
            inflow_ordinal,
        ),
        (
            vec![inflow.clone(), positioned.clone(), positioned.clone()],
            positioned_ordinal,
        ),
    ] {
        protocol(
            Layout::from_finalized_lines(placement(&prepared, &line, duplicates)).unwrap_err(),
            ProtocolError::DuplicateAtomic { occurrence },
        );
    }
    for wrong in [
        AtomicPlacement {
            baseline: 30.0,
            ..inflow.clone()
        },
        AtomicPlacement {
            border: Rect::new(95.0, 30.0, 6.0, 14.0),
            ..inflow.clone()
        },
        AtomicPlacement {
            content: Rect::new(98.0, 32.0, 4.0, 10.0),
            ..inflow.clone()
        },
    ] {
        protocol(
            Layout::from_finalized_lines(placement(
                &prepared,
                &line,
                vec![wrong, positioned.clone()],
            ))
            .unwrap_err(),
            ProtocolError::AtomicPlacementMismatch {
                line: 0,
                occurrence: inflow_ordinal,
            },
        );
    }
    let mut other_input = mixed_input();
    let mut other_source = Source::identified("A", Id::from_u64(1_556_999), 1);
    for actual in other_input.source.boxes() {
        other_source.inline_box(*actual);
    }
    other_input.origins[0].source = other_source.clone();
    other_input.source = other_source;
    let foreign = prepare(&mut system, other_input, CW);
    let foreign_oof = foreign
        .atomic_views()
        .find(|view| view.original_box().kind() == InlineBoxKind::OutOfFlow)
        .unwrap()
        .handle();
    let foreign_ordinal = foreign
        .atomic_views()
        .position(|view| view.handle() == foreign_oof)
        .unwrap();
    assert_ne!(foreign_oof, positioned.occurrence);
    protocol(
        Layout::from_finalized_lines(placement(
            &prepared,
            &line,
            vec![inflow.clone(), out_of_flow(foreign_oof)],
        ))
        .unwrap_err(),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::Atomic,
            occurrence: foreign_ordinal,
        },
    );
    for (bad_inflow, bad_oof, field) in [
        (
            AtomicPlacement {
                border: Rect::new(f64::NAN, 30.0, 6.0, 14.0),
                ..inflow.clone()
            },
            positioned.clone(),
            "placed x",
        ),
        (
            inflow.clone(),
            AtomicPlacement {
                content: Rect::new(305.0, f64::INFINITY, 48.0, 126.0),
                ..positioned.clone()
            },
            "placed y",
        ),
        (
            inflow.clone(),
            AtomicPlacement {
                baseline: f64::NEG_INFINITY,
                ..positioned.clone()
            },
            "placed atomic baseline",
        ),
    ] {
        let error =
            Layout::from_finalized_lines(placement(&prepared, &line, vec![bad_inflow, bad_oof]))
                .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric { field: actual, phase: TextNumericPhase::Placement, category: TextNumericError::NonFinite, .. }) if *actual == field),
            "{error:?}"
        );
    }
    let moved = AtomicPlacement {
        border: Rect::new(-40.0, 90.0, 6.0, 14.0),
        content: Rect::new(-39.0, 92.0, 4.0, 10.0),
        baseline: -37.0,
        ..positioned
    };
    let retry =
        Layout::from_finalized_lines(placement(&prepared, &line, vec![inflow, moved.clone()]))
            .unwrap();
    assert_eq!(
        retry
            .inline_boxes()
            .iter()
            .find(|b| b.id() == POSITIONED)
            .unwrap()
            .rect(),
        moved.border
    );
    assert_eq!(
        old.inline_boxes()
            .iter()
            .find(|b| b.id() == POSITIONED)
            .unwrap()
            .rect(),
        Rect::new(300.0, 500.0, 60.0, 140.0)
    );
}

fn qualify_rtl_font(bytes: &[u8]) {
    let parsed = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    // The real authored cmap maps Alef to Ahem X58, then automatic Sideways
    // vrtr selects274. This enables real Hebrew UBA, not a Hebrew-design claim.
    assert_eq!(parsed.charmap().map(0x5d0_u32), Some(GlyphId::new(58)));
    assert_eq!(parsed.charmap().map('A'), Some(GlyphId::new(35)));
    assert!(parsed.gsub().is_ok());
    let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for glyph in [35, 58, 274] {
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(1000.0));
    }
}

#[test]
fn clockwise_genuine_odd_level_inflow_and_positioned_affinities_keep_source_custody() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    qualify_rtl_font(&bytes);
    let mut system = system_with(bytes.clone());
    let prepared = prepare(
        &mut system,
        value(
            "\u{5d0}\u{5d0}",
            true,
            &[InlineBoxKind::InFlow],
            Affinity::After,
        ),
        CW,
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| (unit.source_range(), unit.level().get()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 1), (Range::new(1, 2), 1)]
    );
    assert_eq!(prepared.atomic_views().next().unwrap().level().get(), 1);
    let line = finalized(&prepared, LineBoxAlignment::Baseline);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (34.0, 8.0, 12.0)
    );
    let mut positions = line
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), run.origin().x + glyph.pen()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    positions.sort_by_key(|row| row.0.start);
    assert_eq!(
        positions,
        [(Range::new(0, 1), 274, 24.0), (Range::new(1, 2), 274, 0.0)]
    );
    let atomic = prepared.atomic_views().next().unwrap().handle();
    assert_eq!(
        line.atomic_slots().next().unwrap().origin,
        Point::new(10.0, 8.0)
    );
    let layout = Layout::from_finalized_lines(placement(
        &prepared,
        &line,
        vec![AtomicPlacement {
            occurrence: atomic,
            border: Rect::new(96.0, 30.0, 6.0, 14.0),
            content: Rect::new(97.0, 32.0, 4.0, 10.0),
            baseline: 100.0,
        }],
    ))
    .unwrap();
    assert_eq!(layout.source().text().code_units(), [0x5d0, 0x5d0]);
    assert_eq!(layout.hit(Point::new(98.0, 35.0)), Hit::InlineBox(INFLOW));
    for run in layout.run_views() {
        assert_eq!(run.glyph_basis(), crate::GlyphBasis::Clockwise);
    }

    // RTL Hebrew1 + Latin2 produces Latin at u0 and Hebrew at u10. At their
    // original seam, Before follows Hebrew's logical end u10, while After
    // follows Latin's logical start u0; OOF adds no UBA placeholder.
    for (affinity, static_u) in [(Affinity::Before, 10.0), (Affinity::After, 0.0)] {
        let raw = value("\u{5d0}A", true, &[InlineBoxKind::OutOfFlow], affinity);
        let analysis = analyze(raw);
        let horizontal = system
            .prepare(&analysis, PreparationOptions::default())
            .unwrap();
        let hline = finalized(&horizontal, LineBoxAlignment::Baseline);
        let mut hids = hline
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|g| (g.range(), g.id()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        hids.sort_by_key(|row| row.0.start);
        assert_eq!(hids, [(Range::new(0, 1), 58), (Range::new(1, 2), 35)]);
        let prepared = system
            .prepare(
                &analysis,
                PreparationOptions::default().with_orientation(CW),
            )
            .unwrap();
        assert_eq!(
            prepared
                .unit_views()
                .map(|unit| (unit.source_range(), unit.level().get()))
                .collect::<Vec<_>>(),
            [(Range::new(0, 1), 1), (Range::new(1, 2), 2)]
        );
        let line = finalized(&prepared, LineBoxAlignment::Baseline);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (20.0, 8.0, 10.0)
        );
        assert_eq!(
            line.atomic_slots().next().unwrap().origin,
            Point::new(static_u, 8.0)
        );
        let atomic = prepared.atomic_views().next().unwrap().handle();
        let layout =
            Layout::from_finalized_lines(placement(&prepared, &line, vec![out_of_flow(atomic)]))
                .unwrap();
        assert_eq!(
            layout
                .cursor(Cursor::new(1, Affinity::Before))
                .unwrap()
                .rect(),
            Rect::new(98.0, 30.0, 10.0, 1.0)
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(1, Affinity::After))
                .unwrap()
                .rect(),
            Rect::new(98.0, 20.0, 10.0, 1.0)
        );
        assert_eq!(text_hit(&layout, 103.0, 22.0), 1);
        assert_eq!(text_hit(&layout, 103.0, 38.0), 0);
        assert_eq!(
            layout.hit(Point::new(310.0, 510.0)),
            Hit::InlineBox(POSITIONED)
        );
        let mut actual = layout
            .glyph_runs()
            .into_iter()
            .flat_map(|run| {
                run.glyphs()
                    .iter()
                    .map(|g| (g.range(), g.id(), run.origin().y + g.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        actual.sort_by_key(|row| row.0.start);
        assert_eq!(
            actual,
            [(Range::new(0, 1), 274, 30.0), (Range::new(1, 2), 35, 20.0)]
        );
    }
}

#[test]
fn clockwise_source_free_inflow_formats_but_positioned_only_keeps_nonformatted_completion() {
    let mut system = system_with(AHEM.to_vec());
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let prepared = prepare(&mut system, value("", false, &[kind], Affinity::After), CW);
        assert_eq!(prepared.origin_views().count(), 0);
        assert_eq!(prepared.atomic_views().count(), 1);
        let atomic = prepared.atomic_views().next().unwrap().handle();
        assert_eq!(
            prepared.participants().collect::<Vec<_>>(),
            [PreparedParticipant::Atomic(atomic.clone())]
        );
        let line = finalized(&prepared, LineBoxAlignment::Baseline);
        assert_eq!(
            line.run_views()
                .map(|run| run.glyphs().count())
                .sum::<usize>(),
            0
        );
        assert_eq!(line.fragment_views().count(), 0);
        if kind == InlineBoxKind::InFlow {
            assert!(line.has_in_flow_content());
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (14.0, 8.0, 12.0)
            );
            protocol(
                line.nonformatted().unwrap_err(),
                ProtocolError::NonFormattedInFlowContent,
            );
            let actual = AtomicPlacement {
                occurrence: atomic,
                border: Rect::new(96.0, 20.0, 6.0, 14.0),
                content: Rect::new(97.0, 22.0, 4.0, 10.0),
                baseline: 100.0,
            };
            let layout =
                Layout::from_finalized_lines(placement(&prepared, &line, vec![actual.clone()]))
                    .unwrap();
            assert_eq!(layout.lines().len(), 1);
            assert_eq!(layout.lines()[0].range(), Range::new(0, 0));
            assert_eq!(
                layout.lines()[0].bounds(),
                Rect::new(96.0, 20.0, 12.0, 14.0)
            );
            assert_eq!(layout.inline_boxes()[0].rect(), actual.border);
            assert_eq!(layout.hit(Point::new(98.0, 25.0)), Hit::InlineBox(INFLOW));
        } else {
            assert!(!line.has_in_flow_content());
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (0.0, 8.0, 10.0)
            );
            let selected = line.selected_line();
            assert_eq!(selected.request().context.end, LineEnd::Paragraph);
            let PreparedTermination::Ordinary(offer) = &selected.request().termination else {
                panic!("genuine terminal offer required")
            };
            assert_eq!(offer.view().kind(), PreparedBreakKind::ParagraphEnd);
            let actual = out_of_flow(atomic);
            let completion = line.nonformatted().unwrap();
            let layout = Layout::from_finalized_lines(PlacementRequest {
                prepared: prepared.clone(),
                coverage: PlacementCoverage::WholeParagraph,
                entries: vec![PlacementEntry::NonFormatted(NonFormattedLinePlacement {
                    line: completion,
                    origin: Point::new(7.0, 11.0),
                    orientation: CW,
                    atomics: vec![actual.clone()],
                })],
            })
            .unwrap();
            assert!(layout.lines().is_empty());
            assert_eq!(layout.metrics().line_count(), 0);
            assert_eq!(layout.positions().count(), 0);
            let retained = layout.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(retained.len(), 1);
            assert_eq!(retained[0].line.finalized_line(), &line);
            assert_eq!(retained[0].origin, Point::new(7.0, 11.0));
            assert_eq!(retained[0].atomics, [actual]);
        }
    }
}
