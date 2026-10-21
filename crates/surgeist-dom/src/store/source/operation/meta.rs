//! One borrowed native advance under the original Source and ancestor owners.
use super::*;
use crate::meta_atomic::{DomMetaAtomicLimits, DomMetaRuntimeUsage};

#[cfg(test)]
mod tests;

/// Original native accounting, retained outside the sparse semantic journal.
/// Its actual atom backing and work dependencies outlive every alias together.
pub(in crate::store) struct NativeWork {
    work: AtomicU64,
    outputs: AtomicUsize,
    peak_bytes: AtomicUsize,
    peak_units: AtomicUsize,
    private_bytes: AtomicUsize,
    private_units: AtomicUsize,
    external_bytes: AtomicUsize,
    external_units: AtomicUsize,
    active: AtomicBool,
    _charge: checkpoint::Charge,
}
pub(super) struct Policy {
    limits: DomMetaAtomicLimits,
    shared: Arc<NativeWork>,
}
impl NativeWork {
    pub(in crate::store) fn step(
        &self,
        limits: DomMetaAtomicLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > limits.work() {
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit {
                limit: limits.work(),
            }));
        }
        self.work.store(next, Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn outputs(&self) -> usize {
        self.outputs.load(Ordering::Relaxed)
    }
    pub(in crate::store) fn reserve_outputs(
        &self,
        count: usize,
        source: (usize, usize),
        limits: DomMetaAtomicLimits,
    ) -> Result<(), DomError> {
        let next = self
            .outputs()
            .checked_add(count)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(next, limits.outputs(), DomResource::MetaAtomicOutputs)?;
        if source.0.checked_add(next).is_none_or(|n| n > source.1) {
            return Err(DomError::SourceOutputLimit);
        }
        self.outputs.store(next, Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn usage(
        &self,
        usage: DomMetaRuntimeUsage,
        limits: DomMetaAtomicLimits,
    ) -> Result<(), DomError> {
        let bytes = self
            .private_bytes
            .load(Ordering::Relaxed)
            .checked_add(usage.bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .private_units
            .load(Ordering::Relaxed)
            .checked_add(usage.units())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            bytes,
            limits.retained_bytes(),
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            units,
            limits.retained_units(),
            DomResource::MetaAtomicRetainedUnits,
        )?;
        self.peak_bytes.fetch_max(bytes, Ordering::Relaxed);
        self.peak_units.fetch_max(units, Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn check_private(
        &self,
        usage: custody::Usage,
        limits: DomMetaAtomicLimits,
    ) -> Result<(), DomError> {
        let bytes = self
            .private_bytes
            .load(Ordering::Relaxed)
            .checked_add(usage.bytes())
            .and_then(|n| n.checked_add(self.external_bytes.load(Ordering::Relaxed)))
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .private_units
            .load(Ordering::Relaxed)
            .checked_add(usage.units())
            .and_then(|n| n.checked_add(self.external_units.load(Ordering::Relaxed)))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            bytes,
            limits.retained_bytes(),
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            units,
            limits.retained_units(),
            DomResource::MetaAtomicRetainedUnits,
        )?;
        self.peak_bytes.fetch_max(bytes, Ordering::Relaxed);
        self.peak_units.fetch_max(units, Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn retain_private(&self, usage: custody::Usage) {
        self.private_bytes
            .fetch_add(usage.bytes(), Ordering::Relaxed);
        self.private_units
            .fetch_add(usage.units(), Ordering::Relaxed);
    }
    pub(in crate::store) fn release_private(&self, usage: custody::Usage) {
        self.private_bytes
            .fetch_sub(usage.bytes(), Ordering::Relaxed);
        self.private_units
            .fetch_sub(usage.units(), Ordering::Relaxed);
    }
    fn set_external(&self, usage: DomMetaRuntimeUsage) {
        self.external_bytes.store(usage.bytes(), Ordering::Relaxed);
        self.external_units.store(usage.units(), Ordering::Relaxed);
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum HoldUse {
    Prefix,
    PreparationOutstanding,
    Closed,
}

/// Exclusive admission and one genuine advance of the original Source lease.
/// Work/admission precedes Runtime bank formation. No mutable lease can escape.
///
/// The original lease cannot advance beside its hold:
/// ```compile_fail,E0499
/// use surgeist_dom::{DomSourceLease, DomMetaAtomicLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(lease: &mut DomSourceLease<'_>, limits: DomMetaAtomicLimits, cancel: &AtomicBool) {
///     let hold = lease.begin_meta_runtime(limits, cancel).unwrap();
///     let _ = lease.advance(cancel);
///     let _ = hold.step(cancel);
/// }
/// ```
/// Nor can a borrowed inline resolver open another advance:
/// ```compile_fail,E0499
/// use surgeist_dom::{DomSourceLease, DomMetaAtomicLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(lease: &mut DomSourceLease<'_>, limits: DomMetaAtomicLimits, cancel: &AtomicBool) {
///     let hold = lease.begin_meta_runtime(limits, cancel).unwrap();
///     let _ = lease.advance_with_inline_styles(|_, _| unreachable!(), cancel);
///     let _ = hold.step(cancel);
/// }
/// ```
/// The hold prevents dropping its actual lease:
/// ```compile_fail,E0505
/// use surgeist_dom::{DomSourceLease, DomMetaAtomicLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(mut lease: DomSourceLease<'_>, limits: DomMetaAtomicLimits, cancel: &AtomicBool) {
///     let hold = lease.begin_meta_runtime(limits, cancel).unwrap();
///     drop(lease);
///     let _ = hold.step(cancel);
/// }
/// ```
/// A second hold cannot borrow the same lease:
/// ```compile_fail,E0499
/// use surgeist_dom::{DomSourceLease, DomMetaAtomicLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(lease: &mut DomSourceLease<'_>, limits: DomMetaAtomicLimits, cancel: &AtomicBool) {
///     let hold = lease.begin_meta_runtime(limits, cancel).unwrap();
///     let _ = lease.begin_meta_runtime(limits, cancel);
///     let _ = hold.step(cancel);
/// }
/// ```
/// Event acceptance remains behind that same exclusive borrow:
/// ```compile_fail,E0499
/// use surgeist_dom::{DomSourceLease, DomMetaAtomicLimits, DomSourceEventRequest,
///     DomSourceEventPublication};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(lease: &mut DomSourceLease<'_>, limits: DomMetaAtomicLimits, cancel: &AtomicBool,
///     request: &DomSourceEventRequest, receipt: &mut Option<DomSourceEventPublication>) {
///     let hold = lease.begin_meta_runtime(limits, cancel).unwrap();
///     let _ = lease.accept_source_event(request, receipt, cancel);
///     let _ = hold.step(cancel);
/// }
/// ```
/// Reacquiring the canonical Source owner also conflicts with its actual lease:
/// ```compile_fail,E0499
/// use surgeist_dom::{Dom, DomSourceOperationId, DomSourceLimits, DomMetaAtomicLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(dom: &mut Dom, operation: DomSourceOperationId, source: DomSourceLimits,
///     limits: DomMetaAtomicLimits, cancel: &AtomicBool) {
///     let mut lease = dom.resume_source_edit(operation, source, cancel).unwrap();
///     let hold = lease.begin_meta_runtime(limits, cancel).unwrap();
///     let _ = dom.resume_source_edit(operation, source, cancel);
///     let _ = hold.step(cancel);
/// }
/// ```
#[must_use = "dispose represented Runtime backing before releasing its hold"]
pub struct DomMetaSourceRuntimeHold<'lease, 'dom> {
    lease: &'lease mut DomSourceLease<'dom>,
    shared: Arc<NativeWork>,
    limits: DomMetaAtomicLimits,
    usage: DomMetaRuntimeUsage,
    state: HoldUse,
    // The starting event and reconstructed Draft have distinct mirror pairs.
    charges: [Option<checkpoint::Charge>; 2],
}
impl Drop for DomMetaSourceRuntimeHold<'_, '_> {
    fn drop(&mut self) {
        drop(self.charges[0].take());
        drop(self.charges[1].take());
        self.shared.release_private(custody::Usage::new(
            size_of::<DomMetaSourceRuntimeHold<'_, '_>>(),
            0,
        ));
        self.shared.set_external(DomMetaRuntimeUsage {
            bytes: 0,
            units: 0,
            outputs: 0,
        });
        if self.state != HoldUse::PreparationOutstanding {
            self.shared.active.store(false, Ordering::Relaxed);
        }
    }
}
impl DomSourceLease<'_> {
    fn qualify_meta_lease(&self) -> Result<(), DomError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|op| !self.complete && op.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if matches!(operation.commands, OperationRoot::DialogTask(_))
            && !self.task_authorization.as_ref().is_some_and(|binding| {
                self.dom
                    .element_tasks
                    .as_ref()
                    .and_then(std::sync::Weak::upgrade)
                    .is_some_and(|actual| Arc::ptr_eq(binding, &actual))
            })
        {
            return Err(DomError::InvalidSourceOperation);
        }
        if let Some(select) = &operation.select {
            let authorization = self
                .select_authorization
                .as_ref()
                .ok_or(DomError::InvalidNativeTask)?;
            if authorization.id != select.operation
                || authorization.target != select.target
                || authorization.status() != 0
            {
                return Err(DomError::InvalidNativeTask);
            }
        }
        if operation.observation().is_some()
            && let Some((element, identity)) = self.dom.inline_feedback_observation()
            && Some(identity) == operation.inline_feedback_observation()
        {
            return Err(DomError::InlineStyleOperationPending { element });
        }
        Ok(())
    }
    fn meta_formation(&self, cancel: &AtomicBool) -> Result<formation::Formation, DomError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| !self.complete && operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation.observation().is_some() {
            self.dom
                .source_observation_formation(cancel)?
                .ok_or(DomError::InvalidSourceOperation)
        } else {
            formation::Formation::new(&operation.draft, cancel)?
                .ok_or(DomError::InvalidSourceOperation)
        }
    }
}
impl<'dom> DomSourceLease<'dom> {
    /// Bind or componentwise raise the retained finite native envelope. An
    /// already active (including forgotten) hold cannot be replaced or reset.
    pub fn begin_meta_runtime<'lease>(
        &'lease mut self,
        limits: DomMetaAtomicLimits,
        cancel: &AtomicBool,
    ) -> Result<DomMetaSourceRuntimeHold<'lease, 'dom>, DomError> {
        self.qualify_meta_lease()?;
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| !self.complete && operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if let Some(policy) = &operation.meta_policy {
            if policy.shared.active.load(Ordering::Relaxed) {
                return Err(DomError::SourceOperationPending);
            }
            if limits.retained_bytes() < policy.limits.retained_bytes()
                || limits.retained_units() < policy.limits.retained_units()
                || limits.work() < policy.limits.work()
                || limits.outputs() < policy.limits.outputs()
            {
                return Err(DomError::SourceLimitReduction);
            }
        }
        let previous = operation
            .meta_policy
            .as_ref()
            .map(|policy| policy.shared.clone());
        check_cancel(cancel)?;
        self.dom
            .source_operation
            .as_mut()
            .expect("qualified original Source")
            .entry_scope
            .seal();
        let formation = self.meta_formation(cancel)?;
        formation.step(cancel)?;
        // The inline policy is already in the admitted Operation layout.
        let shared = if let Some(shared) = previous {
            shared
        } else {
            admit(
                size_of::<NativeWork>(),
                limits.retained_bytes(),
                DomResource::MetaAtomicRetainedBytes,
            )?;
            if limits.work() == 0 {
                return Err(DomError::Meta(crate::DomMetaError::WorkLimit { limit: 0 }));
            }
            formation.retain_bytes(size_of::<NativeWork>(), cancel)?;
            Arc::new(NativeWork {
                work: AtomicU64::new(1),
                outputs: AtomicUsize::new(0),
                peak_bytes: AtomicUsize::new(size_of::<NativeWork>()),
                peak_units: AtomicUsize::new(0),
                private_bytes: AtomicUsize::new(size_of::<NativeWork>()),
                private_units: AtomicUsize::new(0),
                external_bytes: AtomicUsize::new(0),
                external_units: AtomicUsize::new(0),
                active: AtomicBool::new(false),
                _charge: formation.into_charge(),
            })
        };
        self.dom
            .source_operation
            .as_mut()
            .expect("qualified original Source")
            .meta_policy = Some(Policy {
            limits,
            shared: shared.clone(),
        });
        let formation = self.meta_formation(cancel)?;
        formation.step(cancel)?;
        shared.step(limits, cancel)?;
        shared.check_private(
            custody::Usage::new(size_of::<DomMetaSourceRuntimeHold<'_, '_>>(), 0),
            limits,
        )?;
        formation.retain_bytes(size_of::<DomMetaSourceRuntimeHold<'_, '_>>(), cancel)?;
        let charge = formation.into_charge();
        shared.retain_private(custody::Usage::new(
            size_of::<DomMetaSourceRuntimeHold<'_, '_>>(),
            0,
        ));
        shared.active.store(true, Ordering::Relaxed);
        Ok(DomMetaSourceRuntimeHold {
            lease: self,
            shared,
            limits,
            usage: DomMetaRuntimeUsage {
                bytes: 0,
                units: 0,
                outputs: 0,
            },
            state: HoldUse::Prefix,
            charges: [Some(charge), None],
        })
    }
}
impl DomMetaSourceRuntimeHold<'_, '_> {
    fn require_prefix(&self) -> Result<(), DomError> {
        if self.state != HoldUse::Prefix {
            return Err(DomError::SourceOperationPending);
        }
        Ok(())
    }
    fn charge(&mut self, incoming: checkpoint::Charge) -> Result<(), DomError> {
        charge_slot(&mut self.charges, incoming)
    }

    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.require_prefix()?;
        self.lease.admit_hosted_runtime_work(1, cancel)?;
        self.shared.step(self.limits, cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomMetaRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.require_prefix()?;
        self.step(cancel)?;
        let next = DomMetaRuntimeUsage::new(
            self.usage.bytes().max(usage.bytes()),
            self.usage.units().max(usage.units()),
            self.usage.outputs().max(usage.outputs()),
        )?;
        self.shared.usage(next, self.limits)?;
        let operation = self
            .lease
            .dom
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        self.shared.reserve_outputs(
            next.outputs() - self.usage.outputs(),
            (operation.outputs, operation.limits.outputs),
            self.limits,
        )?;
        let formation = self.lease.meta_formation(cancel)?;
        formation.retain_payload(
            storage::Payload {
                bytes: next.bytes() - self.usage.bytes(),
                units: next.units() - self.usage.units(),
            },
            cancel,
        )?;
        self.charge(formation.into_charge())?;
        self.shared.set_external(next);
        self.usage = next;
        Ok(())
    }
    pub fn request_cancel(&mut self) -> Result<(), DomError> {
        let operation = self
            .lease
            .dom
            .source_operation
            .as_mut()
            .filter(|operation| !self.lease.complete && operation.id == self.lease.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        operation.cancelled = true;
        Ok(())
    }
}

/// Read-only scope of the original journal mark; this retains no inverse data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubjectScope(transaction::JournalObservation<transaction::Undo>);
impl SubjectScope {
    pub(super) fn new(checkpoint: &checkpoint::Checkpoint) -> Self {
        Self(checkpoint.observation())
    }
}

/// Private invocation proof; only the owner front can construct it after
/// acquiring and validating the original accepted Style receipt guard.
pub(super) struct Execution {
    shared: Arc<NativeWork>,
    limits: DomMetaAtomicLimits,
    runtime: DomMetaRuntimeUsage,
}
pub(super) enum Disposition {
    Style,
    Hosted,
    Dialog,
    Focusing,
    Autofocus,
    DesignMode,
    Complete {
        installation: Option<NodeId>,
        select: bool,
    },
}
pub(super) struct Pending {
    segment: SegmentPublication,
    checkpoint: Option<checkpoint::Checkpoint>,
    disposition: Disposition,
    bases: Vec<(NodeId, surgeist_cssom::CssomSnapshot)>,
}
pub(super) struct SavedReceipt {
    pub(super) receipt: Receipt,
    pub(super) request: Option<DomStyleRequest>,
    pub(super) binding: Option<DomStyleCssomOwnerBinding>,
}
impl Operation {
    pub(super) fn meta_reached(&self) -> bool {
        matches!(&self.draft, dialog::OperationDraft::Segment(draft)
            if draft.meta_recording.as_ref().is_some_and(|r| !r.entries.is_empty()))
    }
    pub(super) fn save_meta_receipt(&mut self) {
        if self.meta_receipt.is_none()
            && let Some(receipt) = self.receipt.take()
        {
            self.meta_receipt = Some(SavedReceipt {
                receipt,
                request: self.request.take(),
                binding: self.owner_binding.take(),
            });
        }
    }
    pub(super) fn prepare_meta_execution(
        &mut self,
        execution: &inline::Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.pending_meta.is_some() || (self.meta_reached() && execution.native.is_none()) {
            return Err(DomError::Meta(crate::DomMetaError::DefaultStyleUnsupported));
        }
        if let Some(native) = &execution.native {
            if self.draft.meta_recording.is_none() {
                let formation = formation::Formation::new(&self.draft, cancel)?
                    .ok_or(DomError::InvalidSourceOperation)?;
                let mut recording = crate::store::meta_atomic::Recording {
                    entries: Vec::new(),
                    payloads: Default::default(),
                    envelope: crate::store::meta_atomic::Envelope::source(
                        native.limits,
                        native.runtime,
                        native.shared.clone(),
                        formation,
                        (self.outputs, self.limits.outputs),
                    )?,
                };
                recording.attach(cancel)?;
                self.draft.meta_recording = Some(recording);
            }
        } else {
            self.draft.meta_recording = None;
        }
        Ok(())
    }
    pub(super) fn park_meta_checkpoint(&mut self, checkpoint: checkpoint::Checkpoint) {
        let pending = self
            .pending_meta
            .as_mut()
            .expect("original native boundary retained before escape");
        pending.checkpoint = Some(checkpoint);
    }
}

impl Pending {
    pub(super) fn new(segment: SegmentPublication, disposition: Disposition) -> Self {
        Self {
            segment,
            checkpoint: None,
            disposition,
            bases: Vec::new(),
        }
    }
}

use crate::store::meta_atomic::{Envelope, Root};
use crate::{
    DomMetaAtomicError, DomMetaInlineResolver, DomMetaPreferredOccurrence, DomMetaRetirement,
    DomMetaRootCommit, DomMetaSourceCommit,
};
use surgeist_cssom::{CssomBatch, CssomEditGuard, CssomSheetId, CssomSnapshot};

/// The one actual advance either parks a native boundary or returns the
/// existing genuine progress, with its original inline evidence.
#[expect(
    clippy::large_enum_variant,
    reason = "keep the affine prepared graph and its debit together without introducing an allocation or admission failure in this advance"
)]
pub enum DomMetaSourceStep<'hold, 'dom, 'caps> {
    Prepared(DomMetaSourcePreparation<'hold, 'dom, 'caps>),
    Progress(crate::DomInlineStyleSourceProgress),
}
/// An exclusive loan of a phase already owned by the original Operation.
#[must_use = "commit or drop this original native Source preparation"]
pub struct DomMetaSourcePreparation<'hold, 'dom, 'caps> {
    lease: &'hold mut DomSourceLease<'dom>,
    state: &'hold mut HoldUse,
    runtime_usage: &'hold mut DomMetaRuntimeUsage,
    runtime_charges: &'hold mut [Option<checkpoint::Charge>; 2],
    roots: Option<Vec<Root<'caps>>>,
    inline: Option<CssomBatch>,
    // Graphs above are disposed before their own debit and work dependencies.
    envelope: Option<Envelope>,
}
impl Drop for DomMetaSourcePreparation<'_, '_, '_> {
    fn drop(&mut self) {
        drop(self.inline.take());
        drop(self.roots.take());
        drop(self.envelope.take());
        *self.state = HoldUse::Closed;
    }
}
/// Holds real locks until Runtime has installed its admitted preference state.
pub struct DomMetaSourcePublished<'caps> {
    guards: Vec<std::sync::MutexGuard<'caps, surgeist_cssom::CssomStore>>,
    commit: DomMetaSourceCommit,
    retirement: DomMetaRetirement,
    _charge: Option<crate::store::meta_atomic::SourceGraphCharge>,
}
impl DomMetaSourcePublished<'_> {
    pub fn into_parts(self) -> (DomMetaSourceCommit, DomMetaRetirement) {
        let Self {
            guards,
            commit,
            retirement,
            _charge,
        } = self;
        drop(guards);
        // The real output moves to its caller at this infallible frontier.
        drop(_charge);
        (commit, retirement)
    }
}
struct ReceiptLoan<'caps> {
    root: NodeId,
    guard: Option<std::sync::MutexGuard<'caps, surgeist_cssom::CssomStore>>,
    snapshot: CssomSnapshot,
    _charge: crate::store::meta_atomic::SourceGraphCharge,
}
impl DomSourceLease<'_> {
    fn meta_receipt(&self) -> Option<(&Receipt, &DomStyleRequest)> {
        let operation = self.dom.source_operation.as_ref()?;
        if let Some(saved) = &operation.meta_receipt {
            Some((&saved.receipt, saved.request.as_ref()?))
        } else {
            Some((operation.receipt.as_ref()?, operation.request.as_ref()?))
        }
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    fn meta_receipt_loan<'caps>(
        &self,
        caps: &'caps [&'caps crate::DomStyleCssomRoot],
        shared: &Arc<NativeWork>,
        limits: DomMetaAtomicLimits,
        cancel: &AtomicBool,
    ) -> Result<Option<ReceiptLoan<'caps>>, DomMetaAtomicError> {
        let Some((Receipt::Cssom { revision, .. }, request)) = self.meta_receipt() else {
            return Ok(None);
        };
        let root = request.cssom_root().ok_or(DomError::StyleReceiptMismatch)?;
        let canonical = self
            .dom
            .style_cssom_roots
            .get(root)
            .ok_or(DomError::StyleReceiptMismatch)?;
        let formation = self.meta_formation(cancel)?;
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .expect("qualified Source");
        let envelope = Envelope::source(
            limits,
            DomMetaRuntimeUsage {
                bytes: 0,
                units: 0,
                outputs: 0,
            },
            shared.clone(),
            formation,
            (operation.outputs, operation.limits.outputs),
        )?;
        let mut selected: Option<&'caps crate::DomStyleCssomRoot> = None;
        for cap in caps {
            envelope.step(cancel)?;
            if cap.root() == root {
                if !canonical.same_owner(cap) || selected.is_some_and(|old| !old.same_owner(cap)) {
                    return Err(DomMetaAtomicError::MismatchedOwner { root });
                }
                selected = Some(*cap);
            }
        }
        let cap = selected.ok_or(DomMetaAtomicError::MissingOwner { root })?;
        let payload = storage::Payload {
            bytes: size_of::<ReceiptLoan<'_>>(),
            units: 0,
        };
        envelope.retain(payload.bytes, payload.units, 0, cancel)?;
        envelope.step(cancel)?;
        let guard = cap.acquire().map_err(|error| match error {
            crate::style::CssomRootAccessError::Busy => DomMetaAtomicError::OwnerBusy { root },
            crate::style::CssomRootAccessError::Poisoned => {
                DomMetaAtomicError::OwnerPoisoned { root }
            }
        })?;
        let snapshot = guard.snapshot();
        if snapshot.revision() != revision {
            return Err(DomError::StyleReceiptMismatch.into());
        }
        Ok(Some(ReceiptLoan {
            root,
            guard: Some(guard),
            snapshot,
            _charge: envelope.into_source_charge(payload),
        }))
    }
}
impl<'lease, 'dom> DomMetaSourceRuntimeHold<'lease, 'dom> {
    /// Seal this hold before advancing. The escaping preparation lends the
    /// original retained phase; it never owns the only continuation/checkpoint.
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    pub fn prepare<'hold, 'caps>(
        &'hold mut self,
        caps: &'caps [&'caps crate::DomStyleCssomRoot],
        mut resolver: Option<&mut DomMetaInlineResolver<'_>>,
        cancel: &AtomicBool,
    ) -> Result<DomMetaSourceStep<'hold, 'dom, 'caps>, DomMetaAtomicError> {
        self.require_prefix()?;
        self.state = HoldUse::PreparationOutstanding;
        let has_resolver = resolver.is_some();
        let mut forward = |candidates: &[surgeist_cssom::CssomDeclarationCandidate],
                           context: &surgeist_cssom::CssomContext| {
            resolver
                .as_deref_mut()
                .expect("resolver selected for this invocation")(candidates, context)
        };
        let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut receipt =
                self.lease
                    .meta_receipt_loan(caps, &self.shared, self.limits, cancel)?;
            if self
                .lease
                .dom
                .source_operation
                .as_ref()
                .is_some_and(|op| op.pending_meta.is_none())
            {
                let mut execution = inline::Execution {
                    resolver: if has_resolver {
                        Some(&mut forward)
                    } else {
                        None
                    },
                    publication: None,
                    native: Some(Execution {
                        shared: self.shared.clone(),
                        limits: self.limits,
                        runtime: self.usage,
                    }),
                };
                match self.lease.advance_execution(&mut execution, cancel) {
                    Ok(source) => {
                        return Ok(Err(crate::DomInlineStyleSourceProgress {
                            source,
                            inline: execution.publication,
                        }));
                    }
                    Err(AdvanceError::NativePrepared) => (),
                    Err(AdvanceError::Dom(error)) => return Err(error.into()),
                    Err(AdvanceError::Inline(error)) => return Err(error.into()),
                    Err(AdvanceError::Unwind(payload)) => std::panic::resume_unwind(payload),
                }
            }
            {
                let operation = self
                    .lease
                    .dom
                    .source_operation
                    .as_mut()
                    .ok_or(DomError::InvalidSourceOperation)?;
                if let Some(recording) = operation.draft.meta_recording.as_mut() {
                    recording.envelope.rebind_source(
                        self.limits,
                        self.usage,
                        (operation.outputs, operation.limits.outputs),
                    );
                }
            }
            let formation = self.lease.meta_formation(cancel)?;
            let op = self
                .lease
                .dom
                .source_operation
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            let envelope = Envelope::source(
                self.limits,
                self.usage,
                self.shared.clone(),
                formation,
                (
                    op.pending_meta
                        .as_ref()
                        .ok_or(DomError::InvalidSourceOperation)?
                        .segment
                        .outputs,
                    op.limits.outputs,
                ),
            )?;
            let mut roots: Vec<Root<'caps>> = Vec::new();
            let operation = self
                .lease
                .dom
                .source_operation
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            let recording = operation
                .draft
                .meta_recording
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            for occurrence in &recording.entries {
                envelope.step(cancel)?;
                let root = occurrence.document();
                let mut found = false;
                for held in &roots {
                    envelope.step(cancel)?;
                    if held.root == root {
                        found = true;
                        break;
                    }
                }
                if found {
                    continue;
                }
                let canonical = self.lease.dom.style_cssom_roots.get(root);
                let mut selected: Option<&'caps crate::DomStyleCssomRoot> = None;
                for cap in caps {
                    envelope.step(cancel)?;
                    if cap.root() != root {
                        continue;
                    }
                    if canonical.is_some_and(|old| !old.same_owner(cap))
                        || selected.is_some_and(|old| !old.same_owner(cap))
                    {
                        return Err(DomMetaAtomicError::MismatchedOwner { root });
                    }
                    selected = Some(*cap);
                }
                let cap = selected.ok_or(DomMetaAtomicError::MissingOwner { root })?;
                envelope.reserve(&mut roots, 1, cancel)?;
                let store = if let Some(loan) = receipt.as_mut().filter(|loan| loan.root == root) {
                    loan.guard.take().expect("one receipt guard reused once")
                } else {
                    cap.acquire().map_err(|error| match error {
                        crate::style::CssomRootAccessError::Busy => {
                            DomMetaAtomicError::OwnerBusy { root }
                        }
                        crate::style::CssomRootAccessError::Poisoned => {
                            DomMetaAtomicError::OwnerPoisoned { root }
                        }
                    })?
                };
                let snapshot = store.snapshot();
                let batch = store
                    .batch(CssomEditGuard::from_snapshot(&snapshot))
                    .map_err(|error| DomMetaAtomicError::Cssom { root, error })?;
                roots.push(Root {
                    root,
                    store,
                    snapshot,
                    batch: Some(batch),
                });
            }
            // All actual bases belong to the parked phase, independent of locks.
            let operation = self
                .lease
                .dom
                .source_operation
                .as_mut()
                .expect("parked original Source");
            let pending = operation
                .pending_meta
                .as_mut()
                .ok_or(DomError::InvalidSourceOperation)?;
            if pending.bases.is_empty() {
                let formation = pending
                    .segment
                    .prepared
                    ._formation
                    .as_ref()
                    .expect("Source publication custody");
                formation::reserve(&mut pending.bases, roots.len(), Some(formation), cancel)?;
                for row in &roots {
                    envelope.step(cancel)?;
                    pending.bases.push((row.root, row.snapshot.clone()));
                }
            } else {
                for (root, basis) in &pending.bases {
                    envelope.step(cancel)?;
                    let held = roots
                        .iter()
                        .find(|row| row.root == *root)
                        .ok_or(DomMetaAtomicError::MissingOwner { root: *root })?;
                    if held.snapshot.revision() != basis.revision() {
                        return Err(DomMetaAtomicError::StaleProducer { root: *root });
                    }
                }
            }
            // Stage any first zero-sheet owner in the original publication bank.
            self.lease.dom.custody_registry.reconcile_releases();
            let basis = self.lease.dom.custody_registry.ledger().total();
            for row in &roots {
                envelope.step(cancel)?;
                if self.lease.dom.style_cssom_roots.get(row.root).is_some() {
                    continue;
                }
                let cap = caps
                    .iter()
                    .find(|cap| cap.root() == row.root)
                    .ok_or(DomMetaAtomicError::MissingOwner { root: row.root })?;
                let pending = self
                    .lease
                    .dom
                    .source_operation
                    .as_mut()
                    .expect("original Source")
                    .pending_meta
                    .as_mut()
                    .expect("native phase");
                if !pending
                    .segment
                    .prepared
                    .state
                    .node(self.lease.dom.owner, row.root)
                    .is_ok_and(|node| matches!(node.kind(), DomNodeKind::Document(_)))
                {
                    continue;
                }
                let bank = pending
                    .segment
                    .prepared
                    .style_custody
                    .as_ref()
                    .unwrap_or(&self.lease.dom.style_cssom_roots);
                let next = bank.prepare_insert(
                    cap,
                    DocumentId(row.root),
                    |usage| envelope.controlled_extra(usage, basis, cancel),
                    || envelope.step(cancel),
                )?;
                pending.segment.prepared.style_custody = Some(next);
            }
            let operation = self
                .lease
                .dom
                .source_operation
                .as_ref()
                .expect("retained native Source");
            let pending = operation.pending_meta.as_ref().expect("native phase");
            let inline = if pending
                .segment
                .prepared
                .inline_recording
                .as_ref()
                .is_some_and(crate::store::inline_style::recording::Recording::has_inputs)
            {
                let limits = operation
                    .inline_request_limits()
                    .ok_or(DomError::StyleProtocol)?;
                if !has_resolver {
                    pending
                        .segment
                        .prepared
                        .inline_recording
                        .as_ref()
                        .expect("actual inline inputs")
                        .require_unreached(|| envelope.step(cancel))?;
                }
                let resolver = &mut forward;
                self.lease.dom.prepare_inline_batch(
                    &pending.segment.prepared,
                    limits,
                    resolver,
                    cancel,
                )?
            } else {
                None
            };
            // Receipt-only lock remains held until this one entry finishes.
            if let Some(mut loan) = receipt
                && let Some(store) = loan.guard.take()
            {
                envelope.reserve(&mut roots, 1, cancel)?;
                roots.push(Root {
                    root: loan.root,
                    store,
                    snapshot: loan.snapshot,
                    batch: None,
                });
            }
            Ok(Ok((roots, inline, envelope)))
        }));
        match built {
            Ok(Ok(Ok((roots, inline, envelope)))) => {
                Ok(DomMetaSourceStep::Prepared(DomMetaSourcePreparation {
                    lease: self.lease,
                    state: &mut self.state,
                    runtime_usage: &mut self.usage,
                    runtime_charges: &mut self.charges,
                    roots: Some(roots),
                    inline,
                    envelope: Some(envelope),
                }))
            }
            Ok(Ok(Err(progress))) => {
                self.state = HoldUse::Closed;
                Ok(DomMetaSourceStep::Progress(progress))
            }
            Ok(Err(error)) => {
                self.state = HoldUse::Closed;
                Err(error)
            }
            Err(payload) => {
                self.state = HoldUse::Closed;
                std::panic::resume_unwind(payload)
            }
        }
    }
}

