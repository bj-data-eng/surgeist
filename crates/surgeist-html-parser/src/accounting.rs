//! Where a parse charges its work and resources.
//!
//! A parse charges every work category and resource through a `surgeist-ledger`
//! operation of the HTML schema pair, [`HtmlWork`] and [`HtmlResource`]. Without
//! a request setting the parse builds its own operation. A host that attributes
//! HTML work to its own operation supplies a [`HtmlParseAccounting`], which names
//! a shared operation and a scope of it.
//!
//! The scope tree a parse creates and the custody its report keeps are described
//! on [`HtmlParseAccounting`].

use crate::{HtmlParseLimits, HtmlParseStop};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlChange, HtmlLedgerOutcome,
    HtmlLimits, HtmlResource, HtmlWork, aggregate_tariffs, charging_tariffs, classify,
};
use surgeist_ledger::{
    Change, Dimension, Error, Operation, ProfileMode, Report, Scope, SharedOperation, Tariffs,
};

/// The ledger error type of the HTML schema pair.
pub(crate) type LedgerError = Error<HtmlWork, HtmlResource>;

/// Why a parse could not set up its accounting: the supplied accounting already serves another
/// parse, or the ledger failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SetupFault {
    InUse,
    Accounting(HtmlAccountingFault),
}

impl SetupFault {
    /// The stop a parse that could not set up reports.
    pub(crate) const fn stop(self) -> HtmlParseStop {
        match self {
            Self::InUse => HtmlParseStop::AccountingInUse,
            Self::Accounting(fault) => HtmlParseStop::Accounting(fault),
        }
    }
}

impl From<HtmlAccountingFault> for SetupFault {
    fn from(fault: HtmlAccountingFault) -> Self {
        Self::Accounting(fault)
    }
}

/// The scopes of the parse phases that receive work when phases are observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParsePhase {
    Admission,
    Construction,
    Finalization,
    Publication,
}

impl ParsePhase {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Admission => HtmlAuxiliaryScope::ParseAdmission.name(),
            Self::Construction => HtmlAuxiliaryScope::ParseConstruction.name(),
            Self::Finalization => HtmlAuxiliaryScope::ParseFinalization.name(),
            Self::Publication => HtmlAuxiliaryScope::ParsePublication.name(),
        }
    }
}

/// The accounting handle shared by a request, its parse and its report or
/// failure. Dropping the last holder releases the custody a supplied operation
/// still holds for the parse.
pub(crate) struct Lease {
    kind: LeaseKind,
    /// Whether the parse creates and observes the phase scopes.
    phases: bool,
    /// The charging scope, set when a parse binds the lease. Binding holds the
    /// lock from the check to the end of scope creation, so one parse binds.
    charging: Mutex<Option<Scope>>,
}

enum LeaseKind {
    /// An operation and scope the host supplied.
    Host {
        ledger: SharedOperation<HtmlWork, HtmlResource>,
        scope: Scope,
    },
    /// The parse's own operation, shared so that its profile outlives the
    /// parse.
    Profiled {
        mode: ProfileMode,
        ledger: OnceLock<SharedOperation<HtmlWork, HtmlResource>>,
    },
}

impl Lease {
    fn new(kind: LeaseKind, phases: bool) -> Arc<Self> {
        Arc::new(Self {
            kind,
            phases,
            charging: Mutex::new(None),
        })
    }

    /// The profile of the parse's own operation, if profiling was selected.
    pub(crate) fn profile(&self) -> Result<Option<Report<HtmlWork, HtmlResource>>, LedgerError> {
        match &self.kind {
            LeaseKind::Profiled { ledger, .. } => {
                ledger.get().map_or(Ok(None), SharedOperation::snapshot)
            }
            LeaseKind::Host { .. } => Ok(None),
        }
    }
}

