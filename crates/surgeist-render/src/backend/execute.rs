#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use super::present;
use super::{Backend, DeviceSlotIdentity, create_headless_texture};
use crate::accounting::{Account, Generation, GpuObjectKind, OwnedBox, OwnedGpu, RenderPhase};

use crate::pass::{
    BasePreparableGraph, CompositionPreparableGraph, ForegroundBlurPreparableGraph,
    GraphExternalOutputView, PreparedGraph,
};
use crate::resource::{FrameCleanup, WorkingFormat};
use crate::surface::{HeadlessPublication, SurfaceBackend};
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use crate::surface::{PresentedSurface, PresentedSurfaceState};
use crate::vello_engine::{
    ActiveVelloEncodingScope, EncodedVelloPass, RasterParameters, TransactionEncodingState,
    TransactionTargetIntent, scene::VelloScene,
};
use crate::{
    Antialiasing, BackendErrorCode, Color, Error, Format, Parameters, PhysicalSize, Result,
    RuntimeCapabilityUnavailable, RuntimeOperation, Surface,
    gpu_transaction::{
        GpuOperationStage, GpuOperationTransaction, GraphOutputCommit, GraphSubmissionPayload,
        InternalVelloPayload,
    },
    shader::ProvisionalDevicePassCacheUpdate,
};
use std::sync::Arc;

pub(super) fn create_frame_encoder(
    device: &wgpu::Device,
    generation: &Generation,
    label: &'static str,
    account: &Account,
) -> Result<OwnedGpu<wgpu::CommandEncoder>> {
    let account = account.at(RenderPhase::GpuPrepare);
    account.phase(RenderPhase::GpuPrepare, || {
        generation.opaque(GpuObjectKind::CommandEncoder, Some(&account), || {
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
        })
    })
}
pub(super) fn finish_frame_encoder(
    encoder: OwnedGpu<wgpu::CommandEncoder>,
    account: &Account,
) -> Result<OwnedGpu<wgpu::CommandBuffer>> {
    let account = account.at(RenderPhase::Encode);
    account.phase(RenderPhase::Encode, || encoder.finish(Some(&account)))
}

/// One validated exact graph selected for atomic surface execution.
#[must_use = "an exact surface graph must enter its GPU transaction"]
pub(crate) enum ExactSurfaceGraph {
    Base(BasePreparableGraph),
    Composition(CompositionPreparableGraph),
    Backdrop(crate::pass::BackdropPreparableGraph),
    ForegroundBlur(ForegroundBlurPreparableGraph),
}

#[cfg(test)]
thread_local! {
    static HOLD_EXACT_PREPARATION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn hold_exact_preparation_for_test() -> impl Drop {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            HOLD_EXACT_PREPARATION.set(self.0);
        }
    }
    Reset(HOLD_EXACT_PREPARATION.replace(true))
}

/// A deterministic downstream wait with the real prepared graph and native
/// acquisitions live. Native work can otherwise complete on its first poll.
#[cfg(test)]
async fn exact_preparation_wait_for_test() {
    if HOLD_EXACT_PREPARATION.get() {
        std::future::pending::<()>().await;
    }
}

impl ExactSurfaceGraph {
    pub(crate) fn working_format(&self) -> WorkingFormat {
        match self {
            Self::Base(preparable) => preparable.working_format(),
            Self::Composition(preparable) => preparable.working_format(),
            Self::Backdrop(preparable) => preparable.working_format(),
            Self::ForegroundBlur(preparable) => preparable.spatial_filter().working_format(),
        }
    }

    pub(crate) fn output_format(&self) -> Format {
        match self {
            Self::Base(preparable) => preparable.output_format(),
            Self::Composition(preparable) => preparable.output_format(),
            Self::Backdrop(preparable) => preparable.output_format(),
            Self::ForegroundBlur(preparable) => preparable.spatial_filter().output_format(),
        }
    }

