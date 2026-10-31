//! Independent numeric-only NativePair metric acceptance.
//!
//! Intended existing integration target: tests/layout.rs, child module
//! tests/layout/numeric_box_metrics.rs. No prepared Text or font surrogate.
//! Source inspected at 7ab12bfe9a0acf94853605d7a64cee7e2337a560 plus the
//! functional metric-contribution candidate present in the shared workspace.
//! No execution or historical RED is claimed. Only public API is used.
use surgeist_layout::*;
use surgeist_text::{BidiLevel, LineBoxAlignment, LineBoxStrut, LineStrut};

struct NumericTree<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
    children: Vec<usize>,
    atomics: Vec<usize>,
    scopes: Vec<WrappingScopeInput<usize>>,
    non_box: NodeInputOf<S>,
}

impl<S: LayoutScalar> Traverse for NumericTree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let values: &[usize] = if node == 0 { &self.children } else { &[] };
        values.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.children.len() } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!(node, 0);
        self.children[index]
    }
}

impl<S: LayoutScalar> LayoutTree for NumericTree<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn strut(baseline: f64, extent: f64, alignment: LineBoxAlignment) -> LineBoxStrut {
    LineBoxStrut {
        scope: None,
        metrics: LineStrut {
            baseline,
            line_extent: extent,
        },
        alignment,
    }
}

fn vertical<S: LayoutScalar>(alignment: LineBoxAlignment) -> VerticalAlignOf<S> {
    match alignment {
        LineBoxAlignment::Baseline => VerticalAlignOf::Baseline,
        LineBoxAlignment::Top => VerticalAlignOf::Top,
        LineBoxAlignment::Bottom => VerticalAlignOf::Bottom,
        LineBoxAlignment::Shift(offset) => {
            let value = S::from_f64(offset.get());
            assert!(
                offset.get() == 0.0 || value != S::ZERO,
                "an actual nonzero Text shift must survive scalar narrowing"
            );
            VerticalAlignOf::Shift(
                BaselineOffsetOf::try_new(value)
                    .expect("the actual Text shift must fit the Layout scalar"),
            )
        }
    }
}

impl<S: LayoutScalar> NumericTree<S> {
    fn new(width: f64) -> Self {
        Self {
            inputs: vec![LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(width)).unwrap()),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            })],
            children: vec![],
            atomics: vec![],
            scopes: vec![],
            non_box: NodeInputOf::non_box(),
        }
    }

    fn boundary(&mut self, kind: InlineBoundaryKind, actual: LineBoxStrut) -> usize {
        let node = self.inputs.len();
        self.inputs.push(LayoutInputOf::inline_boundary(
            InlineBoundaryInputOf::new(
                kind,
                InlineMetricsOf::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    S::from_f64(actual.metrics.baseline),
                    S::from_f64(actual.metrics.line_extent),
                )
                .unwrap(),
                InlineBoundaryEdgeOf::ZERO,
            )
            .with_vertical_align(vertical::<S>(actual.alignment)),
        ));
        self.children.push(node);
        node
    }

    fn atomic(&mut self) {
        let node = self.inputs.len();
        self.inputs.push(LayoutInputOf::box_input(NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
                PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
            ),
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                InlineBreakOpportunity::Allowed,
            )),
            ..NodeInputOf::default()
        }));
        self.children.push(node);
        self.atomics.push(node);
    }

    fn associate(
        &mut self,
        start: usize,
        end: usize,
        contribution: Option<InlineBoxMetricContribution>,
    ) {
        self.scopes.push(WrappingScopeInput::NativePair {
            start,
            end,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: contribution,
        });
    }

    fn compute(
        &self,
        width: f64,
    ) -> Result<CompletedLayoutBatchOf<usize, S>, Box<LayoutErrorOf<usize, S>>> {
        let flow = LayoutFlowInput::try_new(
            self,
            0,
            vec![],
            WrappingInput {
                scopes: self.scopes.clone(),
                child_order: vec![],
            },
            Vec::new(),
        )?;
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(width)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        compute_layout_with_flow(&flow, 0, request).map_err(Box::new)
    }
}

