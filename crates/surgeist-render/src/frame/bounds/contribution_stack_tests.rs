use super::*;
use crate::{
    BlendMode, Capabilities, Layer, LayerComposition, PorterDuffOperator, RenderAccountingContext,
    RenderAccountingError, RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus,
    RenderLimits, RenderPhase, RenderQuantity, RenderResource, Scene,
};

fn frame_context() -> FrameContext {
    FrameContext::try_new(Size::new(2.0, 2.0), 1.0, Antialiasing::Area, Color::BLACK).unwrap()
}

#[test]
fn branching_contribution_preserves_siblings_around_removed_empty_layers() {
    use crate::command::{RenderPaint, RenderShape};
    let rects = [
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Rect::new(1.0, 0.0, 1.0, 1.0),
        Rect::new(0.0, 1.0, 1.0, 1.0),
        Rect::new(1.0, 1.0, 1.0, 1.0),
    ];
    let mut scene = Scene::new();
    scene.fill(rects[0], Color::BLACK);
    scene.layer(Layer::new(), |scene| {
        scene.fill(rects[1], Color::BLACK);
        scene.layer(Layer::new(), |_| {});
        scene.fill(rects[2], Color::BLACK);
    });
    scene.fill(rects[3], Color::BLACK);
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let frame = frame_context();
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            let contribution = SemanticSourceContribution::try_from_commands(
                commands,
                frame.initial_parent_contribution(),
                SemanticContributionDomain::RootOutputBounded(frame.output_bounds),
                frame,
                Transform::identity(),
                account,
            )?;
            let [first, RenderCommand::Layer { children, .. }, last] =
                contribution.commands.as_ref()
            else {
                panic!("expected outer siblings around one contributing wrapper");
            };
            let [inner_first, inner_last] = children.as_ref() else {
                panic!("expected empty inner wrapper removal without sibling loss");
            };
            for (actual, rect) in [first, inner_first, inner_last, last]
                .into_iter()
                .zip(rects)
            {
                assert_eq!(
                    actual,
                    &RenderCommand::Fill {
                        shape: RenderShape::Rect(rect),
                        paint: RenderPaint::Color(Color::BLACK)
                    }
                );
            }
            for bounds in [
                contribution.source_bounds,
                contribution.affected_bounds,
                contribution.current_parent,
            ] {
                assert_eq!(
                    bounds
                        .require_non_empty_for_graph("expected filled bounds")?
                        .unwrap()
                        .rect(),
                    Rect::new(0.0, 0.0, 2.0, 2.0)
                );
            }
            Ok(())
        })
        .unwrap();
    assert_released(&context);
}

#[test]
fn resumed_wrapper_receives_the_actual_preceding_sibling_parent() {
    let covered = Rect::new(0.0, 0.0, 1.0, 1.0);
    let mut scene = Scene::new();
    scene.fill(covered, Color::BLACK);
    scene.layer(Layer::new(), |scene| {
        scene.layer(
            Layer::new().with_composition(
                LayerComposition::try_porter_duff(PorterDuffOperator::SourceIn, BlendMode::Normal)
                    .unwrap(),
            ),
            |scene| {
                scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
            },
        );
    });
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let frame = FrameContext::try_new(
        Size::new(2.0, 2.0),
        1.0,
        Antialiasing::Area,
        Color::TRANSPARENT,
    )
    .unwrap();
    let mut context = RenderAccountingContext::default();
    context
        .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            let contribution = SemanticSourceContribution::try_from_commands(
                commands,
                frame.initial_parent_contribution(),
                SemanticContributionDomain::RootOutputBounded(frame.output_bounds),
                frame,
                Transform::identity(),
                account,
            )?;
            assert_eq!(contribution.commands.len(), 2);
            assert_eq!(
                contribution
                    .current_parent
                    .require_non_empty_for_graph("parent after source-in")?
                    .unwrap()
                    .rect(),
                covered
            );
            Ok(())
        })
        .unwrap();
    assert_released(&context);
}

fn qualify_empty(context: &mut RenderAccountingContext, commands: &RenderCommands) -> Result<()> {
    let frame = frame_context();
    context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
        let contribution = SemanticSourceContribution::try_from_commands(
            commands.clone(),
            frame.initial_parent_contribution(),
            SemanticContributionDomain::RootOutputBounded(frame.output_bounds),
            frame,
            Transform::identity(),
            account,
        )?;
        assert!(contribution.commands.is_empty());
        assert!(contribution.source_bounds.is_exactly_empty());
        assert!(contribution.affected_bounds.is_exactly_empty());
        assert_eq!(
            contribution.current_parent,
            frame.initial_parent_contribution()
        );
        Ok(())
    })
}

fn assert_released(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
}

#[test]
fn empty_contribution_work_refusal_releases_parents_and_retries() {
    const DEPTH: u64 = 128;
    let mut scene = Scene::new();
    for _ in 0..DEPTH {
        let child = scene;
        scene = Scene::new();
        scene.layer(Layer::new(), |scene| *scene = child);
    }
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    qualify_empty(&mut context, &commands).unwrap();
    assert_released(&context);
    let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
    // One qualification per layer and descendant scans of N-1 down to zero.
    let visits = DEPTH * (DEPTH + 1) / 2;
    assert_eq!(usage.work(RenderWork::CommandVisits), visits);
    let scratch = usage.peak(RenderResource::ScratchCpuBytes);
    assert!(scratch > 0);
    assert!(usage.peak(RenderResource::RetainedCpuBytes) > 0);

    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CommandVisits, visits - 1)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    let error = qualify_empty(&mut context, &commands).unwrap_err();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let refusal = report.attempt().unwrap().refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal))
    );
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::CommandVisits)
    );
    assert_eq!(refusal.current(), visits - 1);
    assert_eq!(refusal.amount(), 1);
    assert_released(&context);

    context.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_scratch_bytes(scratch - 1)),
    );
    let error = qualify_empty(&mut context, &commands).unwrap_err();
    let report = context.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let refusal = report.attempt().unwrap().refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal))
    );
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Resource(RenderResource::ScratchCpuBytes)
    );
    assert_released(&context);

    let limits = RenderLimits::new()
        .with_scratch_bytes(scratch)
        .with_work_limit(RenderWork::CommandVisits, visits)
        .unwrap();
    context.set_options(RenderAccountingOptions::new().with_limits(limits));
    qualify_empty(&mut context, &commands).unwrap();
    assert_released(&context);
}
