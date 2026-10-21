//! One native ordinary publication and the actual borrowed CSSOM owner bank.
use super::*;
use crate::meta_atomic::*;
use std::{cell::Cell, sync::MutexGuard};
use surgeist_cssom::{CssomBatch, CssomEditGuard, CssomSheetId, CssomSnapshot, CssomStore};

pub(in crate::store) struct Envelope {
    limits: DomMetaAtomicLimits,
    work: Cell<u64>,
    bytes: Cell<usize>,
    units: Cell<usize>,
    outputs: Cell<usize>,
    runtime: Cell<DomMetaRuntimeUsage>,
    controlled: Cell<custody::Usage>,
    source: Option<SourceAccounting>,
}
struct SourceAccounting {
    work: Arc<source::operation::meta::NativeWork>,
    formation: Option<source::formation::Formation>,
    output_bound: (usize, usize),
    own: Cell<custody::Usage>,
}
impl Drop for SourceAccounting {
    fn drop(&mut self) {
        self.work.release_private(self.own.get());
    }
}
impl SourceAccounting {
    fn retain(
        &self,
        payload: source::storage::Payload,
        limits: DomMetaAtomicLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let usage = custody::Usage::new(payload.bytes, payload.units);
        let next = self.own.get().checked_add(usage)?;
        self.work.check_private(usage, limits)?;
        self.formation
            .as_ref()
            .expect("live Source graph")
            .retain_payload(payload, cancel)?;
        self.work.retain_private(usage);
        self.own.set(next);
        Ok(())
    }
    fn release(&self, payload: source::storage::Payload) {
        let usage = custody::Usage::new(payload.bytes, payload.units);
        self.formation
            .as_ref()
            .expect("live Source graph")
            .release_payload(payload);
        self.work.release_private(usage);
        self.own.set(
            self.own
                .get()
                .checked_sub(usage)
                .expect("owned Source graph extent"),
        );
    }
}
/// Actual graph debit plus its original native shared union receipt. The graph
/// must be destroyed/transferred before this holder is released or shrunk.
pub(in crate::store) struct SourceGraphCharge {
    work: Arc<source::operation::meta::NativeWork>,
    usage: custody::Usage,
    _charge: source::checkpoint::Charge,
}
impl Drop for SourceGraphCharge {
    fn drop(&mut self) {
        self.work.release_private(self.usage);
    }
}

