//! Private accounting owner for parsing. Counts describe admitted physical work
//! and owned storage, not CPU instructions, elapsed time, allocator metadata or RSS.

use crate::accounting::{Lease, LedgerError, ParsePhase};
use crate::keyed::{Table, slots};
use crate::settings::{RESOURCES, ResourceTable};
use crate::{
    HtmlParseCancellation, HtmlParseLimits, HtmlParseResource, HtmlSource, HtmlSourceCoordinates,
    HtmlSourceId,
};
use std::{
    marker::PhantomData,
    mem::size_of,
    sync::{Arc, Weak},
};
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlChange, HtmlLedgerOutcome, HtmlResource, HtmlWork, classify,
    record_largest, retained_provider_storage,
};
use surgeist_ledger::Change;

mod accounts;
use accounts::{Accounts, LedgerUsage};
mod admission;
use admission::Site;
mod attributes;
mod construction;
mod css_value;
mod finalization;
pub(crate) use css_value::CssSchemeValue;
pub(crate) use finalization::FinalReleaseError;
#[cfg(test)]
pub(crate) use finalization::{
    FinalizationCut, FinalizationObservation, FinalizationTransferObservation,
    FinalizationTransferStage,
};
mod string_extension;
mod text_append;
pub(crate) use construction::ConstructionAllocation;

const WORK_COUNT: usize = 8;
/// One value per [`HtmlParseWorkCategory`]; a lookup cannot be out of range.
type WorkTable<T> = Table<HtmlParseWorkCategory, T, WORK_COUNT>;
pub(crate) const WORK_QUANTUM: usize = 256;
/// The resources that are plain storage: every one that
/// [`HtmlParseResource::storage`] accepts.
const STORAGE_RESOURCES: usize = 14;

/// Each category has weight one; units are physical bytes or individual events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum HtmlParseWorkCategory {
    InputUnitRead,
    InspectByte,
    CopyByte,
    StateStep,
    RecordVisit,
    RecordWrite,
    ReserveByte,
    ProviderCall,
}
slots!(HtmlParseWorkCategory, WORK_COUNT, {
    InputUnitRead => 0,
    InspectByte => 1,
    CopyByte => 2,
    StateStep => 3,
    RecordVisit => 4,
    RecordWrite => 5,
    ReserveByte => 6,
    ProviderCall => 7,
});
impl HtmlParseWorkCategory {
    /// Every category, in declaration order.
    const ALL: [Self; WORK_COUNT] = [
        Self::InputUnitRead,
        Self::InspectByte,
        Self::CopyByte,
        Self::StateStep,
        Self::RecordVisit,
        Self::RecordWrite,
        Self::ReserveByte,
        Self::ProviderCall,
    ];
    /// The schema work dimension that carries this category.
    pub(crate) const fn work(self) -> HtmlWork {
        match self {
            Self::InputUnitRead => HtmlWork::InputUnitRead,
            Self::InspectByte => HtmlWork::InspectByte,
            Self::CopyByte => HtmlWork::CopyByte,
            Self::StateStep => HtmlWork::StateStep,
            Self::RecordVisit => HtmlWork::RecordVisit,
            Self::RecordWrite => HtmlWork::RecordWrite,
            Self::ReserveByte => HtmlWork::ReserveByte,
            Self::ProviderCall => HtmlWork::ProviderCall,
        }
    }
    /// One work change.
    pub(crate) const fn change(self, units: u64) -> HtmlChange {
        Change::Work(self.work(), units)
    }
}

/// A state the parser's own sequencing rules out, reported instead of a panic.
///
/// No valid use produces one: each variant names the part of the parser whose
/// contract was found broken, so a report can say which part to read. The parser
/// latches the stop first, hands back every owner it holds and releases all
/// charged usage. The enum grows as more states are classified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlParseInvariant {
    /// A source other than the one bound to the parse was presented.
    ForeignSource,
    /// A storage accounting operation named a resource it does not apply to:
    /// one that is not storage, or not the one the operation accounts.
    NonStorageResource,
    /// A release returned more units than the parse currently holds.
    ReleaseUnderflow,
    /// A node release was requested for nodes, which are never reclaimed one by
    /// one.
    NonReclaimableNodes,
    /// A buffer, ticket or allocation differs from what was charged for it: its
    /// capacity, layout, ownership or finalization state, or a byte extent that
    /// no charged allocation could have. A work plan that counts less than was
    /// already charged for it is the same disagreement.
    CapacityMismatch,
    /// Storage was requested for a zero-sized element type.
    ZeroSizedStorage,
    /// A stopped parse has no source binding where one was admitted.
    MissingSource,
    /// The input cursor (decoded input, position or checkpoint) is in a state
    /// its own contract excludes.
    InputCursorState,
    /// A value, ticket or capacity owned under one parse controller was
    /// presented to another.
    ForeignControl,
    /// A record identifier or evidence handle from one evidence owner was
    /// presented to another.
    ForeignEvidence,
    /// A cleanup-only operation ran while the parse had not stopped.
    CleanupWhileRunning,
    /// A shared allocation that must be uniquely held was aliased.
    SharedAllocationAliased,
    /// A construction session was used, or a frozen base was validated, while
    /// an operation it owes was still pending: a form, radio, range, transfer,
    /// SVG-wrapper or task effect, an open checkpoint, an intrinsic operation
    /// not yet performed, or a base response not yet returned.
    PendingFormEffects,
    /// A creation that holds a form, link, select or details association
    /// returned the outcome of script, SVG-wrapper or completion work, which
    /// none of its steps runs, so the association it registered stays unbound.
    /// Outcomes of the creation's own image, associated-fact, nonce and context
    /// work keep the classification the driver gives them, and a creation that
    /// fails for a fault of its own reports what that fault violates instead: a
    /// missing row, an aliased allocation, a pending operation.
    IncompleteFormAssociationCreation,
    /// A construction registry row or retained step is not in the state that
    /// internal sequencing leaves it, in one of these ways:
    ///
    /// - a row, record, child or payload that internal state names is missing,
    ///   out of range or of another kind, or a node handle does not resolve;
    /// - a join or binding between rows and records cannot be made or
    ///   disagrees, or a new row cannot be registered;
    /// - a position, count or text extent is outside what it indexes, or
    ///   registry arithmetic over one overflows;
    /// - an owner, lease, pool, ticket, attribute edit, pending operation,
    ///   terminal slot or finalization state that is kept in a slot is absent,
    ///   or is occupied when the step needs it free;
    /// - a retained step lacks the value its stage consumes: the form-owner
    ///   reset's owner or stage, the stage of an attribute bank, a text-run
    ///   scan's run kind, a recognition cursor's result, or an image-source
    ///   guard.
    ///
    /// The range-input machine, HTML's construction owner refusing a request
    /// or answering unexpectedly, and a work plan that disagrees with what was
    /// charged are reported separately.
    RegistryRow,
    /// The range-input machine is in a state its own protocol excludes: a
    /// staged scratch or output buffer is missing, mis-sized or not where the
    /// step expects it, the machine or its numeric request is not the one the
    /// step drives, or the number grammar or constraint policy returned a result
    /// that the parser's well-formed requests cannot produce.
    RangeMachineState,
    /// HTML's construction owner refused a request that the parser's own
    /// sequencing guarantees valid: a record of the wrong kind, a placement the
    /// tree cannot take (a root or association move, a parent that cannot hold
    /// children, a cycle), a shadow, template or contents association that is
    /// already held or malformed, an attribute that is duplicated or out of
    /// order, an invalid output root, or an operation that was already
    /// consumed or is not ready. It is also reported when the owner answers a
    /// step with an action or observation the parser's sequencing does not
    /// expect, such as an action of another kind or an observation without the
    /// allocation, capacity or backing the step was planned around. The
    /// owner's error enum is open, so an error this parser does not yet
    /// classify reports the same fault.
    ConstructionOwnerRefusal,
    /// The tree-builder drivers or algorithm are in a state their sequencing
    /// excludes: a driver, stage or prologue owner that the sequencing keeps in
    /// a slot is gone, the formatting list is invalid, script nesting or the
    /// return mode is wrong, the boundary or before-head sequence is broken,
    /// the tokenizer is exhausted early, or a token slot holds another kind of
    /// event.
    TreeBuilderState,
    /// The tokenizer is in a state its transitions exclude: a draft tag,
    /// attribute, comment, doctype or processing-instruction part that must
    /// exist is missing, its event queue is full, a source-boundary proof is
    /// absent, a character-reference table lookup is out of range, or the
    /// lexical evidence it records is not in the state its sequencing requires.
    TokenizerState,
    /// CSS colour processing is in a state its own sequencing excludes: a
    /// digit or conversion bank is missing or out of range, a read ran past
    /// its input, or the colour adapter reported malformed input that the
    /// parser builds well formed.
    ColorConversionState,
    /// A Meta CSP processor, or the CSS color-scheme parser, called an
    /// admission or release callback while another callback of the same call
    /// was still running.
    ReentrantProviderCallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseStop {
    Cancelled,
    CspProcessing,
    CspProtocol(surgeist_html::security::csp::HtmlCspProtocolError),
    LimitExceeded {
        resource: HtmlParseResource,
        limit: u64,
        current: u64,
        requested_delta: u64,
    },
    RepresentationExhausted {
        resource: HtmlParseResource,
    },
    AllocationFailed {
        resource: HtmlParseResource,
        layout_bytes: u64,
    },
    InternalInvariant(HtmlParseInvariant),
    UnsupportedStandaloneState(HtmlParseUnsupportedState),
    InvalidAssociatedFacts,
    /// The ledger the parse charges failed in a way that is not a refusal of a
    /// charged change, or the supplied accounting cannot serve the parse.
    Accounting(HtmlAccountingFault),
    /// The accounting the request supplied already serves another parse.
    AccountingInUse,
    /// Reached CSS processing/fact failure, distinct from invalid CSS syntax.
    ColorProcessing(HtmlColorProcessingError),
    /// A finite CSS property owner refused its actual reached resource.
    CssPropertyLimit {
        resource: surgeist_css::source::CssUtf16Resource,
    },
    /// A CSS-owned allocation failed; its private layout is not fabricated here.
    CssPropertyAllocationFailed,
    BaseFacts(crate::HtmlBaseMissing),
}
/// Processing error from HTML's source-only CSS color adapter. Colour
/// numerics are total and CSS protocol misuse is static or a panic, so neither
/// a numerical nor a protocol failure exists. The parser's original source
/// binding remains separately owned by ParseControl.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlColorProcessingError {
    Unavailable(surgeist_css::color::CssColorUnavailable),
    Unsupported(surgeist_css::color::CssColorUnsupported),
    Facts(surgeist_css::color::CssColorFactDefect),
}
impl std::fmt::Display for HtmlColorProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(fact) => write!(f, "reached color fact is unavailable: {fact:?}"),
            Self::Unsupported(capability) => {
                write!(f, "reached color capability is unsupported: {capability:?}")
            }
            Self::Facts(error) => write!(f, "invalid color fact envelope: {error:?}"),
        }
    }
}
impl std::error::Error for HtmlColorProcessingError {}
impl HtmlColorProcessingError {
    /// The source operation's error, unchanged in kind. The adapter's UTF-16
    /// input admits every unit, so a malformed-input report has no
    /// processing kind; the caller records it as an adapter invariant.
    pub(crate) fn from_source(error: surgeist_css::color::CssColorError) -> Option<Self> {
        if let Some(fact) = error.unavailable() {
            return Some(Self::Unavailable(fact));
        }
        if let Some(capability) = error.unsupported() {
            return Some(Self::Unsupported(capability));
        }
        // Otherwise a fact defect or a malformed-input report.
        error.fact_defect().map(Self::Facts)
    }
}
/// Actual selected closed-environment execution boundaries, not limit errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseUnsupportedState {
    Provenance,
    ScriptDocument,
    ScriptPause,
    PendingScript,
    InvalidNesting,
    Completion,
    DetailsNotification,
    ImageResource,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlParseResourceUsage {
    pub(crate) current: u64,
    pub(crate) high_water: u64,
}

/// The parse's usage as the ledger holds it: each resource's live value and
/// peak, each work category's total, and the credit total.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlParseUsage {
    resources: ResourceTable<HtmlParseResourceUsage>,
    work: WorkTable<u64>,
    /// Actual capacities can exceed requested layouts. This is observed after
    /// allocation; no promise is made about preventing allocator overshoot.
    pub(crate) actual_capacity_bytes: u64,
    pub(crate) actual_capacity_high_water: u64,
    /// Largest indivisible reserve/copy submission (not a time bound).
    pub(crate) largest_atomic_layout: u64,
}
impl HtmlParseUsage {
    /// A usage that records nothing.
    fn empty() -> Self {
        Self {
            resources: ResourceTable::filled(HtmlParseResourceUsage::default()),
            work: WorkTable::filled(0),
            actual_capacity_bytes: 0,
            actual_capacity_high_water: 0,
            largest_atomic_layout: 0,
        }
    }
    /// Reads the usage of a charging scope and the credit total of its cap
    /// scope.
    fn from_ledger(usage: &LedgerUsage, credits: u64) -> Self {
        let mut view = Self::empty();
        for resource in RESOURCES {
            let reading = match resource.dimension() {
                Some(dimension) => HtmlParseResourceUsage {
                    current: usage.live(dimension),
                    high_water: usage.peak(dimension),
                },
                None => HtmlParseResourceUsage {
                    current: credits,
                    high_water: credits,
                },
            };
            *view.resources.get_mut(resource) = reading;
        }
        for category in HtmlParseWorkCategory::ALL {
            *view.work.get_mut(category) = usage.work(category.work());
        }
        view.actual_capacity_bytes = usage.live(HtmlResource::ManagedCapacityBytes);
        view.actual_capacity_high_water = usage.peak(HtmlResource::ManagedCapacityBytes);
        view.largest_atomic_layout = usage.peak(HtmlResource::AtomicLayoutBytes);
        view
    }
    pub fn resource(&self, resource: HtmlParseResource) -> HtmlParseResourceUsage {
        self.resources.copy(resource)
    }
    pub fn work(&self, category: HtmlParseWorkCategory) -> u64 {
        self.work.copy(category)
    }
}

/// The first latched stop and the usage observed at that moment.
struct FirstStop {
    reason: HtmlParseStop,
    usage: HtmlParseUsage,
}

/// How many parents' sibling hints a controller keeps.
const SIBLING_HINTS: usize = 16;
/// Exclusively mutable owner. Only the cancellation handle can be cloned.
pub(crate) struct ParseControl {
    /// The ledger operation and scopes every charge goes through. It is absent
    /// only when the parse could not set them up; the control is then stopped
    /// from its creation with the fault.
    accounts: Option<Accounts>,
    /// The caps the accounts were built with, for tests that lower one.
    #[cfg(test)]
    configured: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
    // The first stop and its fixed usage observation, captured together before
    // any producer cleanup, so neither can exist without the other.
    stopped: Option<FirstStop>,
    source: Option<(HtmlSourceId, HtmlSourceCoordinates)>,
    identity: Arc<()>,
    /// Where recent searches for a node among its siblings found it; see
    /// [`Self::sibling_hint`].
    sibling_hints: [Option<(usize, usize)>; SIBLING_HINTS],
    #[cfg(test)]
    owned_spans: OwnedSpanObservation,
    #[cfg(test)]
    finalization_cut: Option<finalization::CutRequest>,
    #[cfg(test)]
    finalization_observation: Option<FinalizationObservation>,
    #[cfg(test)]
    finalization_transfer_cut: Option<finalization::TransferCut>,
    #[cfg(test)]
    finalization_transfer_observation: Option<FinalizationTransferObservation>,
    #[cfg(test)]
    css_scheme_fault: Option<css_value::SchemeFault>,
    /// Refuses the next `release_shared` as if its payload were aliased.
    #[cfg(test)]
    shared_release_fault: bool,
    /// Makes the next construction-owner retirement report a refusal after it
    /// has retired everything the owner holds.
    #[cfg(test)]
    construction_retire_fault: bool,
    /// Makes the next construction-owner recovery report a refusal after it
    /// has taken the storage back and retired the publication wrapper.
    #[cfg(test)]
    construction_recovery_fault: bool,
    /// Refuses the next text-append ticket restoration before it changes
    /// anything, as an impossible precondition would.
    #[cfg(test)]
    text_append_restore_fault: bool,
    /// Fails the next text-append backing initialization after its pending
    /// ticket is prepared, as an allocation refusal would.
    #[cfg(test)]
    text_append_backing_fault: bool,
    /// Remaining forced refusals per cleanup-walk site, and how many have
    /// been issued so far.
    #[cfg(test)]
    walk_refusals: [usize; 3],
    #[cfg(test)]
    walk_refusals_issued: usize,
    /// Extra steps the next registry walk takes past the registry's end, as a
    /// registry that shrank during the walk would.
    #[cfg(test)]
    registry_walk_overshoot: usize,
    /// Extra units the next space-separated token reaches past its value, as
    /// a cursor out of step with its value would.
    #[cfg(test)]
    space_token_overshoot: usize,
    /// Set when the stop requested by [`stop_after_prepared_slot`] is due: the
    /// next checkpoint cancels the parse.
    #[cfg(test)]
    stop_armed: bool,
    /// Bytes the next recorded capacity exceeds the observed one by, as a
    /// provider that allocates more than it was asked to would.
    #[cfg(test)]
    capacity_overshoot: u64,
    /// Runs once after the next retention check, as a parse sharing a host
    /// scope would between the check and the acquisition.
    #[cfg(test)]
    after_retention_check: Option<Box<dyn FnOnce() + Send>>,
}

/// Which prepared payload slot a test stops the parse after, and how many the
/// parse has prepared so far. Held per thread, which is where a parse runs.
#[cfg(test)]
#[derive(Clone, Copy)]
struct PreparedSlotStop {
    after: Option<usize>,
    prepared: usize,
}
#[cfg(test)]
thread_local! {
    static PREPARED_SLOT_STOP: std::cell::Cell<PreparedSlotStop> =
        const { std::cell::Cell::new(PreparedSlotStop { after: None, prepared: 0 }) };
}
// Where each payload slot the parse prepared was asked for, in order.
#[cfg(test)]
thread_local! {
    static PREPARED_SLOT_SITES: std::cell::RefCell<Vec<&'static std::panic::Location<'static>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}
/// Makes the parse that runs next on this thread stop, as a cancellation, at the
/// first checkpoint after it prepares its `index`th payload slot (counting
/// from zero), and restarts the count [`prepared_slots`] reports. `None` stops
/// nothing.
#[cfg(test)]
pub(crate) fn stop_after_prepared_slot(index: Option<usize>) {
    PREPARED_SLOT_STOP.set(PreparedSlotStop {
        after: index,
        prepared: 0,
    });
    PREPARED_SLOT_SITES.with_borrow_mut(Vec::clear);
}
/// The source location that asked for each payload slot the parse run since
/// [`stop_after_prepared_slot`] prepared, in order.
#[cfg(test)]
pub(crate) fn prepared_slot_sites() -> Vec<&'static std::panic::Location<'static>> {
    PREPARED_SLOT_SITES.with_borrow(Clone::clone)
}
/// How many payload slots the parse run since [`stop_after_prepared_slot`]
/// prepared.
#[cfg(test)]
pub(crate) fn prepared_slots() -> usize {
    PREPARED_SLOT_STOP.get().prepared
}
/// Clears the stop [`stop_after_prepared_slot`] set when it is dropped, so a
/// test that panics before it does can leave nothing armed for the next test
/// that runs on its thread.
#[cfg(test)]
pub(crate) struct PreparedSlotStopScope;
#[cfg(test)]
impl PreparedSlotStopScope {
    pub(crate) fn begin() -> Self {
        stop_after_prepared_slot(None);
        Self
    }
}
#[cfg(test)]
impl Drop for PreparedSlotStopScope {
    fn drop(&mut self) {
        stop_after_prepared_slot(None);
    }
}

/// A cleanup walk that keeps going after a refusal, so tests can make one of
/// its steps refuse.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum WalkSite {
    /// The credit of each bank the construction owner retires.
    ConstructionBank,
    /// The retirement of each name witness of an attribute edit.
    AttributeWitness,
    /// Each owner a frozen arena releases.
    FrozenOwner,
}

/// Fixed-size diagnostic metadata for actual execution between real polls.
/// It neither admits work/storage nor participates in parser decisions. Only
/// the observed append and registry operations are instrumented here; this is
/// not an observation of every parser instruction or provider implementation.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OwnedSpanObservation {
    pub(crate) polls: u64,
    pub(crate) append_calls: u64,
    pub(crate) append_slots: usize,
    pub(crate) largest_append_copy_bytes: usize,
    pub(crate) registry_iterations: usize,
    pub(crate) largest_registry_iterations_between_polls: usize,
    pub(crate) largest_registry_credits_between_polls: usize,
    registry_iterations_since_poll: usize,
    registry_credits_since_poll: usize,
}

#[derive(Clone)]
pub(crate) struct ControlIdentity(Arc<()>);

/// Bytes inspected for one stored UTF-16 code unit.
const UNIT_BYTES: u64 = 2;
/// Bytes inspected comparing a stored UTF-16 code unit with a literal ASCII byte:
/// the unit's two bytes and the literal's one.
const UNIT_LITERAL_PAIR_BYTES: u64 = 3;
/// Bytes inspected comparing one byte of each of two byte sequences.
const BYTE_PAIR_BYTES: u64 = 2;

