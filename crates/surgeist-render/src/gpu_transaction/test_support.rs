use crate::accounting::Account;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum ScopePopPosition {
    Validation,
    OutOfMemory,
    Internal,
}

/// A per-owner pause after the real consuming pop. The native result future,
/// unpopped guards and generation lease remain owned by the real resolver.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct ScopePopGate {
    position: ScopePopPosition,
    reached: std::cell::Cell<bool>,
    released: std::cell::Cell<bool>,
    waker: std::cell::RefCell<Option<std::task::Waker>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ScopePopGate {
    pub(crate) fn new(position: ScopePopPosition) -> std::rc::Rc<Self> {
        std::rc::Rc::new(Self {
            position,
            reached: std::cell::Cell::new(false),
            released: std::cell::Cell::new(false),
            waker: std::cell::RefCell::new(None),
        })
    }

    pub(crate) fn reached(&self) -> bool {
        self.reached.get()
    }

    pub(crate) fn release(&self) {
        self.released.set(true);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    pub(super) async fn wait(&self, position: ScopePopPosition) {
        if position != self.position {
            return;
        }
        std::future::poll_fn(|cx| {
            self.reached.set(true);
            if self.released.get() {
                std::task::Poll::Ready(())
            } else {
                *self.waker.borrow_mut() = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        })
        .await;
    }
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
impl GpuOperationTransaction {
    pub(crate) fn with_scope_pop_gate_for_test(mut self, gate: std::rc::Rc<ScopePopGate>) -> Self {
        self.scopes.scope_gate = Some(gate);
        self
    }
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
thread_local! {
    static LEASE_DROP_OBSERVER: std::cell::RefCell<Option<std::rc::Rc<LeaseDropObserverForTest>>> = const { std::cell::RefCell::new(None) };
}

/// One actual lease's disposal event, scoped to its signal and generation.
/// The callback records facts; assertions belong outside destructor execution.
#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
struct LeaseDropObserverForTest {
    signal: Arc<DeviceSignal>,
    generation: u64,
    observe: std::cell::RefCell<Box<dyn FnMut()>>,
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
pub(crate) struct LeaseDropObserverGuardForTest {
    previous: Option<std::rc::Rc<LeaseDropObserverForTest>>,
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
impl LeaseDropObserverGuardForTest {
    pub(crate) fn install(
        signal: Arc<DeviceSignal>,
        generation: u64,
        observe: impl FnMut() + 'static,
    ) -> Self {
        let observer = std::rc::Rc::new(LeaseDropObserverForTest {
            signal,
            generation,
            observe: std::cell::RefCell::new(Box::new(observe)),
        });
        let previous = LEASE_DROP_OBSERVER.with_borrow_mut(|slot| slot.replace(observer));
        Self { previous }
    }
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
impl Drop for LeaseDropObserverGuardForTest {
    fn drop(&mut self) {
        LEASE_DROP_OBSERVER.with_borrow_mut(|slot| *slot = self.previous.take());
    }
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
pub(super) fn observe_lease_drop_for_test(lease: &super::GpuOperationLease) {
    let observer = LEASE_DROP_OBSERVER.with_borrow(Clone::clone);
    if let Some(observer) = observer
        && Arc::ptr_eq(&observer.signal, &lease.signal)
        && observer.generation == lease.generation
    {
        (observer.observe.borrow_mut())();
    }
}

#[cfg(feature = "render-window")]
thread_local! {
    static GRAPH_AUTHORIZATION_PROBE: std::cell::RefCell<Option<std::rc::Rc<GraphAuthorizationProbe>>> = const { std::cell::RefCell::new(None) };
}

/// Scoped to one fixture device on its owning thread. The first matching graph
/// authorization identifies its exact active generation; nested fixtures restore
/// their predecessor on every exit, including unwind.
#[cfg(feature = "render-window")]
pub(super) struct GraphAuthorizationProbe {
    signal: Arc<DeviceSignal>,
    generation: std::cell::Cell<Option<u64>>,
}

#[cfg(feature = "render-window")]
pub(super) struct GraphAuthorizationProbeGuard {
    probe: std::rc::Rc<GraphAuthorizationProbe>,
    previous: Option<std::rc::Rc<GraphAuthorizationProbe>>,
}

#[cfg(feature = "render-window")]
impl GraphAuthorizationProbeGuard {
    pub(super) fn install(signal: Arc<DeviceSignal>) -> Self {
        let probe = std::rc::Rc::new(GraphAuthorizationProbe {
            signal,
            generation: std::cell::Cell::new(None),
        });
        let previous = GRAPH_AUTHORIZATION_PROBE
            .with_borrow_mut(|slot| slot.replace(std::rc::Rc::clone(&probe)));
        Self { probe, previous }
    }

    pub(super) fn generation(&self) -> Option<u64> {
        self.probe.generation.get()
    }
}

#[cfg(feature = "render-window")]
impl Drop for GraphAuthorizationProbeGuard {
    fn drop(&mut self) {
        GRAPH_AUTHORIZATION_PROBE.with_borrow_mut(|slot| *slot = self.previous.take());
    }
}

#[cfg(feature = "render-window")]
pub(super) fn after_graph_authorization_for_test(lease: &super::GpuOperationLease) {
    GRAPH_AUTHORIZATION_PROBE.with_borrow(|slot| {
        if let Some(probe) = slot
            && Arc::ptr_eq(&probe.signal, &lease.signal)
        {
            assert_eq!(
                lease.signal.active_generation_for_test(),
                Some(lease.generation)
            );
            assert_eq!(probe.generation.replace(Some(lease.generation)), None);
            lease.signal.record_uncaptured_fault_for_test(
                crate::GpuFaultKind::Internal,
                "terminal after actual graph authorization",
            );
        }
    });
}

use super::{
    GpuOperationDraft, GpuOperationStage, GpuOperationTransaction, InternalVelloPayload,
    VelloResourceCommitProof,
};
#[cfg(feature = "render-window")]
use crate::DeviceLossReason;
use crate::{
    Result, RuntimeOperation,
    backend::DeviceSignal,
    resource::{ResourceCacheKey, ResourceManager},
    vello_engine::{PendingVelloResourceCommit, VelloResourceAllocationSummaryForTest},
};

/// Exercises the real transaction submission, scope resolution, resource abort,
/// and draft-publication boundaries using only explicit test-owned inputs.
pub(crate) async fn graph_scope_failure_after_submission_for_test(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    signal: Arc<DeviceSignal>,
    resources: &ResourceManager,
    resource_key: ResourceCacheKey,
    publication: &mut Option<u64>,
) -> Result<()> {
    let account = Account::for_test();

    let generation = signal.next_test_generation()?;
    let transaction =
        GpuOperationTransaction::begin(device, signal, generation, GpuOperationStage::Render);
    let mut frame = resources.begin_frame(Some(&account))?;
    let _lease = frame.acquire(resource_key, 16)?;
    frame.discard_on_drop();
    let draft = GpuOperationDraft::new(publication, 2);
    let command_buffer = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surgeist explicit graph transaction failure harness"),
        })
        .finish();
    queue.submit([command_buffer]);
    let _ = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Surgeist explicit graph transaction validation failure"),
        size: wgpu::Extent3d {
            width: 0,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    transaction
        .finish(RuntimeOperation::EffectRendering, Some(&account))
        .await?;
    let _cleanup = frame.finish_checked()?;
    draft.commit();
    Ok(())
}

/// Holds a real submitted transaction, prepared resource frame, and draft at
/// an explicit test-owned cancellation boundary. Dropping the returned future
/// exercises their normal cancellation cleanup.
pub(crate) async fn graph_cancellation_after_submission_for_test(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    signal: Arc<DeviceSignal>,
    resources: &ResourceManager,
    resource_key: ResourceCacheKey,
    publication: &mut Option<u64>,
) -> Result<()> {
    let account = Account::for_test();

    let generation = signal.next_test_generation()?;
    let _transaction =
        GpuOperationTransaction::begin(device, signal, generation, GpuOperationStage::Render);
    let mut frame = resources.begin_frame(Some(&account))?;
    let _lease = frame.acquire(resource_key, 16)?;
    frame.discard_on_drop();
    let _draft = GpuOperationDraft::new(publication, 2);
    let command_buffer = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surgeist explicit graph cancellation harness"),
        })
        .finish();
    queue.submit([command_buffer]);
    std::future::pending::<()>().await;
    Ok(())
}

/// Exercises clean scope resolution followed by an explicit resource-accounting
/// rejection before the draft publication may commit.
pub(crate) async fn graph_accounting_failure_after_submission_for_test(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    signal: Arc<DeviceSignal>,
    resources: &ResourceManager,
    resource_key: ResourceCacheKey,
    publication: &mut Option<u64>,
) -> Result<()> {
    let account = Account::for_test();

    let generation = signal.next_test_generation()?;
    let transaction =
        GpuOperationTransaction::begin(device, signal, generation, GpuOperationStage::Render);
    let mut frame = resources.begin_frame(Some(&account))?;
    let _lease = frame.acquire(resource_key, 16)?;
    frame.discard_on_drop();
    let draft = GpuOperationDraft::new(publication, 2);
    let command_buffer = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surgeist explicit graph accounting failure harness"),
        })
        .finish();
    queue.submit([command_buffer]);
    let _fault = frame.poison_retained_byte_accounting_for_test();
    transaction
        .finish(RuntimeOperation::EffectRendering, Some(&account))
        .await?;
    let _cleanup = frame.finish_checked()?;
    draft.commit();
    Ok(())
}

/// Exercises a terminal device signal after a real submission and before the
/// transaction or its draft publication can commit.
#[cfg(feature = "render-window")]
pub(crate) async fn graph_terminal_loss_after_submission_for_test(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    signal: Arc<DeviceSignal>,
    resources: &ResourceManager,
    resource_key: ResourceCacheKey,
    publication: &mut Option<u64>,
) -> Result<()> {
    let account = Account::for_test();

    let generation = signal.next_test_generation()?;
    let transaction = GpuOperationTransaction::begin(
        device,
        Arc::clone(&signal),
        generation,
        GpuOperationStage::Render,
    );
    let mut frame = resources.begin_frame(Some(&account))?;
    let _lease = frame.acquire(resource_key, 16)?;
    frame.discard_on_drop();
    let draft = GpuOperationDraft::new(publication, 2);
    let command_buffer = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surgeist explicit graph terminal-loss harness"),
        })
        .finish();
    queue.submit([command_buffer]);
    signal.record_loss_for_test(DeviceLossReason::Destroyed);
    transaction
        .finish(RuntimeOperation::EffectRendering, Some(&account))
        .await?;
    let _cleanup = frame.finish_checked()?;
    draft.commit();
    Ok(())
}

/// Explicit test-owned facts around one real generic transaction submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GpuOperationSubmissionObservationForTest {
    queue_submission_count: usize,
    transaction_generation: u64,
    active_generation: Option<u64>,
    scopes_resolved: bool,
}

