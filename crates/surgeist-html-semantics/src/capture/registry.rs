//! One finite registry-only capture, using the original capture owner.
use super::*;
use crate::accounting::{Ledger, LedgerError, wide};
use preflight::ActionPolicy;
use std::mem::size_of;
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlCapScope, HtmlHostScope, HtmlLedgerOutcome, HtmlLimits, HtmlResource,
    aggregate_tariffs, classify,
};
use surgeist_html::bounded::lexical::{
    HtmlCustomNameScanAction, validate_custom_element_name_with,
};
use surgeist_html::values::DocumentString;
use surgeist_ledger::{Change, Operation, Scope};

/// Finite formation allowances for the replacement graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlRegistryFormationLimits {
    bytes: usize,
    units: usize,
}

impl HtmlRegistryFormationLimits {
    pub const fn new(bytes: usize, retained_utf16_units: usize) -> Self {
        Self {
            bytes,
            units: retained_utf16_units,
        }
    }
}

/// One complete registry and its owning immutable evidence.
/// The local registry identity is distinct from a consumer's canonical identity.
#[derive(Debug)]
pub struct HtmlRegistryCapture {
    capture: HtmlCapture,
    registry: HtmlCaptureRegistryId,
    retained: HtmlCaptureRetainedStorageUsage,
    peak: usize,
}

impl HtmlRegistryCapture {
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

/// First refusal of controlled registry formation.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlRegistryFormationError<E> {
    FormationBytes {
        limit: usize,
        required: usize,
    },
    FormationUnits {
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
impl<E> HtmlRegistryFormationError<E> {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Capture(error) => error.accounting_fault(),
            Self::FormationBytes { .. }
            | Self::FormationUnits { .. }
            | Self::RepresentationExhausted
            | Self::AllocationFailed
            | Self::Interrupted(_) => None,
        }
    }
}