impl ParseControl {
    /// A control with its own ledger operation.
    #[cfg(test)]
    pub(crate) fn new(limits: HtmlParseLimits, cancellation: HtmlParseCancellation) -> Self {
        Self::with_accounting(limits, cancellation, None)
    }
    /// A control that charges the operation of `lease`, or its own operation
    /// when there is none. A setup fault latches as the first stop.
    pub(crate) fn with_accounting(
        limits: HtmlParseLimits,
        cancellation: HtmlParseCancellation,
        lease: Option<&Arc<Lease>>,
    ) -> Self {
        let (accounts, stopped) = match Accounts::new(&limits, lease) {
            Ok(accounts) => (Some(accounts), None),
            Err(fault) => (
                None,
                Some(FirstStop {
                    reason: fault.stop(),
                    usage: HtmlParseUsage::empty(),
                }),
            ),
        };
        let mut control = Self {
            accounts,
            #[cfg(test)]
            configured: limits,
            cancellation,
            stopped,
            source: None,
            identity: Arc::new(()),
            sibling_hints: [None; SIBLING_HINTS],
            #[cfg(test)]
            owned_spans: OwnedSpanObservation::default(),
            #[cfg(test)]
            finalization_cut: None,
            #[cfg(test)]
            finalization_observation: None,
            #[cfg(test)]
            finalization_transfer_cut: None,
            #[cfg(test)]
            finalization_transfer_observation: None,
            #[cfg(test)]
            css_scheme_fault: None,
            #[cfg(test)]
            shared_release_fault: false,
            #[cfg(test)]
            construction_retire_fault: false,
            #[cfg(test)]
            construction_recovery_fault: false,
            #[cfg(test)]
            text_append_restore_fault: false,
            #[cfg(test)]
            text_append_backing_fault: false,
            #[cfg(test)]
            walk_refusals: [0; 3],
            #[cfg(test)]
            walk_refusals_issued: 0,
            #[cfg(test)]
            registry_walk_overshoot: 0,
            #[cfg(test)]
            space_token_overshoot: 0,
            #[cfg(test)]
            stop_armed: false,
            #[cfg(test)]
            capacity_overshoot: 0,
            #[cfg(test)]
            after_retention_check: None,
        };
        control.enter_phase(ParsePhase::Admission);
        control
    }
    /// Runs `hook` once, right after the next retention check.
    #[cfg(test)]
    pub(crate) fn after_next_retention_check(&mut self, hook: impl FnOnce() + Send + 'static) {
        self.after_retention_check = Some(Box::new(hook));
    }
    /// Makes the next capacity a provider reports exceed the observed one by
    /// `extra` bytes.
    #[cfg(test)]
    pub(crate) fn overshoot_next_capacity_record(&mut self, extra: u64) {
        self.capacity_overshoot = extra;
    }
    /// Counts one prepared payload slot, records where it was asked for, and arms
    /// the stop a test asked for.
    #[cfg(test)]
    pub(crate) fn note_prepared_slot(&mut self, site: &'static std::panic::Location<'static>) {
        PREPARED_SLOT_SITES.with_borrow_mut(|sites| sites.push(site));
        let mut hook = PREPARED_SLOT_STOP.get();
        self.stop_armed |= hook.after == Some(hook.prepared);
        hook.prepared = hook.prepared.saturating_add(1);
        PREPARED_SLOT_STOP.set(hook);
    }
    /// The usage the ledger holds. A ledger that cannot be read has found a
    /// defect of its own, which the next admission reports as the stop; the
    /// usage then reads as empty.
    pub(crate) fn usage(&self) -> HtmlParseUsage {
        self.accounts
            .as_ref()
            .and_then(|accounts| accounts.usage().ok())
            .map_or_else(HtmlParseUsage::empty, |(usage, credits)| {
                HtmlParseUsage::from_ledger(&usage, credits)
            })
    }
    pub(crate) fn identity(&self) -> ControlIdentity {
        ControlIdentity(self.identity.clone())
    }
    pub(crate) fn require_identity(
        &mut self,
        identity: &ControlIdentity,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        if Arc::ptr_eq(&identity.0, &self.identity) {
            Ok(())
        } else {
            self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ))
        }
    }
    pub(crate) fn stop(&self) -> Option<HtmlParseStop> {
        self.stopped.as_ref().map(|first| first.reason)
    }
    pub(crate) fn first_stop_usage(&self) -> Option<&HtmlParseUsage> {
        self.stopped.as_ref().map(|first| &first.usage)
    }
    /// The first stop with the usage observed when it was latched; one test
    /// answers both, since they exist together or not at all.
    pub(crate) fn stopped(&self) -> Option<(HtmlParseStop, &HtmlParseUsage)> {
        self.stopped
            .as_ref()
            .map(|first| (first.reason, &first.usage))
    }
    /// Fixed identity readback, including after stop. The actual entry copies
    /// the fresh immutable request's capability into this sole controller.
    pub(crate) fn owns_request(&self, cancellation: &HtmlParseCancellation) -> bool {
        self.cancellation.same_handle(cancellation)
    }
    /// Fixed stopped-prefix observation. None means source admission never
    /// installed a binding; it is not an invented zero-length source.
    pub(crate) fn stopped_source_binding(
        &self,
    ) -> Result<Option<&(HtmlSourceId, HtmlSourceCoordinates)>, HtmlParseStop> {
        if self.stopped.is_none() {
            return Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CleanupWhileRunning,
            ));
        }
        Ok(self.source.as_ref())
    }
    /// Cleanup-only move of the actual optional binding, including a prefix
    /// stopped before source admission. The caller preflights its owning slots.
    pub(crate) fn take_stopped_optional_source_binding(
        &mut self,
    ) -> Result<Option<(HtmlSourceId, HtmlSourceCoordinates)>, HtmlParseStop> {
        self.stopped_source_binding()?;
        Ok(self.source.take())
    }
    /// A stopped, cleaned consumer moves the already-admitted source binding.
    /// Refusal leaves it intact; no polling, charging or handle clone follows
    /// stop. The caller retains this controller for diagnostic ticket disposal.
    pub(crate) fn take_stopped_source_binding(
        &mut self,
    ) -> Result<(HtmlSourceId, HtmlSourceCoordinates), HtmlParseStop> {
        if self.stopped.is_none() {
            return Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CleanupWhileRunning,
            ));
        }
        self.source.take().ok_or(HtmlParseStop::InternalInvariant(
            HtmlParseInvariant::MissingSource,
        ))
    }
    pub(crate) fn csp_processing_stop(&mut self) -> HtmlParseStop {
        self.latch_stop(HtmlParseStop::CspProcessing)
    }
    pub(crate) fn csp_protocol_stop(
        &mut self,
        error: surgeist_html::security::csp::HtmlCspProtocolError,
    ) -> HtmlParseStop {
        self.latch_stop(HtmlParseStop::CspProtocol(error))
    }
    pub(crate) fn cancelled_stop(&mut self) -> HtmlParseStop {
        self.latch_stop(HtmlParseStop::Cancelled)
    }
    fn latch_stop(&mut self, reason: HtmlParseStop) -> HtmlParseStop {
        if let Some(first) = &self.stopped {
            return first.reason;
        }
        // No allocation or useful progression: the fixed control layout owns
        // this observation. Denied requests remain in the stop, not in usage.
        // A capacity record the ledger refused is in the stop too: the usage
        // shows the custody the ledger admitted, without the refused overshoot.
        self.stopped = Some(FirstStop {
            reason,
            usage: self.usage(),
        });
        reason
    }
    pub(crate) fn fail_invariant(&mut self, invariant: HtmlParseInvariant) -> HtmlParseStop {
        if let Err(reason) = self.checkpoint() {
            return reason;
        }
        self.latch_stop(HtmlParseStop::InternalInvariant(invariant))
    }
    /// Latches the stop for a construction outcome that no producer latched, as
    /// the producers of the outcomes that have a stop of their own do.
    pub(crate) fn stop_outcome(&mut self, stop: HtmlParseStop) -> HtmlParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(stop)
    }
    pub(crate) fn stop_unsupported_standalone(
        &mut self,
        reason: HtmlParseUnsupportedState,
    ) -> HtmlParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(HtmlParseStop::UnsupportedStandaloneState(reason))
    }
    pub(crate) fn stop_color_processing(
        &mut self,
        reason: surgeist_css::color::CssColorError,
    ) -> HtmlParseStop {
        let Some(error) = HtmlColorProcessingError::from_source(reason) else {
            return self.fail_invariant(HtmlParseInvariant::ColorConversionState);
        };
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(HtmlParseStop::ColorProcessing(error))
    }
    pub(crate) fn stop_invalid_associated_facts(&mut self) -> HtmlParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(HtmlParseStop::InvalidAssociatedFacts)
    }
    pub(crate) fn stop_base_facts(&mut self, reason: crate::HtmlBaseMissing) -> HtmlParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(HtmlParseStop::BaseFacts(reason))
    }
    fn fail<T>(&mut self, reason: HtmlParseStop) -> Result<T, HtmlParseStop> {
        Err(self.latch_stop(reason))
    }
    pub(crate) fn checkpoint(&mut self) -> Result<(), HtmlParseStop> {
        if let Some(first) = &self.stopped {
            return Err(first.reason);
        }
        #[cfg(test)]
        {
            self.owned_spans.polls = self.owned_spans.polls.saturating_add(1);
            self.owned_spans.registry_iterations_since_poll = 0;
            self.owned_spans.registry_credits_since_poll = 0;
        }
        if self.cancellation.is_cancelled() {
            return self.fail(HtmlParseStop::Cancelled);
        }
        #[cfg(test)]
        if std::mem::take(&mut self.stop_armed) {
            return self.fail(HtmlParseStop::Cancelled);
        }
        Ok(())
    }
    /// Forces the next `release_shared` to refuse, returning the payload and
    /// its ticket unchanged, as an aliased payload would.
    #[cfg(test)]
    pub(crate) fn refuse_next_shared_release(&mut self) {
        self.shared_release_fault = true;
    }
    /// Forces the next construction-owner retirement to report a refusal once
    /// its releases are done.
    #[cfg(test)]
    pub(crate) fn refuse_next_construction_retire(&mut self) {
        self.construction_retire_fault = true;
    }
    #[cfg(test)]
    pub(crate) fn take_construction_retire_fault(&mut self) -> bool {
        std::mem::take(&mut self.construction_retire_fault)
    }
    /// Forces the next construction-owner recovery to report a refusal once
    /// the storage is back and the wrapper is retired.
    #[cfg(test)]
    pub(crate) fn refuse_next_construction_recovery(&mut self) {
        self.construction_recovery_fault = true;
    }
    #[cfg(test)]
    pub(crate) fn take_construction_recovery_fault(&mut self) -> bool {
        std::mem::take(&mut self.construction_recovery_fault)
    }
    /// Forces the next text-append ticket restoration to refuse.
    #[cfg(test)]
    pub(crate) fn refuse_next_text_append_restore(&mut self) {
        self.text_append_restore_fault = true;
    }
    #[cfg(test)]
    pub(crate) fn take_text_append_restore_fault(&mut self) -> bool {
        std::mem::take(&mut self.text_append_restore_fault)
    }
    /// Forces the next text-append backing initialization to fail once its
    /// pending ticket has been prepared.
    #[cfg(test)]
    pub(crate) fn fail_next_text_append_backing(&mut self) {
        self.text_append_backing_fault = true;
    }
    /// Makes the next `count` steps of a cleanup walk refuse once their work
    /// is done. The first refusal issued is reported as a wrong token, the
    /// later ones as the registry row, so the order of reporting is visible.
    #[cfg(test)]
    pub(crate) fn refuse_walk_steps(&mut self, site: WalkSite, count: usize) {
        self.walk_refusals[site as usize] = count;
        self.walk_refusals_issued = 0;
    }
    /// The position of the refusal to issue at this step, when one is due.
    #[cfg(test)]
    pub(crate) fn take_walk_refusal(&mut self, site: WalkSite) -> Option<usize> {
        let remaining = &mut self.walk_refusals[site as usize];
        *remaining = remaining.checked_sub(1)?;
        let issued = self.walk_refusals_issued;
        self.walk_refusals_issued = issued.saturating_add(1);
        Some(issued)
    }
    #[cfg(test)]
    pub(crate) fn overshoot_next_registry_walk(&mut self, steps: usize) {
        self.registry_walk_overshoot = steps;
    }
    #[cfg(test)]
    pub(crate) fn take_registry_walk_overshoot(&mut self) -> usize {
        std::mem::take(&mut self.registry_walk_overshoot)
    }
    #[cfg(test)]
    pub(crate) fn overshoot_next_space_token(&mut self, units: usize) {
        self.space_token_overshoot = units;
    }
    #[cfg(test)]
    pub(crate) fn take_space_token_overshoot(&mut self) -> usize {
        std::mem::take(&mut self.space_token_overshoot)
    }
    #[cfg(test)]
    pub(crate) fn take_text_append_backing_fault(&mut self) -> bool {
        std::mem::take(&mut self.text_append_backing_fault)
    }
    #[cfg(test)]
    pub(crate) fn clear_owned_span_observation(&mut self) {
        self.owned_spans = OwnedSpanObservation::default();
    }
    #[cfg(test)]
    pub(crate) fn owned_span_observation(&self) -> OwnedSpanObservation {
        self.owned_spans
    }
    #[cfg(test)]
    fn observe_attribute_append(&mut self, slots: usize) {
        // The actual fixed pop/push transfers each slot twice. Observe its
        // performed copy span, not an unrelated prepaid whole-vector total.
        let bytes = slots.saturating_mul(2 * size_of::<surgeist_html::values::Attribute>());
        self.owned_spans.append_calls = self.owned_spans.append_calls.saturating_add(1);
        self.owned_spans.append_slots = self.owned_spans.append_slots.saturating_add(slots);
        self.owned_spans.largest_append_copy_bytes =
            self.owned_spans.largest_append_copy_bytes.max(bytes);
    }
    #[cfg(test)]
    pub(crate) fn observe_registry_iteration(&mut self) {
        let observed = &mut self.owned_spans;
        observed.registry_iterations = observed.registry_iterations.saturating_add(1);
        observed.registry_iterations_since_poll =
            observed.registry_iterations_since_poll.saturating_add(1);
        observed.largest_registry_iterations_between_polls = observed
            .largest_registry_iterations_between_polls
            .max(observed.registry_iterations_since_poll);
        self.observe_registry_credits(1);
    }
    #[cfg(test)]
    pub(crate) fn observe_registry_location_write(&mut self, bytes: usize) {
        // Actual read and write of one location, plus its record-write event.
        self.observe_registry_credits(bytes.saturating_mul(2).saturating_add(1));
    }
    #[cfg(test)]
    fn observe_registry_credits(&mut self, credits: usize) {
        let observed = &mut self.owned_spans;
        observed.registry_credits_since_poll =
            observed.registry_credits_since_poll.saturating_add(credits);
        observed.largest_registry_credits_between_polls = observed
            .largest_registry_credits_between_polls
            .max(observed.registry_credits_since_poll);
    }
    /// Latches a checked-arithmetic failure observed after the caller's initial
    /// checkpoint. No extra poll can replace that already observed first stop.
    pub(crate) fn stop_representation(&mut self, resource: HtmlParseResource) -> HtmlParseStop {
        self.latch_stop(HtmlParseStop::RepresentationExhausted { resource })
    }
    /// The same latched stop as [`Self::stop_representation`], for a caller
    /// whose own result type is `Result`.
    pub(crate) fn representation<T>(
        &mut self,
        resource: HtmlParseResource,
    ) -> Result<T, HtmlParseStop> {
        Err(self.stop_representation(resource))
    }
    /// Admits one known indivisible owned-record layout before its move/copy.
    /// The caller separately precharges actual bytes and record writes. This
    /// observes layout only; it creates no allocation or cumulative work credit.
    pub(crate) fn admit_atomic_layout(&mut self, bytes: usize) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        if bytes > isize::MAX as usize {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        }
        let Ok(bytes) = u64::try_from(bytes) else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.record_layout(bytes)
    }
    /// Records `bytes` as the latest indivisible layout; the largest one reached
    /// is its peak.
    pub(super) fn record_layout(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        let layout = self.gauge_changes(HtmlResource::AtomicLayoutBytes, bytes)?;
        self.admit_resources(&layout, Site::PLAIN)
    }
    pub(crate) fn admit_source(&mut self, source: &HtmlSource) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        self.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some((identity, coordinates)) = &self.source {
            return if identity == source.identity() && *coordinates == source.coordinates() {
                Ok(())
            } else {
                self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ForeignSource,
                ))
            };
        }
        let Ok(units) = u64::try_from(source.len()) else {
            return self.representation(HtmlParseResource::SourceUnits);
        };
        self.admit_resources(
            &[Change::Acquire(HtmlResource::SourceUnits, units)],
            Site::PLAIN,
        )?;
        self.source = Some((source.identity().clone(), source.coordinates()));
        Ok(())
    }
    pub(crate) fn require_source(&mut self, source: &HtmlSource) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        match &self.source {
            Some((identity, coordinates))
                if identity == source.identity() && *coordinates == source.coordinates() =>
            {
                Ok(())
            }
            Some(_) => self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignSource,
            )),
            None => self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::MissingSource,
            )),
        }
    }
    pub(crate) fn require_source_range(
        &mut self,
        range: &crate::HtmlSourceRange,
    ) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (HtmlParseWorkCategory::RecordVisit, 1),
            (HtmlParseWorkCategory::ProviderCall, 1),
        ])?;
        match &self.source {
            Some((identity, coordinates))
                if identity == range.source() && *coordinates == range.coordinates() =>
            {
                Ok(())
            }
            Some(_) => self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignSource,
            )),
            None => self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::MissingSource,
            )),
        }
    }
    /// The ordinal at which a walk last reached a child of `parent`, if the
    /// table still holds it. A hint only says where to look first: the caller
    /// checks that the child it wants is there, so a stale hint costs a search
    /// and cannot change an answer.
    pub(crate) fn sibling_hint(&self, parent: usize) -> Option<usize> {
        match self.sibling_hints.get(parent % SIBLING_HINTS) {
            Some(Some((hinted, ordinal))) if *hinted == parent => Some(*ordinal),
            _ => None,
        }
    }
    /// Remembers that a walk reached the child of `parent` at `ordinal`.
    pub(crate) fn note_sibling(&mut self, parent: usize, ordinal: usize) {
        if let Some(slot) = self.sibling_hints.get_mut(parent % SIBLING_HINTS) {
            *slot = Some((parent, ordinal));
        }
    }
    pub(crate) fn charge(
        &mut self,
        category: HtmlParseWorkCategory,
        units: u64,
    ) -> Result<(), HtmlParseStop> {
        self.charge_many(&[(category, units)])
    }
    /// Charges the inspection of one stored UTF-16 code unit: its two bytes.
    pub(crate) fn charge_unit_inspection(&mut self) -> Result<(), HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::InspectByte, UNIT_BYTES)
    }
    /// Charges comparing one stored UTF-16 code unit with one byte of an ASCII
    /// literal: the two bytes of the unit and the one byte of the literal.
    pub(crate) fn charge_unit_literal_comparison(&mut self) -> Result<(), HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::InspectByte, UNIT_LITERAL_PAIR_BYTES)
    }
    /// Compares a stored string with an ASCII literal as an ASCII case-insensitive match, charging
    /// one record visit and then one literal comparison per paired unit, exactly as far as the
    /// comparison proceeds. A length mismatch is decided after the record visit alone.
    pub(crate) fn ascii_case_insensitive_literal_matches(
        &mut self,
        value: &surgeist_html::values::DocumentString,
        literal: &str,
    ) -> Result<bool, HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if value.len() != literal.len() {
            return Ok(false);
        }
        // The lengths are equal, so the pairs are exactly the literal's indices.
        for (&unit, byte) in value.code_units().iter().zip(literal.bytes()) {
            self.charge_unit_literal_comparison()?;
            if !surgeist_html::bounded::lexical::ascii_case_insensitive_unit(unit, byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// Charges comparing one byte of each of two byte sequences.
    pub(crate) fn charge_byte_pair_comparison(&mut self) -> Result<(), HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::InspectByte, BYTE_PAIR_BYTES)
    }
    fn charge_many<const N: usize>(
        &mut self,
        charges: &[(HtmlParseWorkCategory, u64); N],
    ) -> Result<(), HtmlParseStop> {
        let changes: [HtmlChange; N] = charges.map(|(category, units)| category.change(units));
        self.charge_changes(&changes)
    }
    /// Charges work changes, which name schema work dimensions, as one batch.
    pub(crate) fn charge_changes(&mut self, changes: &[HtmlChange]) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        self.admit_work(changes)
    }
    /// Acquires one resource change of a conversion.
    pub(crate) fn acquire_change(&mut self, change: HtmlChange) -> Result<(), HtmlParseStop> {
        let Change::Acquire(dimension, units) = change else {
            return Err(self.unexpected_change());
        };
        let Some(resource) = HtmlParseResource::from_dimension(dimension) else {
            return Err(self.unexpected_change());
        };
        self.acquire(resource, units)
    }
    /// Owned scans/loops must execute only the returned span before calling again.
    /// Allocations are separately precharged indivisible provider submissions.
    #[cfg(test)]
    pub(crate) fn work_batch(
        &mut self,
        category: HtmlParseWorkCategory,
        remaining: usize,
    ) -> Result<usize, HtmlParseStop> {
        let batch = remaining.min(WORK_QUANTUM);
        self.charge(category, batch as u64)?;
        Ok(batch)
    }
    /// The acquisitions of `requests` as one batch in resource precedence order,
    /// one change per resource with the sum of its requests. Zero sums are left
    /// out: they change nothing.
    fn storage_batch(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<([HtmlChange; STORAGE_RESOURCES], usize), HtmlParseStop> {
        self.checkpoint()?;
        let mut deltas = ResourceTable::filled(0_u64);
        for &(resource, delta) in requests {
            if !resource.storage() {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::NonStorageResource,
                ));
            }
            let Some(sum) = deltas.copy(resource).checked_add(delta) else {
                // A live value plus an earlier request can overflow first.
                let first = self.storage_overflow(requests).unwrap_or(resource);
                return self.representation(first);
            };
            *deltas.get_mut(resource) = sum;
        }
        let mut changes = [Change::Acquire(HtmlResource::SourceUnits, 0); STORAGE_RESOURCES];
        let mut count = 0_usize;
        for resource in RESOURCES {
            let delta = deltas.copy(resource);
            if !resource.storage() || delta == 0 {
                continue;
            }
            let (Some(dimension), Some(slot)) = (resource.dimension(), changes.get_mut(count))
            else {
                return Err(self.unexpected_change());
            };
            *slot = Change::Acquire(dimension, delta);
            count = count.saturating_add(1);
        }
        Ok((changes, count))
    }
    /// Admits the storage `requests` as one atomic batch.
    fn admit_storage(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), HtmlParseStop> {
        let (changes, count) = self.storage_batch(requests)?;
        let Some(batch) = changes.get(..count) else {
            return Err(self.unexpected_change());
        };
        let result = self.accounts_mut()?.charge(batch);
        result.map_err(|error| self.storage_stop(error, requests, batch))
    }
    pub(crate) fn acquire(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), HtmlParseStop> {
        // One request is its own batch: nothing to sum.
        self.checkpoint()?;
        let Some(dimension) = resource.dimension().filter(|_| resource.storage()) else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        };
        if units == 0 {
            return Ok(());
        }
        let change = [Change::Acquire(dimension, units)];
        let result = self.accounts_mut()?.charge(&change);
        result.map_err(|error| self.storage_stop(error, &[(resource, units)], &change))
    }
    /// Whether the ledger would refuse to retain one more diagnostic. The
    /// `Diagnostics` cap bounds retention only: a diagnostic emitted while this
    /// holds is dropped and counted, never refused, so the parse continues. The
    /// cap that refuses may be the parse's or any scope above it.
    pub(crate) fn diagnostics_retention_full(&mut self) -> Result<bool, HtmlParseStop> {
        let full = self.retention_check();
        #[cfg(test)]
        if let Some(hook) = self.after_retention_check.take() {
            hook();
        }
        full
    }
    /// Acquires the slot of one retained diagnostic. `Ok(false)` is a refusal
    /// by a `Diagnostics` cap of the parse or of a scope above it: retention
    /// bounds what is kept and never ends a parse, so the diagnostic is dropped
    /// and nothing latches. The cap can fill between [`Self::diagnostics_retention_full`]
    /// and this call when a parse shares a host scope.
    pub(crate) fn try_retain_diagnostic(&mut self) -> Result<bool, HtmlParseStop> {
        self.checkpoint()?;
        let one = [Change::Acquire(HtmlResource::Diagnostics, 1)];
        let Err(error) = self.accounts_mut()?.charge(&one) else {
            return Ok(true);
        };
        match classify(error) {
            HtmlLedgerOutcome::Cap(_) => Ok(false),
            HtmlLedgerOutcome::Exhausted(_) | HtmlLedgerOutcome::Fault(_) => {
                Err(self.ledger_stop(error, Site::PLAIN, &one))
            }
        }
    }
    /// Gives back the slot of a diagnostic [`Self::try_retain_diagnostic`]
    /// acquired and a later failure kept from being retained.
    pub(crate) fn release_unretained_diagnostic(&mut self) {
        // A slot acquired by the caller is held, so the release is not refused; a
        // ledger fault is latched as the stop.
        let _ = self.release_storage(HtmlParseResource::Diagnostics, 1);
    }
    fn retention_check(&mut self) -> Result<bool, HtmlParseStop> {
        let one = [Change::Acquire(HtmlResource::Diagnostics, 1)];
        let Err(error) = self.accounts_mut()?.check(&one) else {
            return Ok(false);
        };
        match classify(error) {
            HtmlLedgerOutcome::Cap(_) => Ok(true),
            HtmlLedgerOutcome::Exhausted(_) | HtmlLedgerOutcome::Fault(_) => {
                Err(self.ledger_stop(error, Site::PLAIN, &one))
            }
        }
    }
    /// Charges counting one dropped diagnostic, before the count is written:
    /// one visit of the retention state and one write of the counter.
    pub(crate) fn charge_dropped_diagnostic(&mut self) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (HtmlParseWorkCategory::RecordVisit, 1),
            (HtmlParseWorkCategory::RecordWrite, 1),
        ])
    }
    /// Coupled logical allocations admit atomically in resource precedence order.
    pub(crate) fn acquire_many(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), HtmlParseStop> {
        if let &[(resource, units)] = requests {
            return self.acquire(resource, units);
        }
        self.admit_storage(requests)
    }
    /// Admits an immutable provider graph retained by this original owner.
    /// Provider-computed bytes include its backing capacities, rather than a
    /// second inferred layout. Logical extents and capacity commit atomically.
    pub(crate) fn acquire_retained_provider(
        &mut self,
        bytes: u64,
        units: u64,
        entries: u64,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        let changes = retained_provider_storage(bytes, units, entries)
            .map(|(resource, amount)| Change::Acquire(resource, amount));
        let result = self.accounts_mut()?.charge(&changes);
        result.map_err(|error| {
            self.storage_stop(
                error,
                &[
                    (HtmlParseResource::ManagedUtf16Units, units),
                    (HtmlParseResource::AuxiliaryEntries, entries),
                ],
                &changes,
            )
        })
    }
    /// Ownership disposal after the retained provider handle has been dropped.
    /// This does not execute source work or observe a later cancellation.
    pub(crate) fn release_retained_provider(
        &mut self,
        owner: &ControlIdentity,
        bytes: u64,
        units: u64,
        entries: u64,
    ) -> Result<(), HtmlParseStop> {
        if !Arc::ptr_eq(&owner.0, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        self.commit_prepared_release(&Self::provider_release(bytes, units, entries))
    }
    /// Whether [`Self::release_retained_provider`] of this storage by this
    /// controller's own identity would be accepted. Nothing changes.
    pub(crate) fn can_release_retained_provider(
        &mut self,
        bytes: u64,
        units: u64,
        entries: u64,
    ) -> bool {
        let Some(changes) = Self::release_changes(&Self::provider_release(bytes, units, entries))
        else {
            return false;
        };
        self.accounts_mut()
            .is_ok_and(|accounts| accounts.check(&changes).is_ok())
    }
    fn provider_release(bytes: u64, units: u64, entries: u64) -> [(HtmlParseResource, u64); 4] {
        [
            (HtmlParseResource::ManagedCapacityBytes, bytes),
            (HtmlParseResource::PeakTransientBytes, bytes),
            (HtmlParseResource::ManagedUtf16Units, units),
            (HtmlParseResource::AuxiliaryEntries, entries),
        ]
    }
    /// The release of each request, or `None` for a request that names the work
    /// credit total, which is not a resource.
    fn release_changes<const N: usize>(
        requests: &[(HtmlParseResource, u64); N],
    ) -> Option<[HtmlChange; N]> {
        let mut changes = [Change::Release(HtmlResource::SourceUnits, 0); N];
        for (slot, &(resource, units)) in changes.iter_mut().zip(requests) {
            *slot = Change::Release(resource.dimension()?, units);
        }
        Some(changes)
    }
    /// Nonmutating logical preflight before actual storage reservations. The
    /// session rechecks with `acquire_many` immediately before graph mutation;
    /// no logical acquisition may intervene between that commit and mutation.
    pub(crate) fn check_acquire_many(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), HtmlParseStop> {
        let (changes, count) = self.storage_batch(requests)?;
        let Some(batch) = changes.get(..count) else {
            return Err(self.unexpected_change());
        };
        let result = self.accounts_mut()?.check(batch);
        result.map_err(|error| self.storage_stop(error, requests, batch))
    }
    /// Commits the actual ordinary-edge delta of a child-order change. The
    /// construction owner precharges the bounded in-place edits before this
    /// final checkpoint, then changes topology immediately after success,
    /// without an intervening allocation, poll or fallible operation. Rejection
    /// retains the edge admission for terminal cleanup of the unchanged graph.
    pub(crate) fn commit_construction_edges(
        &mut self,
        owner: &ControlIdentity,
        edge_add: u64,
        edge_remove: u64,
    ) -> Result<(), HtmlParseStop> {
        self.require_identity(owner)?;
        // The edges that remain are checked against the cap, so the release
        // comes first.
        self.admit_resources(
            &[
                Change::Release(HtmlResource::ManagedEdges, edge_remove),
                Change::Acquire(HtmlResource::ManagedEdges, edge_add),
            ],
            Site::PLAIN,
        )
    }
    /// Produces the actual empty vector and its unique accounting ticket.
    /// This creates no allocation or logical node/edge/storage admission.
    pub(crate) fn new_vector<T>(&mut self) -> Result<(Vec<T>, VectorCapacity<T>), HtmlParseStop> {
        self.checkpoint()?;
        if size_of::<T>() == 0 {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ZeroSizedStorage,
            ));
        }
        let Some(bytes) = size_of::<Vec<T>>()
            .checked_add(size_of::<VectorCapacity<T>>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::RecordWrite, 2),
            (HtmlParseWorkCategory::CopyByte, bytes),
        ])?;
        Ok((
            Vec::new(),
            VectorCapacity {
                owner: self.identity.clone(),
                capacity: 0,
                admitted: 0,
                element: PhantomData,
            },
        ))
    }
    /// Reserves the consumer's actual vector. Only the construction owner may
    /// hold its mutable storage; engine operations receive no vector escape.
    /// Logical admission and actual record copies/writes remain caller duties.
    pub(crate) fn reserve_vector<T>(
        &mut self,
        vector: &mut Vec<T>,
        allocation: &mut VectorCapacity<T>,
        additional: usize,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if allocation.capacity != vector.capacity() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        let plan = match ReservationPlan::for_vec(vector, additional) {
            Ok(Some(plan)) => plan,
            Ok(None) => return Ok(()),
            Err(stop) => return self.fail(stop),
        };
        let Some(bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        let result = self.reserve(vector, plan, &mut allocation.admitted);
        // A reserve can change the physical capacity before a post-call stop. The
        // ticket records that capacity for its identity checks, and the bytes the
        // ledger admitted for release, which a refused record leaves as they were.
        Self::record_vector_capacity(vector, allocation, result)
    }
    /// Geometric growth of an actual vector. Spare capacity is submitted and
    /// charged, never counted as live entries.
    pub(crate) fn reserve_vector_geometric<T>(
        &mut self,
        vector: &mut Vec<T>,
        allocation: &mut VectorCapacity<T>,
        additional: usize,
    ) -> Result<(), HtmlParseStop> {
        self.reserve_vector_geometric_toward(vector, allocation, additional, usize::MAX)
    }
    /// Geometric growth for a vector whose final length is known to be at most
    /// `final_len`: spare capacity never exceeds that length, so a vector that
    /// fills reaches exactly the capacity an exact policy would have retained.
    /// A required length above `final_len` is still reserved in full.
    pub(crate) fn reserve_vector_geometric_toward<T>(
        &mut self,
        vector: &mut Vec<T>,
        allocation: &mut VectorCapacity<T>,
        additional: usize,
        final_len: usize,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if allocation.capacity != vector.capacity() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        let plan = match self.geometric_sized(
            size_of::<T>(),
            vector.len(),
            vector.capacity(),
            additional,
            final_len,
        ) {
            Ok(Some(plan)) => plan,
            Ok(None) => return Ok(()),
            Err(stop) => return self.fail(stop),
        };
        let Some(bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        let result = self.reserve(vector, plan, &mut allocation.admitted);
        Self::record_vector_capacity(vector, allocation, result)
    }
    fn geometric_plan<T>(
        &self,
        vector: &Vec<T>,
        additional: usize,
    ) -> Result<Option<ReservationPlan>, HtmlParseStop> {
        self.geometric_capacity::<T>(vector.len(), vector.capacity(), additional)
    }
    fn geometric_plan_toward<T>(
        &self,
        vector: &Vec<T>,
        additional: usize,
        final_len: usize,
    ) -> Result<Option<ReservationPlan>, HtmlParseStop> {
        self.geometric_sized(
            size_of::<T>(),
            vector.len(),
            vector.capacity(),
            additional,
            final_len,
        )
    }
    fn geometric_capacity<T>(
        &self,
        len: usize,
        capacity: usize,
        additional: usize,
    ) -> Result<Option<ReservationPlan>, HtmlParseStop> {
        self.geometric_sized(size_of::<T>(), len, capacity, additional, usize::MAX)
    }
    /// Geometric growth of a bank whose element layout is only known by size.
    /// Spare capacity beyond `final_len` slots is never planned.
    ///
    /// The plan is `None` when the capacity already suffices. Otherwise the target
    /// is the greater of the required length and double the capacity (never beyond
    /// `final_len`). It depends only on the vector, never on a limit, as in
    /// `surgeist-html`'s amortized growth: the allocation path of an input is then
    /// the same under every limit, so a parse that succeeds under one limit succeeds
    /// under every larger one. The reserve gate admits the target before it is
    /// allocated, and a limit that cannot hold it refuses the reservation. The one
    /// exception is a target whose layout is not a valid allocation size: that
    /// depends on the vector alone too, and the exact plan is used instead.
    fn geometric_sized(
        &self,
        element_size: usize,
        len: usize,
        capacity: usize,
        additional: usize,
        final_len: usize,
    ) -> Result<Option<ReservationPlan>, HtmlParseStop> {
        let Some(exact) = ReservationPlan::for_sized(element_size, len, capacity, additional)?
        else {
            return Ok(None);
        };
        let representation = || HtmlParseStop::RepresentationExhausted {
            resource: HtmlParseResource::ManagedCapacityBytes,
        };
        let required = len.checked_add(additional).ok_or_else(representation)?;
        let target = required.max(capacity.saturating_mul(2).min(final_len));
        let layout = target
            .checked_mul(element_size)
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .and_then(|bytes| u64::try_from(bytes).ok());
        let Some(new_bytes) = layout else {
            return Ok(Some(exact));
        };
        Ok(Some(ReservationPlan {
            additional: target.checked_sub(len).ok_or_else(representation)?,
            needed: exact.needed,
            old_bytes: exact.old_bytes,
            new_bytes,
        }))
    }
    fn record_vector_capacity<T>(
        vector: &Vec<T>,
        allocation: &mut VectorCapacity<T>,
        result: Result<(), HtmlParseStop>,
    ) -> Result<(), HtmlParseStop> {
        allocation.capacity = vector.capacity();
        result
    }
    /// The only Weak reference permitted here is this actual wrapper ticket.
    /// Its temporary removal and restoration are prepaid, even if reserve
    /// latches a terminal failure. Opaque Arc headers stay outside byte counts.
    fn prepare_attribute_edit(
        &mut self,
        values: &Arc<Vec<surgeist_html::values::Attribute>>,
        shared: &SharedAllocation<Vec<surgeist_html::values::Attribute>>,
        vector: &VectorCapacity<surgeist_html::values::Attribute>,
    ) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (HtmlParseWorkCategory::RecordVisit, 3),
            (HtmlParseWorkCategory::ProviderCall, 9),
            (
                HtmlParseWorkCategory::CopyByte,
                const { (2 * size_of::<Weak<Vec<surgeist_html::values::Attribute>>>()) as u64 },
            ),
            (HtmlParseWorkCategory::RecordWrite, 2),
        ])?;
        if !Arc::ptr_eq(&shared.owner, &self.identity)
            || !Arc::ptr_eq(&vector.owner, &self.identity)
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if shared.finalized
            || shared.shared.as_ptr() != Arc::as_ptr(values)
            || shared.bytes != size_of::<Vec<surgeist_html::values::Attribute>>() as u64
            || vector.capacity != values.capacity()
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        if Arc::strong_count(values) != 1 || Arc::weak_count(values) != 1 {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::SharedAllocationAliased,
            ));
        }
        Ok(())
    }
    /// Consumes real storage plus its ticket, returning both on invalid owner
    /// or capacity. Cleanup needs no budget, and releases capacity only: managed
    /// nodes, logical edges/slots, cumulative work and high-water stay admitted.
    /// The session separately extracts/releases nested allocation tickets first.
    pub(crate) fn release_vector<T>(
        &mut self,
        vector: Vec<T>,
        allocation: VectorCapacity<T>,
    ) -> Result<(), VectorReleaseError<T>> {
        let release = (|| {
            if !Arc::ptr_eq(&allocation.owner, &self.identity) {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ForeignControl,
                ));
            }
            if allocation.capacity != vector.capacity() {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::CapacityMismatch,
                ));
            }
            self.cleanup_capacity(allocation.admitted)
        })();
        match release {
            Ok(()) => {
                drop(vector);
                Ok(())
            }
            Err(stop) => Err(VectorReleaseError {
                stop,
                vector,
                allocation,
            }),
        }
    }
    fn cleanup_capacity(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        self.admit_resources(&Self::capacity_release(bytes), Site::PLAIN)
    }
    /// The credit of `bytes` of retained capacity.
    const fn capacity_release(bytes: u64) -> [HtmlChange; 2] {
        [
            Change::Release(HtmlResource::ManagedCapacityBytes, bytes),
            Change::Release(HtmlResource::PeakTransientBytes, bytes),
        ]
    }
    /// Checks that `bytes` of retained capacity can be credited; nothing
    /// changes.
    fn check_capacity_release(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        self.check_resources(&Self::capacity_release(bytes), Site::PLAIN)
    }
    /// Builds an actual fixed shared payload after admission and a final poll.
    /// Nested vectors keep their own tickets. Arc headers/allocator metadata and
    /// infallible Arc setup are opaque provider limits; zero-sized identity is
    /// valid and still charges the provider call. The last poll precedes the
    /// only `Arc`, so every failure returns the owned payload and no `Arc`
    /// exists to be aliased.
    pub(crate) fn share_owned<T>(
        &mut self,
        payload: T,
    ) -> Result<(Arc<T>, SharedAllocation<T>), (HtmlParseStop, T)> {
        let bytes = match self.admit_shared_layout::<T>() {
            Ok(bytes) => bytes,
            Err(stop) => return Err((stop, payload)),
        };
        if let Err(stop) = self.confirm_shared_admission(bytes) {
            return Err((stop, payload));
        }
        let shared = Arc::new(payload);
        let allocation = SharedAllocation {
            owner: self.identity.clone(),
            shared: Arc::downgrade(&shared),
            bytes,
            element: PhantomData,
            finalized: false,
        };
        Ok((shared, allocation))
    }
    /// Charges and records the layout of one shared `T`; returns its bytes.
    fn admit_shared_layout<T>(&mut self) -> Result<u64, HtmlParseStop> {
        self.checkpoint()?;
        let Ok(bytes) = u64::try_from(size_of::<T>()) else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
            (HtmlParseWorkCategory::CopyByte, bytes),
            (HtmlParseWorkCategory::RecordWrite, 1),
        ])?;
        self.admit_layout(bytes)?;
        Ok(bytes)
    }
    /// Admits `bytes` of fixed layout as retained capacity and as the largest
    /// layout reached: the two capacity caps, then the layout cap.
    fn admit_layout(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        let [release, acquire] = self.gauge_changes(HtmlResource::AtomicLayoutBytes, bytes)?;
        self.admit_resources(
            &[
                Change::Acquire(HtmlResource::ManagedCapacityBytes, bytes),
                Change::Acquire(HtmlResource::PeakTransientBytes, bytes),
                release,
                acquire,
            ],
            Site::PLAIN,
        )
    }
    /// Polls once more after [`Self::admit_shared_layout`]; a stop refunds the
    /// capacity just charged. Admission proved that capacity and no other
    /// storage mutation precedes this refund. Were it refused anyway, the first
    /// stop is already latched, so the skipped credit is what that stop reports.
    fn confirm_shared_admission(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        match self.checkpoint() {
            Ok(()) => Ok(()),
            Err(stop) => {
                let _ = self.cleanup_capacity(bytes);
                Err(stop)
            }
        }
    }
    /// Recovers the unique payload and releases only its known shared layout.
    /// Aliased/foreign failures return the actual Arc and ticket unchanged, so
    /// the original owner may retry cleanup after outstanding handles are gone.
    pub(crate) fn release_shared<T>(
        &mut self,
        shared: Arc<T>,
        allocation: SharedAllocation<T>,
    ) -> Result<T, SharedReleaseError<T>> {
        #[cfg(test)]
        if std::mem::take(&mut self.shared_release_fault) {
            let stop = self.latch_stop(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::SharedAllocationAliased,
            ));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            let stop = self.latch_stop(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if !std::ptr::eq(Arc::as_ptr(&shared), allocation.shared.as_ptr()) {
            let stop = self.latch_stop(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if let Err(stop) = self.check_capacity_release(allocation.bytes) {
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        match Arc::try_unwrap(shared) {
            Ok(payload) => {
                // The release was checked above and nothing intervened.
                if let Err(stop) = self.cleanup_capacity(allocation.bytes) {
                    return Err(SharedReleaseError {
                        stop,
                        shared: Arc::new(payload),
                        allocation,
                    });
                }
                Ok(payload)
            }
            Err(shared) => {
                let stop = self.latch_stop(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::SharedAllocationAliased,
                ));
                Err(SharedReleaseError {
                    stop,
                    shared,
                    allocation,
                })
            }
        }
    }
    /// Drops one actual zero-sized shared payload and its accounting ticket.
    /// Other strong owners may retain the opaque Arc header; this operation
    /// makes no deallocation claim and refunds no capacity. Invalid tickets
    /// return the actual Arc and ticket for cleanup by their original owner.
    pub(crate) fn release_zero_sized_shared<T>(
        &mut self,
        shared: Arc<T>,
        allocation: SharedAllocation<T>,
    ) -> Result<(), SharedReleaseError<T>> {
        let invariant = if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            Some(HtmlParseInvariant::ForeignControl)
        } else if !std::ptr::eq(Arc::as_ptr(&shared), allocation.shared.as_ptr())
            || size_of::<T>() != 0
            || allocation.bytes != 0
        {
            Some(HtmlParseInvariant::CapacityMismatch)
        } else {
            None
        };
        if let Some(invariant) = invariant {
            let stop = self.latch_stop(HtmlParseStop::InternalInvariant(invariant));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        drop(shared);
        drop(allocation);
        Ok(())
    }
    pub(crate) fn release(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        if !resource.storage() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        }
        if resource == HtmlParseResource::ManagedNodes {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonReclaimableNodes,
            ));
        }
        self.release_storage(resource, units)
    }
    /// Credits `units` of a storage resource; a credit beyond what is held is a
    /// release underflow.
    fn release_storage(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), HtmlParseStop> {
        let Some(dimension) = resource.dimension() else {
            return Err(self.fail_invariant(HtmlParseInvariant::NonStorageResource));
        };
        self.admit_resources(&[Change::Release(dimension, units)], Site::PLAIN)
    }
    /// Fixed original-owner disposal of exactly admitted resources. Selected
    /// replacement and cleanup retain the private ownership receipt; final
    /// report publication additionally preflights release before its last poll.
    /// This cannot execute source work or observe a later cancellation. A
    /// request beyond what is admitted latches `ReleaseUnderflow` and credits
    /// nothing, not even the requests that precede it.
    pub(crate) fn commit_prepared_release<const N: usize>(
        &mut self,
        requests: &[(HtmlParseResource, u64); N],
    ) -> Result<(), HtmlParseStop> {
        let Some(changes) = Self::release_changes(requests) else {
            return Err(self.fail_invariant(HtmlParseInvariant::NonStorageResource));
        };
        self.admit_resources(&changes, Site::PLAIN)
    }
    /// Releases actually deallocated non-node logical slots after first-stop.
    /// No successful engine transition may use this cleanup-only path.
    pub(crate) fn cleanup_release(
        &mut self,
        identity: &ControlIdentity,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), HtmlParseStop> {
        if self.stop().is_none() {
            return Err(self.fail_invariant(HtmlParseInvariant::CleanupWhileRunning));
        }
        if !Arc::ptr_eq(&identity.0, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if !resource.storage() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        }
        if resource == HtmlParseResource::ManagedNodes {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonReclaimableNodes,
            ));
        }
        self.release_storage(resource, units)
    }
    pub(crate) fn observe_tree_depth(&mut self, depth: u64) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        let record = self.gauge_changes(HtmlResource::TreeDepth, depth)?;
        self.admit_resources(&record, Site::PLAIN)
    }
    /// Reserves for `plan`. `admitted` is the bytes the ledger holds for the
    /// vector; it becomes the capacity recorded once the ledger admits it, and
    /// stays as it was when the record is refused.
    fn reserve<T>(
        &mut self,
        vector: &mut Vec<T>,
        plan: ReservationPlan,
        admitted: &mut u64,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        // A reservation never shrinks the allocation it replaces.
        self.preflight_capacity(plan.old_bytes, plan.new_bytes, true)?;
        // Only an allocator refusal of the spare room falls back to the slots
        // that are needed; a limit that refuses the target never does.
        if vector.try_reserve_exact(plan.additional).is_err()
            && (plan.needed >= plan.additional || vector.try_reserve_exact(plan.needed).is_err())
        {
            return self.fail(HtmlParseStop::AllocationFailed {
                resource: HtmlParseResource::ManagedCapacityBytes,
                layout_bytes: plan.new_bytes,
            });
        }
        let Some(actual) = vector
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        *admitted = self.record_capacity(*admitted, actual, plan.new_bytes)?;
        if actual > plan.new_bytes {
            // `actual` exceeds `new_bytes`, which is at least `old_bytes`.
            let requested_delta = actual.saturating_sub(plan.old_bytes);
            return self.fail(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: plan.new_bytes,
                current: plan.old_bytes,
                requested_delta,
            });
        }
        self.checkpoint()
    }
    /// Admits the layout of a replacement allocation before it is made: the
    /// capacity left once `old` bytes are credited and `new` are held must fit
    /// the capacity cap, the overlap of both layouts must fit the transient
    /// cap, and `new` must fit the layout cap. The capacity cap is checked and
    /// not recorded; the overlap and the layout are recorded, the overlap as a
    /// peak that is released again.
    pub(super) fn preflight_capacity(
        &mut self,
        old: u64,
        new: u64,
        never_shrinks: bool,
    ) -> Result<(), HtmlParseStop> {
        let current = self.live(HtmlParseResource::ManagedCapacityBytes)?;
        let Some(without_old) = current.checked_sub(old) else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        };
        if without_old.checked_add(new).is_none() || current.checked_add(new).is_none() {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        }
        if never_shrinks && new < old {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        self.check_resources(
            &[
                Change::Release(HtmlResource::ManagedCapacityBytes, old),
                Change::Acquire(HtmlResource::ManagedCapacityBytes, new),
            ],
            Site::PLAIN.growth(new.saturating_sub(old)),
        )?;
        let [release, acquire] = self.gauge_changes(HtmlResource::AtomicLayoutBytes, new)?;
        self.admit_resources(
            &[
                Change::Acquire(HtmlResource::PeakTransientBytes, new),
                Change::Release(HtmlResource::PeakTransientBytes, new),
                release,
                acquire,
            ],
            Site::PLAIN,
        )
    }
    /// Records the capacity an allocation holds in place of the `old` bytes it
    /// was charged. The allocation exists whatever its size, so the record is
    /// not refused for exceeding the planned `expected` layout: an overshoot is
    /// recorded and the caller stops. The result is the bytes the ledger holds
    /// for the allocation. When the ledger cannot hold an overshoot under the
    /// cap the stop names the overshoot and the ledger keeps the `old` bytes:
    /// the caller records the physical capacity in its ticket and keeps `old` as
    /// the admitted bytes, so that cleanup credits what the ledger holds. The
    /// first-stop usage shows that custody without the refused overshoot.
    pub(super) fn record_capacity(
        &mut self,
        old: u64,
        actual: u64,
        expected: u64,
    ) -> Result<u64, HtmlParseStop> {
        #[cfg(test)]
        let actual = actual.saturating_add(std::mem::take(&mut self.capacity_overshoot));
        let batch = [
            Change::Release(HtmlResource::ManagedCapacityBytes, old),
            Change::Acquire(HtmlResource::ManagedCapacityBytes, actual),
            Change::Acquire(HtmlResource::PeakTransientBytes, actual),
            Change::Release(HtmlResource::PeakTransientBytes, old),
        ];
        let recorded = self.accounts_mut()?.charge(&batch);
        let Err(error) = recorded else {
            return Ok(actual);
        };
        if actual > expected
            && matches!(
                classify(error),
                HtmlLedgerOutcome::Cap(_) | HtmlLedgerOutcome::Exhausted(_)
            )
        {
            return self.fail(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: expected,
                current: old,
                requested_delta: actual.saturating_sub(old),
            });
        }
        Err(self.ledger_stop(error, Site::PLAIN, &batch))
    }
    /// Cleanup is permitted after a terminal stop; it refunds no admitted work or
    /// high-water observation. The vector owner supplies the bytes the ledger
    /// admitted for it.
    fn cleanup_storage(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
        bytes: u64,
    ) -> Result<(), HtmlParseStop> {
        let Some(dimension) = resource.dimension() else {
            return Err(self.fail_invariant(HtmlParseInvariant::NonStorageResource));
        };
        self.admit_resources(
            &[
                Change::Release(dimension, units),
                Change::Release(HtmlResource::ManagedCapacityBytes, bytes),
                Change::Release(HtmlResource::PeakTransientBytes, bytes),
            ],
            Site::PLAIN,
        )
    }
    /// Admits only the known fixed Arc payload. Arc headers, allocator metadata,
    /// and its infallible setup are provider limitations, not measured capacity.
    /// The consumer polls after the actual shared-wrapper construction.
    fn admit_document_string_wrapper(
        &mut self,
        mut buffer: OwnedVec<u16>,
    ) -> Result<(Vec<u16>, DocumentStringAllocation), HtmlParseStop> {
        let admission = buffer
            .require_owner(self)
            .and_then(|()| self.admit_document_string_payload(buffer.resource));
        let bytes = match admission {
            Ok(bytes) => bytes,
            Err(reason) => {
                if buffer
                    .owner
                    .as_ref()
                    .is_none_or(|owner| Arc::ptr_eq(owner, &self.identity))
                {
                    buffer.release(self)?;
                }
                return Err(reason);
            }
        };
        let allocation = DocumentStringAllocation {
            owner: self.identity.clone(),
            units: buffer.len() as u64,
            vector_capacity: buffer.capacity(),
            vector_admitted: buffer.admitted,
            wrapper_bytes: bytes,
            resource: HtmlParseResource::BufferedUtf16Units,
            data_pointer: buffer.as_slice().as_ptr() as usize,
            backing: None,
            finalized: false,
        };
        Ok((buffer.into_vec(), allocation))
    }
    fn admit_document_string_payload(
        &mut self,
        resource: HtmlParseResource,
    ) -> Result<u64, HtmlParseStop> {
        self.checkpoint()?;
        if resource != HtmlParseResource::BufferedUtf16Units {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        }
        let bytes = size_of::<Vec<u16>>() as u64;
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
        ])?;
        // This total includes known fixed payloads, never opaque provider headers.
        self.admit_layout(bytes)?;
        Ok(bytes)
    }
    /// Owns actual wrapper construction, including the empty-vector case where
    /// a dangling data pointer cannot by itself prove backing provenance.
    /// Arc's fixed known Vec payload is admitted; opaque headers, allocator
    /// bookkeeping and infallible std Arc setup remain explicit limitations.
    pub(crate) fn freeze_document_string(
        &mut self,
        buffer: OwnedVec<u16>,
    ) -> Result<
        (
            surgeist_html::values::DocumentString,
            DocumentStringAllocation,
        ),
        HtmlParseStop,
    > {
        let (units, mut allocation) = self.admit_document_string_wrapper(buffer)?;
        let text = surgeist_html::values::DocumentString::from_utf16(units);
        if let Err(stop) = self.bind_document_string(&mut allocation, &text) {
            drop(text);
            self.release_document_string(allocation)?;
            return Err(stop);
        }
        Ok((text, allocation))
    }
    /// Binds the actual immutable wrapper to the vector admitted by the sole
    /// producer. Pointer equality observes identity only; no pointer is read.
    fn bind_document_string(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        self.charge_many(&[
            (HtmlParseWorkCategory::RecordVisit, 1),
            (HtmlParseWorkCategory::ProviderCall, 1),
        ])?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if allocation.backing.is_some()
            || value.len() as u64 != allocation.units
            || value.shared_code_units().capacity() != allocation.vector_capacity
            || value.code_units().as_ptr() as usize != allocation.data_pointer
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        allocation.backing = Some(Arc::downgrade(value.shared_code_units()));
        Ok(())
    }
    pub(crate) fn require_document_string(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        self.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.validate_document_string(allocation, value)
    }
    /// Qualifies the new historical-materialization output slot before cleanup
    /// takes it. This fixed disposal proof admits no parsing work or storage.
    #[cfg(test)]
    pub(crate) fn require_materialized_string_disposal(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.validate_document_string(allocation, value)?;
        if allocation.resource != HtmlParseResource::BufferedUtf16Units
            || allocation.wrapper_bytes != size_of::<Vec<u16>>() as u64
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        let Some(bytes) = allocation
            .vector_admitted
            .checked_add(allocation.wrapper_bytes)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        if allocation.units > self.live(allocation.resource)?
            || bytes > self.live(HtmlParseResource::ManagedCapacityBytes)?
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ReleaseUnderflow,
            ));
        }
        Ok(())
    }
    /// Disposal-only inspection of already-admitted backing. Terminal cleanup
    /// may validate ownership before taking a caller's retained payload; it
    /// cannot resume parsing, acquire storage, or refund execution observations.
    pub(crate) fn require_document_string_for_disposal(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        if self.stop().is_none() {
            return self.require_document_string(allocation, value);
        }
        self.validate_document_string(allocation, value)
    }
    fn validate_document_string(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if allocation
            .backing
            .as_ref()
            .is_none_or(|backing| backing.as_ptr() != Arc::as_ptr(value.shared_code_units()))
            || value.len() as u64 != allocation.units
            || value.shared_code_units().capacity() != allocation.vector_capacity
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        Ok(())
    }
    /// One atomic logical domain transition; real capacity and the immutable
    /// backing are unchanged. A failed cap or poll leaves both currents and the
    /// ticket phase unchanged.
    pub(crate) fn move_document_string_to_managed(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.require_document_string(allocation, value)?;
        if allocation.resource != HtmlParseResource::BufferedUtf16Units {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        }
        self.admit_resources(
            &[
                Change::Release(HtmlResource::BufferedUtf16Units, allocation.units),
                Change::Acquire(HtmlResource::ManagedUtf16Units, allocation.units),
            ],
            Site::PLAIN,
        )?;
        allocation.resource = HtmlParseResource::ManagedUtf16Units;
        Ok(())
    }

    /// Transfers this producer's actual independent text ticket into the CSP
    /// provider aggregate. The aggregate is acquired before the vector ticket
    /// retires; the immutable backing remains live under its new sole receipt.
    pub(crate) fn csp_document_string(
        &mut self,
        source: &surgeist_html::values::DocumentString,
    ) -> Result<(surgeist_html::values::DocumentString, usize, usize), HtmlParseStop> {
        let mut buffer = OwnedVec::new(HtmlParseResource::BufferedUtf16Units)?;
        // The final length is known, so one reservation admits the whole copy.
        if let Err(stop) = buffer.try_extend_from_slice(self, source.code_units()) {
            buffer.release(self)?;
            return Err(stop);
        }
        let (text, allocation) = self.freeze_document_string(buffer)?;
        let Some(bytes) = allocation
            .vector_capacity
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| bytes.checked_add(allocation.wrapper_bytes as usize))
        else {
            drop(text);
            self.release_document_string(allocation)?;
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let units = text.shared_code_units().capacity();
        let admitted = self
            .charge(HtmlParseWorkCategory::ProviderCall, 2)
            .and_then(|()| self.acquire_retained_provider(bytes as u64, units as u64, 1));
        if let Err(stop) = admitted {
            drop(text);
            self.release_document_string(allocation)?;
            return Err(stop);
        }
        // This is an ownership transfer, not finalization or destruction of
        // live backing. The new aggregate is already admitted; consume the
        // unique prior ticket and retire only its duplicate debit.
        let debit = allocation
            .vector_admitted
            .saturating_add(allocation.wrapper_bytes);
        self.commit_prepared_release(&[
            (HtmlParseResource::ManagedCapacityBytes, debit),
            (HtmlParseResource::PeakTransientBytes, debit),
            (allocation.resource, allocation.units),
        ])?;
        drop(allocation);
        Ok((text, bytes, units))
    }

    pub(crate) fn release_document_string(
        &mut self,
        allocation: DocumentStringAllocation,
    ) -> Result<(), HtmlParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        let Some(bytes) = allocation
            .vector_admitted
            .checked_add(allocation.wrapper_bytes)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        if !allocation.finalized
            && allocation
                .backing
                .as_ref()
                .is_some_and(|backing| backing.strong_count() != 0)
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::SharedAllocationAliased,
            ));
        }
        let (resource, units) = (allocation.resource, allocation.units);
        drop(allocation);
        self.cleanup_storage(resource, units, bytes)
    }
}

