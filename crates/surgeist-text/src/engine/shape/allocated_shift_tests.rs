//! Functional qualification of actual allocated shifts; standalone metadata is not used.
use super::*;
use crate::{
    AtomicAllocation, AtomicBoxAllocation, AtomicFlowAllocation, AtomicLineAllocation,
    AtomicPlacement, BaselineShift, ControlAllocation, ControlStrut, ErrorDetail, GlyphBasis, Id,
    InlineBoundaryAllocation, InlineBox, InlineBoxKind, Layout, LineAnchor, LineBoxAlignment,
    LineBoxStrut, LinePlacement, LineStrut, ParagraphAnchor, ParagraphAnchorItem,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphControl, ParagraphControlKind,
    PlacementCoverage, PlacementEntry, PlacementRequest, Rect, Size as PhysicalSize,
    TextNumericError, TextNumericPhase,
};
use skrifa::raw::TableProvider;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
#[derive(Clone, Copy, Debug)]
enum Row {
    Atomic,
    Control,
    Additional,
}
fn shift(value: f64) -> LineBoxAlignment {
    LineBoxAlignment::Shift(BaselineShift::try_new(value).unwrap())
}
fn orientation(frame: u8) -> LineOrientation {
    match frame {
        0 => LineOrientation::Horizontal,
        1 => LineOrientation::SidewaysClockwise,
        _ => LineOrientation::Vertical,
    }
}
fn qualify() {
    let face = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.charmap().map('X'), Some(GlyphId::new(58)));
    assert_eq!(face.hmtx().unwrap().h_metrics()[58].advance.get(), 1000);
    assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
    let bounds = face
        .glyph_metrics(FontSize::unscaled(), LocationRef::default())
        .bounds(GlyphId::new(58))
        .unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    let h = face.hhea().unwrap();
    assert_eq!(
        (
            h.ascender().to_i16(),
            h.descender().to_i16(),
            h.line_gap().to_i16()
        ),
        (800, -200, 0)
    );
    for tag in [b"BASE", b"vhea", b"vmtx", b"GSUB", b"fvar"] {
        assert!(face.table_data(skrifa::Tag::new(tag)).is_none());
    }
    let face = surgeist_shaper::FontRef::new(AHEM).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    input.add('X', 0);
    input.set_direction(surgeist_shaper::Direction::LeftToRight);
    input.set_script(surgeist_shaper::script::LATIN);
    let output = shaper.shape(input, &[]);
    assert_eq!(
        output.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    assert_eq!(output.glyph_infos()[0].glyph_id, 58);
    assert_eq!(output.glyph_positions()[0].x_advance, 1000);
}
fn fixture(row: Row, frame: u8, scale: f32, text: &str) -> (System, PreparedText) {
    let mut source = Source::new(text);
    let vertical = orientation(frame) != LineOrientation::Horizontal;
    if matches!(row, Row::Atomic) {
        // These are real physical dimensions for this fixture's input box.
        // Its local inline/under allocation below maps back to exactly them.
        source.inline_box(InlineBox::new(
            Id::from_u64(1574010),
            InlineBoxKind::InFlow,
            0,
            if vertical {
                PhysicalSize::new(10.0, 6.0)
            } else {
                PhysicalSize::new(6.0, 10.0)
            },
        ));
    }
    let mut value = input(
        &source,
        Style {
            size: 1.0,
            line_height: crate::LineHeight::Absolute(1.0),
            ..own(0.0)
        },
        LTR,
    );
    value.paragraph_strut = ParagraphStrut::AllocatedContributions;
    if text.is_empty() {
        value.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::EmptyOrigin(0),
            affinity: Affinity::Before,
        });
    }
    match row {
        Row::Atomic => {
            value.anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: Affinity::After,
            });
            value.boundary_policies = [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .collect();
        }
        Row::Control => {
            value.controls.push(ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            });
            value.anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: Affinity::After,
            });
        }
        Row::Additional => {}
    }
    let typography = match frame {
        0 | 1 => AnalysisTypography::HorizontalComposition,
        2 => AnalysisTypography::Upright {
            computed_direction: LTR,
        },
        _ => AnalysisTypography::Mixed,
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, typography).unwrap(),
        Wrap::Word,
    );
    let mut system = system(AHEM);
    let options = PreparationOptions::try_new(scale, crate::TabPreparation::FontAdvance)
        .unwrap()
        .with_orientation(orientation(frame));
    let p = system.prepare(&analyzed, options).unwrap();
    (system, p)
}
fn allocated(
    p: &PreparedText,
    row: Row,
    alignment: LineBoxAlignment,
    parent: bool,
) -> LineSelectionRequest {
    let mut r = request(p, LTR, SpacingPolicy::None);
    if parent {
        r.line_strut = Some(LineStrut {
            baseline: 8.0,
            line_extent: 10.0,
        });
    }
    match row {
        Row::Atomic => {
            r.atomics = vec![AtomicAllocation {
                occurrence: p.atomic_views().next().unwrap().handle(),
                allocation: AtomicLineAllocation::InFlow {
                    fit: crate::AtomicFitAllocation::NativeOccupied,
                    flow: AtomicFlowAllocation {
                        advance: 6.0,
                        before_baseline: 6.0,
                        after_baseline: 4.0,
                        occupied_start: 0.0,
                        occupied_end: 6.0,
                    },
                    box_geometry: AtomicBoxAllocation {
                        border: Rect::new(0.0, -6.0, 6.0, 10.0),
                        content: Rect::new(1.0, -5.0, 4.0, 8.0),
                        baseline: 0.0,
                    },
                    alignment,
                },
            }]
        }
        Row::Control => {
            r.controls = vec![ControlAllocation {
                occurrence: p.control_views().next().unwrap().handle(),
                boundary: Some(InlineBoundaryAllocation::ZERO),
                strut: Some(ControlStrut {
                    baseline: 6.0,
                    line_extent: 10.0,
                    alignment,
                }),
            }]
        }
        Row::Additional => {
            r.line_box_struts = vec![LineBoxStrut {
                scope: None,
                metrics: LineStrut {
                    baseline: 6.0,
                    line_extent: 10.0,
                },
                alignment,
            }]
        }
    }
    r
}
fn raw_packets(line: &FinalizedLine, bytes: &[u8]) -> Vec<(Range, u32, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(
                run.face().catalogue_identity(),
                line.prepared_text().font_catalogue_identity()
            );
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            assert_eq!(
                instance.font_size(),
                line.prepared_text().preparation_options().scale()
            );
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "actual {a}, expected {b}");
}
fn place(p: &PreparedText, line: FinalizedLine, delta: f64, anchor: f64) -> crate::Result<Layout> {
    let orientation = p.orientation();
    let atomics = line
        .atomic_slots()
        .map(|slot| {
            let horizontal = orientation == LineOrientation::Horizontal;
            AtomicPlacement {
                occurrence: slot.occurrence,
                border: if horizontal {
                    Rect::new(20.0, anchor - (delta + 6.0), 6.0, 10.0)
                } else {
                    Rect::new(anchor + (delta - 4.0), 20.0, 10.0, 6.0)
                },
                content: if horizontal {
                    Rect::new(21.0, anchor - (delta + 5.0), 4.0, 8.0)
                } else {
                    Rect::new(anchor + (delta - 3.0), 21.0, 8.0, 4.0)
                },
                baseline: if horizontal {
                    anchor - delta
                } else {
                    anchor + delta
                },
            }
        })
        .collect();
    Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: anchor,
            },
            orientation,
            atomics,
        })],
    })
}

