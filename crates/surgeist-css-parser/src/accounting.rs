//! The accounting schema of the CSS crates and the scope a host supplies for it.
//!
//! A host that attributes CSS work to a ledger operation of its own shares an
//! operation of the pair [`CssWork`] and [`CssResource`], and opens a
//! [`CssHostScope`] below one of its scopes. A CSS front that receives the
//! handle charges what it counted, once per outermost call, to the scope of
//! its [`CssFront`], and runs inside a phase of that scope. A front that
//! receives no handle has no operation to call.
//!
//! Request limits are not part of this schema. Each front checks its own
//! limits in its own counters, with or without a handle.
//!
//! # Scopes
//!
//! [`CssHostScope::open`] opens one scope per front below the supplied scope,
//! with no caps and with tariffs that map each dimension to itself:
//!
//! | Front | Scope name | Work it charges | Gauges it holds |
//! | --- | --- | --- | --- |
//! | [`CssFront::Input`] | `css-input` | [`CssWork::InputUnit`], [`CssWork::PreparationStep`] | |
//! | [`CssFront::Parse`] | `css-parse` | [`CssWork::Document`], [`CssWork::Component`], [`CssWork::SpellingByte`] | |
//! | [`CssFront::Serialization`] | `css-serialize` | [`CssWork::SerializedInputNode`], [`CssWork::SerializedProjectionNode`], [`CssWork::SerializedByte`] | |
//! | [`CssFront::Normalization`] | `css-normalize` | [`CssWork::NormalizedRule`], [`CssWork::NormalizedDeclaration`], [`CssWork::NormalizedContribution`] | |
//! | [`CssFront::SourceRetention`] | `css-source` | [`CssWork::SourceVisit`] | |
//! | [`CssFront::Color`] | `css-color` | [`CssWork::ColorInputUnit`], [`CssWork::ColorAction`] | |
//! | [`CssFront::Cssom`] | `cssom` | [`CssWork::SourceVisit`], [`CssWork::CensusRun`], [`CssWork::CensusObject`], [`CssWork::CensusEntry`], [`CssWork::RuleFormatNode`] | [`CssResource::RetainedObject`], [`CssResource::RetainedEntry`], [`CssResource::RetainedStringByte`] |
//!
//! A host adds [`CssFront::ALL`]'s length to the scope bound of its operation
//! for each handle it opens.
//!
//! # Where each front charges
//!
//! Each entry point below takes the handle, runs its work inside one phase of
//! its scope and charges what it counted as one batch when it returns,
//! refused or not; a refused call is charged the work admitted before the
//! refusal. The units are those of each dimension's documentation.
//!
//! - `css-input` and `css-parse`: the methods of `CssAccountedParser`, which
//!   `CssParser::accounted` returns in `surgeist-css`. A parse charges the
//!   documents it built, their components and spelling bytes to `css-parse`
//!   and its input units to `css-input`; a UTF-16 parse also charges its
//!   preparation steps to `css-input`, inside a phase of that scope.
//! - `css-serialize`: `serialize_specified` and `serialize_cssom` of the
//!   `CssAccounted` view that `accounted` returns on a sheet, a rule or a rule
//!   graph input in `surgeist-css`.
//! - `css-normalize`: `normalize` of that view on a sheet.
//! - `css-source`: `CssSourceRetention::charge_walk` in `surgeist-css`, the
//!   steps of the walk it runs.
//! - `css-color`: `parse_used_color` of the view on a colour input and
//!   `resolve` of the view on an authored colour in `surgeist-css`.
//! - `cssom`: every public operation of a store built with
//!   `CssomStore::with_host` in `surgeist-cssom`, of its batches and rule
//!   handles, and the beginning, finishing and cancelling of a sheet
//!   replacement. A commit charges before it publishes, with the increases
//!   of the store's gauges in the same batch, and releases the decreases when
//!   it publishes.
//!
//! An accounted call that another accounted call makes adds to the outer
//! call's record and is not charged on its own. No front charges inside a
//! loop over tokens, units, nodes or actions; a source check in
//! `surgeist-cssom` keeps it so.
//!
//! # What is not charged
//!
//! - Request limits. Each front checks its own limits with or without a
//!   handle, with the same amounts and in the same order.
//! - The stepwise protocols a host drives itself, such as this crate's
//!   `InputCursor` and `TokenCursor` with their `Meter`, and the controlled
//!   colour parse and serialization continuations: the host meters them.
//! - Serialization of a single value or declaration called on its own, whose
//!   work is a few hundred nanoseconds; it is part of the charge of an
//!   accounted sheet, rule or graph that contains it. Shortest-form
//!   comparisons run outside the serialization limits and are not counted.
//! - CSSOM identity, revision and owner-effect issuance, snapshots, captures
//!   and the parse a sheet replacement job runs outside its store.
//!
//! # Host caps
//!
//! The front scopes carry no caps. A cap of the supplied scope or of one of
//! its ancestors refuses the charge that would cross it, and the charge is a
//! [`CssHostRefusal`] with that scope's limit and total.
//! [`CssAccountingError::host_refusal`] tells a host cap from every other stop.
//!
//! ```
//! use surgeist_css_parser::accounting::{
//!     CssAccount, CssChange, CssFront, CssHostScope, CssLimits, CssResourceTable, CssWork,
//!     CssWorkFields, CssWorkTable,
//! };
//! use surgeist_ledger::Operation;
//!
//! // The host's operation: its root caps documents, and it leaves room for the
//! // front scopes.
//! let caps = CssLimits::new(
//!     CssWorkTable::new(CssWorkFields { Document: 100, ..CssWorkFields::splat(u64::MAX) }),
//!     CssResourceTable::splat(u64::MAX),
//! );
//! let operation = Operation::new("host", caps, 1 + CssFront::ALL.len())?.share();
//! let host = CssHostScope::open(operation.clone(), operation.root())?;
//!
//! // What an accounted parse front does at the end of its call.
//! let account = CssAccount::host(&host);
//! account.charge(CssFront::Parse, &[CssChange::Work(CssWork::Document, 60)])?;
//! assert_eq!(operation.usage(operation.root())?.work(CssWork::Document), 60);
//!
//! // The host's cap refuses the charge that would cross it.
//! let stop = account
//!     .charge(CssFront::Parse, &[CssChange::Work(CssWork::Document, 41)])
//!     .unwrap_err();
//! let refusal = stop.host_refusal().expect("a host cap refused the charge");
//! assert_eq!((refusal.limit(), refusal.total(), refusal.amount()), (100, 60, 41));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::{error::Error, fmt};

