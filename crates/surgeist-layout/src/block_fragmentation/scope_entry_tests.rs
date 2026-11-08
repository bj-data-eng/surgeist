use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn with_driver<S: LayoutScalar>(run: impl FnOnce(&Driver<'_, '_, '_, PublicLayoutTreeOf<S>>)) {
    let block = || NodeInputOf::<S> {
        display: Display::Block,
        ..Default::default()
    };
    let mut leaf = block();
    leaf.size.height = PreferredSizeOf::px(S::from_f64(10.0));
    let tree = PublicLayoutTreeOf::new()
        .style(0, block())
        .style(1, leaf.clone())
        .style(2, leaf)
        .children(0, [1, 2]);
    with_tree(&tree, run);
}

fn with_tree<
    S: LayoutScalar,
    Tree: LayoutTree<Node = u32, Scalar = S, MeasureError = std::convert::Infallible>,
>(
    tree: &Tree,
    run: impl FnOnce(&Driver<'_, '_, '_, Tree>),
) {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let sizing = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = LayoutRootRequestOf::standalone(sizing, axes, Size::NONE);
    let frames = [BlockFragmentainerOf {
        origin: Point::ZERO,
        size: Size::splat(S::from_f64(100.0)),
        breaks: FragmentationBreakKinds::NONE,
        page_side: None,
    }];
    let bound = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        request,
        FragmentationKind::Page,
        axes,
        frames.to_vec(),
    )
    .unwrap();
    let capture = LayoutFlowInput::try_new_bound(
        tree,
        0,
        bound.clone(),
        Vec::new(),
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let scopes = Scopes::new(
        Inventory::new(tree, &capture.graph, axes).unwrap(),
        tree,
        axes,
    )
    .unwrap();
    let environment = NativeProvider {
        capture: &capture,
        inventory: &scopes.source,
        frames: &frames,
        axes,
        kind: FragmentationKind::Page,
        target: bound.target_basis(),
        template: request,
        intrinsics: Rc::new(RefCell::new(Vec::new())),
    };
    let driver = Driver::new(&environment, &scopes).unwrap();
    run(&driver);
}
fn pending<
    S: LayoutScalar,
    Tree: LayoutTree<Node = u32, Scalar = S, MeasureError = std::convert::Infallible>,
>(
    driver: &Driver<'_, '_, '_, Tree>,
    width: f64,
) -> crate::block::scope::Pending<u32, S> {
    let axes = driver.environment.axes;
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(width)), None),
        Size::NONE,
        ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
        driver.environment.template.sizing_request(),
    );
    let mut session =
        crate::engine::ComputeSession::new(driver.environment.capture.tree, Vec::new());
    crate::block::scope::root(&mut session, 0, input).unwrap()
}
fn selection<
    S: LayoutScalar,
    Tree: LayoutTree<Node = u32, Scalar = S, MeasureError = std::convert::Infallible>,
>(
    driver: &Driver<'_, '_, '_, Tree>,
    end: usize,
) -> Selection<S> {
    assert!(end <= driver.scopes.scopes[0].units.len());
    Selection {
        range: 0..end,
        partial: None,
        resumed: None,
        provenance: FragmentationStart::InFrame,
        begin: None,
        cut: FragmentationCutOf::Source(FragmentationPosition {
            occurrence: end,
            offset: 0,
        }),
    }
}
fn actual<
    S: LayoutScalar,
    Tree: LayoutTree<Node = u32, Scalar = S, MeasureError = std::convert::Infallible>,
>(
    driver: &Driver<'_, '_, '_, Tree>,
    pending: crate::block::scope::Pending<u32, S>,
) -> Packet<u32, S> {
    crate::engine::qualify_block_scope_body(
        driver.environment.capture.tree,
        crate::engine::BlockScopeEntryRequest {
            pending,
            binding: crate::engine::BlockFrameBinding {
                allocations: Vec::new(),
                intrinsics: driver
                    .environment
                    .intrinsics(driver.environment.frames[0])
                    .unwrap(),
                dispatch: driver.dispatch(&driver.stack[0]),
            },
        },
    )
    .unwrap()
    .packet
}
fn retain<
    S: LayoutScalar,
    Tree: LayoutTree<Node = u32, Scalar = S, MeasureError = std::convert::Infallible>,
