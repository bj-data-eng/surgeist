//! A captured End inside the float's whole shared unit is already charged.
use super::*;

fn shared_end_is_charged_once<S: LayoutScalar>() {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let font = super::super::canonical::fonts::mark_ahem();
    let mut binding = capture(
        ["A", "\u{301}B"],
        [
            style(text::WordBreak::Normal, text::LineBreak::Anywhere, false),
            style(text::WordBreak::KeepAll, text::LineBreak::Anywhere, false),
        ],
        &font,
        false,
        Wrap::Word,
        false,
        true,
    );
    assert_eq!(binding.nodes, [100, 1, 101, 102, 2, 103]);
    assert_eq!(
        binding
            .prepared
            .analyzed_source()
            .graphemes()
            .collect::<Vec<_>>(),
        [text::Range::new(0, 2), text::Range::new(2, 3)]
    );
    let first = binding
        .origins
        .iter()
        .find(|row| row.node == 1)
        .unwrap()
        .origin
        .clone();
    let following = binding
        .origins
        .iter()
        .find(|row| row.node == 2)
        .unwrap()
        .origin
        .clone();
    assert_eq!(first.view().source().text(), &text::Text::from("A"));
    assert_eq!(
        following.view().source().text(),
        &text::Text::from("\u{301}B")
    );
    let endpoint = |node| {
        binding
            .controls
            .iter()
            .find(|row| row.node == node)
            .unwrap()
            .occurrence
            .clone()
    };
    let end = endpoint(101);
    let before_scope = endpoint(100).captured_control().unwrap().scope();
    let after_scope = endpoint(102).captured_control().unwrap().scope();
    assert_eq!(before_scope.exit_control(), end.captured_control());
    assert_eq!(after_scope.exit_control(), endpoint(103).captured_control());
    assert_eq!(before_scope.parent(), after_scope.parent());
    // The float is inside C after A, before C's actual End. D then opens and
    // contributes the connected combining original to the same true EGC.
    binding.nodes.insert(2, 50);
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let mut children = vec![52];
    children.extend(binding.nodes.iter().copied());
    let mut tree = tree::<S>(&binding, false, &[])
        .children(0, children)
        .style(0, ordinary.clone())
        .children(50, [])
        .style(
            50,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSizeOf::px(px(17.0)), PreferredSizeOf::px(px(20.0))),
                ..Default::default()
            },
        )
        .children(52, [])
        .style(
            52,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::px(px(10.0))),
                margin: Edges {
                    bottom: LengthAutoOf::px(px(12.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..ordinary
            },
        );
    for row in &binding.controls {
        let kind = match row.occurrence.view().kind() {
            text::ParagraphControlKind::InlineBoundaryStart => InlineBoundaryKind::Start,
            text::ParagraphControlKind::InlineBoundaryEnd => InlineBoundaryKind::End,
            _ => unreachable!("the maintained capture issues only paired boundaries"),
        };
        tree = tree.input(
            row.node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, px(8.0), px(10.0))
                    .unwrap(),
                InlineBoundaryEdgeOf::try_new(
                    S::ZERO,
                    S::ZERO,
                    if row.node == 101 { px(3.0) } else { S::ZERO },
                )
                .unwrap(),
                axes,
            )),
        );
    }
    let before = GeneratedBoxRef::Inline(InlineBoxRef::Scope(before_scope.clone()));
    let after = GeneratedBoxRef::Inline(InlineBoxRef::Scope(after_scope.clone()));
    let child = |box_, participation| GeneratedChildBoxInput {
        box_,
        participation,
    };
    let flow = |box_| child(box_, GeneratedBoxParticipation::InFlow);
    let order = |parent, children| InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let wrapping = WrappingInput {
        scopes: [before_scope, after_scope]
            .into_iter()
            .map(|scope| WrappingScopeInput::Captured {
                scope,
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
            })
            .collect(),
        child_order: vec![
            order(
                GeneratedBoxRef::Node(0),
                vec![
                    flow(GeneratedBoxRef::Node(52)),
                    flow(before.clone()),
                    flow(after.clone()),
                ],
            ),
            order(
                before,
                vec![
                    flow(GeneratedBoxRef::Node(1)),
                    child(
                        GeneratedBoxRef::Node(50),
                        GeneratedBoxParticipation::OutOfFlow,
                    ),
                ],
            ),
            order(after, vec![flow(GeneratedBoxRef::Node(2))]),
        ],
        ..Default::default()
    };
    let capture =
        LayoutFlowInput::try_new(&tree, 0, vec![binding.clone()], wrapping, Vec::new()).unwrap();
    let width = NonNegativeFiniteOf::new(px(30.0)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
    let source_seam = text::TextSeam::Origin {
        origin: first.clone(),
        local: text::SourcePosition::try_new(first.view().source().text(), 1).unwrap(),
    };
    let end_seam = text::TextSeam::BeforeParticipant(text::PreparedParticipant::Control(end));
    let line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.prefix_at(&source_seam)
                .is_ok_and(|prefix| prefix.interior_unit().is_some())
        })
        .expect("the actual shared A/mark line owns the original interior seam");
    assert_eq!(line.prepared_text(), binding.prepared);
    let prefix = line.prefix_at(&source_seam).unwrap();
    let end_prefix = line.prefix_at(&end_seam).unwrap();
    assert!(prefix.has_in_flow_content());
    assert_eq!(prefix.advance(), 13.0);
    assert_eq!(end_prefix.advance(), 13.0);
    let shared = prefix.interior_unit().unwrap();
    let end_shared = end_prefix.interior_unit().unwrap();
    assert_eq!(shared.presentation_range(), end_shared.presentation_range());
    assert_eq!(shared.source_range(), text::Range::new(0, 2));
    assert_eq!(shared.advance().unwrap(), 13.0);
    // The maintained mark font supplies A10 and acute0. This exact original
    // seam counts the entire shared group, including physical End101's3px.
    // Prefix13+F17=30 fits exactly; adding that same End again would make33.
    let prior = readback::single_box_output(&batch, 52, Phase::Final);
    assert_eq!((prior.location.y, prior.size.height), (S::ZERO, px(10.0)));
    let float = readback::single_box_output(&batch, 50, Phase::Final);
    assert_eq!(float.size, Size::new(px(17.0), px(20.0)));
    assert_eq!(float.location.y, px(22.0));
    assert_eq!(
        readback::single_box_output(&batch, 0, Phase::Final)
            .size
            .height,
        px(42.0)
    );
    let [placed] = batch.placed_text() else {
        panic!("one actual shared-unit paragraph")
    };
    assert_eq!(
        placed
            .layout()
            .lines()
            .iter()
            .map(|line| (line.bounds().origin.y, line.range()))
            .collect::<Vec<_>>(),
        [
            (22.0, text::Range::new(0, 2)),
            (32.0, text::Range::new(2, 3))
        ]
    );
}

#[test]
fn joint_unit_end_is_not_reserved_twice_at_exact_float_fit_f32() {
    shared_end_is_charged_once::<f32>();
}
#[test]
fn joint_unit_end_is_not_reserved_twice_at_exact_float_fit_f64() {
    shared_end_is_charged_once::<f64>();
}