use surgeist_ledger::{
    Change, Defect, Error as LedgerError, Limits, RefusalReason, Scope, SharedOperation, Tariffs,
};

surgeist_ledger::dimensions! {
    /// Work the CSS crates charge. Every dimension is cumulative.
    pub enum CssWork => CssWorkTable, CssWorkFields {
        /// One byte or UTF-16 code unit of the original input.
        InputUnit,
        /// One step of decoding, preprocessing or checkpoint recovery.
        PreparationStep,
        /// One token document built from source text or from components.
        Document,
        /// One component built.
        Component,
        /// One byte of retained token spelling.
        SpellingByte,
        /// One input node serialized.
        SerializedInputNode,
        /// One projection node visited during serialization.
        SerializedProjectionNode,
        /// One byte of CSS text produced.
        SerializedByte,
        /// One rule normalized.
        NormalizedRule,
        /// One declaration normalized.
        NormalizedDeclaration,
        /// One contribution normalized.
        NormalizedContribution,
        /// One visit of a source retention walk, duplicate comparisons included.
        SourceVisit,
        /// One unit of colour input.
        ColorInputUnit,
        /// One action of a colour driver.
        ColorAction,
        /// One run of the CSSOM state census.
        CensusRun,
        /// One object the CSSOM state census examined.
        CensusObject,
        /// One entry the CSSOM state census examined.
        CensusEntry,
        /// One node formatted by CSSOM rule serialization.
        RuleFormatNode,
    }
}

surgeist_ledger::dimensions! {
    /// Live custody the CSS crates hold. Every dimension is a gauge.
    pub enum CssResource => CssResourceTable, CssResourceFields {
        /// Objects a CSSOM store retains.
        RetainedObject,
        /// Entries a CSSOM store retains.
        RetainedEntry,
        /// String bytes a CSSOM store retains.
        RetainedStringByte,
    }
}

/// One change of the CSS schema.
pub type CssChange = Change<CssWork, CssResource>;

/// Caps of a scope of the CSS schema.
pub type CssLimits = Limits<CssWork, CssResource>;

/// A host's shared operation of the CSS schema.
pub type CssLedger = SharedOperation<CssWork, CssResource>;