    pub(super) fn known_output_extent(&self, account: &Account) -> Result<Option<PhysicalSize>> {
        match self {
            Self::Base(preparable) => preparable.output_extent(account).map(Some),
            Self::Composition(_) => Ok(None),
            Self::Backdrop(preparable) => preparable.output_extent(account).map(Some),
            Self::ForegroundBlur(preparable) => {
                preparable.spatial_filter().output_extent(account).map(Some)
            }
        }
    }
}

/// Exact texture-bound raster stage shared by direct surface and local offscreen execution.
pub(super) struct InternalVelloTextureTarget<'a> {
    view: &'a wgpu::TextureView,
    extent: PhysicalSize,
    usage: wgpu::TextureUsages,
}

impl<'a> InternalVelloTextureTarget<'a> {
    pub(super) const fn new(
        view: &'a wgpu::TextureView,
        extent: PhysicalSize,
        usage: wgpu::TextureUsages,
    ) -> Self {
        Self {
            view,
            extent,
            usage,
        }
    }
}

pub(super) struct InternalVelloTextureStage<'a> {
    identity: DeviceSlotIdentity,
    operation: RuntimeOperation,
    scene: &'a VelloScene,
    target: InternalVelloTextureTarget<'a>,
    base_color: Color,
    antialiasing: Antialiasing,
}

impl<'a> InternalVelloTextureStage<'a> {
    pub(super) fn new(
        identity: DeviceSlotIdentity,
        operation: RuntimeOperation,
        scene: &'a VelloScene,
        target: InternalVelloTextureTarget<'a>,
        base_color: Color,
        antialiasing: Antialiasing,
    ) -> Self {
        Self {
            identity,
            operation,
            scene,
            target,
            base_color,
            antialiasing,
        }
    }
}

impl Backend {
    fn create_headless_surface_texture(
        &mut self,
        identity: DeviceSlotIdentity,
        physical_size: PhysicalSize,
        format: Format,
        account: &Account,
    ) -> Result<(OwnedGpu<wgpu::Texture>, OwnedGpu<wgpu::TextureView>)> {
        let ready = self.ready_state_mut(
            identity,
            RuntimeOperation::SurfaceRendering,
            BackendErrorCode::RenderFailed,
            "headless Vello device resources are unavailable before allocation",
        )?;
        create_headless_texture(
            &ready.device,
            ready.resources.generation(),
            physical_size,
            format,
            Some(account),
        )
    }

    pub(super) async fn render_internal_vello_to_texture(
        &mut self,
        transaction: GpuOperationTransaction,
        stage: InternalVelloTextureStage<'_>,
        account: &Account,
    ) -> Result<()> {
        let prepared = stage.scene.prepare_raster(
            RasterParameters::try_new(
                stage.target.extent,
                peniko::Color::from(stage.base_color),
                stage.antialiasing,
            )?,
            account,
        )?;
        {
            let ready = self.ready_state_mut(
                stage.identity,
                stage.operation,
                BackendErrorCode::RenderFailed,
                "internal Vello device resources are unavailable before rendering",
            )?;
            let mut command_encoder = create_frame_encoder(
                &ready.device,
                ready.resources.generation(),
                "Surgeist internal Vello frame encoder",
                account,
            )?;
            let mut scope = ActiveVelloEncodingScope::begin(&ready.device);
            let encode_account = account.at(RenderPhase::Encode);
            let encoded: EncodedVelloPass = account.phase(RenderPhase::Encode, || {
                let mut encoding = TransactionEncodingState::new(
                    &mut scope,
                    &ready.queue,
                    &mut command_encoder,
                    stage.target.view,
                    TransactionTargetIntent::new(
                        stage.target.extent,
                        wgpu::TextureFormat::Rgba8Unorm,
                        stage.target.usage,
                    ),
                    &encode_account,
                );
                prepared
                    .encode_into(&ready.engine, &ready.resources, &mut encoding)
                    .map_err(|failure| failure.into_error_and_aborted_resources().0)
            })?;
            let (lease, logical_pass) = encoded.into_resources_and_logical_pass();
            let resolved = scope.finish_with_lease(lease, account).await;
            let lease = match resolved {
                Ok(lease) => lease,
                Err(failure) => {
                    return Err(failure.into_error_and_aborted_resources().0);
                }
            };
            let payload = InternalVelloPayload::new(
                finish_frame_encoder(command_encoder, account)?,
                crate::vello_engine::PendingVelloResourceCommit::new(lease)?,
                logical_pass,
            );
            transaction
                .submit_internal_vello(
                    &ready.device,
                    &ready.queue,
                    payload,
                    stage.operation,
                    account,
                )
                .await?;
        }
        self.commit_checked_pass_cache_update(stage.identity, None, stage.operation, account)
    }

