//! A real early native pair completes final admission without an invented
//! canonical inline scope; actual original controls remain native participants.
use super::*;

struct FinalUnscopedPair<'a, S: layout::scalar::LayoutScalar> {
    host: &'a PairedEarly<S>,
    installed: Vec<Option<layout::node::LayoutInputOf<S>>>,
}
impl<S: layout::scalar::LayoutScalar> layout::tree::Traverse for FinalUnscopedPair<'_, S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        self.host.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.host.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.host.children[node as usize][index]
    }
}
impl<S: layout::scalar::LayoutScalar> layout::inline_wrapping::WrapEdgeSource
    for FinalUnscopedPair<'_, S>
{
    fn wrap_edge_source_node(
        &self,
        node: u32,
    ) -> layout::inline_wrapping::WrapEdgeSourceNodeView<'_, S> {
        layout::inline_wrapping::WrapEdgeSource::wrap_edge_source_node(self.host, node)
    }
    fn source_wrapping_scopes(&self) -> &[layout::inline_wrapping::SourceWrappingScopeInput<u32>] {
        &self.host.declarations
    }
    fn source_scope_associations(&self) -> &[layout::inline_wrapping::SourceScopeAssociation<u32>] {
        &self.host.scopes
    }
    fn generated_child_order(
        &self,
    ) -> &[layout::inline_wrapping::InlineChildOrderInput<u32, usize>] {
        &self.host.base.orders
    }
}
impl<S: layout::scalar::LayoutScalar> layout::tree::LayoutTree for FinalUnscopedPair<'_, S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &layout::node::NodeInputOf<S> {
        if node == 0 {
            &self.host.base.root
        } else {
            &self.host.base.non_box
        }
    }
    fn layout_input(&self, node: u32) -> layout::node::LayoutInputViewOf<'_, S> {
        if node == 0 {
            layout::node::LayoutInputViewOf::Box(&self.host.base.root)
        } else {
            self.installed[node as usize]
                .as_ref()
                .expect("each real origin/control has its actual final input")
                .as_view()
        }
    }
}
fn final_pair_wrapping<S: layout::scalar::LayoutScalar>(
    host: &PairedEarly<S>,
) -> layout::inline_wrapping::WrappingInput<u32> {
    assert_eq!(host.declarations.len(), 1);
    assert_eq!(
        host.declarations[0].box_,
        layout::inline_wrapping::InlineBoxRef::NativePair { start: 3, end: 4 }
    );
    layout::inline_wrapping::WrappingInput {
        native_inline_boxes: Vec::new(),
        gaps: Vec::new(),
        scopes: vec![layout::inline_wrapping::WrappingScopeInput::NativePair {
            start: 3,
            end: 4,
            wrap_inside: host.declarations[0].wrap_inside,
            edges: host.declarations[0].edges,
            metric_contribution: Some(layout::inline_wrapping::InlineBoxMetricContribution::Strut(
                text::allocation::LineBoxStrut {
                    scope: None,
                    metrics: text::allocation::LineStrut {
                        baseline: 0.0,
                        line_extent: 0.0,
                    },
                    alignment: text::allocation::LineBoxAlignment::Baseline,
                },
            )),
        }],
        child_order: vec![],
    }
}
fn qualify_final_pair<S: layout::scalar::LayoutScalar>(
    batch: &layout::batch::CompletedLayoutBatchOf<u32, S>,
    prepared: &text::prepared::PreparedText,
    direction: layout::geometry::Direction,
) {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert!(placed.analyzed_source().identity() == prepared.analyzed_source().identity());
    assert_eq!(
        placed.source().id(),
        Some(text::source::Id::from_u64(656000))
    );
    assert_eq!(placed.source().revision(), 15);
    assert_eq!(placed.source().text(), &text::source::Text::from("AB"));
    assert_eq!(
        placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [
            text::source::Range::new(0, 1),
            text::source::Range::new(1, 2)
        ]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        [8.0, 18.0]
    );
    let root = batch
        .node_layout(0)
        .unwrap()
        .single_box()
        .expect("fixture node produces one box")
        .final_output();
    assert_eq!(
        root.size,
        layout::geometry::Size::new(S::from_f64(40.0), S::from_f64(20.0))
    );
    let mut glyphs = vec![];
    let mut advances = [0.0, 0.0];
    for run in placed.run_views() {
        for glyph in run.glyphs() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                if direction == layout::geometry::Direction::Ltr {
                    0.0
                } else {
                    30.0
                }
            );
            advances[run.line()] += glyph.advance();
            glyphs.push((glyph.id(), glyph.advance(), run.range()));
        }
    }
    assert_eq!(advances, [10.0, 10.0]);
    assert_eq!(
        glyphs,
        [
            (35, 10.0, text::source::Range::new(0, 1)),
            (36, 10.0, text::source::Range::new(1, 2))
        ]
    );
    assert_eq!(prepared.control_views().count(), 2);
    assert_eq!(
        prepared
            .opportunity_views()
            .map(|o| (o.kind(), o.boundary().view().source_before().get()))
            .collect::<Vec<_>>(),
        [
            (text::prepared::PreparedBreakKind::Forced, 1),
            (text::prepared::PreparedBreakKind::ParagraphEnd, 2),
        ]
    );
}
fn unscoped_pair_complete_pipeline<S: layout::scalar::LayoutScalar>() {
    for direction in [
        layout::geometry::Direction::Ltr,
        layout::geometry::Direction::Rtl,
    ] {
        let mut host = PairedEarly::<S>::new(direction, false);
        host.declarations.retain(|row| {
            matches!(
                &row.box_,
                layout::inline_wrapping::InlineBoxRef::NativePair { .. }
            )
        });
        host.scopes.clear();
        host.base.orders.clear();
        let (processed, receipt) = layout::inline_wrapping::process_wrapping_source(
            &host,
            0,
            unscoped_native_pair_input(direction),
            host.occurrences(),
        )
        .unwrap()
        .into_parts();
        assert!(processed.captured_scope(1).is_none());
        assert_eq!(
            processed
                .captured_computed_line_boundary(0)
                .unwrap()
                .target(),
            text::capture::CapturedComputedLineBoundaryTarget::Endpoint(
                processed.captured_control(0).unwrap()
            )
        );
        let processed = Arc::new(processed);
        let prepared = normal_prepare(&processed);
        let mut binding = host.base.install(&prepared);
        assert_eq!(binding.nodes, [1, 3, 2, 4]);
        let mut installed = vec![None, None, None, None, None];
        for origin in &binding.origins {
            installed[origin.node as usize] = host.base.installed[origin.node as usize].clone();
        }
        for view in prepared.control_views() {
            let handle = view.handle();
            let captured = handle.captured_control().unwrap();
            let (node, input) = if captured == processed.captured_control(0).unwrap() {
                assert_eq!(
                    view.kind(),
                    &text::paragraph::ParagraphControlKind::InlineBoundaryStart
                );
                (3, host.start)
            } else {
                assert_eq!(captured, processed.captured_control(1).unwrap());
                assert_eq!(
                    view.kind(),
                    &text::paragraph::ParagraphControlKind::InlineBoundaryEnd
                );
                (4, host.end)
            };
            // Actual existing endpoint metrics are an affirmative zero. No
            // source scope or default/font strut is fabricated for this pair.
            assert_eq!(input.metrics().baseline(), S::ZERO);
            assert_eq!(input.metrics().line_extent(), S::ZERO);
            assert_eq!(input.edge().border_padding_advance(), S::ZERO);
            installed[node as usize] = Some(layout::node::LayoutInputOf::inline_boundary(input));
            binding
                .controls
                .push(layout::text_layout::TextControlBinding {
                    occurrence: handle,
                    node,
                });
        }
        assert_eq!(binding.controls.len(), 2);
        let tree = FinalUnscopedPair {
            host: &host,
            installed,
        };
        let flow=layout::text_layout::LayoutFlowInput::try_new_with_edges(&tree,0,vec![binding.clone()],
            final_pair_wrapping(&host),Vec::new(), &receipt)
            .expect("the same real native pair must admit with its own early receipt, without an InlineScope");
        let batch = layout::compute_layout_with_flow(
            &flow,
            0,
            request(layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::HorizontalTb,
                direction,
            )),
        )
        .unwrap();
        qualify_final_pair(&batch, &prepared, direction);
        let late = layout::text_layout::LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            final_pair_wrapping(&host),
            Vec::new(),
        );
        let error = late
            .err()
            .expect("existing computed causes alone do not authenticate an early Layout plan");
        assert!(
            matches!(
                error.kind(),
                layout::error::LayoutErrorKindOf::InvalidInput(
                    layout::error::LayoutInvalidInputOf::WrappingScope(
                        layout::inline_wrapping::WrappingScopeError::SourceProcessingRequired
                    )
                )
            ),
            "{error:?}"
        );
    }
}
#[test]
fn unscoped_native_pair_early_receipt_admits_and_places_real_controls_through_final_layout() {
    unscoped_pair_complete_pipeline::<f32>();
    unscoped_pair_complete_pipeline::<f64>();
}

mod child_order;