/// A CSS front that charges a scope of its own below the host's scope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssFront {
    /// Decoding and preprocessing of source input.
    Input,
    /// Token document building.
    Parse,
    /// Specified-value, rule and sheet serialization.
    Serialization,
    /// Sheet normalization.
    Normalization,
    /// Source retention walks.
    SourceRetention,
    /// Colour parsing and resolution.
    Color,
    /// CSSOM store operations.
    Cssom,
}

impl CssFront {
    /// Every front, in the order [`CssHostScope::open`] opens their scopes.
    pub const ALL: [Self; 7] = [
        Self::Input,
        Self::Parse,
        Self::Serialization,
        Self::Normalization,
        Self::SourceRetention,
        Self::Color,
        Self::Cssom,
    ];

    /// The most changes one charge of a front may carry: one for each
    /// dimension the widest front charges.
    pub const CHARGE_CHANGES: usize = 8;

    /// The name of the front's scope in the host's operation.
    #[must_use]
    pub const fn scope_name(self) -> &'static str {
        match self {
            Self::Input => "css-input",
            Self::Parse => "css-parse",
            Self::Serialization => "css-serialize",
            Self::Normalization => "css-normalize",
            Self::SourceRetention => "css-source",
            Self::Color => "css-color",
            Self::Cssom => "cssom",
        }
    }

    /// The work dimensions the front charges.
    #[must_use]
    pub const fn work(self) -> &'static [CssWork] {
        match self {
            Self::Input => &[CssWork::InputUnit, CssWork::PreparationStep],
            Self::Parse => &[CssWork::Document, CssWork::Component, CssWork::SpellingByte],
            Self::Serialization => &[
                CssWork::SerializedInputNode,
                CssWork::SerializedProjectionNode,
                CssWork::SerializedByte,
            ],
            Self::Normalization => &[
                CssWork::NormalizedRule,
                CssWork::NormalizedDeclaration,
                CssWork::NormalizedContribution,
            ],
            Self::SourceRetention => &[CssWork::SourceVisit],
            Self::Color => &[CssWork::ColorInputUnit, CssWork::ColorAction],
            Self::Cssom => &[
                CssWork::SourceVisit,
                CssWork::CensusRun,
                CssWork::CensusObject,
                CssWork::CensusEntry,
                CssWork::RuleFormatNode,
            ],
        }
    }

    /// The resource dimensions the front holds.
    #[must_use]
    pub const fn resources(self) -> &'static [CssResource] {
        match self {
            Self::Cssom => &[
                CssResource::RetainedObject,
                CssResource::RetainedEntry,
                CssResource::RetainedStringByte,
            ],
            Self::Input
            | Self::Parse
            | Self::Serialization
            | Self::Normalization
            | Self::SourceRetention
            | Self::Color => &[],
        }
    }

    /// Whether the front charges the dimension `change` names.
    fn charges(self, change: &CssChange) -> bool {
        match change {
            Change::Work(dimension, _) => self.work().contains(dimension),
            Change::Acquire(dimension, _) | Change::Release(dimension, _) => {
                self.resources().contains(dimension)
            }
        }
    }
}

/// The scopes of the CSS fronts below a scope of an operation the host owns.
///
/// The operation is an `Operation<CssWork, CssResource>` the host shared with
/// [`surgeist_ledger::Operation::share`]. The front scopes carry no caps, so
/// a cap that refuses a charge is one of the supplied scope or of a scope
/// above it. A refusal reports the dimension and the figures of the scope
/// that refused, in that scope's units. Where the tariffs between the
/// supplied scope and the scopes above it map each dimension to itself, they
/// are those of the charged change.
///
/// Cloning the value shares the operation and the front scopes.
#[derive(Clone)]
pub struct CssHostScope {
    ledger: CssLedger,
    parent: Scope,
    input: Scope,
    parse: Scope,
    serialization: Scope,
    normalization: Scope,
    source_retention: Scope,
    color: Scope,
    cssom: Scope,
}

