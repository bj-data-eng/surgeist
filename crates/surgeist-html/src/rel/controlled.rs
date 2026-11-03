//! One retained local relationship interpretation and its checked refinement.
use super::*;

/// The explicit original operands, independent of captured-tree qualification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelInputs<'raw> {
    pub(super) role: Option<HtmlRelRole>,
    pub(super) rel: Option<&'raw DocumentString>,
    pub(super) rev: Option<&'raw DocumentString>,
    pub(super) href: Option<&'raw DocumentString>,
    pub(super) imagesrcset: Option<&'raw DocumentString>,
    pub(super) itemprop: Option<&'raw DocumentString>,
    pub(super) title: Option<&'raw DocumentString>,
    pub(super) type_hint: Option<&'raw DocumentString>,
    pub(super) as_destination: Option<HtmlLinkDestination>,
}
impl<'raw> RelInputs<'raw> {
    /// Creates inputs for an element of `role`, with every operand absent. A `role` of `None` is an
    /// element that takes no `rel`; interpretation then reports no occurrences.
    pub fn new(role: Option<HtmlRelRole>) -> Self {
        Self {
            role,
            rel: None,
            rev: None,
            href: None,
            imagesrcset: None,
            itemprop: None,
            title: None,
            type_hint: None,
            as_destination: None,
        }
    }
    /// Sets the raw `rel` attribute value, or marks the attribute absent with `None`. The value is
    /// borrowed exactly as authored; interpretation reads its UTF-16 code units.
    pub fn with_rel(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.rel = raw;
        self
    }
    /// Sets the raw `rev` attribute value, or marks the attribute absent with `None`. Only the
    /// exact value `made` produces an occurrence.
    pub fn with_rev(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.rev = raw;
        self
    }
    /// Sets the raw `href` attribute value, or marks the attribute absent with `None`. Only its
    /// presence and whether it contains a non-whitespace unit matter; its URL grammar is not
    /// checked.
    pub fn with_href(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.href = raw;
        self
    }
    /// Sets the raw `imagesrcset` attribute value, or marks the attribute absent with `None`. Only
    /// its presence matters here; its source-set grammar is not checked.
    pub fn with_imagesrcset(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.imagesrcset = raw;
        self
    }
    /// Sets the raw `itemprop` attribute value, or marks the attribute absent with `None`. Only its
    /// presence matters; it selects the body allowance and the `rel`/`itemprop` presence rule.
    pub fn with_itemprop(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.itemprop = raw;
        self
    }
    /// Sets the raw `title` attribute value, or marks the attribute absent with `None`. An
    /// alternative style sheet needs it nonempty.
    pub fn with_title(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.title = raw;
        self
    }
    /// Sets the raw `type` attribute value, or marks the attribute absent with `None`. Exactly
    /// `application/rss+xml` or `application/atom+xml` makes an `alternate` link a feed.
    pub fn with_type_hint(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.type_hint = raw;
        self
    }
    /// Sets the `as` attribute's already interpreted destination, or `None` when the attribute is
    /// absent or invalid. The caller derives it with the `as` interpreter; this type does not
    /// re-read the attribute.
    pub fn with_as_destination(mut self, destination: Option<HtmlLinkDestination>) -> Self {
        self.as_destination = destination;
        self
    }
    /// The element's role, or `None` when the element takes no `rel`.
    pub fn role(&self) -> Option<HtmlRelRole> {
        self.role
    }
    /// The raw `rel` value as supplied, or `None` when the attribute is absent.
    pub fn rel(&self) -> Option<&'raw DocumentString> {
        self.rel
    }
    /// The raw `rev` value as supplied, or `None` when the attribute is absent.
    pub fn rev(&self) -> Option<&'raw DocumentString> {
        self.rev
    }
    /// The raw `href` value as supplied, or `None` when the attribute is absent.
    pub fn href(&self) -> Option<&'raw DocumentString> {
        self.href
    }
    /// The raw `imagesrcset` value as supplied, or `None` when the attribute is absent.
    pub fn imagesrcset(&self) -> Option<&'raw DocumentString> {
        self.imagesrcset
    }
    /// The raw `itemprop` value as supplied, or `None` when the attribute is absent.
    pub fn itemprop(&self) -> Option<&'raw DocumentString> {
        self.itemprop
    }
    /// The raw `title` value as supplied, or `None` when the attribute is absent.
    pub fn title(&self) -> Option<&'raw DocumentString> {
        self.title
    }
    /// The raw `type` value as supplied, or `None` when the attribute is absent.
    pub fn type_hint(&self) -> Option<&'raw DocumentString> {
        self.type_hint
    }
    /// The interpreted `as` destination, or `None` when none was supplied.
    pub fn as_destination(&self) -> Option<HtmlLinkDestination> {
        self.as_destination
    }
}
/// A violation of the rel interpretation protocol or a failed storage reservation. The cause is
/// private; query it with the `is_*` predicates, exactly one of which is true. New causes do not
/// break callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlRelInterpretationError {
    kind: HtmlRelInterpretationErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlRelInterpretationErrorKind {
    Consumed,
    NotReady,
    Stopped,
    WrongPhase,
    UnknownOccurrence,
    WrongSource,
    WrongSpelling,
    RepeatedRefinement,
    OutOfOrderRefinement,
    InvalidExtensionEffect,
    ReservationNotPerformed,
    ReservationAlreadyAttempted,
    RepresentationExhausted,
    AllocationFailed,
}

