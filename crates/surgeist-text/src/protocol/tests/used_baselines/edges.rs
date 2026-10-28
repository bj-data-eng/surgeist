//! Genuine RTL and finite/supplied geometry boundaries for named preparation.
use super::*;

fn registered(bytes: Vec<u8>) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}

fn capture(source: Source, style: Style, rtl: bool, forced: bool) -> AnalyzedSource {
    let len = source.text().len();
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: if rtl {
                    ParagraphBaseDirection::RightToLeft
                } else {
                    ParagraphBaseDirection::LeftToRight
                },
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, len),
                    aggregate: Range::new(0, len),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: if forced {
                    vec![crate::ParagraphControl {
                        at: 0,
                        kind: crate::ParagraphControlKind::ForcedBreak,
                    }]
                } else {
                    vec![]
                },
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: if forced {
                    vec![crate::ParagraphAnchor {
                        item: crate::ParagraphAnchorItem::Control(0),
                        affinity: Affinity::Before,
                    }]
                } else {
                    vec![]
                },
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    )
}

#[test]
fn actual_captured_rtl_keeps_odd_level_source_order_and_carets_in_both_named_frames() {
    let original = crate::shaping_test_fonts::sideways_ahem();
    // Existing licensed fixture maps actual Alef to retained X outlines; it
    // provides genuine Hebrew UBA semantics, not Hebrew glyph-design coverage.
    let bytes = sfnt::replace_tables(
        &original,
        vec![(
            *b"BASE",
            base::base(
                0,
                Some(base::Axis {
                    tags: vec![*b"romn"],
                    scripts: [*b"hebr", *b"latn"]
                        .into_iter()
                        .map(|tag| base::Script {
                            tag,
                            default_index: 0,
                            coordinates: vec![base::Coord::Design(100)],
                        })
                        .collect(),
                }),
                None,
                None,
            ),
        )],
    );
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.charmap().map('\u{5d0}'), Some(GlyphId::new(58)));
    assert!(parsed.gsub().is_ok());
    for glyph in [58, 274] {
        assert_eq!(
            parsed
                .glyph_metrics(FontSize::unscaled(), LocationRef::default())
                .advance_width(GlyphId::new(glyph)),
            Some(1000.0)
        );
    }
    let axis = parsed.base().unwrap().horiz_axis().unwrap().unwrap();
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(
        scripts
            .base_script_records()
            .iter()
            .map(|record| {
                let values = record
                    .base_script(scripts.offset_data())
                    .unwrap()
                    .base_values()
                    .unwrap()
                    .unwrap();
                (
                    record.base_script_tag().to_be_bytes(),
                    values.base_coords().get(0).unwrap().coordinate(),
                )
            })
            .collect::<Vec<_>>(),
        [(*b"hebr", 100), (*b"latn", 100)]
    );
    let mut system = registered(bytes);
    let source = Source::identified("\u{5d0}\u{5d0}", Id::from_u64(1561040), 7);
    let input = capture(source.clone(), own_style(), true, false);
    assert_eq!(input.direction(), crate::Direction::RightToLeft);
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        let prepared = system
            .prepare(
                &input,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(ALPHA),
            )
            .unwrap();
        assert_eq!(
            prepared
                .unit_views()
                .map(|unit| (unit.source_range(), unit.level().get()))
                .collect::<Vec<_>>(),
            [(Range::new(0, 1), 1), (Range::new(1, 2), 1)]
        );
        let mut selection = request(&prepared);
        selection.context.orientation = orientation;
        selection.context.base_direction = ParagraphBaseDirection::RightToLeft;
        selection.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
        selection.context.available_inline = Some(20.0);
        selection.context.spacing = SpacingPolicy::None;
        let line = prepared
            .select(selection)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (20.0, 7.0, 10.0)
        );
        let glyph = if orientation == LineOrientation::Horizontal {
            58
        } else {
            274
        };
        assert_eq!(
            line.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| (g.range(), g.id(), g.pen(), g.x(), g.y(), g.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [
                (Range::new(1, 2), glyph, 0.0, 0.0, 0.0, 10.0),
                (Range::new(0, 1), glyph, 10.0, 0.0, 0.0, 10.0)
            ]
        );
        let layout = placed(
            &prepared,
            line,
            if orientation == LineOrientation::Horizontal {
                70.0
            } else {
                100.0
            },
        );
        assert_eq!(layout.source(), &source);
        let run = layout.run_views().next().unwrap();
        let raw = run
            .baseline(system.font_catalogue_identity(), AXIS, ALPHA)
            .unwrap();
        assert_eq!(raw.script().iso15924(), *b"Hebr");
        assert!(
            matches!(raw.outcome(), FontBaselineOutcome::Available(value) if value.coordinate() == 1.0 && value.base_script() == Some(*b"hebr"))
        );
        let (origin, bounds, start, end) = if orientation == LineOrientation::Horizontal {
            (
                Point::new(20.0, 71.0),
                Rect::new(20.0, 63.0, 20.0, 10.0),
                Rect::new(40.0, 63.0, 1.0, 10.0),
                Rect::new(20.0, 63.0, 1.0, 10.0),
            )
        } else {
            (
                Point::new(99.0, 20.0),
                Rect::new(97.0, 20.0, 10.0, 20.0),
                Rect::new(97.0, 40.0, 10.0, 1.0),
                Rect::new(97.0, 20.0, 10.0, 1.0),
            )
        };
        assert_eq!(run.origin(), origin);
        assert_eq!(layout.bounds(), bounds);
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::After))
                .unwrap()
                .rect(),
            start
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(2, Affinity::Before))
                .unwrap()
                .rect(),
            end
        );
        let selected = layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(2, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(selected.rects()[0].rect(), bounds);
        assert_eq!(
            layout.glyph_runs()[0]
                .glyphs()
                .iter()
                .map(|glyph| glyph.range())
                .collect::<Vec<_>>(),
            [Range::new(1, 2), Range::new(0, 1)]
        );
    }
}