impl CssHostScope {
    /// Opens the front scopes below `parent`, a scope of `ledger`.
    ///
    /// Each call opens its own scopes, so two handles opened below one scope
    /// charge different front scopes whose usage adds up in `parent`.
    ///
    /// # Errors
    ///
    /// [`CssAccountingFault::ForeignScope`] when `parent` is not a scope of
    /// `ledger`, and [`CssAccountingFault::ScopeLimit`] when the operation's
    /// scope bound leaves no room for every front scope. The ledger does not
    /// retire scopes, so the front scopes opened before the bound was reached
    /// stay in the operation, unused.
    pub fn open(ledger: CssLedger, parent: Scope) -> Result<Self, CssAccountingFault> {
        let open = |front: CssFront| {
            ledger
                .child(
                    parent,
                    front.scope_name(),
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(fault_of)
        };
        let input = open(CssFront::Input)?;
        let parse = open(CssFront::Parse)?;
        let serialization = open(CssFront::Serialization)?;
        let normalization = open(CssFront::Normalization)?;
        let source_retention = open(CssFront::SourceRetention)?;
        let color = open(CssFront::Color)?;
        let cssom = open(CssFront::Cssom)?;
        Ok(Self {
            ledger,
            parent,
            input,
            parse,
            serialization,
            normalization,
            source_retention,
            color,
            cssom,
        })
    }

    /// The scope `front` charges.
    #[must_use]
    pub const fn scope(&self, front: CssFront) -> Scope {
        match front {
            CssFront::Input => self.input,
            CssFront::Parse => self.parse,
            CssFront::Serialization => self.serialization,
            CssFront::Normalization => self.normalization,
            CssFront::SourceRetention => self.source_retention,
            CssFront::Color => self.color,
            CssFront::Cssom => self.cssom,
        }
    }
}

impl fmt::Debug for CssHostScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssHostScope")
            .field("parent", &self.parent)
            .finish_non_exhaustive()
    }
}

/// The accounting of one front call: the host scope it charges below, or none.
///
/// A plain front runs its core with [`CssAccount::NONE`]; an accounted front
/// runs the same core with [`CssAccount::host`]. With no host the account has
/// no operation, so neither method makes a ledger call.
#[derive(Clone, Copy, Debug)]
pub struct CssAccount<'host> {
    host: Option<&'host CssHostScope>,
}

impl<'host> CssAccount<'host> {
    /// The account of a call no host observes.
    pub const NONE: Self = Self { host: None };

    /// The account of a call that charges below `host`.
    #[must_use]
    pub const fn host(host: &'host CssHostScope) -> Self {
        Self { host: Some(host) }
    }

    /// Charges `changes` to the scope of `front` as one batch: all of them
    /// are admitted or none is. An empty batch, or an account with no host,
    /// makes no ledger call.
    ///
    /// # Errors
    ///
    /// [`CssAccountingError::Host`] when a cap of the host's scope or of a
    /// scope above it refuses a change. [`CssAccountingError::Fault`] when
    /// the batch is longer than [`CssFront::CHARGE_CHANGES`], when a change
    /// names a dimension `front` does not charge, or when the ledger fails
    /// for a reason other than a cap. A refused or faulted batch changes
    /// nothing.
    pub fn charge(&self, front: CssFront, changes: &[CssChange]) -> Result<(), CssAccountingError> {
        let Some(host) = self.host else {
            return Ok(());
        };
        if changes.is_empty() {
            return Ok(());
        }
        if changes.len() > CssFront::CHARGE_CHANGES {
            return Err(CssAccountingFault::ChargeLength {
                front,
                length: changes.len(),
            }
            .into());
        }
        if !changes.iter().all(|change| front.charges(change)) {
            return Err(CssAccountingFault::UnclassifiedChange { front }.into());
        }
        host.ledger
            .apply_batch(host.scope(front), changes)
            .map_err(classify)
    }

    /// Runs `run` inside a phase of the scope of `front`, so a host that
    /// profiles with timing reads the inclusive time of the front. With no
    /// host it only runs `run`.
    ///
    /// # Errors
    ///
    /// A [`CssAccountingFault`] when the ledger cannot open the phase; `run`
    /// has not been called.
    pub fn timed<T>(
        &self,
        front: CssFront,
        run: impl FnOnce() -> T,
    ) -> Result<T, CssAccountingFault> {
        match self.host {
            None => Ok(run()),
            Some(host) => host
                .ledger
                .with_phase(host.scope(front), run)
                .map_err(fault_of),
        }
    }
}

/// A charge that a cap of the host's scope, or of a scope above it, refused.
///
/// The figures are in the units of the scope that refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssHostRefusal {
    scope: Scope,
    change: CssChange,
    limit: u64,
    total: u64,
}

impl CssHostRefusal {
    /// The scope whose cap refused the charge.
    #[must_use]
    pub const fn scope(&self) -> Scope {
        self.scope
    }