impl GpuOperationSubmissionObservationForTest {
    pub(crate) const fn queue_submission_count_for_test(&self) -> usize {
        self.queue_submission_count
    }

    pub(crate) const fn transaction_generation_for_test(&self) -> Option<u64> {
        Some(self.transaction_generation)
    }

    pub(crate) const fn active_generation_for_test(&self) -> Option<u64> {
        self.active_generation
    }

    pub(crate) const fn scopes_resolved_for_test(&self) -> bool {
        self.scopes_resolved
    }
}

/// Submits one explicit command buffer through the real transaction and records
/// only facts owned by the test at that stage boundary.
pub(crate) async fn submit_command_buffer_observed_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    command_buffer: crate::accounting::OwnedGpu<wgpu::CommandBuffer>,
    operation: RuntimeOperation,
) -> Result<GpuOperationSubmissionObservationForTest> {
    let account = Account::for_test();

    let transaction_generation = transaction.lease.generation;
    let active_generation = transaction.lease.signal.active_generation_for_test();
    command_buffer.submit(queue, Some(&account))?;
    transaction.finish(operation, Some(&account)).await?;
    Ok(GpuOperationSubmissionObservationForTest {
        queue_submission_count: 1,
        transaction_generation,
        active_generation,
        scopes_resolved: true,
    })
}