fn output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<usize, S>,
    node: usize,
) -> NodeOutputOf<S> {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn assert_geometry<S: LayoutScalar>(
    tree: &NumericTree<S>,
    width: f64,
    height: f64,
    positions: &[(f64, f64)],
) {
    let batch = tree.compute(width).unwrap();
    assert_eq!(
        output(&batch, 0).size,
        Size::new(S::from_f64(width), S::from_f64(height))
    );
    assert_eq!(tree.atomics.len(), positions.len());
    for (&node, &(x, y)) in tree.atomics.iter().zip(positions) {
        let actual = output(&batch, node);
        assert_eq!(actual.location, Point::new(S::from_f64(x), S::from_f64(y)));
        assert_eq!(actual.size, Size::splat(S::from_f64(10.0)));
    }
    // This run has no semantic Text contribution; numeric controls/boxes stay
    // on the numeric path and cannot fabricate prepared line publication.
    assert!(batch.placed_text().is_empty());
}

fn one_box<S: LayoutScalar>(contribution: Option<InlineBoxMetricContribution>) -> NumericTree<S> {
    let actual = strut(12.0, 20.0, LineBoxAlignment::Baseline);
    let mut tree = NumericTree::new(40.0);
    let start = tree.boundary(InlineBoundaryKind::Start, actual.clone());
    tree.atomic();
    let end = tree.boundary(InlineBoundaryKind::End, actual);
    tree.associate(start, end, contribution);
    tree
}

fn selected_endpoints_own_same_box<S: LayoutScalar>() {
    let actual = strut(12.0, 20.0, LineBoxAlignment::Baseline);
    // Atomic10/0 and the real same-box endpoint12/8 yield baseline12,
    // below-baseline8: height20 and body y=2. Supplying that SAME witness does
    // not create another band or another 20px contribution.
    for contribution in [None, Some(InlineBoxMetricContribution::Strut(actual))] {
        assert_geometry(&one_box::<S>(contribution), 40.0, 20.0, &[(0.0, 2.0)]);
    }
}

fn conflicting_same_box_witness<S: LayoutScalar>() {
    for contradiction in [
        InlineBoxMetricContribution::Strut(strut(13.0, 20.0, LineBoxAlignment::Baseline)),
        InlineBoxMetricContribution::Strut(strut(12.0, 21.0, LineBoxAlignment::Baseline)),
        InlineBoxMetricContribution::Strut(strut(12.0, 20.0, LineBoxAlignment::Top)),
        InlineBoxMetricContribution::NoContribution,
    ] {
        let error = one_box::<S>(Some(contradiction.clone()))
            .compute(40.0)
            .expect_err("contradictory same-box witness must refuse");
        assert!(
            matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                    WrappingScopeError::ConflictingBoxMetrics
                ))
            ),
            "contradictory actual same-box evidence: {contradiction:?}: {error:?}"
        );
    }
}

fn fragmented_mixed_groups<S: LayoutScalar>() {
    let values = [
        strut(12.0, 20.0, LineBoxAlignment::Baseline),
        strut(3.0, 30.0, LineBoxAlignment::Top),
        strut(7.0, 40.0, LineBoxAlignment::Bottom),
    ];
    let mut tree: NumericTree<S> = NumericTree::new(10.0);
    let starts = values
        .clone()
        .map(|value| tree.boundary(InlineBoundaryKind::Start, value));
    for _ in 0..4 {
        tree.atomic();
    }
    for i in (0..values.len()).rev() {
        let end = tree.boundary(InlineBoundaryKind::End, values[i].clone());
        tree.associate(
            starts[i],
            end,
            Some(InlineBoxMetricContribution::Strut(values[i].clone())),
        );
    }
    // Actual atomic following cuts produce four one-box fragments. All three
    // boxes genuinely continue across every fragment. Middle lines contain no
    // local endpoints, so their used struts must come from the SAME box facts.
    // Baseline12/8 + Top30 gives [0,30] about baseline12; Bottom40 extends
    // line-over by10. Final baseline22, extent40, atomic top22-10=12.
    assert_geometry(
        &tree,
        10.0,
        160.0,
        &[(0.0, 12.0), (0.0, 52.0), (0.0, 92.0), (0.0, 132.0)],
    );
}

#[test]
fn numeric_same_box_endpoint_witness_is_not_added_again() {
    selected_endpoints_own_same_box::<f32>();
    selected_endpoints_own_same_box::<f64>();
}

#[test]
fn numeric_supplied_box_metrics_must_agree_with_actual_selected_endpoints() {
    conflicting_same_box_witness::<f32>();
    conflicting_same_box_witness::<f64>();
}

#[test]
fn numeric_fragmented_boxes_retain_baseline_top_and_bottom_groups_without_local_endpoints() {
    fragmented_mixed_groups::<f32>();
    fragmented_mixed_groups::<f64>();
}
