//! One retained local relationship interpretation and its checked refinement.
use super::*;

/// The explicit original operands, independent of captured-tree qualification.
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
    pub fn with_rel(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.rel = raw;
        self
    }
    pub fn with_rev(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.rev = raw;
        self
    }
    pub fn with_href(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.href = raw;
        self
    }
    pub fn with_imagesrcset(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.imagesrcset = raw;
        self
    }
    pub fn with_itemprop(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.itemprop = raw;
        self
    }
    pub fn with_title(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.title = raw;
        self
    }
    pub fn with_type_hint(mut self, raw: Option<&'raw DocumentString>) -> Self {
        self.type_hint = raw;
        self
    }
    pub fn with_as_destination(mut self, destination: Option<HtmlLinkDestination>) -> Self {
        self.as_destination = destination;
        self
    }
    pub fn role(&self) -> Option<HtmlRelRole> {
        self.role
    }
    pub fn rel(&self) -> Option<&'raw DocumentString> {
        self.rel
    }
    pub fn rev(&self) -> Option<&'raw DocumentString> {
        self.rev
    }
    pub fn href(&self) -> Option<&'raw DocumentString> {
        self.href
    }
    pub fn imagesrcset(&self) -> Option<&'raw DocumentString> {
        self.imagesrcset
    }
    pub fn itemprop(&self) -> Option<&'raw DocumentString> {
        self.itemprop
    }
    pub fn title(&self) -> Option<&'raw DocumentString> {
        self.title
    }
    pub fn type_hint(&self) -> Option<&'raw DocumentString> {
        self.type_hint
    }
    pub fn as_destination(&self) -> Option<HtmlLinkDestination> {
        self.as_destination
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlRelInterpretationError {
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
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlRelActionError<E> {
    Admission(E),
    Protocol(HtmlRelInterpretationError),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRelRefinement {
    Unavailable,
    NoEffect,
    Standard(HtmlRelKeyword),
    Extension(HtmlRelEffect),
}

enum ReservationStorage<'a> {
    Occurrences(&'a mut Vec<HtmlRelOccurrence>),
    Data(&'a mut Vec<RelData>),
}
/// A fixed one-record reservation against the actual private output allocation.
pub struct HtmlRelReservation<'a> {
    storage: ReservationStorage<'a>,
    attempted: bool,
    completed: bool,
}
impl HtmlRelReservation<'_> {
    pub fn len(&self) -> usize {
        match &self.storage {
            ReservationStorage::Occurrences(v) => v.len(),
            ReservationStorage::Data(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn capacity(&self) -> usize {
        match &self.storage {
            ReservationStorage::Occurrences(v) => v.capacity(),
            ReservationStorage::Data(v) => v.capacity(),
        }
    }
    pub fn record_size(&self) -> usize {
        match self.storage {
            ReservationStorage::Occurrences(_) => std::mem::size_of::<HtmlRelOccurrence>(),
            ReservationStorage::Data(_) => std::mem::size_of::<RelData>(),
        }
    }
    pub fn try_reserve(&mut self) -> Result<(), HtmlRelInterpretationError> {
        if self.attempted {
            return Err(HtmlRelInterpretationError::ReservationAlreadyAttempted);
        }
        self.attempted = true;
        let needed = self
            .len()
            .checked_add(1)
            .ok_or(HtmlRelInterpretationError::RepresentationExhausted)?;
        if needed > self.capacity() {
            match &mut self.storage {
                ReservationStorage::Occurrences(v) => v.try_reserve_exact(1),
                ReservationStorage::Data(v) => v.try_reserve_exact(1),
            }
            .map_err(|_| HtmlRelInterpretationError::AllocationFailed)?;
        }
        self.completed = true;
        Ok(())
    }
}
/// Consumer-owned admission and allocation accounting.
pub trait HtmlRelMeter {
    type Error;
    fn work(&mut self, units: usize) -> Result<(), Self::Error>;
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
                HtmlRelInterpretationError::ReservationNotPerformed,
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
#[derive(Clone, Copy)]
enum Phase {
    Initial,
    Interpreted,
    Refining,
    Refreshed,
    Stopped,
}
/// Owns all unpublished backing, even after a consumer stops inside a rule.
pub struct HtmlRelInterpretation<'raw> {
    inputs: RelInputs<'raw>,
    occurrences: Vec<HtmlRelOccurrence>,
    report: HtmlRelReport,
    phase: Phase,
    last_refinement: Option<usize>,
}
impl<'raw> HtmlRelInterpretation<'raw> {
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
            Err(HtmlRelInterpretationError::Stopped)
        } else if test(self.phase) {
            Ok(())
        } else {
            Err(HtmlRelInterpretationError::WrongPhase)
        }
    }
    pub fn interpret<M: HtmlRelMeter>(
        &mut self,
        meter: &mut M,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        self.require(|p| matches!(p, Phase::Initial))
            .map_err(HtmlRelActionError::Protocol)?;
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
    pub fn inputs(&self) -> Result<&RelInputs<'raw>, HtmlRelInterpretationError> {
        self.require(|p| !matches!(p, Phase::Initial))?;
        Ok(&self.inputs)
    }
    pub fn occurrences(&self) -> Result<&[HtmlRelOccurrence], HtmlRelInterpretationError> {
        self.require(|p| !matches!(p, Phase::Initial))?;
        Ok(self.report.occurrences())
    }
    pub fn occurrence_units(
        &self,
        index: usize,
    ) -> Result<Option<&[u16]>, HtmlRelInterpretationError> {
        self.require(|p| !matches!(p, Phase::Initial))?;
        Ok(self.report.occurrence_units(index))
    }
    pub fn begin_refinement(&mut self) -> Result<(), HtmlRelInterpretationError> {
        self.require(|p| matches!(p, Phase::Interpreted))?;
        self.phase = Phase::Refining;
        Ok(())
    }
    pub fn refine<M: HtmlRelMeter>(
        &mut self,
        index: usize,
        refinement: HtmlRelRefinement,
        meter: &mut M,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        use HtmlRelInterpretationError as E;
        self.require(|p| matches!(p, Phase::Refining))
            .map_err(HtmlRelActionError::Protocol)?;
        let token = self
            .report
            .occurrences()
            .get(index)
            .ok_or(HtmlRelActionError::Protocol(E::UnknownOccurrence))?;
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
            return Err(HtmlRelActionError::Protocol(error));
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
    pub fn refresh<M: HtmlRelMeter>(
        &mut self,
        meter: &mut M,
    ) -> Result<(), HtmlRelActionError<M::Error>> {
        self.require(|p| matches!(p, Phase::Refining))
            .map_err(HtmlRelActionError::Protocol)?;
        let result = super::refresh(&mut self.report, &self.inputs, &mut Meter(meter));
        self.phase = if result.is_ok() {
            Phase::Refreshed
        } else {
            Phase::Stopped
        };
        result
    }
    pub fn stop(&mut self) {
        self.phase = Phase::Stopped;
    }
    pub fn abort(self) -> RelInputs<'raw> {
        self.inputs
    }
    pub fn try_finish(
        slot: &mut Option<Self>,
    ) -> Result<HtmlRelReport, HtmlRelInterpretationError> {
        let owner = slot.as_ref().ok_or(HtmlRelInterpretationError::Consumed)?;
        match owner.phase {
            Phase::Interpreted | Phase::Refreshed => Ok(slot.take().expect("checked owner").report),
            Phase::Stopped => Err(HtmlRelInterpretationError::Stopped),
            _ => Err(HtmlRelInterpretationError::NotReady),
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

/// ASCII case folding used by the relationship grammar's comparisons.
pub fn fold_rel_ascii(unit: u16) -> u16 {
    super::fold(unit)
}
/// Exact-length, ASCII-insensitive comparison against one grammar keyword.
pub fn rel_ascii_keyword(units: &[u16], keyword: &str) -> bool {
    super::ascii_keyword(units, keyword)
}