pub(crate) async fn submit_command_buffer_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    command_buffer: crate::accounting::OwnedGpu<wgpu::CommandBuffer>,
    operation: RuntimeOperation,
) -> Result<()> {
    submit_command_buffer_observed_for_test(transaction, queue, command_buffer, operation)
        .await
        .map(|_| ())
}

/// Holds a real generic transaction after queue submission. Dropping the future
/// releases the transaction lease without reporting a successful completion.
pub(crate) async fn hold_command_buffer_after_submit_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    command_buffer: crate::accounting::OwnedGpu<wgpu::CommandBuffer>,
) -> Result<()> {
    let account = Account::for_test();

    command_buffer.submit(queue, Some(&account))?;
    let _transaction = transaction;
    std::future::pending::<()>().await;
    Ok(())
}

/// Records a real uncaptured fault only after the explicit queue submission and
/// resolves it through the real transaction error path.
pub(crate) async fn fault_command_buffer_after_submit_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    command_buffer: crate::accounting::OwnedGpu<wgpu::CommandBuffer>,
    signal: &DeviceSignal,
    kind: crate::GpuFaultKind,
    message: &str,
    operation: RuntimeOperation,
) -> Result<()> {
    let account = Account::for_test();

    command_buffer.submit(queue, Some(&account))?;
    signal.record_uncaptured_fault_for_test(kind, message);
    transaction.finish(operation, Some(&account)).await
}

