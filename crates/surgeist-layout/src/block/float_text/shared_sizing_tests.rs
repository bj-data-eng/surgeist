//! Shared child sizing preserves the owning captured-float computation.
use crate::engine::contracts::Compute;
use crate::test_support::inline_endpoint::Fixture;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;
use surgeist_text as text;

fn request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
    ))
}

fn assert_admitted<S: LayoutScalar>(capture: &LayoutFlowInput<'_, PublicLayoutTreeOf<S>>) {
    let [row] = capture.associations.runs() else {
        panic!("one actual captured native run");
    };
    let admitted = capture.associations.admitted_run(&capture.paragraphs, row);
    assert_eq!(admitted.binding.container, 0);
    assert_eq!(admitted.nodes(), [20, 2, 10, 21]);
    assert_eq!(admitted.binding.origins.len(), 1);
    assert_eq!(admitted.binding.origins[0].node, 10);
    assert_eq!(
        admitted.binding.prepared.analyzed_source().source().text(),
        &text::Text::from("A B C D")
    );
}

fn assert_scene<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    opening: &text::PreparedControl,
) {
    let px = S::from_f64;
    let [placed] = batch.placed_text() else {
        panic!("one actual processed paragraph");
    };
    let layout = placed.layout();
    assert_eq!(layout.source().text(), &text::Text::from("A B C D"));
    let line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.selected_line()
                .request()
                .controls
                .iter()
                .any(|row| row.occurrence == *opening)
        })
        .expect("the actual opening allocation retains its line receipt");
    let prefix = line
        .prefix_at(&text::TextSeam::AfterParticipant(
            text::PreparedParticipant::Control(opening.clone()),
        ))
        .unwrap();
    assert_eq!(prefix.advance(), 0.0);
    assert!(prefix.has_retained_flow_participation());
    assert!(!prefix.has_in_flow_content());
    let before = single_box_output(batch, 52, Phase::Unrounded);
    assert_eq!((before.location.y, before.size.height), (S::ZERO, px(10.0)));
    let float = single_box_output(batch, 2, Phase::Unrounded);
    assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
    assert_eq!(
        (float.location.y, float.location.y + float.size.height),
        (px(22.0), px(42.0))
    );
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.bounds().origin.y)
            .collect::<Vec<_>>(),
        [42.0, 52.0]
    );
    let e = single_box_output(batch, 0, Phase::Unrounded);
    assert_eq!(e.location, Point::ZERO);
    assert_eq!(e.size, Size::new(px(30.0), px(62.0)));
}

struct Scene<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    binding: TextParagraphBinding<u32>,
    opening: text::PreparedControl,
    ordinary: NodeInputOf<S>,
}