#[test]
fn allocated_shift_atomic_control_and_additional_struts_keep_raw_metrics_across_frames_and_scale() {
    qualify();
    for row in [Row::Atomic, Row::Control, Row::Additional] {
        for frame in 0..4 {
            for scale in [1.0, 2.0] {
                let (mut system, p) = fixture(row, frame, scale, "X");
                let ordinary = p
                    .select(allocated(&p, row, LineBoxAlignment::Baseline, true))
                    .unwrap()
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::None,
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();
                let raw = raw_packets(&ordinary, AHEM);
                assert_eq!(raw.len(), 1);
                assert_eq!(raw[0].0, Range::new(0, 1));
                assert_eq!(raw[0].1, 58);
                assert_eq!(raw[0].4, scale);
                assert_eq!(
                    (raw[0].2, raw[0].3),
                    if frame == 2 {
                        (-scale / 2.0, scale * 0.8)
                    } else {
                        (0.0, 0.0)
                    }
                );
                assert_eq!(
                    ordinary.run_views().next().unwrap().glyph_basis(),
                    if frame == 0 || frame == 2 {
                        GlyphBasis::Unrotated
                    } else {
                        GlyphBasis::Clockwise
                    }
                );
                for (delta, baseline, height) in [
                    (0.0, 8.0, 12.0),
                    (3.0, 9.0, 11.0),
                    (-3.0, 8.0, 15.0),
                    (20.0, 26.0, 28.0),
                    (-20.0, 8.0, 32.0),
                ] {
                    // At zero, child6/4 and parent8/2 require8/4 =12. There are no
                    // scaled font extents larger than that real allocated parent.
                    let r = allocated(&p, row, shift(delta), true);
                    reset_provider_calls();
                    let selected = p.select(r.clone()).unwrap();
                    let calls = provider_calls();
                    assert!(calls > 0);
                    let line = selected
                        .finalize(crate::LineAllocations {
                            spacing: SpacingAllocations::None,
                            scopes: crate::ScopeAllocations::None,
                        })
                        .unwrap();
                    assert_eq!(provider_calls(), calls);
                    assert_eq!(selected.request().line_strut, r.line_strut);
                    near(line.feedback().baseline, baseline);
                    near(line.feedback().line_height, height);
                    assert_eq!(raw_packets(&line, AHEM), raw);
                    match row {
                        Row::Atomic => {
                            let slot = line.atomic_slots().next().unwrap();
                            near(slot.origin.y, baseline - delta);
                            assert_eq!(slot.allocation, r.atomics[0].allocation);
                        }
                        Row::Control => {
                            let slot = line.control_slots().next().unwrap();
                            near(slot.slot().y, baseline - delta);
                            assert_eq!(slot.control(), r.controls[0].occurrence);
                        }
                        Row::Additional => {
                            assert_eq!(selected.request().line_box_struts, r.line_box_struts)
                        }
                    }
                    let again = selected
                        .finalize(crate::LineAllocations {
                            spacing: SpacingAllocations::None,
                            scopes: crate::ScopeAllocations::None,
                        })
                        .unwrap();
                    let restored = line.with_spacing(SpacingAllocations::None).unwrap();
                    assert_eq!(provider_calls(), calls);
                    assert_eq!(again.selected_line(), selected);
                    assert_eq!(restored.selected_line(), selected);
                    assert_eq!(raw_packets(&restored, AHEM), raw);
                    let layout = place(&p, line, delta, 100.0).unwrap();
                    assert_eq!(layout.source().text().code_units(), [88]);
                    let selection = layout
                        .selection(crate::Selection::new(
                            crate::Cursor::new(0, Affinity::After),
                            crate::Cursor::new(1, Affinity::Before),
                        ))
                        .unwrap();
                    assert_eq!(selection.rects().len(), 1);
                    let start = layout
                        .positions()
                        .find(|p| p.source_cursor() == crate::Cursor::new(0, Affinity::After))
                        .unwrap();
                    let end = layout
                        .positions()
                        .find(|p| p.source_cursor() == crate::Cursor::new(1, Affinity::Before))
                        .unwrap();
                    assert_eq!(
                        layout
                            .position_selection(&start, &end)
                            .unwrap()
                            .source_text(),
                        p.analyzed_source().source().text()
                    );
                    #[cfg(feature = "text-accessibility")]
                    {
                        let parent = accesskit::NodeId(1574000);
                        let update = layout
                            .accessibility_update(
                                &mut crate::Accessibility::new(10),
                                parent,
                                parent,
                                crate::Point::default(),
                            )
                            .unwrap();
                        let text = update
                            .nodes
                            .iter()
                            .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
                            .map(|(_, node)| node)
                            .collect::<Vec<_>>();
                        assert_eq!(
                            text.iter()
                                .filter_map(|node| node.value())
                                .collect::<String>(),
                            "X"
                        );
                        for node in text {
                            let bounds = node.bounds().unwrap();
                            near(
                                if frame == 0 {
                                    bounds.height()
                                } else {
                                    bounds.width()
                                },
                                height,
                            );
                        }
                    }
                    assert_eq!(provider_calls(), calls);
                }
                // Historical instances remain sufficient for immutable replay.
                let selected = ordinary.selected_line();
                system.refresh_fonts().unwrap();
                drop(system);
                let calls = provider_calls();
                let replay = selected
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::None,
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();
                assert_eq!(provider_calls(), calls);
                assert_eq!(raw_packets(&replay, AHEM), raw);
            }
        }
    }
}

