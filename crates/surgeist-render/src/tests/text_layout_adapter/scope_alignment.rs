//! Real captured nested scopes through the actual immutable Scene adapter.
//! Literal Ahem tables and mathematical line frames own the expectations.
use super::*;
use kurbo::Shape as _;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};
use std::sync::Arc;

const X: f64 = 100.25;
const Y: f64 = -20.5;
const SHIFTS: [f64; 4] = [0.0, 3.0, 2.0, 0.0];

fn reference(script: [u8; 4]) -> text::ScopeFontReference {
    text::ScopeFontReference {
        script: text::FontBaselineScript::from_iso15924(script).unwrap(),
        axis: text::FontBaselineAxis::HorizontalY,
        baseline: text::FontBaseline::Alphabetic,
    }
}
fn authored(effects: bool) -> text::Style {
    text::Style {
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        underline: if effects {
            text::Decoration::solid()
                .with_offset(text::DecorationOffset::try_absolute(-2.0).unwrap())
                .with_thickness(text::DecorationThickness::try_absolute(1.0).unwrap())
        } else {
            text::Decoration::none()
        },
        ..style()
    }
}
fn captured(effects: bool) -> Arc<text::ProcessedSource> {
    use text::SourceProcessingEvent as E;
    let mut source =
        text::Source::identified("A\u{3a7}\u{3a7}B", text::Id::from_u64(1_578_711), 31);
    // Distinct real outside paint brushes make their independent scene packets
    // observable; the two black chi packets require different executed frames.
    source.span(
        text::Range::new(0, 1),
        text::Style {
            brush: text::Brush::color(1.0, 0.0, 0.0, 1.0),
            ..authored(effects)
        },
    );
    source.span(
        text::Range::new(3, 4),
        text::Style {
            brush: text::Brush::color(0.0, 0.0, 1.0, 1.0),
            ..authored(effects)
        },
    );
    let origin = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: Some(text::ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            computed_line_boundaries: vec![],
            origins: (0..4)
                .map(|i| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(i, i + 1),
                    aggregate: text::Range::new(i, i + 1),
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..3)
                .map(|i| text::SourceProcessingConnection {
                    before_origin: i,
                    after_origin: i + 1,
                    kind: text::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..3)
                .map(|i| text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(i),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: authored(effects),
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                    font_reference: Some(reference(*b"Latn")),
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: Default::default(),
                    style: Some(authored(effects)),
                    boundaries: None,
                    alignment_boundary: Some(text::ScopeAlignmentBoundary::NonInitial),
                    font_reference: Some(reference(*b"Grek")),
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: Default::default(),
                    style: Some(authored(effects)),
                    boundaries: None,
                    alignment_boundary: Some(text::ScopeAlignmentBoundary::NonInitial),
                    font_reference: Some(reference(*b"Grek")),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
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
            ],
        })
        .unwrap(),
    )
}

fn native(orientation: text::LineOrientation, effects: bool) -> text::Layout {
    let face = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    let gm = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (ch, glyph, bounds) in [
        ('A', 35, [0.0, -200.0, 1000.0, 800.0]),
        ('\u{3a7}', 274, [0.0, 400.0, 1000.0, 600.0]),
        ('B', 36, [0.0, -200.0, 1000.0, 800.0]),
    ] {
        assert_eq!(face.charmap().map(ch), Some(GlyphId::new(glyph)));
        assert_eq!(gm.advance_width(GlyphId::new(glyph)), Some(1000.0));
        let b = gm.bounds(GlyphId::new(glyph)).unwrap();
        assert_eq!([b.x_min, b.y_min, b.x_max, b.y_max], bounds);
    }
    let processed = captured(effects);
    let outer = processed.captured_scope(1).unwrap();
    let inner = processed.captured_scope(2).unwrap();
    assert_eq!(inner.parent(), Some(outer.clone()));
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| text::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: text::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut fonts = system(AHEM.to_vec());
    let prepared = fonts
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    let terminal = prepared
        .opportunity_views()
        .find(|o| o.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let selected = prepared
        .select(text::LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end: terminal.view().boundary(),
            termination: text::PreparedTermination::Ordinary(terminal),
            participants: prepared.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Left,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap();
    let scopes = selected
        .scopes()
        .map(|scope| {
            let displacement = if scope.captured_scope() == outer {
                3.0
            } else {
                assert_eq!(scope.captured_scope(), inner);
                -1.0
            };
            assert_eq!(
                scope.metrics().effective_instance().unwrap().face().bytes(),
                AHEM
            );
            text::ScopeAllocation {
                scope,
                displacement: text::BaselineShift::try_new(displacement).unwrap(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 2);
    let line = selected
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::Complete(scopes),
        })
        .unwrap();
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (40.0, 11.0, 13.0)
    );
    // The real preparation and all selected scope resources remain usable after
    // dropping the System; the Scene adapter has no font-query owner to borrow.
    drop(fonts);
    text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            orientation,
            atomics: vec![],
            anchor: if orientation == text::LineOrientation::Horizontal {
                text::LineAnchor {
                    inline_origin: X,
                    baseline: Y,
                }
            } else {
                text::LineAnchor {
                    inline_origin: Y,
                    baseline: X,
                }
            },
        })],
    })
    .unwrap()
}

