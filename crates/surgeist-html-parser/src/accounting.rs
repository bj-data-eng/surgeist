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

use crate::HtmlParseLimits;
use std::{
    error::Error as StdError,
    fmt,
    sync::{Arc, Mutex, OnceLock, PoisonError},
};
use surgeist_html::accounting::{
    HtmlChange, HtmlLimits, HtmlResource, HtmlWork, aggregate_tariffs, charging_tariffs,
};
use surgeist_ledger::{
    Change, Defect, Dimension, Error, Operation, ProfileMode, Report, Scope, SharedOperation,
    Tariffs,
};

/// The ledger error type of the HTML schema pair.
pub(crate) type LedgerError = Error<HtmlWork, HtmlResource>;

/// A ledger failure that is not a refusal of a charged change.
///
/// A refusal at a parse cap is reported as
/// [`HtmlParseStop::LimitExceeded`](crate::HtmlParseStop::LimitExceeded). Every
/// other ledger error stops the parse with one of these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlParseAccountingFault {
    /// The supplied scope belongs to another operation.
    ForeignScope,
    /// The operation's scope bound leaves no room for the scopes of the parse.
    ScopeLimit {
        /// The operation's scope bound.
        limit: usize,
    },
    /// The ledger could not allocate its storage.
    Allocation,
    /// The ledger's identities are exhausted.
    IdentityExhausted,
    /// Timed profiling needs a clock the platform does not supply.
    TimingUnavailable,
    /// The supplied accounting already serves another parse.
    InUse,
    /// A checkpoint operation failed. The parser opens no checkpoint, so only a
    /// ledger that reports one for another reason produces it.
    Checkpoint,
    /// The ledger refused a change where the parser charges none, such as the
    /// creation of a scope.
    UnexpectedRefusal,
    /// The ledger found a broken invariant of its own. It stays latched in the
    /// operation.
    Defect(Defect),
}

impl fmt::Display for HtmlParseAccountingFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignScope => f.write_str("the supplied scope belongs to another operation"),
            Self::ScopeLimit { limit } => {
                write!(
                    f,
                    "the operation's scope bound of {limit} leaves no room for the parse"
                )
            }
            Self::Allocation => f.write_str("the ledger could not allocate its storage"),
            Self::IdentityExhausted => f.write_str("the ledger's identities are exhausted"),
            Self::TimingUnavailable => f.write_str("timed profiling needs a host clock"),
            Self::InUse => f.write_str("the supplied accounting already serves another parse"),
            Self::Checkpoint => f.write_str("a ledger checkpoint operation failed"),
            Self::UnexpectedRefusal => {
                f.write_str("the ledger refused a change where the parser charges none")
            }
            Self::Defect(defect) => write!(f, "the ledger found a defect: {defect}"),
        }
    }
}

impl StdError for HtmlParseAccountingFault {}

/// The split of a ledger error: a refusal the parser maps to a limit stop, or a
/// fault.
pub(crate) enum Classified {
    Refusal(surgeist_ledger::Refusal<HtmlWork, HtmlResource>),
    Fault(HtmlParseAccountingFault),
}

