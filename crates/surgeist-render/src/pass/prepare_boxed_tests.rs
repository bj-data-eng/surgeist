use super::*;
use crate::accounting::{
    FrameAttempt, FrameReportSlot, OwnedBox, RenderAccountingOptions, RenderCpuScope, RenderLimits,
    RenderResource, RendererCpu,
};
use crate::backend::{Backend, RendererExactSurfaceGraph as ExactGraph};
use crate::{
    Antialiasing, BackdropCaptureBounds, BackdropFilterInput, Capabilities, Color, FilterAmount,
    FilterList, FilterOp, Layer, Rect, Scene, Size,
};

fn source(kind: usize, caps: &DeviceCapabilities, account: &Account) -> OwnedBox<ExactGraph> {
    let mut scene = Scene::new();
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    if kind == 0 {
        scene.fill(bounds, Color::BLACK);
    } else {
        let layer = if kind == 1 {
            Layer::new().with_composition(
                crate::LayerComposition::try_porter_duff(
                    crate::PorterDuffOperator::Clear,
                    crate::BlendMode::Normal,
                )
                .unwrap(),
            )
        } else {
            scene.fill(bounds, Color::BLACK);
            let filter = FilterList::try_ops(vec![FilterOp::brightness(
                FilterAmount::try_new(1.25).unwrap(),
            )])
            .unwrap();
            Layer::new()
                .try_backdrop_filter(
                    BackdropFilterInput::try_new(
                        filter,
                        BackdropCaptureBounds::try_new(bounds).unwrap(),
                        None,
                    )
                    .unwrap(),
                )
                .unwrap()
        };
        scene.layer(layer, |child| {
            child.fill(bounds, Color::BLACK);
        });
    }
    let commands = scene.normalize(Capabilities::CURRENT, account).unwrap();
    let context = crate::frame::FrameContext::try_new(
        Size::new(2.0, 2.0),
        1.0,
        Antialiasing::Area,
        Color::TRANSPARENT,
    )
    .unwrap();
    let graph = if kind == 0 {
        OwnedBox::new(
            crate::frame::forced_base_graph_for_test(commands, context).unwrap(),
            &account.storage(),
        )
        .unwrap()
    } else {
        let crate::frame::FramePlan::GpuGraph(graph) = commands.plan_for(context, account).unwrap()
        else {
            panic!("fixture requires a graph")
        };
        graph
    };
    let format = caps
        .resolve_effect_working_format(EffectQualityPolicy::AllowReducedPrecision)
        .unwrap();
    let lowered =
        LoweredGraphPlan::try_lower_validated_graph(&graph, format, Format::Rgba8, caps, account)
            .unwrap();
    let exact = match PrePreparationGraphClassification::classify(lowered, account).unwrap() {
        PrePreparationGraphClassification::ExactBase(base) if kind == 0 => ExactGraph::Base(base),
        PrePreparationGraphClassification::ExactComposition(closed) if kind == 1 => {
            ExactGraph::Composition(
                CompositionPreparableGraph::try_from_closed(closed)
                    .ok()
                    .unwrap(),
            )
        }
        PrePreparationGraphClassification::ExactBackdrop(backdrop) if kind == 2 => {
            ExactGraph::Backdrop(backdrop)
        }
        _ => panic!("fixture must retain its selected exact family"),
    };
    OwnedBox::new(exact, &account.storage()).unwrap()
}

fn view(graph: &ExactGraph) -> GraphPreparationSource<'_> {
    match graph {
        ExactGraph::Base(base) => GraphPreparationSource::Base(base),
        ExactGraph::Composition(composition) => {
            GraphPreparationSource::Composition(composition.closed())
        }
        ExactGraph::Backdrop(backdrop) => GraphPreparationSource::Backdrop(backdrop),
    }
}

fn surface(identity: crate::backend::DeviceSlotIdentity) -> crate::Surface {
    crate::Surface::with_backend(
        crate::Attachment::Headless,
        crate::SurfaceOptions {
            size: Size::new(2.0, 2.0),
            ..crate::SurfaceOptions::default()
        },
        crate::surface::SurfaceBackend::Headless {
            device_identity: identity,
            resources: crate::surface::HeadlessResources::for_physical_size(PhysicalSize::new(
                2, 2,
            )),
            physical_size: PhysicalSize::new(2, 2),
        },
        crate::surface::RendererIdentity::new(),
    )
}