fn origin(orientation: text::LineOrientation, i: usize) -> (f64, f64) {
    let u = i as f64 * 10.0;
    let shift = SHIFTS[i];
    match orientation {
        text::LineOrientation::Horizontal => (X + u, Y - shift),
        text::LineOrientation::SidewaysClockwise => (X + shift, Y + u),
        text::LineOrientation::SidewaysCounterclockwise => (X - shift, Y - u),
        text::LineOrientation::Vertical => unreachable!(),
    }
}
fn physical_ink(run: &PreparedGlyphRun, transform: Transform) -> kurbo::Rect {
    let mut path = run.path().clone();
    path.apply_affine(kurbo::Affine::new(transform.as_array()));
    path.bounding_box()
}

#[test]
fn actual_nested_scope_glyphs_keep_their_instance_and_once_applied_ink_frame_in_h_cw_and_ccw() {
    for orientation in [
        text::LineOrientation::Horizontal,
        text::LineOrientation::SidewaysClockwise,
        text::LineOrientation::SidewaysCounterclockwise,
    ] {
        let input = native(orientation, true);
        assert_eq!(input.source().text().code_units(), [65, 0x3a7, 0x3a7, 66]);
        for (sx, sy, tx, ty) in [(1.0, 1.0, 0.0, 0.0), (2.0, 3.0, 7.0, 9.0)] {
            let placement = Transform::try_new([sx, 0.0, 0.0, sy, tx, ty]).unwrap();
            let scene = prepare_text_layout(&input, placement, limits()).unwrap();
            assert!(std::ptr::eq(scene.layout(), &input));
            let actual = scene
                .scene()
                .commands
                .iter()
                .filter_map(|command| match command {
                    Command::PreparedGlyphRun { run, transform, .. } => Some((run, *transform)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(
                actual.len(),
                4,
                "different actual brushes or executed frames cannot share a paint packet"
            );
            for (i, (run, transform)) in actual.into_iter().enumerate() {
                let (gx, gy) = origin(orientation, i);
                let basis = match orientation {
                    text::LineOrientation::Horizontal => {
                        [sx, 0.0, 0.0, sy, sx * gx + tx, sy * gy + ty]
                    }
                    text::LineOrientation::SidewaysClockwise => {
                        [0.0, sy, -sx, 0.0, sx * gx + tx, sy * gy + ty]
                    }
                    text::LineOrientation::SidewaysCounterclockwise => {
                        [0.0, -sy, sx, 0.0, sx * gx + tx, sy * gy + ty]
                    }
                    text::LineOrientation::Vertical => unreachable!(),
                };
                assert_eq!(transform.as_array(), basis);
                assert_eq!(run.font().bytes(), AHEM);
                assert_eq!(run.instance().size(), 10.0);
                assert!(run.instance().coordinates().is_empty());
                assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
                assert_eq!(
                    run.glyphs(),
                    [TextGlyph::try_new([35, 274, 274, 36][i], 0.0, 0.0, 10.0).unwrap()]
                );
                let (top, bottom) = if i == 1 || i == 2 {
                    (-6.0, -4.0)
                } else {
                    (-8.0, 2.0)
                };
                assert_eq!(
                    run.bounds().ink_rect(),
                    Some(Rect::try_new(0.0, top, 10.0, bottom - top).unwrap())
                );
                // Independent corner geometry from pinned glyf bounds, never
                // advance/line box substituted for the narrow chi's ink.
                let (left, upper, right, lower) = match orientation {
                    text::LineOrientation::Horizontal => (gx, gy + top, gx + 10.0, gy + bottom),
                    text::LineOrientation::SidewaysClockwise => {
                        (gx - bottom, gy, gx - top, gy + 10.0)
                    }
                    text::LineOrientation::SidewaysCounterclockwise => {
                        (gx + top, gy - 10.0, gx + bottom, gy)
                    }
                    text::LineOrientation::Vertical => unreachable!(),
                };
                assert_eq!(
                    physical_ink(run, transform),
                    kurbo::Rect::new(
                        sx * left + tx,
                        sy * upper + ty,
                        sx * right + tx,
                        sy * lower + ty
                    )
                );
            }
            let decorations = if sx == 1.0 && sy == 1.0 && tx == 0.0 && ty == 0.0 {
                scene
                    .scene()
                    .commands
                    .iter()
                    .filter_map(|command| match command {
                        Command::Fill { shape, .. } => Some(shape),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            } else {
                let (layer, children) = scene
                    .scene()
                    .commands
                    .iter()
                    .find_map(|command| match command {
                        Command::Layer { layer, children } => Some((layer, children)),
                        _ => None,
                    })
                    .expect("caller placement wraps the already physical decorations once");
                assert_eq!(layer.transform().as_array(), [sx, 0.0, 0.0, sy, tx, ty]);
                children
                    .iter()
                    .map(|command| {
                        let Command::Fill { shape, .. } = command else {
                            panic!("one physical decoration fill")
                        };
                        shape
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(decorations.len(), 4);
            for (i, shape) in decorations.into_iter().enumerate() {
                let (gx, gy) = origin(orientation, i);
                let rect = match orientation {
                    text::LineOrientation::Horizontal => {
                        Rect::try_new(gx, gy + 2.0, 10.0, 1.0).unwrap()
                    }
                    text::LineOrientation::SidewaysClockwise => {
                        Rect::try_new(gx - 3.0, gy, 1.0, 10.0).unwrap()
                    }
                    text::LineOrientation::SidewaysCounterclockwise => {
                        Rect::try_new(gx + 2.0, gy - 10.0, 1.0, 10.0).unwrap()
                    }
                    text::LineOrientation::Vertical => unreachable!(),
                };
                assert_eq!(*shape, crate::Shape::rect(rect));
            }
        }
    }
}

#[test]
fn scoped_scene_numeric_and_budget_failures_publish_no_prefix_and_keep_prior_preparation() {
    // Decoration bounds are qualified during preflight, before glyph ink.
    // Author this genuine input without effects to reach glyph placement.
    let input = native(text::LineOrientation::SidewaysCounterclockwise, false);
    assert_eq!(input.decoration_views().count(), 0);
    let accepted = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let retained = accepted.scene().clone();
    let error = refuse(
        &input,
        Transform::identity(),
        bounded(0, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&error, TextLayoutResource::Glyphs, 1, 0);
    let error = refuse(
        &input,
        Transform::translation(1e30, 1e30).unwrap(),
        limits(),
    );
    let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 0 });
    assert!(matches!(
        cause.glyph_preparation_diagnostic(),
        Some(GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::Placement,
            ..
        })
    ));
    let before = retained.commands.iter().filter_map(|c| match c {
        Command::PreparedGlyphRun { run, .. } => Some(run),
        _ => None,
    });
    let after = accepted.scene().commands.iter().filter_map(|c| match c {
        Command::PreparedGlyphRun { run, .. } => Some(run),
        _ => None,
    });
    for (before, after) in before.zip(after) {
        assert!(before.is_same_preparation(after));
    }
    assert_eq!(input.source().text().code_units(), [65, 0x3a7, 0x3a7, 66]);
    assert!(prepare_text_layout(&input, Transform::identity(), limits()).is_ok());
}