impl HtmlRelInterpretationErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlRelInterpretationError {
        HtmlRelInterpretationError { kind: self }
    }
}

impl From<HtmlRelInterpretationErrorKind> for HtmlRelInterpretationError {
    fn from(kind: HtmlRelInterpretationErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlRelInterpretationError {
    /// Whether the cause is: rel interpretation was already consumed.
    pub fn is_consumed(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::Consumed)
    }
    /// Whether the cause is: rel interpretation has an unfinished step.
    pub fn is_not_ready(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::NotReady)
    }
    /// Whether the cause is: rel interpretation was stopped.
    pub fn is_stopped(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::Stopped)
    }
    /// Whether the cause is: rel interpretation step was requested out of order.
    pub fn is_wrong_phase(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::WrongPhase)
    }
    /// Whether the cause is: rel refinement names an occurrence that does not exist.
    pub fn is_unknown_occurrence(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::UnknownOccurrence)
    }
    /// Whether the cause is: rel refinement does not match the source token.
    pub fn is_wrong_source(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::WrongSource)
    }
    /// Whether the cause is: rel refinement does not match the token spelling.
    pub fn is_wrong_spelling(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::WrongSpelling)
    }
    /// Whether the cause is: rel token was refined more than once.
    pub fn is_repeated_refinement(&self) -> bool {
        matches!(
            self.kind,
            HtmlRelInterpretationErrorKind::RepeatedRefinement
        )
    }
    /// Whether the cause is: rel refinements were not given in token order.
    pub fn is_out_of_order_refinement(&self) -> bool {
        matches!(
            self.kind,
            HtmlRelInterpretationErrorKind::OutOfOrderRefinement
        )
    }
    /// Whether the cause is: rel extension effect is not valid for its keyword.
    pub fn is_invalid_extension_effect(&self) -> bool {
        matches!(
            self.kind,
            HtmlRelInterpretationErrorKind::InvalidExtensionEffect
        )
    }
    /// Whether the cause is: rel storage reservation was not performed.
    pub fn is_reservation_not_performed(&self) -> bool {
        matches!(
            self.kind,
            HtmlRelInterpretationErrorKind::ReservationNotPerformed
        )
    }
    /// Whether the cause is: rel storage reservation was already attempted.
    pub fn is_reservation_already_attempted(&self) -> bool {
        matches!(
            self.kind,
            HtmlRelInterpretationErrorKind::ReservationAlreadyAttempted
        )
    }
    /// Whether the cause is: rel interpretation size cannot be represented on this platform.
    pub fn is_representation_exhausted(&self) -> bool {
        matches!(
            self.kind,
            HtmlRelInterpretationErrorKind::RepresentationExhausted
        )
    }
    /// Whether the cause is: reserving rel interpretation storage failed.
    pub fn is_allocation_failed(&self) -> bool {
        matches!(self.kind, HtmlRelInterpretationErrorKind::AllocationFailed)
    }
}

