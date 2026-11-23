//! Early computed mandatory incidence through actual placement and retained paint.

use super::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct ComputedTree {
    inputs: [layout::node::LayoutInput; 3],
    non_box: layout::node::NodeInput,
}

impl layout::tree::Traverse for ComputedTree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 { 1..3 } else { 0..0 }
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 2 } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl layout::tree::LayoutTree for ComputedTree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::node::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::node::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn glyph_rows(
    placed: &text::layout::Layout,
) -> Vec<(usize, text::source::Range, u32, f32, f64, f64)> {
    placed
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.face().bytes(), AHEM);
            assert_eq!(instance.font_size(), 10.0);
            assert_eq!(instance.axis_count(), 0);
            assert_eq!(instance.normalized_coords().count(), 0);
            assert_eq!((instance.x_shear(), instance.embolden()), (0.0, 0.0));
            let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
            assert_eq!(metrics.units_per_em(), 1000);
            assert_eq!(
                (metrics.ascent(), metrics.descent(), metrics.leading()),
                (8.0, -2.0, 0.0)
            );
            let origin = run.origin();
            let line = run.line();
            let range = run.range();
            run.glyphs()
                .map(|glyph| {
                    (
                        line,
                        range,
                        glyph.id(),
                        glyph.advance(),
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y + f64::from(glyph.y()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn computed_scope_before_keeps_original_receiving_caret_and_two_painted_lines() {
    assert_computed_scope_before(
        layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
        None,
    );
}

#[test]
fn demanded_box_fragment_survives_computed_break_placement_and_retained_paint() {
    assert_computed_scope_before(
        layout::text_layout::InlineBoxFragmentationDemand::CompleteBodies,
        None,
    );
}

#[test]
fn demanded_family_counts_borrowed_backing_and_respects_capture_limits() {
    assert_computed_scope_before(
        layout::text_layout::InlineBoxFragmentationDemand::CompleteBodies,
        Some(text::scope_alignment::ScopeAlignmentLimits::new(
            100_000,
            64,
            10_000_000,
            16 << 20,
        )),
    );
}

fn assert_computed_scope_before(
    demand: layout::text_layout::InlineBoxFragmentationDemand,
    limits: Option<text::scope_alignment::ScopeAlignmentLimits>,
) {
    use text::capture::SourceProcessingEvent as E;
    let aggregate = text::source::Source::identified("AA", text::source::Id::from_u64(650_810), 7);
    let originals = [
        text::source::Source::identified("A", text::source::Id::from_u64(650_811), 11),
        text::source::Source::identified("A", text::source::Id::from_u64(650_812), 13),
    ];
    let base = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        white_space: text::style::WhiteSpace::Preserve,
        wrap: text::style::Wrap::None,
        ..text::style::Style::default()
    };
    let processed = Arc::new(
        text::capture::process_contextual_source(text::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: limits,
            computed_line_boundaries: vec![text::capture::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::capture::SourceBoxSide::Before,
            }],
            source: aggregate.clone(),
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::capture::SourceProcessingEnvelope::Complete,
            origins: originals
                .iter()
                .enumerate()
                .map(|(index, source)| text::paragraph::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::source::Range::new(0, 1),
                    aggregate: text::source::Range::new(index, index + 1),
                    kind: text::paragraph::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![text::capture::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: text::paragraph::ParagraphConnectionKind::Continuous,
            }],
            controls: vec![],
            boundary_policies: vec![text::paragraph::ParagraphBoundaryPolicy {
                definition: text::paragraph::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: text::style::Wrap::None,
                following_wrap: text::style::Wrap::None,
                ancestor_wrap: text::style::Wrap::None,
            }],
            replacements: vec![],
            scopes: vec![
                text::capture::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: text::capture::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: base,
                    used_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                },
                text::capture::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![text::capture::SourceProcessingParagraph {
                base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::layout::Affinity::After,
                },
                E::Connection(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 1,
                    affinity: text::layout::Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.processed_text().code_units(), [65, 65]);
    let scope = processed.captured_scope(1).unwrap();
    assert!(scope.entry_control().is_none());
    assert!(scope.exit_control().is_none());
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        cause.target(),
        text::capture::CapturedComputedLineBoundaryTarget::Scope {
            scope,
            side: text::capture::SourceBoxSide::Before,
        }
    );
    let analyzed = text::analyzed::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: text::style::Wrap::None,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = text::System::default();
    system
        .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::prepared::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.control_views().count(), 0);
    if limits.is_some() {
        // Existing capture limits already fund this actual preparation. Family
        // admission must count its additional owners without charging borrowed
        // caller stack headers or changing the preparation's retained state.
        let prepared_retention = prepared.scope_alignment_retention().unwrap();
        let allowance = text::scope_alignment::ScopeAlignmentAllowance::new(100_000, 16 << 20);
        let products = text::scope_alignment::ScopeAlignmentProducts::Mixed(&[]);
        let baseline = text::scope_alignment::scope_alignment_census(
            text::scope_alignment::ScopeAlignmentCensusInput::Product(
                text::scope_alignment::ScopeAlignmentProduct::Prepared(&prepared),
            ),
            products,
            allowance,
        )
        .unwrap()
        .input_retention();
        let borrowed_census = |native_boxes: &Vec<text::inline_box::InlineBoxSource>| {
            text::scope_alignment::scope_alignment_census(
                text::scope_alignment::ScopeAlignmentCensusInput::InlineBoxFamily {
                    prepared: &prepared,
                    native_boxes,
                },
                products,
                allowance,
            )
            .unwrap()
            .input_retention()
        };
        // Caller-owned stack headers are outside feature payload. A borrowed
        // empty vector owns no backing; reservation contributes its real
        // backing capacity even before any native declaration is initialized.
        assert_eq!(borrowed_census(&Vec::new()), baseline);
        let reserved = Vec::<text::inline_box::InlineBoxSource>::with_capacity(3);
        let expected_backing = reserved
            .capacity()
            .checked_mul(std::mem::size_of::<text::inline_box::InlineBoxSource>())
            .unwrap();
        assert_eq!(borrowed_census(&reserved).records(), baseline.records());
        assert_eq!(
            borrowed_census(&reserved).payload_bytes(),
            baseline
                .payload_bytes()
                .checked_add(expected_backing)
                .unwrap(),
        );
        let refused = prepared
            .inline_box_family(
                vec![],
                Some(text::scope_alignment::ScopeAlignmentAllowance::new(0, 0)),
            )
            .unwrap_err();
        assert_eq!(refused.code, text::ErrorCode::ScopeAlignmentRefused);
        assert!(matches!(
            refused.detail(),
            Some(text::ErrorDetail::ScopeAlignment {
                reason: text::scope_alignment::ScopeAlignmentError::Limit { required, limit: 0, .. }
            }) if *required > 0
        ));
        let family = prepared.inline_box_family(vec![], None).unwrap();
        let retained = family.scope_alignment_retention().unwrap();
        assert!(retained.records() > 0);
        assert!(retained.payload_bytes() > 0);
        let family_input = text::scope_alignment::ScopeAlignmentCensusInput::Product(
            text::scope_alignment::ScopeAlignmentProduct::InlineBoxFamily(&family),
        );
        let single =
            text::scope_alignment::scope_alignment_census(family_input, products, allowance)
                .unwrap();
        let alias = family.clone();
        let related = [
            text::scope_alignment::ScopeAlignmentProduct::InlineBoxFamily(&alias),
            text::scope_alignment::ScopeAlignmentProduct::Prepared(&prepared),
        ];
        let shared = text::scope_alignment::scope_alignment_census(
            family_input,
            text::scope_alignment::ScopeAlignmentProducts::Mixed(&related),
            allowance,
        )
        .unwrap();
        assert_eq!(shared.input_retention(), single.input_retention());
        assert_eq!(
            shared.external_retention(),
            text::scope_alignment::ScopeAlignmentRetention::default()
        );
        let usage = family.scope_alignment_usage().unwrap();
        assert!(usage.payload_bytes() >= retained.payload_bytes());
        assert!(usage.peak_payload_bytes() >= usage.payload_bytes());
        assert_eq!(
            prepared.scope_alignment_retention(),
            Some(prepared_retention)
        );
    }
    let origins = prepared
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(origins.len(), 2);
    for (index, (origin, original)) in origins.iter().zip(&originals).enumerate() {
        assert_eq!(origin.view().source(), original);
        assert_eq!(
            original.id(),
            Some(text::source::Id::from_u64(650_811 + index as u64))
        );
        assert_eq!(original.revision(), [11, 13][index]);
    }
    let tree = ComputedTree {
        inputs: [
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                size: layout::geometry::Size::new(
                    layout::sizing::PreferredSizeOf::value(
                        layout::length::LengthPercentageOf::px(40.0).unwrap(),
                    ),
                    layout::sizing::PreferredSize::AUTO,
                ),
                ..layout::node::NodeInput::DEFAULT
            }),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                origins[0].clone(),
            )),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                origins[1].clone(),
            )),
        ],
        non_box: layout::node::NodeInput::non_box(),
    };
    let flow = layout::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::text_layout::TextParagraphBinding {
            inline_box_fragments: demand,
            prepared: prepared.clone(),
            container: 0,
            nodes: vec![1, 2],
            origins: origins
                .iter()
                .enumerate()
                .map(|(index, origin)| layout::text_layout::TextOriginBinding {
                    origin: origin.clone(),
                    node: index + 1,
                })
                .collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: vec![],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &flow,
        0,
        NativeViewport::new(
            layout::geometry::Size::new(Some(40.0), None),
            layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::HorizontalTb,
                layout::geometry::Direction::Ltr,
            ),
        )
        .request(layout::geometry::Size::new(
            layout::constraint::SizingOperation::Normal,
            layout::constraint::SizingOperation::Normal,
        )),
    )
    .unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(batch.placed_text()[0].nodes(), &[1, 2]);
    let placed_output = &batch.placed_text()[0];
    match demand {
        layout::text_layout::InlineBoxFragmentationDemand::NotRequested => {
            assert!(placed_output.inline_box_fragments().is_none());
        }
        layout::text_layout::InlineBoxFragmentationDemand::CompleteBodies => {
            let fragments = placed_output.inline_box_fragments().unwrap();
            assert_eq!(fragments.family().prepared_text(), prepared);
            assert_eq!(
                fragments.family().scope_alignment_retention().is_some(),
                limits.is_some()
            );
            let owner =
                text::inline_box::InlineBoxSource::captured(processed.captured_scope(1).unwrap());
            let pieces = fragments
                .pieces()
                .filter(|piece| piece.fragment().owner() == owner)
                .collect::<Vec<_>>();
            // Only the receiving A belongs to the captured Inline.
            // The mandatory cause puts it on line one; neither an edge control
            // nor decoration is needed to retain that actual box's identity.
            assert_eq!(pieces.len(), 1);
            assert_eq!(pieces[0].line_index(), 1);
            assert_eq!(pieces[0].fragment().owner(), owner);
        }
    }
    let placed = placed_output.layout();
    let retained_box_fragments = placed
        .inline_box_pieces()
        .map(|piece| (piece.fragment(), piece.line_index()))
        .collect::<Vec<_>>();
    assert_eq!(placed.source(), &aggregate);
    assert_eq!(
        placed.source().id(),
        Some(text::source::Id::from_u64(650_810))
    );
    assert_eq!(placed.source().revision(), 7);
    // Retained Ahem A: advance1em, ascent0.8em, descent0.2em. An early
    // mandatory cause starts the receiver despite ample width and WrapNone.
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| (line.range(), line.baseline(), line.bounds()))
            .collect::<Vec<_>>(),
        [
            (
                text::source::Range::new(0, 1),
                8.0,
                text::geometry::Rect::new(0.0, 0.0, 40.0, 10.0)
            ),
            (
                text::source::Range::new(1, 2),
                18.0,
                text::geometry::Rect::new(0.0, 10.0, 40.0, 10.0)
            ),
        ]
    );
    let expected_glyphs = [
        (0, text::source::Range::new(0, 1), 35, 10.0, 0.0, 8.0),
        (1, text::source::Range::new(1, 2), 35, 10.0, 0.0, 18.0),
    ];
    assert_eq!(glyph_rows(placed), expected_glyphs);
    let finalized = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .collect::<Vec<_>>();
    assert_eq!(finalized.len(), 2);
    for line in &finalized {
        assert_eq!(line.prepared_text(), prepared);
    }
    assert_eq!(
        finalized
            .iter()
            .map(|line| line.selected_line().request().context.end)
            .collect::<Vec<_>>(),
        [
            text::selection::LineEnd::Forced,
            text::selection::LineEnd::Paragraph
        ]
    );
    let selected_line = finalized[0].selected_line();
    let text::selection::PreparedTermination::Ordinary(offer) =
        &selected_line.request().termination
    else {
        panic!("real issued forced offer")
    };
    assert_eq!(
        offer.view().kind(),
        text::prepared::PreparedBreakKind::Forced
    );
    assert_eq!(
        offer
            .view()
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [cause]
    );
    let endpoint = |offset, affinity| {
        placed
            .positions()
            .find(|position| {
                position.source_cursor() == text::layout::Cursor::new(offset, affinity)
            })
            .unwrap()
    };
    let selected = placed
        .position_selection(
            &endpoint(0, text::layout::Affinity::After),
            &endpoint(2, text::layout::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selected.source_text().code_units(), [65, 65]);
    assert_eq!(selected.plaintext().code_units(), [65, 65]);
    assert_eq!(selected.source_pieces().len(), 2);
    for (index, piece) in selected.source_pieces().iter().enumerate() {
        assert_eq!(piece.origin(), &origins[index]);
        assert_eq!(piece.origin().view().source(), &originals[index]);
        assert_eq!(piece.local_range(), text::source::Range::new(0, 1));
        assert_eq!(
            piece.aggregate_range(),
            text::source::Range::new(index, index + 1)
        );
        assert_eq!(piece.code_units(), [65]);
    }
    let cursor = text::layout::Cursor::new(2, text::layout::Affinity::Before);
    let caret = placed.cursor(cursor).unwrap().rect();
    assert_eq!(caret.origin, text::geometry::Point::new(10.0, 10.0));
    assert_eq!(caret.size.height, 10.0);
    drop(system);
    let paint = render::text_layout::prepare_text_layout(
        placed,
        render::geometry::Transform::identity(),
        render::text_layout::TextLayoutLimits::new(
            render::text::GlyphRunLimits::new(8, 1024, 8 << 20, 8 << 20),
            8,
            8,
            0,
        ),
        &mut surgeist_render::accounting::RenderAccountingContext::default(),
    )
    .unwrap();
    assert_eq!(paint.scene().len(), 2);
    let summary = paint
        .scene()
        .inspect(&mut render::accounting::RenderAccountingContext::default())
        .unwrap();
    assert_eq!((summary.commands, summary.glyphs), (2, 2));
    assert_eq!(paint.layout().lines(), placed.lines());
    assert_eq!(
        paint
            .layout()
            .inline_box_pieces()
            .map(|piece| (piece.fragment(), piece.line_index()))
            .collect::<Vec<_>>(),
        retained_box_fragments
    );
    assert_eq!(glyph_rows(paint.layout()), expected_glyphs);
    assert_eq!(paint.layout().cursor(cursor).unwrap().rect(), caret);
    assert_eq!(paint.layout().source(), &aggregate);
    #[cfg(feature = "text-accessibility")]
    {
        let mut accessibility = text::accessibility::Accessibility::new(10);
        let parent = 1000_u64.into();
        let update = paint
            .layout()
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::geometry::Point::new(100.0, 200.0),
            )
            .unwrap();
        let position = paint
            .layout()
            .access_position(&accessibility, cursor)
            .unwrap()
            .unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == position.node)
            .unwrap()
            .1;
        assert_eq!(node.value(), Some("A"));
        assert_eq!(node.font_size(), Some(10.0));
        assert_eq!(position.character_index, 1);
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (100.0, 210.0, 110.0, 220.0)
        );
    }
}