#[test]
fn tiny_forced_base_boxed_execution_publishes_literal_black_pixels() {
    let mut renderer = pollster::block_on(crate::Renderer::new(crate::Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    let result = pollster::block_on(renderer.render_forced_base_graph_for_test(
        &mut surface,
        &scene,
        crate::Parameters::default(),
        WorkingFormat::HighPrecision,
    ))
    .unwrap();
    assert_eq!(result.output_extent, PhysicalSize::new(2, 2));
    assert_eq!(result.stats.route, Some(crate::RenderRoute::GpuGraph));
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(
        pixels.rgba(),
        &[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]
    );
    assert_eq!(surface.headless_publication_count_for_test(), 1);
}

#[test]
fn exact_execution_disposes_source_before_downstream_wait_and_cancellation() {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    let device_cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let mut backend = Backend::new(
        crate::ResourceCacheBudget::DISABLED,
        device_cpu,
        RenderAccountingOptions::new(),
    )
    .unwrap();
    let identity = pollster::block_on(backend.select_device(None))
        .unwrap()
        .expect("real adapter");
    let caps = backend.device_capabilities(identity).unwrap();
    let mut pending_count = 0;
    let mut completed = Vec::new();
    for kind in 0..3 {
        let cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let attempt = FrameAttempt::begin(
            FrameReportSlot::default(),
            &cpu,
            RenderAccountingOptions::new(),
        )
        .unwrap();
        let account = attempt.account();
        let source = source(kind, &caps, account);
        let (lowered, ..) = view(&source).parts();
        let resources_live = lowered.resources.allocation_liveness_for_test();
        let passes_live = lowered.passes.allocation_liveness_for_test();
        let surface = surface(identity);
        let wait = crate::backend::hold_exact_preparation_for_test();
        {
            let future = crate::backend::render_renderer_exact_headless_graph_surface(
                &mut backend,
                &surface,
                source,
                account,
            );
            let mut future = std::pin::pin!(future);
            assert!(
                resources_live() && passes_live(),
                "unpolled future retains its actual input"
            );
            let poll = future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()));
            assert!(
                !resources_live() && !passes_live(),
                "synchronous preparation releases source before its first wait"
            );
            match poll {
                Poll::Pending => {
                    pending_count += 1;
                }
                Poll::Ready(result) => {
                    drop(result.unwrap());
                }
            }
        }
        drop(wait);
        assert_eq!(surface.headless_publication_count_for_test(), 0);
        let ready = backend
            .ready_device_state_borrow_for_test(identity)
            .unwrap();
        let after = ready.internal_resource_manager_observation_for_test();
        assert_eq!((after.active_frame_count, after.leased_count), (0, 0));
        drop(attempt);
        completed.push((cpu, baseline));
    }
    drop(backend);
    for (cpu, baseline) in completed {
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline
        );
    }
    eprintln!("prepared native resource cancellations at explicit test wait={pending_count}");
    assert_eq!(
        pending_count, 3,
        "each exact family must cancel with real prepared resources live"
    );
}