>(
    driver: &Driver<'_, '_, '_, Tree>,
    pending: crate::block::scope::Pending<u32, S>,
    target: u32,
) {
    let work = &driver.stack[0];
    let body = driver.body(work, pending, 0).unwrap();
    let mut uses = Vec::new();
    driver
        .use_body(&mut uses, &body, pending, target, 0)
        .unwrap();
    driver.retain_uses(work, target, uses).unwrap();
}
fn changed_input<S: LayoutScalar>() {
    with_driver::<S>(|driver| {
        let first = pending(driver, 100.0);
        let changed = pending(driver, 60.0);
        retain(driver, first, 0);
        let provider = driver.provider(&driver.stack[0]);
        let complete = selection(driver, driver.scopes.scopes[0].units.len());
        provider
            .validate_qualifications(&actual(driver, first), &complete, 0)
            .unwrap();
        let changed_packet = actual(driver, changed);
        assert!(
            matches!(provider.validate_qualifications(&changed_packet, &complete, 0),
            Err(ComputeStop::Layout(error)) if matches!(error.kind(), LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)))
        );
        // A distinct exact producer input qualifies its real 60-unit body.
        // Adoption then validates against that body, rather than reusing 100.
        let qualified = driver.body(&driver.stack[0], changed, 0).unwrap();
        assert_eq!(
            qualified.output.size,
            Size::new(S::from_f64(60.0), S::from_f64(20.0))
        );
        retain(driver, changed, 0);
        provider
            .validate_qualifications(&changed_packet, &complete, 0)
            .unwrap();
        assert!(driver.stack[0].progress.borrow().completed.is_empty());
    });
}
fn candidate_free_retry<S: LayoutScalar>() {
    with_driver::<S>(|driver| {
        retain(driver, pending(driver, 100.0), 0);
        // This replacement is the result of an exact same-target placement
        // retry whose entry no longer demands ordinary body qualification.
        driver.retain_uses(&driver.stack[0], 0, Vec::new()).unwrap();
        let provider = driver.provider(&driver.stack[0]);
        provider
            .validate_qualifications(
                &actual(driver, pending(driver, 60.0)),
                &selection(driver, driver.scopes.scopes[0].units.len()),
                0,
            )
            .unwrap();
    });
}
fn selected_incomplete<S: LayoutScalar>() {
    with_driver::<S>(|driver| {
        retain(driver, pending(driver, 100.0), 1);
        let provider = driver.provider(&driver.stack[0]);
        let prefix = selection(driver, 1);
        let produced = provider.trial(&prefix, 0, Vec::new(), S::ZERO).unwrap();
        assert!(
            matches!(provider.validate_qualifications(&produced.packet, &prefix, 0),
            Err(ComputeStop::Layout(error)) if matches!(error.kind(),
                LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::BlockFragmentationProfile)))
        );
        assert!(driver.stack[0].progress.borrow().completed.is_empty());
    });
}
#[test]
fn scope_entry_changed_exact_input_refuses_stale_body_and_requalifies_f32() {
    changed_input::<f32>();
}
#[test]
fn scope_entry_changed_exact_input_refuses_stale_body_and_requalifies_f64() {
    changed_input::<f64>();
}
#[test]
fn scope_entry_candidate_free_retry_replaces_old_body_constraint_f32() {
    candidate_free_retry::<f32>();
}
#[test]
fn scope_entry_candidate_free_retry_replaces_old_body_constraint_f64() {
    candidate_free_retry::<f64>();
}
#[test]
fn scope_entry_selected_incomplete_qualified_subtree_refuses_before_accounting_f32() {
    selected_incomplete::<f32>();
}
#[test]
fn scope_entry_selected_incomplete_qualified_subtree_refuses_before_accounting_f64() {
    selected_incomplete::<f64>();
}