impl DomMetaSourcePreparation<'_, '_, '_> {
    pub fn occurrences(&self) -> &[DomMetaPreferredOccurrence] {
        &self
            .lease
            .dom
            .source_operation
            .as_ref()
            .expect("exclusive original Source")
            .draft
            .meta_recording
            .as_ref()
            .expect("retained native inputs")
            .entries
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    pub fn root_snapshot(
        &self,
        root: NodeId,
        cancel: &AtomicBool,
    ) -> Result<&CssomSnapshot, DomMetaAtomicError> {
        for row in self.roots.as_ref().expect("live root bank") {
            self.step(cancel)?;
            if row.root == root && row.batch.is_some() {
                return Ok(&row.snapshot);
            }
        }
        Err(DomMetaAtomicError::MissingOwner { root })
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    pub fn set_disabled(
        &mut self,
        root: NodeId,
        sheet: &CssomSheetId,
        disabled: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomMetaAtomicError> {
        let envelope = self.envelope.as_ref().expect("live native custody");
        for row in self.roots.as_mut().expect("live root bank") {
            envelope.step(cancel)?;
            if row.root != root || row.batch.is_none() {
                continue;
            }
            let mut found = false;
            for id in row.snapshot.sheets() {
                envelope.step(cancel)?;
                if id == sheet {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(DomMetaAtomicError::StaleProducer { root });
            }
            row.batch
                .as_mut()
                .expect("actual unattempted Meta batch")
                .set_disabled(sheet, disabled)
                .map_err(|error| DomMetaAtomicError::Cssom { root, error })?;
            return Ok(());
        }
        Err(DomMetaAtomicError::MissingOwner { root })
    }
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.envelope
            .as_ref()
            .expect("live native custody")
            .step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomMetaRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // External backing remains owned by the hold, not this preparation.
        let envelope = self.envelope.as_ref().expect("live native custody");
        envelope.runtime(usage, cancel)?;
        // Its real phase-specific custody is borrowed separately from the hold.
        self.runtime_admission(usage, cancel)
    }
}

fn charge_slot(
    slots: &mut [Option<checkpoint::Charge>; 2],
    incoming: checkpoint::Charge,
) -> Result<(), DomError> {
    for existing in slots.iter_mut().flatten() {
        if existing.same_phase(&incoming) {
            existing.merge(incoming);
            return Ok(());
        }
    }
    if let Some(slot) = slots.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(incoming);
        return Ok(());
    }
    Err(DomError::InvalidSourceOperation)
}
impl DomMetaSourcePreparation<'_, '_, '_> {
    fn runtime_admission(
        &mut self,
        usage: DomMetaRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let next = DomMetaRuntimeUsage::new(
            self.runtime_usage.bytes().max(usage.bytes()),
            self.runtime_usage.units().max(usage.units()),
            self.runtime_usage.outputs().max(usage.outputs()),
        )?;
        let formation = self.lease.meta_formation(cancel)?;
        formation.retain_payload(
            storage::Payload {
                bytes: next.bytes() - self.runtime_usage.bytes(),
                units: next.units() - self.runtime_usage.units(),
            },
            cancel,
        )?;
        charge_slot(self.runtime_charges, formation.into_charge())?;
        self.lease
            .dom
            .source_operation
            .as_ref()
            .expect("original native policy")
            .meta_policy
            .as_ref()
            .expect("retained native policy")
            .shared
            .set_external(next);
        *self.runtime_usage = next;
        Ok(())
    }
}

