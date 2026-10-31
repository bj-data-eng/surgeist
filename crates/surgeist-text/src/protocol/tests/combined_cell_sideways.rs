//! Sideways Cells retain genuine vertical Source custody and horizontal payloads.
//! The current existing-callable RED is public prepare's Cell orientation gate.
use super::*;
use skrifa::raw::TableProvider;

const CW: LineOrientation = LineOrientation::SidewaysClockwise;
const CCW: LineOrientation = LineOrientation::SidewaysCounterclockwise;

fn horizontal_reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    }
}

fn context() -> ContainerContext {
    ContainerContext {
        typography: AnalysisTypography::HorizontalComposition,
        direction: ParagraphBaseDirection::LeftToRight,
        writing_axis: SourceWritingAxis::Vertical,
        root_reference: horizontal_reference(),
        container_reference: Some(horizontal_reference()),
        combination: TextCombineUpright::All,
        alignment: None,
    }
}

fn options(orientation: LineOrientation, reference: Option<FontBaseline>) -> PreparationOptions {
    let options = PreparationOptions::default().with_orientation(orientation);
    let options = reference.map_or(options, |name| options.with_baseline_reference(name));
    assert_eq!(options.requested_baseline_reference(), reference);
    options
}

fn select(prepared: &PreparedText) -> SelectedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let rtl = prepared.analyzed_source().direction() == Direction::RightToLeft;
    prepared
        .select(LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                alignment_insets: LineAlignmentInsets::ZERO,
                formatted_line: FormattedLineContext::default(),
                base_direction: if rtl {
                    ParagraphBaseDirection::RightToLeft
                } else {
                    ParagraphBaseDirection::LeftToRight
                },
                line_box_direction: if rtl {
                    LineBoxDirection::RightToLeft
                } else {
                    LineBoxDirection::LeftToRight
                },
                orientation: prepared.orientation(),
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
}

fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn font_premises(instance: EffectiveFontInstance<'_>) {
    let face = instance.face();
    let bytes = face.bytes();
    let parsed = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(parsed.hhea().unwrap().descender().to_i16(), -200);
    assert_eq!(parsed.os2().unwrap().fs_selection().bits() & 0x80, 0);
    assert!(parsed.fvar().is_err());
    assert!(parsed.mvar().is_err());
    assert_eq!(instance.axis_count(), 0);
    assert_eq!(instance.font_size(), 10.0);
    let metrics = instance.metrics(face.catalogue_identity()).unwrap();
    assert_eq!(metrics.line_metric_source(), FontLineMetricSource::Hhea);
    assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
    let baseline = instance
        .baseline(
            face.catalogue_identity(),
            horizontal_reference().script,
            FontBaselineAxis::HorizontalY,
            FontBaseline::Alphabetic,
        )
        .unwrap();
    let FontBaselineOutcome::Available(value) = baseline.outcome() else {
        panic!("the actual horizontal Alphabetic fallback must be available");
    };
    assert_eq!(value.coordinate(), 0.0);
    assert_eq!(
        value.provenance(),
        FontBaselineProvenance::HorizontalAlphabeticZero
    );
}

fn ordinary_controls(orientation: LineOrientation) {
    // The genuine vertical capture has None combination, so ordinary sideways
    // typography already succeeds before the Cell admission assertion is made.
    let mut ordinary = fixture(
        ContainerShape::Inline,
        "A",
        ContainerContext {
            combination: TextCombineUpright::None,
            ..context()
        },
    );
    assert_eq!(ordinary.analyzed.combined_cells().len(), 0);
    for reference in [None, Some(FontBaseline::Alphabetic)] {
        let prepared = ordinary
            .system
            .prepare(&ordinary.analyzed, options(orientation, reference))
            .unwrap();
        assert_eq!(prepared.cell_views().len(), 0);
        let line = select(&prepared)
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().advance, 10.0);
        let runs = line.run_views().collect::<Vec<_>>();
        let [run] = runs.as_slice() else {
            panic!("one actual ordinary A run");
        };
        font_premises(run.effective_instance().unwrap());
        assert_eq!(run.face().bytes(), ordinary.body.as_slice());
        assert_eq!(
            run.glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>(),
            [(35, 10.0)]
        );
        assert_eq!(
            run.glyph_basis(),
            if orientation == CW {
                GlyphBasis::Clockwise
            } else {
                GlyphBasis::Counterclockwise
            }
        );
    }
    // All is ineffective under a genuinely horizontal source writing axis.
    let mut horizontal = fixture(
        ContainerShape::Inline,
        "A",
        ContainerContext {
            writing_axis: SourceWritingAxis::Horizontal,
            ..context()
        },
    );
    assert_eq!(horizontal.analyzed.combined_cells().len(), 0);
    horizontal
        .system
        .prepare(&horizontal.analyzed, options(orientation, None))
        .unwrap();
}