    pub(super) fn commit_checked_pass_cache_update(
        &mut self,
        identity: DeviceSlotIdentity,
        update: Option<ProvisionalDevicePassCacheUpdate>,
        operation: RuntimeOperation,
        account: &Account,
    ) -> Result<()> {
        let Some(update) = update else {
            return Ok(());
        };
        let ready = self.ready_state_mut(
            identity,
            operation,
            BackendErrorCode::RenderFailed,
            "checked core pass objects lost their persistent device cache",
        )?;
        update.commit_into(&mut ready.pass_cache, account)
    }

    pub(crate) fn begin_gpu_operation(
        &mut self,
        identity: DeviceSlotIdentity,
        stage: GpuOperationStage,
        operation: RuntimeOperation,
    ) -> Result<GpuOperationTransaction> {
        let state = self.device_states.get_mut(identity.slot()).ok_or_else(|| {
            Error::new(
                stage.error_code(),
                "GPU device slot disappeared before transaction setup",
            )
        })?;
        if state.generation != identity.generation {
            return Err(Error::new(
                stage.error_code(),
                "GPU device generation changed before transaction setup",
            ));
        }
        if let Some(terminal) = state.terminal() {
            return Err(terminal.error(operation));
        }
        state.next_operation_generation = state
            .next_operation_generation
            .checked_add(1)
            .ok_or_else(|| {
                Error::invalid_value(
                    "GPU operation generation",
                    state.next_operation_generation,
                    "must have remaining generation space",
                )
            })?;
        let signal = Arc::clone(&state.signal);
        let ready = state.ready().ok_or_else(|| {
            Error::new(
                stage.error_code(),
                "GPU device slot disappeared before transaction scopes",
            )
        })?;
        Ok(GpuOperationTransaction::begin(
            &ready.device,
            signal,
            state.next_operation_generation,
            stage,
        ))
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "base graph calls the validated prepared-graph handoff before execution"
        )
    )]
    pub(crate) fn prepare_graph_resources(
        &mut self,
        identity: DeviceSlotIdentity,
        lowered: crate::pass::LoweredGraphPlan,
        policy: crate::EffectQualityPolicy,
        account: &Account,
    ) -> Result<PreparedGraph<'_>> {
        let state = self.device_states.get_mut(identity.slot()).ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "GPU device slot is unavailable for graph preparation",
            )
        })?;
        if state.generation != identity.generation {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "GPU device generation changed before graph preparation",
            ));
        }
        if let Some(terminal) = state.terminal() {
            return Err(terminal.error(RuntimeOperation::EffectRendering));
        }
        let capabilities = state.capabilities;
        let realize_checked_passes = state.signal.has_active_operation();
        let ready = state.ready().ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "ready GPU device resources disappeared before graph preparation",
            )
        })?;
        PreparedGraph::try_prepare(
            lowered,
            policy,
            &capabilities,
            (&ready.device, &ready.queue),
            &ready.resources,
            (&ready.pass_cache, realize_checked_passes),
            account,
        )
        .map(|prepared| prepared.with_vello_engine(&ready.engine))
    }

    fn prepare_exact_surface_graph_resources(
        &mut self,
        identity: DeviceSlotIdentity,
        graph: &ExactSurfaceGraph,
        account: &Account,
    ) -> Result<PreparedGraph<'_>> {
        let state = self.device_states.get_mut(identity.slot()).ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "GPU device slot is unavailable for exact graph preparation",
            )
        })?;
        if state.generation != identity.generation {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "GPU device generation changed before exact graph preparation",
            ));
        }
        if let Some(terminal) = state.terminal() {
            return Err(terminal.error(RuntimeOperation::SurfaceRendering));
        }
        if !state.signal.has_active_operation() {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "exact graph preparation requires one active GPU transaction",
            ));
        }
        let selected_working_format = graph.working_format();
        let capabilities = state
            .capabilities
            .for_selected_working_format(selected_working_format)?;
        let ready = state.ready().ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "ready GPU device resources disappeared before exact graph preparation",
            )
        })?;
        let prepared = match graph {
            ExactSurfaceGraph::Base(preparable) => {
                PreparedGraph::try_prepare_base_with_working_format(
                    preparable,
                    selected_working_format,
                    &capabilities,
                    (&ready.device, &ready.queue),
                    &ready.resources,
                    (&ready.pass_cache, true),
                    account,
                )
            }
            ExactSurfaceGraph::Composition(preparable) => PreparedGraph::try_prepare_composition(
                preparable,
                &capabilities,
                (&ready.device, &ready.queue),
                &ready.resources,
                (&ready.pass_cache, true),
                account,
            ),
            ExactSurfaceGraph::Backdrop(preparable) => PreparedGraph::try_prepare_backdrop(
                preparable,
                selected_working_format,
                &capabilities,
                (&ready.device, &ready.queue),
                &ready.resources,
                (&ready.pass_cache, true),
                account,
            ),
            ExactSurfaceGraph::ForegroundBlur(preparable) => {
                PreparedGraph::try_prepare_spatial_filter(
                    preparable.spatial_filter(),
                    selected_working_format,
                    &capabilities,
                    (&ready.device, &ready.queue),
                    &ready.resources,
                    (&ready.pass_cache, true),
                    account,
                )
            }
        }?
        .with_vello_engine(&ready.engine);
        Ok(prepared)
    }
}