impl Policy {
    pub(super) fn outputs(&self) -> usize {
        self.shared.outputs()
    }
}

impl Pending {
    fn finish(
        self,
        operation: &mut Operation,
        lease: &mut DomSourceLease<'_>,
    ) -> (DomSourceProgress, crate::DomChanges) {
        let Self {
            segment,
            checkpoint,
            disposition,
            bases: _,
        } = self;
        let published = segment.publish(operation, lease.dom, checkpoint.as_ref());
        let changes = published.commit.changes.clone();
        operation.receipt = None;
        operation.owner_binding = None;
        operation.meta_receipt = None;
        // The real journal is now rebased; no old native input may replay.
        drop(operation.draft.meta_recording.take());
        operation.draft.meta_scope = None;
        let source = match disposition {
            Disposition::Style => DomSourceProgress::Request(DomSourceRequest::Style(
                operation
                    .request
                    .as_ref()
                    .expect("preformed Style request")
                    .clone(),
            )),
            Disposition::Hosted => DomSourceProgress::Request(DomSourceRequest::HostedChild(
                operation
                    .child
                    .as_ref()
                    .expect("preformed child")
                    .request
                    .clone(),
            )),
            Disposition::Dialog => DomSourceProgress::Request(DomSourceRequest::Dialog(
                published.dialog.expect("preformed Dialog request"),
            )),
            Disposition::Focusing => DomSourceProgress::Request(match published.focus_delegate {
                Some(request) => DomSourceRequest::FocusDelegate(request),
                None => DomSourceRequest::DialogFocusing(
                    published.focusing.expect("preformed focusing request"),
                ),
            }),
            Disposition::Autofocus => {
                DomSourceProgress::Request(DomSourceRequest::AutofocusInsertion(
                    published.autofocus.expect("preformed insertion request"),
                ))
            }
            Disposition::DesignMode => DomSourceProgress::Request(DomSourceRequest::DesignMode(
                published.design_mode.expect("preformed designMode request"),
            )),
            Disposition::Complete {
                installation,
                select,
            } => {
                operation.request = None;
                let installation = installation.map(|root| {
                    let input = operation
                        .installation
                        .as_mut()
                        .expect("original installation");
                    let bindings = std::mem::take(&mut input.bindings);
                    let payload = input.bindings_payload.take();
                    let value = crate::DomNativeHtmlInstallation {
                        destination: input.destination,
                        source_root: input.source_root.clone(),
                        root,
                        snapshot: lease.dom.snapshot(),
                        changes: published.commit.changes.clone(),
                        bindings,
                    };
                    drop(payload);
                    value
                });
                if select {
                    lease
                        .select_authorization
                        .as_ref()
                        .expect("qualified Select permission")
                        .consume();
                    lease.dom.active_task = std::sync::Weak::new();
                }
                DomSourceProgress::Complete(DomSourceCompletion {
                    operation: operation.id,
                    cancelled: operation.cancelled,
                    published: std::mem::take(&mut operation.historical),
                    commit: published.commit,
                    installation,
                    fullscreen_exit_return: match operation.draft.method.as_ref() {
                        Some(crate::store::method::Frame::FullscreenExit(value)) => *value,
                        _ => None,
                    },
                    design_mode_range_disposition: match operation.draft.method.as_ref() {
                        Some(crate::store::method::Frame::DesignMode(frame)) => frame.range,
                        _ => None,
                    },
                    dialog_toggle_disposition: match operation.commands {
                        OperationRoot::DialogTask(crate::DomDialogTaskInfo::Toggle(_)) => {
                            Some(crate::DomDialogToggleTaskDisposition::TrackerCleared)
                        }
                        _ => None,
                    },
                })
            }
        };
        (source, changes)
    }
}
impl<'caps> DomMetaSourcePreparation<'_, '_, 'caps> {
    /// Prepare every producer and final gate before publishing any owner. The
    /// original pending phase stays in Operation on every earlier refusal/Drop.
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    pub fn commit(
        mut self,
        cancel: &AtomicBool,
    ) -> Result<DomMetaSourcePublished<'caps>, DomMetaAtomicError> {
        let envelope = self.envelope.as_ref().expect("live native custody");
        let roots = self.roots.as_mut().expect("real guarded bank");
        let mut count = 0usize;
        for row in roots.iter() {
            envelope.step(cancel)?;
            if row.batch.is_some() {
                count = count
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let all = count
            .checked_add(usize::from(self.inline.is_some()))
            .ok_or(DomError::RepresentationExhausted)?;
        let mut outputs = Vec::new();
        envelope.reserve(&mut outputs, count, cancel)?;
        let mut retired = Vec::new();
        envelope.reserve(&mut retired, all, cancel)?;
        let mut guards = Vec::new();
        envelope.reserve(&mut guards, roots.len(), cancel)?;
        let mut prepared = Vec::new();
        envelope.reserve(&mut prepared, count, cancel)?;
        envelope.retain(
            size_of::<DomMetaSourcePublished<'caps>>(),
            0,
            all.checked_add(2)
                .ok_or(DomError::RepresentationExhausted)?,
            cancel,
        )?;
        let surviving = storage::Payload {
            bytes: guards
                .capacity()
                .checked_mul(size_of::<
                    std::sync::MutexGuard<'caps, surgeist_cssom::CssomStore>,
                >())
                .and_then(|n| {
                    n.checked_add(
                        outputs
                            .capacity()
                            .checked_mul(size_of::<DomMetaRootCommit>())?,
                    )
                })
                .and_then(|n| {
                    n.checked_add(
                        retired
                            .capacity()
                            .checked_mul(size_of::<surgeist_cssom::CssomCommitRetirement>())?,
                    )
                })
                .and_then(|n| n.checked_add(size_of::<DomMetaSourcePublished<'caps>>()))
                .ok_or(DomError::RepresentationExhausted)?,
            units: 0,
        };
        for row in roots.iter_mut() {
            envelope.step(cancel)?;
            if let Some(batch) = row.batch.take() {
                let root = row.root;
                let value = row
                    .store
                    .prepare_commit(batch)
                    .map_err(|error| DomMetaAtomicError::Cssom { root, error })?;
                prepared.push((root, value));
            }
        }
        let inline = if let Some(batch) = self.inline.take() {
            let owner = self
                .lease
                .dom
                .inline_styles
                .as_mut()
                .expect("recorded actual inline producer");
            Some(
                owner
                    .store
                    .as_mut()
                    .expect("actual inline store")
                    .prepare_commit(batch)
                    .map_err(crate::DomInlineStyleError::from)?,
            )
        } else {
            None
        };
        envelope.step(cancel)?;
        // The suffix owns every required slot/loan. Nothing below may refuse.
        for (root, value) in prepared.drain(..) {
            let (commit, dead) = value.publish();
            outputs.push(DomMetaRootCommit {
                root,
                cssom: commit,
            });
            retired.push(dead);
        }
        let inline = inline.map(|value| {
            let (commit, dead) = value.publish();
            retired.push(dead);
            commit
        });
        drop(prepared);
        let mut operation = self
            .lease
            .dom
            .source_operation
            .take()
            .expect("exclusive original Source");
        let pending = operation
            .pending_meta
            .take()
            .expect("owner-retained native phase");
        let (source, changes) = pending.finish(&mut operation, self.lease);
        let stamp = changes.to();
        let inline = inline.map(|inline| crate::DomInlineStyleSourcePublication {
            dom: changes,
            inline,
        });
        if matches!(&source, DomSourceProgress::Complete(_)) {
            self.lease.complete = true;
            self.lease.dom.source_operation = self.lease.dom.source_parents.pop();
        } else {
            self.lease.dom.source_operation = Some(operation);
        }
        let mut roots = self.roots.take().expect("published real root bank");
        for row in roots.drain(..) {
            guards.push(row.store);
        }
        drop(roots);
        let envelope = self.envelope.take().expect("actual graph custody");
        let charge = envelope.into_source_charge(surviving);
        *self.state = HoldUse::Closed;
        Ok(DomMetaSourcePublished {
            guards,
            commit: DomMetaSourceCommit {
                source,
                roots: outputs,
                inline,
                stamp,
            },
            retirement: DomMetaRetirement { _cssom: retired },
            _charge: Some(charge),
        })
    }
}
