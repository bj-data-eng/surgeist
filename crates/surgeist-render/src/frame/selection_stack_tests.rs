use super::*;
use crate::{
    BlendMode, Capabilities, ErrorCode, FontRef, Layer, LayerComposition, PorterDuffOperator,
    RenderAccountingContext, RenderAccountingError, RenderAccountingOptions, RenderAttemptKind,
    RenderAttemptStatus, RenderLimits, RenderQuantity, RenderResource, Scene, TextGlyph, TextPaint,
    TextRun, TextRunBounds,
};

fn normalized_scene(depth: u64) -> RenderCommands {
    let mut scene = Scene::new();
    scene.layer(Layer::new().isolated(), |scene| {
        scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    });
    for _ in 0..depth {
        let child = scene;
        scene = Scene::new();
        scene.layer(Layer::new(), |scene| *scene = child);
    }
    scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap()
}

fn check_walk(depth: u64, collect: bool) {
    std::thread::Builder::new()
        .name(
            if collect {
                "collect graph requirements"
            } else {
                "validate graph text bounds"
            }
            .into(),
        )
        .stack_size(512 * 1024)
        .spawn(move || {
            let commands = normalized_scene(depth);
            let mut context = RenderAccountingContext::default();
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                    if collect {
                        let requirements = graph_selection_requirements(&commands, account)?;
                        assert_eq!(
                            requirements.as_slice(),
                            &[GraphSelectionRequirement::ExplicitIsolation]
                        );
                    } else {
                        let mut path = account.temporary(0)?;
                        require_graph_text_bounds(&commands, &mut path, account)?;
                        assert!(path.is_empty());
                    }
                    Ok(())
                })
                .unwrap();
            let report = context.last_attempt().unwrap();
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::CommandVisits), depth + 2);
            for resource in [
                RenderResource::ScratchCpuBytes,
                RenderResource::RetainedCpuBytes,
            ] {
                assert_eq!(usage.live(resource), 0);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn shallow_graph_requirements_preserve_the_isolated_descendant() {
    check_walk(2, true);
}

#[test]
fn deep_graph_requirements_preserve_the_isolated_descendant() {
    check_walk(8_192, true);
}

#[test]
fn shallow_graph_text_validation_restores_the_path() {
    check_walk(2, false);
}

#[test]
fn deep_graph_text_validation_restores_the_path() {
    check_walk(8_192, false);
}

fn assert_released(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
}

fn run_branches(
    context: &mut RenderAccountingContext,
    commands: &RenderCommands,
    collect: bool,
) -> Result<()> {
    context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
        if collect {
            let requirements = graph_selection_requirements(commands, account)?;
            assert_eq!(
                requirements.as_slice(),
                &[
                    GraphSelectionRequirement::ExplicitIsolation,
                    GraphSelectionRequirement::BoundedPorterDuff,
                ]
            );
        } else {
            let mut path = account.temporary(2)?;
            path.push(7)?;
            path.push(3)?;
            require_graph_text_bounds(commands, &mut path, account)?;
            assert_eq!(path.as_slice(), &[7, 3]);
        }
        Ok(())
    })
}

#[test]
fn branching_preflight_preserves_requirements_and_retries_after_budgets() {
    let mut scene = Scene::new();
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    scene.layer(Layer::new(), |scene| {
        for layer in [
            Layer::new().isolated(),
            Layer::new().with_composition(
                LayerComposition::try_porter_duff(PorterDuffOperator::SourceIn, BlendMode::Normal)
                    .unwrap(),
            ),
            Layer::new().isolated(),
        ] {
            scene.layer(layer, |scene| {
                scene.fill(rect, Color::BLACK);
            });
        }
    });
    scene.fill(rect, Color::BLACK);
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    for collect in [false, true] {
        let mut context = RenderAccountingContext::default();
        run_branches(&mut context, &commands, collect).unwrap();
        assert_released(&context);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        // Outer wrapper, three child layers/fills, then the outer sibling.
        assert_eq!(usage.work(RenderWork::CommandVisits), 8);
        let scratch = usage.peak(RenderResource::ScratchCpuBytes);
        assert!(scratch > 0);

        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CommandVisits, 7)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        let error = run_branches(&mut context, &commands, collect).unwrap_err();
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
        assert_eq!(refusal.current(), 7);
        assert_eq!(refusal.amount(), 1);
        assert_released(&context);

        context.set_options(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(scratch - 1)),
        );
        let error = run_branches(&mut context, &commands, collect).unwrap_err();
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
            .with_work_limit(RenderWork::CommandVisits, 8)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        run_branches(&mut context, &commands, collect).unwrap();
        assert_released(&context);
    }
}

fn unspecified_text(scene: &mut Scene) {
    let glyphs = [TextGlyph::try_new(1, 1.0, 2.0, 5.0).unwrap()];
    scene.text_run(
        TextRun::try_new(
            FontRef::new(41),
            16.0,
            Transform::identity(),
            TextPaint::try_fill(Color::BLACK.into()).unwrap(),
            &glyphs,
            TextRunBounds::unspecified(),
        )
        .unwrap(),
    );
}

#[test]
fn text_diagnostic_resumes_correct_sibling_after_a_chain() {
    let rect = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut chain = Scene::new();
    chain.fill(rect, Color::BLACK);
    for _ in 0..3 {
        let child = chain;
        chain = Scene::new();
        chain.layer(Layer::new(), |scene| *scene = child);
    }
    let mut scene = Scene::new();
    scene.fill(rect, Color::BLACK);
    scene.layer(Layer::new(), |scene| {
        scene.layer(Layer::new(), |scene| *scene = chain);
        scene.layer(Layer::new(), |scene| {
            scene.fill(rect, Color::BLACK);
            unspecified_text(scene);
            unspecified_text(scene);
        });
    });
    unspecified_text(&mut scene);
    let commands = scene
        .normalize(Capabilities::CURRENT, &Account::for_test())
        .unwrap();
    let mut context = RenderAccountingContext::default();
    let error = context
        .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            require_graph_text_bounds(&commands, &mut account.temporary(0)?, account)
        })
        .unwrap_err();
    let diagnostic = error.unresolved_resource_diagnostic().unwrap();
    assert_eq!(diagnostic.kind(), UnresolvedResourceKind::TextRunInkBounds);
    assert_eq!(diagnostic.identifier(), "normalized command 1.1.1");
    let report = context.last_attempt().unwrap();
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(ErrorCode::UnresolvedResource)
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandVisits),
        10
    );
    assert_released(&context);
    for cap in [9, 10] {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CommandVisits, cap)
            .unwrap();
        context.set_options(RenderAccountingOptions::new().with_limits(limits));
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                require_graph_text_bounds(&commands, &mut account.temporary(0)?, account)
            })
            .unwrap_err();
        if cap == 9 {
            let refusal = context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .refusal()
                .unwrap();
            assert_eq!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::Refused(refusal))
            );
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::CommandVisits)
            );
            assert_eq!(refusal.current(), 9);
            assert_eq!(refusal.amount(), 1);
        } else {
            assert_eq!(
                error.unresolved_resource_diagnostic().unwrap().identifier(),
                "normalized command 1.1.1"
            );
        }
        assert_released(&context);
    }
}