pub(crate) fn classify(error: LedgerError) -> Classified {
    Classified::Fault(match error {
        Error::Refused(refusal) => return Classified::Refusal(refusal),
        Error::ForeignScope => HtmlParseAccountingFault::ForeignScope,
        Error::ScopeLimit { limit } => HtmlParseAccountingFault::ScopeLimit { limit },
        Error::Allocation => HtmlParseAccountingFault::Allocation,
        Error::IdentityExhausted => HtmlParseAccountingFault::IdentityExhausted,
        Error::TimingUnavailable => HtmlParseAccountingFault::TimingUnavailable,
        Error::ForeignCheckpoint
        | Error::StaleCheckpoint
        | Error::CheckpointLimit { .. }
        | Error::CheckpointsConfigured => HtmlParseAccountingFault::Checkpoint,
        Error::Defect(defect) => HtmlParseAccountingFault::Defect(defect),
    })
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
            Self::Admission => "html-parse-admission",
            Self::Construction => "html-parse-construction",
            Self::Finalization => "html-parse-finalization",
            Self::Publication => "html-parse-publication",
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
    pub(crate) fn bind(&self, limits: &HtmlParseLimits) -> Result<Bound, HtmlParseAccountingFault> {
        let mut charging = self.charging.lock().unwrap_or_else(PoisonError::into_inner);
        if charging.is_some() {
            return Err(HtmlParseAccountingFault::InUse);
        }
        let (ledger, parent) = match &self.kind {
            LeaseKind::Host { ledger, scope } => (ledger.clone(), *scope),
            LeaseKind::Profiled { mode, ledger } => {
                // The root, the two parse scopes and the four phase scopes.
                let operation =
                    Operation::with_profile("html-parse-host", HtmlLimits::unlimited(), 7, *mode)
                        .map_err(fault)?;
                let root = operation.root();
                let shared = operation.share();
                if ledger.set(shared.clone()).is_err() {
                    return Err(HtmlParseAccountingFault::InUse);
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

fn fault(error: LedgerError) -> HtmlParseAccountingFault {
    match classify(error) {
        Classified::Fault(fault) => fault,
        Classified::Refusal(_) => HtmlParseAccountingFault::UnexpectedRefusal,
    }
}

/// Creates the parse scopes below `parent`.
fn scopes(
    ledger: &SharedOperation<HtmlWork, HtmlResource>,
    parent: Scope,
    limits: &HtmlParseLimits,
    phases: bool,
) -> Result<(Scope, Scope, Option<PhaseScopes>), HtmlParseAccountingFault> {
    let cap = ledger
        .child(
            parent,
            "html-parse",
            limits.ledger_limits(),
            aggregate_tariffs(),
        )
        .map_err(fault)?;
    let charging = child(ledger, cap, "html-parse-charging", charging_tariffs())?;
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
) -> Result<Scope, HtmlParseAccountingFault> {
    ledger
        .child(parent, name, HtmlLimits::unlimited(), tariffs)
        .map_err(fault)
}

/// The private operation of a parse that selected no profiling: the cap scope is
/// the root, and the charging scope its only child.
pub(crate) fn private(
    limits: &HtmlParseLimits,
) -> Result<(Operation<HtmlWork, HtmlResource>, Scope, Scope), HtmlParseAccountingFault> {
    let mut operation = Operation::new("html-parse", limits.ledger_limits(), 2).map_err(fault)?;
    let cap = operation.root();
    let charging = operation
        .child(
            cap,
            "html-parse-charging",
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
            // The batch releases exactly the live balances read above, and the
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
    /// this the profile reports the two parse scopes only.
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
    ) -> Result<Option<Report<HtmlWork, HtmlResource>>, HtmlParseAccountingFault> {
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
        let in_use = HtmlParseStop::Accounting(HtmlParseAccountingFault::InUse);
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
    fn every_ledger_error_but_a_refusal_maps_to_a_fault() {
        use surgeist_ledger::Error;
        for (error, fault) in [
            (Error::ForeignScope, HtmlParseAccountingFault::ForeignScope),
            (
                Error::ScopeLimit { limit: 3 },
                HtmlParseAccountingFault::ScopeLimit { limit: 3 },
            ),
            (Error::Allocation, HtmlParseAccountingFault::Allocation),
            (
                Error::IdentityExhausted,
                HtmlParseAccountingFault::IdentityExhausted,
            ),
            (
                Error::TimingUnavailable,
                HtmlParseAccountingFault::TimingUnavailable,
            ),
            (
                Error::ForeignCheckpoint,
                HtmlParseAccountingFault::Checkpoint,
            ),
            (Error::StaleCheckpoint, HtmlParseAccountingFault::Checkpoint),
            (
                Error::CheckpointLimit { limit: 1 },
                HtmlParseAccountingFault::Checkpoint,
            ),
            (
                Error::CheckpointsConfigured,
                HtmlParseAccountingFault::Checkpoint,
            ),
            (
                Error::Defect(Defect::ScopeEntry),
                HtmlParseAccountingFault::Defect(Defect::ScopeEntry),
            ),
        ] {
            let Classified::Fault(found) = classify(error) else {
                panic!("{error:?} is not a refusal");
            };
            assert_eq!(found, fault);
        }
    }
}