/// The host changes one natural-size source only between complete producer
/// calls. Within each phase it honors known dimensions and records the exact
/// normalized measurement request; every layout operation still runs real Block.
struct PhaseTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    height: Cell<S>,
    measurements: RefCell<Vec<LeafMeasureInputOf<S>>>,
}
impl<S: LayoutScalar> Traverse for PhaseTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::vec::IntoIter<u32>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for PhaseTree<S> {
    type MeasureError = std::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        LayoutInputViewOf::Box(self.node_input(node))
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == 1
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        self.measurements.borrow_mut().push(input);
        Some(Ok(Size::new(
            input
                .known_content_size()
                .width
                .unwrap_or(S::from_f64(100.0)),
            input
                .known_content_size()
                .height
                .unwrap_or(self.height.get()),
        )))
    }
}
fn changed_same_input_body<S: LayoutScalar>() {
    let block = || NodeInputOf::<S> {
        display: Display::Block,
        ..Default::default()
    };
    let mut leaf = block();
    leaf.size.height = PreferredSizeOf::px(S::from_f64(10.0));
    let tree = PhaseTree {
        tree: PublicLayoutTreeOf::new()
            .style(0, block())
            .style(1, block())
            .style(2, leaf)
            .children(0, [1, 2]),
        height: Cell::new(S::from_f64(10.0)),
        measurements: RefCell::new(Vec::new()),
    };
    with_tree(&tree, |driver| {
        let request = pending(driver, 100.0);
        let first = driver.body(&driver.stack[0], request, 0).unwrap();
        assert_eq!(
            first.output.size,
            Size::new(S::from_f64(100.0), S::from_f64(20.0))
        );
        retain(driver, request, 0);
        let normalized = *tree
            .measurements
            .borrow()
            .last()
            .expect("qualification measures the real leaf");
        tree.height.set(S::from_f64(30.0));
        let replay = actual(driver, request);
        assert_eq!(*tree.measurements.borrow().last().unwrap(), normalized);
        let actual_root = replay.bodies.iter().find(|body| body.node == 0).unwrap();
        assert_eq!(actual_root.input, request.input);
        assert_eq!(
            actual_root.output.size,
            Size::new(S::from_f64(100.0), S::from_f64(40.0))
        );
        let provider = driver.provider(&driver.stack[0]);
        assert!(
            matches!(provider.validate_qualifications(&replay, &selection(driver, driver.scopes.scopes[0].units.len()), 0),
            Err(ComputeStop::Layout(error)) if matches!(error.kind(), LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)))
        );
        assert!(driver.stack[0].progress.borrow().completed.is_empty());
    });
}
#[test]
fn scope_entry_same_input_changed_body_refuses_before_accounting_f32() {
    changed_same_input_body::<f32>();
}
#[test]
fn scope_entry_same_input_changed_body_refuses_before_accounting_f64() {
    changed_same_input_body::<f64>();
}

fn trimmed_margin_body<S: LayoutScalar>() {
    let block = || NodeInputOf::<S> {
        display: Display::Block,
        ..Default::default()
    };
    let mut first = block();
    first.size.height = PreferredSizeOf::px(S::from_f64(10.0));
    first.margin.top = LengthAutoOf::px(S::from_f64(5.0));
    let mut second = block();
    second.size.height = PreferredSizeOf::px(S::from_f64(10.0));
    let tree = PublicLayoutTreeOf::new()
        .style(0, block())
        .style(1, first)
        .style(2, second)
        .children(0, [1, 2]);
    with_tree(&tree, |driver| {
        let input = pending(driver, 100.0).input.with_fragment_allocation(
            Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(20.0))),
            PhysicalAxis::Vertical,
        );
        let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
        let request = crate::block::scope::root(&mut session, 0, input).unwrap();
        let first = driver.body(&driver.stack[0], request, 0).unwrap();
        assert_eq!(
            first
                .output
                .block_margin_collapse
                .at(PhysicalSide::Top)
                .resolve(),
            S::from_f64(5.0)
        );
        retain(driver, request, 0);
        let complete = selection(driver, driver.scopes.scopes[0].units.len());
        let view = ScopeView::new(
            &tree,
            driver.scopes,
            &ViewKey {
                scope: ScopeId(0),
                frame: 0,
                range: complete.range.clone(),
                partial: None,
                resumed: None,
                opening: true,
                closing: None,
                trimmed: TrimContextId::EMPTY,
            },
            driver.environment.axes,
            &[crate::block::fragment_margins::MarginOwner {
                node: 1,
                side: PhysicalSide::Top,
            }],
        );
        let replay = crate::engine::qualify_block_scope_body(
            &view,
            crate::engine::BlockScopeEntryRequest {
                pending: request,
                binding: crate::engine::BlockFrameBinding {
                    allocations: Vec::new(),
                    intrinsics: driver
                        .environment
                        .intrinsics(driver.environment.frames[0])
                        .unwrap(),
                    dispatch: driver.dispatch(&driver.stack[0]),
                },
            },
        )
        .unwrap();
        assert_eq!(replay.output.size, first.output.size);
        assert_eq!(
            replay
                .output
                .block_margin_collapse
                .at(PhysicalSide::Top)
                .resolve(),
            S::ZERO
        );
        assert_eq!(
            replay
                .packet
                .bodies
                .iter()
                .find(|body| body.node == 0)
                .unwrap()
                .input,
            request.input
        );
        let provider = driver.provider(&driver.stack[0]);
        assert!(
            matches!(provider.validate_qualifications(&replay.packet, &complete, 0),
            Err(ComputeStop::Layout(error)) if matches!(error.kind(), LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)))
        );
    });
}
#[test]
fn scope_entry_equal_size_trimmed_margin_refuses_changed_boundary_f32() {
    trimmed_margin_body::<f32>();
}
#[test]
fn scope_entry_equal_size_trimmed_margin_refuses_changed_boundary_f64() {
    trimmed_margin_body::<f64>();
}