/// Test-owned facts around one real readback submission and scope transition.
#[derive(Clone, Debug)]
pub(crate) struct ReadbackSubmissionObservationForTest {
    queue_submission_count: usize,
    transaction_generation: u64,
    active_generation: Option<u64>,
    submission_index: wgpu::SubmissionIndex,
    scopes_resolved: bool,
}

impl ReadbackSubmissionObservationForTest {
    pub(crate) const fn queue_submission_count_for_test(&self) -> usize {
        self.queue_submission_count
    }

    pub(crate) const fn transaction_generation_for_test(&self) -> Option<u64> {
        Some(self.transaction_generation)
    }

    pub(crate) const fn active_generation_for_test(&self) -> Option<u64> {
        self.active_generation
    }

    pub(crate) fn submission_index_for_test(&self) -> wgpu::SubmissionIndex {
        self.submission_index.clone()
    }

    pub(crate) const fn scopes_resolved_for_test(&self) -> bool {
        self.scopes_resolved
    }
}

pub(crate) async fn submit_readback_observed_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    command_buffer: crate::accounting::OwnedGpu<wgpu::CommandBuffer>,
    operation: RuntimeOperation,
) -> Result<ReadbackSubmissionObservationForTest> {
    let transaction_generation = transaction.lease.generation;
    let active_generation = transaction.lease.signal.active_generation_for_test();
    let account = Account::for_test();
    let pending = transaction.submit_readback(queue, command_buffer, &account)?;
    let completed = pending.finish(operation, &account).await?;
    let submission_index = completed.into_submission_index();
    Ok(ReadbackSubmissionObservationForTest {
        queue_submission_count: 1,
        transaction_generation,
        active_generation,
        submission_index,
        scopes_resolved: true,
    })
}

/// Explicit test-owned facts captured from the Vello payload before its real
/// transaction submission consumes it.
#[derive(Clone, Debug)]
pub(crate) struct InternalVelloSubmissionObservationForTest {
    queue_submission_count: usize,
    transaction_generation: u64,
    active_generation: Option<u64>,
    payload_raster_pass_count: usize,
    allocation_summary: VelloResourceAllocationSummaryForTest,
}