#[test]
fn allocated_shift_without_parent_strut_preserves_one_sided_bounds_and_absence() {
    for row in [Row::Atomic, Row::Control, Row::Additional] {
        for delta in [20.0, -20.0, 3.0, -3.0, 0.0] {
            let (_system, p) = fixture(row, 0, 1.0, "");
            let r = allocated(&p, row, shift(delta), false);
            assert!(r.line_strut.is_none());
            assert_eq!(
                p.analyzed_source().paragraph_strut(),
                ParagraphStrut::AllocatedContributions
            );
            let line = p
                .select(r)
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            // Only child[-6-delta,4-delta] exists; neither baseline zero nor a
            // DefaultFont strut is an occupied contribution. Extent stays10.
            near(line.feedback().baseline, 6.0 + delta);
            near(line.feedback().line_height, 10.0);
            assert_eq!(line.run_views().count(), 0);
            match row {
                Row::Atomic => near(line.atomic_slots().next().unwrap().origin.y, 6.0),
                Row::Control => near(line.control_slots().next().unwrap().slot().y, 6.0),
                Row::Additional => {}
            }
        }
    }
}

#[test]
fn allocated_shift_context_distinguishes_variant_value_and_canonical_signed_zero() {
    for row in [Row::Atomic, Row::Control, Row::Additional] {
        let (_system, p) = fixture(row, 0, 1.0, "X");
        let context = |alignment| {
            crate::protocol::request_context((&allocated(&p, row, alignment, true)).into())
        };
        assert_ne!(context(LineBoxAlignment::Baseline), context(shift(0.0)));
        assert_eq!(context(shift(0.0)), context(shift(-0.0)));
        assert_ne!(context(shift(3.0)), context(shift(-3.0)));
        let selected = p.select(allocated(&p, row, shift(0.0), true)).unwrap();
        match row {
            Row::Atomic => assert!(matches!(
                selected.request().atomics[0].allocation,
                AtomicLineAllocation::InFlow {
                    alignment: LineBoxAlignment::Shift(_),
                    ..
                }
            )),
            Row::Control => assert!(matches!(
                selected.request().controls[0].strut.unwrap().alignment,
                LineBoxAlignment::Shift(_)
            )),
            Row::Additional => assert!(matches!(
                selected.request().line_box_struts[0].alignment,
                LineBoxAlignment::Shift(_)
            )),
        }
    }
}