/// The scopes one parse created below its supplied scope.
pub(crate) struct Bound {
    pub(crate) ledger: SharedOperation<HtmlWork, HtmlResource>,
    pub(crate) cap: Scope,
    pub(crate) charging: Scope,
    pub(crate) phases: Option<PhaseScopes>,
}

/// One unlimited scope per parse phase.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PhaseScopes {
    admission: Scope,
    construction: Scope,
    finalization: Scope,
    publication: Scope,
}

impl PhaseScopes {
    /// Every phase at one scope, for a test that points the phases elsewhere.
    #[cfg(test)]
    pub(crate) const fn all_at(scope: Scope) -> Self {
        Self {
            admission: scope,
            construction: scope,
            finalization: scope,
            publication: scope,
        }
    }
    pub(crate) const fn scope(&self, phase: ParsePhase) -> Scope {
        match phase {
            ParsePhase::Admission => self.admission,
            ParsePhase::Construction => self.construction,
            ParsePhase::Finalization => self.finalization,
            ParsePhase::Publication => self.publication,
        }
    }
}

impl Lease {
    /// Creates the parse's scopes and records its charging scope, once. A lease
    /// serves one parse: the check and the creation of the scopes happen under
    /// one lock, so a second parse creates none.
    pub(crate) fn bind(&self, limits: &HtmlParseLimits) -> Result<Bound, SetupFault> {
        let mut charging = self.charging.lock().unwrap_or_else(PoisonError::into_inner);
        if charging.is_some() {
            return Err(SetupFault::InUse);
        }
        let (ledger, parent) = match &self.kind {
            LeaseKind::Host { ledger, scope } => (ledger.clone(), *scope),
            LeaseKind::Profiled { mode, ledger } => {
                // The root, the two parse scopes and the four phase scopes.
                let operation = Operation::with_profile(
                    HtmlAuxiliaryScope::ParseHost.name(),
                    HtmlLimits::unlimited(),
                    7,
                    *mode,
                )
                .map_err(fault)?;
                let root = operation.root();
                let shared = operation.share();
                if ledger.set(shared.clone()).is_err() {
                    return Err(SetupFault::InUse);
                }
                (shared, root)
            }
        };
        let (cap, scope, phases) = scopes(&ledger, parent, limits, self.phases)?;
        *charging = Some(scope);
        Ok(Bound {
            ledger,
            cap,
            charging: scope,
            phases,
        })
    }
}

/// The fault of a ledger error where the parser charges no change, such as the creation of a
/// scope or a profile read.
fn fault(error: LedgerError) -> HtmlAccountingFault {
    match classify(error) {
        HtmlLedgerOutcome::Fault(fault) => fault,
        HtmlLedgerOutcome::Cap(_) | HtmlLedgerOutcome::Exhausted(_) => {
            HtmlAccountingFault::UnexpectedChange
        }
    }
}

/// Creates the parse scopes below `parent`.
fn scopes(
    ledger: &SharedOperation<HtmlWork, HtmlResource>,
    parent: Scope,
    limits: &HtmlParseLimits,
    phases: bool,
) -> Result<(Scope, Scope, Option<PhaseScopes>), HtmlAccountingFault> {
    let cap = ledger
        .child(
            parent,
            HtmlCapScope::Parse.name(),
            limits.ledger_limits(),
            aggregate_tariffs(),
        )
        .map_err(fault)?;
    let charging = child(
        ledger,
        cap,
        HtmlAuxiliaryScope::ParseCharging.name(),
        charging_tariffs(),
    )?;
    let phases = if phases {
        let phase = |phase: ParsePhase| child(ledger, charging, phase.name(), Tariffs::identity());
        Some(PhaseScopes {
            admission: phase(ParsePhase::Admission)?,
            construction: phase(ParsePhase::Construction)?,
            finalization: phase(ParsePhase::Finalization)?,
            publication: phase(ParsePhase::Publication)?,
        })
    } else {
        None
    };
    Ok((cap, charging, phases))
}