fn scene<S: LayoutScalar>() -> Scene<S> {
    let px = S::from_f64;
    let fixture = Fixture::new("A B C D", (0, 7), text::WhiteSpace::Collapse, false, false);
    let mut binding = fixture.binding.clone();
    binding.nodes.insert(1, 2);
    assert_eq!(binding.nodes, [20, 2, 10, 21]);
    let opening = binding
        .controls
        .iter()
        .find(|row| row.node == 20)
        .unwrap()
        .occurrence
        .clone();
    let closing = binding
        .controls
        .iter()
        .find(|row| row.node == 21)
        .unwrap()
        .occurrence
        .clone();
    let scope = opening.captured_control().unwrap().scope();
    assert_eq!(scope.entry_control(), opening.captured_control());
    assert_eq!(scope.exit_control(), closing.captured_control());
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let mut children = vec![52];
    children.extend(binding.nodes.iter().copied());
    let tree = fixture
        .tree::<S>(
            InlineBoundaryEdgeOf::ZERO,
            InlineBoundaryEdgeOf::ZERO,
            false,
            Some(30.0),
            true,
        )
        .children(0, children)
        .style(0, ordinary.clone())
        .children(52, [])
        .style(
            52,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::px(px(10.0))),
                margin: Edges {
                    bottom: LengthAutoOf::px(px(12.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..ordinary.clone()
            },
        );
    Scene {
        tree,
        binding,
        opening,
        ordinary,
    }
}

fn shared_child<S: LayoutScalar>() {
    let px = S::from_f64;
    let Scene {
        tree,
        binding,
        opening,
        ordinary,
    } = scene::<S>();
    // Both calls use the same host sizing request and authored E30. At its
    // owning root, A10+MB12 enters the run at22; nominal-only Start0 leaves
    // room for F25. Its remaining5px cannot hold an Ahem10 word, so real
    // source lines use42/52 and E's height is62.
    let owning_capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding.clone()],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    assert_admitted(&owning_capture);
    let owning = compute_layout_with_flow(&owning_capture, 0, request()).unwrap();
    assert_scene(&owning, &opening);

    // The only new tree context is a real ordinary parent. E remains Block,
    // so it shares that parent's BFC; no independent formatting root or
    // inherited physical frame is supplied by the fixture.
    let shared_tree = tree.clone().children(100, [0]).style(
        100,
        NodeInputOf {
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            ..ordinary
        },
    );
    let shared_capture = LayoutFlowInput::try_new(
        &shared_tree,
        100,
        vec![binding],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    assert_admitted(&shared_capture);
    let shared = compute_layout_with_flow(&shared_capture, 100, request()).unwrap();
    assert_scene(&shared, &opening);
    assert_eq!(
        single_box_output(&shared, 100, Phase::Unrounded).size.width,
        px(100.0)
    );
}

#[test]
fn ordinary_shared_child_sizes_captured_float_f32() {
    shared_child::<f32>();
}
#[test]
fn ordinary_shared_child_sizes_captured_float_f64() {
    shared_child::<f64>();
}

fn inherited_ancestor<S: LayoutScalar>() {
    let px = S::from_f64;
    let Scene {
        tree,
        binding,
        opening,
        ordinary,
    } = scene::<S>();
    let tree = tree
        .children(100, [1, 0])
        .style(
            100,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
                ..ordinary.clone()
            },
        )
        .children(1, [])
        .style(
            1,
            NodeInputOf {
                float: Float::Left,
                size: Size::new(PreferredSizeOf::px(px(5.0)), PreferredSizeOf::px(px(42.0))),
                ..ordinary
            },
        );
    let capture = LayoutFlowInput::try_new(
        &tree,
        100,
        vec![binding],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    assert_admitted(&capture);
    let batch = compute_layout_with_flow(&capture, 100, request()).unwrap();
    // At source22 the ancestor occupies x0..5 through42. The remaining25px
    // admits F25 exactly atx5. Both floats end42, so the unchanged source
    // lines use42/52 and E/root height62. The x5 origin proves inheritance.
    assert_scene(&batch, &opening);
    let ancestor = single_box_output(&batch, 1, Phase::Unrounded);
    assert_eq!(ancestor.location, Point::ZERO);
    assert_eq!(ancestor.size, Size::new(px(5.0), px(42.0)));
    assert_eq!(
        single_box_output(&batch, 2, Phase::Unrounded).location.x,
        px(5.0)
    );
    assert_eq!(
        single_box_output(&batch, 100, Phase::Unrounded).size,
        Size::new(px(100.0), px(62.0))
    );
}

fn unframed_full_size<S: LayoutScalar>() {
    let px = S::from_f64;
    let Scene {
        tree,
        binding,
        ordinary,
        ..
    } = scene::<S>();
    let tree = tree.children(100, [0]).style(
        100,
        NodeInputOf {
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            ..ordinary
        },
    );
    let capture = LayoutFlowInput::try_new(
        &tree,
        100,
        vec![binding],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    assert_admitted(&capture);
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(px(100.0)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        request::<S>().sizing_request(),
    );
    let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    assert_eq!(
        super::super::float_context::Role::for_input(owner.node_input(0), input),
        super::super::float_context::Role::Shared
    );
    assert!(
        owner
            .block_float_state()
            .unwrap()
            .incoming(0, input)
            .is_none()
    );
    // This direct full-body call has no containing traversal to install real
    // Incoming custody. It must retain the typed missing-authority refusal
    // when captured float replay would otherwise export physical rows.
    let error = owner.compute_child(0, input).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));
}

#[test]
fn shared_captured_float_uses_preceding_ancestor_exclusion_f32() {
    inherited_ancestor::<f32>();
}
#[test]
fn shared_captured_float_uses_preceding_ancestor_exclusion_f64() {
    inherited_ancestor::<f64>();
}
#[test]
fn unframed_shared_full_size_retains_required_basis_refusal_f32() {
    unframed_full_size::<f32>();
}
#[test]
fn unframed_shared_full_size_retains_required_basis_refusal_f64() {
    unframed_full_size::<f64>();
}

fn owner_sizes_shared_capture<S: LayoutScalar>() {
    let px = S::from_f64;
    let Scene {
        tree,
        binding,
        ordinary,
        ..
    } = scene::<S>();
    let tree = tree.children(100, [0]).style(
        100,
        NodeInputOf {
            size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
            ..ordinary
        },
    );
    let capture = LayoutFlowInput::try_new(
        &tree,
        100,
        vec![binding],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    assert_admitted(&capture);
    // The actual owner can determine a finite Normal size by issuing its
    // shared child a physical frame. This is a size request before any
    // performed result exists; it does not supply an inherited frame.
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(px(100.0)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        request::<S>().sizing_request(),
    );
    let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    assert_eq!(
        super::super::float_context::Role::for_input(owner.node_input(100), input),
        super::super::float_context::Role::Owner
    );
    let output = owner.compute_child(100, input).unwrap();
    // A10+MB12 starts F at22. F ends42; the two Text lines end62.
    // E's authored width remains30 within its100px owning parent.
    assert_eq!(output.size, Size::new(px(100.0), px(62.0)));
}

#[test]
fn owner_size_request_frames_shared_captured_child_f32() {
    owner_sizes_shared_capture::<f32>();
}
#[test]
fn owner_size_request_frames_shared_captured_child_f64() {
    owner_sizes_shared_capture::<f64>();
}