/// Unique ticket for an actual consumer-owned vector. It cannot register an
/// arbitrary preallocated vector or be cloned to refund the same capacity twice.
#[derive(Debug)]
pub(crate) struct VectorCapacity<T> {
    owner: Arc<()>,
    /// The capacity the vector physically holds, which identity and capacity
    /// checks compare.
    capacity: usize,
    /// The bytes of that capacity the ledger admitted, which release credits. A
    /// capacity the ledger refused to record is physical but not admitted.
    admitted: u64,
    element: PhantomData<T>,
}
impl<T> VectorCapacity<T> {
    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
}

#[derive(Debug)]
pub(crate) struct VectorReleaseError<T> {
    stop: HtmlParseStop,
    vector: Vec<T>,
    allocation: VectorCapacity<T>,
}
impl<T> VectorReleaseError<T> {
    pub(crate) fn into_parts(self) -> (HtmlParseStop, Vec<T>, VectorCapacity<T>) {
        (self.stop, self.vector, self.allocation)
    }
}

/// Known fixed payload only, excluding opaque shared headers and allocator
/// bookkeeping. Nested dynamic payloads retain distinct vector/text tickets.
#[derive(Debug)]
pub(crate) struct SharedAllocation<T> {
    owner: Arc<()>,
    shared: Weak<T>,
    bytes: u64,
    element: PhantomData<T>,
    finalized: bool,
}
impl<T> SharedAllocation<T> {
    #[cfg(test)]
    pub(crate) fn payload_bytes(&self) -> u64 {
        self.bytes
    }
}