impl Envelope {
    fn new(
        limits: DomMetaAtomicLimits,
        usage: DomMetaRuntimeUsage,
        work: u64,
    ) -> Result<Self, DomError> {
        if work > limits.work {
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit {
                limit: limits.work,
            }));
        }
        admit(
            usage.bytes,
            limits.retained_bytes,
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            usage.units,
            limits.retained_units,
            DomResource::MetaAtomicRetainedUnits,
        )?;
        admit(
            usage.outputs,
            limits.outputs,
            DomResource::MetaAtomicOutputs,
        )?;
        Ok(Self {
            limits,
            work: Cell::new(work),
            bytes: Cell::new(0),
            units: Cell::new(0),
            outputs: Cell::new(0),
            runtime: Cell::new(usage),
            controlled: Cell::new(custody::Usage::ZERO),
            source: None,
        })
    }
    pub(in crate::store) fn source(
        limits: DomMetaAtomicLimits,
        runtime: DomMetaRuntimeUsage,
        work: Arc<source::operation::meta::NativeWork>,
        formation: source::formation::Formation,
        outputs: (usize, usize),
    ) -> Result<Self, DomError> {
        let mut envelope = Self::new(limits, runtime, 0)?;
        envelope.source = Some(SourceAccounting {
            work,
            formation: Some(formation),
            output_bound: outputs,
            own: Cell::new(custody::Usage::ZERO),
        });
        Ok(envelope)
    }
    pub(in crate::store) fn rebind_source(
        &mut self,
        limits: DomMetaAtomicLimits,
        runtime: DomMetaRuntimeUsage,
        outputs: (usize, usize),
    ) {
        assert!(self.source.is_some(), "original native Source graph");
        self.limits = limits;
        self.runtime.set(runtime);
        self.source
            .as_mut()
            .expect("original Source envelope")
            .output_bound = outputs;
    }
    pub(in crate::store) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if let Some(source) = &self.source {
            source
                .formation
                .as_ref()
                .expect("live Source graph")
                .step(cancel)?;
        }
        self.local_step(cancel)
    }
    pub(in crate::store) fn local_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if let Some(source) = &self.source {
            return source.work.step(self.limits, cancel);
        }
        let next = self
            .work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit {
                limit: self.limits.work,
            }));
        }
        self.work.set(next);
        Ok(())
    }
    pub(in crate::store) fn retain(
        &self,
        bytes: usize,
        units: usize,
        outputs: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let next_bytes = self
            .bytes
            .get()
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let next_units = self
            .units
            .get()
            .checked_add(units)
            .ok_or(DomError::RepresentationExhausted)?;
        let next_outputs = self
            .outputs
            .get()
            .checked_add(outputs)
            .ok_or(DomError::RepresentationExhausted)?;
        let runtime = self.runtime.get();
        admit(
            next_bytes
                .checked_add(runtime.bytes)
                .and_then(|n| n.checked_add(self.controlled.get().bytes()))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_bytes,
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            next_units
                .checked_add(runtime.units)
                .and_then(|n| n.checked_add(self.controlled.get().units()))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_units,
            DomResource::MetaAtomicRetainedUnits,
        )?;
        admit(
            next_outputs
                .checked_add(runtime.outputs)
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.outputs,
            DomResource::MetaAtomicOutputs,
        )?;
        if let Some(source) = &self.source {
            source
                .work
                .reserve_outputs(outputs, source.output_bound, self.limits)?;
            source.retain(
                source::storage::Payload { bytes, units },
                self.limits,
                cancel,
            )?;
        }
        self.bytes.set(next_bytes);
        self.units.set(next_units);
        self.outputs.set(next_outputs);
        Ok(())
    }
    pub(in crate::store) fn runtime(
        &self,
        usage: DomMetaRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let old = self.runtime.get();
        let next = DomMetaRuntimeUsage {
            bytes: old.bytes.max(usage.bytes),
            units: old.units.max(usage.units),
            outputs: old.outputs.max(usage.outputs),
        };
        admit(
            self.bytes
                .get()
                .checked_add(next.bytes)
                .and_then(|n| n.checked_add(self.controlled.get().bytes()))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_bytes,
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            self.units
                .get()
                .checked_add(next.units)
                .and_then(|n| n.checked_add(self.controlled.get().units()))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_units,
            DomResource::MetaAtomicRetainedUnits,
        )?;
        admit(
            self.outputs
                .get()
                .checked_add(next.outputs)
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.outputs,
            DomResource::MetaAtomicOutputs,
        )?;
        if let Some(source) = &self.source {
            source.work.usage(next, self.limits)?;
            source.work.reserve_outputs(
                next.outputs - old.outputs,
                source.output_bound,
                self.limits,
            )?;
        }
        self.runtime.set(next);
        Ok(())
    }
    /// The canonical registry already owns and prices each actual controlled
    /// allocation. Include only its extra simultaneous union over the captured
    /// pre-binding basis, retaining its peak beside this invocation's holders.
    pub(in crate::store) fn controlled_extra(
        &self,
        usage: custody::Usage,
        basis: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let previous = self.controlled.get();
        let next = custody::Usage::new(
            previous
                .bytes()
                .max(usage.bytes().saturating_sub(basis.bytes())),
            previous
                .units()
                .max(usage.units().saturating_sub(basis.units())),
        );
        let runtime = self.runtime.get();
        admit(
            self.bytes
                .get()
                .checked_add(runtime.bytes)
                .and_then(|n| n.checked_add(next.bytes()))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_bytes,
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            self.units
                .get()
                .checked_add(runtime.units)
                .and_then(|n| n.checked_add(next.units()))
                .ok_or(DomError::RepresentationExhausted)?,
            self.limits.retained_units,
            DomResource::MetaAtomicRetainedUnits,
        )?;
        if let Some(source) = &self.source {
            source
                .formation
                .as_ref()
                .expect("live Source graph")
                .admit_custody_usage(usage, cancel)?;
            let delta = next.checked_sub(previous)?;
            let own = source.own.get().checked_add(delta)?;
            source.work.check_private(delta, self.limits)?;
            source.work.retain_private(delta);
            source.own.set(own);
        }
        self.controlled.set(next);
        Ok(())
    }
    pub(in crate::store) fn into_source_charge(
        self,
        payload: source::storage::Payload,
    ) -> SourceGraphCharge {
        let mut source = self.source.expect("actual Source graph custody");
        let mut charge = source
            .formation
            .take()
            .expect("live Source graph")
            .into_charge();
        charge.shrink_to_payload(payload);
        let usage = custody::Usage::new(payload.bytes, payload.units);
        source.work.release_private(
            source
                .own
                .get()
                .checked_sub(usage)
                .expect("surviving graph is an admitted subset"),
        );
        source.own.set(custody::Usage::ZERO);
        SourceGraphCharge {
            work: source.work.clone(),
            usage,
            _charge: charge,
        }
    }
    fn release(&self, payload: source::storage::Payload) {
        self.bytes.set(
            self.bytes
                .get()
                .checked_sub(payload.bytes)
                .expect("owned native bytes"),
        );
        self.units.set(
            self.units
                .get()
                .checked_sub(payload.units)
                .expect("owned native units"),
        );
        if let Some(source) = &self.source {
            source.release(payload);
        }
    }
    pub(in crate::store) fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let required = values
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= values.capacity() {
            return Ok(());
        }
        let old = values.capacity();
        self.retain(
            required
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?,
            0,
            0,
            cancel,
        )?;
        let requested = required
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        // Keep the original bank intact until the replacement's actual capacity
        // is admitted; excess-capacity refusal cannot strand an unpriced bank.
        let mut replacement = Vec::new();
        if replacement.try_reserve_exact(required).is_err() {
            self.release(source::storage::Payload {
                bytes: requested,
                units: 0,
            });
            return Err(DomError::AllocationFailed);
        }
        let excess = (replacement.capacity() - required)
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        if let Err(error) = self.retain(excess, 0, 0, cancel) {
            drop(replacement);
            self.release(source::storage::Payload {
                bytes: requested,
                units: 0,
            });
            return Err(error);
        }
        replacement.append(values);
        std::mem::swap(values, &mut replacement);
        drop(replacement);
        let old_bytes = old
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        self.release(source::storage::Payload {
            bytes: old_bytes,
            units: 0,
        });
        Ok(())
    }
}
pub(in crate::store) struct Recording {
    pub(in crate::store) entries: Vec<DomMetaPreferredOccurrence>,
    pub(in crate::store) payloads: std::cell::RefCell<Vec<PayloadOwner>>,
    pub(in crate::store) envelope: Envelope,
}
pub(in crate::store) struct PayloadOwner {
    entry: usize,
    payload: source::storage::Payload,
}
impl Recording {
    pub(in crate::store) fn rollback(&mut self, len: usize) {
        self.entries.truncate(len);
        while self
            .payloads
            .get_mut()
            .last()
            .is_some_and(|p| p.entry >= len)
        {
            let payload = self
                .payloads
                .get_mut()
                .pop()
                .expect("actual admitted payload");
            self.envelope.release(payload.payload);
        }
    }
    pub(super) fn attach(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.envelope.retain(size_of::<Self>(), 0, 0, cancel)
    }
}
impl Draft {
    pub(super) fn admit_meta_content(
        &self,
        content: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let recording = self
            .meta_recording
            .as_ref()
            .expect("explicit Meta capability");
        recording.envelope.step(cancel)?;
        for prior in &recording.entries {
            recording.envelope.step(cancel)?;
            if Arc::ptr_eq(
                prior.content.shared_code_units(),
                content.shared_code_units(),
            ) {
                return Ok(());
            }
        }
        let payload = source::storage::Payload {
            bytes: content
                .shared_code_units()
                .capacity()
                .checked_mul(2)
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .ok_or(DomError::RepresentationExhausted)?,
            units: content.shared_code_units().capacity(),
        };
        if recording.envelope.source.is_some() {
            let mut payloads = recording.payloads.borrow_mut();
            recording.envelope.reserve(&mut payloads, 1, cancel)?;
            recording
                .envelope
                .retain(payload.bytes, payload.units, 0, cancel)?;
            payloads.push(PayloadOwner {
                entry: recording.entries.len(),
                payload,
            });
            Ok(())
        } else {
            recording
                .envelope
                .retain(payload.bytes, payload.units, 0, cancel)
        }
    }
    pub(super) fn record_meta_preferred(
        &mut self,
        document: Key,
        meta: Key,
        content: DocumentString,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Key::Existing(document) = document else {
            return Err(DomError::Meta(crate::DomMetaError::DefaultStyleUnsupported));
        };
        let meta = DomMetaSubject {
            kind: if let Some(id) = self.published_id(meta) {
                MetaSubjectKind::Existing(id)
            } else if let (Key::Created(index), Some((operation, scope))) =
                (meta, self.meta_scope.as_ref())
            {
                MetaSubjectKind::SourceCreated {
                    operation: *operation,
                    scope: scope.clone(),
                    index,
                }
            } else {
                let Key::Created(index) = meta else {
                    unreachable!("existing subjects resolve without allocation")
                };
                MetaSubjectKind::Created {
                    owner: self.owner.clone(),
                    index,
                }
            },
        };
        let recording = self
            .meta_recording
            .as_mut()
            .expect("explicit Meta capability");
        recording
            .envelope
            .reserve(&mut recording.entries, 1, cancel)?;
        recording.entries.push(DomMetaPreferredOccurrence {
            document,
            meta,
            content,
        });
        Ok(())
    }
}
pub(in crate::store) struct Root<'caps> {
    pub(in crate::store) root: NodeId,
    pub(in crate::store) store: MutexGuard<'caps, CssomStore>,
    pub(in crate::store) snapshot: CssomSnapshot,
    pub(in crate::store) batch: Option<CssomBatch>,
}
/// An ordinary private native publication. Drop abandons all staging and unlocks
/// its actual owners. No resolver, tentative producer output or live ID escapes.
#[must_use = "commit or drop the private Meta preparation"]
pub struct DomMetaOrdinaryPreparation<'dom, 'caps> {
    dom: &'dom mut Dom,
    publication: PreparedPublication,
    occurrences: Vec<DomMetaPreferredOccurrence>,
    roots: Vec<Root<'caps>>,
    inline: Option<CssomBatch>,
    envelope: Envelope,
}
/// Published custody retains all actual root locks while Runtime installs its
/// already-prepared names. Drop cannot undo this publication or enqueue work.
#[must_use = "extract the actual committed outputs after installing prepared names"]
pub struct DomMetaOrdinaryPublished<'caps> {
    _roots: Vec<Root<'caps>>,
    commit: DomMetaOrdinaryCommit,
    retirement: DomMetaRetirement,
}
impl DomMetaOrdinaryPublished<'_> {
    /// Release locks and transfer authentic outputs, with no further admission,
    /// validation, cancellation polling, allocation or canonical mutation.
    pub fn into_parts(self) -> (DomMetaOrdinaryCommit, DomMetaRetirement) {
        (self.commit, self.retirement)
    }
}
impl DomMetaOrdinaryPreparation<'_, '_> {
    pub fn occurrences(&self) -> &[DomMetaPreferredOccurrence] {
        &self.occurrences
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
        for row in &self.roots {
            self.envelope.step(cancel)?;
            if row.root == root {
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
        for row in &mut self.roots {
            self.envelope.step(cancel)?;
            if row.root != root {
                continue;
            }
            let mut member = false;
            for id in row.snapshot.sheets() {
                self.envelope.step(cancel)?;
                if id == sheet {
                    member = true;
                    break;
                }
            }
            if !member {
                return Err(DomMetaAtomicError::StaleProducer { root });
            }
            row.batch
                .as_mut()
                .expect("unattempted real batch")
                .set_disabled(sheet, disabled)
                .map_err(|error| DomMetaAtomicError::Cssom { root, error })?;
            return Ok(());
        }
        Err(DomMetaAtomicError::MissingOwner { root })
    }
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.envelope.step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomMetaRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.envelope.runtime(usage, cancel)
    }
}
impl<'caps> DomMetaOrdinaryPreparation<'_, 'caps> {
    /// Finish every actual producer preparation before publishing any owner.
    /// Err always precedes publication. The returned custody keeps root locks
    /// until its consuming extraction, for the caller's infallible name moves.
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    pub fn commit(
        self,
        cancel: &AtomicBool,
    ) -> Result<DomMetaOrdinaryPublished<'caps>, DomMetaAtomicError> {
        let Self {
            dom,
            publication,
            occurrences: _,
            mut roots,
            inline,
            envelope,
        } = self;
        let count = roots
            .len()
            .checked_add(usize::from(inline.is_some()))
            .ok_or(DomError::RepresentationExhausted)?;
        let mut outputs = Vec::new();
        envelope.reserve(&mut outputs, roots.len(), cancel)?;
        let mut retirement = Vec::new();
        envelope.reserve(&mut retirement, count, cancel)?;
        let mut prepared = Vec::new();
        envelope.reserve(&mut prepared, roots.len(), cancel)?;
        envelope.retain(
            size_of::<DomMetaOrdinaryPublished<'caps>>(),
            0,
            count
                .checked_add(2)
                .ok_or(DomError::RepresentationExhausted)?,
            cancel,
        )?;
        for row in &mut roots {
            envelope.step(cancel)?;
            let root = row.root;
            let batch = row.batch.take().expect("unattempted actual owner batch");
            let value = row
                .store
                .prepare_commit(batch)
                .map_err(|error| DomMetaAtomicError::Cssom { root, error })?;
            prepared.push((root, value));
        }
        let Dom {
            state,
            state_custody,
            style_cssom_roots,
            revision,
            history,
            active_task,
            inline_styles,
            ..
        } = dom;
        let (inline_store, inline_index, inline_objects) = match inline_styles.as_mut() {
            Some(owner) => (
                owner.store.as_mut(),
                Some(&mut owner.index),
                Some(&mut owner.objects),
            ),
            None => (None, None, None),
        };
        let inline_prepared = inline
            .map(|batch| {
                inline_store
                    .expect("recorded actual inline owner")
                    .prepare_commit(batch)
                    .map_err(crate::DomInlineStyleError::from)
            })
            .transpose()?;
        // This is the final fallible/cancellation/work check. Below are only
        // reserved output moves, producer publication and canonical DOM publish.
        envelope.step(cancel)?;
        for (root, value) in prepared {
            let (cssom, dead) = value.publish();
            outputs.push(DomMetaRootCommit { root, cssom });
            retirement.push(dead);
        }
        let inline = inline_prepared.map(|value| {
            let (commit, dead) = value.publish();
            retirement.push(dead);
            commit
        });
        let dom = PublicationFields {
            state,
            state_custody,
            style_cssom_roots,
            revision,
            history,
            active_task,
            inline_index,
            inline_objects,
        }
        .publish(publication)
        .commit;
        Ok(DomMetaOrdinaryPublished {
            _roots: roots,
            commit: DomMetaOrdinaryCommit {
                dom,
                roots: outputs,
                inline,
            },
            retirement: DomMetaRetirement { _cssom: retirement },
        })
    }
}

