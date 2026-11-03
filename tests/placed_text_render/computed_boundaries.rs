//! Early computed mandatory incidence through actual placement and retained paint.

use super::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct ComputedTree {
    inputs: [layout::LayoutInput; 3],
    non_box: layout::NodeInput,
}

impl layout::Traverse for ComputedTree {
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

impl layout::LayoutTree for ComputedTree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn glyph_rows(placed: &text::Layout) -> Vec<(usize, text::Range, u32, f32, f64, f64)> {
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
    assert_computed_scope_before(layout::InlineBoxFragmentationDemand::NotRequested, None);
}

#[test]
fn demanded_box_fragment_survives_computed_break_placement_and_retained_paint() {
    assert_computed_scope_before(layout::InlineBoxFragmentationDemand::CompleteBodies, None);
}

#[test]
fn demanded_family_counts_borrowed_backing_and_respects_capture_limits() {
    assert_computed_scope_before(
        layout::InlineBoxFragmentationDemand::CompleteBodies,
        Some(text::ScopeAlignmentLimits::new(
            100_000,
            64,
            10_000_000,
            16 << 20,
        )),
    );
}

fn assert_computed_scope_before(
    demand: layout::InlineBoxFragmentationDemand,
    limits: Option<text::ScopeAlignmentLimits>,
) {
    use text::SourceProcessingEvent as E;
    let aggregate = text::Source::identified("AA", text::Id::from_u64(650_810), 7);
    let originals = [
        text::Source::identified("A", text::Id::from_u64(650_811), 11),
        text::Source::identified("A", text::Id::from_u64(650_812), 13),
    ];
    let base = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::None,
        ..text::Style::default()
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: limits,
            computed_line_boundaries: vec![text::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::SourceBoxSide::Before,
            }],
            source: aggregate.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: originals
                .iter()
                .enumerate()
                .map(|(index, source)| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(0, 1),
                    aggregate: text::Range::new(index, index + 1),
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![text::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: text::ParagraphConnectionKind::Continuous,
            }],
            controls: vec![],
            boundary_policies: vec![text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: text::Wrap::None,
                following_wrap: text::Wrap::None,
                ancestor_wrap: text::Wrap::None,
            }],
            replacements: vec![],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: base,
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::Connection(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 1,
                    affinity: text::Affinity::After,
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
        text::CapturedComputedLineBoundaryTarget::Scope {
            scope,
            side: text::SourceBoxSide::Before,
        }
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: text::Wrap::None,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.control_views().count(), 0);
    if limits.is_some() {
        // Existing capture limits already fund this actual preparation. Family
        // admission must count its additional owners without charging borrowed
        // caller stack headers or changing the preparation's retained state.
        let prepared_retention = prepared.scope_alignment_retention().unwrap();
        let allowance = text::ScopeAlignmentAllowance::new(100_000, 16 << 20);
        let products = text::ScopeAlignmentProducts::Mixed(&[]);
        let baseline = text::scope_alignment_census(
            text::ScopeAlignmentCensusInput::Product(text::ScopeAlignmentProduct::Prepared(
                &prepared,
            )),
            products,
            allowance,
        )
        .unwrap()
        .input_retention();
        let borrowed_census = |native_boxes: &Vec<text::InlineBoxSource>| {
            text::scope_alignment_census(
                text::ScopeAlignmentCensusInput::InlineBoxFamily {
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
        let reserved = Vec::<text::InlineBoxSource>::with_capacity(3);
        let expected_backing = reserved
            .capacity()
            .checked_mul(std::mem::size_of::<text::InlineBoxSource>())
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
            .inline_box_family(vec![], Some(text::ScopeAlignmentAllowance::new(0, 0)))
            .unwrap_err();
        assert_eq!(refused.code, text::ErrorCode::ScopeAlignmentRefused);
        assert!(matches!(
            refused.detail(),
            Some(text::ErrorDetail::ScopeAlignment {
                reason: text::ScopeAlignmentError::Limit { required, limit: 0, .. }
            }) if *required > 0
        ));
        let family = prepared.inline_box_family(vec![], None).unwrap();
        let retained = family.scope_alignment_retention().unwrap();
        assert!(retained.records() > 0);
        assert!(retained.payload_bytes() > 0);
        let family_input = text::ScopeAlignmentCensusInput::Product(
            text::ScopeAlignmentProduct::InlineBoxFamily(&family),
        );
        let single = text::scope_alignment_census(family_input, products, allowance).unwrap();
        let alias = family.clone();
        let related = [
            text::ScopeAlignmentProduct::InlineBoxFamily(&alias),
            text::ScopeAlignmentProduct::Prepared(&prepared),
        ];
        let shared = text::scope_alignment_census(
            family_input,
            text::ScopeAlignmentProducts::Mixed(&related),
            allowance,
        )
        .unwrap();
        assert_eq!(shared.input_retention(), single.input_retention());
        assert_eq!(
            shared.external_retention(),
            text::ScopeAlignmentRetention::default()
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
            Some(text::Id::from_u64(650_811 + index as u64))
        );
        assert_eq!(original.revision(), [11, 13][index]);
    }
    let tree = ComputedTree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                size: layout::Size::new(
                    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(40.0).unwrap()),
                    layout::PreferredSize::AUTO,
                ),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origins[0].clone())),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origins[1].clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            inline_box_fragments: demand,
            prepared: prepared.clone(),
            container: 0,
            nodes: vec![1, 2],
            origins: origins
                .iter()
                .enumerate()
                .map(|(index, origin)| layout::TextOriginBinding {
                    origin: origin.clone(),
                    node: index + 1,
                })
                .collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
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
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(40.0),
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(batch.placed_text()[0].nodes(), &[1, 2]);
    let placed_output = &batch.placed_text()[0];
    match demand {
        layout::InlineBoxFragmentationDemand::NotRequested => {
            assert!(placed_output.inline_box_fragments().is_none());
        }
        layout::InlineBoxFragmentationDemand::CompleteBodies => {
            let fragments = placed_output.inline_box_fragments().unwrap();
            assert_eq!(fragments.family().prepared_text(), prepared);
            assert_eq!(
                fragments.family().scope_alignment_retention().is_some(),
                limits.is_some()
            );
            let owner = text::InlineBoxSource::captured(processed.captured_scope(1).unwrap());
            let pieces = fragments
                .pieces()
                .filter(|piece| piece.fragment().owner() == owner)
                .collect::<Vec<_>>();
            // Only the receiving A belongs to the genuine captured Inline.
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
    assert_eq!(placed.source().id(), Some(text::Id::from_u64(650_810)));
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
                text::Range::new(0, 1),
                8.0,
                text::Rect::new(0.0, 0.0, 40.0, 10.0)
            ),
            (
                text::Range::new(1, 2),
                18.0,
                text::Rect::new(0.0, 10.0, 40.0, 10.0)
            ),
        ]
    );
    let expected_glyphs = [
        (0, text::Range::new(0, 1), 35, 10.0, 0.0, 8.0),
        (1, text::Range::new(1, 2), 35, 10.0, 0.0, 18.0),
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
        [text::LineEnd::Forced, text::LineEnd::Paragraph]
    );
    let selected_line = finalized[0].selected_line();
    let text::PreparedTermination::Ordinary(offer) = &selected_line.request().termination else {
        panic!("real issued forced offer")
    };
    assert_eq!(offer.view().kind(), text::PreparedBreakKind::Forced);
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
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
            .unwrap()
    };
    let selected = placed
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(2, text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selected.source_text().code_units(), [65, 65]);
    assert_eq!(selected.plaintext().code_units(), [65, 65]);
    assert_eq!(selected.source_pieces().len(), 2);
    for (index, piece) in selected.source_pieces().iter().enumerate() {
        assert_eq!(piece.origin(), &origins[index]);
        assert_eq!(piece.origin().view().source(), &originals[index]);
        assert_eq!(piece.local_range(), text::Range::new(0, 1));
        assert_eq!(piece.aggregate_range(), text::Range::new(index, index + 1));
        assert_eq!(piece.code_units(), [65]);
    }
    let cursor = text::Cursor::new(2, text::Affinity::Before);
    let caret = placed.cursor(cursor).unwrap().rect();
    assert_eq!(caret.origin, text::Point::new(10.0, 10.0));
    assert_eq!(caret.size.height, 10.0);
    drop(system);
    let paint = render::prepare_text_layout(
        placed,
        render::Transform::identity(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(8, 1024, 8 << 20, 8 << 20),
            8,
            8,
            0,
        ),
    )
    .unwrap();
    assert_eq!(paint.scene().len(), 2);
    assert_eq!(
        (paint.scene().stats().commands, paint.scene().stats().glyphs),
        (2, 2)
    );
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
        let mut accessibility = text::Accessibility::new(10);
        let parent = 1000_u64.into();
        let update = paint
            .layout()
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::Point::new(100.0, 200.0),
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
