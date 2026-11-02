//! Private accounting owner for parsing. Counts describe admitted physical work
//! and owned storage, not CPU instructions, elapsed time, allocator metadata or RSS.

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
mod text_append;
pub(crate) use construction::ConstructionAllocation;

const WORK_COUNT: usize = 8;
/// One value per [`WorkCategory`]; a lookup cannot be out of range.
type WorkTable<T> = Table<WorkCategory, T, WORK_COUNT>;
pub(crate) const WORK_QUANTUM: usize = 256;

/// Each category has weight one; units are physical bytes or individual events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum WorkCategory {
    InputUnitRead,
    InspectByte,
    CopyByte,
    StateStep,
    RecordVisit,
    RecordWrite,
    ReserveByte,
    ProviderCall,
}
slots!(WorkCategory, WORK_COUNT, {
    InputUnitRead => 0,
    InspectByte => 1,
    CopyByte => 2,
    StateStep => 3,
    RecordVisit => 4,
    RecordWrite => 5,
    ReserveByte => 6,
    ProviderCall => 7,
});

/// A state the parser's own sequencing rules out, reported instead of a panic.
///
/// No valid use produces one: each variant names the part of the parser whose
/// contract was found broken, so a report can say which part to read. The parser
/// latches the stop first, hands back every owner it holds and releases all
/// charged usage. The enum grows as more states are classified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Invariant {
    /// A source other than the one bound to the parse was presented.
    ForeignSource,
    /// A storage-only accounting operation named a resource that is not storage.
    NonStorageResource,
    /// A release returned more units than the parse currently holds.
    ReleaseUnderflow,
    /// A node release was requested for nodes, which are never reclaimed one by
    /// one.
    NonReclaimableNodes,
    /// A buffer's actual capacity differs from the capacity that was charged.
    CapacityMismatch,
    /// Storage was requested for a zero-sized element type.
    ZeroSizedStorage,
    /// A stopped parse has no source binding where one was admitted.
    MissingSource,
    /// The input cursor (decoded input, position, checkpoint, pending-input
    /// poll or consumed boundary) is in a state its own contract excludes.
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
    /// A construction session was used while form effects were still pending.
    PendingFormEffects,
    /// A creation that holds a form, link, select or details association
    /// failed with an error other than a stop.
    IncompleteFormAssociationCreation,
    /// A construction registry row, record, child or payload that internal
    /// state names is missing, out of range or of another kind, or a
    /// construction owner that the registry keeps in a slot is not there.
    RegistryRow,
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
    /// absent, or a character-reference table lookup is out of range.
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
pub enum ParseStop {
    Cancelled,
    CspProcessing,
    CspProtocol(surgeist_html::HtmlCspProtocolError),
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
    InternalInvariant(Invariant),
    UnsupportedStandaloneState(StandaloneUnsupported),
    InvalidAssociatedFacts,
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
pub enum StandaloneUnsupported {
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
pub struct ResourceUsage {
    pub(crate) current: u64,
    pub(crate) high_water: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseUsage {
    resources: ResourceTable<ResourceUsage>,
    work: WorkTable<u64>,
    /// Actual capacities can exceed requested layouts. This is observed after
    /// allocation; no promise is made about preventing allocator overshoot.
    pub(crate) actual_capacity_bytes: u64,
    pub(crate) actual_capacity_high_water: u64,
    /// Largest indivisible reserve/copy submission (not a time bound).
    pub(crate) largest_atomic_layout: u64,
}
impl ParseUsage {
    pub fn resource(&self, resource: HtmlParseResource) -> ResourceUsage {
        self.resources.copy(resource)
    }
    pub fn work(&self, category: WorkCategory) -> u64 {
        self.work.copy(category)
    }
}

/// The first latched stop and the usage observed at that moment.
struct FirstStop {
    reason: ParseStop,
    usage: ParseUsage,
}

/// Exclusively mutable owner. Only the cancellation handle can be cloned.
pub(crate) struct ParseControl {
    limits: ResourceTable<u64>,
    usage: ParseUsage,
    cancellation: HtmlParseCancellation,
    // The first stop and its fixed usage observation, captured together before
    // any producer cleanup, so neither can exist without the other.
    stopped: Option<FirstStop>,
    source: Option<(HtmlSourceId, HtmlSourceCoordinates)>,
    identity: Arc<()>,
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

impl ParseControl {
    pub(crate) fn new(limits: HtmlParseLimits, cancellation: HtmlParseCancellation) -> Self {
        let limits = limits.values();
        Self {
            limits,
            usage: ParseUsage {
                resources: ResourceTable::filled(ResourceUsage::default()),
                work: WorkTable::filled(0),
                actual_capacity_bytes: 0,
                actual_capacity_high_water: 0,
                largest_atomic_layout: 0,
            },
            cancellation,
            stopped: None,
            source: None,
            identity: Arc::new(()),
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
        }
    }
    pub(crate) fn usage(&self) -> &ParseUsage {
        &self.usage
    }
    pub(crate) fn identity(&self) -> ControlIdentity {
        ControlIdentity(self.identity.clone())
    }
    pub(crate) fn require_identity(&mut self, identity: &ControlIdentity) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if Arc::ptr_eq(&identity.0, &self.identity) {
            Ok(())
        } else {
            self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl))
        }
    }
    pub(crate) fn stop(&self) -> Option<ParseStop> {
        self.stopped.as_ref().map(|first| first.reason)
    }
    pub(crate) fn first_stop_usage(&self) -> Option<&ParseUsage> {
        self.stopped.as_ref().map(|first| &first.usage)
    }
    /// The first stop with the usage observed when it was latched; one test
    /// answers both, since they exist together or not at all.
    pub(crate) fn stopped(&self) -> Option<(ParseStop, &ParseUsage)> {
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
    ) -> Result<Option<&(HtmlSourceId, HtmlSourceCoordinates)>, ParseStop> {
        if self.stopped.is_none() {
            return Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning));
        }
        Ok(self.source.as_ref())
    }
    /// Cleanup-only move of the actual optional binding, including a prefix
    /// stopped before source admission. The caller preflights its owning slots.
    pub(crate) fn take_stopped_optional_source_binding(
        &mut self,
    ) -> Result<Option<(HtmlSourceId, HtmlSourceCoordinates)>, ParseStop> {
        self.stopped_source_binding()?;
        Ok(self.source.take())
    }
    /// A stopped, cleaned consumer moves the already-admitted source binding.
    /// Refusal leaves it intact; no polling, charging or handle clone follows
    /// stop. The caller retains this controller for diagnostic ticket disposal.
    pub(crate) fn take_stopped_source_binding(
        &mut self,
    ) -> Result<(HtmlSourceId, HtmlSourceCoordinates), ParseStop> {
        if self.stopped.is_none() {
            return Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning));
        }
        self.source
            .take()
            .ok_or(ParseStop::InternalInvariant(Invariant::MissingSource))
    }
    pub(crate) fn csp_processing_stop(&mut self) -> ParseStop {
        self.latch_stop(ParseStop::CspProcessing)
    }
    pub(crate) fn csp_protocol_stop(
        &mut self,
        error: surgeist_html::HtmlCspProtocolError,
    ) -> ParseStop {
        self.latch_stop(ParseStop::CspProtocol(error))
    }
    pub(crate) fn cancelled_stop(&mut self) -> ParseStop {
        self.latch_stop(ParseStop::Cancelled)
    }
    fn latch_stop(&mut self, reason: ParseStop) -> ParseStop {
        if let Some(first) = &self.stopped {
            return first.reason;
        }
        // No allocation or useful progression: the fixed control layout owns
        // this observation. Denied requests remain in the stop, not in usage.
        self.stopped = Some(FirstStop {
            reason,
            usage: self.usage.clone(),
        });
        reason
    }
    pub(crate) fn fail_invariant(&mut self, invariant: Invariant) -> ParseStop {
        if let Err(reason) = self.checkpoint() {
            return reason;
        }
        self.latch_stop(ParseStop::InternalInvariant(invariant))
    }
    pub(crate) fn stop_unsupported_standalone(
        &mut self,
        reason: StandaloneUnsupported,
    ) -> ParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(ParseStop::UnsupportedStandaloneState(reason))
    }
    pub(crate) fn stop_color_processing(
        &mut self,
        reason: surgeist_css::color::CssColorError,
    ) -> ParseStop {
        let Some(error) = HtmlColorProcessingError::from_source(reason) else {
            return self.fail_invariant(Invariant::ColorConversionState);
        };
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(ParseStop::ColorProcessing(error))
    }
    pub(crate) fn stop_invalid_associated_facts(&mut self) -> ParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(ParseStop::InvalidAssociatedFacts)
    }
    pub(crate) fn stop_base_facts(&mut self, reason: crate::HtmlBaseMissing) -> ParseStop {
        if let Err(first) = self.checkpoint() {
            return first;
        }
        self.latch_stop(ParseStop::BaseFacts(reason))
    }
    fn fail<T>(&mut self, reason: ParseStop) -> Result<T, ParseStop> {
        Err(self.latch_stop(reason))
    }
    pub(crate) fn checkpoint(&mut self) -> Result<(), ParseStop> {
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
            return self.fail(ParseStop::Cancelled);
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
        let bytes = slots.saturating_mul(2 * size_of::<crate::Attribute>());
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
    pub(crate) fn stop_representation(&mut self, resource: HtmlParseResource) -> ParseStop {
        self.latch_stop(ParseStop::RepresentationExhausted { resource })
    }
    /// The same latched stop as [`Self::stop_representation`], for a caller
    /// whose own result type is `Result`.
    pub(crate) fn representation<T>(
        &mut self,
        resource: HtmlParseResource,
    ) -> Result<T, ParseStop> {
        Err(self.stop_representation(resource))
    }
    /// Admits one known indivisible owned-record layout before its move/copy.
    /// The caller separately precharges actual bytes and record writes. This
    /// observes layout only; it creates no allocation or cumulative work credit.
    pub(crate) fn admit_atomic_layout(&mut self, bytes: usize) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if bytes > isize::MAX as usize {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        }
        let Ok(bytes) = u64::try_from(bytes) else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
        self.set(HtmlParseResource::AtomicLayoutBytes, bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
        Ok(())
    }
    fn check_value(
        &mut self,
        resource: HtmlParseResource,
        value: u64,
        delta: u64,
    ) -> Result<(), ParseStop> {
        let limit = self.limits.copy(resource);
        if value > limit {
            return self.fail(ParseStop::LimitExceeded {
                resource,
                limit,
                current: self.usage.resource(resource).current,
                requested_delta: delta,
            });
        }
        Ok(())
    }
    fn set(&mut self, resource: HtmlParseResource, value: u64) {
        let usage = self.usage.resources.get_mut(resource);
        usage.current = value;
        usage.high_water = usage.high_water.max(value);
    }
    pub(crate) fn admit_source(&mut self, source: &HtmlSource) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge(WorkCategory::RecordVisit, 1)?;
        if let Some((identity, coordinates)) = &self.source {
            return if identity == source.identity() && *coordinates == source.coordinates() {
                Ok(())
            } else {
                self.fail(ParseStop::InternalInvariant(Invariant::ForeignSource))
            };
        }
        let Ok(units) = u64::try_from(source.len()) else {
            return self.representation(HtmlParseResource::SourceUnits);
        };
        self.check_value(HtmlParseResource::SourceUnits, units, units)?;
        self.set(HtmlParseResource::SourceUnits, units);
        self.source = Some((source.identity().clone(), source.coordinates()));
        Ok(())
    }
    pub(crate) fn require_source(&mut self, source: &HtmlSource) -> Result<(), ParseStop> {
        self.checkpoint()?;
        match &self.source {
            Some((identity, coordinates))
                if identity == source.identity() && *coordinates == source.coordinates() =>
            {
                Ok(())
            }
            Some(_) => self.fail(ParseStop::InternalInvariant(Invariant::ForeignSource)),
            None => self.fail(ParseStop::InternalInvariant(Invariant::MissingSource)),
        }
    }
    pub(crate) fn require_source_range(
        &mut self,
        range: &crate::HtmlSourceRange,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::ProviderCall, 1),
        ])?;
        match &self.source {
            Some((identity, coordinates))
                if identity == range.source() && *coordinates == range.coordinates() =>
            {
                Ok(())
            }
            Some(_) => self.fail(ParseStop::InternalInvariant(Invariant::ForeignSource)),
            None => self.fail(ParseStop::InternalInvariant(Invariant::MissingSource)),
        }
    }
    pub(crate) fn charge(&mut self, category: WorkCategory, units: u64) -> Result<(), ParseStop> {
        self.charge_many(&[(category, units)])
    }
    fn charge_many(&mut self, charges: &[(WorkCategory, u64)]) -> Result<(), ParseStop> {
        self.checkpoint()?;
        let mut work = self.usage.work;
        let mut increment = 0_u64;
        for &(category, units) in charges {
            let (Some(next), Some(total)) = (
                work.copy(category).checked_add(units),
                increment.checked_add(units),
            ) else {
                return self.representation(HtmlParseResource::WorkCredits);
            };
            *work.get_mut(category) = next;
            increment = total;
        }
        let Some(total) = self
            .usage
            .resource(HtmlParseResource::WorkCredits)
            .current
            .checked_add(increment)
        else {
            return self.representation(HtmlParseResource::WorkCredits);
        };
        self.check_value(HtmlParseResource::WorkCredits, total, increment)?;
        self.usage.work = work;
        self.set(HtmlParseResource::WorkCredits, total);
        Ok(())
    }
    /// Owned scans/loops must execute only the returned span before calling again.
    /// Allocations are separately precharged indivisible provider submissions.
    #[cfg(test)]
    pub(crate) fn work_batch(
        &mut self,
        category: WorkCategory,
        remaining: usize,
    ) -> Result<usize, ParseStop> {
        let batch = remaining.min(WORK_QUANTUM);
        self.charge(category, batch as u64)?;
        Ok(batch)
    }
    fn plan_storage(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<ResourceTable<u64>, ParseStop> {
        self.checkpoint()?;
        let mut values = self.usage.resources.map(|usage| usage.current);
        let mut deltas = ResourceTable::filled(0_u64);
        for &(resource, delta) in requests {
            if !resource.storage() {
                return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
            }
            let (Some(value), Some(sum)) = (
                values.copy(resource).checked_add(delta),
                deltas.copy(resource).checked_add(delta),
            ) else {
                return self.representation(resource);
            };
            *values.get_mut(resource) = value;
            *deltas.get_mut(resource) = sum;
        }
        for resource in RESOURCES {
            if resource.storage() {
                self.check_value(resource, values.copy(resource), deltas.copy(resource))?;
            }
        }
        Ok(values)
    }
    pub(crate) fn acquire(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), ParseStop> {
        self.acquire_many(&[(resource, units)])
    }
    /// Coupled logical allocations admit atomically in resource precedence order.
    pub(crate) fn acquire_many(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), ParseStop> {
        let values = self.plan_storage(requests)?;
        for resource in RESOURCES {
            if resource.storage() {
                self.set(resource, values.copy(resource));
            }
        }
        Ok(())
    }
    /// Admits an immutable provider graph retained by this original owner.
    /// Provider-computed bytes include its backing capacities, rather than a
    /// second inferred layout. Logical extents and capacity commit atomically.
    pub(crate) fn acquire_retained_provider(
        &mut self,
        bytes: u64,
        units: u64,
        entries: u64,
    ) -> Result<(), ParseStop> {
        let values = self.plan_storage(&[
            (HtmlParseResource::ManagedUtf16Units, units),
            (HtmlParseResource::AuxiliaryEntries, entries),
        ])?;
        let Some(total) = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            .checked_add(bytes)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.check_value(HtmlParseResource::ManagedCapacityBytes, total, bytes)?;
        self.check_value(HtmlParseResource::PeakTransientBytes, total, bytes)?;
        for resource in RESOURCES {
            if resource.storage() {
                self.set(resource, values.copy(resource));
            }
        }
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        Ok(())
    }
    /// Ownership disposal after the retained provider handle has been dropped.
    /// This does not execute source work or observe a later cancellation.
    pub(crate) fn release_retained_provider(
        &mut self,
        owner: &ControlIdentity,
        bytes: u64,
        units: u64,
        entries: u64,
    ) -> Result<(), ParseStop> {
        if !Arc::ptr_eq(&owner.0, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        self.commit_prepared_release(&Self::provider_release(bytes, units, entries))
    }
    /// Whether [`Self::release_retained_provider`] of this storage by this
    /// controller's own identity would be accepted. Nothing changes.
    pub(crate) fn can_release_retained_provider(
        &self,
        bytes: u64,
        units: u64,
        entries: u64,
    ) -> bool {
        self.after_release(&Self::provider_release(bytes, units, entries))
            .is_some()
    }
    fn provider_release(bytes: u64, units: u64, entries: u64) -> [(HtmlParseResource, u64); 4] {
        [
            (HtmlParseResource::ManagedCapacityBytes, bytes),
            (HtmlParseResource::PeakTransientBytes, bytes),
            (HtmlParseResource::ManagedUtf16Units, units),
            (HtmlParseResource::AuxiliaryEntries, entries),
        ]
    }
    /// Nonmutating logical preflight before actual storage reservations. The
    /// session rechecks with `acquire_many` immediately before graph mutation;
    /// no logical acquisition may intervene between that commit and mutation.
    pub(crate) fn check_acquire_many(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), ParseStop> {
        self.plan_storage(requests).map(|_| ())
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
    ) -> Result<(), ParseStop> {
        self.require_identity(owner)?;
        let Some(retained_edges) = self
            .usage
            .resource(HtmlParseResource::ManagedEdges)
            .current
            .checked_sub(edge_remove)
        else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        let Some(edges) = retained_edges.checked_add(edge_add) else {
            return self.representation(HtmlParseResource::ManagedEdges);
        };
        self.check_value(HtmlParseResource::ManagedEdges, edges, edge_add)?;
        self.set(HtmlParseResource::ManagedEdges, edges);
        Ok(())
    }
    /// Produces the actual empty vector and its unique accounting ticket.
    /// This creates no allocation or logical node/edge/storage admission.
    pub(crate) fn new_vector<T>(&mut self) -> Result<(Vec<T>, VectorCapacity<T>), ParseStop> {
        self.checkpoint()?;
        if size_of::<T>() == 0 {
            return self.fail(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage));
        }
        let Some(bytes) = size_of::<Vec<T>>()
            .checked_add(size_of::<VectorCapacity<T>>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.charge_many(&[
            (WorkCategory::RecordWrite, 2),
            (WorkCategory::CopyByte, bytes),
        ])?;
        Ok((
            Vec::new(),
            VectorCapacity {
                owner: self.identity.clone(),
                capacity: 0,
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
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        self.charge(WorkCategory::RecordVisit, 2)?;
        if allocation.capacity != vector.capacity() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
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
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        let result = self.reserve(vector, plan);
        // Successful reserve can change capacity before a post-call stop. That
        // physical ownership must remain releasable even when logical work ends.
        Self::record_vector_capacity(vector, allocation, result)
    }
    /// Local append-owner growth. Other callers retain their exact policy.
    /// Spare capacity is submitted and charged, never counted as live entries.
    pub(crate) fn reserve_vector_geometric<T>(
        &mut self,
        vector: &mut Vec<T>,
        allocation: &mut VectorCapacity<T>,
        additional: usize,
    ) -> Result<(), ParseStop> {
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
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        self.charge(WorkCategory::RecordVisit, 2)?;
        if allocation.capacity != vector.capacity() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
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
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        let result = self.reserve(vector, plan);
        Self::record_vector_capacity(vector, allocation, result)
    }
    fn geometric_plan<T>(
        &self,
        vector: &Vec<T>,
        additional: usize,
    ) -> Result<Option<ReservationPlan>, ParseStop> {
        self.geometric_capacity::<T>(vector.len(), vector.capacity(), additional)
    }
    fn geometric_plan_toward<T>(
        &self,
        vector: &Vec<T>,
        additional: usize,
        final_len: usize,
    ) -> Result<Option<ReservationPlan>, ParseStop> {
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
    ) -> Result<Option<ReservationPlan>, ParseStop> {
        self.geometric_sized(size_of::<T>(), len, capacity, additional, usize::MAX)
    }
    /// Geometric growth of a bank whose element layout is only known by size.
    /// Spare capacity beyond `final_len` slots is never planned.
    fn geometric_sized(
        &self,
        element_size: usize,
        len: usize,
        capacity: usize,
        additional: usize,
        final_len: usize,
    ) -> Result<Option<ReservationPlan>, ParseStop> {
        Ok(self
            .geometric_pair(element_size, len, capacity, additional, final_len)?
            .map(|(_, geometric)| geometric))
    }
    /// The exact plan for `additional` slots and the geometric plan that
    /// contains it, or `None` when the capacity already suffices. The two are
    /// the same plan when the exact layout cannot fit the limits.
    fn geometric_pair(
        &self,
        element_size: usize,
        len: usize,
        capacity: usize,
        additional: usize,
        final_len: usize,
    ) -> Result<Option<(ReservationPlan, ReservationPlan)>, ParseStop> {
        let Some(exact) = ReservationPlan::for_sized(element_size, len, capacity, additional)?
        else {
            return Ok(None);
        };
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let without_old = current
            .checked_sub(exact.old_bytes)
            .ok_or(ParseStop::InternalInvariant(Invariant::CapacityMismatch))?;
        let ceiling = (isize::MAX as u64)
            .min(
                self.limits
                    .copy(HtmlParseResource::ManagedCapacityBytes)
                    .saturating_sub(without_old),
            )
            .min(
                self.limits
                    .copy(HtmlParseResource::PeakTransientBytes)
                    .saturating_sub(current),
            )
            .min(self.limits.copy(HtmlParseResource::AtomicLayoutBytes));
        // A required layout that cannot fit is submitted to the existing
        // reserve gate, preserving its concrete limit precedence and stop.
        if exact.new_bytes > ceiling {
            return Ok(Some((exact, exact)));
        }
        let representation = || ParseStop::RepresentationExhausted {
            resource: HtmlParseResource::ManagedCapacityBytes,
        };
        // `for_sized` refused a zero element size, so the division is total.
        let slots = ceiling
            .checked_div(element_size as u64)
            .and_then(|slots| usize::try_from(slots).ok())
            .ok_or_else(representation)?;
        let required = len.checked_add(additional).ok_or_else(representation)?;
        let target = required
            .max(capacity.saturating_mul(2).min(final_len))
            .min(slots);
        Ok(Some((
            exact,
            ReservationPlan {
                additional: target.checked_sub(len).ok_or_else(representation)?,
                old_bytes: exact.old_bytes,
                new_bytes: target
                    .checked_mul(element_size)
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(representation)?,
            },
        )))
    }
    fn record_vector_capacity<T>(
        vector: &Vec<T>,
        allocation: &mut VectorCapacity<T>,
        result: Result<(), ParseStop>,
    ) -> Result<(), ParseStop> {
        allocation.capacity = vector.capacity();
        result
    }
    /// The only Weak reference permitted here is this actual wrapper ticket.
    /// Its temporary removal and restoration are prepaid, even if reserve
    /// latches a terminal failure. Opaque Arc headers stay outside byte counts.
    fn prepare_attribute_edit(
        &mut self,
        values: &Arc<Vec<crate::Attribute>>,
        shared: &SharedAllocation<Vec<crate::Attribute>>,
        vector: &VectorCapacity<crate::Attribute>,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::RecordVisit, 3),
            (WorkCategory::ProviderCall, 9),
            (
                WorkCategory::CopyByte,
                const { (2 * size_of::<Weak<Vec<crate::Attribute>>>()) as u64 },
            ),
            (WorkCategory::RecordWrite, 2),
        ])?;
        if !Arc::ptr_eq(&shared.owner, &self.identity)
            || !Arc::ptr_eq(&vector.owner, &self.identity)
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if shared.finalized
            || shared.shared.as_ptr() != Arc::as_ptr(values)
            || shared.bytes != size_of::<Vec<crate::Attribute>>() as u64
            || vector.capacity != values.capacity()
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        if Arc::strong_count(values) != 1 || Arc::weak_count(values) != 1 {
            return self.fail(ParseStop::InternalInvariant(
                Invariant::SharedAllocationAliased,
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
                return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
            if allocation.capacity != vector.capacity() {
                return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
            }
            let Some(bytes) = allocation
                .capacity
                .checked_mul(size_of::<T>())
                .and_then(|bytes| u64::try_from(bytes).ok())
            else {
                return self.representation(HtmlParseResource::ManagedCapacityBytes);
            };
            self.cleanup_capacity(bytes)
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
    fn cleanup_capacity(&mut self, bytes: u64) -> Result<(), ParseStop> {
        let capacity = self.capacity_after_release(bytes)?;
        self.commit_capacity(capacity);
        Ok(())
    }
    /// The retained capacity left once `bytes` are credited; nothing changes.
    fn capacity_after_release(&mut self, bytes: u64) -> Result<u64, ParseStop> {
        match self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            .checked_sub(bytes)
        {
            Some(capacity) => Ok(capacity),
            None => self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow)),
        }
    }
    /// Records a capacity that [`Self::capacity_after_release`] has qualified.
    fn commit_capacity(&mut self, capacity: u64) {
        self.set(HtmlParseResource::ManagedCapacityBytes, capacity);
        self.set(HtmlParseResource::PeakTransientBytes, capacity);
        self.usage.actual_capacity_bytes = capacity;
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
    ) -> Result<(Arc<T>, SharedAllocation<T>), (ParseStop, T)> {
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
    fn admit_shared_layout<T>(&mut self) -> Result<u64, ParseStop> {
        self.checkpoint()?;
        let Ok(bytes) = u64::try_from(size_of::<T>()) else {
            return self.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        self.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::CopyByte, bytes),
            (WorkCategory::RecordWrite, 1),
        ])?;
        let Some(total) = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            .checked_add(bytes)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.check_value(HtmlParseResource::ManagedCapacityBytes, total, bytes)?;
        self.check_value(HtmlParseResource::PeakTransientBytes, total, bytes)?;
        self.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        self.set(HtmlParseResource::AtomicLayoutBytes, bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        Ok(bytes)
    }
    /// Polls once more after [`Self::admit_shared_layout`]; a stop refunds the
    /// capacity just charged. Admission proved that capacity and no other
    /// storage mutation precedes this refund. Were it refused anyway, the first
    /// stop is already latched, so the skipped credit is what that stop reports.
    fn confirm_shared_admission(&mut self, bytes: u64) -> Result<(), ParseStop> {
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
            let stop = self.latch_stop(ParseStop::InternalInvariant(
                Invariant::SharedAllocationAliased,
            ));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            let stop = self.latch_stop(ParseStop::InternalInvariant(Invariant::ForeignControl));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        if !std::ptr::eq(Arc::as_ptr(&shared), allocation.shared.as_ptr()) {
            let stop = self.latch_stop(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
            return Err(SharedReleaseError {
                stop,
                shared,
                allocation,
            });
        }
        let remaining = match self.capacity_after_release(allocation.bytes) {
            Ok(remaining) => remaining,
            Err(stop) => {
                return Err(SharedReleaseError {
                    stop,
                    shared,
                    allocation,
                });
            }
        };
        match Arc::try_unwrap(shared) {
            Ok(payload) => {
                self.commit_capacity(remaining);
                Ok(payload)
            }
            Err(shared) => {
                let stop = self.latch_stop(ParseStop::InternalInvariant(
                    Invariant::SharedAllocationAliased,
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
            Some(Invariant::ForeignControl)
        } else if !std::ptr::eq(Arc::as_ptr(&shared), allocation.shared.as_ptr())
            || size_of::<T>() != 0
            || allocation.bytes != 0
        {
            Some(Invariant::CapacityMismatch)
        } else {
            None
        };
        if let Some(invariant) = invariant {
            let stop = self.latch_stop(ParseStop::InternalInvariant(invariant));
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
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if !resource.storage() {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if resource == HtmlParseResource::ManagedNodes {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes));
        }
        let Some(value) = self.usage.resource(resource).current.checked_sub(units) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, value);
        Ok(())
    }
    /// Fixed original-owner disposal of exactly admitted resources. Selected
    /// replacement and cleanup retain the private ownership receipt; final
    /// report publication additionally preflights release before its last poll.
    /// This cannot execute source work or observe a later cancellation. A
    /// request beyond what is admitted latches `ReleaseUnderflow` and credits
    /// nothing, not even the requests that precede it.
    pub(crate) fn commit_prepared_release(
        &mut self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<(), ParseStop> {
        let Some(values) = self.after_release(requests) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        for &(resource, _) in requests {
            self.set(resource, values.copy(resource));
        }
        self.usage.actual_capacity_bytes = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        Ok(())
    }
    /// Every resource's current value once `requests` are credited, or `None`
    /// when one asks for more than is admitted.
    fn after_release(&self, requests: &[(HtmlParseResource, u64)]) -> Option<ResourceTable<u64>> {
        let mut values = self.usage.resources.map(|usage| usage.current);
        for &(resource, units) in requests {
            *values.get_mut(resource) = values.copy(resource).checked_sub(units)?;
        }
        Some(values)
    }
    /// Releases actually deallocated non-node logical slots after first-stop.
    /// No successful engine transition may use this cleanup-only path.
    pub(crate) fn cleanup_release(
        &mut self,
        identity: &ControlIdentity,
        resource: HtmlParseResource,
        units: u64,
    ) -> Result<(), ParseStop> {
        if self.stop().is_none() {
            return Err(self.fail_invariant(Invariant::CleanupWhileRunning));
        }
        if !Arc::ptr_eq(&identity.0, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if !resource.storage() {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if resource == HtmlParseResource::ManagedNodes {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes));
        }
        let Some(value) = self.usage.resource(resource).current.checked_sub(units) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, value);
        Ok(())
    }
    pub(crate) fn observe_tree_depth(&mut self, depth: u64) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.check_value(
            HtmlParseResource::TreeDepth,
            depth,
            depth.saturating_sub(self.usage.resource(HtmlParseResource::TreeDepth).current),
        )?;
        self.set(HtmlParseResource::TreeDepth, depth);
        Ok(())
    }
    fn reserve<T>(&mut self, vector: &mut Vec<T>, plan: ReservationPlan) -> Result<(), ParseStop> {
        self.checkpoint()?;
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(without_old) = current.checked_sub(plan.old_bytes) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        let (Some(new_total), Some(overlap)) = (
            without_old.checked_add(plan.new_bytes),
            current.checked_add(plan.new_bytes),
        ) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        // A reservation never shrinks the allocation it replaces.
        let Some(growth) = plan.new_bytes.checked_sub(plan.old_bytes) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        self.check_value(HtmlParseResource::ManagedCapacityBytes, new_total, growth)?;
        self.check_value(
            HtmlParseResource::PeakTransientBytes,
            overlap,
            plan.new_bytes,
        )?;
        self.check_value(
            HtmlParseResource::AtomicLayoutBytes,
            plan.new_bytes,
            plan.new_bytes,
        )?;
        self.set(HtmlParseResource::PeakTransientBytes, overlap);
        self.set(HtmlParseResource::AtomicLayoutBytes, plan.new_bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(plan.new_bytes);
        if vector.try_reserve_exact(plan.additional).is_err() {
            self.set(HtmlParseResource::PeakTransientBytes, current);
            return self.fail(ParseStop::AllocationFailed {
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
        let Some(actual_total) = without_old.checked_add(actual) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.usage.actual_capacity_bytes = actual_total;
        self.usage.actual_capacity_high_water =
            self.usage.actual_capacity_high_water.max(actual_total);
        self.set(HtmlParseResource::ManagedCapacityBytes, actual_total);
        let Some(actual_overlap) = current.checked_add(actual) else {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        };
        self.set(HtmlParseResource::PeakTransientBytes, actual_overlap);
        self.set(HtmlParseResource::PeakTransientBytes, actual_total);
        if actual > plan.new_bytes {
            // `actual` exceeds `new_bytes`, which is at least `old_bytes`.
            let requested_delta = actual.saturating_sub(plan.old_bytes);
            return self.fail(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: plan.new_bytes,
                current: plan.old_bytes,
                requested_delta,
            });
        }
        self.checkpoint()
    }
    /// Cleanup is permitted after a terminal stop; it refunds no admitted work or
    /// high-water observation. The vector owner supplies its exact retained size.
    fn cleanup_storage(
        &mut self,
        resource: HtmlParseResource,
        units: u64,
        bytes: u64,
    ) -> Result<(), ParseStop> {
        let (Some(logical), Some(capacity)) = (
            self.usage.resource(resource).current.checked_sub(units),
            self.usage
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                .checked_sub(bytes),
        ) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        self.set(resource, logical);
        self.set(HtmlParseResource::ManagedCapacityBytes, capacity);
        self.set(HtmlParseResource::PeakTransientBytes, capacity);
        self.usage.actual_capacity_bytes = capacity;
        Ok(())
    }
    /// Admits only the known fixed Arc payload. Arc headers, allocator metadata,
    /// and its infallible setup are provider limitations, not measured capacity.
    /// The consumer polls after the actual shared-wrapper construction.
    fn admit_document_string_wrapper(
        &mut self,
        mut buffer: OwnedVec<u16>,
    ) -> Result<(Vec<u16>, DocumentStringAllocation), ParseStop> {
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
    ) -> Result<u64, ParseStop> {
        self.checkpoint()?;
        if resource != HtmlParseResource::BufferedUtf16Units {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        let bytes = size_of::<Vec<u16>>() as u64;
        self.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
        ])?;
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(total) = current.checked_add(bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.check_value(HtmlParseResource::ManagedCapacityBytes, total, bytes)?;
        self.check_value(HtmlParseResource::PeakTransientBytes, total, bytes)?;
        self.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        self.set(HtmlParseResource::AtomicLayoutBytes, bytes);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(bytes);
        // This total includes known fixed payloads, never opaque provider headers.
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        Ok(bytes)
    }
    /// Owns actual wrapper construction, including the empty-vector case where
    /// a dangling data pointer cannot by itself prove backing provenance.
    /// Arc's fixed known Vec payload is admitted; opaque headers, allocator
    /// bookkeeping and infallible std Arc setup remain explicit limitations.
    pub(crate) fn freeze_document_string(
        &mut self,
        buffer: OwnedVec<u16>,
    ) -> Result<(crate::DocumentString, DocumentStringAllocation), ParseStop> {
        let (units, mut allocation) = self.admit_document_string_wrapper(buffer)?;
        let text = crate::DocumentString::from_utf16(units);
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
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge_many(&[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::ProviderCall, 1),
        ])?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if allocation.backing.is_some()
            || value.len() as u64 != allocation.units
            || value.shared_code_units().capacity() != allocation.vector_capacity
            || value.code_units().as_ptr() as usize != allocation.data_pointer
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        allocation.backing = Some(Arc::downgrade(value.shared_code_units()));
        Ok(())
    }
    pub(crate) fn require_document_string(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge(WorkCategory::RecordVisit, 1)?;
        self.validate_document_string(allocation, value)
    }
    /// Qualifies the new historical-materialization output slot before cleanup
    /// takes it. This fixed disposal proof admits no parsing work or storage.
    #[cfg(test)]
    pub(crate) fn require_materialized_string_disposal(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.validate_document_string(allocation, value)?;
        if allocation.resource != HtmlParseResource::BufferedUtf16Units
            || allocation.wrapper_bytes != size_of::<Vec<u16>>() as u64
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        let Some(bytes) = allocation
            .vector_capacity
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(allocation.wrapper_bytes))
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        if allocation.units > self.usage.resource(allocation.resource).current
            || bytes
                > self
                    .usage
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        }
        Ok(())
    }
    /// Disposal-only inspection of already-admitted backing. Terminal cleanup
    /// may validate ownership before taking a caller's retained payload; it
    /// cannot resume parsing, acquire storage, or refund execution observations.
    pub(crate) fn require_document_string_for_disposal(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        if self.stop().is_none() {
            return self.require_document_string(allocation, value);
        }
        self.validate_document_string(allocation, value)
    }
    fn validate_document_string(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if allocation
            .backing
            .as_ref()
            .is_none_or(|backing| backing.as_ptr() != Arc::as_ptr(value.shared_code_units()))
            || value.len() as u64 != allocation.units
            || value.shared_code_units().capacity() != allocation.vector_capacity
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        Ok(())
    }
    /// One atomic logical domain transition; real capacity and the immutable
    /// backing are unchanged. A failed cap or poll leaves both currents and the
    /// ticket phase unchanged.
    pub(crate) fn move_document_string_to_managed(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.require_document_string(allocation, value)?;
        if allocation.resource != HtmlParseResource::BufferedUtf16Units {
            return self.fail(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        let buffered = self
            .usage
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current;
        let managed = self
            .usage
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current;
        let Some(buffered) = buffered.checked_sub(allocation.units) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        let Some(managed) = managed.checked_add(allocation.units) else {
            return self.representation(HtmlParseResource::ManagedUtf16Units);
        };
        self.check_value(
            HtmlParseResource::ManagedUtf16Units,
            managed,
            allocation.units,
        )?;
        self.set(HtmlParseResource::BufferedUtf16Units, buffered);
        self.set(HtmlParseResource::ManagedUtf16Units, managed);
        allocation.resource = HtmlParseResource::ManagedUtf16Units;
        Ok(())
    }

    /// Transfers this producer's actual independent text ticket into the CSP
    /// provider aggregate. The aggregate is acquired before the vector ticket
    /// retires; the immutable backing remains live under its new sole receipt.
    pub(crate) fn csp_document_string(
        &mut self,
        source: &crate::DocumentString,
    ) -> Result<(crate::DocumentString, usize, usize), ParseStop> {
        let mut buffer = OwnedVec::new(HtmlParseResource::BufferedUtf16Units)?;
        for &unit in source.code_units() {
            if let Err(stop) = buffer.try_push(self, unit) {
                buffer.release(self)?;
                return Err(stop);
            }
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
            .charge(WorkCategory::ProviderCall, 2)
            .and_then(|()| self.acquire_retained_provider(bytes as u64, units as u64, 1));
        if let Err(stop) = admitted {
            drop(text);
            self.release_document_string(allocation)?;
            return Err(stop);
        }
        // This is an ownership transfer, not finalization or destruction of
        // live backing. The new aggregate is already admitted; consume the
        // unique prior ticket and retire only its duplicate debit.
        self.commit_prepared_release(&[
            (HtmlParseResource::ManagedCapacityBytes, bytes as u64),
            (HtmlParseResource::PeakTransientBytes, bytes as u64),
            (allocation.resource, allocation.units),
        ])?;
        drop(allocation);
        Ok((text, bytes, units))
    }

    pub(crate) fn release_document_string(
        &mut self,
        allocation: DocumentStringAllocation,
    ) -> Result<(), ParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        let Some(bytes) = allocation
            .vector_capacity
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(allocation.wrapper_bytes))
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        if !allocation.finalized
            && allocation
                .backing
                .as_ref()
                .is_some_and(|backing| backing.strong_count() != 0)
        {
            return self.fail(ParseStop::InternalInvariant(
                Invariant::SharedAllocationAliased,
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
    capacity: usize,
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
    stop: ParseStop,
    vector: Vec<T>,
    allocation: VectorCapacity<T>,
}
impl<T> VectorReleaseError<T> {
    pub(crate) fn into_parts(self) -> (ParseStop, Vec<T>, VectorCapacity<T>) {
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
    stop: ParseStop,
    shared: Arc<T>,
    allocation: SharedAllocation<T>,
}
impl<T> SharedReleaseError<T> {
    pub(crate) fn into_parts(self) -> (ParseStop, Arc<T>, SharedAllocation<T>) {
        (self.stop, self.shared, self.allocation)
    }
}

/// Non-cloneable accounting ticket for the transferred payload. It must remain
/// alongside the frozen text until the execution owner releases that allocation.
pub(crate) struct DocumentStringAllocation {
    owner: Arc<()>,
    units: u64,
    vector_capacity: usize,
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
    additional: usize,
    old_bytes: u64,
    new_bytes: u64,
}
impl ReservationPlan {
    fn for_vec<T>(vector: &Vec<T>, additional: usize) -> Result<Option<Self>, ParseStop> {
        Self::for_capacity::<T>(vector.len(), vector.capacity(), additional)
    }
    fn for_capacity<T>(
        len: usize,
        capacity: usize,
        additional: usize,
    ) -> Result<Option<Self>, ParseStop> {
        Self::for_sized(size_of::<T>(), len, capacity, additional)
    }
    fn for_sized(
        element_size: usize,
        len: usize,
        capacity: usize,
        additional: usize,
    ) -> Result<Option<Self>, ParseStop> {
        if element_size == 0 {
            return Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage));
        }
        let length = len
            .checked_add(additional)
            .ok_or(ParseStop::RepresentationExhausted {
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
                .ok_or(ParseStop::RepresentationExhausted {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                })
        };
        Ok(Some(Self {
            additional,
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
    /// Spare capacity that at least doubles it, clamped to the finite limits
    /// and never planned beyond the given final length (`usize::MAX`: unknown).
    Geometric(usize),
}

/// Private variable storage. No mutable Vec escape can bypass fallible growth.
/// Elements must be flat owned records; recursive provider drop is not bounded
/// by this byte ledger. `into_vec` transfers admitted storage without a copy.
pub(crate) struct OwnedVec<T> {
    vector: Vec<T>,
    resource: HtmlParseResource,
    owner: Option<Arc<()>>,
}
impl<T> OwnedVec<T> {
    pub(crate) fn new(resource: HtmlParseResource) -> Result<Self, ParseStop> {
        if !resource.storage() || resource == HtmlParseResource::ManagedNodes {
            return Err(ParseStop::InternalInvariant(Invariant::NonStorageResource));
        }
        if size_of::<T>() == 0 {
            return Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage));
        }
        Ok(Self {
            vector: Vec::new(),
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
    ) -> Result<Option<&mut T>, ParseStop> {
        self.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.vector.get_mut(index))
    }
    /// Swaps two actual admitted fixed slots. No payload clone, allocation,
    /// logical-slot change, or mutable-vector escape is involved.
    pub(crate) fn swap_slots(
        &mut self,
        left: usize,
        right: usize,
        control: &mut ParseControl,
    ) -> Result<bool, ParseStop> {
        self.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
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
            (WorkCategory::CopyByte, bytes as u64),
            (WorkCategory::RecordWrite, 2),
        ])?;
        self.vector.swap(left, right);
        Ok(true)
    }
    /// Removes one admitted slot and transfers its value without releasing its
    /// nested payload tickets. The caller owns their separate release/transfer.
    /// Capacity and all high-water observations are retained. An empty attempt
    /// visits only the container; it charges no write or payload copy.
    pub(crate) fn try_pop(&mut self, control: &mut ParseControl) -> Result<Option<T>, ParseStop> {
        self.require_owner(control)?;
        if self.vector.is_empty() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            return Ok(None);
        }
        let Ok(bytes) = u64::try_from(size_of::<T>()) else {
            return control.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        control.charge_many(&[
            (WorkCategory::RecordVisit, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, bytes),
        ])?;
        if size_of::<T>() > WORK_QUANTUM - 2 {
            control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
            control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
            control.usage.largest_atomic_layout = control.usage.largest_atomic_layout.max(bytes);
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
    ) -> Result<Option<T>, ParseStop> {
        if control.stop().is_none() {
            return Err(control.fail_invariant(Invariant::CleanupWhileRunning));
        }
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if self.vector.is_empty() {
            return Ok(None);
        }
        control.cleanup_storage(self.resource, 1, 0)?;
        Ok(self.vector.pop())
    }
    pub(crate) fn require_owner(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        control.checkpoint()?;
        if let Some(owner) = &self.owner {
            if !Arc::ptr_eq(owner, &control.identity) {
                return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
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
    ) -> Result<(), ParseStop> {
        if control.stop().is_none() {
            control.checkpoint()?;
            control.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(owner) = &self.owner {
            if control.stop().is_none() {
                control.charge(WorkCategory::ProviderCall, 1)?;
            }
            if !Arc::ptr_eq(owner, &control.identity) {
                return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
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
    ) -> Result<(), ParseStop> {
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
        self.prepare_plan(control, additional, copy_bytes, writes, plan)
    }
    fn prepare_plan(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
        copy_bytes: u64,
        writes: u64,
        plan: Option<ReservationPlan>,
    ) -> Result<(), ParseStop> {
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
            (WorkCategory::CopyByte, copy_bytes),
            (WorkCategory::RecordWrite, writes),
            (WorkCategory::ReserveByte, reserve_bytes),
            (WorkCategory::ProviderCall, u64::from(plan.is_some())),
        ])?;
        control.plan_storage(&[(self.resource, units)])?;
        if let Some(plan) = plan {
            control.reserve(&mut self.vector, plan)?;
        }
        Ok(())
    }
    /// Pair-local spare capacity must leave room for the other required slot.
    /// Only fixed Vec headers are probed here, without latching the other side's
    /// representation failure or moving its owner/work/logical admission earlier.
    /// The caller has already admitted the original combined atomic layout.
    fn prepare_pair_slot<U>(
        &mut self,
        other: &OwnedVec<U>,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        let pair = match control.geometric_pair(
            size_of::<T>(),
            self.vector.len(),
            self.vector.capacity(),
            1,
            usize::MAX,
        ) {
            Ok(pair) => pair,
            Err(stop) => return control.fail(stop),
        };
        let plan = if let Some((exact, mut plan)) = pair {
            control.charge(WorkCategory::RecordVisit, 2)?;
            match ReservationPlan::for_vec(&other.vector, 1) {
                Ok(Some(required)) => {
                    // After this reserve, the other required reserve must fit
                    // both retained capacity and old/new transient overlap.
                    let (Some(without_old), Some(required_growth)) = (
                        control
                            .usage
                            .resource(HtmlParseResource::ManagedCapacityBytes)
                            .current
                            .checked_sub(plan.old_bytes),
                        required.new_bytes.checked_sub(required.old_bytes),
                    ) else {
                        return control
                            .fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
                    };
                    let ceiling = control
                        .limits
                        .copy(HtmlParseResource::ManagedCapacityBytes)
                        .saturating_sub(without_old)
                        .saturating_sub(required_growth)
                        .min(
                            control
                                .limits
                                .copy(HtmlParseResource::PeakTransientBytes)
                                .saturating_sub(without_old)
                                .saturating_sub(required.new_bytes),
                        );
                    let clipped = plan.new_bytes.min(ceiling);
                    if clipped < exact.new_bytes {
                        // Keep the first exact preparation and let the second
                        // required operation report its original concrete refusal.
                        plan = exact;
                    } else {
                        let element = size_of::<T>() as u64;
                        let Some(slots) = clipped
                            .checked_div(element)
                            .and_then(|slots| usize::try_from(slots).ok())
                        else {
                            return control
                                .fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
                        };
                        // `clipped` covers the exact layout, so the slots
                        // exceed the length and their bytes fit `clipped`.
                        let (Some(additional), Some(new_bytes)) = (
                            slots.checked_sub(self.vector.len()),
                            u64::try_from(slots)
                                .ok()
                                .and_then(|slots| slots.checked_mul(element)),
                        ) else {
                            return control
                                .fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
                        };
                        plan.additional = additional;
                        plan.new_bytes = new_bytes;
                    }
                }
                Ok(None) => {}
                Err(_) => {
                    // Do not fail on the second side before its original turn.
                    plan = exact;
                }
            }
            Some(plan)
        } else {
            None
        };
        self.prepare_plan(control, 1, size_of::<T>() as u64, 1, plan)
    }
    /// Reserves only real physical append storage; initialized entries and
    /// their payload copies/writes remain the actual push caller's duty.
    pub(crate) fn reserve_geometric(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), ParseStop> {
        self.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        let plan = match control.geometric_plan(&self.vector, additional) {
            Ok(Some(plan)) => plan,
            Ok(None) => return Ok(()),
            Err(stop) => return control.fail(stop),
        };
        let Some(bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
            return control.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        control.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
        ])?;
        control.reserve(&mut self.vector, plan)
    }
    pub(crate) fn try_push(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), ParseStop> {
        self.push_with(control, value, Growth::Exact)
    }
    /// `try_push` for a list that grows with the document: the same admission
    /// in the same order, but a full vector reserves geometric spare capacity
    /// (charged and clamped to the limits) instead of exactly one more slot.
    pub(crate) fn try_push_geometric(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), ParseStop> {
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
    ) -> Result<(), ParseStop> {
        self.push_with(control, value, Growth::Geometric(final_len))
    }
    fn push_with(
        &mut self,
        control: &mut ParseControl,
        value: T,
        growth: Growth,
    ) -> Result<(), ParseStop> {
        control.checkpoint()?;
        let bytes = size_of::<T>() as u64;
        self.prepare(control, 1, bytes, 1, growth)?;
        if size_of::<T>() >= WORK_QUANTUM {
            control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
            control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
            control.usage.largest_atomic_layout = control.usage.largest_atomic_layout.max(bytes);
        }
        control.acquire(self.resource, 1)?;
        self.vector.push(value);
        Ok(())
    }
    /// One consumed evidence commit. Both capacities, logical slots and fixed
    /// writes are admitted before either vector's committed length changes.
    pub(crate) fn try_push_pair<U>(
        &mut self,
        other: &mut OwnedVec<U>,
        control: &mut ParseControl,
        value: T,
        other_value: U,
    ) -> Result<(), ParseStop> {
        let bytes = size_of::<T>()
            .checked_add(size_of::<U>())
            .ok_or_else(|| control.fail_invariant(Invariant::CapacityMismatch))?;
        control.admit_atomic_layout(bytes)?;
        self.prepare_pair_slot(other, control)?;
        other.prepare_pair_slot(self, control)?;
        control.acquire_many(&[(self.resource, 1), (other.resource, 1)])?;
        // No fallible operation follows admission. Record is published last.
        other.vector.push(other_value);
        self.vector.push(value);
        Ok(())
    }
    pub(crate) fn clear(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        control.checkpoint()?;
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        while !self.vector.is_empty() {
            let bytes = size_of::<T>() as u64;
            control.charge_many(&[
                (WorkCategory::RecordVisit, 1),
                (WorkCategory::RecordWrite, 1),
                (WorkCategory::CopyByte, bytes),
            ])?;
            if const { size_of::<T>() + 2 > WORK_QUANTUM } {
                control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
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
    ) -> Result<(), (ParseStop, T)> {
        self.recover_with(control, value, Growth::Exact)
    }
    /// `try_push_recover` with the geometric growth of `try_push_geometric`.
    pub(crate) fn try_push_recover_geometric(
        &mut self,
        control: &mut ParseControl,
        value: T,
    ) -> Result<(), (ParseStop, T)> {
        self.recover_with(control, value, Growth::Geometric(usize::MAX))
    }
    fn recover_with(
        &mut self,
        control: &mut ParseControl,
        value: T,
        growth: Growth,
    ) -> Result<(), (ParseStop, T)> {
        let admission = (|| {
            control.checkpoint()?;
            let bytes = size_of::<T>() as u64;
            self.prepare(control, 1, bytes, 1, growth)?;
            if size_of::<T>() >= WORK_QUANTUM {
                control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
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
    pub(crate) fn release(self, control: &mut ParseControl) -> Result<(), ParseStop> {
        if self
            .owner
            .as_ref()
            .is_some_and(|owner| !Arc::ptr_eq(owner, &control.identity))
        {
            return control.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        let Some(bytes) = self
            .vector
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return control.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        control.cleanup_storage(self.resource, self.vector.len() as u64, bytes)?;
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
    ) -> Result<(), ParseStop> {
        control.checkpoint()?;
        self.prepare(control, values.len(), 0, 0, Growth::Exact)?;
        // A zero-sized element has no byte stride; refuse it when this is compiled.
        let stride = const {
            assert!(size_of::<T>() != 0);
            WORK_QUANTUM / size_of::<T>()
        }
        .max(1);
        for chunk in values.chunks(stride) {
            let bytes = std::mem::size_of_val(chunk) as u64;
            control.charge(WorkCategory::CopyByte, bytes)?;
            if bytes > WORK_QUANTUM as u64 {
                control.check_value(HtmlParseResource::AtomicLayoutBytes, bytes, bytes)?;
                control.set(HtmlParseResource::AtomicLayoutBytes, bytes);
                control.usage.largest_atomic_layout =
                    control.usage.largest_atomic_layout.max(bytes);
            }
            control.acquire(self.resource, chunk.len() as u64)?;
            self.vector.extend_from_slice(chunk);
        }
        Ok(())
    }
}

impl ResourceUsage {
    pub fn current(&self) -> u64 {
        self.current
    }
    pub fn high_water(&self) -> u64 {
        self.high_water
    }
}
impl ParseUsage {
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
mod html_value_tests;
#[cfg(test)]
mod release_tests;

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
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            2
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
            4
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
            2
        );
        assert_eq!(
            c.usage().work(WorkCategory::ReserveByte),
            before.work(WorkCategory::ReserveByte)
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::AuxiliaryEntries),
            before.resource(HtmlParseResource::AuxiliaryEntries)
        );
        let before = c.usage().clone();
        assert!(slots.swap_slots(0, 0, &mut c).unwrap());
        assert!(!slots.swap_slots(0, 2, &mut c).unwrap());
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            4
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte),
            before.work(WorkCategory::CopyByte)
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite),
            before.work(WorkCategory::RecordWrite)
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
            c.charge(WorkCategory::StateStep, remaining - 8 + u64::from(short))
                .unwrap();
            let result = slots.swap_slots(0, 1, &mut c);
            if short {
                assert!(matches!(
                    result,
                    Err(ParseStop::LimitExceeded {
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(slots.as_slice(), &[11, 22]);
        cancel.cancel();
        assert_eq!(slots.swap_slots(0, 1, &mut c), Err(ParseStop::Cancelled));
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
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance);
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
            stop
        );
        assert_eq!(c.usage(), &before);
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::PendingScript),
            stop
        );
        assert_eq!(c.charge(WorkCategory::StateStep, 1), Err(stop));
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
            c.stop_unsupported_standalone(StandaloneUnsupported::InvalidNesting),
            ParseStop::Cancelled
        );
        assert_eq!(
            c.stop_unsupported_standalone(StandaloneUnsupported::ScriptPause),
            ParseStop::Cancelled
        );
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }
    use super::*;

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
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        assert!(c.stop().is_none());
        c.require_source(&source).unwrap();
        request.cancellation().cancel();
        assert_eq!(c.checkpoint(), Err(ParseStop::Cancelled));
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
        assert_eq!(c.usage(), &current);
        assert_eq!(c.first_stop_usage(), Some(&first));
    }
    use crate::DocumentString;

    #[test]
    fn pending_form_effect_stop_is_canonical_and_cannot_replace_first_failure() {
        for invariant in [
            Invariant::PendingFormEffects,
            Invariant::IncompleteFormAssociationCreation,
        ] {
            let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
            let stop = ParseStop::InternalInvariant(invariant);
            assert_eq!(c.fail_invariant(invariant), stop);
            assert_eq!(c.charge(WorkCategory::RecordWrite, 1), Err(stop));
            assert_eq!(c.fail_invariant(Invariant::ForeignSource), stop);
            assert_eq!(c.checkpoint(), Err(stop));
        }
        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            cancelled.fail_invariant(Invariant::PendingFormEffects),
            ParseStop::Cancelled
        );
        assert_eq!(cancelled.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn first_stop_snapshot_excludes_denied_growth_and_survives_terminal_refund() {
        let caps = limits()
            .into_builder()
            .auxiliary_entries(5)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        assert!(c.first_stop_usage().is_none());
        c.acquire(HtmlParseResource::AuxiliaryEntries, 4).unwrap();
        c.charge(WorkCategory::RecordWrite, 3).unwrap();
        let before = c.usage().clone();
        let stop = c
            .acquire(HtmlParseResource::AuxiliaryEntries, 2)
            .unwrap_err();
        assert_eq!(
            stop,
            ParseStop::LimitExceeded {
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
        assert_eq!(c.fail_invariant(Invariant::ForeignControl), stop);
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
        assert_eq!(c.checkpoint(), Err(ParseStop::Cancelled));
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
            c.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
            ParseStop::Cancelled
        );
        assert_eq!(c.first_stop_usage(), Some(&before));
    }
    fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(1_000)
            .buffered_utf16_units(1_000)
            .token_attributes(1_000)
            .auxiliary_entries(1_000)
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
            Err(ParseStop::LimitExceeded {
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
                matches!(result, Err(ParseStop::LimitExceeded { resource, .. })
                if resource == if work { HtmlParseResource::WorkCredits } else { HtmlParseResource::Diagnostics })
            );
            assert_eq!((records.len(), locations.len()), (0, 0));
            assert_eq!((records.capacity(), locations.capacity()), (0, 0));
            assert_eq!(foreign.usage(), &foreign_before);
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
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
    fn local_geometric_growth_charges_spare_capacity_and_clamps_to_finite_limits() {
        let caps = limits()
            .into_builder()
            .managed_capacity_bytes(6)
            .peak_transient_bytes(10)
            .atomic_layout_bytes(6)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let (mut values, mut ticket) = c.new_vector::<u16>().unwrap();
        for capacity in [1, 2, 3] {
            c.reserve_vector_geometric(&mut values, &mut ticket, 1)
                .unwrap();
            assert_eq!(values.capacity(), capacity);
            values.push(7);
        }
        assert_eq!(c.usage().work(WorkCategory::ReserveByte), 18); // 2 +(2+4) +(4+6)
        assert_eq!(c.usage().actual_capacity_bytes, 6);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::PeakTransientBytes)
                .high_water,
            10
        );
        let reserve = c.usage().work(WorkCategory::ReserveByte);
        // Existing exact caller still submits its required layout, without a
        // geometric fallback or a change to the other callers' policy.
        assert!(matches!(
            c.reserve_vector(&mut values, &mut ticket, 1),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: 6,
                current: 6,
                requested_delta: 2
            })
        ));
        assert_eq!(values, [7, 7, 7]);
        assert_eq!(c.usage().work(WorkCategory::ReserveByte), reserve + 14);
        let at_stop = c.first_stop_usage().unwrap().clone();
        c.release_vector(values, ticket).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.first_stop_usage(), Some(&at_stop));
        assert_eq!(c.usage().work, at_stop.work);
    }

    #[test]
    fn sized_geometric_growth_doubles_clamps_to_limits_and_defers_to_the_exact_gate() {
        // A bank described only by its element size: sixteen-byte elements,
        // room for six, so the clamp falls at ninety-six bytes.
        let caps = limits()
            .into_builder()
            .managed_capacity_bytes(96)
            .peak_transient_bytes(200)
            .atomic_layout_bytes(96)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut additional = |held: u64, len: usize, capacity: usize| {
            c.set(HtmlParseResource::ManagedCapacityBytes, held);
            c.geometric_sized(16, len, capacity, 1, usize::MAX)
                .unwrap()
                .map(|plan| plan.additional)
        };
        // Spare room still fits: no reservation is planned.
        assert_eq!(additional(32, 1, 2), None);
        // Doubling two slots to four.
        assert_eq!(additional(32, 2, 2), Some(2));
        // Doubling four slots would exceed the limit, so six slots are asked.
        assert_eq!(additional(64, 4, 4), Some(2));
        // The required layout alone is over the limit: it is submitted
        // unchanged so the reserve gate reports its own refusal.
        assert_eq!(additional(96, 6, 6), Some(1));
        assert_eq!(
            c.geometric_sized(0, 0, 0, 1, usize::MAX).unwrap_err(),
            ParseStop::InternalInvariant(Invariant::ZeroSizedStorage)
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
        assert_eq!(c.usage().work(WorkCategory::ReserveByte), 184);
        values.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
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
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values.try_push_geometric(&mut c, 1u32).unwrap();
        let capacity = values.capacity();
        assert!(matches!(
            values.try_push_geometric(&mut c, 2u32),
            Err(ParseStop::LimitExceeded {
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
        let reserved = c.usage().work(WorkCategory::ReserveByte);
        values.reserve_geometric(&mut c, 1).unwrap();
        assert_eq!(values.capacity(), 6);
        assert_eq!(values.as_slice(), [10, 20, 30]);
        assert_eq!(
            c.usage().work(WorkCategory::ReserveByte),
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
                    ParseStop::InternalInvariant(Invariant::ForeignControl)
                } else {
                    ParseStop::Cancelled
                })
            );
            assert_eq!(values.capacity(), 0);
            assert_eq!(active.usage().work(WorkCategory::ReserveByte), 0);
            c.release_vector(values, ticket).unwrap();
            assert_eq!(c.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn hidden_input_metadata_slots_are_admitted_before_take_and_bound_only_to_actual_nodes() {
        use crate::construction::{SessionError, release_token_kind};
        use crate::parser::tokenizer::{
            states::State,
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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
                *c.limits.get_mut(HtmlParseResource::AuxiliaryEntries) = slots + additional;
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
                        Err(SessionError::Stop(ParseStop::LimitExceeded {
                            resource: HtmlParseResource::AuxiliaryEntries,
                            ..
                        })),
                        Some(ParseStop::LimitExceeded {
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
        let copies = c.usage().work(WorkCategory::CopyByte);
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
        assert_eq!(c.usage().work(WorkCategory::CopyByte), copies);
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
            Err(ParseStop::LimitExceeded {
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
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
            Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
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
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
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
            Err(ParseStop::Cancelled)
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
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
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
            .tree_depth(15)
            .managed_capacity_bytes(16)
            .peak_transient_bytes(17)
            .atomic_layout_bytes(18)
            .work_credits(19)
            .finish()
            .unwrap();
        assert_eq!(
            limits.values().into_array(),
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignSource))
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
            Err(ParseStop::LimitExceeded {
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
            control.charge(WorkCategory::StateStep, 1),
            Err(ParseStop::Cancelled)
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
            control.charge(WorkCategory::StateStep, 1),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control.acquire(HtmlParseResource::Diagnostics, u64::MAX),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn stop_constructors_return_the_stop_the_result_form_carries_and_latch_it() {
        let expected = ParseStop::RepresentationExhausted {
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
            ParseStop::InvalidAssociatedFacts
        );
        assert_eq!(facts.stop(), Some(ParseStop::InvalidAssociatedFacts));
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
            control.latch_stop(ParseStop::InternalInvariant(Invariant::ForeignControl)),
            first
        );
        assert_eq!(control.stop(), Some(first));

        let cancellation = HtmlParseCancellation::new();
        let mut cancelled = ParseControl::new(limits(), cancellation.clone());
        cancellation.cancel();
        assert_eq!(
            cancelled.stop_invalid_associated_facts(),
            ParseStop::Cancelled
        );
        assert_eq!(
            cancelled.stop_representation(HtmlParseResource::ManagedEdges),
            ParseStop::Cancelled
        );
        assert_eq!(cancelled.stop(), Some(ParseStop::Cancelled));
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
            WorkCategory::InputUnitRead,
            WorkCategory::InspectByte,
            WorkCategory::CopyByte,
            WorkCategory::StateStep,
            WorkCategory::RecordVisit,
            WorkCategory::RecordWrite,
            WorkCategory::ReserveByte,
            WorkCategory::ProviderCall,
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
        let first = control.charge(WorkCategory::ProviderCall, 1).unwrap_err();
        cancellation.cancel();
        assert_eq!(control.checkpoint(), Err(first));
    }
    #[test]
    fn all_work_categories_accumulate_without_refunds() {
        let mut control = control();
        for category in [
            WorkCategory::InputUnitRead,
            WorkCategory::InspectByte,
            WorkCategory::CopyByte,
            WorkCategory::StateStep,
            WorkCategory::RecordVisit,
            WorkCategory::RecordWrite,
            WorkCategory::ReserveByte,
            WorkCategory::ProviderCall,
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
        control.charge(WorkCategory::StateStep, 2).unwrap();
        assert_eq!(
            control.charge(WorkCategory::InputUnitRead, 2),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 3,
                current: 2,
                requested_delta: 2
            })
        );
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
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
            control.charge(WorkCategory::StateStep, 4).unwrap();
            let before = control.usage().clone();
            let request = 6 + u64::from(short);
            let result = control.charge(WorkCategory::RecordWrite, request);
            if short {
                let first = ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: 10,
                    current: 4,
                    requested_delta: 7,
                };
                // The refused charge applies nothing to any counter.
                assert_eq!(result, Err(first));
                assert_eq!(control.usage(), &before);
                assert_eq!(control.stop(), Some(first));
                assert_eq!(control.first_stop_usage(), Some(&before));
                // The first stop is sticky: a request that would now fit, and
                // a different category, report the same stop and change nothing.
                assert_eq!(control.charge(WorkCategory::StateStep, 1), Err(first));
                assert_eq!(control.charge(WorkCategory::InputUnitRead, 7), Err(first));
                assert_eq!(control.usage(), &before);
                assert_eq!(control.first_stop_usage(), Some(&before));
            } else {
                // Spending exactly the cap succeeds and records every credit.
                assert_eq!(result, Ok(()));
                let usage = control.usage();
                assert_eq!(usage.resource(HtmlParseResource::WorkCredits).current, 10);
                assert_eq!(usage.work(WorkCategory::RecordWrite), 6);
                assert_eq!(control.stop(), None);
                // The next credit is the first one over the cap.
                assert_eq!(
                    control.charge(WorkCategory::StateStep, 1),
                    Err(ParseStop::LimitExceeded {
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
        control.charge(WorkCategory::StateStep, u64::MAX).unwrap();
        assert_eq!(
            control.charge(WorkCategory::RecordVisit, 1),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::WorkCredits
            })
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 0);
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
            Err(ParseStop::LimitExceeded {
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
    #[test]
    fn diagnostic_exhaustion_is_a_terminal_error() {
        let mut limits = limits();
        limits = limits.into_builder().diagnostics(1).finish().unwrap();
        let mut control = ParseControl::new(limits, HtmlParseCancellation::new());
        control.acquire(HtmlParseResource::Diagnostics, 1).unwrap();
        assert!(matches!(
            control.acquire(HtmlParseResource::Diagnostics, 1),
            Err(ParseStop::LimitExceeded {
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
            ResourceUsage {
                current: 2,
                high_water: 7
            }
        );
        control.acquire(HtmlParseResource::ManagedNodes, 1).unwrap();
        assert_eq!(
            control.release(HtmlParseResource::ManagedNodes, 1),
            Err(ParseStop::InternalInvariant(Invariant::NonReclaimableNodes))
        );
    }
    #[test]
    fn release_underflow_cannot_wrap() {
        let mut control = control();
        assert_eq!(
            control.release(HtmlParseResource::ManagedEdges, 1),
            Err(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow))
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
            ResourceUsage {
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
                .work_batch(WorkCategory::InspectByte, 1_000)
                .unwrap(),
            256
        );
        assert_eq!(control.work_batch(WorkCategory::InspectByte, 3).unwrap(), 3);
        control.cancellation.cancel();
        assert_eq!(
            control.work_batch(WorkCategory::InspectByte, 0),
            Err(ParseStop::Cancelled)
        );
    }
    #[test]
    fn reservation_layout_rejects_unrepresentable_growth_without_allocator() {
        assert_eq!(
            ReservationPlan::for_vec(&Vec::<u16>::new(), usize::MAX),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedCapacityBytes
            })
        );
        assert_eq!(
            ReservationPlan::for_vec(&Vec::<()>::new(), 1),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
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
            Err(ParseStop::LimitExceeded {
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
            ResourceUsage {
                current: 0,
                high_water: 4
            }
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
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
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), 1);
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
                matches!(buffer.try_push(&mut control, 1), Err(ParseStop::LimitExceeded { resource: failed, .. }) if failed == resource)
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
            Err(ParseStop::InternalInvariant(Invariant::MissingSource))
        );
        let mut admitted = control();
        admitted.admit_source(&source).unwrap();
        admitted.require_source(&source.clone()).unwrap();
        assert_eq!(
            admitted.require_source(&HtmlSource::utf8("a")),
            Err(ParseStop::InternalInvariant(Invariant::ForeignSource))
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
            control.fail_invariant(Invariant::InputCursorState),
            ParseStop::InternalInvariant(Invariant::InputCursorState)
        );
        control.cancellation.cancel();
        assert_eq!(
            control.fail_invariant(Invariant::ForeignSource),
            ParseStop::InternalInvariant(Invariant::InputCursorState)
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
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
            ParseStop::InternalInvariant(Invariant::ReleaseUnderflow)
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }
    #[test]
    fn fixed_shared_payload_is_admitted_without_variable_copy() {
        let mut control = control();
        let mut buffer = OwnedVec::<u16>::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        buffer.try_push(&mut control, 0xD800).unwrap();
        let copied = control.usage().work(WorkCategory::CopyByte);
        let (vector, allocation) = control.admit_document_string_wrapper(buffer).unwrap();
        let wrapper = allocation.wrapper_bytes();
        assert_eq!(allocation.units(), 1);
        assert_eq!(allocation.vector_capacity(), 1);
        let text = DocumentString::from_utf16(vector);
        control.checkpoint().unwrap();
        assert_eq!(text.code_units(), &[0xD800]);
        assert_eq!(wrapper, size_of::<Vec<u16>>() as u64);
        assert_eq!(control.usage().actual_capacity_bytes, 2 + wrapper);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), copied);
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
            .charge(WorkCategory::ReserveByte, isize::MAX as u64)
            .unwrap();
        control.charge(WorkCategory::ProviderCall, 1).unwrap();
        assert_eq!(
            control.reserve(&mut vector, plan),
            Err(ParseStop::AllocationFailed {
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
        control.charge(WorkCategory::StateStep, u64::MAX).unwrap();
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
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 1);
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(control.usage().work(WorkCategory::InspectByte), 0);
    }
    #[test]
    fn every_storage_cap_rejects_its_own_growth() {
        for resource in RESOURCES.into_iter().filter(|resource| resource.storage()) {
            let mut control = control();
            *control.limits.get_mut(resource) = 0;
            assert_eq!(
                control.acquire(resource, 1),
                Err(ParseStop::LimitExceeded {
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
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
        ));
        assert!(matches!(
            OwnedVec::<()>::new(HtmlParseResource::BufferedUtf16Units),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
        ));
        let mut control = control();
        assert_eq!(
            control.acquire(HtmlParseResource::WorkCredits, 1),
            Err(ParseStop::InternalInvariant(Invariant::NonStorageResource))
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(second.usage(), &previous);
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
        let copy = control.usage().work(WorkCategory::CopyByte);
        buffer.clear(&mut control).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            previous + 12
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 3);
        assert_eq!(control.usage().work(WorkCategory::RecordWrite), 3);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), copy + 6);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units),
            ResourceUsage {
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
        *control.limits.get_mut(HtmlParseResource::WorkCredits) = previous + 4;
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
            Err(ParseStop::LimitExceeded {
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
        control.charge(WorkCategory::InspectByte, 2).unwrap();
        assert_eq!(*entry, 7);
        control
            .charge_many(&[(WorkCategory::RecordWrite, 1), (WorkCategory::CopyByte, 2)])
            .unwrap();
        *entry = 9;
        assert_eq!(entries.as_slice(), &[9]);
        assert_eq!(
            control.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit) + 1
        );
        assert_eq!(
            control.usage().work(WorkCategory::InspectByte),
            before.work(WorkCategory::InspectByte) + 2
        );
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte),
            before.work(WorkCategory::CopyByte) + 2
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            before.resource(HtmlParseResource::AuxiliaryEntries)
        );
        assert_eq!(entries.get_mut(1, &mut control).unwrap(), None);
        assert_eq!(
            control.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit) + 2
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(foreign.usage(), &before);
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
        assert_eq!(entries.get_mut(0, &mut control), Err(ParseStop::Cancelled));
        assert_eq!(entries.try_pop(&mut control), Err(ParseStop::Cancelled));
        assert_eq!(control.usage(), &before);
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
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 6,
                current: 6,
                requested_delta: 1,
            })
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 0);
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
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 1);
        assert_eq!(control.usage().work(WorkCategory::RecordWrite), 0);
        assert_eq!(control.usage().work(WorkCategory::CopyByte), 0);
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
            ResourceUsage {
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
            control.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit) + 1
        );
        assert_eq!(
            control.usage().work(WorkCategory::RecordWrite),
            before.work(WorkCategory::RecordWrite) + 1
        );
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte),
            before.work(WorkCategory::CopyByte) + 2
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
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
            Err(ParseStop::LimitExceeded {
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
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        assert_eq!(entries.as_slice(), &[7]);
        assert_eq!(control.usage(), &before);
        assert_eq!(
            entries.try_pop(&mut control),
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
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
            ResourceUsage {
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
        assert_eq!(foreign.usage(), &before);
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
            ParseStop::LimitExceeded {
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
            Err((ParseStop::InternalInvariant(Invariant::ForeignControl), 11))
        );
        owner.cancellation.cancel();
        assert_eq!(
            entries.try_push_recover(&mut owner, 13),
            Err((ParseStop::Cancelled, 13))
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
                ParseStop::LimitExceeded {
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
            .charge_many(&[(WorkCategory::RecordWrite, 2), (WorkCategory::CopyByte, 8)])
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
        assert_eq!(control.usage().work(WorkCategory::ReserveByte), 28);
        let calls = control.usage().work(WorkCategory::ProviderCall);
        control.reserve_vector(&mut vector, &mut ticket, 1).unwrap();
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), calls);
        control.cancellation.cancel();
        let stop = control.checkpoint().unwrap_err();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        control.release_vector(vector, ticket).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedNodes),
            ResourceUsage {
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(foreign.usage(), &before);
        assert_eq!(ticket.capacity(), 2);
        let (stop, vector, ticket) = foreign
            .release_vector(vector, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::ForeignControl)
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
        let calls = control.usage().work(WorkCategory::ProviderCall);
        assert_eq!(
            control.reserve_vector(&mut second, &mut first_ticket, 3),
            Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
        );
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), calls);
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
            Err(ParseStop::LimitExceeded {
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
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.usage(), &before);
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
            Err(ParseStop::AllocationFailed {
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
                (WorkCategory::ReserveByte, 8),
                (WorkCategory::ProviderCall, 1),
            ])
            .unwrap();
        control.reserve(&mut vector, plan).unwrap();
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
            Err(ParseStop::LimitExceeded {
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
            .charge_many(&[(WorkCategory::CopyByte, 8), (WorkCategory::RecordWrite, 2)])
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
        assert_eq!(control.usage().work(WorkCategory::ProviderCall), 1);
        assert_eq!(control.release_shared(shared, ticket).unwrap(), ());
        assert!(matches!(
            control.new_vector::<()>(),
            Err(ParseStop::InternalInvariant(Invariant::ZeroSizedStorage))
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
                ParseStop::LimitExceeded {
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
            (ParseStop::Cancelled, 91)
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
            Err(ParseStop::Cancelled)
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
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
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        );
        assert_eq!(owner.usage().actual_capacity_bytes, 4);
        let other_handle = shared.clone();
        let (stop, shared, ticket) = owner
            .release_shared(shared, ticket)
            .unwrap_err()
            .into_parts();
        assert_eq!(
            stop,
            ParseStop::InternalInvariant(Invariant::SharedAllocationAliased)
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
        control.charge(WorkCategory::StateStep, 5).unwrap();
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
        assert_eq!(control.usage(), &before);
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
            ResourceUsage {
                current: 0,
                high_water: 3
            }
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::ManagedEdges),
            ResourceUsage {
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
            Err(ParseStop::InternalInvariant(Invariant::CleanupWhileRunning))
        );
        assert_eq!(control.usage(), &before);
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
            ResourceUsage {
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
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::AtomicLayoutBytes,
                limit: 511,
                current: 0,
                requested_delta: 512,
            }
        );
        owner.cancellation.cancel();
        assert_eq!(owner.admit_atomic_layout(0), Err(stop));
        assert_eq!(owner.usage(), &before);

        let mut owner = control();
        owner.cancellation.cancel();
        let before = owner.usage().clone();
        assert_eq!(owner.admit_atomic_layout(1), Err(ParseStop::Cancelled));
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn fixed_record_layout_checks_representable_span_and_zero_layout() {
        let mut owner = control();
        owner.admit_atomic_layout(0).unwrap();
        let before = owner.usage().clone();
        assert_eq!(
            owner.admit_atomic_layout(usize::MAX),
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::AtomicLayoutBytes,
            })
        );
        assert_eq!(owner.usage(), &before);
    }
    #[test]
    fn checked_arithmetic_failure_latches_observation_and_preserves_first_stop() {
        let mut owner = control();
        owner.checkpoint().unwrap();
        let before = owner.usage().clone();
        // HtmlParseCancellation arrives after the arithmetic operation's initial poll.
        // The already observed failure remains the first terminal transition.
        owner.cancellation.cancel();
        let stop = ParseStop::RepresentationExhausted {
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
        assert_eq!(owner.usage(), &before);

        let mut owner = control();
        owner.cancellation.cancel();
        assert_eq!(owner.checkpoint(), Err(ParseStop::Cancelled));
        assert_eq!(
            owner.representation::<()>(HtmlParseResource::ManagedUtf16Units),
            Err(ParseStop::Cancelled)
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
        assert_eq!(owner.usage(), &before);
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
            ParseStop::InternalInvariant(Invariant::ForeignControl)
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
            ParseStop::InternalInvariant(Invariant::CapacityMismatch)
        );
        assert_eq!(owner.usage(), &before);
        owner
            .release_zero_sized_shared(first, first_ticket)
            .unwrap();
        owner
            .release_zero_sized_shared(second, second_ticket)
            .unwrap();
        assert_eq!(owner.usage(), &before);
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
            ParseStop::InternalInvariant(Invariant::CapacityMismatch)
        );
        assert_eq!(owner.usage(), &before);
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
            .charge_many(&[(WorkCategory::RecordWrite, 3), (WorkCategory::CopyByte, 24)])
            .unwrap();
        let before = owner.usage().clone();
        owner.commit_construction_edges(&identity, 2, 1).unwrap();
        assert_eq!(
            owner.usage().resource(HtmlParseResource::ManagedEdges),
            ResourceUsage {
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
            ResourceUsage {
                current: 2,
                high_water: 2,
            }
        );
        let before = owner.usage().clone();
        owner.commit_construction_edges(&identity, 0, 0).unwrap();
        assert_eq!(owner.usage(), &before);
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
        let stop = ParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedEdges,
            limit: 1,
            current: 1,
            requested_delta: 1,
        };
        assert_eq!(owner.commit_construction_edges(&identity, 1, 0), Err(stop));
        assert_eq!(owner.usage(), &before);
        owner.cancellation.cancel();
        assert_eq!(owner.commit_construction_edges(&identity, 0, 0), Err(stop));
        assert_eq!(owner.usage(), &before);
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
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(owner.usage(), &before);
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
            Err(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow))
        );
        assert_eq!(owner.usage(), &before);
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
            Err(ParseStop::RepresentationExhausted {
                resource: HtmlParseResource::ManagedEdges,
            })
        );
        assert_eq!(owner.usage(), &before);
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
                Err(ParseStop::Cancelled)
            );
            assert_eq!(owner.usage(), &before);
            assert_eq!(
                owner.commit_construction_edges(&identity, 0, 1),
                Err(ParseStop::Cancelled)
            );
            assert_eq!(owner.usage(), &before);
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
        let source = HtmlSource::utf16(crate::DocumentString::from("<input type=color value=red>"));
        owner.admit_source(&source).unwrap();
        let unavailable = surgeist_css::color::CssColorUnavailable::CurrentColor;
        let stop = owner.stop_color_processing(css_color_error(b"currentcolor"));
        assert_eq!(
            stop,
            ParseStop::ColorProcessing(HtmlColorProcessingError::Unavailable(unavailable))
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
            ParseStop::InternalInvariant(Invariant::ColorConversionState)
        );
    }
}