fn child(
    ledger: &SharedOperation<HtmlWork, HtmlResource>,
    parent: Scope,
    name: &'static str,
    tariffs: surgeist_html::accounting::HtmlTariffs,
) -> Result<Scope, HtmlAccountingFault> {
    ledger
        .child(parent, name, HtmlLimits::unlimited(), tariffs)
        .map_err(fault)
}

/// The private operation of a parse that selected no profiling: the cap scope is
/// the root, and the charging scope its only child.
pub(crate) fn private(
    limits: &HtmlParseLimits,
) -> Result<(Operation<HtmlWork, HtmlResource>, Scope, Scope), HtmlAccountingFault> {
    let mut operation =
        Operation::new(HtmlCapScope::Parse.name(), limits.ledger_limits(), 2).map_err(fault)?;
    let cap = operation.root();
    let charging = operation
        .child(
            cap,
            HtmlAuxiliaryScope::ParseCharging.name(),
            HtmlLimits::unlimited(),
            charging_tariffs(),
        )
        .map_err(fault)?;
    Ok((operation, cap, charging))
}

impl Drop for Lease {
    fn drop(&mut self) {
        let LeaseKind::Host { ledger, .. } = &self.kind else {
            return;
        };
        let charging = *self.charging.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(charging) = charging else {
            return;
        };
        // A usage the ledger cannot read is a latched defect; the operation's
        // next fallible call returns it.
        let Ok(usage) = ledger.usage(charging) else {
            return;
        };
        const N: usize = HtmlResource::ALL.len();
        let mut changes: [HtmlChange; N] = [Change::Release(HtmlResource::SourceUnits, 0); N];
        let mut count = 0_usize;
        for &resource in HtmlResource::ALL {
            let live = usage.live(resource);
            if live == 0 {
                continue;
            }
            if let Some(slot) = changes.get_mut(count) {
                *slot = Change::Release(resource, live);
                count = count.saturating_add(1);
            }
        }
        if let Some(batch) = changes.get(..count) {
            // The batch releases the live balances read above, and the
            // only writer of the charging scope is the parse that has ended. A
            // refusal leaves the ledger unchanged; a defect is latched there.
            let _ = ledger.apply_batch(charging, batch);
        }
    }
}

/// Where a parse charges its work and resources.
///
/// A request takes one through `with_accounting`; without one the parse charges
/// an operation of its own that nothing observes. There are two kinds:
///
/// - [`Self::host`] charges a scope of an operation the host owns, so the host
///   sees the parse once in its own totals.
/// - [`Self::profiled`] charges the parse's own operation under a ledger profile
///   mode, read back from the report's or failure's `profile()`.
///
/// # Scope tree
///
/// Under the supplied scope, or under the root `html-parse-host` of a profiled
/// operation, the parse creates two scopes:
///
/// ```text
/// supplied scope                  (the host's)
/// └── html-parse                  caps of the parse limits; aggregate tariffs to the supplied scope
///     └── html-parse-charging     unlimited; charging tariffs to html-parse
/// ```
///
/// Every resource and every gauge is charged in `html-parse-charging`, which
/// also records the work of each category. `html-parse` records the credit total
/// and carries the caps. A gauge, such as the tree depth, stays in the scope that
/// observed it, so the supplied scope sees no gauge of the parse. With
/// [`Self::observing_phases`], four more unlimited scopes below
/// `html-parse-charging` receive the work of the parse phases:
/// `html-parse-admission`, `html-parse-construction`, `html-parse-finalization`
/// and `html-parse-publication`. Each phase is observed as a ledger phase, so a
/// profile of the operation reports the time and admissions of each.
///
/// A refusal by a cap of the supplied scope or of an ancestor stops the parse
/// with a limit stop that carries that scope's limit and value.
///
/// A host operation never frees a scope. A parse adds two scopes to it, six with
/// phase observation, so an operation that runs several parses sizes its scope
/// bound for all of them. A profiled operation holds the root and the scopes of
/// its one parse.
///
/// # Custody
///
/// A report or failure of a hosted parse keeps the accounting handle. When the
/// last holder drops, the live balances that remain in `html-parse-charging` are
/// released into the operation, once, so the operation shows no live custody of
/// the parse afterward. A release the ledger refuses cannot panic; the custody
/// stays, and a defect found there latches in the operation and is returned by
/// its next fallible call.
///
/// A value serves one parse; it is not `Clone`.
pub struct HtmlParseAccounting {
    source: AccountingSource,
    phases: bool,
}