#[test]
fn source_disposal_fault_precedes_submission_and_preserves_prior_semantic_error() {
    use crate::accounting::{RenderClock, RenderProfile, RenderWork};
    use std::{cell::RefCell, time::Duration};
    thread_local! { static DISPOSED: RefCell<Option<Box<dyn Fn() -> bool>>> = RefCell::new(None); }
    fn clock() -> Duration {
        DISPOSED.with(|slot| {
            let probe = slot.borrow_mut().take();
            if let Some(probe) = probe
                && !probe()
            {
                *slot.borrow_mut() = Some(probe);
            }
        });
        Duration::from_millis(10)
    }
    let device_cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let mut backend = Backend::new(
        crate::ResourceCacheBudget::DISABLED,
        device_cpu,
        RenderAccountingOptions::new(),
    )
    .unwrap();
    let identity = pollster::block_on(backend.select_device(None))
        .unwrap()
        .expect("real adapter");
    let caps = backend.device_capabilities(identity).unwrap();
    for early_error in [false, true] {
        let options = RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock));
        let cpu = RendererCpu::new(options).unwrap();
        let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
        let account = attempt.account();
        let source = source(0, &caps, account);
        let (lowered, ..) = view(&source).parts();
        let resources_live = lowered.resources.allocation_liveness_for_test();
        let passes_live = lowered.passes.allocation_liveness_for_test();
        let cleanup_cpu = cpu.at(RenderCpuScope::FrameBacking);
        DISPOSED.with(|probe| {
            *probe.borrow_mut() = Some(Box::new(move || {
                if !resources_live() && !passes_live() {
                    cleanup_cpu.note_fault(crate::RenderAccountingError::Invariant);
                    true
                } else {
                    false
                }
            }))
        });
        let mut surface = surface(identity);
        if early_error {
            surface.options.format = Format::Bgra8;
        }
        let error = pollster::block_on(
            crate::backend::render_renderer_exact_headless_graph_surface(
                &mut backend,
                &surface,
                source,
                account,
            ),
        )
        .err()
        .expect("source disposal injects its CPU fault before execution continues");
        let fault = cpu
            .snapshot()
            .unwrap()
            .fault()
            .expect("actual disposal callback fault is retained");
        if early_error {
            assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
            assert_eq!(error.accounting_diagnostic(), None);
        } else {
            assert_eq!(error.accounting_diagnostic(), Some(fault));
        }
        assert_eq!(account.work_count(RenderWork::Submissions).unwrap(), 0);
        assert_eq!(surface.headless_publication_count_for_test(), 0);
        DISPOSED.with(|probe| *probe.borrow_mut() = None);
        let (report, cleanup) = attempt.finish(
            crate::RenderAttemptStatus::Failed(error.code()),
            error.accounting_diagnostic(),
        );
        assert_eq!(cleanup, Some(fault));
        assert_eq!(report.cleanup_fault(), Some(fault));
        let after = backend
            .ready_device_state_borrow_for_test(identity)
            .unwrap()
            .internal_resource_manager_observation_for_test();
        assert_eq!((after.active_frame_count, after.leased_count), (0, 0));
    }
}

#[test]
fn borrowed_exact_sources_retain_fact_identity_and_release_lists_before_prepared_output() {
    let device_cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let mut backend = Backend::new(
        crate::ResourceCacheBudget::DISABLED,
        device_cpu.clone(),
        RenderAccountingOptions::new(),
    )
    .unwrap();
    let identity = pollster::block_on(backend.select_device(None))
        .unwrap()
        .expect("real adapter");
    let caps = backend.device_capabilities(identity).unwrap();
    let ready = backend
        .ready_device_state_borrow_for_test(identity)
        .unwrap();
    let resources = ResourceManager::new(crate::ResourceCacheBudget::DISABLED);
    let cache = DevicePassCache::new(
        resources.generation().clone(),
        device_cpu.at(RenderCpuScope::DeviceBacking),
    );
    for kind in 0..3 {
        let cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let attempt = FrameAttempt::begin(
            FrameReportSlot::default(),
            &cpu,
            RenderAccountingOptions::new(),
        )
        .unwrap();
        let account = attempt.account();
        let source = source(kind, &caps, account);
        let (lowered, base, composition, _, _, backdrop) = view(&source).parts();
        let resources_live = lowered.resources.allocation_liveness_for_test();
        let passes_live = lowered.passes.allocation_liveness_for_test();
        let source_bytes = std::mem::size_of::<ExactGraph>()
            + lowered.resources.capacity() * std::mem::size_of::<RuntimeResourceRequest>()
            + SharedValue::<Vec<RuntimeResourceRequest>>::allocation_size_for_test()
            + lowered.passes.capacity() * std::mem::size_of::<RuntimePass>()
            + SharedValue::<Vec<RuntimePass>>::allocation_size_for_test();
        eprintln!(
            "family={kind}, source-box/list bytes={source_bytes}, resource-capacity={}, pass-capacity={}, resource-record={}, pass-record={}, shared-vec-payload={}",
            lowered.resources.capacity(),
            lowered.passes.capacity(),
            std::mem::size_of::<RuntimeResourceRequest>(),
            std::mem::size_of::<RuntimePass>(),
            SharedValue::<Vec<RuntimePass>>::allocation_size_for_test()
        );
        let first_pass = lowered.passes[0].clone();
        let first_resource = lowered.resources[0].clone();
        let source_live = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let prepared = PreparedGraph::try_prepare_inner(
            view(&source),
            source.working_format(),
            &caps,
            (ready.device_for_test(), ready.queue_for_test()),
            &resources,
            (&cache, false),
            account,
        )
        .unwrap();
        assert!(resources_live() && passes_live());
        let overlap = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        assert!(
            overlap > source_live,
            "source and real derived requests coexist"
        );
        assert_eq!(prepared.plan.passes[0].runtime, first_pass);
        assert_eq!(prepared.plan.resources[0].runtime, first_resource);
        if let Some(base) = base {
            assert!(
                base.captures
                    .same_allocation(&prepared.base_execution.as_ref().unwrap().captures)
            );
        }
        if let Some(facts) = composition {
            assert!(facts.same_allocation(prepared.composition_execution.as_ref().unwrap()));
        }
        if let Some(facts) = backdrop {
            assert!(facts.same_allocation(prepared.backdrop_execution.as_ref().unwrap()));
        }
        let format = source.working_format();
        drop(source);
        assert!(
            !resources_live() && !passes_live(),
            "prepared output cannot retain source lists"
        );
        assert_eq!(
            prepared.output_extent(account).unwrap(),
            PhysicalSize::new(2, 2)
        );
        assert_eq!(prepared.output_format(), Format::Rgba8);
        assert_eq!(prepared.working_format(), format);
        assert_eq!(
            overlap
                - cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
            source_bytes as u64,
            "physical source box and both actual source-list allocations release only after preparation"
        );
        eprintln!(
            "family={kind}, box wrapper={}, exact payload={}, source-live={source_live}, source/request-live={overlap}, peak={}",
            std::mem::size_of::<OwnedBox<ExactGraph>>(),
            std::mem::size_of::<ExactGraph>(),
            cpu.snapshot()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes)
        );
        drop(prepared);
        drop((first_pass, first_resource));
        drop(attempt);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline
        );
        assert_eq!(resources.observation_for_test().leased_count, 0);
    }
}