impl std::fmt::Display for HtmlRelInterpretationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlRelInterpretationErrorKind::Consumed => {
                f.write_str("rel interpretation was already consumed")
            }
            HtmlRelInterpretationErrorKind::NotReady => {
                f.write_str("rel interpretation has an unfinished step")
            }
            HtmlRelInterpretationErrorKind::Stopped => {
                f.write_str("rel interpretation was stopped")
            }
            HtmlRelInterpretationErrorKind::WrongPhase => {
                f.write_str("rel interpretation step was requested out of order")
            }
            HtmlRelInterpretationErrorKind::UnknownOccurrence => {
                f.write_str("rel refinement names an occurrence that does not exist")
            }
            HtmlRelInterpretationErrorKind::WrongSource => {
                f.write_str("rel refinement does not match the source token")
            }
            HtmlRelInterpretationErrorKind::WrongSpelling => {
                f.write_str("rel refinement does not match the token spelling")
            }
            HtmlRelInterpretationErrorKind::RepeatedRefinement => {
                f.write_str("rel token was refined more than once")
            }
            HtmlRelInterpretationErrorKind::OutOfOrderRefinement => {
                f.write_str("rel refinements were not given in token order")
            }
            HtmlRelInterpretationErrorKind::InvalidExtensionEffect => {
                f.write_str("rel extension effect is not valid for its keyword")
            }
            HtmlRelInterpretationErrorKind::ReservationNotPerformed => {
                f.write_str("rel storage reservation was not performed")
            }
            HtmlRelInterpretationErrorKind::ReservationAlreadyAttempted => {
                f.write_str("rel storage reservation was already attempted")
            }
            HtmlRelInterpretationErrorKind::RepresentationExhausted => {
                f.write_str("rel interpretation size cannot be represented on this platform")
            }
            HtmlRelInterpretationErrorKind::AllocationFailed => {
                f.write_str("reserving rel interpretation storage failed")
            }
        }
    }
}

impl std::error::Error for HtmlRelInterpretationError {}

/// Why one controlled rel action failed, separating the caller's refusal from this crate's protocol
/// or allocation fault. Exhaustive: a driver handles its own error and the crate's two cases.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlRelActionError<E> {
    /// The caller's meter refused work or a reservation with its own error `E`. The interpretation
    /// is stopped.
    Admission(E),
    /// The action was a protocol misuse or the reservation failed; the cause is the wrapped error.
    Protocol(HtmlRelInterpretationError),
}
impl<E> From<HtmlRelInterpretationError> for HtmlRelActionError<E> {
    fn from(error: HtmlRelInterpretationError) -> Self {
        Self::Protocol(error)
    }
}
impl<E> std::fmt::Display for HtmlRelActionError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Admission(_) => "rel interpretation was refused by the caller's admission",
            Self::Protocol(_) => "rel interpretation protocol was violated",
        })
    }
}
impl<E: std::fmt::Debug + std::error::Error + 'static> std::error::Error for HtmlRelActionError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::Protocol(error) => Some(error),
        }
    }
}
/// The registry's answer for one unknown `rel` token, supplied to `HtmlRelInterpretation::refine`.
/// The registry of extension link types is outside this crate, so the consumer decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelRefinement {
    /// The registry could not be consulted. The token stays an unresolved unknown token.
    Unavailable,
    /// The registry resolved the token as having no effect on this element.
    NoEffect,
    /// The registry maps the token onto a standard keyword. The token takes that keyword's effect
    /// for its role.
    Standard(HtmlRelKeyword),
    /// The registry resolved the token as an extension with this effect. `InternalResource` and
    /// `StylesheetModifier` are never valid for an extension, and `Annotation` is invalid on a
    /// `link`; `refine` rejects them.
    Extension(HtmlRelEffect),
}

