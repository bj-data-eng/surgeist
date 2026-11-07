//! Performed baseline custody belongs to its actual body, request and stage.
use super::*;
use crate::grid::{
    GridMeasurementBoundary, GridSizingPhase, GridSizingStage,
    compute_grid_with_scrollbar_settlement,
};
use core::convert::Infallible;

#[derive(Clone, Copy)]
struct Consumption<S: LayoutScalar> {
    child: u32,
    axes: FlowAxes,
    output: ComputeOutputOf<S>,
    input: ComputeInputOf<S>,
    stage: GridSizingStage,
}

fn performed_actor<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    batch: &CompletedLayoutBatchOf<u32, S>,
    child: u32,
    body: Size<S>,
    space: Size<S>,
) -> (Consumption<S>, ExecutedBaselineInventory<u32, S>) {
    assert!(Traverse::children(tree, 2).any(|node| node == child));
    assert_eq!(LayoutTree::node_input(tree, child).display, Display::Grid);
    assert!(!LayoutTree::has_leaf_measurement(tree, child));
    let parent_input = LayoutTree::node_input(tree, 2);
    let parent_axes = FlowAxes::new(parent_input.writing_mode, parent_input.direction);
    let child_input = LayoutTree::node_input(tree, child);
    let axes = FlowAxes::new(child_input.writing_mode, child_input.direction);
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        body.map(Some),
        normal_box(batch, 2).content_box_size().map(Some),
        ContainingLayoutContext::new(parent_axes, ParentFormattingContext::Grid),
        SizingRequestOf::new(
            space.map(|extent| {
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(extent).unwrap()),
                    SizingOperation::Normal,
                )
            }),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    let stage = GridSizingStage::COMPLETE;
    let mut session = crate::engine::ComputeSession::new(tree, Vec::new());
    // The existing private result owner retains the actual inventory instead of
    // discarding it at the public ComputeOutput boundary. No packet is patched.
    let invocation = GridInvocation {
        container: crate::grid::input::GridContainerInput::from_node(session.node_input(child)),
        boundary: GridMeasurementBoundary::Ordinary,
    };
    let result = compute_grid_with_scrollbar_settlement(
        &mut session,
        child,
        input,
        input.settled_auto_scrollbars(),
        |_| Ok(GridParentContext::none()),
        invocation,
        stage,
    )
    .expect("the real fixed native actor performs its own Grid layout");
    assert_eq!(result.output.size, body);
    assert_eq!(
        result.output.baselines().first_block(axes),
        Some(body.width - value(20.0))
    );
    assert_eq!(
        result.output.baselines().last_block(axes),
        Some(body.width - value(50.0))
    );
    let source = result
        .executed_baselines
        .expect("a performed Grid result retains its execution inventory");
    let consumption = Consumption {
        child,
        axes,
        output: result.output,
        input,
        stage,
    };
    assert_eq!(
        source.validate_execution(child, axes, result.output.size, input, stage),
        Ok(())
    );
    (consumption, source)
}

fn assert_observable_refusal<S: LayoutScalar>(
    error: LayoutErrorOf<u32, S, Infallible>,
    child: u32,
) {
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 2,
            subject: child
        }
    );
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridBaselineInheritance),
    );
}

fn consume<S: LayoutScalar>(
    input: Consumption<S>,
    source: Option<ExecutedBaselineInventory<u32, S>>,
) -> LayoutResultOf<u32, ExecutedBaselineInventory<u32, S>, S, Infallible> {
    require_executed_baselines(
        2,
        input.child,
        input.axes,
        input.output,
        input.input,
        input.stage,
        source,
    )
}

#[derive(Clone, Copy, Debug)]
enum Refusal {
    Missing,
    Body,
    Request,
    Stage,
    Owner,
}