fn interaction(
    layout: &Layout,
    processed: &ProcessedSource,
    square: Rect,
    orientation: LineOrientation,
    empty: bool,
) {
    let start = Cursor::new(0, Affinity::After);
    let end = Cursor::new(1, Affinity::Before);
    let positions = layout.positions().collect::<Vec<_>>();
    assert_eq!(
        positions
            .iter()
            .map(PlacedPosition::source_cursor)
            .collect::<Vec<_>>(),
        [start, end]
    );
    let (start_y, end_y, near_start, near_end) = if orientation == CW {
        (20.0, 30.0, 21.0, 29.0)
    } else {
        // A negative inline unit maps [0,1] to [19,20], not [20,21].
        (19.0, 9.0, 19.0, 11.0)
    };
    for (position, cursor, y) in [(&positions[0], start, start_y), (&positions[1], end, end_y)] {
        let expected = Rect::new(square.origin.x, y, 10.0, 1.0);
        assert_eq!(layout.position_geometry(position).unwrap().rect(), expected);
        assert_eq!(layout.cursor(cursor).unwrap().rect(), expected);
    }
    for (y, position) in [(near_start, &positions[0]), (near_end, &positions[1])] {
        let Hit::Text(actual) = layout.hit(Point::new(square.origin.x + 5.0, y)) else {
            panic!("actual Cell square is text hit geometry");
        };
        assert_eq!(&actual, position);
    }
    assert_eq!(
        layout
            .move_position(&positions[0], Movement::NextCluster)
            .unwrap(),
        positions[1]
    );
    assert_eq!(
        layout.move_cursor(start, Movement::NextCluster).unwrap(),
        end
    );
    assert_eq!(
        layout
            .move_position(&positions[1], Movement::PreviousCluster)
            .unwrap(),
        positions[0]
    );
    let selection = layout
        .position_selection(&positions[0], &positions[1])
        .unwrap();
    assert_eq!(
        selection.geometry(),
        &SelectionGeometry::new(vec![SelectionRect::new(square, 0)])
    );
    assert_eq!(selection.source_text(), processed.source().text());
    assert_eq!(
        selection.plaintext(),
        &Text::from(if empty { "" } else { "A" })
    );
    let [piece] = selection.source_pieces() else {
        panic!("one genuine original occurrence");
    };
    assert_eq!(piece.local_range(), Range::new(0, 1));
    assert_eq!(piece.aggregate_range(), Range::new(0, 1));
    assert_eq!(
        piece.origin().view().capture_position(),
        Some(processed.captured_origin(0).unwrap())
    );
    let range_error = layout
        .source_range_fragments(piece.origin(), Range::new(0, 1))
        .unwrap_err();
    assert_eq!(range_error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(
        range_error.detail(),
        Some(ErrorDetail::SourceRangeFragments {
            reason: SourceRangeFragmentError::UnsupportedCell { line: 0, cell: 0 }
        })
    ));
}