pub(crate) enum InternalVelloSubmissionActionForTest<'a> {
    Observe,
    ScopeFailure(&'a mut Option<u64>),
    AccountingFailure,
}

pub(crate) enum InternalVelloSubmissionOutcomeForTest {
    Observed(InternalVelloSubmissionObservationForTest),
    Completed,
}

impl InternalVelloSubmissionObservationForTest {
    pub(crate) const fn queue_submission_count_for_test(&self) -> usize {
        self.queue_submission_count
    }

    pub(crate) const fn transaction_generation_for_test(&self) -> Option<u64> {
        Some(self.transaction_generation)
    }

    pub(crate) const fn active_generation_for_test(&self) -> Option<u64> {
        self.active_generation
    }

    pub(crate) const fn payload_raster_pass_count_for_test(&self) -> usize {
        self.payload_raster_pass_count
    }

    pub(crate) fn allocation_summary_for_test(
        &self,
    ) -> Option<VelloResourceAllocationSummaryForTest> {
        Some(self.allocation_summary.clone())
    }
}

pub(crate) async fn submit_internal_vello_observed_for_test(
    transaction: GpuOperationTransaction,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    payload: InternalVelloPayload,
    operation: RuntimeOperation,
) -> Result<InternalVelloSubmissionObservationForTest> {
    let account = Account::for_test();

    let transaction_generation = transaction.lease.generation;
    let active_generation = transaction.lease.signal.active_generation_for_test();
    let (command_buffer, resources, logical_pass) = payload.into_parts_for_test();
    let payload_raster_pass_count = logical_pass.cardinality_for_test();
    let allocation_summary = resources.allocation_summary_for_test();
    let payload = InternalVelloPayload::new(command_buffer, resources, logical_pass);
    transaction
        .submit_internal_vello(device, queue, payload, operation, &account)
        .await?;
    Ok(InternalVelloSubmissionObservationForTest {
        queue_submission_count: 1,
        transaction_generation,
        active_generation,
        payload_raster_pass_count,
        allocation_summary,
    })
}

/// Resolves a real transaction and commits prepared Vello resources without a
/// queue submission for tests that isolate resource-commit accounting.
pub(crate) async fn finish_vello_resources_without_submission_for_test(
    transaction: GpuOperationTransaction,
    resources: PendingVelloResourceCommit,
    operation: RuntimeOperation,
) -> Result<()> {
    let account = Account::for_test();

    transaction.finish(operation, Some(&account)).await?;
    resources
        .into_accounting_ready()?
        .commit(VelloResourceCommitProof::new())?;
    Ok(())
}

/// Submits a real Vello payload, injects a scoped validation error at the
/// explicit post-submit boundary, and commits neither resources nor draft.
pub(crate) async fn vello_scope_failure_after_submission_for_test(
    transaction: GpuOperationTransaction,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    payload: InternalVelloPayload,
    operation: RuntimeOperation,
    publication: &mut Option<u64>,
) -> Result<()> {
    let account = Account::for_test();

    let (command_buffer, resources, _logical_pass) = payload.into_parts_for_test();
    let draft = GpuOperationDraft::new(publication, 2);
    command_buffer.submit(queue, Some(&account))?;
    let _ = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Surgeist explicit Vello transaction validation failure"),
        size: wgpu::Extent3d {
            width: 0,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    transaction.finish(operation, Some(&account)).await?;
    resources
        .into_accounting_ready()?
        .commit(VelloResourceCommitProof::new())?;
    draft.commit();
    Ok(())
}

/// Submits a real Vello payload and poisons its resource accounting only at the
/// explicit post-submit boundary.
pub(crate) async fn vello_accounting_failure_after_submission_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    payload: InternalVelloPayload,
    operation: RuntimeOperation,
) -> Result<()> {
    let account = Account::for_test();

    let (command_buffer, resources, _logical_pass) = payload.into_parts_for_test();
    command_buffer.submit(queue, Some(&account))?;
    let _fault = resources.poison_retained_byte_accounting_for_test();
    transaction.finish(operation, Some(&account)).await?;
    resources
        .into_accounting_ready()?
        .commit(VelloResourceCommitProof::new())?;
    Ok(())
}