impl<E: fmt::Display> fmt::Display for HtmlRegistryFormationError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormationBytes { limit, required } => write!(
                f,
                "registry requires {required} formation bytes, limit {limit}"
            ),
            Self::FormationUnits { limit, required } => write!(
                f,
                "registry requires {required} retained UTF-16 units, limit {limit}"
            ),
            Self::RepresentationExhausted => f.write_str("registry representation exhausted"),
            Self::AllocationFailed => f.write_str("registry backing allocation failed"),
            Self::Accounting(fault) => {
                write!(f, "registry formation accounting failed: {fault}")
            }
            Self::Capture(error) => error.fmt(f),
            Self::Interrupted(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlRegistryFormationError<E> {
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

enum Definitions<'a> {
    EmptyGlobal,
    Append {
        capture: &'a HtmlCapture,
        registry: &'a HtmlCaptureRegistryId,
        definition: &'a HtmlCustomElementDefinition,
    },
}

/// The formation byte allowance and the bytes a formation holds. The allowance is the cap of one
/// ledger scope on the largest total a formation needs at one point, and the live backing is a
/// balance of that scope.
struct Control {
    operation: Ledger,
    scope: Scope,
    fixed: usize,
    units: usize,
    unit_limit: usize,
}
impl Control {
    #[cfg(test)]
    fn new<E>(limit: usize, fixed: usize) -> Result<Self, HtmlRegistryFormationError<E>> {
        Self::charging(limit, fixed, None)
    }
    /// The control of a formation: a scope below the one `hosted` supplies when it is `Some`, else
    /// the root of an operation of its own.
    fn charging<E>(
        limit: usize,
        fixed: usize,
        hosted: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlRegistryFormationError<E>> {
        let caps = HtmlLimits::from_fn(
            |_| u64::MAX,
            |resource| match resource {
                HtmlResource::FormationBytes => wide(limit),
                _ => u64::MAX,
            },
        );
        let (operation, scope) = match hosted {
            Some(host) => {
                let mut operation = Ledger::Shared(host.ledger().clone());
                let scope = operation
                    .child(
                        host.scope(),
                        HtmlCapScope::RegistryFormation.name(),
                        caps,
                        aggregate_tariffs(),
                    )
                    .map_err(Self::ledger_error)?;
                (operation, scope)
            }
            None => {
                let operation = Operation::new(HtmlCapScope::RegistryFormation.name(), caps, 1)
                    .map_err(Self::ledger_error)?;
                let scope = operation.root();
                (Ledger::Owned(operation), scope)
            }
        };
        Ok(Self {
            operation,
            scope,
            fixed,
            units: 0,
            unit_limit: usize::MAX,
        })
    }
    /// The typed refusal of a ledger error: a refused allowance keeps its limit and the total the
    /// formation would have needed.
    fn ledger_error<E>(error: LedgerError) -> HtmlRegistryFormationError<E> {
        use HtmlRegistryFormationError as Error;
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
    fn backing<E>(&self) -> Result<usize, HtmlRegistryFormationError<E>> {
        let live = self
            .operation
            .read(self.scope, |usage| {
                usage.live(HtmlResource::FormationBacking)
            })
            .map_err(Self::ledger_error)?;
        usize::try_from(live).map_err(|_| HtmlRegistryFormationError::RepresentationExhausted)
    }
    /// The largest total admitted so far.
    fn peak<E>(&self) -> Result<usize, HtmlRegistryFormationError<E>> {
        let peak = self
            .operation
            .read(self.scope, |usage| usage.peak(HtmlResource::FormationBytes))
            .map_err(Self::ledger_error)?;
        usize::try_from(peak).map_err(|_| HtmlRegistryFormationError::RepresentationExhausted)
    }
    /// Admits a total the formation needs at one point, refusing one beyond the allowance.
    fn admit<E>(&mut self, required: usize) -> Result<(), HtmlRegistryFormationError<E>> {
        self.operation
            .record_largest(self.scope, HtmlResource::FormationBytes, wide(required))
            .map_err(Self::ledger_error)
    }
    /// Replaces a backing of `old` bytes with one of `actual`, which a formation holds live.
    fn replace_backing<E>(
        &mut self,
        old: usize,
        actual: usize,
    ) -> Result<(), HtmlRegistryFormationError<E>> {
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
    fn plus<E>(a: usize, b: usize) -> Result<usize, HtmlRegistryFormationError<E>> {
        a.checked_add(b)
            .ok_or(HtmlRegistryFormationError::RepresentationExhausted)
    }
    fn retain_units<E>(&mut self, additional: usize) -> Result<(), HtmlRegistryFormationError<E>> {
        let required = Self::plus(self.units, additional)?;
        if required > self.unit_limit {
            return Err(HtmlRegistryFormationError::FormationUnits {
                limit: self.unit_limit,
                required,
            });
        }
        self.units = required;
        Ok(())
    }
    fn step<E>(
        step: &mut impl FnMut(usize) -> Result<(), E>,
        units: usize,
    ) -> Result<(), HtmlRegistryFormationError<E>> {
        step(units).map_err(HtmlRegistryFormationError::Interrupted)
    }
}
/// The registry control paired with the caller's work step, applying the formation
/// byte policy to a pass's observations.
struct Formation<'a, E> {
    control: &'a mut Control,
    step: &'a mut dyn FnMut(usize) -> Result<(), E>,
}
impl<E> preflight::ActionPolicy for Formation<'_, E> {
    type Error = HtmlRegistryFormationError<E>;
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

impl<E> Formation<'_, E> {
    fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        entries: usize,
    ) -> Result<(), HtmlRegistryFormationError<E>> {
        preflight::reserve_storage(values, entries, true, true, &mut |action| {
            self.action(action)
        })
        .map_err(preflight_error)
    }

    fn append(
        &mut self,
        builder: &mut HtmlCaptureBuilder,
        scope: HtmlRegistryScope,
        previous: &[HtmlCustomElementDefinition],
        definition: &HtmlCustomElementDefinition,
    ) -> Result<usize, HtmlRegistryFormationError<E>> {
        use HtmlRegistryFormationError::Capture;

        self.on_work(1)?;
        if validate_custom_element_name_with(definition.name(), |action| {
            self.on_work(match action {
                HtmlCustomNameScanAction::ReservedPair => 2,
                _ => 1,
            })
        })?
        .is_err()
            || definition.local_name().is_empty()
        {
            return Err(Capture(HtmlCaptureError::InvalidDefinitionName));
        }
        let mut units = definition_units(definition)?;
        for existing in previous {
            self.on_work(1)?;
            if self.names_equal(existing.name(), definition.name())? {
                return Err(Capture(HtmlCaptureError::DuplicateDefinitionName));
            }
            units = Control::plus(units, definition_units(existing)?)?;
        }
        let count = Control::plus(previous.len(), 1)?;
        let charges = builder.admit(0, 0, 0, units, 1, count).map_err(Capture)?;
        let mut definitions = Vec::new();
        self.reserve(&mut definitions, count)?;
        for entry in previous.iter().chain(std::iter::once(definition)) {
            self.on_work(1)?;
            definitions.push(HtmlCustomElementDefinition::new(
                self.retain_string(entry.name())?,
                self.retain_string(entry.local_name())?,
                entry.form_associated(),
                entry.disable_shadow(),
            ));
        }
        builder.commit(charges).map_err(Capture)?;
        let index = builder.data.registries.len();
        builder.data.registries.push(RegistryRecord {
            definitions,
            complete: true,
            scope,
            scope_established: true,
        });
        Ok(index)
    }

    fn names_equal(
        &mut self,
        left: &DocumentString,
        right: &DocumentString,
    ) -> Result<bool, HtmlRegistryFormationError<E>> {
        if left.len() != right.len() {
            return Ok(false);
        }
        for index in 0..left.len() {
            self.on_work(2)?;
            if left.code_units()[index] != right.code_units()[index] {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn retain_string(
        &mut self,
        value: &DocumentString,
    ) -> Result<DocumentString, HtmlRegistryFormationError<E>> {
        self.on_work(1)?;
        let header = size_of::<Vec<u16>>();
        if value.is_whole_block() {
            let units = value.retained_units();
            self.control.retain_units(units)?;
            let backing = units
                .checked_mul(size_of::<u16>())
                .ok_or(HtmlRegistryFormationError::RepresentationExhausted)?;
            let bytes = Control::plus(header, backing)?;
            self.on_storage_before(bytes)?;
            self.on_storage_after(0, bytes, true)?;
            return Ok(value.clone());
        }
        // A range can borrow a much larger source block. Keep only its exact units
        // so the new capture does not need to retain or index the source block.
        let requested = value.len();
        self.control.retain_units(requested)?;
        self.on_storage_before(header)?;
        self.on_storage_after(0, header, true)?;
        let mut units = Vec::new();
        preflight::reserve_storage(&mut units, requested, true, true, &mut |action| {
            if let preflight::Action::StorageAfter { actual, .. } = action {
                let exposed = actual / size_of::<u16>();
                self.control.retain_units(exposed - requested)?;
            }
            self.action(action)
        })
        .map_err(preflight_error)?;
        for index in 0..requested {
            self.on_work(1)?;
            units.push(value.code_units()[index]);
        }
        Ok(DocumentString::from_utf16(units))
    }
}

fn definition_units<E>(
    definition: &HtmlCustomElementDefinition,
) -> Result<usize, HtmlRegistryFormationError<E>> {
    Control::plus(definition.name().len(), definition.local_name().len())
}

fn preflight_error<E>(
    error: preflight::Failure<HtmlRegistryFormationError<E>>,
) -> HtmlRegistryFormationError<E> {
    match error {
        preflight::Failure::Admission(error) => error,
        preflight::Failure::Capture(error) => HtmlRegistryFormationError::Capture(error),
        preflight::Failure::AllocationFailed => HtmlRegistryFormationError::AllocationFailed,
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
        step: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<HtmlRegistryCapture, HtmlRegistryFormationError<E>> {
        Self::form_registry(
            host_revision,
            Definitions::EmptyGlobal,
            formation_bytes,
            step,
            None,
            0,
        )
    }

    /// Forms the registry as [`Self::complete_empty_global_registry_with`] does, charging scopes
    /// below the scope `accounting` names instead of operations of its own. The formation opens
    /// `html-registry-formation`, which carries the allowance, and its capture builder opens
    /// `html-capture`. The host operation keeps the captured registry and the formation backing
    /// the capture holds; the formation peak is a gauge, which stays below the host scope. A cap
    /// of the host scope or of an ancestor refuses with the limit that scope carries.
    pub fn complete_empty_global_registry_with_accounting<E>(
        host_revision: HtmlHostFactsRevision,
        formation_bytes: usize,
        step: impl FnMut(usize) -> Result<(), E>,
        accounting: &HtmlHostScope,
    ) -> Result<HtmlRegistryCapture, HtmlRegistryFormationError<E>> {
        Self::form_registry(
            host_revision,
            Definitions::EmptyGlobal,
            formation_bytes,
            step,
            Some(accounting),
            0,
        )
    }

    /// Forms a new registry-only capture by appending to this capture's complete
    /// registry of known scope. Its original scope and definition order are
    /// preserved. All IDs in the returned capture are local to that new capture.
    ///
    /// `step` admits work and polls cancellation before each source observation.
    /// The finite byte allowance follows
    /// [the registry formation contract](Self::complete_empty_global_registry_with).
    /// Whole UTF-16 blocks are shared and their retained extent is charged to the
    /// new capture. Borrowed ranges admit their requested units before copying
    /// and check the exposed capacity before writing any units.
    pub fn append_registry_definition_with<E>(
        &self,
        registry: &HtmlCaptureRegistryId,
        definition: &HtmlCustomElementDefinition,
        host_revision: HtmlHostFactsRevision,
        limits: HtmlRegistryFormationLimits,
        step: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<HtmlRegistryCapture, HtmlRegistryFormationError<E>> {
        Self::form_registry(
            host_revision,
            Definitions::Append {
                capture: self,
                registry,
                definition,
            },
            limits.bytes,
            step,
            None,
            limits.units,
        )
    }

    fn form_registry<E>(
        host_revision: HtmlHostFactsRevision,
        definitions: Definitions<'_>,
        formation_bytes: usize,
        mut step: impl FnMut(usize) -> Result<(), E>,
        hosted: Option<&HtmlHostScope>,
        retained_units: usize,
    ) -> Result<HtmlRegistryCapture, HtmlRegistryFormationError<E>> {
        Control::step(&mut step, 1)?;
        let mut fixed = Control::plus(size_of::<Workspace>(), size_of::<HtmlRegistryCapture>())?;
        let (unit_limit, definition_limit) = match definitions {
            Definitions::EmptyGlobal => (0, 0),
            Definitions::Append { .. } => {
                fixed = Control::plus(fixed, size_of::<Vec<HtmlCustomElementDefinition>>())?;
                // One staged definition and, for borrowed input, its UTF-16 Vec.
                fixed = Control::plus(fixed, size_of::<HtmlCustomElementDefinition>())?;
                fixed = Control::plus(fixed, size_of::<Vec<u16>>())?;
                (usize::MAX, usize::MAX)
            }
        };
        let mut control = Control::charging(formation_bytes, fixed, hosted)?;
        control.unit_limit = retained_units;
        control.admit(fixed)?;
        let capture_limits = HtmlCaptureLimits::new(0, 0, 0, unit_limit, 0, 1, definition_limit, 0);
        let mut builder = match hosted {
            Some(host) => HtmlCaptureBuilder::with_accounting(capture_limits, host_revision, host)
                .map_err(HtmlRegistryFormationError::Accounting)?,
            None => HtmlCaptureBuilder::new(capture_limits, host_revision),
        };
        let mut scratch = preflight::DepthScratch::new();
        builder.enable_controlled_storage();
        let mut formation = Formation {
            control: &mut control,
            step: &mut step,
        };
        formation.reserve(&mut builder.data.registries, 1)?;
        let index = match definitions {
            Definitions::EmptyGlobal => {
                formation.on_work(1)?;
                let registry_call_headers = Control::plus(
                    size_of::<Option<Vec<HtmlCustomElementDefinition>>>(),
                    size_of::<HashSet<&DocumentString>>(),
                )?;
                let held = Control::plus(fixed, formation.control.backing()?)?;
                formation
                    .control
                    .admit(Control::plus(held, registry_call_headers)?)?;
                let registry = builder
                    .add_registry(Some(Vec::new()))
                    .map_err(HtmlRegistryFormationError::Capture)?;
                formation.on_work(1)?;
                builder
                    .establish_registry_scope(&registry, HtmlRegistryScope::Global)
                    .map_err(HtmlRegistryFormationError::Capture)?;
                registry.index
            }
            Definitions::Append {
                capture,
                registry,
                definition,
            } => {
                formation.on_work(1)?;
                let scope = capture
                    .registry_scope(registry)
                    .map_err(HtmlRegistryFormationError::Capture)?;
                let HtmlCapturedDefinitions::Complete(previous) = capture
                    .definitions(registry)
                    .map_err(HtmlRegistryFormationError::Capture)?
                else {
                    return Err(HtmlRegistryFormationError::Capture(
                        HtmlCaptureError::UnavailableDefinitions(registry.clone()),
                    ));
                };
                if scope == HtmlRegistryScope::Unavailable {
                    return Err(HtmlRegistryFormationError::Capture(
                        HtmlCaptureError::MissingFact {
                            subject: None,
                            fact: HtmlCaptureFact::RegistryScope,
                        },
                    ));
                }
                formation.append(&mut builder, scope, previous, definition)?
            }
        };
        builder
            .preflight_finish(&mut scratch, &mut |action| formation.action(action))
            .map_err(preflight_error)?;
        let retained =
            storage::retained_storage_usage_with(&builder.data, || Control::step(&mut step, 1))
                .map_err(|error| match error {
                    HtmlCaptureStorageError::Interrupted(error) => error,
                    HtmlCaptureStorageError::RepresentationExhausted => {
                        HtmlRegistryFormationError::RepresentationExhausted
                    }
                })?;
        Control::step(&mut step, 1)?;
        let required = Control::plus(
            Control::plus(fixed, control.backing()?)?,
            size_of::<CaptureData>(),
        )?;
        control.admit(required)?;
        let peak = control.peak()?;
        let registry = HtmlCaptureRegistryId {
            capture: builder.data.identity.clone(),
            index,
        };
        let capture = builder.publish();
        Ok(HtmlRegistryCapture {
            capture,
            registry,
            retained,
            peak,
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
    ) -> Result<(), HtmlRegistryFormationError<()>> {
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
            Err(HtmlRegistryFormationError::FormationBytes {
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
            Err(HtmlRegistryFormationError::FormationBytes {
                limit: 125,
                required: 130
            })
        );
        assert_eq!(control.backing::<()>(), Ok(30));
        // The peak is the largest total the allowance admitted; the refused 130 is not one.
        assert_eq!(control.peak::<()>(), Ok(120));
        assert_eq!(
            Control::plus::<()>(usize::MAX, 1),
            Err(HtmlRegistryFormationError::RepresentationExhausted)
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
                matches!(result, Err(HtmlRegistryFormationError::Interrupted((at, _)))
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
            Err(HtmlRegistryFormationError::FormationBytes { .. })
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
                + size_of::<HtmlRegistryCapture>()
                + actual_bank
                + size_of::<CaptureData>().max(registry_call_headers)
        );
    }

    /// A backing the formation never took cannot be released, a ledger that cannot allocate and
    /// a foreign scope are ledger failures: each is the one accounting fault, and none is
    /// reported as exhaustion or as a backing allocation of the formation itself.
    #[test]
    fn ledger_failures_other_than_a_refused_allowance_are_accounting_faults() {
        type Error = HtmlRegistryFormationError<()>;
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
            HtmlRegistryFormationError::AllocationFailed
        );
        assert_eq!(
            preflight_error::<()>(preflight::Failure::Capture(
                HtmlCaptureError::ForeignBuilder
            )),
            HtmlRegistryFormationError::Capture(HtmlCaptureError::ForeignBuilder)
        );
        assert_eq!(
            preflight_error(preflight::Failure::Admission(
                HtmlRegistryFormationError::Interrupted("cancelled")
            )),
            HtmlRegistryFormationError::Interrupted("cancelled")
        );
    }
}