fn cell_case(
    orientation: LineOrientation,
    shape: ContainerShape,
    empty: bool,
    named: Option<FontBaseline>,
    displacement: Option<f64>,
    direction: ParagraphBaseDirection,
) {
    let mut fixture = fixture(
        shape,
        if empty { " " } else { "A" },
        ContainerContext {
            direction,
            alignment: displacement.map(|_| ScopeAlignmentBoundary::NonInitial),
            ..context()
        },
    );
    source_custody(&fixture, empty);
    assert_eq!(
        fixture.analyzed.direction(),
        if direction == ParagraphBaseDirection::RightToLeft {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        }
    );
    assert_eq!(
        fixture
            .analyzed
            .combined_cells()
            .next()
            .unwrap()
            .inner_direction(),
        ParagraphBaseDirection::LeftToRight
    );
    let prepared = fixture.system.prepare(&fixture.analyzed, options(orientation, named))
        .expect("a genuine vertical Source Cell with matching horizontal typography must admit sideways preparation");
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine prepared Cell");
    };
    assert_eq!(
        (cell.extent(), cell.before(), cell.after()),
        (10.0, 8.0, 2.0)
    );
    assert_eq!(cell.has_provider_input(), !empty);
    let container = fixture.processed.captured_scope(fixture.container).unwrap();
    let metric = prepared.scope_metrics(&container).unwrap();
    assert_eq!(metric.captured_scope(), container);
    assert_eq!(metric.font_reference(), horizontal_reference());
    assert_eq!(metric.orientation(), orientation);
    assert_eq!(metric.common_baseline_reference(), named);
    assert_eq!(metric.native_reference_coordinate(), 0.0);
    assert_eq!((metric.over(), metric.under()), (8.0, 2.0));
    assert_eq!(metric.font_size(), 10.0);
    assert_eq!(
        metric.effective_instance().unwrap().face().bytes(),
        fixture.body.as_slice()
    );
    font_premises(metric.effective_instance().unwrap());
    let selected = select(&prepared);
    let scope_rows = selected.scopes().collect::<Vec<_>>();
    let scopes = if let Some(d) = displacement {
        let [scope] = scope_rows.as_slice() else {
            panic!("one actual NonInitial Inline target");
        };
        assert_eq!(
            scope.captured_scope(),
            fixture.processed.captured_scope(fixture.container).unwrap()
        );
        assert_eq!(scope.metrics().font_size(), 10.0);
        assert_eq!(scope.parent_metrics().unwrap().font_size(), 20.0);
        ScopeAllocations::Complete(vec![ScopeAllocation {
            scope: scope.clone(),
            displacement: BaselineShift::try_new(d).unwrap(),
        }])
    } else {
        assert!(scope_rows.is_empty());
        ScopeAllocations::None
    };
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    let d = displacement.unwrap_or(0.0);
    let expected = if orientation == CW {
        Rect::new(98.0 + d, 20.0, 10.0, 10.0)
    } else {
        Rect::new(92.0 - d, 10.0, 10.0, 10.0)
    };
    let verify = |line: FinalizedLine| {
        assert_eq!(line.feedback().advance, 10.0);
        if direction == ParagraphBaseDirection::RightToLeft {
            // One upright-L Cell in a base-level1 paragraph resolves to even
            // level2. Its issued source edges and explicit inner LTR packet
            // are not reversed by the outer paragraph or CCW physical map.
            assert_eq!(line.cell_views().next().unwrap().level().get(), 2);
        }
        assert_eq!(
            line.cell_views().next().unwrap().scope_displacement().get(),
            d
        );
        if empty {
            assert!(line.run_views().all(|run| run.cell().is_none()));
        }
        let layout = place(&prepared, line);
        assert_eq!(layout.bounds(), expected);
        assert_eq!(layout.lines()[0].bounds(), expected);
        assert_eq!(
            layout.analyzed_source().identity(),
            fixture.analyzed.identity()
        );
        assert_eq!(layout.source(), fixture.processed.source());
        let runs = layout
            .run_views()
            .filter(|run| run.glyphs().next().is_some())
            .collect::<Vec<_>>();
        if empty {
            assert!(runs.is_empty());
            assert!(
                layout
                    .glyph_runs()
                    .iter()
                    .all(|run| run.glyphs().is_empty())
            );
        } else {
            let [run] = runs.as_slice() else {
                panic!("one actual horizontal inner packet");
            };
            assert_eq!(run.face().bytes(), fixture.body.as_slice());
            assert_eq!(run.orientation(), orientation);
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            let frame = run.glyph_frame();
            assert_eq!(
                frame.origin(),
                Point::new(expected.origin.x, expected.origin.y + 8.0)
            );
            assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
            assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
            assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
            assert_eq!(
                run.glyphs()
                    .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                    .collect::<Vec<_>>(),
                [(35, 0.0, 0.0, 0.0, 10.0)]
            );
            let owned = layout.glyph_runs();
            assert_eq!(owned.len(), 1);
            assert_eq!(owned[0].glyph_frame(), frame);
            assert_eq!(owned[0].glyph_basis(), GlyphBasis::Unrotated);
        }
        interaction(&layout, &fixture.processed, expected, orientation, empty);
        #[cfg(feature = "text-accessibility")]
        {
            use accesskit::NodeId;
            let mut state = Accessibility::new(49_000);
            let parent = NodeId(1);
            fixture
                .analyzed
                .accessibility_update(&mut state, parent, parent)
                .unwrap();
            let start = Cursor::new(0, Affinity::After);
            let before = fixture.analyzed.access_position(&state, start).unwrap();
            assert!(before.is_some());
            let error = layout
                .accessibility_update(&mut state, parent, parent, Point::new(0.0, 0.0))
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::UnsupportedAccessibilityCell { line: 0, cell: 0 })
            );
            assert_eq!(
                fixture.analyzed.access_position(&state, start).unwrap(),
                before
            );
        }
    };
    verify(line.clone());
    drop(fixture.system);
    verify(line.with_spacing(SpacingAllocations::None).unwrap());
}