#[derive(Debug)]
enum ReservationStorage<'a> {
    Occurrences(&'a mut Vec<HtmlRelOccurrence>),
    Data(&'a mut Vec<RelData>),
}
/// A one-record reservation against the actual private output allocation. A full allocation may
/// grow by more than one record; the meter admits and charges the capacity it reaches.
#[derive(Debug)]
pub struct HtmlRelReservation<'a> {
    storage: ReservationStorage<'a>,
    attempted: bool,
    completed: bool,
}
impl HtmlRelReservation<'_> {
    /// The number of records currently in the backing allocation.
    pub fn len(&self) -> usize {
        match &self.storage {
            ReservationStorage::Occurrences(v) => v.len(),
            ReservationStorage::Data(v) => v.len(),
        }
    }
    /// Whether the backing allocation holds no records.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The number of records the backing allocation can hold without growing.
    pub fn capacity(&self) -> usize {
        match &self.storage {
            ReservationStorage::Occurrences(v) => v.capacity(),
            ReservationStorage::Data(v) => v.capacity(),
        }
    }
    /// The size in bytes of one record, so a meter can charge the allocation it admits.
    pub fn record_size(&self) -> usize {
        match self.storage {
            ReservationStorage::Occurrences(_) => std::mem::size_of::<HtmlRelOccurrence>(),
            ReservationStorage::Data(_) => std::mem::size_of::<RelData>(),
        }
    }
    /// The number of records the backing allocation can hold once this guard's reservation has run.
    /// The current capacity when one more record fits. For the occurrences, which grow one at a
    /// time, a full allocation grows to the greater of one more record and double its capacity (at
    /// least 4), so that `n` occurrences request `O(log n)` reallocations. The interpretation's
    /// single report-data record is always reserved exactly. It depends only on the allocation,
    /// never on a limit: a meter that charges for the growth admits this capacity before calling
    /// [`try_reserve`](Self::try_reserve), and refuses the reservation when it cannot.
    pub fn target_capacity(&self) -> usize {
        match self.storage {
            ReservationStorage::Occurrences(_) => {
                crate::amortized::amortized_capacity(self.len(), self.capacity())
            }
            // One report-data record per interpretation: spare room would never be used.
            ReservationStorage::Data(_) => self.len().saturating_add(1).max(self.capacity()),
        }
    }
    /// Reserves room for one more record, growing the allocation only when it is full, to
    /// [`target_capacity`](Self::target_capacity). A meter must call this once during
    /// `HtmlRelMeter::reserve` after it admits the cost. A second call fails with the
    /// reservation-already-attempted cause, and a failed allocation or an unrepresentable size
    /// fails with its own cause. When the allocator refuses a larger capacity the guard requests
    /// the one record it needs instead; the meter charges the capacity it then finds.
    pub fn try_reserve(&mut self) -> Result<(), HtmlRelInterpretationError> {
        if self.attempted {
            return Err(HtmlRelInterpretationErrorKind::ReservationAlreadyAttempted.error());
        }
        self.attempted = true;
        let needed = self
            .len()
            .checked_add(1)
            .ok_or(HtmlRelInterpretationErrorKind::RepresentationExhausted)?;
        match &mut self.storage {
            ReservationStorage::Occurrences(v) => crate::amortized::reserve_amortized(v),
            ReservationStorage::Data(v) => crate::amortized::reserve_to(v, needed),
        }
        .map_err(|_| HtmlRelInterpretationErrorKind::AllocationFailed)?;
        self.completed = true;
        Ok(())
    }
}
/// Consumer-owned admission and allocation accounting.
pub trait HtmlRelMeter {
    /// The meter's own refusal, carried to the caller in `HtmlRelActionError::Admission`.
    type Error;
    /// Charges `units` of interpretation work before it is done. An error stops the interpretation.
    fn work(&mut self, units: usize) -> Result<(), Self::Error>;
    /// Admits and performs room for one more record by calling `HtmlRelReservation::try_reserve`
    /// exactly once, after it admits the capacity from `HtmlRelReservation::target_capacity`.
    /// Returning `Ok` without reserving is a protocol fault, reported as the
    /// reservation-not-performed cause.
    fn reserve(&mut self, reservation: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error>;
}
struct Meter<'a, M>(&'a mut M);
impl<M: HtmlRelMeter> Meter<'_, M> {
    fn reserve(
        &mut self,
        storage: ReservationStorage<'_>,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        let mut guard = HtmlRelReservation {
            storage,
            attempted: false,
            completed: false,
        };
        self.0
            .reserve(&mut guard)
            .map_err(HtmlRelActionError::Admission)?;
        if !guard.completed {
            return Err(HtmlRelActionError::Protocol(
                HtmlRelInterpretationErrorKind::ReservationNotPerformed.error(),
            ));
        }
        Ok(())
    }
}
impl<M: HtmlRelMeter> super::RelMeter for Meter<'_, M> {
    type Error = HtmlRelActionError<M::Error>;
    fn work(&mut self, units: usize) -> Result<(), Self::Error> {
        self.0.work(units).map_err(HtmlRelActionError::Admission)
    }
    fn reserve_occurrences(
        &mut self,
        values: &mut Vec<HtmlRelOccurrence>,
    ) -> Result<(), Self::Error> {
        self.reserve(ReservationStorage::Occurrences(values))
    }
    fn reserve_data(&mut self, values: &mut Vec<RelData>) -> Result<(), Self::Error> {
        self.reserve(ReservationStorage::Data(values))
    }
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Initial,
    Interpreted,
    Refining,
    Refreshed,
    Stopped,
}
/// Owns all unpublished backing, even after a consumer stops inside a rule.
#[derive(Debug)]
pub struct HtmlRelInterpretation<'raw> {
    inputs: RelInputs<'raw>,
    occurrences: Vec<HtmlRelOccurrence>,
    report: HtmlRelReport,
    phase: Phase,
    last_refinement: Option<usize>,
}
impl<'raw> HtmlRelInterpretation<'raw> {
    /// Starts an interpretation over `inputs`. Nothing runs until `interpret`.
    pub fn new(inputs: RelInputs<'raw>) -> Self {
        Self {
            inputs,
            occurrences: Vec::new(),
            report: HtmlRelReport { data: Vec::new() },
            phase: Phase::Initial,
            last_refinement: None,
        }
    }
    fn require(&self, test: impl FnOnce(Phase) -> bool) -> Result<(), HtmlRelInterpretationError> {
        if matches!(self.phase, Phase::Stopped) {
            Err(HtmlRelInterpretationErrorKind::Stopped.error())
        } else if test(self.phase) {
            Ok(())
        } else {
            Err(HtmlRelInterpretationErrorKind::WrongPhase.error())
        }
    }
    /// Interprets the inputs under `meter`, producing the occurrences and the summary. It is valid
    /// once, from the initial state. A refused charge or reservation stops the interpretation,
    /// which keeps its partial backing until dropped.
    pub fn interpret<M: HtmlRelMeter>(
        &mut self,
        meter: &mut M,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        self.require(|p| matches!(p, Phase::Initial))?;
        let result = super::interpret(
            &self.inputs,
            &mut self.occurrences,
            &mut self.report.data,
            &mut Meter(meter),
        );
        self.phase = if result.is_ok() {
            Phase::Interpreted
        } else {
            Phase::Stopped
        };
        result
    }
    /// The inputs this interpretation was created from. Valid after `interpret` has started; before
    /// then it fails with the wrong-phase cause, and after a stop with the stopped cause.
    pub fn inputs(&self) -> Result<&RelInputs<'raw>, HtmlRelInterpretationError> {
        self.require(|p| !matches!(p, Phase::Initial))?;
        Ok(&self.inputs)
    }
    /// The occurrences found, in source order. Valid under the same conditions as `inputs`.
    pub fn occurrences(&self) -> Result<&[HtmlRelOccurrence], HtmlRelInterpretationError> {
        self.require(|p| !matches!(p, Phase::Initial))?;
        Ok(self.report.occurrences())
    }
    /// The exact UTF-16 code units of the occurrence at `index`, or `Ok(None)` when out of range.
    /// Valid under the same conditions as `inputs`.
    pub fn occurrence_units(
        &self,
        index: usize,
    ) -> Result<Option<&[u16]>, HtmlRelInterpretationError> {
        self.require(|p| !matches!(p, Phase::Initial))?;
        Ok(self.report.occurrence_units(index))
    }
    /// Moves from the interpreted state to refinement. Valid only right after a successful
    /// `interpret`.
    pub fn begin_refinement(&mut self) -> Result<(), HtmlRelInterpretationError> {
        self.require(|p| matches!(p, Phase::Interpreted))?;
        self.phase = Phase::Refining;
        Ok(())
    }
    /// Applies a registry `refinement` to the unknown `rel` token at `index`, charging a fixed
    /// amount of work. Tokens must be refined in increasing index order, each at most once, only if
    /// their spelling is unknown and they came from `rel`; an extension effect must be valid for
    /// the role. Any violation is a protocol error that leaves the token unchanged, while a refused
    /// charge stops the interpretation.
    pub fn refine<M: HtmlRelMeter>(
        &mut self,
        index: usize,
        refinement: HtmlRelRefinement,
        meter: &mut M,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        use HtmlRelInterpretationErrorKind as E;
        self.require(|p| matches!(p, Phase::Refining))?;
        let token = self
            .report
            .occurrences()
            .get(index)
            .ok_or(HtmlRelActionError::Protocol(E::UnknownOccurrence.error()))?;
        let refusal = if token.source != HtmlRelOccurrenceSource::Rel {
            Some(E::WrongSource)
        } else if token.spelling != HtmlRelSpelling::Unknown {
            Some(E::WrongSpelling)
        } else if self.last_refinement == Some(index) {
            Some(E::RepeatedRefinement)
        } else if self.last_refinement.is_some_and(|prior| index < prior) {
            Some(E::OutOfOrderRefinement)
        } else if matches!(
            refinement,
            HtmlRelRefinement::Extension(
                HtmlRelEffect::InternalResource | HtmlRelEffect::StylesheetModifier
            )
        ) || matches!(
            refinement,
            HtmlRelRefinement::Extension(HtmlRelEffect::Annotation)
        ) && self.inputs.role == Some(HtmlRelRole::Link)
        {
            Some(E::InvalidExtensionEffect)
        } else {
            None
        };
        if let Some(error) = refusal {
            return Err(HtmlRelActionError::Protocol(error.error()));
        }
        if let Err(error) = meter.work(4) {
            self.stop();
            return Err(HtmlRelActionError::Admission(error));
        }
        let (keyword, effect, resolved) = match refinement {
            HtmlRelRefinement::Unavailable => (None, None, false),
            HtmlRelRefinement::NoEffect => (None, None, true),
            HtmlRelRefinement::Standard(keyword) => (
                Some(keyword),
                self.inputs.role.and_then(|role| keyword.effect(role)),
                true,
            ),
            HtmlRelRefinement::Extension(effect) => (None, Some(effect), true),
        };
        let token = &mut self.report.data[0].occurrences[index];
        token.keyword = keyword;
        token.effect = effect;
        token.registry_resolved = resolved;
        self.last_refinement = Some(index);
        Ok(())
    }
    /// Recomputes the summary (links, alternate use, authoring findings, body allowance and
    /// image-source preload context) from the refined tokens. It ends refinement; a refused charge
    /// or reservation stops the interpretation.
    pub fn refresh<M: HtmlRelMeter>(
        &mut self,
        meter: &mut M,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        self.require(|p| matches!(p, Phase::Refining))?;
        let result = super::refresh(&mut self.report, &self.inputs, &mut Meter(meter));
        self.phase = if result.is_ok() {
            Phase::Refreshed
        } else {
            Phase::Stopped
        };
        result
    }
    /// Abandons the interpretation. Every later action fails with the stopped cause, and
    /// `try_finish` reports it. Partial backing is released when the owner drops.
    pub fn stop(&mut self) {
        self.phase = Phase::Stopped;
    }
    /// Gives the inputs back and discards all partial results.
    pub fn abort(self) -> RelInputs<'raw> {
        self.inputs
    }
    /// Takes the finished report out of `slot`. It succeeds when the interpretation is interpreted
    /// or refreshed, and then leaves `None` in the slot. An empty slot fails with the consumed
    /// cause, a stopped interpretation with the stopped cause, and one mid-refinement with the
    /// not-ready cause; in those failures the slot keeps its owner.
    pub fn try_finish(
        slot: &mut Option<Self>,
    ) -> Result<HtmlRelReport, HtmlRelInterpretationError> {
        let owner = slot
            .as_ref()
            .ok_or(HtmlRelInterpretationErrorKind::Consumed)?;
        match owner.phase {
            Phase::Interpreted | Phase::Refreshed => Ok(slot.take().expect("checked owner").report),
            Phase::Stopped => Err(HtmlRelInterpretationErrorKind::Stopped.error()),
            _ => Err(HtmlRelInterpretationErrorKind::NotReady.error()),
        }
    }
}
/// HTML's existing bounded classifier over an already selected exact token.
pub fn classify_rel<M: HtmlRelMeter>(
    units: &[u16],
    meter: &mut M,
) -> Result<(Option<HtmlRelKeyword>, HtmlRelSpelling), HtmlRelActionError<M::Error>> {
    super::classify(units, &mut Meter(meter))
}
pub(super) struct OrdinaryRelMeter;
impl HtmlRelMeter for OrdinaryRelMeter {
    type Error = std::convert::Infallible;
    fn work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        guard.try_reserve().expect("ordinary rel allocation");
        Ok(())
    }
}