/// Private result of a clean frame transaction, held until the renderer publishes it.
#[must_use = "clean frame results must be committed or dropped"]
pub(crate) struct SurfaceFrameCommit {
    headless_publication: Option<HeadlessPublication>,
    _frame_cleanup: Option<FrameCleanup>,
    working_format: Option<WorkingFormat>,
}
impl SurfaceFrameCommit {
    pub(super) fn without_headless_publication() -> Self {
        Self {
            headless_publication: None,
            _frame_cleanup: None,
            working_format: None,
        }
    }
    fn headless(publication: HeadlessPublication) -> Self {
        Self {
            headless_publication: Some(publication),
            _frame_cleanup: None,
            working_format: None,
        }
    }
    pub(super) fn headless_graph(
        publication: HeadlessPublication,
        frame_cleanup: FrameCleanup,
        working_format: WorkingFormat,
    ) -> Self {
        Self {
            headless_publication: Some(publication),
            _frame_cleanup: Some(frame_cleanup),
            working_format: Some(working_format),
        }
    }
    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    pub(super) fn presented_graph(
        frame_cleanup: FrameCleanup,
        working_format: WorkingFormat,
    ) -> Self {
        Self {
            headless_publication: None,
            _frame_cleanup: Some(frame_cleanup),
            working_format: Some(working_format),
        }
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<HeadlessPublication>,
        Option<FrameCleanup>,
        Option<WorkingFormat>,
    ) {
        (
            self.headless_publication,
            self._frame_cleanup,
            self.working_format,
        )
    }
}