#[test]
fn clockwise_cell_preparation_retains_horizontal_parent_metrics_and_frame() {
    ordinary_controls(CW);
    cell_case(
        CW,
        ContainerShape::Inline,
        false,
        None,
        None,
        ParagraphBaseDirection::LeftToRight,
    );
    cell_case(
        CW,
        ContainerShape::Inline,
        false,
        Some(FontBaseline::Alphabetic),
        None,
        ParagraphBaseDirection::LeftToRight,
    );
}

#[test]
fn counterclockwise_cell_preparation_retains_horizontal_parent_metrics_and_frame() {
    ordinary_controls(CCW);
    cell_case(
        CCW,
        ContainerShape::Inline,
        false,
        None,
        None,
        ParagraphBaseDirection::LeftToRight,
    );
    cell_case(
        CCW,
        ContainerShape::Inline,
        false,
        Some(FontBaseline::Alphabetic),
        None,
        ParagraphBaseDirection::LeftToRight,
    );
}

#[test]
fn sideways_derived_root_and_nearest_computed_inline_keep_their_real_metric_owner() {
    for orientation in [CW, CCW] {
        for shape in [
            ContainerShape::RootInline,
            ContainerShape::NestedComputedAll,
        ] {
            cell_case(
                orientation,
                shape,
                false,
                None,
                None,
                ParagraphBaseDirection::LeftToRight,
            );
        }
    }
}

#[test]
fn sideways_provider_empty_cell_keeps_the_real_square_source_and_two_outer_edges() {
    for orientation in [CW, CCW] {
        cell_case(
            orientation,
            ContainerShape::Inline,
            true,
            None,
            None,
            ParagraphBaseDirection::LeftToRight,
        );
    }
}

#[test]
fn sideways_noninitial_inline_displacement_moves_the_cell_once_in_the_signed_cross_axis() {
    for orientation in [CW, CCW] {
        for d in [-3.0, 3.0] {
            cell_case(
                orientation,
                ContainerShape::Inline,
                false,
                None,
                Some(d),
                ParagraphBaseDirection::LeftToRight,
            );
        }
    }
}

fn protocol_refusal(error: Error, expected: ProtocolError) {
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol { reason: expected })
    );
}

#[test]
fn sideways_typography_and_baseline_mismatches_keep_their_existing_protocol_owners() {
    for orientation in [CW, CCW] {
        let mut mixed = fixture(
            ContainerShape::Inline,
            "A",
            ContainerContext {
                typography: AnalysisTypography::Mixed,
                ..context()
            },
        );
        source_custody(&mixed, false);
        protocol_refusal(
            mixed
                .system
                .prepare(&mixed.analyzed, options(orientation, None))
                .unwrap_err(),
            ProtocolError::MismatchedContext {
                field: ProtocolContextField::Orientation,
            },
        );
        let mut named = fixture(ContainerShape::Inline, "A", context());
        protocol_refusal(
            named
                .system
                .prepare(
                    &named.analyzed,
                    options(orientation, Some(FontBaseline::Central)),
                )
                .unwrap_err(),
            ProtocolError::UnsupportedContext {
                field: ProtocolContextField::BaselineReference,
            },
        );
    }
    let mut horizontal = fixture(ContainerShape::Inline, "A", context());
    protocol_refusal(
        horizontal
            .system
            .prepare(
                &horizontal.analyzed,
                options(LineOrientation::Vertical, None),
            )
            .unwrap_err(),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation,
        },
    );
    let error = horizontal
        .system
        .prepare(
            &horizontal.analyzed,
            options(LineOrientation::Horizontal, None),
        )
        .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::TextCombine {
            reason: TextCombineError::UnsupportedCellPreparation
        })
    ));
}

#[test]
fn sideways_cell_requires_its_exact_container_reference_and_matching_axis() {
    for orientation in [CW, CCW] {
        for reference in [None, Some(font_reference())] {
            let mut fixture = fixture(
                ContainerShape::Inline,
                "A",
                ContainerContext {
                    container_reference: reference,
                    ..context()
                },
            );
            source_custody(&fixture, false);
            let error = fixture
                .system
                .prepare(&fixture.analyzed, options(orientation, None))
                .unwrap_err();
            let expected = if reference.is_none() {
                ScopeAlignmentError::MissingFontReference { scope: 1 }
            } else {
                ScopeAlignmentError::UnsupportedReference { scope: 1 }
            };
            assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::ScopeAlignment { reason: expected })
            );
        }
    }
}

#[test]
fn rtl_sideways_cells_keep_their_upright_l_role_and_explicit_horizontal_inner_direction() {
    for orientation in [CW, CCW] {
        cell_case(
            orientation,
            ContainerShape::Inline,
            false,
            None,
            None,
            ParagraphBaseDirection::RightToLeft,
        );
    }
}