fn qualifies_performed_custody<S: LayoutScalar>(refusal: Refusal) {
    let FixedCompositeQualification { tree, batch, .. } =
        qualified_fixed_composites::<S>(WritingMode::VerticalRl);
    let before: Vec<_> = batch
        .unrounded_entries()
        .iter()
        .map(|entry| (entry.node(), entry.output()))
        .collect();
    let body = normal_box(&batch, 10).size;
    assert_eq!(body, physical_size(WritingMode::VerticalRl, 40.0, 50.0));
    assert_eq!(normal_box(&batch, 11).size, body);
    let (control, source) = performed_actor(&tree, &batch, 10, body, body);
    let accepted = consume(control, Some(source))
        .expect("the exact actual body, owner, request and stage are admitted");

    let consumer = match refusal {
        Refusal::Missing | Refusal::Stage => control,
        Refusal::Body => {
            // A different known border allocation executes the same
            // fixed actor; its output is not manufactured from the old packet.
            let other_body = Size::new(value(60.0), body.height);
            let (other, _) = performed_actor(&tree, &batch, 10, other_body, other_body);
            assert_ne!(other.output.size, control.output.size);
            other
        }
        Refusal::Request => {
            // Known body50x40 stays fixed while the actual available width is60.
            // Same returned dimensions cannot transfer another request's proof.
            let (other, _) =
                performed_actor(&tree, &batch, 10, body, Size::new(value(60.0), body.height));
            assert_eq!(other.output.size, control.output.size);
            assert_ne!(other.input, control.input);
            other
        }
        Refusal::Owner => {
            // The real sibling actor has the same fixed border and ports.
            let (other, _) = performed_actor(&tree, &batch, 11, body, body);
            assert_eq!(other.output.size, control.output.size);
            assert_eq!(other.output.baselines(), control.output.baselines());
            other
        }
    };
    let consumer = if matches!(refusal, Refusal::Stage) {
        Consumption {
            // This changes the consumer's claimed complete sizing order, not
            // the inventory or the output produced in Column-first.
            stage: GridSizingStage {
                first_axis: GridAxisKind::Row,
                through: GridSizingPhase::Complete,
            },
            ..consumer
        }
    } else {
        consumer
    };
    if matches!(refusal, Refusal::Missing) {
        assert_observable_refusal(consume(consumer, None).unwrap_err(), consumer.child);
    } else {
        let (_, stale) = performed_actor(&tree, &batch, 10, body, body);
        assert_eq!(
            stale.validate_execution(
                consumer.child,
                consumer.axes,
                consumer.output.size,
                consumer.input,
                consumer.stage,
            ),
            Err(InheritedCurrentGridBaselinePlacementError::ExecutedSourceMismatch),
            "{refusal:?} cannot consume another performed execution's custody",
        );
        assert_observable_refusal(consume(consumer, Some(stale)).unwrap_err(), consumer.child);
    }

    assert_eq!(
        accepted.validate_execution(
            control.child,
            control.axes,
            control.output.size,
            control.input,
            control.stage,
        ),
        Ok(()),
        "a refusal preserves the prior admitted execution"
    );
    assert_eq!(
        batch
            .unrounded_entries()
            .iter()
            .map(|entry| (entry.node(), entry.output()))
            .collect::<Vec<_>>(),
        before,
        "the previously completed native result is unchanged",
    );
    let retry = completed(&tree);
    assert_eq!(
        retry
            .unrounded_entries()
            .iter()
            .map(|entry| (entry.node(), entry.output()))
            .collect::<Vec<_>>(),
        before,
        "a real valid retry preserves the original full unrounded geometry",
    );
    let (again, fresh) = performed_actor(&tree, &batch, 10, body, body);
    consume(again, Some(fresh)).expect("fresh performed custody is usable after the refusal");
}

macro_rules! custody_case {
    ($name:ident, $scalar:ty, $refusal:ident) => {
        #[test]
        fn $name() {
            qualifies_performed_custody::<$scalar>(Refusal::$refusal);
        }
    };
}
custody_case!(
    missing_performed_inventory_refuses_at_consumer_f32,
    f32,
    Missing
);
custody_case!(
    missing_performed_inventory_refuses_at_consumer_f64,
    f64,
    Missing
);
custody_case!(stale_body_inventory_refuses_at_consumer_f32, f32, Body);
custody_case!(stale_body_inventory_refuses_at_consumer_f64, f64, Body);
custody_case!(
    different_request_inventory_refuses_at_consumer_f32,
    f32,
    Request
);
custody_case!(
    different_request_inventory_refuses_at_consumer_f64,
    f64,
    Request
);
custody_case!(
    different_stage_inventory_refuses_at_consumer_f32,
    f32,
    Stage
);
custody_case!(
    different_stage_inventory_refuses_at_consumer_f64,
    f64,
    Stage
);
custody_case!(foreign_owner_inventory_refuses_at_consumer_f32, f32, Owner);
custody_case!(foreign_owner_inventory_refuses_at_consumer_f64, f64, Owner);
