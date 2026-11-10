use super::*;
use crate::{
    RenderAccountingOptions, RenderAttemptStatus, RenderClock, RenderPhase, RenderProfile,
    RenderResource, RenderWork,
};
#[cfg(feature = "render-window")]
use crate::{
    SurfaceIdentityMismatchKind,
    backend::{DeviceSlotIdentity, presented_configuration_count_for_test},
    surface::DisplayFreePresentedSurfaceObservationForTest,
};
use surgeist_ledger::Dimension;

fn options() -> Options {
    Options::default().with_accounting(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(|| Duration::ZERO)),
    )
}

fn assert_readback_report(renderer: &Renderer, code: ErrorCode) {
    let report = renderer.last_readback_attempt().unwrap();
    let attempt = report.attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Failed(code));
    assert_eq!(attempt.status(), report.status());
    assert_eq!(attempt.phase_count(RenderPhase::Readback), 1);
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::Readback),
        Some(Duration::ZERO)
    );
    assert!(report.generation().is_none());
    assert_eq!(report.accounting_diagnostic(), None);
    assert_eq!(report.cleanup_fault(), None);
    for &work in RenderWork::ALL {
        assert_eq!(attempt.usage().work(work), 0, "attempt {work:?}");
    }
}

#[cfg(feature = "render-window")]
#[derive(Debug, PartialEq)]
struct PresentedObservation {
    attachment: String,
    device: DeviceSlotIdentity,
    target: u64,
    resources: Option<u64>,
    configurations: usize,
    lifecycle: PresentedLifecycle,
    availability: SurfaceState,
    options: SurfaceOptions,
    parameters: Option<Parameters>,
    presentation: DisplayFreePresentedSurfaceObservationForTest,
    headless_publications: usize,
}

#[cfg(feature = "render-window")]
fn observe(surface: &Surface) -> PresentedObservation {
    let Attachment::WebCanvas(canvas) = &surface.attachment else {
        panic!("display-free fixture must retain its canvas descriptor");
    };
    PresentedObservation {
        attachment: canvas.id().to_owned(),
        device: presented_device_identity_for_test(surface),
        target: presented_target_identity_for_test(surface),
        resources: presented_resource_id_for_test(surface),
        configurations: presented_configuration_count_for_test(surface),
        lifecycle: presented_lifecycle_for_test(surface),
        availability: surface.state(),
        options: surface.options,
        parameters: surface.last_parameters,
        presentation: presented_observation_for_test(surface),
        headless_publications: surface.headless_publication_count_for_test(),
    }
}

#[cfg(feature = "render-window")]
fn assert_readback_refusal(renderer: &mut Renderer, surface: &Surface, code: ErrorCode) -> Error {
    let cpu = renderer.cpu_accounting_snapshot().unwrap().usage();
    let generation = renderer.device_accounting_snapshot(None).unwrap();
    let stats = renderer.stats();
    let frame = renderer
        .last_attempt()
        .map(|report| report.snapshot_for_test());

    let error = pollster::block_on(renderer.read_headless(surface)).unwrap_err();
    assert_eq!(error.code(), code);
    assert_readback_report(renderer, code);
    let report = renderer.last_readback_attempt().unwrap();
    let interval = report.renderer_cpu().unwrap().usage();
    for &work in RenderWork::ALL {
        assert_eq!(interval.work(work), 0, "CPU {work:?}");
    }
    for &resource in RenderResource::ALL {
        assert_eq!(interval.opening_live(resource), cpu.live(resource));
        assert_eq!(interval.live(resource), cpu.live(resource));
    }
    assert_eq!(renderer.cpu_accounting_snapshot().unwrap().usage(), cpu);
    assert_eq!(
        renderer.device_accounting_snapshot(None).unwrap(),
        generation
    );
    assert_eq!(renderer.last_attempt().as_deref(), frame.as_ref());
    assert_eq!(renderer.stats(), stats);
    error
}

#[cfg(feature = "render-window")]
#[test]
fn presented_readback_checks_renderer_then_backend_before_device_or_availability() {
    let mut owner = pollster::block_on(Renderer::new(options())).unwrap();
    let mut foreign = pollster::block_on(Renderer::new(options())).unwrap();
    let mut surface = configured_display_free_presented_surface_for_test(&mut owner);
    surface.suspend().unwrap();
    let SurfaceBackend::Presented {
        device_identity, ..
    } = &mut surface.backend
    else {
        panic!("display-free fixture must have a presented backend");
    };
    device_identity.mark_stale_for_test();
    let stale = owner
        .device_accounting_snapshot(Some(&surface))
        .unwrap_err();
    assert_eq!(
        stale
            .runtime_capability_unavailable_diagnostic()
            .unwrap()
            .reason(),
        RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch {
            kind: SurfaceIdentityMismatchKind::StaleDeviceGeneration,
        }
    );
    let before = observe(&surface);
    assert_eq!(before.availability, SurfaceState::Suspended);
    assert!(before.resources.is_some());

    let error = assert_readback_refusal(&mut owner, &surface, ErrorCode::UnsupportedBackend);
    assert_eq!(
        error.message(),
        "surface backend does not support this operation"
    );
    assert_eq!(observe(&surface), before);

    let error = assert_readback_refusal(
        &mut foreign,
        &surface,
        ErrorCode::RuntimeCapabilityUnavailable,
    );
    let diagnostic = error.runtime_capability_unavailable_diagnostic().unwrap();
    assert_eq!(diagnostic.operation(), RuntimeOperation::SurfaceReadback);
    assert_eq!(
        diagnostic.reason(),
        RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch {
            kind: SurfaceIdentityMismatchKind::ForeignRenderer,
        }
    );
    assert_eq!(observe(&surface), before);
}