#[test]
fn allocated_shift_arithmetic_and_physical_loss_refuse_without_changing_prior_output() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(BaselineShift::try_new(value).is_err());
    }
    for row in [Row::Atomic, Row::Control, Row::Additional] {
        let (_system, p) = fixture(row, 0, 1.0, "X");
        let good = p
            .select(allocated(&p, row, shift(3.0), true))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let raw = raw_packets(&good, AHEM);
        let feedback = good.feedback();
        for delta in [f64::from_bits(1), 1e308, -1e308] {
            let selected = p.select(allocated(&p, row, shift(delta), true)).unwrap();
            let error = selected
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap_err();
            assert!(
                matches!(
                    error.detail(),
                    Some(ErrorDetail::InvalidTextNumeric {
                        phase: TextNumericPhase::Finalization,
                        category: TextNumericError::NonZeroLost,
                        ..
                    })
                ),
                "{error:?}"
            );
            assert_eq!(raw_packets(&good, AHEM), raw);
            assert_eq!(good.feedback(), feedback);
        }
        let line = p
            .select(allocated(&p, row, shift(1.0), true))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let error = place(&p, line, 1.0, 1e16).unwrap_err();
        assert!(
            matches!(
                error.detail(),
                Some(ErrorDetail::InvalidTextNumeric {
                    phase: TextNumericPhase::Placement,
                    category: TextNumericError::NonZeroLost,
                    ..
                })
            ),
            "{error:?}"
        );
        if matches!(row, Row::Atomic) {
            let mut r = allocated(&p, row, shift(1.0), true);
            if let AtomicLineAllocation::InFlow { box_geometry, .. } = &mut r.atomics[0].allocation
            {
                box_geometry.baseline = 1e16;
            }
            let selected = p.select(r).unwrap();
            let error = selected
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap_err();
            assert!(
                matches!(
                    error.detail(),
                    Some(ErrorDetail::InvalidTextNumeric {
                        phase: TextNumericPhase::Finalization,
                        category: TextNumericError::NonZeroLost,
                        ..
                    })
                ),
                "{error:?}"
            );
        }
        let calls = provider_calls();
        let replay = good.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(provider_calls(), calls);
        assert_eq!(raw_packets(&replay, AHEM), raw);
    }
}

