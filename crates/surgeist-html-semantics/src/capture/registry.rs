//! One finite registry-only capture, using the original capture owner.
use super::*;
use crate::accounting::{LedgerError, wide};
use preflight::ActionPolicy;
use std::mem::size_of;
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlCapScope, HtmlLedgerOutcome, HtmlLimits, HtmlResource, HtmlWork,
    classify,
};
use surgeist_html::values::DocumentString;
use surgeist_ledger::{Change, Charge, Operation, Scope};

/// A complete-empty Global registry and its owning immutable evidence.
/// The local registry identity is distinct from a consumer's canonical identity.
#[derive(Debug)]
pub struct HtmlEmptyGlobalRegistryCapture {
    capture: HtmlCapture,
    registry: HtmlCaptureRegistryId,
    retained: HtmlCaptureRetainedStorageUsage,
    peak: usize,
}

impl HtmlEmptyGlobalRegistryCapture {
    pub fn capture(&self) -> &HtmlCapture {
        &self.capture
    }
    pub fn registry(&self) -> &HtmlCaptureRegistryId {
        &self.registry
    }
    /// Actual final capture payload; excludes this wrapper and its ID handle.
    pub fn retained_storage_usage(&self) -> HtmlCaptureRetainedStorageUsage {
        self.retained
    }
    /// Largest checked formation extent, including the fixed workspace and
    /// this returned wrapper, plus simultaneously live requested/actual backing.
    pub fn formation_peak_bytes(&self) -> usize {
        self.peak
    }
    /// Moves the actual evidence and local identity without cloning their graph.
    pub fn into_parts(self) -> (HtmlCapture, HtmlCaptureRegistryId) {
        (self.capture, self.registry)
    }
}

/// First refusal of the fixed empty-Global formation operation.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlEmptyGlobalRegistryError<E> {
    FormationBytes {
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    /// A fallible backing reservation reported an allocation failure. Infallible
    /// standard-library Arc allocation retains its ordinary allocation contract.
    AllocationFailed,
    /// The ledger that carries the formation allowance failed in a way that is not a refused
    /// allowance.
    Accounting(HtmlAccountingFault),
    Capture(HtmlCaptureError),
    Interrupted(E),
}
impl<E> HtmlEmptyGlobalRegistryError<E> {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Capture(error) => error.accounting_fault(),
            Self::FormationBytes { .. }
            | Self::RepresentationExhausted
            | Self::AllocationFailed
            | Self::Interrupted(_) => None,
        }
    }
}

impl<E: fmt::Display> fmt::Display for HtmlEmptyGlobalRegistryError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormationBytes { limit, required } => write!(
                f,
                "empty Global registry requires {required} formation bytes, limit {limit}"
            ),
            Self::RepresentationExhausted => {
                f.write_str("empty Global registry representation exhausted")
            }
            Self::AllocationFailed => {
                f.write_str("empty Global registry backing allocation failed")
            }
            Self::Accounting(fault) => {
                write!(
                    f,
                    "empty Global registry formation accounting failed: {fault}"
                )
            }
            Self::Capture(error) => error.fmt(f),
            Self::Interrupted(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlEmptyGlobalRegistryError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Accounting(fault) => Some(fault),
            Self::Capture(error) => Some(error),
            Self::Interrupted(error) => Some(error),
            _ => None,
        }
    }
}

// The layout of the real inline owners, conservatively live for the complete synchronous call:
// admission prices `size_of::<Workspace>()`, and the call itself holds the three as locals. The
// builder includes CaptureData/identity/Vec headers; scratch includes all four empty finish
// banks. The optional handle is the original builder handle.
struct Workspace {
    _builder: HtmlCaptureBuilder,
    _scratch: preflight::DepthScratch,
    _registry: Option<HtmlCaptureBuilderRegistry>,
}