impl Dom {
    /// Prepare the actual ordinary native batch against borrowed canonical owner
    /// capabilities while Source is idle, including no Observed callback.
    /// An attempted error consumes the batch; Drop publishes nothing.
    /// The prefix is revalidated and seeds this envelope before any lower work.
    #[expect(
        clippy::result_large_err,
        reason = "retain the exact Document and original producer refusal without allocating on failure"
    )]
    pub fn prepare_meta_batch<'dom, 'caps>(
        &'dom mut self,
        batch: DomBatch,
        capabilities: &'caps [&'caps crate::DomStyleCssomRoot],
        inline: Option<DomMetaInlineBatch<'_>>,
        limits: DomMetaAtomicLimits,
        prefix: DomMetaRuntimePrefix,
        cancel: &AtomicBool,
    ) -> Result<DomMetaOrdinaryPreparation<'dom, 'caps>, DomMetaAtomicError> {
        prepare_ordinary(
            self,
            batch,
            capabilities,
            inline,
            (limits, prefix.usage, prefix.work),
            cancel,
        )
    }
}

#[expect(
    clippy::result_large_err,
    reason = "retain the exact Document and original producer refusal without allocating on failure"
)]
fn prepare_ordinary<'dom, 'caps>(
    dom: &'dom mut Dom,
    batch: DomBatch,
    capabilities: &'caps [&'caps crate::DomStyleCssomRoot],
    inline: Option<DomMetaInlineBatch<'_>>,
    request: (DomMetaAtomicLimits, DomMetaRuntimeUsage, u64),
    cancel: &AtomicBool,
) -> Result<DomMetaOrdinaryPreparation<'dom, 'caps>, DomMetaAtomicError> {
    // Ordinary composition is intentionally idle-only. In particular an
    // authenticated Observed callback cannot spend a Runtime-only prefix as
    // original Source/ancestor admission. The separate Source join must prepay it.
    if dom.source_operation.is_some() || !dom.source_parents.is_empty() {
        return Err(DomError::SourceOperationPending.into());
    }
    if dom.snapshot().incomplete_source_operation().is_some()
        || dom.snapshot().incomplete_hosted_operation().is_some()
    {
        return Err(DomError::IncompleteSourceOperation.into());
    }
    // Revalidate the initial declaration against this actual request. No lower
    // work has begun, and this envelope starts with the prepaid prefix already spent.
    let envelope = Envelope::new(request.0, request.1, request.2)?;
    dom.require_source_entry()?;
    envelope.step(cancel)?;
    let (recording, resolver) = match inline {
        Some(input) => {
            if !Arc::ptr_eq(&batch.owner, &input.context.batch) {
                return Err(DomError::ForeignBatch.into());
            }
            envelope.step(cancel)?;
            (
                dom.begin_inline_recording(input.context, cancel)?,
                Some((input.limits, input.resolver)),
            )
        }
        None => (None, None),
    };
    let meta = Recording {
        entries: Vec::new(),
        payloads: Default::default(),
        envelope,
    };
    let (prepared, recording) =
        dom.prepare_with_recordings(batch, recording, Some(meta), cancel)?;
    let Recording {
        entries: occurrences,
        payloads: _,
        envelope,
    } = recording.expect("explicit ordinary Meta recording");
    let mut publication = dom.prepare_publication(prepared, cancel)?;
    publication.close_ordinary_sharing(cancel)?;
    let inline = match resolver {
        Some((limits, resolver)) => {
            envelope.step(cancel)?;
            dom.prepare_inline_batch(&publication, limits, resolver, cancel)?
        }
        None => None,
    };
    dom.custody_registry.reconcile_releases();
    let registry_basis = dom.custody_registry.ledger().total();
    let mut roots: Vec<Root<'caps>> = Vec::new();
    for occurrence in &occurrences {
        envelope.step(cancel)?;
        let root = occurrence.document;
        let mut seen = false;
        for row in &roots {
            envelope.step(cancel)?;
            if row.root == root {
                seen = true;
                break;
            }
        }
        if seen {
            continue;
        }
        // Exact canonical generation and owner custody, rather than an equal
        // producer revision or an unrelated capability with the same root label.
        dom.state.node(dom.owner, root)?;
        let canonical = dom.style_cssom_roots.get(root);
        let mut capability: Option<&'caps crate::DomStyleCssomRoot> = None;
        for actual in capabilities {
            envelope.step(cancel)?;
            if actual.root() != root {
                continue;
            }
            if canonical.is_some_and(|bound| !bound.same_owner(actual))
                || capability.is_some_and(|selected| !selected.same_owner(actual))
            {
                return Err(DomMetaAtomicError::MismatchedOwner { root });
            }
            capability = Some(*actual);
        }
        let capability = capability.ok_or(DomMetaAtomicError::MissingOwner { root })?;
        envelope.reserve(&mut roots, 1, cancel)?;
        let store = capability.acquire().map_err(|error| match error {
            crate::style::CssomRootAccessError::Busy => DomMetaAtomicError::OwnerBusy { root },
            crate::style::CssomRootAccessError::Poisoned => {
                DomMetaAtomicError::OwnerPoisoned { root }
            }
        })?;
        let snapshot = store.snapshot();
        let batch = store
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .map_err(|error| DomMetaAtomicError::Cssom { root, error })?;
        // First binding is real mutable-owner custody, not a DOM tree edit.
        // Existing ownership remains authoritative even when this batch retires
        // that root; only surviving final Documents acquire new custody.
        if canonical.is_none()
            && publication
                .state
                .node(dom.owner, root)
                .is_ok_and(|node| matches!(node.kind(), crate::DomNodeKind::Document(_)))
        {
            let current = publication
                .style_custody
                .as_ref()
                .unwrap_or(&dom.style_cssom_roots);
            let candidate = current.prepare_insert(
                capability,
                DocumentId(root),
                |usage| envelope.controlled_extra(usage, registry_basis, cancel),
                || envelope.step(cancel),
            )?;
            publication.style_custody = Some(candidate);
        }
        roots.push(Root {
            root,
            store,
            snapshot,
            batch: Some(batch),
        });
    }
    envelope.retain(
        size_of::<DomMetaOrdinaryPreparation<'dom, 'caps>>(),
        0,
        0,
        cancel,
    )?;
    Ok(DomMetaOrdinaryPreparation {
        dom,
        publication,
        occurrences,
        roots,
        inline,
        envelope,
    })
}

#[cfg(test)]
mod tests;