enum AccountingSource {
    Host {
        ledger: SharedOperation<HtmlWork, HtmlResource>,
        scope: Scope,
    },
    Profiled(ProfileMode),
}

impl HtmlParseAccounting {
    /// Charges in `scope` of `ledger`, an operation of the HTML schema pair that
    /// the host owns.
    ///
    /// The tariffs between `scope` and the scopes above it must map each parse
    /// resource to a parse resource. The schema does not promise a retargeting
    /// to another dimension. A cap of a scope above such a tariff that refuses a
    /// parse charge stops the parse with `HtmlParseStop::Accounting`
    /// (`HtmlAccountingFault::UnexpectedChange`), because the refusal names a
    /// dimension the parse does not report.
    pub fn host(ledger: SharedOperation<HtmlWork, HtmlResource>, scope: Scope) -> Self {
        Self {
            source: AccountingSource::Host { ledger, scope },
            phases: false,
        }
    }

    /// Charges the parse's own operation, profiled in `mode`.
    /// [`ProfileMode::Off`] reads no clock and keeps no observation.
    pub fn profiled(mode: ProfileMode) -> Self {
        Self {
            source: AccountingSource::Profiled(mode),
            phases: false,
        }
    }

    /// Also creates the four phase scopes and observes each parse phase as a
    /// ledger phase. For a host operation the host's profile reports them. For a
    /// profiled operation the report's or failure's `profile()` does; without
    /// this the profile reports `html-parse-host` and the two parse scopes.
    #[must_use]
    pub fn observing_phases(mut self) -> Self {
        self.phases = true;
        self
    }

    /// What a request keeps: nothing for a profile mode that observes nothing,
    /// else the lease the parse and its report share.
    pub(crate) fn into_request(self) -> RequestAccounting {
        let phases = self.phases;
        match self.source {
            AccountingSource::Host { ledger, scope } => {
                RequestAccounting(Some(Lease::new(LeaseKind::Host { ledger, scope }, phases)))
            }
            AccountingSource::Profiled(ProfileMode::Off) if !phases => RequestAccounting(None),
            AccountingSource::Profiled(mode) => RequestAccounting(Some(Lease::new(
                LeaseKind::Profiled {
                    mode,
                    ledger: OnceLock::new(),
                },
                phases,
            ))),
        }
    }
}

impl std::fmt::Debug for HtmlParseAccounting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("HtmlParseAccounting");
        match &self.source {
            AccountingSource::Host { scope, .. } => debug.field("scope", scope),
            AccountingSource::Profiled(mode) => debug.field("profile", mode),
        };
        debug.field("phases", &self.phases).finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Lease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lease").finish_non_exhaustive()
    }
}

/// What a request selected for its accounting: the parse's own unprofiled
/// operation, the parse's own profiled operation, or a host's operation.
#[derive(Clone, Debug, Default)]
pub(crate) struct RequestAccounting(Option<Arc<Lease>>);