#[cfg(feature = "render-window")]
#[test]
fn public_presented_resume_keeps_the_available_installed_surface() {
    let mut renderer = pollster::block_on(Renderer::new(options())).unwrap();
    let mut surface = configured_display_free_presented_surface_for_test(&mut renderer);
    let before = observe(&surface);
    let cpu = renderer.cpu_accounting_snapshot().unwrap();
    let generation = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
    let stats = renderer.stats();
    assert_eq!(before.availability, SurfaceState::Available);
    assert!(matches!(before.lifecycle, PresentedLifecycle::Ready { .. }));
    assert!(before.resources.is_some());
    let replacement_id = "alternate-resume-descriptor";
    assert_ne!(before.attachment, replacement_id);

    pollster::block_on(
        renderer.resume_surface(&mut surface, Attachment::from_web_canvas(replacement_id)),
    )
    .unwrap();

    assert_eq!(observe(&surface), before);
    assert_eq!(renderer.cpu_accounting_snapshot().unwrap(), cpu);
    assert_eq!(
        renderer.device_accounting_snapshot(Some(&surface)).unwrap(),
        generation
    );
    assert_eq!(renderer.stats(), stats);
    assert!(renderer.last_attempt().is_none());
    assert!(renderer.last_readback_attempt().is_none());
}

#[test]
fn suspended_ready_readback_observes_retirement_before_returning_suspension() {
    use crate::tests::support::{assert_runtime_device_lost, assert_surface_unavailable};

    let mut renderer = pollster::block_on(Renderer::new(options())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
    let frame = renderer.last_attempt().unwrap().snapshot_for_test();
    let publication = headless_publication_texture_for_test(&surface);
    let publications = surface.headless_publication_count_for_test();
    let parameters = surface.last_parameters;
    assert_eq!(publications, 1);
    assert_eq!(surface.resource_state(), SurfaceResourceState::Ready);
    let before = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
    assert!(!before.retired());
    assert!(before.usage().live(RenderResource::GpuBytes) > 0);

    surface.suspend().unwrap();
    renderer.signal_default_device_loss_for_test(DeviceLossReason::Destroyed);
    assert_eq!(
        renderer.device_accounting_snapshot(Some(&surface)).unwrap(),
        before
    );

    let error = pollster::block_on(renderer.read_headless(&surface)).unwrap_err();
    assert_surface_unavailable(
        error,
        RuntimeOperation::SurfaceReadback,
        RenderSurfaceAvailability::Suspended,
    );
    assert_readback_report(&renderer, ErrorCode::RuntimeCapabilityUnavailable);
    let retired = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
    assert!(retired.retired());
    assert_eq!(retired.identity(), before.identity());
    for &work in RenderWork::ALL {
        assert_eq!(
            retired.usage().work(work) - before.usage().work(work),
            u64::from(work == RenderWork::ResourceRetirements),
            "Generation {work:?}"
        );
    }
    assert_eq!(retired.usage().live(RenderResource::GpuBytes), 0);
    assert_eq!(surface.state(), SurfaceState::Suspended);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(
        *headless_publication_texture_for_test(&surface),
        *publication
    );
    assert_eq!(surface.last_parameters, parameters);
    assert_eq!(*renderer.last_attempt().unwrap(), frame);
    assert_eq!(renderer.stats(), stats);

    surface.resume(Attachment::Headless).unwrap();
    let error = pollster::block_on(renderer.read_headless(&surface)).unwrap_err();
    assert_runtime_device_lost(
        error,
        RuntimeOperation::SurfaceReadback,
        DeviceLossReason::Destroyed,
    );
    assert_readback_report(&renderer, ErrorCode::RuntimeCapabilityUnavailable);
    assert_eq!(
        renderer.device_accounting_snapshot(Some(&surface)).unwrap(),
        retired
    );
    assert_eq!(surface.state(), SurfaceState::Available);
    assert_eq!(surface.resource_state(), SurfaceResourceState::Ready);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(
        *headless_publication_texture_for_test(&surface),
        *publication
    );
    assert_eq!(surface.last_parameters, parameters);
    assert_eq!(*renderer.last_attempt().unwrap(), frame);
    assert_eq!(renderer.stats(), stats);
}