/// The formation byte allowance and the bytes a formation holds. The allowance is the cap of one
/// ledger scope on the largest total a formation needs at one point, and the live backing is a
/// balance of that scope.
struct Control {
    operation: Operation<HtmlWork, HtmlResource>,
    scope: Scope,
    fixed: usize,
}
impl Control {
    fn new<E>(limit: usize, fixed: usize) -> Result<Self, HtmlEmptyGlobalRegistryError<E>> {
        let operation = Operation::new(
            HtmlCapScope::RegistryFormation.name(),
            HtmlLimits::from_fn(
                |_| u64::MAX,
                |resource| match resource {
                    HtmlResource::FormationBytes => wide(limit),
                    _ => u64::MAX,
                },
            ),
            1,
        )
        .map_err(Self::ledger_error)?;
        let scope = operation.root();
        Ok(Self {
            operation,
            scope,
            fixed,
        })
    }
    /// The typed refusal of a ledger error: a refused allowance keeps its limit and the total the
    /// formation would have needed.
    fn ledger_error<E>(error: LedgerError) -> HtmlEmptyGlobalRegistryError<E> {
        use HtmlEmptyGlobalRegistryError as Error;
        let refusal = match classify(error) {
            HtmlLedgerOutcome::Fault(fault) => return Error::Accounting(fault),
            HtmlLedgerOutcome::Exhausted(_) => return Error::RepresentationExhausted,
            HtmlLedgerOutcome::Cap(refusal) => refusal,
        };
        match (refusal.change(), refusal.figures()) {
            (Change::Acquire(HtmlResource::FormationBytes, _), Ok(figures)) => {
                Error::FormationBytes {
                    limit: figures.limit,
                    required: figures.required,
                }
            }
            (Change::Acquire(HtmlResource::FormationBytes, _), Err(_)) => {
                Error::RepresentationExhausted
            }
            _ => Error::Accounting(HtmlAccountingFault::UnexpectedChange),
        }
    }
    /// The live backing bytes.
    fn backing<E>(&self) -> Result<usize, HtmlEmptyGlobalRegistryError<E>> {
        let live = self
            .operation
            .usage(self.scope)
            .map_err(Self::ledger_error)?
            .live(HtmlResource::FormationBacking);
        usize::try_from(live).map_err(|_| HtmlEmptyGlobalRegistryError::RepresentationExhausted)
    }
    /// The largest total admitted so far.
    fn peak<E>(&self) -> Result<usize, HtmlEmptyGlobalRegistryError<E>> {
        let peak = self
            .operation
            .usage(self.scope)
            .map_err(Self::ledger_error)?
            .peak(HtmlResource::FormationBytes);
        usize::try_from(peak).map_err(|_| HtmlEmptyGlobalRegistryError::RepresentationExhausted)
    }
    /// Admits a total the formation needs at one point, refusing one beyond the allowance.
    fn admit<E>(&mut self, required: usize) -> Result<(), HtmlEmptyGlobalRegistryError<E>> {
        self.operation
            .charge_at(self.scope)
            .record_largest(HtmlResource::FormationBytes, wide(required))
            .map_err(Self::ledger_error)
    }
    /// Replaces a backing of `old` bytes with one of `actual`, which a formation holds live.
    fn replace_backing<E>(
        &mut self,
        old: usize,
        actual: usize,
    ) -> Result<(), HtmlEmptyGlobalRegistryError<E>> {
        self.operation
            .apply_batch(
                self.scope,
                &[
                    Change::Release(HtmlResource::FormationBacking, wide(old)),
                    Change::Acquire(HtmlResource::FormationBacking, wide(actual)),
                ],
            )
            .map_err(Self::ledger_error)
    }
    fn plus<E>(a: usize, b: usize) -> Result<usize, HtmlEmptyGlobalRegistryError<E>> {
        a.checked_add(b)
            .ok_or(HtmlEmptyGlobalRegistryError::RepresentationExhausted)
    }
    fn step<E>(
        step: &mut impl FnMut(usize) -> Result<(), E>,
        units: usize,
    ) -> Result<(), HtmlEmptyGlobalRegistryError<E>> {
        step(units).map_err(HtmlEmptyGlobalRegistryError::Interrupted)
    }
}
/// The registry control paired with the caller's work step, applying the formation
/// byte policy to a pass's observations.
struct Formation<'a, E> {
    control: &'a mut Control,
    step: &'a mut dyn FnMut(usize) -> Result<(), E>,
}
impl<E> preflight::ActionPolicy for Formation<'_, E> {
    type Error = HtmlEmptyGlobalRegistryError<E>;
    fn on_work(&mut self, units: usize) -> Result<(), Self::Error> {
        Control::step(&mut self.step, units)
    }
    fn on_reserve(&mut self, _entries: usize) -> Result<(), Self::Error> {
        Control::step(&mut self.step, 0)
    }
    fn on_storage_before(&mut self, requested: usize) -> Result<(), Self::Error> {
        Control::step(&mut self.step, 0)?;
        let control = &mut *self.control;
        let held = Control::plus(control.fixed, control.backing()?)?;
        control.admit(Control::plus(held, requested)?)
    }
    fn on_storage_after(
        &mut self,
        old: usize,
        actual: usize,
        _retained: bool,
    ) -> Result<(), Self::Error> {
        // The request was admitted before allocation. Record/check any
        // extra exposed capacity before proceeding or publishing.
        let control = &mut *self.control;
        let replacement_peak =
            Control::plus(Control::plus(control.fixed, control.backing()?)?, actual)?;
        control.replace_backing(old, actual)?;
        control.admit(replacement_peak)
    }
    fn on_storage_transfer(&mut self, _bytes: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn on_element_values(
        &mut self,
        _bytes: usize,
        _units: usize,
        _observed: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn preflight_error<E>(
    error: preflight::Failure<HtmlEmptyGlobalRegistryError<E>>,
) -> HtmlEmptyGlobalRegistryError<E> {
    match error {
        preflight::Failure::Admission(error) => error,
        preflight::Failure::Capture(error) => HtmlEmptyGlobalRegistryError::Capture(error),
        preflight::Failure::AllocationFailed => HtmlEmptyGlobalRegistryError::AllocationFailed,
    }
}

impl HtmlCapture {
    /// Forms exactly one complete-empty Global registry, with no captured nodes.
    ///
    /// `formation_bytes` is the caller's current total allowance for this entire
    /// synchronous producer call. `step` admits actual work and polls cancellation
    /// before source observations; it must not allocate other retained graphs or
    /// change that allowance. Used work is not refunded after a refusal.
    ///
    /// Before the first heap allocation, the allowance covers the real builder,
    /// finish scratch and builder handle workspace, plus the returned wrapper.
    /// The original registry call's empty definition Vec argument and empty
    /// validation HashSet header are also admitted while live. Scalar numerical
    /// control bookkeeping is excluded, as in the controlled OptionValue owner.
    /// Registry backing is admitted before its requested-capacity reservation;
    /// its actual exposed capacity is checked immediately afterward. The final
    /// CaptureData header is admitted before Arc publication, while the workspace
    /// remains priced. Public result/ID headers are included during formation;
    /// consumers price their own retained inline occurrences after transfer.
    ///
    /// The receipt uses the original capture storage observer and excludes opaque
    /// Arc control blocks/allocator bookkeeping, including the zero-payload capture
    /// identity. Requested capacity admission is not a strict allocator/RSS bound:
    /// safe Vec reservation may expose extra capacity, which is checked before
    /// proceeding. Every refusal drops all partial product owners. All fallible
    /// validation, admission and receipt observations precede publication.
    pub fn complete_empty_global_registry_with<E>(
        host_revision: HtmlHostFactsRevision,
        formation_bytes: usize,
        mut step: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<HtmlEmptyGlobalRegistryCapture, HtmlEmptyGlobalRegistryError<E>> {
        Control::step(&mut step, 1)?;
        let fixed = Control::plus(
            size_of::<Workspace>(),
            size_of::<HtmlEmptyGlobalRegistryCapture>(),
        )?;
        let mut control = Control::new(formation_bytes, fixed)?;
        control.admit(fixed)?;
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(0, 0, 0, 0, 0, 1, 0, 16 * 1024 * 1024),
            host_revision,
        );
        let mut scratch = preflight::DepthScratch::new();
        builder.enable_controlled_storage();
        // Pre-reserve the exact existing registry bank. The original add_registry
        // call then retains its complete validation path without another growth.
        preflight::reserve_storage(&mut builder.data.registries, 1, true, true, &mut |action| {
            Formation {
                control: &mut control,
                step: &mut step,
            }
            .action(action)
        })
        .map_err(preflight_error)?;
        Control::step(&mut step, 1)?;
        let registry_call_headers = Control::plus(
            size_of::<Option<Vec<HtmlCustomElementDefinition>>>(),
            size_of::<HashSet<&DocumentString>>(),
        )?;
        control.admit(Control::plus(
            Control::plus(fixed, control.backing()?)?,
            registry_call_headers,
        )?)?;
        let registry = builder
            .add_registry(Some(Vec::new()))
            .map_err(HtmlEmptyGlobalRegistryError::Capture)?;
        Control::step(&mut step, 1)?;
        builder
            .establish_registry_scope(&registry, HtmlRegistryScope::Global)
            .map_err(HtmlEmptyGlobalRegistryError::Capture)?;
        builder
            .preflight_finish(&mut scratch, &mut |action| {
                Formation {
                    control: &mut control,
                    step: &mut step,
                }
                .action(action)
            })
            .map_err(preflight_error)?;
        let retained =
            storage::retained_storage_usage_with(&builder.data, || Control::step(&mut step, 1))
                .map_err(|error| match error {
                    HtmlCaptureStorageError::Interrupted(error) => error,
                    HtmlCaptureStorageError::RepresentationExhausted => {
                        HtmlEmptyGlobalRegistryError::RepresentationExhausted
                    }
                })?;
        Control::step(&mut step, 1)?;
        let required = Control::plus(
            Control::plus(fixed, control.backing()?)?,
            size_of::<CaptureData>(),
        )?;
        control.admit(required)?;
        let registry = HtmlCaptureRegistryId {
            capture: builder.data.identity.clone(),
            index: registry.index,
        };
        let capture = builder.publish();
        Ok(HtmlEmptyGlobalRegistryCapture {
            capture,
            registry,
            retained,
            peak: control.peak()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html::values::DocumentString;

    /// Applies one observation to the formation policy with a work step that never refuses.
    fn action(
        control: &mut Control,
        action: preflight::Action,
    ) -> Result<(), HtmlEmptyGlobalRegistryError<()>> {
        Formation {
            control,
            step: &mut |_| Ok(()),
        }
        .action(action)
    }

    #[test]
    fn requested_backing_is_admitted_before_growth_and_extra_capacity_is_checked() {
        let mut control = Control::new::<()>(119, 100).unwrap();
        assert_eq!(
            action(
                &mut control,
                preflight::Action::StorageBefore { requested: 20 }
            ),
            Err(HtmlEmptyGlobalRegistryError::FormationBytes {
                limit: 119,
                required: 120
            })
        );
        assert_eq!(control.backing::<()>(), Ok(0));
        let mut control = Control::new::<()>(125, 100).unwrap();
        assert_eq!(
            action(
                &mut control,
                preflight::Action::StorageBefore { requested: 20 }
            ),
            Ok(())
        );
        assert_eq!(
            action(
                &mut control,
                preflight::Action::StorageAfter {
                    old: 0,
                    actual: 30,
                    retained: true,
                }
            ),
            Err(HtmlEmptyGlobalRegistryError::FormationBytes {
                limit: 125,
                required: 130
            })
        );
        assert_eq!(control.backing::<()>(), Ok(30));
        // The peak is the largest total the allowance admitted; the refused 130 is not one.
        assert_eq!(control.peak::<()>(), Ok(120));
        assert_eq!(
            Control::plus::<()>(usize::MAX, 1),
            Err(HtmlEmptyGlobalRegistryError::RepresentationExhausted)
        );
    }

    #[test]
    fn every_interruption_drops_all_partial_capture_owners() {
        let revision = HtmlHostFactsRevision::fresh();
        let mut calls = 0;
        HtmlCapture::complete_empty_global_registry_with(revision.clone(), usize::MAX, |_| {
            calls += 1;
            Ok::<_, ()>(())
        })
        .unwrap();
        for stop in 0..calls {
            let mut seen = 0;
            let result = HtmlCapture::complete_empty_global_registry_with(
                revision.clone(),
                usize::MAX,
                |units| {
                    let current = seen;
                    seen += 1;
                    if current == stop {
                        Err((current, units))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(
                matches!(result, Err(HtmlEmptyGlobalRegistryError::Interrupted((at, _)))
                if at == stop)
            );
            assert_eq!(seen, stop + 1);
            // A retained partial CaptureData would still own this revision.
            assert_eq!(Arc::strong_count(&revision.0), 1);
        }
        let result = HtmlCapture::complete_empty_global_registry_with(revision.clone(), 0, |_| {
            Ok::<_, ()>(())
        });
        assert!(matches!(
            result,
            Err(HtmlEmptyGlobalRegistryError::FormationBytes { .. })
        ));
        assert_eq!(Arc::strong_count(&revision.0), 1);
    }

    #[test]
    fn receipt_counts_each_real_header_once_and_only_actual_registry_backing() {
        let formed = HtmlCapture::complete_empty_global_registry_with(
            HtmlHostFactsRevision::fresh(),
            usize::MAX,
            |_| Ok::<_, ()>(()),
        )
        .unwrap();
        let data = &formed.capture.0;
        let actual_bank = data.registries.capacity() * size_of::<RegistryRecord>();
        assert!(data.registries.capacity() >= 1);
        assert_eq!(data.records.capacity(), 0);
        assert_eq!(data.registries[0].definitions.capacity(), 0);
        let retained = size_of::<CaptureData>() + actual_bank;
        assert_eq!(formed.retained.retained_bytes(), retained);
        assert_eq!(formed.retained.utf16_extent(), 0);
        // Both inline owners remain conservatively priced through publication.
        let registry_call_headers = size_of::<Option<Vec<HtmlCustomElementDefinition>>>()
            + size_of::<HashSet<&DocumentString>>();
        assert_eq!(
            formed.peak,
            size_of::<Workspace>()
                + size_of::<HtmlEmptyGlobalRegistryCapture>()
                + actual_bank
                + size_of::<CaptureData>().max(registry_call_headers)
        );
    }

    /// A backing the formation never took cannot be released, a ledger that cannot allocate and
    /// a foreign scope are ledger failures: each is the one accounting fault, and none is
    /// reported as exhaustion or as a backing allocation of the formation itself.
    #[test]
    fn ledger_failures_other_than_a_refused_allowance_are_accounting_faults() {
        type Error = HtmlEmptyGlobalRegistryError<()>;
        let mut control = Control::new::<()>(1_000, 10).unwrap();
        assert_eq!(
            control.replace_backing::<()>(5, 8),
            Err(Error::Accounting(HtmlAccountingFault::ReleaseUnderflow))
        );
        assert_eq!(control.backing::<()>(), Ok(0));
        for (error, fault) in [
            (LedgerError::Allocation, HtmlAccountingFault::Allocation),
            (LedgerError::ForeignScope, HtmlAccountingFault::ForeignScope),
            (
                LedgerError::ScopeLimit { limit: 1 },
                HtmlAccountingFault::ScopeLimit { limit: 1 },
            ),
        ] {
            assert_eq!(Control::ledger_error::<()>(error), Error::Accounting(fault));
        }
    }

    #[test]
    fn existing_safe_allocation_capture_and_admission_causes_are_distinct() {
        assert_eq!(
            preflight_error::<()>(preflight::Failure::AllocationFailed),
            HtmlEmptyGlobalRegistryError::AllocationFailed
        );
        assert_eq!(
            preflight_error::<()>(preflight::Failure::Capture(
                HtmlCaptureError::ForeignBuilder
            )),
            HtmlEmptyGlobalRegistryError::Capture(HtmlCaptureError::ForeignBuilder)
        );
        assert_eq!(
            preflight_error(preflight::Failure::Admission(
                HtmlEmptyGlobalRegistryError::Interrupted("cancelled")
            )),
            HtmlEmptyGlobalRegistryError::Interrupted("cancelled")
        );
    }
}