pub(crate) async fn render_exact_headless_graph_surface(
    backend: &mut Backend,
    surface: &Surface,
    graph: OwnedBox<ExactSurfaceGraph>,
    account: &Account,
) -> Result<SurfaceFrameCommit> {
    let (device_identity, physical_size, selected_working_format) =
        exact_headless_graph_target(surface, &graph, account)?;
    let capabilities = backend
        .device_capabilities(device_identity)
        .ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "the exact graph executor lost immutable device capabilities",
            )
        })?;
    capabilities.validate_supported_working_format(selected_working_format)?;

    let generation = backend.generation(device_identity)?.clone();
    let transaction = backend.begin_gpu_operation(
        device_identity,
        GpuOperationStage::Render,
        RuntimeOperation::SurfaceRendering,
    )?;
    let (device, queue) = {
        let ready = backend.ready_state_mut(
            device_identity,
            RuntimeOperation::SurfaceRendering,
            BackendErrorCode::RenderFailed,
            "the exact graph executor lost its ready device before draft allocation",
        )?;
        (ready.device.clone(), ready.queue.clone())
    };
    let (draft_texture, draft_view) = create_headless_texture(
        &device,
        &generation,
        physical_size,
        surface.options.format,
        Some(account),
    )?;
    let mut prepared =
        backend.prepare_exact_surface_graph_resources(device_identity, &graph, account)?;
    // Preparation owns independent requests and shallow execution facts. Dispose
    // the actual source before any asynchronous execution or acquisition wait.
    drop(graph);
    account.cpu()?.ensure_healthy()?;
    #[cfg(test)]
    exact_preparation_wait_for_test().await;
    if prepared.output_extent(account)? != physical_size
        || prepared.output_format() != surface.options.format
        || prepared.working_format() != selected_working_format
    {
        return Err(Error::new(
            BackendErrorCode::RenderFailed,
            "prepared exact graph output changed after eligibility validation",
        ));
    }
    let mut encoder = create_frame_encoder(
        &device,
        &generation,
        "Surgeist exact headless graph encoder",
        account,
    )?;
    let pending_encoding = prepared
        .encode_custom_spine(
            &mut encoder,
            GraphExternalOutputView::try_new(&draft_view, surface.options.format, physical_size)?,
            account,
        )
        .await;
    let pending_encoding = pending_encoding?;
    let prepared_submission = prepared.finish_graph_submission(pending_encoding, account)?;
    let payload = GraphSubmissionPayload::new(
        finish_frame_encoder(encoder, account)?,
        prepared_submission,
        HeadlessPublication::new(draft_texture),
    );
    let clean = {
        let ready = backend.ready_state_mut(
            device_identity,
            RuntimeOperation::SurfaceRendering,
            BackendErrorCode::RenderFailed,
            "the exact graph executor lost its ready device before submission",
        )?;
        transaction
            .submit_base_graph(
                &device,
                &queue,
                &mut ready.pass_cache,
                payload,
                RuntimeOperation::SurfaceRendering,
                account,
            )
            .await?
    };
    let (output, frame_cleanup) = clean.into_parts();
    #[cfg(not(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    )))]
    let GraphOutputCommit::Headless(publication) = output;
    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    let publication = match output {
        GraphOutputCommit::Headless(publication) => publication,
        GraphOutputCommit::Presented => {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "the headless exact graph transaction returned a presented host effect",
            ));
        }
    };
    Ok(SurfaceFrameCommit::headless_graph(
        publication,
        frame_cleanup,
        selected_working_format,
    ))
}

