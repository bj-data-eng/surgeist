use super::*;
use crate::engine::block_scope::{Dispatch, ReceiptId, Requests, SourceViewId};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use std::{cell::RefCell, rc::Rc, sync::Arc};

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn axes() -> FlowAxes {
    FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl)
}
fn block<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        direction: Direction::Rtl,
        ..Default::default()
    }
}
fn wrapper<S: LayoutScalar>(block_percentage: bool, fractional: bool) -> NodeInputOf<S> {
    let mut input = block::<S>();
    input.size.width = PreferredSizeOf::px(n(40.0));
    input.margin.left = LengthAutoOf::AUTO;
    input.margin.right = LengthAutoOf::AUTO;
    input.padding.left = LengthOf::px(n(5.0));
    input.padding.right = LengthOf::px(n(5.0));
    if block_percentage {
        input.inset.top = LengthAutoOf::percent(n(0.1));
    } else {
        input.inset.left = LengthAutoOf::percent(n(if fractional { 0.125 } else { 0.1 }));
    }
    input
}
fn make_tree<S: LayoutScalar>(block_percentage: bool, fractional: bool) -> PublicLayoutTreeOf<S> {
    let mut owner = block::<S>();
    owner.max_size.height = MaxSizeOf::px(n(20.0));
    let mut first = block::<S>();
    first.size.height = PreferredSizeOf::px(n(30.0));
    let mut last = first.clone();
    last.break_before = FragmentationBreak::Page;
    if fractional {
        owner.direction = Direction::Ltr;
        first.direction = Direction::Ltr;
        last.direction = Direction::Ltr;
        last.size.width = PreferredSizeOf::px(n(30.5));
    }
    PublicLayoutTreeOf::new()
        .style(0, block())
        .style(1, wrapper(block_percentage, fractional))
        .style(2, owner)
        .style(3, first)
        .style(4, last)
        .children(0, [1])
        .children(1, [2])
        .children(2, [3, 4])
}
fn request<S: LayoutScalar>(width: f64) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(n(width)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        axes(),
        Size::new(Some(NonNegativeFiniteOf::new(n(width)).unwrap()), None),
    )
}
fn frame<S: LayoutScalar>(width: f64) -> BlockFragmentainerOf<S> {
    BlockFragmentainerOf {
        origin: Point::new(n(0.25), S::ZERO),
        size: Size::new(n(width), n(40.0)),
        breaks: FragmentationBreakKinds::PAGE,
        page_side: None,
    }
}
fn bound<S: LayoutScalar>() -> LayoutBoundRequestOf<S> {
    let mut first = frame::<S>(100.0);
    first.breaks = FragmentationBreakKinds::NONE;
    first.size.height = n(50.0);
    LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        request(100.0),
        FragmentationKind::Page {
            progression: crate::PageProgression::LeftToRight,
        },
        axes(),
        vec![first, frame(60.0)],
    )
    .unwrap()
}
fn snapshot<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    width: f64,
    target: LayoutTargetBasis,
) -> Arc<intrinsic::FragmentContext<u32, S>> {
    Arc::new(
        crate::engine::prepare_block_fragment_intrinsics(tree, 0, request(width), target, axes())
            .unwrap(),
    )
}
fn binding<S: LayoutScalar>(target: LayoutTargetBasis) -> ContentInsetBinding<u32, S> {
    ContentInsetBinding {
        owner: 2,
        role: BlockGeneration::Principal,
        target,
        placement: ReceiptId(7),
        trial: content_insets::ContentTrialId(3),
        frame: 1,
        axes: axes(),
        start: FragmentationCutOf::Source(FragmentationPosition {
            occurrence: 0,
            offset: 0,
        }),
        end: FragmentationCutOf::Source(FragmentationPosition {
            occurrence: 2,
            offset: 0,
        }),
    }
}
fn perform<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    width: f64,
    snapshot: Arc<intrinsic::FragmentContext<u32, S>>,
    binding: ContentInsetBinding<u32, S>,
) -> PerformedPath<u32, S> {
    let input = snapshot
        .entries
        .iter()
        .find(|entry| entry.node == 2)
        .unwrap()
        .input
        .with_run_mode(RunMode::PerformLayout);
    let candidate = crate::engine::prepare_block_carrier_path(
        tree,
        &[0, 1, 2],
        request(width),
        frame(width),
        snapshot.clone(),
        binding.clone(),
    )
    .unwrap();
    let produced = crate::engine::compute_block_content_frame(
        tree,
        crate::engine::BlockContentFrameRequest {
            root: 2,
            input,
            original: crate::LayoutTree::node_input(tree, 2),
            reference: S::ZERO,
            opening: false,
            complete: true,
            principal_content: None,
            accepted: Some(binding.clone()),
            carrier: Some(candidate),
            binding: crate::engine::BlockFrameBinding {
                allocations: Vec::new(),
                intrinsics: snapshot,
                dispatch: Dispatch {
                    scope: binding.placement,
                    view: SourceViewId(binding.trial.0),
                    target: binding.target,
                    owners: Vec::<u32>::new().into(),
                    paths: Vec::<u32>::new().into(),
                    ready: Rc::new(RefCell::new(Vec::new())),
                    requests: Rc::new(RefCell::new(Requests::new())),
                },
            },
        },
    )
    .unwrap();
    assert_eq!(produced.context.inline_outer, n(30.0));
    assert!(produced.principal.is_none());
    produced.carrier.unwrap()
}
fn exact_path_adoption<S: LayoutScalar>() {
    let tree = make_tree::<S>(false, false);
    let target = bound::<S>().target_basis();
    let binding = binding(target.clone());
    let old = snapshot(&tree, 100.0, target.clone());
    let current = snapshot(&tree, 60.0, target);
    let stale = perform(&tree, 100.0, old, binding.clone());
    let fresh = perform(&tree, 60.0, current.clone(), binding.clone());
    // Both terminal Normal inputs have W width30. Ancestor R width100/60
    // changes U centering30/10 and its receiving basis100/60 independently.
    assert_eq!(
        stale.0.facts.last().unwrap().outer,
        fresh.0.facts.last().unwrap().outer
    );
    assert_eq!(stale.0.facts[1].normal.x, n(30.0));
    assert_eq!(fresh.0.facts[1].normal.x, n(10.0));
    assert!(
        stale
            .adopt(
                &tree,
                ProductionId(13),
                &binding,
                request(60.0),
                frame(60.0),
                &current
            )
            .is_err()
    );
    let accepted = fresh
        .adopt(
            &tree,
            ProductionId(13),
            &binding,
            request(60.0),
            frame(60.0),
            &current,
        )
        .unwrap();
    let session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let origin = accepted
        .origin(&session, ProductionId(13), &binding, 1, None)
        .unwrap();
    assert_eq!(origin.normal, Point::new(n(10.0), S::ZERO));
    assert_eq!(origin.visual, Point::new(n(16.0), S::ZERO));
    assert!(
        accepted
            .origin(&session, ProductionId(14), &binding, 1, None)
            .is_err()
    );
    for changed in [
        ContentInsetBinding {
            frame: 2,
            ..binding.clone()
        },
        ContentInsetBinding {
            placement: ReceiptId(8),
            ..binding.clone()
        },
        ContentInsetBinding {
            trial: content_insets::ContentTrialId(4),
            ..binding.clone()
        },
        ContentInsetBinding {
            target: bound::<S>().target_basis(),
            ..binding.clone()
        },
    ] {
        let prospective = perform(&tree, 60.0, current.clone(), binding.clone());
        assert!(
            prospective
                .adopt(
                    &tree,
                    ProductionId(13),
                    &changed,
                    request(60.0),
                    frame(60.0),
                    &current
                )
                .is_err()
        );
    }
    let prospective = perform(&tree, 60.0, current.clone(), binding.clone());
    let mut changed = wrapper::<S>(false, false);
    changed.margin.right = LengthAutoOf::px(n(2.0));
    let changed_tree = make_tree::<S>(false, false).style(1, changed);
    assert!(
        prospective
            .adopt(
                &changed_tree,
                ProductionId(13),
                &binding,
                request(60.0),
                frame(60.0),
                &current
            )
            .is_err()
    );
}
fn no_block_authority<S: LayoutScalar>() {
    let tree = make_tree::<S>(true, false);
    let target = bound::<S>().target_basis();
    let binding = binding(target.clone());
    let snapshot = snapshot(&tree, 60.0, target);
    let path = perform(&tree, 60.0, snapshot.clone(), binding.clone())
        .adopt(
            &tree,
            ProductionId(13),
            &binding,
            request(60.0),
            frame(60.0),
            &snapshot,
        )
        .unwrap();
    let session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let error = path
        .origin(&session, ProductionId(13), &binding, 1, None)
        .unwrap_err();
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::FragmentContainingBlock {
            container: 0,
            subject: 1,
            fragmentainer_index: 1,
            axis: PhysicalAxis::Vertical
        }
    );
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));
}
fn fractional_composed_path<S: LayoutScalar>() {
    let tree = make_tree::<S>(false, true);
    let bound = bound::<S>();
    let capture = LayoutFlowInput::try_new_bound(
        &tree,
        0,
        bound.clone(),
        Vec::new(),
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, bound.root_request()).unwrap();
    let child = batch.node_layout(4).unwrap().single_box().unwrap();
    let crate::output::GeometryStateOf::OrdinaryCanonical { normal } =
        child.final_entry().geometry_state()
    else {
        panic!("paired result retains canonical normal geometry");
    };
    assert_eq!(normal.size.width, n(31.0));
    assert_eq!(child.final_output().size.width, n(30.0));
    let wrapper = batch.node_layout(1).unwrap().fragments().unwrap()[1]
        .overflow_only()
        .unwrap();
    assert_eq!(wrapper.unrounded().origin.x, n(17.5));
    assert_eq!(wrapper.final_geometry().origin.x, n(18.0));
    assert_eq!(child.unrounded_output().location.x, S::ZERO);
    assert_eq!(child.final_output().location.x, S::ZERO);
}
#[test]
fn carrier_path_rejects_stale_ancestor_frame_source_and_adoption_f32() {
    exact_path_adoption::<f32>();
}
#[test]
fn carrier_path_rejects_stale_ancestor_frame_source_and_adoption_f64() {
    exact_path_adoption::<f64>();
}
#[test]
fn carrier_path_inline_receipt_does_not_measure_block_zero_f32() {
    no_block_authority::<f32>();
}
#[test]
fn carrier_path_inline_receipt_does_not_measure_block_zero_f64() {
    no_block_authority::<f64>();
}
#[test]
fn carrier_path_composed_fractional_normal31_visual30_f32() {
    fractional_composed_path::<f32>();
}
#[test]
fn carrier_path_composed_fractional_normal31_visual30_f64() {
    fractional_composed_path::<f64>();
}