#[derive(Debug)]
pub(crate) struct SharedReleaseError<T> {
    stop: HtmlParseStop,
    shared: Arc<T>,
    allocation: SharedAllocation<T>,
}
impl<T> SharedReleaseError<T> {
    pub(crate) fn into_parts(self) -> (HtmlParseStop, Arc<T>, SharedAllocation<T>) {
        (self.stop, self.shared, self.allocation)
    }
}

/// Non-cloneable accounting ticket for the transferred payload. It must remain
/// alongside the frozen text until the execution owner releases that allocation.
pub(crate) struct DocumentStringAllocation {
    owner: Arc<()>,
    units: u64,
    /// The capacity the backing physically holds.
    vector_capacity: usize,
    /// The bytes of that capacity the ledger admitted; see [`VectorCapacity`].
    vector_admitted: u64,
    wrapper_bytes: u64,
    resource: HtmlParseResource,
    data_pointer: usize,
    backing: Option<Weak<Vec<u16>>>,
    finalized: bool,
}
impl DocumentStringAllocation {
    #[cfg(test)]
    pub(crate) fn units(&self) -> u64 {
        self.units
    }
    #[cfg(test)]
    pub(crate) fn vector_capacity(&self) -> usize {
        self.vector_capacity
    }
    #[cfg(test)]
    pub(crate) fn wrapper_bytes(&self) -> u64 {
        self.wrapper_bytes
    }
}

/// Checked layout only; it neither allocates nor admits logical storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReservationPlan {
    /// The slots requested: the required ones, plus amortized spare room.
    additional: usize,
    /// The slots the caller needs. The allocator is asked for them alone only
    /// when it refuses `additional`.
    needed: usize,
    old_bytes: u64,
    new_bytes: u64,
}
impl ReservationPlan {
    fn for_vec<T>(vector: &Vec<T>, additional: usize) -> Result<Option<Self>, HtmlParseStop> {
        Self::for_capacity::<T>(vector.len(), vector.capacity(), additional)
    }
    fn for_capacity<T>(
        len: usize,
        capacity: usize,
        additional: usize,
    ) -> Result<Option<Self>, HtmlParseStop> {
        Self::for_sized(size_of::<T>(), len, capacity, additional)
    }
    fn for_sized(
        element_size: usize,
        len: usize,
        capacity: usize,
        additional: usize,
    ) -> Result<Option<Self>, HtmlParseStop> {
        if element_size == 0 {
            return Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ZeroSizedStorage,
            ));
        }
        let length = len
            .checked_add(additional)
            .ok_or(HtmlParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedCapacityBytes,
            })?;
        if length <= capacity {
            return Ok(None);
        }
        let layout = |slots: usize| {
            slots
                .checked_mul(element_size)
                .filter(|bytes| *bytes <= isize::MAX as usize)
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(HtmlParseStop::RepresentationExhausted {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                })
        };
        Ok(Some(Self {
            additional,
            needed: additional,
            old_bytes: layout(capacity)?,
            new_bytes: layout(length)?,
        }))
    }
}

/// How a full vector reserves room for its next entry.
#[derive(Clone, Copy)]
enum Growth {
    /// Exactly the slots required.
    Exact,
    /// Spare capacity that at least doubles it, never planned beyond the given
    /// final length (`usize::MAX`: unknown). Independent of every limit.
    Geometric(usize),
}

/// Private variable storage. No mutable Vec escape can bypass fallible growth.
/// Elements must be flat owned records; recursive provider drop is not bounded
/// by this byte ledger. `into_vec` transfers admitted storage without a copy.
pub(crate) struct OwnedVec<T> {
    vector: Vec<T>,
    /// The bytes of the vector's capacity the ledger admitted, which release
    /// credits; see [`VectorCapacity`].
    admitted: u64,
    resource: HtmlParseResource,
    owner: Option<Arc<()>>,
}
impl<T> OwnedVec<T> {
    pub(crate) fn new(resource: HtmlParseResource) -> Result<Self, HtmlParseStop> {
        if !resource.storage() || resource == HtmlParseResource::ManagedNodes {
            return Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        }
        if size_of::<T>() == 0 {
            return Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ZeroSizedStorage,
            ));
        }
        Ok(Self {
            vector: Vec::new(),
            admitted: 0,
            resource,
            owner: None,
        })
    }
    pub(crate) fn as_slice(&self) -> &[T] {
        &self.vector
    }
    /// The entries for editing, for tests that corrupt what a report holds.
    #[cfg(test)]
    pub(crate) fn as_mut_slice_for_test(&mut self) -> &mut [T] {
        &mut self.vector
    }
    pub(crate) fn len(&self) -> usize {
        self.vector.len()
    }
    pub(crate) fn capacity(&self) -> usize {
        self.vector.capacity()
    }
    /// Admits one indexed record visit, including an out-of-bounds attempt.
    /// The borrow grants no payload-work or nested-storage admission: the caller
    /// must charge every field inspection, write and copy before performing it,
    /// and admit/release any nested payload and allocation tickets separately.
    pub(crate) fn get_mut(
        &mut self,
        index: usize,
        control: &mut ParseControl,
    ) -> Result<Option<&mut T>, HtmlParseStop> {
        self.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(self.vector.get_mut(index))
    }
    /// Admits one visit to the container and lends its admitted entries for editing in place.
    /// The borrow grants no per-entry admission: the caller must charge every access, and the
    /// entries can neither grow nor shrink.
    pub(crate) fn as_mut_slice(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<&mut [T], HtmlParseStop> {
        self.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(&mut self.vector)
    }
    /// Swaps two actual admitted fixed slots. No payload clone, allocation,
    /// logical-slot change, or mutable-vector escape is involved.
    pub(crate) fn swap_slots(
        &mut self,
        left: usize,
        right: usize,
        control: &mut ParseControl,
    ) -> Result<bool, HtmlParseStop> {
        self.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if left >= self.vector.len() || right >= self.vector.len() {
            return Ok(false);
        }
        if left == right {
            return Ok(true);
        }
        let Some(bytes) = size_of::<T>().checked_mul(2) else {
            return control.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        control.admit_atomic_layout(bytes)?;
        control.charge_many(&[
            (HtmlParseWorkCategory::CopyByte, bytes as u64),
            (HtmlParseWorkCategory::RecordWrite, 2),
        ])?;
        self.vector.swap(left, right);
        Ok(true)
    }
    /// Removes one admitted slot and transfers its value without releasing its
    /// nested payload tickets. The caller owns their separate release/transfer.
    /// Capacity and all high-water observations are retained. An empty attempt
    /// visits only the container; it charges no write or payload copy.
    pub(crate) fn try_pop(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<T>, HtmlParseStop> {
        self.require_owner(control)?;
        if self.vector.is_empty() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            return Ok(None);
        }
        let Ok(bytes) = u64::try_from(size_of::<T>()) else {
            return control.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        control.charge_many(&[
            (HtmlParseWorkCategory::RecordVisit, 1),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, bytes),
        ])?;
        if size_of::<T>() > WORK_QUANTUM - 2 {
            control.record_layout(bytes)?;
        }
        control.release(self.resource, 1)?;
        Ok(self.vector.pop())
    }
    /// Failure cleanup only: a first stop must already be latched. The returned
    /// value may only be deallocated or have its nested tickets released; this
    /// is not successful tokenizer progress. Checks the same owner and logical
    /// slot underflow, retaining first-stop, capacity, work and high-water use.
    /// Cleanup is deliberately uncharged so exhaustion cannot strand tickets.
    pub(crate) fn cleanup_pop(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<T>, HtmlParseStop> {
        if control.stop().is_none() {
            return Err(control.fail_invariant(HtmlParseInvariant::CleanupWhileRunning));
        }
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if self.vector.is_empty() {
            return Ok(None);
        }
        control.cleanup_storage(self.resource, 1, 0)?;
        Ok(self.vector.pop())
    }
    pub(crate) fn require_owner(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        control.checkpoint()?;
        if let Some(owner) = &self.owner {
            if !Arc::ptr_eq(owner, &control.identity) {
                return control.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ForeignControl,
                ));
            }
        } else {
            self.owner = Some(control.identity.clone());
        }
        Ok(())
    }
    /// Read-only owner preflight for actual payload disposal, including after
    /// exhaustion. An unbound empty vector has no admitted storage owner.
    pub(crate) fn require_disposal_owner(
        &self,
        control: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        if control.stop().is_none() {
            control.checkpoint()?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        }
        if let Some(owner) = &self.owner {
            if control.stop().is_none() {
                control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            }
            if !Arc::ptr_eq(owner, &control.identity) {
                return control.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ForeignControl,
                ));
            }
        }
        Ok(())
    }
    fn prepare(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
        copy_bytes: u64,
        writes: u64,
        growth: Growth,
    ) -> Result<(), HtmlParseStop> {
        self.require_owner(control)?;
        let plan = match growth {
            Growth::Exact => ReservationPlan::for_vec(&self.vector, additional),
            Growth::Geometric(final_len) => {
                control.geometric_plan_toward(&self.vector, additional, final_len)
            }
        };
        let plan = match plan {
            Ok(plan) => plan,
            Err(reason) => return control.fail(reason),
        };
        self.prepare_plan(control, additional, copy_bytes, writes, plan, true)
    }
    fn prepare_plan(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
        copy_bytes: u64,
        writes: u64,
        plan: Option<ReservationPlan>,
        check_units: bool,
    ) -> Result<(), HtmlParseStop> {
        let Ok(units) = u64::try_from(additional) else {
            return control.representation(self.resource);
        };
        let reserve_bytes = match plan {
            Some(plan) => match plan.old_bytes.checked_add(plan.new_bytes) {
                Some(bytes) => bytes,
                None => return control.representation(HtmlParseResource::ManagedCapacityBytes),
            },
            None => 0,
        };
        control.charge_many(&[
            (HtmlParseWorkCategory::CopyByte, copy_bytes),
            (HtmlParseWorkCategory::RecordWrite, writes),
            (HtmlParseWorkCategory::ReserveByte, reserve_bytes),
            (
                HtmlParseWorkCategory::ProviderCall,
                u64::from(plan.is_some()),
            ),
        ])?;
        // A vector whose unit the caller already holds is not checked again.
        if check_units {
            control.check_acquire_many(&[(self.resource, units)])?;
        }
        if let Some(plan) = plan {
            control.reserve(&mut self.vector, plan, &mut self.admitted)?;
        }
        Ok(())
    }
    /// One side of a consumed pair: the same geometric reservation as a lone
    /// push, so a pair's capacity does not depend on any limit either. The
    /// caller has already admitted the original combined atomic layout.
    fn prepare_pair_slot(
        &mut self,
        control: &mut ParseControl,
        check_unit: bool,
    ) -> Result<(), HtmlParseStop> {
        self.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let plan = match control.geometric_plan_toward(&self.vector, 1, usize::MAX) {
            Ok(plan) => plan,
            Err(stop) => return control.fail(stop),
        };
        self.prepare_plan(control, 1, size_of::<T>() as u64, 1, plan, check_unit)
    }
    /// Reserves only real physical append storage; initialized entries and
    /// their payload copies/writes remain the actual push caller's duty.
    pub(crate) fn reserve_geometric(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), HtmlParseStop> {
        self.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let plan = match control.geometric_plan(&self.vector, additional) {
            Ok(Some(plan)) => plan,
            Ok(None) => return Ok(()),
            Err(stop) => return control.fail(stop),
        };
        let Some(bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
            return control.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        control.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
        ])?;
        control.reserve(&mut self.vector, plan, &mut self.admitted)
    }
    pub(crate) fn try_push(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), HtmlParseStop> {
        self.push_with(control, value, Growth::Exact)
    }
    /// `try_push` for a list that grows with the document: the same admission
    /// in the same order, but a full vector reserves geometric spare capacity
    /// instead of exactly one more slot. The target is admitted before it is
    /// allocated and charged in full: spare room counts against the limits, and
    /// a limit that cannot hold the target refuses the reservation.
    pub(crate) fn try_push_geometric(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), HtmlParseStop> {
        self.push_with(control, value, Growth::Geometric(usize::MAX))
    }
    /// `try_push_geometric` for a list whose final length is known to be at most
    /// `final_len`: spare capacity never exceeds it, so a list that fills ends
    /// with exactly the capacity an exact policy would have retained.
    pub(crate) fn try_push_geometric_toward(
        &mut self,
        control: &mut ParseControl,
        value: T,
        final_len: usize,
    ) -> Result<(), HtmlParseStop> {
        self.push_with(control, value, Growth::Geometric(final_len))
    }
    fn push_with(
        &mut self,
        control: &mut ParseControl,
        value: T,
        growth: Growth,
    ) -> Result<(), HtmlParseStop> {
        control.checkpoint()?;
        let bytes = size_of::<T>() as u64;
        self.prepare(control, 1, bytes, 1, growth)?;
        if size_of::<T>() >= WORK_QUANTUM {
            control.record_layout(bytes)?;
        }
        control.acquire(self.resource, 1)?;
        self.vector.push(value);
        Ok(())
    }
    /// One consumed evidence commit. Both capacities, logical slots and fixed
    /// writes are admitted before either vector's committed length changes.
    #[cfg(test)]
    pub(crate) fn try_push_pair<U>(
        &mut self,
        other: &mut OwnedVec<U>,
        control: &mut ParseControl,
        value: T,
        other_value: U,
    ) -> Result<(), HtmlParseStop> {
        self.push_pair(other, control, value, other_value, false)
    }
    /// As [`Self::try_push_pair`] for a pair whose first slot the caller has
    /// already acquired, as retention custody is: only the second is admitted.
    pub(crate) fn try_push_pair_held<U>(
        &mut self,
        other: &mut OwnedVec<U>,
        control: &mut ParseControl,
        value: T,
        other_value: U,
    ) -> Result<(), HtmlParseStop> {
        self.push_pair(other, control, value, other_value, true)
    }
    fn push_pair<U>(
        &mut self,
        other: &mut OwnedVec<U>,
        control: &mut ParseControl,
        value: T,
        other_value: U,
        first_held: bool,
    ) -> Result<(), HtmlParseStop> {
        let bytes = size_of::<T>()
            .checked_add(size_of::<U>())
            .ok_or_else(|| control.fail_invariant(HtmlParseInvariant::CapacityMismatch))?;
        control.admit_atomic_layout(bytes)?;
        self.prepare_pair_slot(control, !first_held)?;
        other.prepare_pair_slot(control, true)?;
        if first_held {
            control.acquire(other.resource, 1)?;
        } else {
            control.acquire_many(&[(self.resource, 1), (other.resource, 1)])?;
        }
        // No fallible operation follows admission. Record is published last.
        other.vector.push(other_value);
        self.vector.push(value);
        Ok(())
    }
    pub(crate) fn clear(&mut self, control: &mut ParseControl) -> Result<(), HtmlParseStop> {
        control.checkpoint()?;
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        while !self.vector.is_empty() {
            let bytes = size_of::<T>() as u64;
            control.charge_many(&[
                (HtmlParseWorkCategory::RecordVisit, 1),
                (HtmlParseWorkCategory::RecordWrite, 1),
                (HtmlParseWorkCategory::CopyByte, bytes),
            ])?;
            if const { size_of::<T>() + 2 > WORK_QUANTUM } {
                control.record_layout(bytes)?;
            }
            control.release(self.resource, 1)?;
            drop(self.vector.pop());
        }
        Ok(())
    }
    /// Uses normal push admission, but returns the original value on every
    /// precommit failure so its nested allocation tickets remain recoverable.
    /// The caller must release those tickets through their original owner; this
    /// does not refund admitted work or capacity and never resumes failed work.
    pub(crate) fn try_push_recover(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), (HtmlParseStop, T)> {
        self.recover_with(control, value, Growth::Exact)
    }
    /// `try_push_recover` with the geometric growth of `try_push_geometric`.
    pub(crate) fn try_push_recover_geometric(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), (HtmlParseStop, T)> {
        self.recover_with(control, value, Growth::Geometric(usize::MAX))
    }
    fn recover_with(
        &mut self,
        control: &mut ParseControl,
        value: T,
        growth: Growth,
    ) -> Result<(), (HtmlParseStop, T)> {
        let admission = (|| {
            control.checkpoint()?;
            let bytes = size_of::<T>() as u64;
            self.prepare(control, 1, bytes, 1, growth)?;
            if size_of::<T>() >= WORK_QUANTUM {
                control.record_layout(bytes)?;
            }
            control.acquire(self.resource, 1)
        })();
        match admission {
            Ok(()) => {
                self.vector.push(value);
                Ok(())
            }
            Err(stop) => Err((stop, value)),
        }
    }
    pub(crate) fn release(self, control: &mut ParseControl) -> Result<(), HtmlParseStop> {
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        control.cleanup_storage(self.resource, self.vector.len() as u64, self.admitted)?;
        drop(self);
        Ok(())
    }
    fn into_vec(self) -> Vec<T> {
        self.vector
    }
}
impl<T: Copy> OwnedVec<T> {
    /// Every copied segment polls and has at most 256 bytes, except a single
    /// larger fixed record admitted under AtomicLayoutBytes.
    pub(crate) fn try_extend_from_slice(
        &mut self,
        control: &mut ParseControl,
        values: &[T],
    ) -> Result<(), HtmlParseStop> {
        self.extend_with(control, values, Growth::Exact)
    }
    /// `try_extend_from_slice` for a list that keeps growing: a full vector
    /// reserves the geometric target of `try_push_geometric`, so repeated
    /// appends copy each element a constant number of times.
    pub(crate) fn try_extend_from_slice_geometric(
        &mut self,
        control: &mut ParseControl,
        values: &[T],
    ) -> Result<(), HtmlParseStop> {
        self.extend_with(control, values, Growth::Geometric(usize::MAX))
    }
    fn extend_with(
        &mut self,
        control: &mut ParseControl,
        values: &[T],
        growth: Growth,
    ) -> Result<(), HtmlParseStop> {
        control.checkpoint()?;
        self.prepare(control, values.len(), 0, 0, growth)?;
        // A zero-sized element has no byte stride; refuse it when this is compiled.
        let stride = const {
            assert!(size_of::<T>() != 0);
            WORK_QUANTUM / size_of::<T>()
        }
        .max(1);
        for chunk in values.chunks(stride) {
            let bytes = std::mem::size_of_val(chunk) as u64;
            control.charge(HtmlParseWorkCategory::CopyByte, bytes)?;
            if bytes > WORK_QUANTUM as u64 {
                control.record_layout(bytes)?;
            }
            control.acquire(self.resource, chunk.len() as u64)?;
            self.vector.extend_from_slice(chunk);
        }
        Ok(())
    }
}