fn exact_headless_graph_target(
    surface: &Surface,
    graph: &ExactSurfaceGraph,
    account: &Account,
) -> Result<(DeviceSlotIdentity, PhysicalSize, WorkingFormat)> {
    let selected_working_format = graph.working_format();
    let graph_output_format = graph.output_format();
    let known_output_extent = graph.known_output_extent(account)?;
    let (device_identity, physical_size) = match &surface.backend {
        SurfaceBackend::Headless {
            device_identity,
            physical_size,
            ..
        } => (*device_identity, *physical_size),
        SurfaceBackend::ContractOnly { .. } => {
            return Err(Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "the exact graph executor requires a device-backed headless surface",
            ));
        }
        #[cfg(any(
            feature = "render-window",
            all(feature = "render-web", target_arch = "wasm32")
        ))]
        SurfaceBackend::Presented { .. } => {
            return Err(Error::new(
                BackendErrorCode::UnsupportedBackend,
                "presented exact graph execution requires the presented executor",
            ));
        }
    };
    if physical_size.width() == 0
        || physical_size.height() == 0
        || surface.options.format != Format::Rgba8
        || graph_output_format != surface.options.format
        || known_output_extent.is_some_and(|extent| extent != physical_size)
    {
        return Err(Error::new(
            BackendErrorCode::RenderFailed,
            "the headless draft differs from the exact eligible graph output",
        ));
    }
    Ok((device_identity, physical_size, selected_working_format))
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
pub(crate) async fn render_exact_presented_graph_surface(
    backend: &mut Backend,
    surface: &mut Surface,
    graph: OwnedBox<ExactSurfaceGraph>,
    account: &Account,
) -> Result<SurfaceFrameCommit> {
    let (target, device_identity, physical_size, output_format, selected_working_format) =
        present::exact_presented_graph_target(
            surface,
            graph.working_format(),
            graph.output_format(),
            graph.known_output_extent(account)?,
        )?;
    let capabilities = backend
        .device_capabilities(device_identity)
        .ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "the presented exact graph executor lost immutable device capabilities",
            )
        })?;
    capabilities.validate_supported_working_format(selected_working_format)?;

    let generation = backend.generation(device_identity)?.clone();
    let transaction = backend.begin_gpu_operation(
        device_identity,
        GpuOperationStage::Render,
        RuntimeOperation::SurfaceRendering,
    )?;
    let (device, queue) = {
        let ready = backend.ready_state_mut(
            device_identity,
            RuntimeOperation::SurfaceRendering,
            BackendErrorCode::RenderFailed,
            "the presented exact graph lost its ready device before preparation",
        )?;
        (ready.device.clone(), ready.queue.clone())
    };
    let prepared =
        backend.prepare_exact_surface_graph_resources(device_identity, &graph, account)?;
    // Preparation owns independent requests and shallow execution facts. Dispose
    // the actual source before any asynchronous execution or acquisition wait.
    drop(graph);
    account.cpu()?.ensure_healthy()?;
    #[cfg(test)]
    exact_preparation_wait_for_test().await;
    if prepared.output_extent(account)? != physical_size
        || prepared.output_format() != output_format
        || prepared.working_format() != selected_working_format
    {
        return Err(Error::new(
            BackendErrorCode::RenderFailed,
            "prepared presented exact graph output changed after eligibility validation",
        ));
    }

    let acquired = target
        .acquire(&device, prepared, transaction, account)
        .await?;
    let (acquired, mut prepared, transaction) = acquired;
    let output_view = account.phase(RenderPhase::GpuPrepare, || {
        acquired.create_view(&generation, Some(&account.at(RenderPhase::GpuPrepare)))
    })?;
    let mut encoder = create_frame_encoder(
        &device,
        &generation,
        "Surgeist exact presented graph encoder",
        account,
    )?;
    let pending_encoding = prepared
        .encode_custom_spine(
            &mut encoder,
            GraphExternalOutputView::try_new(&output_view, output_format, physical_size)?,
            account,
        )
        .await;
    let pending_encoding = pending_encoding?;
    let prepared_submission = prepared.finish_graph_submission(pending_encoding, account)?;
    drop(output_view);
    let payload = GraphSubmissionPayload::presented(
        finish_frame_encoder(encoder, account)?,
        prepared_submission,
        acquired,
    );
    let clean = {
        let ready = backend.ready_state_mut(
            device_identity,
            RuntimeOperation::SurfaceRendering,
            BackendErrorCode::RenderFailed,
            "the presented exact graph lost its ready device before submission",
        )?;
        transaction
            .submit_base_graph(
                &device,
                &queue,
                &mut ready.pass_cache,
                payload,
                RuntimeOperation::SurfaceRendering,
                account,
            )
            .await?
    };
    let (output, frame_cleanup) = clean.into_parts();
    if !matches!(output, GraphOutputCommit::Presented) {
        return Err(Error::new(
            BackendErrorCode::PresentFailed,
            "the presented exact graph transaction returned a headless publication",
        ));
    }
    Ok(SurfaceFrameCommit::presented_graph(
        frame_cleanup,
        selected_working_format,
    ))
}