#[test]
fn allocated_shift_keeps_provider_and_source_basis_through_positive_spacing_replacement_and_failure()
 {
    let (mut system, p) = fixture(Row::Additional, 0, 1.0, "XX");
    let mut r = allocated(&p, Row::Additional, shift(3.0), true);
    r.context.spacing = SpacingPolicy::InterCharacter;
    reset_provider_calls();
    let selected = p.select(r).unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    assert_eq!(selected.spacing_slots().len(), 1);
    let original = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let raw = raw_packets(&original, AHEM);
    assert_eq!(
        raw,
        [
            (Range::new(0, 1), 58, 0.0, 0.0, 1.0),
            (Range::new(1, 2), 58, 0.0, 0.0, 1.0)
        ]
    );
    near(original.feedback().advance, 2.0);
    let expanded = original.with_spacing(amounts(&selected, &[4.0])).unwrap();
    let replaced = expanded.with_spacing(amounts(&selected, &[2.0])).unwrap();
    let restored = replaced.with_spacing(SpacingAllocations::None).unwrap();
    for (line, advance) in [(&expanded, 6.0), (&replaced, 4.0), (&restored, 2.0)] {
        near(line.feedback().advance, advance);
        near(line.feedback().baseline, 9.0);
        near(line.feedback().line_height, 11.0);
        assert_eq!(raw_packets(line, AHEM), raw);
        assert_eq!(line.selected_line(), selected);
    }
    assert!(
        expanded
            .with_spacing(amounts(&selected, &[f64::NAN]))
            .is_err()
    );
    assert_eq!(provider_calls(), calls);
    let foreign = p.select(selected.request().clone()).unwrap();
    let after_foreign = provider_calls();
    assert!(after_foreign > calls);
    let foreign_amounts = amounts(&foreign, &[1.0]);
    assert!(expanded.with_spacing(foreign_amounts).is_err());
    assert_eq!(provider_calls(), after_foreign);
    let calls = provider_calls();
    system.refresh_fonts().unwrap();
    drop(system);
    let replay = original.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(provider_calls(), calls);
    assert_eq!(raw_packets(&replay, AHEM), raw);
    near(original.feedback().advance, 2.0);
}