fn placement(prepared: &PreparedText, line: &FinalizedLine, baseline: f64) -> PlacementRequest {
    PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline,
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    }
}

#[test]
fn actual_decoration_overflow_refuses_placement_without_changing_a_retained_named_layout() {
    // Actual BASE romn0 still supplies genuine requested Alphabetic authority.
    // A nonzero b beside this enormous half-leading has a separate checked
    // finalization refusal, qualified below rather than bypassed here.
    let bytes = font(0);
    qualify(&bytes, 0);
    let mut system = registered(bytes);
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        // The finite authored offset is relative to the used alignment line.
        // Native Absolute1e308 gives B5e307/H1e308. At anchor9e307 the
        // metric hull4e307..1.4e308 is finite with a positive representable
        // extent. The decoration's9e307+MAX coordinate alone overflows.
        let offset = if orientation == LineOrientation::Horizontal {
            -f64::MAX
        } else {
            f64::MAX
        };
        let input = analyzed(
            Source::identified("A", Id::from_u64(1561041), 2),
            Style {
                line_height: LineHeight::Absolute(1e308),
                underline: crate::Decoration::solid()
                    .with_offset(crate::DecorationOffset::try_absolute(offset).unwrap())
                    .with_thickness(crate::DecorationThickness::try_absolute(1.0).unwrap()),
                ..own_style()
            },
        );
        let prepared = system
            .prepare(
                &input,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(ALPHA),
            )
            .unwrap();
        assert_eq!(prepared.baseline_reference(), Some(ALPHA));
        let line = finalize(&prepared);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (5e307, 1e308)
        );
        let retained =
            crate::Layout::from_finalized_lines(placement(&prepared, &line, 0.0)).unwrap();
        let glyphs = retained.glyph_runs();
        let decorations = retained
            .decoration_views()
            .map(|decoration| decoration.rect())
            .collect::<Vec<_>>();
        assert_eq!(decorations.len(), 1);
        assert_eq!(
            decorations[0],
            if orientation == LineOrientation::Horizontal {
                Rect::new(20.0, f64::MAX, 10.0, 1.0)
            } else {
                Rect::new(f64::MAX, 20.0, 1.0, 10.0)
            }
        );
        let error =
            crate::Layout::from_finalized_lines(placement(&prepared, &line, 9e307)).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric {
            field,
            phase: crate::TextNumericPhase::Placement,
            category: crate::TextNumericError::NonFinite,
            ..
        }) if *field == if orientation == LineOrientation::Horizontal { "decoration y" } else { "decoration x" }),
            "{error:?}"
        );
        assert_eq!(retained.glyph_runs(), glyphs);
        assert_eq!(
            retained
                .decoration_views()
                .map(|decoration| decoration.rect())
                .collect::<Vec<_>>(),
            decorations
        );
        assert_eq!(retained.source().text().code_units(), [65]);
    }
}