pub(crate) async fn render_internal_vello_surface(
    backend: &mut Backend,
    transaction: GpuOperationTransaction,
    surface: &mut Surface,
    scene: &VelloScene,
    parameters: Parameters,
    antialiasing: Antialiasing,
    account: &Account,
) -> Result<SurfaceFrameCommit> {
    let frame = InternalVelloFrameParameters {
        scene,
        parameters,
        antialiasing,
    };
    match &mut surface.backend {
        SurfaceBackend::ContractOnly { .. } => {
            Ok(SurfaceFrameCommit::without_headless_publication())
        }
        SurfaceBackend::Headless {
            device_identity,
            physical_size,
            ..
        } => {
            render_internal_vello_headless(
                backend,
                transaction,
                *device_identity,
                *physical_size,
                surface.options.format,
                frame,
                account,
            )
            .await
        }
        #[cfg(any(
            feature = "render-window",
            all(feature = "render-web", target_arch = "wasm32")
        ))]
        SurfaceBackend::Presented {
            surface: native,
            device_identity,
            state,
        } => {
            render_internal_vello_presented(
                backend,
                transaction,
                native,
                *device_identity,
                state,
                frame,
                account,
            )
            .await
        }
    }
}

#[derive(Clone, Copy)]
struct InternalVelloFrameParameters<'a> {
    scene: &'a VelloScene,
    parameters: Parameters,
    antialiasing: Antialiasing,
}

async fn render_internal_vello_headless(
    backend: &mut Backend,
    transaction: GpuOperationTransaction,
    device_identity: DeviceSlotIdentity,
    physical_size: PhysicalSize,
    format: Format,
    frame: InternalVelloFrameParameters<'_>,
    account: &Account,
) -> Result<SurfaceFrameCommit> {
    if physical_size.width() == 0 || physical_size.height() == 0 {
        return Ok(SurfaceFrameCommit::without_headless_publication());
    }
    let (texture, view) =
        backend.create_headless_surface_texture(device_identity, physical_size, format, account)?;
    backend
        .render_internal_vello_to_texture(
            transaction,
            InternalVelloTextureStage::new(
                device_identity,
                RuntimeOperation::SurfaceRendering,
                frame.scene,
                InternalVelloTextureTarget::new(
                    &view,
                    physical_size,
                    wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                ),
                frame.parameters.base_color,
                frame.antialiasing,
            ),
            account,
        )
        .await?;
    Ok(SurfaceFrameCommit::headless(HeadlessPublication::new(
        texture,
    )))
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
async fn render_internal_vello_presented(
    backend: &mut Backend,
    transaction: GpuOperationTransaction,
    native: &mut PresentedSurface,
    device_identity: DeviceSlotIdentity,
    state: &mut PresentedSurfaceState,
    frame: InternalVelloFrameParameters<'_>,
    account: &Account,
) -> Result<SurfaceFrameCommit> {
    let Some(resources) = present::internal_vello_presented_resources(native, state)? else {
        return Ok(SurfaceFrameCommit::without_headless_publication());
    };
    let _ = &resources.target_texture;
    backend
        .render_internal_vello_to_texture(
            transaction,
            InternalVelloTextureStage::new(
                device_identity,
                RuntimeOperation::SurfaceRendering,
                frame.scene,
                InternalVelloTextureTarget::new(
                    &resources.target_view,
                    PhysicalSize::new(resources.config.width, resources.config.height),
                    wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                ),
                frame.parameters.base_color,
                frame.antialiasing,
            ),
            account,
        )
        .await?;
    present::present_internal_vello_target(
        backend,
        native,
        device_identity,
        state,
        resources,
        account,
    )
    .await
}