    /// The refused change as the refusing scope saw it: its dimension and
    /// its amount in that scope's units.
    #[must_use]
    pub const fn change(&self) -> CssChange {
        self.change
    }

    /// The cap of the refusing scope on that dimension.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// The refusing scope's usage of that dimension when the change arrived,
    /// including the changes of the same batch that precede it.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// The amount of the refused change.
    #[must_use]
    pub const fn amount(&self) -> u64 {
        match self.change {
            Change::Work(_, amount) | Change::Acquire(_, amount) | Change::Release(_, amount) => {
                amount
            }
        }
    }
}

impl fmt::Display for CssHostRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a host cap of {} refused {:?} at a total of {}",
            self.limit, self.change, self.total
        )
    }
}

/// A ledger failure that is not a refusal at a host cap.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssAccountingFault {
    /// A release returned more than the scope held.
    ReleaseUnderflow,
    /// A counter of the ledger would have passed `u64::MAX`.
    CounterOverflow,
    /// The ledger could not allocate its storage.
    Allocation,
    /// The supplied scope belongs to another operation.
    ForeignScope,
    /// The operation's scope bound leaves no room for the front scopes.
    ScopeLimit {
        /// The operation's scope bound.
        limit: usize,
    },
    /// A charge named a dimension its front does not charge.
    UnclassifiedChange {
        /// The front that was charged.
        front: CssFront,
    },
    /// A charge carried more changes than [`CssFront::CHARGE_CHANGES`].
    ChargeLength {
        /// The front that was charged.
        front: CssFront,
        /// The number of changes in the charge.
        length: usize,
    },
    /// The ledger reported a condition that no call of this module can
    /// cause: one of operation setup or of a checkpoint, or a refusal where
    /// nothing is charged.
    Unexpected,
    /// The ledger found a broken invariant of its own. It stays latched in
    /// the operation.
    Defect(Defect),
}

impl fmt::Display for CssAccountingFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseUnderflow => {
                formatter.write_str("a release returned more than the scope held")
            }
            Self::CounterOverflow => formatter.write_str("a ledger counter would have overflowed"),
            Self::Allocation => formatter.write_str("the ledger could not allocate its storage"),
            Self::ForeignScope => {
                formatter.write_str("the supplied scope belongs to another operation")
            }
            Self::ScopeLimit { limit } => write!(
                formatter,
                "the operation's scope bound of {limit} leaves no room for the front scopes"
            ),
            Self::UnclassifiedChange { front } => write!(
                formatter,
                "a charge to {} named a dimension that front does not charge",
                front.scope_name()
            ),
            Self::ChargeLength { front, length } => write!(
                formatter,
                "a charge to {} carried {length} changes",
                front.scope_name()
            ),
            Self::Unexpected => {
                formatter.write_str("the ledger reported a condition no CSS accounting call causes")
            }
            Self::Defect(defect) => write!(formatter, "the ledger found a defect: {defect}"),
        }
    }
}

impl Error for CssAccountingFault {}

/// Why a charge was not admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssAccountingError {
    /// A cap of the host's scope, or of a scope above it, refused the charge.
    Host(CssHostRefusal),
    /// The ledger failed for a reason other than a cap.
    Fault(CssAccountingFault),
}

impl CssAccountingError {
    /// The refusal, when a host cap stopped the charge. Every other stop,
    /// including every request-limit refusal of a front, returns `None`.
    #[must_use]
    pub const fn host_refusal(&self) -> Option<&CssHostRefusal> {
        match self {
            Self::Host(refusal) => Some(refusal),
            Self::Fault(_) => None,
        }
    }
}

impl From<CssAccountingFault> for CssAccountingError {
    fn from(fault: CssAccountingFault) -> Self {
        Self::Fault(fault)
    }
}

impl fmt::Display for CssAccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(refusal) => refusal.fmt(formatter),
            Self::Fault(fault) => fault.fmt(formatter),
        }
    }
}

impl Error for CssAccountingError {}