#[test]
fn allocated_shift_keeps_foreign_occurrence_checks_and_detects_extent_overflow() {
    for row in [Row::Atomic, Row::Control] {
        let (_system, p) = fixture(row, 0, 1.0, "X");
        let (_other, foreign) = fixture(row, 0, 1.0, "X");
        let mut r = allocated(&p, row, shift(3.0), true);
        let kind = match row {
            Row::Atomic => {
                r.atomics[0].occurrence = foreign.atomic_views().next().unwrap().handle();
                crate::ProtocolHandleKind::Atomic
            }
            Row::Control => {
                r.controls[0].occurrence = foreign.control_views().next().unwrap().handle();
                crate::ProtocolHandleKind::Control
            }
            Row::Additional => unreachable!(),
        };
        let error = p.select(r).unwrap_err();
        assert!(
            matches!(error.detail(),Some(ErrorDetail::InvalidTextProtocol{reason:crate::ProtocolError::ForeignPreparation{kind:actual,..}}) if *actual==kind),
            "{error:?}"
        );
    }
    let (_system, p) = fixture(Row::Additional, 0, 1.0, "X");
    let mut r = allocated(&p, Row::Additional, shift(1e308), true);
    r.line_box_struts[0].metrics = LineStrut {
        baseline: 1e308,
        line_extent: 1e308,
    };
    let selected = p.select(r).unwrap();
    let error = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap_err();
    assert!(
        matches!(
            error.detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                phase: TextNumericPhase::Finalization,
                category: TextNumericError::NonFinite,
                ..
            })
        ),
        "{error:?}"
    );
}

#[test]
fn allocated_shift_default_font_parent_is_real_and_local_atomic_baseline_is_preserved() {
    qualify();
    let source = Source::new("");
    let mut value = input(&source, own(0.0), LTR);
    value.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::EmptyOrigin(0),
        affinity: Affinity::Before,
    });
    assert_eq!(value.paragraph_strut, ParagraphStrut::DefaultFont);
    let mut system = system(AHEM);
    let p = prepare(&mut system, value, LineOrientation::Horizontal);
    let r = allocated(&p, Row::Additional, shift(3.0), false);
    assert!(r.line_strut.is_none());
    let line = p
        .select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    // Raw Ahem10 gives an actual DefaultFont8/2 parent, not the unused
    // reference origin. max(8,6+3)+max(2,4-3)=11.
    near(line.feedback().baseline, 9.0);
    near(line.feedback().line_height, 11.0);
    let (_system, empty) = fixture(Row::Additional, 0, 1.0, "");
    let mut r = allocated(&empty, Row::Additional, shift(20.0), false);
    r.line_strut = Some(LineStrut {
        baseline: 0.0,
        line_extent: 0.0,
    });
    let line = empty
        .select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    // A supplied zero parent is an actual contribution, unlike absence:
    // max(0,26)+max(0,-16)=26, rather than the sole child's extent10.
    near(line.feedback().baseline, 26.0);
    near(line.feedback().line_height, 26.0);
    let (_system, p) = fixture(Row::Atomic, 0, 1.0, "X");
    let mut r = allocated(&p, Row::Atomic, shift(3.0), true);
    if let AtomicLineAllocation::InFlow { box_geometry, .. } = &mut r.atomics[0].allocation {
        box_geometry.baseline = 2.5;
    }
    let original = r.atomics[0].allocation;
    let line = p
        .select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let slot = line.atomic_slots().next().unwrap();
    assert_eq!(slot.allocation, original);
    near(slot.origin.y, 6.0);
    let physical = AtomicPlacement {
        occurrence: slot.occurrence,
        border: Rect::new(20.0, 91.0, 6.0, 10.0),
        content: Rect::new(21.0, 92.0, 4.0, 8.0),
        baseline: 99.5,
    };
    let placed = Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![physical],
        })],
    })
    .unwrap();
    assert_eq!(
        placed.inline_boxes()[0].rect(),
        Rect::new(20.0, 91.0, 6.0, 10.0)
    );
}
