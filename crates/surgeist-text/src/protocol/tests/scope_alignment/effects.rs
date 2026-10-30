//! New-API effects qualification from real capture, pinned Ahem data and the
//! mathematical H/CW/CCW frames. No historical preimplementation RED is claimed.
use crate::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const X: f64 = 100.25;
const Y: f64 = -20.5;
const SHIFTS: [f64; 4] = [0.0, 3.0, 2.0, 0.0];

fn decorated() -> Style {
    Style {
        underline: Decoration::solid()
            .with_offset(DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(DecorationThickness::try_absolute(1.0).unwrap()),
        ..super::capture::style(10.0, 10.0)
    }
}

fn capture() -> Arc<ProcessedSource> {
    use SourceProcessingEvent as E;
    let mut source = Source::identified("A\u{3a7}\u{3a7}B", Id::from_u64(1_578_701), 29);
    source.span(
        Range::new(0, 1),
        Style {
            brush: Brush::color(1.0, 0.0, 0.0, 1.0),
            ..decorated()
        },
    );
    source.span(
        Range::new(3, 4),
        Style {
            brush: Brush::color(0.0, 0.0, 1.0, 1.0),
            ..decorated()
        },
    );
    let mut input = super::capture::captured_input(2);
    input.scope_alignment_limits = Some(super::capture::generous());
    input.source = source.clone();
    input.origins = (0..4)
        .map(|i| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(i, i + 1),
            aggregate: Range::new(i, i + 1),
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    input.connections = (0..3)
        .map(|i| SourceProcessingConnection {
            before_origin: i,
            after_origin: i + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    input.boundary_policies = (0..3)
        .map(|i| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    for (i, scope) in input.scopes.iter_mut().enumerate() {
        match scope {
            SourceProcessingScope::Block {
                base_style,
                font_reference,
                ..
            } => {
                *base_style = decorated();
                *font_reference = Some(super::capture::reference());
            }
            SourceProcessingScope::Inline {
                style,
                alignment_boundary,
                font_reference,
                ..
            } => {
                assert!(i == 1 || i == 2);
                *style = Some(decorated());
                *alignment_boundary = Some(ScopeAlignmentBoundary::NonInitial);
                let mut reference = super::capture::reference();
                reference.script = FontBaselineScript::from_iso15924(*b"Grek").unwrap();
                *font_reference = Some(reference);
            }
            _ => panic!("fixture contains only the actual block and nested inlines"),
        }
    }
    let origin = |origin| E::Origin {
        origin,
        affinity: Affinity::After,
    };
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::EnterScope(1),
        E::Connection(0),
        origin(1),
        E::EnterScope(2),
        E::Connection(1),
        origin(2),
        E::ExitScope(2),
        E::ExitScope(1),
        E::Connection(2),
        origin(3),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    Arc::new(process_contextual_source(input).unwrap())
}

fn prepare(orientation: LineOrientation) -> (PreparedText, [CapturedScope; 2]) {
    let face = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        (
            face.hhea().unwrap().ascender().to_i16(),
            face.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    assert_eq!(face.charmap().map('\u{3a7}'), Some(GlyphId::new(274)));
    let metrics = face.glyph_metrics(
        skrifa::instance::Size::unscaled(),
        skrifa::instance::LocationRef::default(),
    );
    for glyph in [35, 274, 36] {
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(1000.0));
    }
    let processed = capture();
    let scopes = [
        processed.captured_scope(1).unwrap(),
        processed.captured_scope(2).unwrap(),
    ];
    assert_eq!(scopes[1].parent(), Some(scopes[0].clone()));
    let analysis = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analysis = crate::test_wrapping::bind_missing(&analysis, Wrap::Word);
    let mut fonts = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = fonts
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    for scope in &scopes {
        let receipt = prepared.scope_metrics(scope).unwrap();
        assert_eq!(receipt.effective_instance().unwrap().face().bytes(), AHEM);
        assert_eq!((receipt.font_size(), receipt.line_height()), (10.0, 10.0));
    }
    (prepared, scopes)
}

fn select(prepared: &PreparedText) -> SelectedLine {
    let mut request = super::super::request(prepared);
    request.context.orientation = prepared.orientation();
    request.context.available_inline = None;
    request.context.spacing = SpacingPolicy::None;
    prepared.select(request).unwrap()
}

fn allocations(
    selected: &SelectedLine,
    scopes: &[CapturedScope; 2],
    shifts: [f64; 2],
) -> LineAllocations {
    let rows = selected
        .scopes()
        .map(|scope| {
            let index = scopes
                .iter()
                .position(|actual| *actual == scope.captured_scope())
                .expect("actual selected inline");
            ScopeAllocation {
                scope,
                displacement: BaselineShift::try_new(shifts[index]).unwrap(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::Complete(rows),
    }
}

fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    let orientation = prepared.orientation();
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            orientation,
            atomics: vec![],
            anchor: if orientation == LineOrientation::Horizontal {
                LineAnchor {
                    inline_origin: X,
                    baseline: Y,
                }
            } else {
                LineAnchor {
                    inline_origin: Y,
                    baseline: X,
                }
            },
        })],
    })
    .unwrap()
}

// Ahem's original cell is8 over/2 under. Preserve that10px cell before
// sibling expansion; the accepted union becomes11/2, not13px per character.
fn cell(orientation: LineOrientation, i: usize, shift: f64) -> Rect {
    let u = i as f64 * 10.0;
    match orientation {
        LineOrientation::Horizontal => Rect::new(X + u, Y - shift - 8.0, 10.0, 10.0),
        LineOrientation::SidewaysClockwise => Rect::new(X + shift - 2.0, Y + u, 10.0, 10.0),
        LineOrientation::SidewaysCounterclockwise => {
            Rect::new(X - shift - 8.0, Y - u - 10.0, 10.0, 10.0)
        }
        LineOrientation::Vertical => panic!("this fixture has horizontal typography"),
    }
}
fn paint_origin(orientation: LineOrientation, i: usize, shift: f64) -> Point {
    let u = i as f64 * 10.0;
    match orientation {
        LineOrientation::Horizontal => Point::new(X + u, Y - shift),
        LineOrientation::SidewaysClockwise => Point::new(X + shift, Y + u),
        LineOrientation::SidewaysCounterclockwise => Point::new(X - shift, Y - u),
        LineOrientation::Vertical => unreachable!(),
    }
}

#[test]
fn nested_real_scopes_move_paint_source_carets_selection_and_decoration_once_in_each_native_frame()
{
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
        LineOrientation::SidewaysCounterclockwise,
    ] {
        let (prepared, scopes) = prepare(orientation);
        let selected = select(&prepared);
        let line = selected
            .finalize(allocations(&selected, &scopes, [3.0, -1.0]))
            .unwrap();
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (40.0, 11.0, 13.0)
        );
        let layout = place(&prepared, line);
        assert_eq!(layout.source().text().code_units(), [65, 0x3a7, 0x3a7, 66]);
        assert_eq!(
            (layout.source().id(), layout.source().revision()),
            (Some(Id::from_u64(1_578_701)), 29)
        );
        let runs = layout.run_views().collect::<Vec<_>>();
        for (i, shift) in SHIFTS.into_iter().enumerate() {
            let range = Range::new(i, i + 1);
            let run = runs
                .iter()
                .find(|run| run.range() == range)
                .expect("actual source run in its executed frame");
            assert_eq!(run.origin(), paint_origin(orientation, i, shift));
            assert_eq!(run.face().bytes(), AHEM);
            let raw = run.glyphs().next().unwrap();
            assert_eq!(
                (raw.id(), raw.pen(), raw.x(), raw.y(), raw.advance()),
                ([35, 274, 274, 36][i], 0.0, 0.0, 0.0, 10.0)
            );
            let origin = prepared
                .origin_views()
                .find(|o| o.aggregate_range() == range)
                .unwrap()
                .handle();
            let query = layout.source_range_fragments(&origin, range).unwrap();
            let unit = query.units().next().unwrap();
            let piece = unit.pieces().next().unwrap();
            assert_eq!(piece.rect(), cell(orientation, i, shift));
            assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
            let point = layout
                .positions()
                .find(|p| p.source_cursor() == Cursor::new(i, Affinity::After))
                .unwrap();
            let expected = cell(orientation, i, shift);
            let caret = match orientation {
                LineOrientation::Horizontal => {
                    Rect::new(expected.origin.x, expected.origin.y, 1.0, 10.0)
                }
                LineOrientation::SidewaysClockwise => {
                    Rect::new(expected.origin.x, expected.origin.y, 10.0, 1.0)
                }
                LineOrientation::SidewaysCounterclockwise => {
                    Rect::new(expected.origin.x, expected.origin.y + 9.0, 10.0, 1.0)
                }
                LineOrientation::Vertical => unreachable!(),
            };
            assert_eq!(layout.position_geometry(&point).unwrap().rect(), caret);
            let u = i as f64 * 10.0 + 2.0;
            let hit_point = match orientation {
                LineOrientation::Horizontal => Point::new(X + u, Y - shift - 3.0),
                LineOrientation::SidewaysClockwise => Point::new(X + shift + 3.0, Y + u),
                LineOrientation::SidewaysCounterclockwise => Point::new(X - shift - 3.0, Y - u),
                LineOrientation::Vertical => unreachable!(),
            };
            let Hit::Text(hit) = layout.hit(hit_point) else {
                panic!("actual shifted source cell must be hit-testable")
            };
            assert_eq!(hit.source_cursor(), Cursor::new(i, Affinity::After));
            assert_eq!(layout.position_geometry(&hit).unwrap().rect(), caret);
        }
        let selection = layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(4, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(
            selection
                .rects()
                .iter()
                .map(|r| r.rect())
                .collect::<Vec<_>>(),
            SHIFTS
                .into_iter()
                .enumerate()
                .map(|(i, s)| cell(orientation, i, s))
                .collect::<Vec<_>>()
        );
        let expected = SHIFTS
            .into_iter()
            .enumerate()
            .map(|(i, shift)| {
                let origin = paint_origin(orientation, i, shift);
                match orientation {
                    LineOrientation::Horizontal => Rect::new(origin.x, origin.y + 2.0, 10.0, 1.0),
                    LineOrientation::SidewaysClockwise => {
                        Rect::new(origin.x - 3.0, origin.y, 1.0, 10.0)
                    }
                    LineOrientation::SidewaysCounterclockwise => {
                        Rect::new(origin.x + 2.0, origin.y - 10.0, 1.0, 10.0)
                    }
                    LineOrientation::Vertical => unreachable!(),
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            layout
                .decoration_views()
                .map(|d| d.rect())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn explicit_noninitial_zero_preserves_the_same_selected_raw_glyph_basis_and_source_cells() {
    let (prepared, scopes) = prepare(LineOrientation::Horizontal);
    let selected = select(&prepared);
    let zero = selected
        .finalize(allocations(&selected, &scopes, [0.0, 0.0]))
        .unwrap();
    let shifted = selected
        .finalize(allocations(&selected, &scopes, [3.0, -1.0]))
        .unwrap();
    let packet = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|r| {
                r.glyphs()
                    .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(packet(&zero), packet(&shifted));
    assert_eq!(zero.selected_line(), shifted.selected_line());
    assert_eq!(
        (zero.feedback().baseline, zero.feedback().line_height),
        (8.0, 10.0)
    );
    let layout = place(&prepared, zero);
    for (i, run) in layout.run_views().enumerate() {
        assert_eq!(run.origin(), Point::new(X + i as f64 * 10.0, Y));
    }
    assert_eq!(layout.source().text().code_units(), [65, 0x3a7, 0x3a7, 66]);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accessibility_uses_the_same_nested_source_cells_and_applies_host_origin_once() {
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
        LineOrientation::SidewaysCounterclockwise,
    ] {
        let (prepared, scopes) = prepare(orientation);
        let selected = select(&prepared);
        let line = selected
            .finalize(allocations(&selected, &scopes, [3.0, -1.0]))
            .unwrap();
        let layout = place(&prepared, line);
        let mut state = Accessibility::default();
        let parent = accesskit::NodeId(1_578_702);
        let host = Point::new(7.0, -9.0);
        let update = layout
            .accessibility_update(&mut state, parent, parent, host)
            .unwrap();
        for (i, shift) in SHIFTS.into_iter().enumerate() {
            let position = layout
                .access_position(&state, Cursor::new(i, Affinity::After))
                .unwrap()
                .unwrap();
            assert_eq!(position.character_index, 0);
            let node = &update
                .nodes
                .iter()
                .find(|(id, _)| *id == position.node)
                .unwrap()
                .1;
            assert_eq!(node.role(), accesskit::Role::TextRun);
            assert_eq!(node.value(), Some(["A", "\u{3a7}", "\u{3a7}", "B"][i]));
            assert_eq!(node.character_widths(), Some(&[10.0][..]));
            let direction = match orientation {
                LineOrientation::Horizontal => accesskit::TextDirection::LeftToRight,
                LineOrientation::SidewaysClockwise => accesskit::TextDirection::TopToBottom,
                LineOrientation::SidewaysCounterclockwise => accesskit::TextDirection::BottomToTop,
                LineOrientation::Vertical => unreachable!(),
            };
            assert_eq!(node.text_direction(), Some(direction));
            let expected = cell(orientation, i, shift);
            let actual = node.bounds().unwrap();
            assert_eq!(
                (actual.x0, actual.y0, actual.x1, actual.y1),
                (
                    expected.origin.x + host.x,
                    expected.origin.y + host.y,
                    expected.origin.x + host.x + expected.size.width,
                    expected.origin.y + host.y + expected.size.height
                )
            );
        }
        assert_eq!(layout.source().text().code_units(), [65, 0x3a7, 0x3a7, 66]);
    }
}

const EDGE_ATOMICS: [Id; 5] = [
    Id::from_u64(1_578_721),
    Id::from_u64(1_578_722),
    Id::from_u64(1_578_723),
    Id::from_u64(1_578_724),
    Id::from_u64(1_578_725),
];

fn edge_leaves(orientation: LineOrientation) -> (PreparedText, CapturedScope) {
    use SourceProcessingEvent as E;
    let mut source = Source::identified("AB", Id::from_u64(1_578_720), 37);
    source.span(
        Range::new(0, 1),
        Style {
            brush: Brush::color(1.0, 0.0, 0.0, 1.0),
            ..decorated()
        },
    );
    source.span(
        Range::new(1, 2),
        Style {
            brush: Brush::color(0.0, 0.0, 1.0, 1.0),
            ..decorated()
        },
    );
    for (i, id) in EDGE_ATOMICS.into_iter().enumerate() {
        let size = if i == 4 {
            Size::new(9.0, 7.0)
        } else if orientation == LineOrientation::Horizontal {
            Size::new(4.0, 5.0)
        } else {
            Size::new(5.0, 4.0)
        };
        source.inline_box(InlineBox::new(
            id,
            if i == 4 {
                InlineBoxKind::OutOfFlow
            } else {
                InlineBoxKind::InFlow
            },
            2,
            size,
        ));
    }
    let mut input = super::capture::captured_input(1);
    input.scope_alignment_limits = Some(super::capture::generous());
    input.source = source.clone();
    input.origins = (0..2)
        .map(|i| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(i, i + 1),
            aggregate: Range::new(i, i + 1),
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![ParagraphBoundaryPolicy {
        definition: ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }];
    input.boundary_policies.extend(
        (0..5)
            .flat_map(|i| {
                [
                    ParagraphBoundaryDefinition::AtomicBefore(i),
                    ParagraphBoundaryDefinition::AtomicAfter(i),
                ]
            })
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }),
    );
    let SourceProcessingScope::Block {
        base_style,
        font_reference,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!()
    };
    *base_style = decorated();
    *font_reference = Some(super::capture::reference());
    let SourceProcessingScope::Inline {
        style,
        alignment_boundary,
        font_reference,
        ..
    } = &mut input.scopes[1]
    else {
        unreachable!()
    };
    *style = Some(decorated());
    *alignment_boundary = Some(ScopeAlignmentBoundary::NonInitial);
    *font_reference = Some(super::capture::reference());
    // These are four genuine empty child inline boxes, each with its own
    // captured entry/exit controls. They request native control geometry, not
    // independent whole-scope allocations or a proxy for the parent's text.
    for child in 0..4 {
        input.scopes.push(SourceProcessingScope::Inline {
            trim: Default::default(),
            style: Some(decorated()),
            boundaries: Some(SourceProcessingScopeBoundaries {
                start: child * 2,
                end: child * 2 + 1,
            }),
            alignment_boundary: None,
            font_reference: None,
        });
        input.controls.extend([
            ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ]);
    }
    let origin = |origin| E::Origin {
        origin,
        affinity: Affinity::After,
    };
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::EnterScope(1),
        E::Connection(0),
        origin(1),
    ];
    input.events.extend((0..5).map(|atomic| E::Atomic {
        atomic,
        affinity: Affinity::After,
    }));
    for child in 0..4 {
        input.events.extend([
            E::EnterScope(child + 2),
            E::Control {
                control: child * 2,
                affinity: Affinity::After,
            },
            E::Control {
                control: child * 2 + 1,
                affinity: Affinity::After,
            },
            E::ExitScope(child + 2),
        ]);
    }
    input
        .events
        .extend([E::ExitScope(1), E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let parent = processed.captured_scope(1).unwrap();
    assert!(parent.entry_control().is_none());
    assert!(parent.exit_control().is_none());
    for i in 2..6 {
        let child = processed.captured_scope(i).unwrap();
        assert_eq!(child.parent(), Some(parent.clone()));
        assert_eq!(
            child.entry_control(),
            processed.captured_control((i - 2) * 2)
        );
        assert_eq!(
            child.exit_control(),
            processed.captured_control((i - 2) * 2 + 1)
        );
    }
    let analysis = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analysis = crate::test_wrapping::bind_missing(&analysis, Wrap::Word);
    let mut fonts = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = fonts
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    (prepared, parent)
}

fn edge_request(prepared: &PreparedText) -> LineSelectionRequest {
    let mut request = super::super::request(prepared);
    request.context.orientation = prepared.orientation();
    request.context.spacing = SpacingPolicy::None;
    request.context.available_inline = None;
    // A genuine explicit containing-line allocation fixes both edges for every
    // tested parent shift. Neither the8/2 glyphs nor the5px children can grow it.
    request.line_strut = Some(LineStrut {
        baseline: 20.0,
        line_extent: 40.0,
    });
    request.atomics = prepared
        .atomic_views()
        .map(|view| {
            let i = EDGE_ATOMICS
                .iter()
                .position(|id| *id == view.original_box().id())
                .unwrap();
            let allocation = if i == 4 {
                AtomicLineAllocation::OutOfFlow
            } else {
                AtomicLineAllocation::InFlow {
                    flow: AtomicFlowAllocation {
                        advance: 4.0,
                        before_baseline: 3.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 4.0,
                    },
                    box_geometry: AtomicBoxAllocation {
                        border: Rect::new(0.0, -3.0, 4.0, 5.0),
                        content: Rect::new(1.0, -2.0, 2.0, 3.0),
                        baseline: 0.0,
                    },
                    alignment: [
                        LineBoxAlignment::Baseline,
                        LineBoxAlignment::Shift(BaselineShift::try_new(1.0).unwrap()),
                        LineBoxAlignment::Top,
                        LineBoxAlignment::Bottom,
                    ][i],
                }
            };
            AtomicAllocation {
                occurrence: view.handle(),
                allocation,
            }
        })
        .collect();
    request.controls = prepared
        .control_views()
        .enumerate()
        .map(|(i, view)| ControlAllocation {
            occurrence: view.handle(),
            boundary: Some(InlineBoundaryAllocation::ZERO),
            strut: Some(ControlStrut {
                baseline: 8.0,
                line_extent: 10.0,
                alignment: [
                    LineBoxAlignment::Top,
                    LineBoxAlignment::Bottom,
                    LineBoxAlignment::Baseline,
                    LineBoxAlignment::Shift(BaselineShift::try_new(1.0).unwrap()),
                ][i / 2],
            }),
        })
        .collect();
    while let Some(occurrence) = prepared.boundary_context_demand(&request).unwrap() {
        request
            .boundary_context
            .push(InlineBoundaryContextAllocation {
                occurrence,
                boundary: InlineBoundaryAllocation::ZERO,
            });
    }
    request
}

// Explicit mathematical map with B20. This does not read finalized positions.
fn edge_point(orientation: LineOrientation, u: f64, v: f64) -> Point {
    match orientation {
        LineOrientation::Horizontal => Point::new(X + u, Y + v - 20.0),
        LineOrientation::SidewaysClockwise => Point::new(X + 20.0 - v, Y + u),
        LineOrientation::SidewaysCounterclockwise => Point::new(X + v - 20.0, Y - u),
        LineOrientation::Vertical => unreachable!(),
    }
}
fn edge_rect(orientation: LineOrientation, u: f64, v: f64, w: f64, h: f64) -> Rect {
    match orientation {
        LineOrientation::Horizontal => Rect::new(X + u, Y + v - 20.0, w, h),
        LineOrientation::SidewaysClockwise => Rect::new(X + 20.0 - v - h, Y + u, h, w),
        LineOrientation::SidewaysCounterclockwise => Rect::new(X + v - 20.0, Y - u - w, h, w),
        LineOrientation::Vertical => unreachable!(),
    }
}

#[test]
fn genuine_top_bottom_atomic_and_control_leaves_escape_parent_shift_in_one_fixed_accepted_line() {
    // CSS2§10.8.1, pinned visudet lines894–900: Top/Bottom children are excluded
    // from their parent's aligned subtree. Native baseline/Shift leaves remain
    // in that subtree. The same40px accepted line makes the difference visible.
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
        LineOrientation::SidewaysCounterclockwise,
    ] {
        let (prepared, parent) = edge_leaves(orientation);
        let controls = prepared
            .control_views()
            .map(|v| v.handle())
            .collect::<Vec<_>>();
        let selected = prepared.select(edge_request(&prepared)).unwrap();
        assert_eq!(
            selected
                .scopes()
                .map(|s| s.captured_scope())
                .collect::<Vec<_>>()
                .as_slice(),
            std::slice::from_ref(&parent)
        );
        for shift in [-3.0, 0.0, 3.0] {
            let line = selected
                .finalize(LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                        scope: selected.scopes().next().unwrap(),
                        displacement: BaselineShift::try_new(shift).unwrap(),
                    }]),
                })
                .unwrap();
            assert_eq!(line.selected_line(), selected);
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (36.0, 20.0, 40.0)
            );
            assert_eq!(
                line.selected_line().request().line_strut,
                Some(LineStrut {
                    baseline: 20.0,
                    line_extent: 40.0
                })
            );
            for slot in line.atomic_slots() {
                let i = EDGE_ATOMICS
                    .iter()
                    .position(|id| *id == slot.occurrence.view().original_box().id())
                    .unwrap();
                let expected = [20.0 - shift, 19.0 - shift, 3.0, 38.0, 20.0 - shift][i];
                assert_eq!(
                    slot.origin,
                    Point::new(if i == 4 { 36.0 } else { 20.0 + 4.0 * i as f64 }, expected)
                );
                if i == 2 {
                    assert_eq!(
                        slot.origin.y - 3.0,
                        0.0,
                        "Top border stays on the accepted over edge"
                    );
                }
                if i == 3 {
                    assert_eq!(
                        slot.origin.y + 2.0,
                        40.0,
                        "Bottom border stays on the accepted under edge"
                    );
                }
            }
            for (i, control) in controls.iter().enumerate() {
                let slot = line
                    .control_slots()
                    .find(|slot| slot.control() == *control)
                    .unwrap();
                assert_eq!(
                    slot.slot(),
                    Point::new(36.0, [8.0, 38.0, 20.0 - shift, 19.0 - shift][i / 2])
                );
                if i / 2 == 0 {
                    assert_eq!(slot.slot().y - 8.0, 0.0);
                }
                if i / 2 == 1 {
                    assert_eq!(slot.slot().y + 2.0, 40.0);
                }
            }
            let atomics = prepared
                .atomic_views()
                .map(|view| {
                    let i = EDGE_ATOMICS
                        .iter()
                        .position(|id| *id == view.original_box().id())
                        .unwrap();
                    let (border, content, baseline) = if i == 4 {
                        // Text observes the real inherited OOF static slot above.
                        // Its positioned-box owner independently supplies this box;
                        // no Layout inset/static-position behavior is invented here.
                        (
                            Rect::new(300.25, 500.5, 9.0, 7.0),
                            Rect::new(301.25, 501.5, 7.0, 5.0),
                            if orientation == LineOrientation::Horizontal {
                                506.5
                            } else {
                                303.25
                            },
                        )
                    } else {
                        let u = 20.0 + 4.0 * i as f64;
                        let v = [20.0 - shift, 19.0 - shift, 3.0, 38.0][i];
                        let point = edge_point(orientation, u, v);
                        (
                            edge_rect(orientation, u, v - 3.0, 4.0, 5.0),
                            edge_rect(orientation, u + 1.0, v - 2.0, 2.0, 3.0),
                            if orientation == LineOrientation::Horizontal {
                                point.y
                            } else {
                                point.x
                            },
                        )
                    };
                    AtomicPlacement {
                        occurrence: view.handle(),
                        border,
                        content,
                        baseline,
                    }
                })
                .collect();
            let layout = Layout::from_finalized_lines(PlacementRequest {
                prepared: prepared.clone(),
                coverage: PlacementCoverage::WholeParagraph,
                entries: vec![PlacementEntry::Formatted(LinePlacement {
                    line,
                    orientation,
                    atomics,
                    anchor: if orientation == LineOrientation::Horizontal {
                        LineAnchor {
                            inline_origin: X,
                            baseline: Y,
                        }
                    } else {
                        LineAnchor {
                            inline_origin: Y,
                            baseline: X,
                        }
                    },
                })],
            })
            .unwrap();
            assert_eq!(layout.source().text().code_units(), [65, 66]);
            assert_eq!(
                (layout.source().id(), layout.source().revision()),
                (Some(Id::from_u64(1_578_720)), 37)
            );
            assert_eq!(
                layout.lines()[0].bounds(),
                edge_rect(orientation, 0.0, 0.0, 36.0, 40.0)
            );
            let boxes = layout.inline_boxes();
            assert_eq!(
                boxes
                    .iter()
                    .find(|b| b.id() == EDGE_ATOMICS[2])
                    .unwrap()
                    .rect(),
                edge_rect(orientation, 28.0, 0.0, 4.0, 5.0)
            );
            assert_eq!(
                boxes
                    .iter()
                    .find(|b| b.id() == EDGE_ATOMICS[3])
                    .unwrap()
                    .rect(),
                edge_rect(orientation, 32.0, 35.0, 4.0, 5.0)
            );
            assert_eq!(
                boxes
                    .iter()
                    .find(|b| b.id() == EDGE_ATOMICS[4])
                    .unwrap()
                    .rect(),
                Rect::new(300.25, 500.5, 9.0, 7.0)
            );
            let original = prepared
                .origin_views()
                .find(|v| v.aggregate_range() == Range::new(1, 2))
                .unwrap()
                .handle();
            let query = layout
                .source_range_fragments(&original, Range::new(1, 2))
                .unwrap();
            let unit = query.units().next().unwrap();
            assert_eq!(
                unit.pieces().next().unwrap().rect(),
                cell(orientation, 1, shift)
            );
            let run = layout
                .run_views()
                .find(|run| run.range() == Range::new(1, 2))
                .unwrap();
            assert_eq!(run.origin(), paint_origin(orientation, 1, shift));
            assert_eq!(
                run.glyphs()
                    .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                    .collect::<Vec<_>>(),
                [(36, 0.0, 0.0, 0.0, 10.0)]
            );
            let glyph = paint_origin(orientation, 1, shift);
            let expected_underline = match orientation {
                LineOrientation::Horizontal => Rect::new(glyph.x, glyph.y + 2.0, 10.0, 1.0),
                LineOrientation::SidewaysClockwise => Rect::new(glyph.x - 3.0, glyph.y, 1.0, 10.0),
                LineOrientation::SidewaysCounterclockwise => {
                    Rect::new(glyph.x + 2.0, glyph.y - 10.0, 1.0, 10.0)
                }
                LineOrientation::Vertical => unreachable!(),
            };
            assert!(
                layout
                    .decoration_views()
                    .any(|d| d.rect() == expected_underline)
            );
        }
    }
}