/// Holds a real Vello submission, its resources, transaction, and draft at the
/// explicit cancellation boundary. Dropping the future performs normal cleanup.
pub(crate) async fn hold_internal_vello_after_submit_for_test(
    transaction: GpuOperationTransaction,
    queue: &wgpu::Queue,
    payload: InternalVelloPayload,
    publication: &mut Option<u64>,
) -> Result<()> {
    let account = Account::for_test();

    let (command_buffer, resources, logical_pass) = payload.into_parts_for_test();
    command_buffer.submit(queue, Some(&account))?;
    let _transaction = transaction;
    let _resources = resources;
    let _logical_pass = logical_pass;
    let _draft = GpuOperationDraft::new(publication, 2);
    std::future::pending::<()>().await;
    Ok(())
}

/// Per-thread, per-device pause after the ordinary graph queue submission and
/// before its scopes resolve. The public render future retains every normal
/// transaction/resource/publication owner while held here.
#[cfg(not(target_arch = "wasm32"))]
struct GraphSubmissionGate {
    signal: Arc<DeviceSignal>,
    generation: std::cell::Cell<Option<u64>>,
    submissions: std::cell::Cell<usize>,
    released: std::cell::Cell<bool>,
    waker: std::cell::RefCell<Option<std::task::Waker>>,
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static GRAPH_SUBMISSION_GATE: std::cell::RefCell<Option<std::rc::Rc<GraphSubmissionGate>>> = const { std::cell::RefCell::new(None) };
}

/// Restores any enclosing fixture on every exit. Device identity prevents
/// another renderer on the same thread from consuming this fixture's gate.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct GraphSubmissionGateGuardForTest {
    gate: std::rc::Rc<GraphSubmissionGate>,
    previous: Option<std::rc::Rc<GraphSubmissionGate>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GraphSubmissionGateGuardForTest {
    pub(crate) fn install(signal: Arc<DeviceSignal>) -> Self {
        let gate = std::rc::Rc::new(GraphSubmissionGate {
            signal,
            generation: std::cell::Cell::new(None),
            submissions: std::cell::Cell::new(0),
            released: std::cell::Cell::new(false),
            waker: std::cell::RefCell::new(None),
        });
        let previous =
            GRAPH_SUBMISSION_GATE.with_borrow_mut(|slot| slot.replace(std::rc::Rc::clone(&gate)));
        Self { gate, previous }
    }

    pub(crate) fn submitted_generation(&self) -> Option<u64> {
        self.gate.generation.get()
    }

    pub(crate) fn submissions(&self) -> usize {
        self.gate.submissions.get()
    }

    pub(crate) fn release(&self) {
        self.gate.released.set(true);
        if let Some(waker) = self.gate.waker.borrow_mut().take() {
            waker.wake();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for GraphSubmissionGateGuardForTest {
    fn drop(&mut self) {
        self.release();
        GRAPH_SUBMISSION_GATE.with_borrow_mut(|slot| *slot = self.previous.take());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) async fn after_graph_submit_for_test(lease: &super::GpuOperationLease) {
    let gate = GRAPH_SUBMISSION_GATE.with_borrow(Clone::clone);
    let Some(gate) = gate.filter(|gate| Arc::ptr_eq(&gate.signal, &lease.signal)) else {
        return;
    };
    gate.generation.set(Some(lease.generation));
    gate.submissions.set(gate.submissions.get() + 1);
    std::future::poll_fn(|cx| {
        if gate.released.get() {
            std::task::Poll::Ready(())
        } else {
            *gate.waker.borrow_mut() = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    })
    .await;
}