#[test]
fn an_actual_nonzero_font_displacement_cannot_vanish_in_an_enormous_native_line() {
    let bytes = font(100);
    qualify(&bytes, 100);
    let mut system = registered(bytes);
    let source = Source::identified("A", Id::from_u64(1561042), 3);
    let safe = analyzed(source.clone(), own_style());
    let huge = analyzed(
        source,
        Style {
            line_height: LineHeight::Absolute(1e308),
            ..own_style()
        },
    );
    // Binary64 spacing here exceeds1. The raw b1 from romn100 is a required
    // nonzero contribution to font zero, independent of the derived leading.
    assert_eq!(5e307_f64 + 1.0, 5e307_f64);
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        let options = PreparationOptions::default()
            .with_orientation(orientation)
            .with_baseline_reference(ALPHA);
        let prepared = system.prepare(&safe, options).unwrap();
        let retained = placed(
            &prepared,
            finalize(&prepared),
            if orientation == LineOrientation::Horizontal {
                70.0
            } else {
                100.0
            },
        );
        let snapshot = retained.glyph_runs();
        let bounds = retained.bounds();
        let huge = system.prepare(&huge, options).unwrap();
        assert_eq!(huge.baseline_reference(), Some(ALPHA));
        let mut selection = request(&huge);
        selection.context.orientation = orientation;
        selection.context.spacing = SpacingPolicy::None;
        let selected = huge.select(selection).unwrap();
        let error = selected.finalize(SpacingAllocations::None).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextNumeric {
                field: "paint font origin",
                phase: crate::TextNumericPhase::Finalization,
                category: crate::TextNumericError::NonZeroLost,
                value: 1.0,
            })
        );
        assert_eq!(retained.glyph_runs(), snapshot);
        assert_eq!(retained.bounds(), bounds);
        assert_eq!(retained.source().text().code_units(), [65]);
    }
}

#[test]
fn signed_native_font_struts_do_not_relax_supplied_line_or_control_strut_containment() {
    let bytes = font(2000);
    qualify(&bytes, 2000);
    let mut system = registered(bytes);
    let input = capture(Source::new("A"), own_style(), false, true);
    for orientation in [LineOrientation::Horizontal, LineOrientation::Sideways] {
        let prepared = system
            .prepare(
                &input,
                PreparationOptions::default()
                    .with_orientation(orientation)
                    .with_baseline_reference(ALPHA),
            )
            .unwrap();
        let forced = prepared
            .opportunity_views()
            .find(|offer| offer.kind() == PreparedBreakKind::Forced)
            .unwrap()
            .handle();
        let mut selection = request(&prepared);
        selection.context.orientation = orientation;
        selection.context.end = LineEnd::Forced;
        selection.end = forced.view().boundary();
        selection.termination = crate::PreparedTermination::Ordinary(forced);
        selection.participants = prepared
            .participants()
            .take(selection.end.view().participant_offset())
            .collect();
        let control = selection
            .participants
            .iter()
            .find_map(|participant| match participant {
                PreparedParticipant::Control(control) => Some(control.clone()),
                _ => None,
            })
            .unwrap();
        let ordinal = prepared
            .control_views()
            .position(|view| view.handle() == control)
            .unwrap();
        assert_eq!(
            control.view().kind(),
            &crate::ParagraphControlKind::ForcedBreak
        );
        selection.controls = vec![ControlAllocation {
            occurrence: control.clone(),
            strut: None,
            boundary: None,
        }];
        let native = prepared
            .select(selection.clone())
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            (native.feedback().baseline, native.feedback().line_height),
            (-12.0, 10.0)
        );
        for baseline in [-1.0, 11.0] {
            let mut external = selection.clone();
            external.line_strut = Some(crate::LineStrut {
                baseline,
                line_extent: 10.0,
            });
            assert_eq!(
                reason(&prepared.select(external).unwrap_err()),
                &ProtocolError::InvalidLineStrut
            );
            let mut external = selection.clone();
            external.controls[0].strut = Some(ControlStrut {
                baseline,
                line_extent: 10.0,
                alignment: LineBoxAlignment::Baseline,
            });
            assert_eq!(
                reason(&prepared.select(external).unwrap_err()),
                &ProtocolError::InvalidControlGeometry {
                    occurrence: ordinal
                }
            );
        }
    }
}