impl HtmlParseResourceUsage {
    pub fn current(&self) -> u64 {
        self.current
    }
    pub fn high_water(&self) -> u64 {
        self.high_water
    }
}
impl HtmlParseUsage {
    pub fn actual_capacity_bytes(&self) -> u64 {
        self.actual_capacity_bytes
    }
    pub fn actual_capacity_high_water(&self) -> u64 {
        self.actual_capacity_high_water
    }
    pub fn largest_atomic_layout(&self) -> u64 {
        self.largest_atomic_layout
    }
}

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod html_value_tests;
#[cfg(test)]
mod release_tests;
#[cfg(test)]
mod schema_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn fixed_slot_swap_charges_two_real_slot_moves_and_writes_without_storage_growth() {
        let mut c = control();
        let mut slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        slots.try_push(&mut c, 11u16).unwrap();
        slots.try_push(&mut c, 22u16).unwrap();
        let before = c.usage().clone();
        let capacity = slots.capacity();
        assert!(slots.swap_slots(0, 1, &mut c).unwrap());
        assert_eq!(slots.as_slice(), &[22, 11]);
        assert_eq!(slots.capacity(), capacity);
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordVisit)
                - before.work(HtmlParseWorkCategory::RecordVisit),
            2
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::CopyByte)
                - before.work(HtmlParseWorkCategory::CopyByte),
            4
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordWrite)
                - before.work(HtmlParseWorkCategory::RecordWrite),
            2
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::ReserveByte),
            before.work(HtmlParseWorkCategory::ReserveByte)
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::AuxiliaryEntries),
            before.resource(HtmlParseResource::AuxiliaryEntries)
        );
        let before = c.usage().clone();
        assert!(slots.swap_slots(0, 0, &mut c).unwrap());
        assert!(!slots.swap_slots(0, 2, &mut c).unwrap());
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordVisit)
                - before.work(HtmlParseWorkCategory::RecordVisit),
            4
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::CopyByte),
            before.work(HtmlParseWorkCategory::CopyByte)
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordWrite),
            before.work(HtmlParseWorkCategory::RecordWrite)
        );
        slots.release(&mut c).unwrap();
    }

    #[test]
    fn fixed_slot_swap_exact_and_one_short_work_preserve_atomic_mutation() {
        for short in [false, true] {
            let selected = limits();
            let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
            let mut slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
            slots.try_push(&mut c, 11u16).unwrap();
            slots.try_push(&mut c, 22u16).unwrap();
            // Two visits + two u16 moves + two writes; initial admitted
            // allocation already establishes the real two-slot atomic layout.
            let remaining = selected.work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                remaining - 8 + u64::from(short),
            )
            .unwrap();
            let result = slots.swap_slots(0, 1, &mut c);
            if short {
                assert!(matches!(
                    result,
                    Err(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!(slots.as_slice(), &[11, 22]);
            } else {
                assert_eq!(result, Ok(true));
                assert_eq!(slots.as_slice(), &[22, 11]);
            }
            slots.release(&mut c).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                0
            );
        }
    }

    #[test]
    fn fixed_slot_swap_rejects_foreign_owner_and_cancellation_before_mutation() {
        let cancel = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancel.clone());
        let mut slots = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        slots.try_push(&mut c, 11u16).unwrap();
        slots.try_push(&mut c, 22u16).unwrap();
        let mut foreign = control();
        assert_eq!(
            slots.swap_slots(0, 1, &mut foreign),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(slots.as_slice(), &[11, 22]);
        cancel.cancel();
        assert_eq!(
            slots.swap_slots(0, 1, &mut c),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(slots.as_slice(), &[11, 22]);
        slots.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
    }

    #[test]
    fn unsupported_standalone_state_is_terminal_without_refunding_admitted_storage_or_work() {
        let mut c = control();
        let mut storage = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        storage.try_push(&mut c, 7u16).unwrap();
        let before = c.usage().clone();
        let stop = HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance);
        assert_eq!(
            c.stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance),
            stop
        );
        assert_eq!(c.usage(), before);
        assert_eq!(
            c.stop_unsupported_standalone(HtmlParseUnsupportedState::PendingScript),
            stop
        );
        assert_eq!(c.charge(HtmlParseWorkCategory::StateStep, 1), Err(stop));
        assert_eq!(storage.try_push(&mut c, 9), Err(stop));
        assert_eq!(storage.cleanup_pop(&mut c).unwrap(), Some(7));
        storage.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn an_observed_cancellation_precedes_later_unsupported_state() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            c.stop_unsupported_standalone(HtmlParseUnsupportedState::InvalidNesting),
            HtmlParseStop::Cancelled
        );
        assert_eq!(
            c.stop_unsupported_standalone(HtmlParseUnsupportedState::ScriptPause),
            HtmlParseStop::Cancelled
        );
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    }
    use super::*;
    use surgeist_html::values::DocumentString;

    #[test]
    fn optional_source_binding_move_refuses_running_control_and_preserves_stopped_usage() {
        let source = HtmlSource::utf8("x");
        let request = crate::HtmlDocumentParseRequest::offline(
            source.clone(),
            crate::parser::tree_builder::tests::limits(100),
        );
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        assert_eq!(
            c.take_stopped_optional_source_binding(),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CleanupWhileRunning
            ))
        );
        assert!(c.stop().is_none());
        c.require_source(&source).unwrap();
        request.cancellation().cancel();
        assert_eq!(c.checkpoint(), Err(HtmlParseStop::Cancelled));
        let first = c.first_stop_usage().unwrap().clone();
        let current = c.usage().clone();
        assert_eq!(
            c.stopped_source_binding().unwrap().unwrap().0,
            *source.identity()
        );
        let binding = c.take_stopped_optional_source_binding().unwrap().unwrap();
        assert_eq!(&binding.0, source.identity());
        assert_eq!(binding.1, source.coordinates());
        assert!(c.stopped_source_binding().unwrap().is_none());
        assert_eq!(c.usage(), current);
        assert_eq!(c.first_stop_usage(), Some(&first));
    }

    #[test]
    fn pending_form_effect_stop_is_canonical_and_cannot_replace_first_failure() {
        for invariant in [
            HtmlParseInvariant::PendingFormEffects,
            HtmlParseInvariant::IncompleteFormAssociationCreation,
        ] {
            let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
            let stop = HtmlParseStop::InternalInvariant(invariant);
            assert_eq!(c.fail_invariant(invariant), stop);
            assert_eq!(c.charge(HtmlParseWorkCategory::RecordWrite, 1), Err(stop));
            assert_eq!(c.fail_invariant(HtmlParseInvariant::ForeignSource), stop);
            assert_eq!(c.checkpoint(), Err(stop));
        }
        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            cancelled.fail_invariant(HtmlParseInvariant::PendingFormEffects),
            HtmlParseStop::Cancelled
        );
        assert_eq!(cancelled.stop(), Some(HtmlParseStop::Cancelled));
    }

    #[test]
    fn first_stop_snapshot_excludes_denied_growth_and_survives_terminal_refund() {
        let caps = limits()
            .into_builder()
            .auxiliary_entries(5)
            .active_formatting_entries(5)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        assert!(c.first_stop_usage().is_none());
        c.acquire(HtmlParseResource::AuxiliaryEntries, 4).unwrap();
        c.charge(HtmlParseWorkCategory::RecordWrite, 3).unwrap();
        let before = c.usage().clone();
        let stop = c
            .acquire(HtmlParseResource::AuxiliaryEntries, 2)
            .unwrap_err();
        assert_eq!(
            stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                limit: 5,
                current: 4,
                requested_delta: 2,
            }
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
        let owner = c.identity();
        c.cleanup_release(&owner, HtmlParseResource::AuxiliaryEntries, 4)
            .unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            c.first_stop_usage()
                .unwrap()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            4
        );
        c.cancellation.cancel();
        assert_eq!(c.checkpoint(), Err(stop));
        assert_eq!(c.fail_invariant(HtmlParseInvariant::ForeignControl), stop);
        assert_eq!(c.first_stop_usage(), Some(&before));
    }
    #[test]
    fn cancellation_snapshot_retains_real_vector_capacity_before_disposal() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values.try_push(&mut c, 7_u16).unwrap();
        let before = c.usage().clone();
        assert_eq!(
            before.resource(HtmlParseResource::AuxiliaryEntries).current,
            1
        );
        assert!(before.actual_capacity_bytes >= size_of::<u16>() as u64);
        cancellation.cancel();
        assert_eq!(c.checkpoint(), Err(HtmlParseStop::Cancelled));
        assert_eq!(c.first_stop_usage(), Some(&before));
        values.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
        assert_eq!(
            c.usage().actual_capacity_high_water,
            before.actual_capacity_high_water
        );
        assert_eq!(
            c.stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance),
            HtmlParseStop::Cancelled
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
    }
    fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(1_000)
            .buffered_utf16_units(1_000)
            .token_attributes(1_000)
            .auxiliary_entries(1_000)
            .active_formatting_entries(1_000)
            .managed_nodes(1_000)
            .managed_edges(1_000)
            .managed_attributes(1_000)
            .managed_utf16_units(1_000)
            .source_map_entries(1_000)
            .source_map_payload_units(1_000)
            .diagnostics(1_000)
            .diagnostic_payload_units(1_000)
            .context_effects(1_000)
            .open_elements(1_000)
            .tree_depth(1_000)
            .managed_capacity_bytes(16_000)
            .peak_transient_bytes(32_000)
            .atomic_layout_bytes(16_000)
            .work_credits(100_000)
            .finish()
            .unwrap()
    }
    fn control() -> ParseControl {
        ParseControl::new(limits(), HtmlParseCancellation::new())
    }

    /// The documented charge is one record visit, then, per compared unit and literal byte, the
    /// two bytes of the stored unit plus the one literal byte: three inspected bytes. A length
    /// mismatch is decided after the visit alone, and the comparison stops at the first
    /// mismatching pair. Matching follows Infra "ASCII case-insensitive": only U+0041..U+005A
    /// fold, so U+212A KELVIN SIGN never matches `k`.
    #[test]
    fn ascii_case_insensitive_literal_match_charges_a_visit_then_three_bytes_per_compared_pair() {
        let cases: [(&[u16], &str, bool, u64); 9] = [
            (&[0x6F, 0x70, 0x65, 0x6E], "open", true, 4),
            (&[0x4F, 0x70, 0x45, 0x4E], "open", true, 4),
            (&[0x6F, 0x70, 0x65, 0x6E, 0x78], "open", false, 0),
            (&[0x6F, 0x70, 0x65], "open", false, 0),
            (&[], "open", false, 0),
            (&[0x6F, 0x70, 0x58, 0x6E], "open", false, 3),
            (&[0x78, 0x70, 0x65, 0x6E], "open", false, 1),
            (&[0x212A], "k", false, 1),
            (&[], "", true, 0),
        ];
        for (units, literal, expected, pairs) in cases {
            let mut c = control();
            let before = c.usage().clone();
            let value = surgeist_html::values::DocumentString::from_utf16(units.to_vec());
            assert_eq!(
                c.ascii_case_insensitive_literal_matches(&value, literal)
                    .unwrap(),
                expected,
                "{units:04X?} against {literal:?}"
            );
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::RecordVisit)
                    - before.work(HtmlParseWorkCategory::RecordVisit),
                1,
                "{units:04X?} against {literal:?}"
            );
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::InspectByte)
                    - before.work(HtmlParseWorkCategory::InspectByte),
                3 * pairs,
                "{units:04X?} against {literal:?}"
            );
        }
    }

    #[test]
    fn ascii_case_insensitive_literal_match_returns_the_first_refused_charge() {
        let value = surgeist_html::values::DocumentString::from("OPEN");
        // One credit for the visit and three for the first pair; the second pair is refused.
        let caps = limits().into_builder().work_credits(4).finish().unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        assert!(matches!(
            c.ascii_case_insensitive_literal_matches(&value, "open"),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        assert_eq!(c.usage().work(HtmlParseWorkCategory::RecordVisit), 1);
        assert_eq!(c.usage().work(HtmlParseWorkCategory::InspectByte), 3);
    }

    #[test]
    fn paired_fixed_layout_refusal_precedes_growth_work_and_logical_slots() {
        let selected = limits()
            .into_builder()
            .atomic_layout_bytes((size_of::<u64>() + size_of::<u32>() - 1) as u64)
            .work_credits(0)
            .diagnostics(0)
            .source_map_entries(0)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut records = OwnedVec::new(HtmlParseResource::Diagnostics).unwrap();
        let mut locations = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        let combined = (size_of::<u64>() + size_of::<u32>()) as u64;
        assert_eq!(
            records.try_push_pair(&mut locations, &mut c, 7_u64, 9_u32),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AtomicLayoutBytes,
                current: 0,
                requested_delta: combined,
                limit: combined - 1,
            })
        );
        assert_eq!((records.len(), locations.len()), (0, 0));
        assert_eq!((records.capacity(), locations.capacity()), (0, 0));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            0
        );
        let snapshot = c.first_stop_usage().unwrap().clone();
        records.release(&mut c).unwrap();
        locations.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.first_stop_usage(), Some(&snapshot));
    }

    #[test]
    fn paired_first_work_or_logical_refusal_precedes_foreign_second_owner() {
        for work in [true, false] {
            let builder = limits().into_builder();
            let selected = if work {
                builder.work_credits(0)
            } else {
                builder.diagnostics(0)
            }
            .finish()
            .unwrap();
            let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
            let mut foreign = control();
            let mut records = OwnedVec::new(HtmlParseResource::Diagnostics).unwrap();
            let mut locations = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
            locations.require_owner(&mut foreign).unwrap();
            let foreign_before = foreign.usage().clone();
            let result = records.try_push_pair(&mut locations, &mut c, 7_u64, 9_u32);
            assert!(
                matches!(result, Err(HtmlParseStop::LimitExceeded { resource, .. })
                if resource == if work { HtmlParseResource::WorkCredits } else { HtmlParseResource::Diagnostics })
            );
            assert_eq!((records.len(), locations.len()), (0, 0));
            assert_eq!((records.capacity(), locations.capacity()), (0, 0));
            assert_eq!(foreign.usage(), foreign_before);
            assert_eq!(foreign.stop(), None);
            let first = c.stop();
            let snapshot = c.first_stop_usage().unwrap().clone();
            assert!(records.try_push_pair(&mut locations, &mut c, 7, 9).is_err());
            assert_eq!(c.stop(), first);
            assert_eq!(c.first_stop_usage(), Some(&snapshot));
            records.release(&mut c).unwrap();
            locations.release(&mut foreign).unwrap();
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            assert_eq!(foreign.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn paired_foreign_second_owner_retains_prepared_first_without_half_commit() {
        let mut c = control();
        let mut foreign = control();
        let mut records = OwnedVec::new(HtmlParseResource::Diagnostics).unwrap();
        let mut locations = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        locations.require_owner(&mut foreign).unwrap();
        assert_eq!(
            records.try_push_pair(&mut locations, &mut c, 7_u64, 9_u32),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!((records.len(), locations.len()), (0, 0));
        assert_eq!(records.capacity(), 1);
        assert_eq!(locations.capacity(), 0);
        assert_eq!(c.usage().actual_capacity_bytes, size_of::<u64>() as u64);
        assert_eq!(
            c.usage().resource(HtmlParseResource::Diagnostics).current,
            0
        );
        assert_eq!(foreign.stop(), None);
        let snapshot = c.first_stop_usage().unwrap().clone();
        records.release(&mut c).unwrap();
        locations.release(&mut foreign).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.first_stop_usage(), Some(&snapshot));
    }

    #[test]
    fn local_geometric_growth_charges_spare_capacity_and_a_limit_refuses_the_target() {
        // Six bytes hold three u16 slots, which exact growth fills. Geometric growth asks for
        // 1, 2 and then 4 slots: the fourth slot's eight bytes do not fit.
        let caps = limits()
            .into_builder()
            .managed_capacity_bytes(6)
            .peak_transient_bytes(10)
            .atomic_layout_bytes(6)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let (mut values, mut ticket) = c.new_vector::<u16>().unwrap();
        for capacity in [1, 2] {
            c.reserve_vector_geometric(&mut values, &mut ticket, 1)
                .unwrap();
            assert_eq!(values.capacity(), capacity);
            values.push(7);
        }
        assert_eq!(c.usage().work(HtmlParseWorkCategory::ReserveByte), 8); // 2 + (2+4)
        assert_eq!(c.usage().actual_capacity_bytes, 4);
        let reserve = c.usage().work(HtmlParseWorkCategory::ReserveByte);
        // The target is submitted whole to the reserve gate, which refuses it without
        // shrinking the request to the room that is left.
        assert_eq!(
            c.reserve_vector_geometric(&mut values, &mut ticket, 1),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: 6,
                current: 4,
                requested_delta: 4,
            })
        );
        assert_eq!(values, [7, 7]);
        assert_eq!(values.capacity(), 2);
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::ReserveByte),
            reserve + 12 // the target's old and new layouts: 4 + 8
        );
        let at_stop = c.first_stop_usage().unwrap().clone();
        c.release_vector(values, ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.first_stop_usage(), Some(&at_stop));
        assert_eq!(c.usage().work, at_stop.work);

        // The exact policy still fits the same limits with its three slots.
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let (mut values, mut ticket) = c.new_vector::<u16>().unwrap();
        c.reserve_vector(&mut values, &mut ticket, 3).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 6);
        c.release_vector(values, ticket).unwrap();
    }

    /// A forged plan stands in for an allocator that refuses the spare room. `try_reserve_exact`
    /// refuses a request whose byte size exceeds `isize::MAX` as a capacity overflow, before any
    /// allocator call, so asking for `usize::MAX - len` extra slots fails exactly as an
    /// allocator refusal of a larger target would. The reservation must then ask for the one
    /// slot that is needed, and account the capacity it actually reached.
    #[test]
    fn a_refused_larger_target_still_reserves_the_slot_that_is_needed() {
        let mut c = control();
        let mut values: Vec<u64> = vec![1, 2, 3];
        values.resize(values.capacity(), 9);
        let before = values.clone();
        let len = values.len();
        let old_bytes = (len * size_of::<u64>()) as u64;
        let new_bytes = ((len + 1) * size_of::<u64>()) as u64;
        c.set(HtmlParseResource::ManagedCapacityBytes, old_bytes);
        c.set(HtmlParseResource::PeakTransientBytes, old_bytes);
        // Without the fallback the larger request alone is refused.
        assert!(values.clone().try_reserve_exact(usize::MAX - len).is_err());
        let plan = ReservationPlan {
            additional: usize::MAX - len,
            needed: 1,
            old_bytes,
            new_bytes,
        };
        c.reserve(&mut values, plan, &mut plan.old_bytes.clone())
            .unwrap();
        assert!(values.capacity() > len);
        assert_eq!(values, before);
        let actual = (values.capacity() * size_of::<u64>()) as u64;
        assert_eq!(c.usage().actual_capacity_bytes, actual);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            actual
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::PeakTransientBytes)
                .current,
            actual
        );
        assert_eq!(c.stop(), None);

        // A plan that needs no more than it asks for has no fallback: the refusal is final.
        let mut c = control();
        let mut values: Vec<u64> = vec![1, 2, 3];
        values.resize(values.capacity(), 9);
        c.set(HtmlParseResource::ManagedCapacityBytes, old_bytes);
        let plan = ReservationPlan {
            additional: usize::MAX - len,
            needed: usize::MAX - len,
            old_bytes,
            new_bytes,
        };
        assert!(matches!(
            c.reserve(&mut values, plan, &mut plan.old_bytes.clone()),
            Err(HtmlParseStop::AllocationFailed { .. })
        ));
        assert_eq!(values.capacity(), len);
    }

    #[test]
    fn sized_geometric_growth_doubles_whatever_the_limits_and_usage_hold() {
        // A bank described only by its element size: sixteen-byte elements. The plan depends on
        // the lengths alone, so the same plan is made under a tight and under a roomy limit and
        // whatever the held bytes.
        let tight = limits()
            .into_builder()
            .managed_capacity_bytes(96)
            .peak_transient_bytes(200)
            .atomic_layout_bytes(96)
            .finish()
            .unwrap();
        for caps in [tight, limits()] {
            let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
            let mut additional = |held: u64, len: usize, capacity: usize| {
                c.set(HtmlParseResource::ManagedCapacityBytes, held);
                c.geometric_sized(16, len, capacity, 1, usize::MAX)
                    .unwrap()
                    .map(|plan| plan.additional)
            };
            // Spare room still fits: no reservation is planned.
            assert_eq!(additional(32, 1, 2), None);
            // Doubling two slots to four, four to eight, and six to twelve.
            assert_eq!(additional(32, 2, 2), Some(2));
            assert_eq!(additional(64, 4, 4), Some(4));
            assert_eq!(additional(96, 6, 6), Some(6));
            // The spare room stops at a known final length, but never below the need.
            assert_eq!(
                c.geometric_sized(16, 4, 4, 1, 6)
                    .unwrap()
                    .map(|plan| plan.additional),
                Some(2)
            );
            assert_eq!(
                c.geometric_sized(16, 4, 4, 3, 6)
                    .unwrap()
                    .map(|plan| plan.additional),
                Some(3)
            );
            assert_eq!(
                c.geometric_sized(0, 0, 0, 1, usize::MAX).unwrap_err(),
                HtmlParseStop::InternalInvariant(HtmlParseInvariant::ZeroSizedStorage)
            );
        }
        // A target whose layout is no valid allocation falls back to the need.
        let c = control();
        let huge = isize::MAX as usize / 16 - 1;
        assert_eq!(
            c.geometric_sized(16, huge, huge, 1, usize::MAX)
                .unwrap()
                .map(|plan| plan.additional),
            Some(1)
        );
    }

    #[test]
    fn geometric_push_doubles_capacity_and_admits_one_logical_entry_per_push() {
        let mut c = control();
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        let mut capacities = Vec::new();
        for value in 0..9u32 {
            values.try_push_geometric(&mut c, value).unwrap();
            if capacities.last() != Some(&values.capacity()) {
                capacities.push(values.capacity());
            }
        }
        assert_eq!(capacities, [1, 2, 4, 8, 16]);
        assert_eq!(values.as_slice(), [0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            9
        );
        // Old plus new bytes of each growth: 4 + (4+8) + (8+16) + (16+32) + (32+64).
        assert_eq!(c.usage().work(HtmlParseWorkCategory::ReserveByte), 184);
        values.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn copying_a_policy_string_into_a_document_string_reserves_work_linear_in_its_length() {
        // Each doubling of the policy's length may at most double (with a tenth more for the
        // fixed cost and capacity steps) the bytes reserved and the total credits.
        let roomy = HtmlParseLimits::builder()
            .source_units(1_000_000)
            .buffered_utf16_units(1_000_000)
            .token_attributes(1_000)
            .auxiliary_entries(1_000)
            .active_formatting_entries(1_000)
            .managed_nodes(1_000)
            .managed_edges(1_000)
            .managed_attributes(1_000)
            .managed_utf16_units(1_000_000)
            .source_map_entries(1_000)
            .source_map_payload_units(1_000)
            .diagnostics(1_000)
            .diagnostic_payload_units(1_000)
            .context_effects(1_000)
            .open_elements(1_000)
            .tree_depth(1_000)
            .managed_capacity_bytes(1_000_000_000)
            .peak_transient_bytes(1_000_000_000)
            .atomic_layout_bytes(1_000_000_000)
            .work_credits(100_000_000_000)
            .finish()
            .unwrap();
        let mut reserved = Vec::new();
        let mut total = Vec::new();
        for units in [1_000_usize, 2_000, 4_000, 8_000] {
            let mut c = ParseControl::new(roomy, HtmlParseCancellation::new());
            let policy = DocumentString::from_utf16(vec![0x61; units]);
            let (copy, ..) = c.csp_document_string(&policy).unwrap();
            assert_eq!(copy.code_units(), policy.code_units());
            reserved.push(c.usage().work(HtmlParseWorkCategory::ReserveByte));
            total.push(c.usage().resource(HtmlParseResource::WorkCredits).current);
        }
        for (name, work) in [("reservation work", reserved), ("total work", total)] {
            for window in work.windows(2) {
                assert!(
                    window[1] * 10 <= window[0] * 22,
                    "{name} grew faster than linearly with the policy length: {work:?}"
                );
            }
        }
    }

    #[test]
    fn bounded_geometric_push_ends_at_exactly_the_known_final_length() {
        let mut c = control();
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        let mut capacities = Vec::new();
        for value in 0..11u32 {
            values.try_push_geometric_toward(&mut c, value, 11).unwrap();
            if capacities.last() != Some(&values.capacity()) {
                capacities.push(values.capacity());
            }
        }
        // Doubling stops at the known length instead of reaching sixteen.
        assert_eq!(capacities, [1, 2, 4, 8, 11]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            11
        );
        values.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn geometric_push_checks_logical_admission_before_the_physical_reserve() {
        // One logical entry is allowed and the vector is full. The refusal is
        // the logical one, exactly as for an exact push, and nothing is reserved.
        let caps = limits()
            .into_builder()
            .auxiliary_entries(1)
            .active_formatting_entries(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values.try_push_geometric(&mut c, 1u32).unwrap();
        let capacity = values.capacity();
        assert!(matches!(
            values.try_push_geometric(&mut c, 2u32),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                ..
            })
        ));
        assert_eq!(values.capacity(), capacity);
        assert_eq!(values.as_slice(), [1]);
    }

    #[test]
    fn bounded_geometric_growth_never_plans_spare_capacity_beyond_the_final_length() {
        let mut c = control();
        let (mut values, mut ticket) = c.new_vector::<u16>().unwrap();
        let mut capacities = Vec::new();
        for value in 0..11u16 {
            c.reserve_vector_geometric_toward(&mut values, &mut ticket, 1, 11)
                .unwrap();
            if capacities.last() != Some(&values.capacity()) {
                capacities.push(values.capacity());
            }
            values.push(value);
        }
        // Doubling stops at the known final length instead of reaching sixteen.
        assert_eq!(capacities, [1, 2, 4, 8, 11]);
        // A required length above the stated bound is still reserved in full.
        c.reserve_vector_geometric_toward(&mut values, &mut ticket, 1, 3)
            .unwrap();
        assert_eq!(values.capacity(), 12);
        c.release_vector(values, ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn geometric_reservation_admits_only_capacity_until_real_record_pushes() {
        let mut c = control();
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values.reserve_geometric(&mut c, 3).unwrap();
        assert_eq!(values.len(), 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        for value in [10u32, 20, 30] {
            values.try_push(&mut c, value).unwrap();
        }
        let reserved = c.usage().work(HtmlParseWorkCategory::ReserveByte);
        values.reserve_geometric(&mut c, 1).unwrap();
        assert_eq!(values.capacity(), 6);
        assert_eq!(values.as_slice(), [10, 20, 30]);
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::ReserveByte),
            reserved + 12 + 24
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            3
        );
        let work = c.usage().work;
        values.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.usage().work, work);
    }

    #[test]
    fn geometric_growth_rejects_foreign_owner_and_cancellation_before_allocation() {
        for foreign in [false, true] {
            let cancel = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancel.clone());
            let (mut values, mut ticket) = c.new_vector::<u32>().unwrap();
            let mut other = control();
            if !foreign {
                cancel.cancel();
            }
            let active = if foreign { &mut other } else { &mut c };
            let result = active.reserve_vector_geometric(&mut values, &mut ticket, 1);
            assert_eq!(
                result,
                Err(if foreign {
                    HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl)
                } else {
                    HtmlParseStop::Cancelled
                })
            );
            assert_eq!(values.capacity(), 0);
            assert_eq!(active.usage().work(HtmlParseWorkCategory::ReserveByte), 0);
            c.release_vector(values, ticket).unwrap();
            assert_eq!(c.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn hidden_input_metadata_slots_are_admitted_before_take_and_bound_only_to_actual_nodes() {
        use crate::construction::{SessionError, release_token_kind};
        use crate::parser::tokenizer::{
            states::HtmlTokenizerState,
            tests::LexerFixture,
            token::{Event, TokenKind},
        };
        use crate::parser::tree_builder::StandaloneDocumentDriver;
        // Creation holds a measured number of auxiliary entries beyond those
        // already retained. Any smaller allowance refuses with the first
        // auxiliary stop, either before the lexical take (the tag keeps its own
        // backing) or after it, and a larger allowance never refuses earlier.
        let attempt = |additional: Option<u64>| {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head></head><input type=hidden>"),
                limits()
                    .into_builder()
                    .atomic_layout_bytes(1_000_000)
                    .managed_capacity_bytes(1_000_000)
                    .peak_transient_bytes(2_000_000)
                    .work_credits(10_000_000)
                    .finish()
                    .unwrap(),
            );
            let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
                &request,
            ));
            for _ in 0..128 {
                if driver.as_mut().unwrap().standalone_parts().is_some() {
                    break;
                }
                driver.as_mut().unwrap().step().unwrap();
            }
            let (mut c, mut session, capability) =
                StandaloneDocumentDriver::take_standalone_fixture(&mut driver).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let (mut input, token_source) = loop {
                let event = lexer.next(&mut c, false).unwrap().unwrap();
                match event {
                    Event::Token(event) => {
                        let token = event.into_token(&mut c).unwrap();
                        if matches!(&token.kind,TokenKind::Tag(tag) if tag.name.value.text().code_units()==[105,110,112,117,116])
                        {
                            let TokenKind::Tag(tag) = token.kind else {
                                unreachable!()
                            };
                            break (Some(tag), token.source);
                        }
                        release_token_kind(&mut Some(token.kind), &mut c).unwrap();
                    }
                    event => event.release(&mut c).unwrap(),
                }
            };
            let parent = session.root(&mut c).unwrap();
            let pointer = input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr();
            let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
            let slots = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            if let Some(additional) = additional {
                c.test_limit(HtmlParseResource::AuxiliaryEntries, slots + additional);
            }
            let result = session.allocate_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            );
            let taken = input.is_none();
            if !taken {
                assert_eq!(
                    input
                        .as_ref()
                        .unwrap()
                        .name
                        .value
                        .text()
                        .code_units()
                        .as_ptr(),
                    pointer,
                    "a tag refused before the take keeps its own name backing"
                );
                assert_eq!(
                    c.usage().resource(HtmlParseResource::ManagedNodes).current,
                    nodes
                );
            }
            let held = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current
                - slots;
            let first = c.stop();
            if let Some(tag) = input.take() {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), first);
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::TokenAttributes,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::SourceMapEntries,
                HtmlParseResource::ManagedCapacityBytes,
            ] {
                assert_eq!(c.usage().resource(resource).current, 0);
            }
            (result.map(|_| ()), first, taken, held)
        };
        let (created, stop, _, needed) = attempt(None);
        created.unwrap();
        assert_eq!(stop, None);
        let mut taken_before = false;
        let mut refused_before_take = 0;
        for additional in 0..needed {
            let (result, stop, taken, held) = attempt(Some(additional));
            assert!(
                matches!(
                    (&result, stop),
                    (
                        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                            resource: HtmlParseResource::AuxiliaryEntries,
                            ..
                        })),
                        Some(HtmlParseStop::LimitExceeded {
                            resource: HtmlParseResource::AuxiliaryEntries,
                            ..
                        })
                    )
                ),
                "allowance {additional}: {result:?}"
            );
            assert!(held <= additional, "allowance {additional} holds {held}");
            assert!(
                taken || !taken_before,
                "allowance {additional} refused earlier than a smaller allowance"
            );
            taken_before |= taken;
            refused_before_take += usize::from(!taken);
        }
        assert!(
            refused_before_take > 0,
            "the metadata slots are admitted before the lexical take"
        );
        assert!(taken_before, "the creation row follows the lexical take");
        let (created, stop, _, held) = attempt(Some(needed));
        created.unwrap();
        assert_eq!(stop, None);
        assert_eq!(held, needed);
    }

    fn bound_string(
        control: &mut ParseControl,
        units: &[u16],
    ) -> (DocumentString, DocumentStringAllocation) {
        let mut buffer = OwnedVec::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_extend_from_slice(control, units).unwrap();
        let (units, mut ticket) = control.admit_document_string_wrapper(buffer).unwrap();
        let text = DocumentString::from_utf16(units);
        control.bind_document_string(&mut ticket, &text).unwrap();
        (text, ticket)
    }
    #[test]
    fn exact_bound_backing_moves_domains_without_capacity_or_second_copy() {
        let mut c = control();
        let (text, mut ticket) = bound_string(&mut c, &[0xd800, 0xd83d, 0xde00, 0]);
        let capacity = c.usage().actual_capacity_bytes;
        let copies = c.usage().work(HtmlParseWorkCategory::CopyByte);
        c.move_document_string_to_managed(&mut ticket, &text)
            .unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            4
        );
        assert_eq!(c.usage().actual_capacity_bytes, capacity);
        assert_eq!(c.usage().work(HtmlParseWorkCategory::CopyByte), copies);
        assert_eq!(text.code_units(), &[0xd800, 0xd83d, 0xde00, 0]);
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn one_short_managed_domain_cap_preserves_buffered_ticket_and_both_currents() {
        let caps = limits()
            .into_builder()
            .managed_utf16_units(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let (text, mut ticket) = bound_string(&mut c, &[0x61, 0x62]);
        let capacity = c.usage().actual_capacity_bytes;
        assert_eq!(
            c.move_document_string_to_managed(&mut ticket, &text),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                limit: 1,
                current: 0,
                requested_delta: 2,
            })
        );
        assert_eq!(ticket.resource, HtmlParseResource::BufferedUtf16Units);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            2
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );
        assert_eq!(c.usage().actual_capacity_bytes, capacity);
        let stop = c.stop();
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn matching_units_wrong_backing_and_foreign_owner_cannot_move_ticket() {
        let mut c = control();
        let (text, mut ticket) = bound_string(&mut c, &[0x61]);
        let mut foreign = control();
        assert_eq!(
            foreign.move_document_string_to_managed(&mut ticket, &text),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        let other = DocumentString::from_utf16(vec![0x61]);
        assert_eq!(
            c.require_document_string(&ticket, &other),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch
            ))
        );
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn duplicate_transfer_and_cancellation_reject_atomically_and_managed_cleanup_is_terminal() {
        let cancel = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancel.clone());
        let (text, mut ticket) = bound_string(&mut c, &[0x61]);
        c.move_document_string_to_managed(&mut ticket, &text)
            .unwrap();
        assert_eq!(
            c.move_document_string_to_managed(&mut ticket, &text),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource
            ))
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            1
        );
        let work = c.usage().work;
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.usage().work, work);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );

        let mut c = ParseControl::new(limits(), cancel.clone());
        let (text, mut ticket) = bound_string(&mut c, &[0x62]);
        cancel.cancel();
        assert_eq!(
            c.move_document_string_to_managed(&mut ticket, &text),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(ticket.resource, HtmlParseResource::BufferedUtf16Units);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        drop(text);
        c.release_document_string(ticket).unwrap();
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(c.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn all_limits_have_independent_slots() {
        let limits = HtmlParseLimits::builder()
            .source_units(1)
            .buffered_utf16_units(2)
            .token_attributes(3)
            .auxiliary_entries(4)
            .managed_nodes(5)
            .managed_edges(6)
            .managed_attributes(7)
            .managed_utf16_units(8)
            .source_map_entries(9)
            .source_map_payload_units(10)
            .diagnostics(11)
            .diagnostic_payload_units(12)
            .context_effects(13)
            .open_elements(14)
            .active_formatting_entries(15)
            .tree_depth(16)
            .managed_capacity_bytes(17)
            .peak_transient_bytes(18)
            .atomic_layout_bytes(19)
            .work_credits(20)
            .finish()
            .unwrap();
        assert_eq!(
            limits.values().into_array(),
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
            ]
        );
    }
    #[test]
    fn source_counts_supplied_basis_once() {
        let source = HtmlSource::utf8("a😀");
        let mut control = control();
        control.admit_source(&source).unwrap();
        control.admit_source(&source.clone()).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            5
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        let source = HtmlSource::utf16(DocumentString::from_utf16([0xD800, 0x61]));
        let mut control = super::tests::control();
        control.admit_source(&source).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            2
        );
    }
    #[test]
    fn equal_text_different_identity_is_rejected() {
        let mut control = control();
        control.admit_source(&HtmlSource::utf8("x")).unwrap();
        assert_eq!(
            control.admit_source(&HtmlSource::utf8("x")),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignSource
            ))
        );
    }
    #[test]
    fn zero_source_cap_admits_only_empty() {
        let mut limits = limits();
        limits = limits.into_builder().source_units(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.admit_source(&HtmlSource::utf8("")).unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        assert_eq!(
            control.admit_source(&HtmlSource::utf8("a")),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceUnits,
                limit: 0,
                current: 0,
                requested_delta: 1
            })
        );
    }
    #[test]
    fn cancellation_is_sticky_across_threads() {
        let cancellation = HtmlParseCancellation::new();
        let other = cancellation.clone();
        std::thread::spawn(move || {
            other.cancel();
            other.cancel();
        })
        .join()
        .unwrap();
        let mut control = ParseControl::new(limits(), cancellation);
        assert_eq!(
            control.charge(HtmlParseWorkCategory::StateStep, 1),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
    }
    #[test]
    fn observed_cancel_precedes_limit_and_first_stop_never_changes() {
        let cancellation = HtmlParseCancellation::new();
        cancellation.cancel();
        let mut limits = limits();
        limits = limits.into_builder().work_credits(0).finish().unwrap();
        let mut control = ParseControl::new(limits, cancellation);
        assert_eq!(
            control.charge(HtmlParseWorkCategory::StateStep, 1),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(
            control.acquire(HtmlParseResource::Diagnostics, u64::MAX),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(control.stop(), Some(HtmlParseStop::Cancelled));
    }
    #[test]
    fn stop_constructors_return_the_stop_the_result_form_carries_and_latch_it() {
        let expected = HtmlParseStop::RepresentationExhausted {
            resource: HtmlParseResource::ManagedEdges,
        };
        let mut direct = control();
        assert_eq!(
            direct.stop_representation(HtmlParseResource::ManagedEdges),
            expected
        );
        assert_eq!(direct.stop(), Some(expected));
        let mut through_result = control();
        assert_eq!(
            through_result.representation::<()>(HtmlParseResource::ManagedEdges),
            Err(expected)
        );
        assert_eq!(through_result.stop(), Some(expected));

        let mut facts = control();
        assert_eq!(
            facts.stop_invalid_associated_facts(),
            HtmlParseStop::InvalidAssociatedFacts
        );
        assert_eq!(facts.stop(), Some(HtmlParseStop::InvalidAssociatedFacts));
    }
    #[test]
    fn stop_constructors_keep_the_first_stop() {
        let mut control = control();
        let first = control.stop_representation(HtmlParseResource::ManagedEdges);
        assert_eq!(
            control.stop_representation(HtmlParseResource::AuxiliaryEntries),
            first
        );
        assert_eq!(control.stop_invalid_associated_facts(), first);
        assert_eq!(
            control.latch_stop(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            )),
            first
        );
        assert_eq!(control.stop(), Some(first));

        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            cancelled.stop_invalid_associated_facts(),
            HtmlParseStop::Cancelled
        );
        assert_eq!(
            cancelled.stop_representation(HtmlParseResource::ManagedEdges),
            HtmlParseStop::Cancelled
        );
        assert_eq!(cancelled.stop(), Some(HtmlParseStop::Cancelled));
    }
    #[test]
    fn resource_and_work_tables_hold_each_key_at_its_discriminant() {
        let mut resources = ResourceTable::filled(u64::MAX);
        for resource in RESOURCES {
            *resources.get_mut(resource) = resource as u64;
        }
        assert_eq!(
            resources.into_array(),
            std::array::from_fn::<u64, { crate::settings::RESOURCE_COUNT }, _>(|index| index as u64)
        );
        let categories = [
            HtmlParseWorkCategory::InputUnitRead,
            HtmlParseWorkCategory::InspectByte,
            HtmlParseWorkCategory::CopyByte,
            HtmlParseWorkCategory::StateStep,
            HtmlParseWorkCategory::RecordVisit,
            HtmlParseWorkCategory::RecordWrite,
            HtmlParseWorkCategory::ReserveByte,
            HtmlParseWorkCategory::ProviderCall,
        ];
        let mut work = WorkTable::filled(u64::MAX);
        for category in categories {
            *work.get_mut(category) = category as u64;
        }
        assert_eq!(
            work.into_array(),
            std::array::from_fn::<u64, WORK_COUNT, _>(|index| index as u64)
        );
    }
    #[test]
    fn cap_failure_precedes_later_cancel() {
        let cancellation = HtmlParseCancellation::new();
        let mut limits = limits();
        limits = limits.into_builder().work_credits(0).finish().unwrap();
        let mut control = ParseControl::new(limits, cancellation.clone());
        let first = control
            .charge(HtmlParseWorkCategory::ProviderCall, 1)
            .unwrap_err();
        cancellation.cancel();
        assert_eq!(control.checkpoint(), Err(first));
    }
    #[test]
    fn all_work_categories_accumulate_without_refunds() {
        let mut control = control();
        for category in [
            HtmlParseWorkCategory::InputUnitRead,
            HtmlParseWorkCategory::InspectByte,
            HtmlParseWorkCategory::CopyByte,
            HtmlParseWorkCategory::StateStep,
            HtmlParseWorkCategory::RecordVisit,
            HtmlParseWorkCategory::RecordWrite,
            HtmlParseWorkCategory::ReserveByte,
            HtmlParseWorkCategory::ProviderCall,
        ] {
            control.charge(category, 2).unwrap();
            assert_eq!(control.usage().work(category), 2);
        }
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            16
        );
    }
    #[test]
    fn rejected_work_does_not_change_any_work_counter() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(3).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.charge(HtmlParseWorkCategory::StateStep, 2).unwrap();
        assert_eq!(
            control.charge(HtmlParseWorkCategory::InputUnitRead, 2),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 3,
                current: 2,
                requested_delta: 2
            })
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::InputUnitRead),
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            2
        );
    }
    #[test]
    fn exact_work_budget_succeeds_and_a_one_short_charge_is_refused_unchanged_and_sticky() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(10).finish().unwrap();
        for short in [false, true] {
            let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
            control.charge(HtmlParseWorkCategory::StateStep, 4).unwrap();
            let before = control.usage().clone();
            let request = 6 + u64::from(short);
            let result = control.charge(HtmlParseWorkCategory::RecordWrite, request);
            if short {
                let first = HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: 10,
                    current: 4,
                    requested_delta: 7,
                };
                // The refused charge applies nothing to any counter.
                assert_eq!(result, Err(first));
                assert_eq!(control.usage(), before);
                assert_eq!(control.stop(), Some(first));
                assert_eq!(control.first_stop_usage(), Some(&before));
                // The first stop is sticky: a request that would now fit, and
                // a different category, report the same stop and change nothing.
                assert_eq!(
                    control.charge(HtmlParseWorkCategory::StateStep, 1),
                    Err(first)
                );
                assert_eq!(
                    control.charge(HtmlParseWorkCategory::InputUnitRead, 7),
                    Err(first)
                );
                assert_eq!(control.usage(), before);
                assert_eq!(control.first_stop_usage(), Some(&before));
            } else {
                // Spending exactly the cap succeeds and records every credit.
                assert_eq!(result, Ok(()));
                let usage = control.usage();
                assert_eq!(usage.resource(HtmlParseResource::WorkCredits).current, 10);
                assert_eq!(usage.work(HtmlParseWorkCategory::RecordWrite), 6);
                assert_eq!(control.stop(), None);
                // The next credit is the first one over the cap.
                assert_eq!(
                    control.charge(HtmlParseWorkCategory::StateStep, 1),
                    Err(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        limit: 10,
                        current: 10,
                        requested_delta: 1,
                    })
                );
            }
        }
    }
    #[test]
    fn arithmetic_overflow_is_terminal_and_atomic() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control
            .charge(HtmlParseWorkCategory::StateStep, u64::MAX)
            .unwrap();
        assert_eq!(
            control.charge(HtmlParseWorkCategory::RecordVisit, 1),
            Err(HtmlParseStop::RepresentationExhausted {
                resource: HtmlParseResource::WorkCredits
            })
        );
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 0);
    }
    #[test]
    fn coupled_resources_fail_in_declared_order_without_partial_admission() {
        let mut limits = limits();
        limits = limits.into_builder().diagnostics(0).finish().unwrap();
        limits = limits
            .into_builder()
            .buffered_utf16_units(0)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        assert_eq!(
            control.acquire_many(&[
                (HtmlParseResource::Diagnostics, 1),
                (HtmlParseResource::BufferedUtf16Units, 1)
            ]),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::BufferedUtf16Units,
                limit: 0,
                current: 0,
                requested_delta: 1
            })
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::Diagnostics)
                .current,
            0
        );
    }
    // The raw controller keeps every storage resource terminal, `Diagnostics`
    // included. Parsing does not acquire it past the cap: the evidence owner
    // counts a dropped diagnostic instead (see `ParseEvidence::append`).
    #[test]
    fn raw_controller_acquire_past_the_diagnostics_cap_is_a_terminal_error() {
        let mut limits = limits();
        limits = limits.into_builder().diagnostics(1).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.acquire(HtmlParseResource::Diagnostics, 1).unwrap();
        assert!(matches!(
            control.acquire(HtmlParseResource::Diagnostics, 1),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            })
        ));
        assert!(control.checkpoint().is_err());
    }
    #[test]
    fn release_retains_high_water_and_managed_nodes_are_not_refunded() {
        let mut control = control();
        control.acquire(HtmlParseResource::OpenElements, 7).unwrap();
        control.release(HtmlParseResource::OpenElements, 5).unwrap();
        assert_eq!(
            control.usage().resource(HtmlParseResource::OpenElements),
            HtmlParseResourceUsage {
                current: 2,
                high_water: 7
            }
        );
        control.acquire(HtmlParseResource::ManagedNodes, 1).unwrap();
        assert_eq!(
            control.release(HtmlParseResource::ManagedNodes, 1),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonReclaimableNodes
            ))
        );
    }
    #[test]
    fn release_underflow_cannot_wrap() {
        let mut control = control();
        assert_eq!(
            control.release(HtmlParseResource::ManagedEdges, 1),
            Err(HtmlParseStop::Accounting(
                HtmlAccountingFault::ReleaseUnderflow
            ))
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
    }
    #[test]
    fn ancestry_depth_is_independent_of_open_stack() {
        let mut control = control();
        control.acquire(HtmlParseResource::OpenElements, 2).unwrap();
        control.observe_tree_depth(12).unwrap();
        control.observe_tree_depth(3).unwrap();
        assert_eq!(
            control.usage().resource(HtmlParseResource::TreeDepth),
            HtmlParseResourceUsage {
                current: 3,
                high_water: 12
            }
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::OpenElements)
                .current,
            2
        );
    }
    #[test]
    fn batches_bound_owned_work_and_poll_even_empty() {
        let mut control = control();
        assert_eq!(
            control
                .work_batch(HtmlParseWorkCategory::InspectByte, 1_000)
                .unwrap(),
            256
        );
        assert_eq!(
            control
                .work_batch(HtmlParseWorkCategory::InspectByte, 3)
                .unwrap(),
            3
        );
        control.cancellation.cancel();
        assert_eq!(
            control.work_batch(HtmlParseWorkCategory::InspectByte, 0),
            Err(HtmlParseStop::Cancelled)
        );
    }
    #[test]
    fn reservation_layout_rejects_unrepresentable_growth_without_allocator() {
        assert_eq!(
            ReservationPlan::for_vec(&Vec::<u16>::new(), usize::MAX),
            Err(HtmlParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedCapacityBytes
            })
        );
        assert_eq!(
            ReservationPlan::for_vec(&Vec::<()>::new(), 1),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ZeroSizedStorage
            ))
        );
    }
    #[test]
    fn capacity_limit_rejects_before_logical_write() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(1)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        assert!(matches!(
            buffer.try_push(&mut control, 0xD800),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            })
        ));
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn actual_capacity_and_overlap_are_recorded_and_cleanup_after_stop_works() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[0xD800, 0, 0xDC00])
            .unwrap();
        assert_eq!(buffer.as_slice(), &[0xD800, 0, 0xDC00]);
        assert_eq!(control.usage().actual_capacity_bytes, 6);
        buffer.try_push(&mut control, 0x61).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::PeakTransientBytes)
                .high_water,
            14
        );
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        assert_eq!(control.usage().largest_atomic_layout, 8);
        control.cancellation.cancel();
        control.checkpoint().unwrap_err();
        buffer.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().actual_capacity_high_water, 8);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units),
            HtmlParseResourceUsage {
                current: 0,
                high_water: 4
            }
        );
        assert_eq!(control.stop(), Some(HtmlParseStop::Cancelled));
    }
    #[test]
    fn clear_retains_capacity_and_freeze_transfer_retains_accounting() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut control, 7).unwrap();
        buffer.clear(&mut control).unwrap();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 1);
        buffer.try_push(&mut control, 9).unwrap();
        let vector = buffer.into_vec();
        assert_eq!(vector, [9]);
        assert_eq!(control.usage().actual_capacity_bytes, 2);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::ProviderCall), 1);
    }
    #[test]
    fn transient_and_atomic_caps_are_independent() {
        for resource in [
            HtmlParseResource::PeakTransientBytes,
            HtmlParseResource::AtomicLayoutBytes,
        ] {
            let mut limits = limits();
            if resource == HtmlParseResource::PeakTransientBytes {
                limits = limits
                    .into_builder()
                    .peak_transient_bytes(1)
                    .finish()
                    .unwrap();
            } else {
                limits = limits
                    .into_builder()
                    .atomic_layout_bytes(1)
                    .finish()
                    .unwrap();
            }
            let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
            let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
            assert!(
                matches!(buffer.try_push(&mut control, 1), Err(HtmlParseStop::LimitExceeded { resource: failed, .. }) if failed == resource)
            );
            assert_eq!(buffer.len(), 0);
            assert_eq!(buffer.capacity(), 0);
        }
    }
    #[test]
    fn verification_requires_admission_and_keeps_source_identity() {
        let source = HtmlSource::utf8("a");
        let mut missing = control();
        assert_eq!(
            missing.require_source(&source),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::MissingSource
            ))
        );
        let mut admitted = control();
        admitted.admit_source(&source).unwrap();
        admitted.require_source(&source.clone()).unwrap();
        assert_eq!(
            admitted.require_source(&HtmlSource::utf8("a")),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignSource
            ))
        );
        assert_eq!(
            admitted
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            1
        );
    }
    #[test]
    fn cursor_invariant_uses_the_same_first_terminal_owner() {
        let mut control = control();
        assert_eq!(
            control.fail_invariant(HtmlParseInvariant::InputCursorState),
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::InputCursorState)
        );
        control.cancellation.cancel();
        assert_eq!(
            control.fail_invariant(HtmlParseInvariant::ForeignSource),
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::InputCursorState)
        );
    }
    #[test]
    fn storage_cannot_be_mutated_with_a_different_control() {
        let mut owner = control();
        let mut other = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut owner, 9).unwrap();
        assert_eq!(
            buffer.try_push(&mut other, 10),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(buffer.as_slice(), &[9]);
        buffer.release(&mut owner).unwrap();
    }
    #[test]
    fn cleanup_underflow_is_checked_even_after_a_stop() {
        let mut control = control();
        let reason = control
            .cleanup_storage(HtmlParseResource::BufferedUtf16Units, 1, 2)
            .unwrap_err();
        assert_eq!(
            reason,
            HtmlParseStop::Accounting(HtmlAccountingFault::ReleaseUnderflow)
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn fixed_shared_payload_is_admitted_without_variable_copy() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut control, 0xD800).unwrap();
        let copied = control.usage().work(HtmlParseWorkCategory::CopyByte);
        let (vector, allocation) = control.admit_document_string_wrapper(buffer).unwrap();
        let wrapper = allocation.wrapper_bytes();
        assert_eq!(allocation.units(), 1);
        assert_eq!(allocation.vector_capacity(), 1);
        let text = DocumentString::from_utf16(vector);
        control.checkpoint().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(wrapper, size_of::<Vec<u16>>() as u64);
        assert_eq!(control.usage().actual_capacity_bytes, 2 + wrapper);
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::CopyByte),
            copied
        );
        control.cancellation.cancel();
        control.checkpoint().unwrap_err();
        drop(text);
        control.release_document_string(allocation).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn allocation_failure_is_fallible_and_does_not_admit_payload() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .peak_transient_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .atomic_layout_bytes(isize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut vector = Vec::<u8>::new();
        let plan = ReservationPlan::for_vec(&vector, isize::MAX as usize)
            .unwrap()
            .unwrap();
        control
            .charge(HtmlParseWorkCategory::ReserveByte, isize::MAX as u64)
            .unwrap();
        control
            .charge(HtmlParseWorkCategory::ProviderCall, 1)
            .unwrap();
        assert_eq!(
            control.reserve(&mut vector, plan, &mut plan.old_bytes.clone()),
            Err(HtmlParseStop::AllocationFailed {
                resource: HtmlParseResource::ManagedCapacityBytes,
                layout_bytes: isize::MAX as u64,
            })
        );
        assert_eq!(vector.capacity(), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn maximal_finite_work_and_valid_zero_caps_are_constructible() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .buffered_utf16_units(0)
            .finish()
            .unwrap();
        limits = limits.into_builder().source_units(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control
            .charge(HtmlParseWorkCategory::StateStep, u64::MAX)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            u64::MAX
        );
    }
    #[test]
    fn source_admission_is_one_metadata_visit_without_input_scanning() {
        let mut control = control();
        control.admit_source(&HtmlSource::utf8("abcdef")).unwrap();
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 1);
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::InputUnitRead),
            0
        );
        assert_eq!(control.usage().work(HtmlParseWorkCategory::InspectByte), 0);
    }
    #[test]
    fn every_storage_cap_rejects_its_own_growth() {
        for resource in RESOURCES.into_iter().filter(|resource| resource.storage()) {
            let mut control = control();
            control.test_limit(resource, 0);
            assert_eq!(
                control.acquire(resource, 1),
                Err(HtmlParseStop::LimitExceeded {
                    resource,
                    limit: 0,
                    current: 0,
                    requested_delta: 1,
                })
            );
            assert_eq!(control.usage().resource(resource).current, 0);
        }
    }
    #[test]
    fn non_storage_and_zero_sized_owned_vectors_are_rejected() {
        assert!(matches!(
            OwnedVec::<u16>::new(HtmlParseResource::SourceUnits),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource
            ))
        ));
        assert!(matches!(
            OwnedVec::<()>::new(HtmlParseResource::BufferedUtf16Units),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ZeroSizedStorage
            ))
        ));
        let mut control = control();
        assert_eq!(
            control.acquire(HtmlParseResource::WorkCredits, 1),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource
            ))
        );
    }
    #[test]
    fn control_identity_rejects_another_execution_even_for_the_same_source() {
        let source = HtmlSource::utf8("x");
        let mut first = control();
        let mut second = control();
        first.admit_source(&source).unwrap();
        second.admit_source(&source).unwrap();
        let identity = first.identity();
        first.require_identity(&identity).unwrap();
        assert_eq!(
            second.require_identity(&identity),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
    }
    #[test]
    fn frozen_allocation_ticket_cannot_release_a_different_owner() {
        let mut first = control();
        let mut second = control();
        let mut first_buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        first_buffer.try_push(&mut first, 4).unwrap();
        let (vector, ticket) = first.admit_document_string_wrapper(first_buffer).unwrap();
        let mut second_buffer =
            OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        second_buffer.try_push(&mut second, 4).unwrap();
        let (_, second_ticket) = second.admit_document_string_wrapper(second_buffer).unwrap();
        let previous = second.usage().clone();
        assert_eq!(
            second.release_document_string(ticket),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(second.usage(), previous);
        second.release_document_string(second_ticket).unwrap();
        drop(vector);
    }
    #[test]
    fn active_clear_charges_every_popped_slot_and_never_refunds_work() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[1, 2, 3])
            .unwrap();
        let previous = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let copy = control.usage().work(HtmlParseWorkCategory::CopyByte);
        buffer.clear(&mut control).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            previous + 12
        );
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 3);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordWrite), 3);
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::CopyByte),
            copy + 6
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units),
            HtmlParseResourceUsage {
                current: 0,
                high_water: 3
            }
        );
        assert_eq!(buffer.capacity(), 3);
    }
    #[test]
    fn failed_clear_leaves_remaining_entries_for_terminal_cleanup() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer
            .try_extend_from_slice(&mut control, &[1, 2, 3])
            .unwrap();
        let previous = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        control.test_limit(HtmlParseResource::WorkCredits, previous + 4);
        let stop = buffer.clear(&mut control).unwrap_err();
        assert_eq!(buffer.as_slice(), &[1, 2]);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            2
        );
        let work = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        buffer.release(&mut control).unwrap();
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn work_limit_precedes_storage_limit_for_a_growth_operation() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(3).finish().unwrap();
        limits = limits
            .into_builder()
            .buffered_utf16_units(0)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        assert!(matches!(
            buffer.try_push(&mut control, 1),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
    }
    #[test]
    fn indexed_access_preserves_bounds_and_charges_only_its_record_visit() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert_eq!(entries.get_mut(usize::MAX, &mut control).unwrap(), None);
        entries.try_push(&mut control, 7).unwrap();
        let before = control.usage().clone();
        let entry = entries.get_mut(0, &mut control).unwrap().unwrap();
        control
            .charge(HtmlParseWorkCategory::InspectByte, 2)
            .unwrap();
        assert_eq!(*entry, 7);
        control
            .charge_many(&[
                (HtmlParseWorkCategory::RecordWrite, 1),
                (HtmlParseWorkCategory::CopyByte, 2),
            ])
            .unwrap();
        *entry = 9;
        assert_eq!(entries.as_slice(), &[9]);
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::RecordVisit),
            before.work(HtmlParseWorkCategory::RecordVisit) + 1
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::InspectByte),
            before.work(HtmlParseWorkCategory::InspectByte) + 2
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::CopyByte),
            before.work(HtmlParseWorkCategory::CopyByte) + 2
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            before.resource(HtmlParseResource::AuxiliaryEntries)
        );
        assert_eq!(entries.get_mut(1, &mut control).unwrap(), None);
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::RecordVisit),
            before.work(HtmlParseWorkCategory::RecordVisit) + 2
        );
    }
    #[test]
    fn indexed_access_rejects_foreign_owner_before_a_visit() {
        let mut owner = control();
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        let before = foreign.usage().clone();
        assert_eq!(
            entries.get_mut(0, &mut foreign),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(foreign.usage(), before);
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut owner).unwrap();
    }
    #[test]
    fn cancellation_prevents_indexed_borrow_and_normal_pop() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, 7).unwrap();
        let before = control.usage().clone();
        control.cancellation.cancel();
        assert_eq!(
            entries.get_mut(0, &mut control),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(entries.try_pop(&mut control), Err(HtmlParseStop::Cancelled));
        assert_eq!(control.usage(), before);
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn indexed_borrow_work_cap_is_checked_before_exposing_the_slot() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(6).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, 7).unwrap();
        assert_eq!(
            entries.get_mut(0, &mut control),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 6,
                current: 6,
                requested_delta: 1,
            })
        );
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 0);
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn normal_pop_empty_attempt_visits_without_write_copy_or_allocation() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert_eq!(entries.try_pop(&mut control).unwrap(), None);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            1
        );
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 1);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordWrite), 0);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::CopyByte), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn normal_pop_transfers_last_slot_and_retains_capacity_and_high_water() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries
            .try_extend_from_slice(&mut control, &[3, 7])
            .unwrap();
        let before = control.usage().clone();
        let capacity = entries.capacity();
        assert_eq!(entries.try_pop(&mut control).unwrap(), Some(7));
        assert_eq!(entries.as_slice(), &[3]);
        assert_eq!(entries.capacity(), capacity);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            HtmlParseResourceUsage {
                current: 1,
                high_water: 2
            }
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before.resource(HtmlParseResource::WorkCredits).current + 4
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::RecordVisit),
            before.work(HtmlParseWorkCategory::RecordVisit) + 1
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::RecordWrite),
            before.work(HtmlParseWorkCategory::RecordWrite) + 1
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::CopyByte),
            before.work(HtmlParseWorkCategory::CopyByte) + 2
        );
        assert_eq!(
            control.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .high_water,
            2
        );
    }
    #[test]
    fn normal_pop_foreign_owner_and_work_cap_leave_the_value_in_place() {
        let mut limits = limits();
        limits = limits.into_builder().work_credits(6).finish().unwrap();
        let mut owner = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        assert_eq!(
            entries.try_pop(&mut foreign),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(
            foreign
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
        assert_eq!(
            entries.try_pop(&mut owner),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 6,
                current: 6,
                requested_delta: 4,
            })
        );
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(
            owner
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            1
        );
        entries.release(&mut owner).unwrap();
    }
    #[test]
    fn normal_pop_large_fixed_record_preserves_its_admitted_atomic_layout() {
        let mut control = control();
        let mut entries = OwnedVec::<[u8; 512]>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, [7; 512]).unwrap();
        let before = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        assert_eq!(entries.try_pop(&mut control).unwrap(), Some([7; 512]));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before + 514
        );
        assert_eq!(control.usage().largest_atomic_layout, 512);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AtomicLayoutBytes)
                .current,
            512
        );
        assert_eq!(entries.capacity(), 1);
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn active_cleanup_pop_is_rejected_before_removal() {
        let mut control = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut control, 7).unwrap();
        let before = control.usage().clone();
        assert_eq!(
            entries.cleanup_pop(&mut control),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CleanupWhileRunning
            ))
        );
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(control.usage(), before);
        assert_eq!(
            entries.try_pop(&mut control),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CleanupWhileRunning
            ))
        );
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn terminal_cleanup_transfers_nested_tickets_without_work_refund_or_resumption() {
        let mut control = control();
        let mut text = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        text.try_push(&mut control, 0xD800).unwrap();
        let (text, allocation) = control.admit_document_string_wrapper(text).unwrap();
        let text = DocumentString::from_utf16(text);
        let mut entries = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        entries.try_push(&mut control, (text, allocation)).unwrap();
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let before = control.usage().clone();
        let (text, allocation) = entries.cleanup_pop(&mut control).unwrap().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(
            control.usage().resource(HtmlParseResource::TokenAttributes),
            HtmlParseResourceUsage {
                current: 0,
                high_water: 1
            }
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(control.usage().work, before.work);
        assert_eq!(
            control.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        drop(text);
        control.release_document_string(allocation).unwrap();
        assert!(entries.cleanup_pop(&mut control).unwrap().is_none());
        assert!(matches!(entries.get_mut(0, &mut control), Err(reason) if reason == stop));
        assert!(matches!(entries.try_pop(&mut control), Err(reason) if reason == stop));
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
    }
    #[test]
    fn cleanup_pop_rejects_foreign_owner_without_changing_the_first_stop() {
        let mut owner = control();
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        foreign.cancellation.cancel();
        let stop = foreign.checkpoint().unwrap_err();
        let before = foreign.usage().clone();
        assert_eq!(entries.cleanup_pop(&mut foreign), Err(stop));
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(foreign.usage(), before);
        assert_eq!(foreign.stop(), Some(stop));
        owner.cancellation.cancel();
        owner.checkpoint().unwrap_err();
        assert_eq!(entries.cleanup_pop(&mut owner).unwrap(), Some(7));
        entries.release(&mut owner).unwrap();
    }
    #[test]
    fn recovering_push_has_the_same_success_accounting_as_normal_push() {
        let mut normal = control();
        let mut recovering = control();
        let mut normal_entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        let mut recovering_entries =
            OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        normal_entries.try_push(&mut normal, 7).unwrap();
        assert!(
            recovering_entries
                .try_push_recover(&mut recovering, 7)
                .is_ok()
        );
        assert_eq!(normal_entries.as_slice(), recovering_entries.as_slice());
        assert_eq!(normal.usage(), recovering.usage());
        normal_entries.release(&mut normal).unwrap();
        recovering_entries.release(&mut recovering).unwrap();
    }
    #[test]
    fn recovering_push_returns_nested_tickets_on_slot_cap_failure() {
        let mut limits = limits();
        limits = limits.into_builder().token_attributes(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut units = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        units.try_push(&mut control, 0xD800).unwrap();
        let (units, allocation) = control.admit_document_string_wrapper(units).unwrap();
        let text = DocumentString::from_utf16(units);
        let mut entries = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        let (stop, (text, allocation)) = entries
            .try_push_recover(&mut control, (text, allocation))
            .unwrap_err();
        assert_eq!(
            stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::TokenAttributes,
                limit: 0,
                current: 0,
                requested_delta: 1
            }
        );
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(entries.len(), 0);
        assert_eq!(entries.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        let work = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        drop(text);
        control.release_document_string(allocation).unwrap();
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), Some(stop));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
    }
    #[test]
    fn recovering_push_returns_the_value_on_cancel_foreign_owner_and_work_failure() {
        let mut owner = control();
        let mut foreign = control();
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        entries.try_push(&mut owner, 7).unwrap();
        assert_eq!(
            entries.try_push_recover(&mut foreign, 11),
            Err((
                HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl),
                11
            ))
        );
        owner.cancellation.cancel();
        assert_eq!(
            entries.try_push_recover(&mut owner, 13),
            Err((HtmlParseStop::Cancelled, 13))
        );
        assert_eq!(entries.as_slice(), &[7]);
        entries.release(&mut owner).unwrap();
        let mut limits = limits();
        limits = limits.into_builder().work_credits(0).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut entries = OwnedVec::<u16>::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert_eq!(
            entries.try_push_recover(&mut control, 17),
            Err((
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: 0,
                    current: 0,
                    requested_delta: 6
                },
                17
            ))
        );
        assert_eq!(entries.len(), 0);
        assert_eq!(entries.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
        entries.release(&mut control).unwrap();
    }
    #[test]
    fn normal_pop_transfers_nested_tickets_without_releasing_their_payload() {
        let mut control = control();
        let mut units = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        units.try_push(&mut control, 0xD800).unwrap();
        let (units, allocation) = control.admit_document_string_wrapper(units).unwrap();
        let mut entries = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        entries
            .try_push(
                &mut control,
                (DocumentString::from_utf16(units), allocation),
            )
            .unwrap();
        let bytes = control.usage().actual_capacity_bytes;
        let (text, allocation) = entries.try_pop(&mut control).unwrap().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        assert_eq!(control.usage().actual_capacity_bytes, bytes);
        drop(text);
        control.release_document_string(allocation).unwrap();
        entries.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_vector_reservation_tracks_growth_overlap_and_within_capacity_reuse() {
        let mut control = control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        assert_eq!(ticket.capacity(), 0);
        assert_eq!(vector.capacity(), 0);
        control
            .check_acquire_many(&[(HtmlParseResource::ManagedNodes, 2)])
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        control.reserve_vector(&mut vector, &mut ticket, 2).unwrap();
        assert_eq!(vector.capacity(), 2);
        assert_eq!(ticket.capacity(), 2);
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        control
            .acquire_many(&[(HtmlParseResource::ManagedNodes, 2)])
            .unwrap();
        control
            .charge_many(&[
                (HtmlParseWorkCategory::RecordWrite, 2),
                (HtmlParseWorkCategory::CopyByte, 8),
            ])
            .unwrap();
        vector.push(10);
        vector.push(20);
        control.reserve_vector(&mut vector, &mut ticket, 1).unwrap();
        assert_eq!(ticket.capacity(), 3);
        assert_eq!(vector.capacity(), 3);
        assert_eq!(vector, [10, 20]);
        assert_eq!(control.usage().actual_capacity_bytes, 12);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::PeakTransientBytes)
                .high_water,
            20
        );
        assert_eq!(control.usage().work(HtmlParseWorkCategory::ReserveByte), 28);
        let calls = control.usage().work(HtmlParseWorkCategory::ProviderCall);
        control.reserve_vector(&mut vector, &mut ticket, 1).unwrap();
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::ProviderCall),
            calls
        );
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedNodes),
            HtmlParseResourceUsage {
                current: 2,
                high_water: 2
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn actual_vector_ticket_rejects_foreign_owner_and_recovers_owned_storage() {
        let mut owner = control();
        let mut foreign = control();
        let (mut vector, mut ticket) = owner.new_vector::<u32>().unwrap();
        owner.reserve_vector(&mut vector, &mut ticket, 2).unwrap();
        let before = foreign.usage().clone();
        assert_eq!(
            foreign.reserve_vector(&mut vector, &mut ticket, 3),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(foreign.usage(), before);
        assert_eq!(ticket.capacity(), 2);
        let (stop, vector, ticket) = foreign
            .release_vector(vector, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl)
        );
        assert_eq!(vector.capacity(), 2);
        assert_eq!(ticket.capacity(), 2);
        owner.release_vector(vector, ticket).unwrap();
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_vector_ticket_checks_exact_capacity_before_growth_and_cleanup() {
        let mut control = control();
        let (mut first, mut first_ticket) = control.new_vector::<u32>().unwrap();
        let (mut second, mut second_ticket) = control.new_vector::<u32>().unwrap();
        control
            .reserve_vector(&mut first, &mut first_ticket, 1)
            .unwrap();
        control
            .reserve_vector(&mut second, &mut second_ticket, 2)
            .unwrap();
        let calls = control.usage().work(HtmlParseWorkCategory::ProviderCall);
        assert_eq!(
            control.reserve_vector(&mut second, &mut first_ticket, 3),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch
            ))
        );
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::ProviderCall),
            calls
        );
        assert_eq!(second.capacity(), 2);
        assert_eq!(first_ticket.capacity(), 1);
        let (_, second, first_ticket) = control
            .release_vector(second, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(control.usage().actual_capacity_bytes, 12);
        control.release_vector(first, first_ticket).unwrap();
        control.release_vector(second, second_ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn actual_vector_cancel_and_capacity_failure_do_not_grow_or_admit_nodes() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(3)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        assert_eq!(
            control.reserve_vector(&mut vector, &mut ticket, 1),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: 3,
                current: 0,
                requested_delta: 4,
            })
        );
        assert_eq!(ticket.capacity(), 0);
        assert_eq!(vector.capacity(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        control.release_vector(vector, ticket).unwrap();
        let mut control = super::tests::control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        control.cancellation.cancel();
        let before = control.usage().clone();
        assert_eq!(
            control.reserve_vector(&mut vector, &mut ticket, 1),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(control.usage(), before);
        assert_eq!(ticket.capacity(), 0);
        control.release_vector(vector, ticket).unwrap();
    }
    #[test]
    fn actual_allocation_failure_preserves_zero_capacity_ticket_for_cleanup() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .peak_transient_bytes(usize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .atomic_layout_bytes(isize::MAX as u64)
            .finish()
            .unwrap();
        limits = limits
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        let (mut vector, mut ticket) = control.new_vector::<u8>().unwrap();
        assert_eq!(
            control.reserve_vector(&mut vector, &mut ticket, isize::MAX as usize),
            Err(HtmlParseStop::AllocationFailed {
                resource: HtmlParseResource::ManagedCapacityBytes,
                layout_bytes: isize::MAX as u64,
            })
        );
        assert_eq!(ticket.capacity(), 0);
        assert_eq!(vector.capacity(), 0);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn ticket_refresh_records_actual_growth_even_when_post_reserve_phase_stops() {
        let mut control = control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        // Execute the real reservation phase, then deterministically cancel at
        // its post-call boundary before the production ticket-refresh phase.
        let plan = ReservationPlan::for_vec(&vector, 2).unwrap().unwrap();
        control
            .charge_many(&[
                (HtmlParseWorkCategory::ReserveByte, 8),
                (HtmlParseWorkCategory::ProviderCall, 1),
            ])
            .unwrap();
        control
            .reserve(&mut vector, plan, &mut ticket.admitted)
            .unwrap();
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        assert_eq!(
            ParseControl::record_vector_capacity(&vector, &mut ticket, Err(stop)),
            Err(stop)
        );
        assert_eq!(ticket.capacity(), 2);
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn logical_preflight_does_not_commit_and_final_admission_rechecks_the_boundary() {
        let mut limits = limits();
        limits = limits.into_builder().managed_nodes(1).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control
            .check_acquire_many(&[
                (HtmlParseResource::ManagedNodes, 1),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        control.acquire(HtmlParseResource::ManagedNodes, 1).unwrap();
        assert!(matches!(
            control.acquire_many(&[
                (HtmlParseResource::ManagedNodes, 1),
                (HtmlParseResource::ManagedEdges, 1)
            ]),
            Err(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedNodes,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
    }
    #[test]
    fn fixed_shared_payload_moves_actual_vector_and_tickets_without_copying_backing() {
        let mut control = control();
        let (mut vector, mut ticket) = control.new_vector::<u32>().unwrap();
        control.reserve_vector(&mut vector, &mut ticket, 2).unwrap();
        control.acquire(HtmlParseResource::ManagedNodes, 2).unwrap();
        control
            .charge_many(&[
                (HtmlParseWorkCategory::CopyByte, 8),
                (HtmlParseWorkCategory::RecordWrite, 2),
            ])
            .unwrap();
        vector.push(10);
        vector.push(20);
        let pointer = vector.as_ptr();
        let (shared, allocation) = control.share_owned(vector).unwrap();
        assert_eq!(shared.as_ptr(), pointer);
        assert_eq!(shared.as_slice(), &[10, 20]);
        assert_eq!(allocation.payload_bytes(), size_of::<Vec<u32>>() as u64);
        assert_eq!(
            control.usage().actual_capacity_bytes,
            8 + size_of::<Vec<u32>>() as u64
        );
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let vector = control.release_shared(shared, allocation).unwrap();
        assert_eq!(vector.as_ptr(), pointer);
        assert_eq!(vector, [10, 20]);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            2
        );
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn zero_sized_shared_identity_is_real_but_headers_are_not_fabricated_capacity() {
        let mut control = control();
        let (shared, ticket) = control.share_owned(()).unwrap();
        assert_eq!(Arc::strong_count(&shared), 1);
        assert_eq!(ticket.payload_bytes(), 0);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().work(HtmlParseWorkCategory::ProviderCall), 1);
        assert_eq!(control.release_shared(shared, ticket).unwrap(), ());
        assert!(matches!(
            control.new_vector::<()>(),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ZeroSizedStorage
            ))
        ));
    }
    #[test]
    fn shared_admission_cap_or_cancel_failure_returns_the_original_payload() {
        let mut limits = limits();
        limits = limits
            .into_builder()
            .managed_capacity_bytes(3)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        assert_eq!(
            control.share_owned(77_u32).unwrap_err(),
            (
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                    limit: 3,
                    current: 0,
                    requested_delta: 4,
                },
                77
            )
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        let mut control = super::tests::control();
        control.cancellation.cancel();
        assert_eq!(
            control.share_owned(91_u32).unwrap_err(),
            (HtmlParseStop::Cancelled, 91)
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
    }
    #[test]
    fn a_stop_after_shared_admission_refunds_the_charge_before_any_arc_exists() {
        let mut control = control();
        let bytes = control.admit_shared_layout::<u32>().unwrap();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        assert_eq!(control.usage().actual_capacity_bytes, bytes);
        control.cancellation.cancel();
        assert_eq!(
            control.confirm_shared_admission(bytes),
            Err(HtmlParseStop::Cancelled)
        );
        assert_eq!(control.stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().actual_capacity_high_water, 4);
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
    }
    #[test]
    fn shared_foreign_alias_and_wrong_payload_rejections_preserve_actual_ownership() {
        let mut owner = control();
        let mut foreign = control();
        let (shared, ticket) = owner.share_owned(77_u32).unwrap();
        let (stop, shared, ticket) = foreign
            .release_shared(shared, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl)
        );
        assert_eq!(owner.usage().actual_capacity_bytes, 4);
        let other_handle = shared.clone();
        let (stop, shared, ticket) = owner
            .release_shared(shared, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::SharedAllocationAliased)
        );
        assert_eq!(owner.usage().actual_capacity_bytes, 4);
        drop(other_handle);
        assert_eq!(owner.release_shared(shared, ticket).unwrap(), 77);
        let mut control = control();
        let (first, first_ticket) = control.share_owned(10_u32).unwrap();
        let (second, second_ticket) = control.share_owned(20_u32).unwrap();
        let (_, second, first_ticket) = control
            .release_shared(second, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(control.usage().actual_capacity_bytes, 8);
        assert_eq!(control.release_shared(first, first_ticket).unwrap(), 10);
        assert_eq!(control.release_shared(second, second_ticket).unwrap(), 20);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn terminal_logical_cleanup_requires_same_owner_and_never_refunds_nodes_or_work() {
        let mut control = control();
        let identity = control.identity();
        let foreign_identity = super::tests::control().identity();
        control
            .acquire_many(&[
                (HtmlParseResource::ManagedNodes, 2),
                (HtmlParseResource::AuxiliaryEntries, 3),
                (HtmlParseResource::ManagedEdges, 1),
            ])
            .unwrap();
        control.charge(HtmlParseWorkCategory::StateStep, 5).unwrap();
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let before = control.usage().clone();
        assert_eq!(
            control.cleanup_release(&foreign_identity, HtmlParseResource::AuxiliaryEntries, 1),
            Err(stop)
        );
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 4),
            Err(stop)
        );
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::ManagedNodes, 1),
            Err(stop)
        );
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::WorkCredits, 1),
            Err(stop)
        );
        assert_eq!(control.usage(), before);
        control
            .cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 3)
            .unwrap();
        control
            .cleanup_release(&identity, HtmlParseResource::ManagedEdges, 1)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            HtmlParseResourceUsage {
                current: 0,
                high_water: 3
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedEdges),
            HtmlParseResourceUsage {
                current: 0,
                high_water: 1
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedNodes),
            before.resource(HtmlParseResource::ManagedNodes)
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(control.stop(), Some(stop));
    }
    #[test]
    fn active_logical_cleanup_is_rejected_without_changing_admitted_counts() {
        let mut control = control();
        let identity = control.identity();
        control
            .acquire(HtmlParseResource::AuxiliaryEntries, 1)
            .unwrap();
        let before = control.usage().clone();
        assert_eq!(
            control.cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 1),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CleanupWhileRunning
            ))
        );
        assert_eq!(control.usage(), before);
        control
            .cleanup_release(&identity, HtmlParseResource::AuxiliaryEntries, 1)
            .unwrap();
    }
    #[test]
    fn fixed_record_layout_observation_is_separate_from_work_and_storage() {
        let mut control = control();
        let before = control.usage().clone();
        control.admit_atomic_layout(512).unwrap();
        control.admit_atomic_layout(128).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AtomicLayoutBytes),
            HtmlParseResourceUsage {
                current: 128,
                high_water: 512,
            }
        );
        assert_eq!(control.usage().largest_atomic_layout, 512);
        assert_eq!(control.usage().work, before.work);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.usage().actual_capacity_high_water, 0);
        for resource in RESOURCES {
            if resource != HtmlParseResource::AtomicLayoutBytes {
                assert_eq!(
                    control.usage().resource(resource),
                    before.resource(resource)
                );
            }
        }
    }
    #[test]
    fn fixed_record_layout_rejection_and_cancellation_observe_no_layout() {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .atomic_layout_bytes(511)
            .finish()
            .unwrap();
        let mut owner = ParseControl::new(caps, HtmlParseCancellation::new());
        let before = owner.usage().clone();
        let stop = owner.admit_atomic_layout(512).unwrap_err();
        assert_eq!(
            stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AtomicLayoutBytes,
                limit: 511,
                current: 0,
                requested_delta: 512,
            }
        );
        owner.cancellation.cancel();
        assert_eq!(owner.admit_atomic_layout(0), Err(stop));
        assert_eq!(owner.usage(), before);

        let mut owner = control();
        owner.cancellation.cancel();
        let before = owner.usage().clone();
        assert_eq!(owner.admit_atomic_layout(1), Err(HtmlParseStop::Cancelled));
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn fixed_record_layout_checks_representable_span_and_zero_layout() {
        let mut owner = control();
        owner.admit_atomic_layout(0).unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.admit_atomic_layout(usize::MAX),
            Err(HtmlParseStop::RepresentationExhausted {
                resource: HtmlParseResource::AtomicLayoutBytes,
            })
        );
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn checked_arithmetic_failure_latches_observation_and_preserves_first_stop() {
        let mut owner = control();
        owner.checkpoint().unwrap();
        let before = owner.usage().clone();
        // HtmlParseCancellation arrives after the arithmetic operation's initial poll.
        // The already observed failure remains the first terminal transition.
        owner.cancellation.cancel();
        let stop = HtmlParseStop::RepresentationExhausted {
            resource: HtmlParseResource::ManagedUtf16Units,
        };
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedUtf16Units),
            Err(stop)
        );
        assert_eq!(owner.checkpoint(), Err(stop));
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedEdges),
            Err(stop)
        );
        assert_eq!(owner.usage(), before);

        let mut owner = control();
        owner.cancellation.cancel();
        assert_eq!(owner.checkpoint(), Err(HtmlParseStop::Cancelled));
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedUtf16Units),
            Err(HtmlParseStop::Cancelled)
        );
    }
    #[test]
    fn zero_sized_shared_cleanup_allows_actual_alias_without_capacity_refund() {
        let mut owner = control();
        let (shared, allocation) = owner.share_owned(()).unwrap();
        let alias = shared.clone();
        assert_eq!(Arc::strong_count(&alias), 2);
        owner.cancellation.cancel();
        let stop = owner.checkpoint().unwrap_err();
        let before = owner.usage().clone();
        owner.release_zero_sized_shared(shared, allocation).unwrap();
        assert_eq!(Arc::strong_count(&alias), 1);
        assert_eq!(owner.usage(), before);
        assert_eq!(owner.stop(), Some(stop));
    }
    #[test]
    fn zero_sized_shared_cleanup_recovers_foreign_or_mismatched_actual_ownership() {
        let mut owner = control();
        let mut foreign = control();
        let (first, first_ticket) = owner.share_owned(()).unwrap();
        let (stop, first, first_ticket) = foreign
            .release_zero_sized_shared(first, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl)
        );
        assert_eq!(owner.stop(), None);
        let (second, second_ticket) = owner.share_owned(()).unwrap();
        let before = owner.usage().clone();
        let (stop, second, first_ticket) = owner
            .release_zero_sized_shared(second, first_ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::CapacityMismatch)
        );
        assert_eq!(owner.usage(), before);
        owner
            .release_zero_sized_shared(first, first_ticket)
            .unwrap();
        owner
            .release_zero_sized_shared(second, second_ticket)
            .unwrap();
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn zero_sized_cleanup_rejects_nonzero_payload_without_refunding_its_layout() {
        let mut owner = control();
        let (shared, allocation) = owner.share_owned(77_u32).unwrap();
        let before = owner.usage().clone();
        let (stop, shared, allocation) = owner
            .release_zero_sized_shared(shared, allocation)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::CapacityMismatch)
        );
        assert_eq!(owner.usage(), before);
        assert_eq!(owner.release_shared(shared, allocation).unwrap(), 77);
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
        assert_eq!(owner.usage().actual_capacity_high_water, 4);
        assert_eq!(owner.usage().work, before.work);
        assert_eq!(owner.stop(), Some(stop));
    }
    #[test]
    fn child_order_commit_changes_only_the_edge_counter() {
        let mut owner = control();
        let identity = owner.identity();
        owner
            .acquire_many(&[
                (HtmlParseResource::ManagedEdges, 2),
                (HtmlParseResource::ManagedNodes, 4),
            ])
            .unwrap();
        owner
            .charge_many(&[
                (HtmlParseWorkCategory::RecordWrite, 3),
                (HtmlParseWorkCategory::CopyByte, 24),
            ])
            .unwrap();
        let before = owner.usage().clone();
        owner.commit_construction_edges(&identity, 2, 1).unwrap();
        assert_eq!(
            owner.usage().resource(HtmlParseResource::ManagedEdges),
            HtmlParseResourceUsage {
                current: 3,
                high_water: 3,
            }
        );
        for resource in RESOURCES {
            if resource != HtmlParseResource::ManagedEdges {
                assert_eq!(owner.usage().resource(resource), before.resource(resource));
            }
        }
        assert_eq!(owner.usage().work, before.work);
        assert_eq!(owner.stop(), None);
    }
    #[test]
    fn child_order_same_parent_move_can_commit_at_edge_cap_without_transient_growth() {
        let mut caps = limits();
        caps = caps.into_builder().managed_edges(2).finish().unwrap();
        let mut owner = ParseControl::new(caps, HtmlParseCancellation::new());
        let identity = owner.identity();
        owner
            .acquire_many(&[(HtmlParseResource::ManagedEdges, 2)])
            .unwrap();
        owner.commit_construction_edges(&identity, 1, 1).unwrap();
        assert_eq!(
            owner.usage().resource(HtmlParseResource::ManagedEdges),
            HtmlParseResourceUsage {
                current: 2,
                high_water: 2,
            }
        );
        let before = owner.usage().clone();
        owner.commit_construction_edges(&identity, 0, 0).unwrap();
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn child_order_cap_failure_retains_edges_and_first_stop() {
        let mut caps = limits();
        caps = caps.into_builder().managed_edges(1).finish().unwrap();
        let mut owner = ParseControl::new(caps, HtmlParseCancellation::new());
        let identity = owner.identity();
        owner
            .acquire_many(&[(HtmlParseResource::ManagedEdges, 1)])
            .unwrap();
        let before = owner.usage().clone();
        let stop = HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedEdges,
            limit: 1,
            current: 1,
            requested_delta: 1,
        };
        assert_eq!(owner.commit_construction_edges(&identity, 1, 0), Err(stop));
        assert_eq!(owner.usage(), before);
        owner.cancellation.cancel();
        assert_eq!(owner.commit_construction_edges(&identity, 0, 0), Err(stop));
        assert_eq!(owner.usage(), before);
        assert_eq!(owner.stop(), Some(stop));
    }
    #[test]
    fn child_order_foreign_owner_rejects_before_any_counter_commit() {
        let mut owner = control();
        let foreign_identity = control().identity();
        owner
            .acquire_many(&[(HtmlParseResource::ManagedEdges, 1)])
            .unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.commit_construction_edges(&foreign_identity, 0, 1),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn child_order_underflow_rejects_the_counter_before_edge_arithmetic() {
        let mut owner = control();
        let identity = owner.identity();
        owner
            .acquire_many(&[(HtmlParseResource::ManagedEdges, 1)])
            .unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.commit_construction_edges(&identity, u64::MAX, 2),
            Err(HtmlParseStop::Accounting(
                HtmlAccountingFault::ReleaseUnderflow
            ))
        );
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn child_order_edge_addition_overflow_retains_existing_edges() {
        let mut owner = control();
        let identity = owner.identity();
        owner
            .acquire_many(&[(HtmlParseResource::ManagedEdges, 1)])
            .unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.commit_construction_edges(&identity, u64::MAX, 0),
            Err(HtmlParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedEdges,
            })
        );
        assert_eq!(owner.usage(), before);
    }
    #[test]
    fn child_order_final_checkpoint_cancellation_precedes_validation_even_for_empty_commit() {
        for (addition, removal) in [(0, 0), (u64::MAX, 2)] {
            let mut owner = control();
            let identity = owner.identity();
            owner
                .acquire_many(&[(HtmlParseResource::ManagedEdges, 1)])
                .unwrap();
            owner.cancellation.cancel();
            let before = owner.usage().clone();
            assert_eq!(
                owner.commit_construction_edges(&identity, addition, removal),
                Err(HtmlParseStop::Cancelled)
            );
            assert_eq!(owner.usage(), before);
            assert_eq!(
                owner.commit_construction_edges(&identity, 0, 1),
                Err(HtmlParseStop::Cancelled)
            );
            assert_eq!(owner.usage(), before);
        }
    }
    /// The colour operation error CSS itself reports for `input` when every
    /// colour fact is absent.
    fn css_color_error(input: &[u8]) -> surgeist_css::color::CssColorError {
        let facts = surgeist_css::color::CssColorFacts::new(
            surgeist_css::color::CssColorFactIdentity::new(7, 3),
            surgeist_css::color::CssColorPropertyFacts::Unavailable,
            surgeist_css::color::CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            surgeist_css::color::CssDeviceColorFact::Unknown,
        );
        surgeist_css::color::parse_used_color(
            surgeist_css::color::CssColorInput::Utf8(input),
            surgeist_css::color::CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            surgeist_css::color::CssColorOrdinaryLimits::default(),
        )
        .err()
        .and_then(|error| error.color_error().cloned())
        .expect("CSS reports a colour operation error")
    }
    #[test]
    fn source_color_error_keeps_its_kind_and_original_source_binding() {
        let mut owner = control();
        let source = HtmlSource::utf16(surgeist_html::values::DocumentString::from(
            "<input type=color value=red>",
        ));
        owner.admit_source(&source).unwrap();
        let unavailable = surgeist_css::color::CssColorUnavailable::CurrentColor;
        let stop = owner.stop_color_processing(css_color_error(b"currentcolor"));
        assert_eq!(
            stop,
            HtmlParseStop::ColorProcessing(HtmlColorProcessingError::Unavailable(unavailable))
        );
        let observed = owner.first_stop_usage().unwrap().clone();
        owner.cancellation.cancel();
        assert_eq!(
            owner.stop_color_processing(css_color_error(b"canvas")),
            stop
        );
        assert_eq!(owner.first_stop_usage(), Some(&observed));
        let binding = owner.take_stopped_source_binding().unwrap();
        assert_eq!(&binding.0, source.identity());
        assert_eq!(binding.1, source.coordinates());
        assert!(owner.stopped_source_binding().unwrap().is_none());
    }
    #[test]
    fn malformed_input_from_the_utf16_adapter_is_a_color_conversion_invariant() {
        // CSS Syntax preprocessing decodes every UTF-16 unit, a lone surrogate
        // as U+FFFD, so the adapter's input is never malformed for its encoding.
        let mut owner = control();
        assert_eq!(
            owner.stop_color_processing(css_color_error(b"r\xffd")),
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::ColorConversionState)
        );
    }
}