/// Tests whether a link `href` is empty or made only of ASCII whitespace (the Infra definition),
/// which the relationship grammar treats as the empty-or-only-ASCII-spaces case. The scan is
/// caller-metered: it never stops early on content, so every unit is inspected, and `work(6)` is
/// called once before inspecting each one. The first `work` error is returned unchanged and ends
/// the scan. The meter's units are abstract, and this per-unit weight is fixed by this scan.
pub fn rel_href_is_empty_or_ascii_whitespace_with<E>(
    units: &[u16],
    mut work: impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, E> {
    let mut only_whitespace = true;
    for &unit in units {
        work(6)?;
        only_whitespace &= super::ascii_whitespace(unit);
    }
    Ok(only_whitespace)
}
/// Exact-length, ASCII-insensitive comparison against one grammar keyword.
pub fn rel_ascii_keyword(units: &[u16], keyword: &str) -> bool {
    super::ascii_keyword(units, keyword)
}

#[cfg(test)]
mod ascii_helper_tests {
    use super::{rel_ascii_keyword, rel_href_is_empty_or_ascii_whitespace_with};

    /// Runs the scan with a meter that records each requested charge and refuses the call
    /// numbered `refuse_at` (one-based), when given.
    fn scan(units: &[u16], refuse_at: Option<usize>) -> (Result<bool, &'static str>, Vec<usize>) {
        let mut charges = Vec::new();
        let result = rel_href_is_empty_or_ascii_whitespace_with(units, |n| {
            charges.push(n);
            if Some(charges.len()) == refuse_at {
                Err("refused")
            } else {
                Ok(())
            }
        });
        (result, charges)
    }

    /// The documented charge is six per inspected unit, with no early exit on content; Infra
    /// ASCII whitespace excludes U+000B and U+00A0.
    #[test]
    fn every_unit_is_charged_six_and_content_never_ends_the_scan() {
        assert_eq!(scan(&[], None), (Ok(true), vec![]));
        assert_eq!(
            scan(&[0x20, 0x09, 0x0A, 0x0C, 0x0D], None),
            (Ok(true), vec![6; 5])
        );
        assert_eq!(
            scan(&[0x20, 0x41, 0x20, 0x20], None),
            (Ok(false), vec![6; 4])
        );
        assert_eq!(scan(&[0x41], None), (Ok(false), vec![6]));
        assert_eq!(scan(&[0x0B, 0x20], None), (Ok(false), vec![6; 2]));
        assert_eq!(scan(&[0xA0], None), (Ok(false), vec![6]));
        assert_eq!(scan(&[0xD800, 0x20], None), (Ok(false), vec![6; 2]));
    }

    /// Infra "ASCII case-insensitive": both sides lowercase, so an uppercase keyword byte matches
    /// either case of the unit, and a non-ASCII unit never folds.
    #[test]
    fn keyword_comparison_is_symmetric_ascii_case_insensitive() {
        assert!(rel_ascii_keyword(&[0x41, 0x62], "aB"));
        assert!(rel_ascii_keyword(&[0x61, 0x42], "Ab"));
        assert!(rel_ascii_keyword(&[0x61], "A"));
        assert!(!rel_ascii_keyword(&[0x212A], "k"));
        assert!(!rel_ascii_keyword(&[0x61], "ab"));
    }

    #[test]
    fn the_first_refused_charge_is_returned_unchanged_and_ends_the_scan() {
        assert_eq!(scan(&[0x20; 5], Some(3)), (Err("refused"), vec![6; 3]));
        assert_eq!(
            scan(&[0x41, 0x20, 0x20], Some(1)),
            (Err("refused"), vec![6])
        );
        // A refusal on the last unit still wins over the otherwise-known answer.
        assert_eq!(scan(&[0x41, 0x42], Some(2)), (Err("refused"), vec![6; 2]));
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    /// Every cause states its documented meaning, and exactly one of the
    /// cause predicates, its own, recognizes it.
    #[test]
    fn every_cause_has_its_message_and_exactly_one_matching_predicate() {
        type Predicate = fn(&HtmlRelInterpretationError) -> bool;
        let cases: Vec<(HtmlRelInterpretationError, &str, Predicate)> = vec![
            (
                HtmlRelInterpretationErrorKind::Consumed.error(),
                "rel interpretation was already consumed",
                HtmlRelInterpretationError::is_consumed,
            ),
            (
                HtmlRelInterpretationErrorKind::NotReady.error(),
                "rel interpretation has an unfinished step",
                HtmlRelInterpretationError::is_not_ready,
            ),
            (
                HtmlRelInterpretationErrorKind::Stopped.error(),
                "rel interpretation was stopped",
                HtmlRelInterpretationError::is_stopped,
            ),
            (
                HtmlRelInterpretationErrorKind::WrongPhase.error(),
                "rel interpretation step was requested out of order",
                HtmlRelInterpretationError::is_wrong_phase,
            ),
            (
                HtmlRelInterpretationErrorKind::UnknownOccurrence.error(),
                "rel refinement names an occurrence that does not exist",
                HtmlRelInterpretationError::is_unknown_occurrence,
            ),
            (
                HtmlRelInterpretationErrorKind::WrongSource.error(),
                "rel refinement does not match the source token",
                HtmlRelInterpretationError::is_wrong_source,
            ),
            (
                HtmlRelInterpretationErrorKind::WrongSpelling.error(),
                "rel refinement does not match the token spelling",
                HtmlRelInterpretationError::is_wrong_spelling,
            ),
            (
                HtmlRelInterpretationErrorKind::RepeatedRefinement.error(),
                "rel token was refined more than once",
                HtmlRelInterpretationError::is_repeated_refinement,
            ),
            (
                HtmlRelInterpretationErrorKind::OutOfOrderRefinement.error(),
                "rel refinements were not given in token order",
                HtmlRelInterpretationError::is_out_of_order_refinement,
            ),
            (
                HtmlRelInterpretationErrorKind::InvalidExtensionEffect.error(),
                "rel extension effect is not valid for its keyword",
                HtmlRelInterpretationError::is_invalid_extension_effect,
            ),
            (
                HtmlRelInterpretationErrorKind::ReservationNotPerformed.error(),
                "rel storage reservation was not performed",
                HtmlRelInterpretationError::is_reservation_not_performed,
            ),
            (
                HtmlRelInterpretationErrorKind::ReservationAlreadyAttempted.error(),
                "rel storage reservation was already attempted",
                HtmlRelInterpretationError::is_reservation_already_attempted,
            ),
            (
                HtmlRelInterpretationErrorKind::RepresentationExhausted.error(),
                "rel interpretation size cannot be represented on this platform",
                HtmlRelInterpretationError::is_representation_exhausted,
            ),
            (
                HtmlRelInterpretationErrorKind::AllocationFailed.error(),
                "reserving rel interpretation storage failed",
                HtmlRelInterpretationError::is_allocation_failed,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}
