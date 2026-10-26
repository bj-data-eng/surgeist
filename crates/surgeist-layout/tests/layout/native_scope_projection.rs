//! Hidden native slots cannot change a declared inline scope's break incidence.
use surgeist_layout::*;
use surgeist_text::BidiLevel;

struct NativeRun<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
    children: Vec<usize>,
    non_box: NodeInputOf<S>,
}

impl<S: LayoutScalar> Traverse for NativeRun<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &self.children } else { &[] };
        children.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.children.len() } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!(node, 0);
        self.children[index]
    }
}

impl<S: LayoutScalar> LayoutTree for NativeRun<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn fixture<S: LayoutScalar>(hidden: bool, width: f64) -> NativeRun<S> {
    let atomic = || {
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                InlineBreakOpportunity::Allowed,
            )),
            size: Size::new(
                PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
                PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
            ),
            ..NodeInputOf::default()
        })
    };
    let absent = || {
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::None,
            ..NodeInputOf::default()
        })
    };
    let boundary = |kind| {
        LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
            kind,
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(10.0), S::from_f64(10.0))
                .unwrap(),
            InlineBoundaryEdgeOf::ZERO,
        ))
    };
    NativeRun {
        // Actual source order is A, [H1,H2], Start, B, End, C, D. The declared
        // native pair surrounds only B; the two hidden boxes have no flow.
        inputs: vec![
            LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(width)).unwrap()),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            }),
            atomic(),
            absent(),
            absent(),
            boundary(InlineBoundaryKind::Start),
            atomic(),
            boundary(InlineBoundaryKind::End),
            atomic(),
            atomic(),
        ],
        children: if hidden {
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        } else {
            vec![1, 4, 5, 6, 7, 8]
        },
        non_box: NodeInputOf::non_box(),
    }
}

fn positions<S: LayoutScalar>(
    hidden: bool,
    policy: WrapInside,
    width: f64,
) -> ([Point<S>; 4], Size<S>) {
    let tree = fixture::<S>(hidden, width);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        Vec::new(),
        WrappingInput {
            scopes: vec![WrappingScopeInput::NativePair {
                metric_contribution: None,
                edges: WrapEdges::AUTO,
                start: 4,
                end: 6,
                wrap_inside: policy,
            }],
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(width)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = compute_layout_with_flow(&flow, 0, request).unwrap();
    let output = |node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
    };
    for node in [1, 5, 7, 8] {
        assert_eq!(output(node).size, Size::splat(S::from_f64(10.0)));
    }
    if hidden {
        for node in [2, 3] {
            assert_eq!(output(node).size, Size::ZERO);
        }
    }
    (
        [1, 5, 7, 8].map(|node| output(node).location),
        output(0).size,
    )
}

fn qualify<S: LayoutScalar>() {
    let point = |x, y| Point::new(S::from_f64(x), S::from_f64(y));
    // All cuts are following-break permissions on actual atomic boxes. Start
    // and End supply no following ordinary break. At 25 px the only outside
    // fitting cut is after A; avoiding the fitting after-B cut therefore gives
    // A / BC / D. Auto's ordinary latest fit is AB / CD.
    assert_eq!(
        positions::<S>(false, WrapInside::Auto, 25.0),
        (
            [
                point(0.0, 0.0),
                point(10.0, 0.0),
                point(0.0, 10.0),
                point(10.0, 10.0)
            ],
            Size::new(S::from_f64(25.0), S::from_f64(20.0)),
        )
    );
    assert_eq!(
        positions::<S>(false, WrapInside::Avoid, 25.0),
        (
            [
                point(0.0, 0.0),
                point(0.0, 10.0),
                point(10.0, 10.0),
                point(0.0, 20.0)
            ],
            Size::new(S::from_f64(25.0), S::from_f64(30.0)),
        )
    );
    // At 35 px, after C is a real outside cut and ABC fits 30 px. Four boxes need
    // 40 px, so both policies select ABC / D. Display::None adds no box width,
    // opportunity or scope membership; the same result must survive H1/H2.
    let expected = (
        [
            point(0.0, 0.0),
            point(10.0, 0.0),
            point(20.0, 0.0),
            point(0.0, 10.0),
        ],
        Size::new(S::from_f64(35.0), S::from_f64(20.0)),
    );
    for (hidden, policy) in [
        (false, WrapInside::Auto),
        (true, WrapInside::Auto),
        (false, WrapInside::Avoid),
    ] {
        assert_eq!(
            positions::<S>(hidden, policy, 35.0),
            expected,
            "hidden={hidden}, policy={policy:?}"
        );
    }
    assert_eq!(
        positions::<S>(true, WrapInside::Avoid, 35.0),
        expected,
        "hidden native slots cannot move the actual after-C cut inside B"
    );
}

#[test]
fn hidden_native_slots_do_not_shift_numeric_scope_break_class_f32() {
    qualify::<f32>();
}

#[test]
fn hidden_native_slots_do_not_shift_numeric_scope_break_class_f64() {
    qualify::<f64>();
}