impl RequestAccounting {
    pub(crate) fn lease(&self) -> Option<&Arc<Lease>> {
        self.0.as_ref()
    }
    /// A snapshot of the profile of the parse's own operation. A parse in a
    /// host's operation, or without profiling, has none to report.
    pub(crate) fn profile(
        &self,
    ) -> Result<Option<Report<HtmlWork, HtmlResource>>, HtmlAccountingFault> {
        self.0
            .as_ref()
            .map_or(Ok(None), |lease| lease.profile())
            .map_err(fault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HtmlParseCancellation, HtmlParseStop, HtmlParseWorkCategory, parser::control::ParseControl,
    };
    use surgeist_ledger::Operation;

    fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(10)
            .buffered_utf16_units(10)
            .token_attributes(10)
            .auxiliary_entries(10)
            .active_formatting_entries(10)
            .managed_nodes(10)
            .managed_edges(10)
            .managed_attributes(10)
            .managed_utf16_units(10)
            .source_map_entries(10)
            .source_map_payload_units(10)
            .diagnostics(10)
            .diagnostic_payload_units(10)
            .context_effects(10)
            .open_elements(10)
            .tree_depth(10)
            .managed_capacity_bytes(100)
            .peak_transient_bytes(100)
            .atomic_layout_bytes(100)
            .work_credits(100)
            .finish()
            .unwrap()
    }

    fn host(scopes: usize) -> (SharedOperation<HtmlWork, HtmlResource>, Scope) {
        let ledger = Operation::new("host", HtmlLimits::unlimited(), scopes)
            .unwrap()
            .share();
        let root = ledger.root();
        (ledger, root)
    }

    #[test]
    fn a_lease_serves_one_parse() {
        let (ledger, root) = host(8);
        let request = HtmlParseAccounting::host(ledger, root).into_request();
        let mut first =
            ParseControl::with_accounting(limits(), HtmlParseCancellation::new(), request.lease());
        let mut second =
            ParseControl::with_accounting(limits(), HtmlParseCancellation::new(), request.lease());
        assert_eq!(first.stop(), None);
        first.charge(HtmlParseWorkCategory::StateStep, 1).unwrap();
        let in_use = HtmlParseStop::AccountingInUse;
        assert_eq!(second.stop(), Some(in_use));
        assert_eq!(
            second.charge(HtmlParseWorkCategory::StateStep, 1),
            Err(in_use)
        );
        // The control that could not set up charges nothing and reads as empty.
        assert_eq!(second.usage().work(HtmlParseWorkCategory::StateStep), 0);
    }

    #[test]
    fn an_owned_operation_takes_no_lease_and_a_profile_off_request_selects_none() {
        let request = HtmlParseAccounting::profiled(ProfileMode::Off).into_request();
        assert!(request.lease().is_none());
        assert!(request.profile().unwrap().is_none());
        let profiled =
            HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false }).into_request();
        assert!(profiled.lease().is_some());
        // No parse has run, so the operation does not exist yet.
        assert!(profiled.profile().unwrap().is_none());
    }

    #[test]
    fn parses_racing_to_bind_one_lease_create_one_set_of_scopes() {
        use std::sync::Barrier;
        const RACERS: usize = 8;
        let ledger = Operation::with_profile(
            "host",
            HtmlLimits::unlimited(),
            64,
            ProfileMode::Summary { timing: false },
        )
        .unwrap()
        .share();
        let root = ledger.root();
        let request = HtmlParseAccounting::host(ledger.clone(), root).into_request();
        let lease = request.lease().unwrap();
        let barrier = Barrier::new(RACERS);
        let limits = limits();
        let outcomes: Vec<Result<Bound, SetupFault>> = std::thread::scope(|scope| {
            let racers: Vec<_> = (0..RACERS)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        lease.bind(&limits)
                    })
                })
                .collect();
            racers
                .into_iter()
                .map(|racer| racer.join().unwrap())
                .collect()
        });
        let bound = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
        assert_eq!(bound, 1);
        for outcome in outcomes {
            if let Err(fault) = outcome {
                assert_eq!(fault, SetupFault::InUse);
            }
        }
        // The root and the two scopes of the one parse that bound; the others
        // created none.
        assert_eq!(ledger.snapshot().unwrap().unwrap().scopes.len(), 3);
    }
}