#[test]
fn borrowed_source_request_overlap_refuses_one_byte_blocker_and_retries_same_source() {
    let device_cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let mut backend = Backend::new(
        crate::ResourceCacheBudget::DISABLED,
        device_cpu.clone(),
        RenderAccountingOptions::new(),
    )
    .unwrap();
    let identity = pollster::block_on(backend.select_device(None))
        .unwrap()
        .expect("real adapter");
    let caps = backend.device_capabilities(identity).unwrap();
    let ready = backend
        .ready_device_state_borrow_for_test(identity)
        .unwrap();
    for kind in 0..3 {
        let measure = |limit: u64, block: bool| {
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(limit));
            let cpu = RendererCpu::new(options).unwrap();
            let baseline = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
            let account = attempt.account();
            let source = source(kind, &caps, account);
            let resources = ResourceManager::new(crate::ResourceCacheBudget::DISABLED);
            let cache = DevicePassCache::new(
                resources.generation().clone(),
                device_cpu.at(RenderCpuScope::DeviceBacking),
            );
            let before = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let blocker = block.then(|| {
                AccountedVec::<u8>::new_in(&account.storage(), 1, RenderResource::RetainedCpuBytes)
                    .unwrap()
            });
            let prepare = || {
                PreparedGraph::try_prepare_inner(
                    view(&source),
                    source.working_format(),
                    &caps,
                    (ready.device_for_test(), ready.queue_for_test()),
                    &resources,
                    (&cache, false),
                    account,
                )
            };
            let result = prepare();
            if block {
                let error = result
                    .err()
                    .expect("one real retained byte must prevent exact peak admission");
                assert!(matches!(
                    error.accounting_diagnostic(),
                    Some(crate::RenderAccountingError::CpuRefused { .. })
                ));
                assert_eq!(
                    cpu.snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    before + 1
                );
                assert_eq!(resources.observation_for_test().leased_count, 0);
                drop(blocker);
                let retry =
                    prepare().expect("unchanged borrowed input retries after blocker disposal");
                assert_eq!(
                    retry.output_extent(account).unwrap(),
                    PhysicalSize::new(2, 2)
                );
                drop(retry);
            } else {
                drop(result.unwrap());
            }
            let peak = cpu
                .snapshot()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes);
            drop(source);
            drop(attempt);
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                baseline
            );
            peak
        };
        // This measured peak checks budget consistency. The separate source-disposal
        // test derives its physical release oracle from actual layouts/capacities.
        let exact = measure(u64::MAX, false);
        assert_eq!(measure(exact, false), exact);
        assert_eq!(measure(exact, true), exact);
    }
}