/// Maps a ledger error of a charge. This is the one place that names the
/// ledger's error shapes for the CSS crates.
fn classify(error: LedgerError<CssWork, CssResource>) -> CssAccountingError {
    CssAccountingError::Fault(match error {
        LedgerError::Refused(refusal) => match refusal.reason {
            RefusalReason::Limit { limit } => {
                return CssAccountingError::Host(CssHostRefusal {
                    scope: refusal.scope,
                    change: refusal.change,
                    limit,
                    total: refusal.current,
                });
            }
            RefusalReason::Overflow => CssAccountingFault::CounterOverflow,
            RefusalReason::Underflow => CssAccountingFault::ReleaseUnderflow,
        },
        LedgerError::Allocation => CssAccountingFault::Allocation,
        LedgerError::ForeignScope => CssAccountingFault::ForeignScope,
        LedgerError::ScopeLimit { limit } => CssAccountingFault::ScopeLimit { limit },
        // Conditions of operation setup and of checkpoints, which this
        // module never performs.
        LedgerError::IdentityExhausted
        | LedgerError::TimingUnavailable
        | LedgerError::ForeignCheckpoint
        | LedgerError::StaleCheckpoint
        | LedgerError::CheckpointLimit { .. }
        | LedgerError::CheckpointsConfigured => CssAccountingFault::Unexpected,
        LedgerError::Defect(defect) => CssAccountingFault::Defect(defect),
    })
}

/// The fault of a ledger error where the CSS crates charge nothing a cap can
/// refuse: the opening of a scope or of a phase.
fn fault_of(error: LedgerError<CssWork, CssResource>) -> CssAccountingFault {
    match classify(error) {
        CssAccountingError::Fault(fault) => fault,
        CssAccountingError::Host(_) => CssAccountingFault::Unexpected,
    }
}

#[cfg(test)]
mod tests {
    //! The mapping of every ledger error shape. The expected classification
    //! is the one recorded for the schema: a cap is a host refusal, a counter
    //! at its range and an excess release are their own faults, and the
    //! conditions of calls this module never makes are one fault.

    use surgeist_ledger::{Operation, Refusal};

    use super::*;

    fn refusal(reason: RefusalReason) -> (LedgerError<CssWork, CssResource>, Scope) {
        let scope = Operation::<CssWork, CssResource>::new("host", Limits::unlimited(), 1)
            .unwrap()
            .root();
        let refused = Refusal {
            scope,
            item: 1,
            change: Change::Acquire(CssResource::RetainedEntry, 6),
            current: 9,
            reason,
        };
        (LedgerError::Refused(refused), scope)
    }

    #[test]
    fn a_cap_refusal_is_a_host_refusal_with_the_ledger_figures() {
        let (error, scope) = refusal(RefusalReason::Limit { limit: 12 });
        let CssAccountingError::Host(host) = classify(error) else {
            panic!("a cap refusal is a host refusal");
        };
        assert_eq!(host.scope(), scope);
        assert_eq!(
            host.change(),
            Change::Acquire(CssResource::RetainedEntry, 6)
        );
        assert_eq!((host.limit(), host.total(), host.amount()), (12, 9, 6));
        assert_eq!(fault_of(error), CssAccountingFault::Unexpected);
    }

    #[test]
    fn every_other_ledger_error_is_the_fault_recorded_for_it() {
        let (overflow, _) = refusal(RefusalReason::Overflow);
        let (underflow, _) = refusal(RefusalReason::Underflow);
        let expected = [
            (overflow, CssAccountingFault::CounterOverflow),
            (underflow, CssAccountingFault::ReleaseUnderflow),
            (LedgerError::Allocation, CssAccountingFault::Allocation),
            (LedgerError::ForeignScope, CssAccountingFault::ForeignScope),
            (
                LedgerError::ScopeLimit { limit: 3 },
                CssAccountingFault::ScopeLimit { limit: 3 },
            ),
            (
                LedgerError::Defect(Defect::ScopeEntry),
                CssAccountingFault::Defect(Defect::ScopeEntry),
            ),
            (
                LedgerError::IdentityExhausted,
                CssAccountingFault::Unexpected,
            ),
            (
                LedgerError::TimingUnavailable,
                CssAccountingFault::Unexpected,
            ),
            (
                LedgerError::ForeignCheckpoint,
                CssAccountingFault::Unexpected,
            ),
            (LedgerError::StaleCheckpoint, CssAccountingFault::Unexpected),
            (
                LedgerError::CheckpointLimit { limit: 2 },
                CssAccountingFault::Unexpected,
            ),
            (
                LedgerError::CheckpointsConfigured,
                CssAccountingFault::Unexpected,
            ),
        ];
        for (error, fault) in expected {
            assert_eq!(classify(error), CssAccountingError::Fault(fault), "{error}");
            assert_eq!(fault_of(error), fault, "{error}");
            assert_eq!(classify(error).host_refusal(), None, "{error}");
        }
    }
}