fn real_reserving_gutter_normal_context<S: LayoutScalar>() {
    let mut root = block::<S>();
    root.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
    root.scrollbar_gutter = ScrollbarGutter::StableBothEdges;
    root.scrollbar_width = ScrollbarWidthOf::try_new(n(4.0)).unwrap();
    let tree = make_tree::<S>(false, false).style(0, root);
    // This is a real ordinary Block Normal context, independently of Native's
    // retained refusal for non-visible-overflow continuation profiles.
    let ordinary = compute_layout(&tree, 0, request(60.0)).unwrap();
    let geometry = ordinary
        .node_layout(0)
        .unwrap()
        .single_box()
        .unwrap()
        .unrounded_output()
        .scroll_geometry
        .unwrap();
    assert_eq!(geometry.content_box().size().width, n(52.0));
    let gutters = geometry.gutters();
    assert_eq!(gutters.left().unwrap().size().width, n(4.0));
    assert_eq!(gutters.right().unwrap().size().width, n(4.0));
    let target = bound::<S>().target_basis();
    let binding = binding(target.clone());
    let snapshot = snapshot(&tree, 60.0, target);
    let prospective = perform(&tree, 60.0, snapshot.clone(), binding.clone());
    assert_eq!(prospective.0.facts[0].inner, n(52.0));
    let accepted = prospective
        .adopt(
            &tree,
            ProductionId(13),
            &binding,
            request(60.0),
            frame(60.0),
            &snapshot,
        )
        .unwrap();
    let session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let origin = accepted
        .origin(&session, ProductionId(13), &binding, 1, None)
        .unwrap();
    assert_eq!(origin.normal.x, n(10.0));
    let expected = n::<S>(10.0) + n::<S>(52.0) * n::<S>(0.1);
    assert_eq!(origin.visual.x, expected);
}
#[test]
fn carrier_path_inline_basis_uses_real_reserving_gutter_normal_context_f32() {
    real_reserving_gutter_normal_context::<f32>();
}
#[test]
fn carrier_path_inline_basis_uses_real_reserving_gutter_normal_context_f64() {
    real_reserving_gutter_normal_context::<f64>();
}
