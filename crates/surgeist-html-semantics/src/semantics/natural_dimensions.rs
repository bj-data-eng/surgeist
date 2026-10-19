//! Resource-qualified intrinsic CSS-pixel dimension facts and author checks.
//! Pinned HTML [dimension attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dimension-attributes)
//! consumes the intrinsic quantities defined by CSS Images 3 CRD 2023-12-18
//! [natural dimensions](https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#natural-dimensions).
//! This is no loader, geometry or decoding operation. Producer truth remains
//! the adapter's responsibility; exact rationals are the supported API domain.

use super::*;
use crate::bounded::{NaturalComparison, NaturalDifference, NaturalDigits, NaturalProduct};
use crate::{HtmlNonNegativeInteger, HtmlNonNegativeRational};
use std::cmp::Ordering as CmpOrdering;

/// Opaque identity of the producer of exact intrinsic CSS-pixel measurements.
#[derive(Clone, Debug)]
pub struct HtmlNaturalDimensionProducerId(Arc<()>);
impl HtmlNaturalDimensionProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlNaturalDimensionProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlNaturalDimensionProducerId {}
/// Selected resource identity, independent of URLs and unrelated numeric tokens.
#[derive(Clone, Debug)]
pub struct HtmlNaturalResourceId(Arc<()>);
impl HtmlNaturalResourceId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlNaturalResourceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlNaturalResourceId {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlNaturalDimensionRevision {
    producer: HtmlNaturalDimensionProducerId,
    configuration: u64,
    selection: u64,
}
impl HtmlNaturalDimensionRevision {
    pub fn new(
        producer: HtmlNaturalDimensionProducerId,
        configuration: u64,
        selection: u64,
    ) -> Self {
        Self {
            producer,
            configuration,
            selection,
        }
    }
    pub fn producer(&self) -> &HtmlNaturalDimensionProducerId {
        &self.producer
    }
    pub fn configuration(&self) -> u64 {
        self.configuration
    }
    pub fn selection(&self) -> u64 {
        self.selection
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlNaturalResourceKey {
    resource: HtmlNaturalResourceId,
    content_revision: u64,
    metadata_revision: u64,
}
impl HtmlNaturalResourceKey {
    pub fn new(
        resource: HtmlNaturalResourceId,
        content_revision: u64,
        metadata_revision: u64,
    ) -> Self {
        Self {
            resource,
            content_revision,
            metadata_revision,
        }
    }
    pub fn resource(&self) -> &HtmlNaturalResourceId {
        &self.resource
    }
    pub fn content_revision(&self) -> u64 {
        self.content_revision
    }
    pub fn metadata_revision(&self) -> u64 {
        self.metadata_revision
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNaturalDimensionUnsupported {
    /// Established nonnegative non-finite presence, such as positive infinity.
    /// This never admits NaN or a negative value as a natural measurement.
    NonFinite,
    /// Established valid finite nonnegative presence outside exact rationals.
    NonRational,
}
/// Presence is distinct from representability and unavailable measurement.
#[derive(Clone, Debug)]
pub enum HtmlNaturalDimensionAxis {
    KnownAbsent,
    KnownPresent(HtmlNonNegativeRational),
    KnownPresentButUnrepresentable(HtmlNaturalDimensionUnsupported),
    Unavailable,
}
#[derive(Debug)]
enum NaturalObservation {
    SelectionUnavailable,
    SelectedUnavailable(HtmlNaturalResourceKey),
    Observed {
        revision: HtmlNaturalDimensionRevision,
        key: HtmlNaturalResourceKey,
        width: HtmlNaturalDimensionAxis,
        height: HtmlNaturalDimensionAxis,
    },
}
impl NaturalObservation {
    fn key(&self) -> Option<&HtmlNaturalResourceKey> {
        match self {
            Self::SelectionUnavailable => None,
            Self::SelectedUnavailable(k) | Self::Observed { key: k, .. } => Some(k),
        }
    }
}
/// One subject's independently expected binding or supplied coherent observation.
/// Constructors retain typed values without rounding or inferring absence.
#[derive(Debug)]
pub struct HtmlNaturalDimensionFact {
    subject: HtmlCaptureSubjectId,
    state: NaturalObservation,
}
impl HtmlNaturalDimensionFact {
    pub fn selection(subject: HtmlCaptureSubjectId, key: Option<HtmlNaturalResourceKey>) -> Self {
        Self {
            subject,
            state: key.map_or(
                NaturalObservation::SelectionUnavailable,
                NaturalObservation::SelectedUnavailable,
            ),
        }
    }
    pub fn observation(
        subject: HtmlCaptureSubjectId,
        revision: HtmlNaturalDimensionRevision,
        key: HtmlNaturalResourceKey,
        width: HtmlNaturalDimensionAxis,
        height: HtmlNaturalDimensionAxis,
    ) -> Self {
        Self {
            subject,
            state: NaturalObservation::Observed {
                revision,
                key,
                width,
                height,
            },
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn resource(&self) -> Option<&HtmlNaturalResourceKey> {
        self.state.key()
    }
    pub fn axes(&self) -> Option<(&HtmlNaturalDimensionAxis, &HtmlNaturalDimensionAxis)> {
        match &self.state {
            NaturalObservation::Observed { width, height, .. } => Some((width, height)),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Producer admission, separate from the semantic query ledger. Records bounds
/// the sum of incoming vector capacities; bytes includes both vector backing
/// and all incoming rational String capacities. Work bounds record visits,
/// qualification, duplicate comparisons and subject-join comparisons.
pub struct HtmlNaturalDimensionLimits {
    records: usize,
    bytes: usize,
    work: usize,
}
impl HtmlNaturalDimensionLimits {
    pub fn new(records: usize, bytes: usize, work: usize) -> Self {
        Self {
            records,
            bytes,
            work,
        }
    }
    pub fn records(self) -> usize {
        self.records
    }
    pub fn bytes(self) -> usize {
        self.bytes
    }
    pub fn work(self) -> usize {
        self.work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlNaturalDimensionFactsError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    DuplicateSubject(HtmlCaptureSubjectId),
    InvalidExpectedSelection,
    InvalidObservation,
    UnexpectedObservation(HtmlCaptureSubjectId),
    ResourceMismatch(HtmlCaptureSubjectId),
    SelectionRevisionMismatch(HtmlCaptureSubjectId),
    RecordLimit,
    ByteLimit,
    WorkLimit,
    RepresentationExhausted,
}
impl fmt::Display for HtmlNaturalDimensionFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlNaturalDimensionFactsError {}
/// Checked join against independent current selections. Retains one vector;
/// immutable incoming storage has its own finite admission, outside the query
/// ledger. Measurements must already include intrinsic resource-resolution
/// conversion; rounded DOM readbacks or specified/layout sizes are insufficient.
#[derive(Debug)]
pub struct HtmlNaturalDimensions {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    revision: HtmlNaturalDimensionRevision,
    facts: Vec<HtmlNaturalDimensionFact>,
}
impl HtmlNaturalDimensions {
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        revision: HtmlNaturalDimensionRevision,
        mut expected: Vec<HtmlNaturalDimensionFact>,
        mut observations: Vec<HtmlNaturalDimensionFact>,
        limits: HtmlNaturalDimensionLimits,
    ) -> Result<Self, HtmlNaturalDimensionFactsError> {
        use HtmlNaturalDimensionFactsError as E;
        if &host_revision != capture.host_facts_revision() {
            return Err(E::HostRevisionMismatch);
        }
        let records = expected
            .capacity()
            .checked_add(observations.capacity())
            .ok_or(E::RepresentationExhausted)?;
        if records > limits.records {
            return Err(E::RecordLimit);
        }
        let mut bytes = records
            .checked_mul(std::mem::size_of::<HtmlNaturalDimensionFact>())
            .ok_or(E::RepresentationExhausted)?;
        let mut work = 0usize;
        let mut charge = || {
            work = work.checked_add(1).ok_or(E::RepresentationExhausted)?;
            if work > limits.work {
                Err(E::WorkLimit)
            } else {
                Ok(())
            }
        };
        for facts in [&expected, &observations] {
            for (i, fact) in facts.iter().enumerate() {
                charge()?;
                capture.element_id(&fact.subject).map_err(E::Capture)?;
                for prior in &facts[..i] {
                    charge()?;
                    if prior.subject == fact.subject {
                        return Err(E::DuplicateSubject(fact.subject.clone()));
                    }
                }
                if let Some((w, h)) = fact.axes() {
                    if let NaturalObservation::Observed {
                        revision: actual, ..
                    } = &fact.state
                    {
                        charge()?;
                        if actual != &revision {
                            return Err(E::SelectionRevisionMismatch(fact.subject.clone()));
                        }
                    }
                    for axis in [w, h] {
                        charge()?;
                        if let HtmlNaturalDimensionAxis::KnownPresent(v) = axis {
                            bytes = bytes
                                .checked_add(v.retained_bytes().ok_or(E::RepresentationExhausted)?)
                                .ok_or(E::RepresentationExhausted)?;
                        }
                    }
                }
                if bytes > limits.bytes {
                    return Err(E::ByteLimit);
                }
            }
        }
        if bytes > limits.bytes {
            return Err(E::ByteLimit);
        }
        for fact in &mut expected {
            charge()?;
            if matches!(fact.state, NaturalObservation::Observed { .. }) {
                return Err(E::InvalidExpectedSelection);
            }
            let mut found = None;
            for (i, observation) in observations.iter().enumerate() {
                charge()?;
                if observation.subject == fact.subject {
                    found = Some(i);
                    break;
                }
            }
            if let Some(i) = found {
                charge()?;
                if !matches!(observations[i].state, NaturalObservation::Observed { .. }) {
                    return Err(E::InvalidObservation);
                }
                if fact.state.key() != observations[i].state.key() {
                    return Err(E::ResourceMismatch(fact.subject.clone()));
                }
                fact.state = observations.swap_remove(i).state;
            }
        }
        if let Some(extra) = observations.first() {
            return Err(E::UnexpectedObservation(extra.subject.clone()));
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            revision,
            facts: expected,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn revision(&self) -> &HtmlNaturalDimensionRevision {
        &self.revision
    }
    pub fn facts(&self) -> &[HtmlNaturalDimensionFact] {
        &self.facts
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDimensionAuthorSyntax {
    Missing,
    Unapplied,
    Valid,
    Invalid,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlDimensionApplicability {
    Applicable,
    Inapplicable,
    Unavailable(HtmlSemanticMissing),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNaturalDimensionMissing {
    Supplement,
    SubjectSelection,
    Selection,
    Observation,
    Width,
    Height,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDimensionOmission {
    NotApplicable,
    ApplicabilityUnavailable,
    OmittedBoth,
    PresentDimensions,
    Violated,
    Unavailable(HtmlNaturalDimensionMissing),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDimensionTolerance {
    BothZero,
    Width,
    Height,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDimensionPair {
    NotApplicable,
    ApplicabilityUnavailable,
    NotRequired,
    InvalidAuthoredInputs,
    Conforming(HtmlDimensionTolerance),
    Nonconforming,
    NaturalDimensionsAbsent,
    Unavailable(HtmlNaturalDimensionMissing),
    UnsupportedNaturalArithmetic,
}
#[derive(Clone, Debug, Eq, PartialEq)]
/// Fixed-size output sharing the original capture attributes. The outer query
/// result reservation owns its output record; raw strings are not recopied.
pub struct HtmlEmbeddedDimensionAuthorReadback {
    width: Option<Attribute>,
    height: Option<Attribute>,
    width_syntax: HtmlDimensionAuthorSyntax,
    height_syntax: HtmlDimensionAuthorSyntax,
    applicability: HtmlDimensionApplicability,
    omission: HtmlDimensionOmission,
    pair: HtmlDimensionPair,
    resource: Option<HtmlNaturalResourceKey>,
}
impl HtmlEmbeddedDimensionAuthorReadback {
    pub fn width_attribute(&self) -> Option<&Attribute> {
        self.width.as_ref()
    }
    pub fn height_attribute(&self) -> Option<&Attribute> {
        self.height.as_ref()
    }
    pub fn width_syntax(&self) -> HtmlDimensionAuthorSyntax {
        self.width_syntax
    }
    pub fn height_syntax(&self) -> HtmlDimensionAuthorSyntax {
        self.height_syntax
    }
    pub fn applicability(&self) -> &HtmlDimensionApplicability {
        &self.applicability
    }
    pub fn omission(&self) -> HtmlDimensionOmission {
        self.omission
    }
    pub fn pair(&self) -> HtmlDimensionPair {
        self.pair
    }
    pub fn resource(&self) -> Option<&HtmlNaturalResourceKey> {
        self.resource.as_ref()
    }
}

impl<'a> HtmlSemanticSession<'a> {
    fn embedded_dimension_applicability(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlDimensionApplicability, HtmlSemanticFailure> {
        use crate::bounded::{
            EmbeddedDimensionApplicability as A, EmbeddedDimensionContext as C,
            EmbeddedDimensionEvidenceNeeded as E, EmbeddedDimensionParent as P,
            embedded_dimension_applicability,
        };
        let known = self.known(element)?;
        self.charge(1)?;
        let selection = embedded_dimension_applicability(known, None);
        let selected = match selection {
            A::Unavailable(E::InputState) => {
                let state = self
                    .control_input_type_report(element)?
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                self.charge(1)?;
                embedded_dimension_applicability(known, Some(C::InputState(state)))
            }
            A::Unavailable(E::SourceParent) => {
                let node = self.node(subject)?;
                let parent = match self.parent(node)? {
                    Err(missing) => return Ok(HtmlDimensionApplicability::Unavailable(missing)),
                    Ok(None) => P::NoParent,
                    Ok(Some(id)) => {
                        let parent = self.node(&id)?;
                        let parent_known = match parent.kind() {
                            HtmlCapturedNodeKind::Element(e) => self.known(e)?,
                            _ => None,
                        };
                        P::Parent(parent_known)
                    }
                };
                self.charge(1)?;
                embedded_dimension_applicability(known, Some(C::SourceParent(parent)))
            }
            other => other,
        };
        Ok(match selected {
            A::Applicable => HtmlDimensionApplicability::Applicable,
            A::Inapplicable => HtmlDimensionApplicability::Inapplicable,
            A::DifferentContext | A::Unavailable(_) => {
                return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
            }
        })
    }
    pub(super) fn embedded_dimension_author_conformance(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlEmbeddedDimensionAuthorReadback>, HtmlSemanticFailure> {
        let width = self.attribute(element, "width")?;
        let height = self.attribute(element, "height")?;
        for raw in [width, height] {
            self.charge(1)?;
            self.interpreter
                .admit_attribute_raw(raw.map(Attribute::value))
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        }
        // Singular local image owner supplies the shared role/state/parent
        // decision. No dimension role inventory or input-state default lives here.
        let applicability = self.embedded_dimension_applicability(subject, element)?;
        let mut result = HtmlEmbeddedDimensionAuthorReadback {
            width: width.cloned(),
            height: height.cloned(),
            width_syntax: width.map_or(HtmlDimensionAuthorSyntax::Missing, |_| {
                HtmlDimensionAuthorSyntax::Unapplied
            }),
            height_syntax: height.map_or(HtmlDimensionAuthorSyntax::Missing, |_| {
                HtmlDimensionAuthorSyntax::Unapplied
            }),
            omission: HtmlDimensionOmission::NotApplicable,
            pair: HtmlDimensionPair::NotApplicable,
            applicability,
            resource: None,
        };
        if !matches!(result.applicability, HtmlDimensionApplicability::Applicable) {
            if matches!(
                result.applicability,
                HtmlDimensionApplicability::Unavailable(_)
            ) {
                result.omission = HtmlDimensionOmission::ApplicabilityUnavailable;
                result.pair = HtmlDimensionPair::ApplicabilityUnavailable;
            }
            return Ok(HtmlSemanticValue::Known(result));
        }
        let (ws, w) = self.dimension_author_integer(width)?;
        let (hs, h) = self.dimension_author_integer(height)?;
        result.width_syntax = ws;
        result.height_syntax = hs;
        if width.is_none() && height.is_none() {
            result.omission = HtmlDimensionOmission::OmittedBoth;
            result.pair = HtmlDimensionPair::NotRequired;
            return Ok(HtmlSemanticValue::Known(result));
        }
        let observation = self.natural_observation(subject)?;
        let (axes, missing) = match observation {
            None => (
                None,
                Some(if self.context.natural_dimensions.is_some() {
                    HtmlNaturalDimensionMissing::SubjectSelection
                } else {
                    HtmlNaturalDimensionMissing::Supplement
                }),
            ),
            Some(NaturalObservation::SelectionUnavailable) => {
                (None, Some(HtmlNaturalDimensionMissing::Selection))
            }
            Some(NaturalObservation::SelectedUnavailable(key)) => {
                result.resource = Some(key.clone());
                (None, Some(HtmlNaturalDimensionMissing::Observation))
            }
            Some(NaturalObservation::Observed {
                key, width, height, ..
            }) => {
                result.resource = Some(key.clone());
                (Some((width, height)), None)
            }
        };
        self.charge(4)?;
        use HtmlNaturalDimensionAxis as Axis;
        let absent = axes
            .is_some_and(|(w, h)| matches!(w, Axis::KnownAbsent) || matches!(h, Axis::KnownAbsent));
        let gap = missing.or_else(|| {
            axes.and_then(|(w, h)| {
                if matches!(w, Axis::Unavailable) {
                    Some(HtmlNaturalDimensionMissing::Width)
                } else if matches!(h, Axis::Unavailable) {
                    Some(HtmlNaturalDimensionMissing::Height)
                } else {
                    None
                }
            })
        });
        result.omission = if absent {
            HtmlDimensionOmission::Violated
        } else if let Some(gap) = gap {
            HtmlDimensionOmission::Unavailable(gap)
        } else {
            HtmlDimensionOmission::PresentDimensions
        };
        result.pair = if width.is_none() || height.is_none() {
            HtmlDimensionPair::NotRequired
        } else if ws == HtmlDimensionAuthorSyntax::Invalid
            || hs == HtmlDimensionAuthorSyntax::Invalid
        {
            HtmlDimensionPair::InvalidAuthoredInputs
        } else {
            let w = w.as_ref().expect("valid specified width");
            let h = h.as_ref().expect("valid specified height");
            if w.is_zero() && h.is_zero() {
                HtmlDimensionPair::Conforming(HtmlDimensionTolerance::BothZero)
            } else if absent {
                HtmlDimensionPair::NaturalDimensionsAbsent
            } else if let Some(gap) = gap {
                HtmlDimensionPair::Unavailable(gap)
            } else if let Some((Axis::KnownPresent(nw), Axis::KnownPresent(nh))) = axes {
                if nh.numerator().is_zero() {
                    HtmlDimensionPair::UnsupportedNaturalArithmetic
                } else if nw.numerator().is_zero() {
                    if w.is_zero() {
                        HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Width)
                    } else {
                        HtmlDimensionPair::UnsupportedNaturalArithmetic
                    }
                } else {
                    self.natural_ratio_check(w, h, nw, nh)?
                }
            } else {
                HtmlDimensionPair::UnsupportedNaturalArithmetic
            }
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(result))
    }
    fn natural_observation(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a NaturalObservation>, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(facts) = &self.context.natural_dimensions else {
            return Ok(None);
        };
        for fact in &facts.facts {
            self.charge(1)?;
            if &fact.subject == subject {
                self.charge(4)?;
                return Ok(Some(&fact.state));
            }
        }
        Ok(None)
    }
    fn dimension_author_integer(
        &mut self,
        attribute: Option<&Attribute>,
    ) -> Result<(HtmlDimensionAuthorSyntax, Option<HtmlNonNegativeInteger>), HtmlSemanticFailure>
    {
        let Some(attribute) = attribute else {
            return Ok((HtmlDimensionAuthorSyntax::Missing, None));
        };
        let units = attribute.value().code_units();
        // Numeric owns strict whole-value grammar as well as canonical storage.
        // Its existing linear atomic recognizer/materializer is precharged and
        // polled on both outcomes, without invoking processing recovery.
        let work = HtmlNonNegativeInteger::from_borrowed_digits_work_bound(units.len())
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)?;
        let remaining = self.remaining_allocated_bytes();
        let materialized = HtmlNonNegativeInteger::try_from_borrowed_digits(
            units,
            self.limits.integer_digits,
            remaining,
        );
        let baseline = self.usage.allocated_bytes;
        let observed = match &materialized {
            Ok(v) => v.capacity_bytes(),
            Err(e) => e.capacity_bytes(),
        };
        self.record_allocated_bytes(observed)?;
        use surgeist_html::bounded::{IntegerUnitStepError as B, NumericMaterializationError as E};
        if matches!(&materialized, Err(error) if matches!(error.kind(), E::Digits(_))) {
            self.poll()?;
            return Ok((HtmlDimensionAuthorSyntax::Invalid, None));
        }
        let value = materialized
            .map_err(|failure| {
                self.stop(match failure.kind() {
                    E::Digits(_) => HtmlSemanticFailureKind::IntrinsicUnavailable,
                    E::Boundary(B::RepresentationExhausted) => {
                        HtmlSemanticFailureKind::RepresentationExhausted
                    }
                    E::Boundary(B::AllocationFailed { .. }) => {
                        HtmlSemanticFailureKind::AllocationFailed
                    }
                    E::Boundary(B::DigitLimitExceeded { limit, required }) => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::IntegerDigits,
                            limit,
                            required,
                        }
                    }
                    E::Boundary(B::RequestedCapacityLimitExceeded { required, .. }) => {
                        match baseline.checked_add(required) {
                            Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::AllocatedBytes,
                                limit: self.limits.allocated_bytes,
                                required,
                            },
                            None => HtmlSemanticFailureKind::RepresentationExhausted,
                        }
                    }
                    E::Boundary(B::ActualCapacityLimitExceeded { .. }) => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::AllocatedBytes,
                            limit: self.limits.allocated_bytes,
                            required: self.usage.allocated_bytes,
                        }
                    }
                })
            })?
            .into_parts()
            .0;
        self.poll()?;
        Ok((HtmlDimensionAuthorSyntax::Valid, Some(value)))
    }
    fn dimension_digits(&mut self, length: usize) -> Result<(), HtmlSemanticFailure> {
        self.charge(1)?;
        if length > self.limits.integer_digits {
            Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::IntegerDigits,
                limit: self.limits.integer_digits,
                required: length,
            }))
        } else {
            Ok(())
        }
    }
    fn natural_product<'input, 'scratch>(
        &mut self,
        left: NaturalDigits<'input>,
        right: NaturalDigits<'input>,
        out: &'scratch mut Vec<u8>,
    ) -> Result<NaturalDigits<'scratch>, HtmlSemanticFailure> {
        let capacity = NaturalProduct::required_capacity(left, right)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.reserve_scratch(out, capacity)?;
        let mut cursor = NaturalProduct::new(left, right, out)
            .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        loop {
            if self.natural_product_step(&mut cursor)? {
                break;
            }
        }
        self.dimension_digits(cursor.scratch().len())?;
        let mut completed = Some(cursor);
        NaturalProduct::take(&mut completed)
            .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn natural_product_step(
        &mut self,
        cursor: &mut NaturalProduct<'_, '_>,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(8)?;
        Ok(cursor.step())
    }
    fn natural_compare(
        &mut self,
        left: NaturalDigits<'_>,
        right: NaturalDigits<'_>,
    ) -> Result<CmpOrdering, HtmlSemanticFailure> {
        let mut cursor = NaturalComparison::new(left, right);
        loop {
            if let Some(value) = self.natural_comparison_step(&mut cursor)? {
                return Ok(value);
            }
        }
    }
    fn natural_comparison_step(
        &mut self,
        cursor: &mut NaturalComparison<'_>,
    ) -> Result<Option<CmpOrdering>, HtmlSemanticFailure> {
        self.charge(8)?;
        Ok(cursor.step())
    }
    fn natural_difference_step(
        &mut self,
        cursor: &mut NaturalDifference<'_, '_>,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(8)?;
        Ok(cursor.step())
    }
    fn natural_ratio_check(
        &mut self,
        w: &HtmlNonNegativeInteger,
        h: &HtmlNonNegativeInteger,
        nw: &HtmlNonNegativeRational,
        nh: &HtmlNonNegativeRational,
    ) -> Result<HtmlDimensionPair, HtmlSemanticFailure> {
        let (mut n, mut d, mut wd, mut hn, mut a, mut b, mut delta) = (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let result = (|| {
            let n_digits = self.natural_product(
                NaturalDigits::integer(nw.numerator()),
                NaturalDigits::integer(nh.denominator()),
                &mut n,
            )?;
            let d_digits = self.natural_product(
                NaturalDigits::integer(nw.denominator()),
                NaturalDigits::integer(nh.numerator()),
                &mut d,
            )?;
            let wd_digits = self.natural_product(NaturalDigits::integer(w), d_digits, &mut wd)?;
            let hn_digits = self.natural_product(NaturalDigits::integer(h), n_digits, &mut hn)?;
            // This fixed checked operand is not a scan over caller scratch.
            let two = NaturalDigits::try_little(&[2]).expect("fixed canonical digit");
            let a_digits = self.natural_product(wd_digits, two, &mut a)?;
            let b_digits = self.natural_product(hn_digits, two, &mut b)?;
            let mut comparison = NaturalComparison::new(a_digits, b_digits);
            loop {
                if self.natural_comparison_step(&mut comparison)?.is_some() {
                    break;
                }
            }
            self.reserve_scratch(&mut delta, comparison.difference_capacity())?;
            let mut compared = Some(comparison);
            let mut cursor = NaturalComparison::take_difference(&mut compared, &mut delta)
                .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            loop {
                if self.natural_difference_step(&mut cursor)? {
                    break;
                }
            }
            self.dimension_digits(cursor.scratch().len())?;
            let mut completed = Some(cursor);
            let delta_digits = NaturalDifference::take(&mut completed)
                .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            if self.natural_compare(delta_digits, d_digits)? != CmpOrdering::Greater {
                return Ok(HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Width));
            }
            if self.natural_compare(delta_digits, n_digits)? != CmpOrdering::Greater {
                return Ok(HtmlDimensionPair::Conforming(
                    HtmlDimensionTolerance::Height,
                ));
            }
            Ok(HtmlDimensionPair::Nonconforming)
        })();
        for capacity in [
            n.capacity(),
            d.capacity(),
            wd.capacity(),
            hn.capacity(),
            a.capacity(),
            b.capacity(),
            delta.capacity(),
        ] {
            self.release_scratch(capacity);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlDocumentMode,
        HtmlScriptingStatus,
    };
    fn capture() -> HtmlCapture {
        let mut b = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let d = b
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        b.establish_children(&d, &[]).unwrap();
        b.finish().unwrap()
    }
    #[test]
    fn actual_product_difference_and_comparison_checkpoints_observe_real_cancellation_before_the_next_write()
     {
        let c = capture();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            100,
        )
        .unwrap();
        for phase in 0..3 {
            let token = HtmlSemanticCancellation::new();
            let mut s = HtmlSemanticSession::new(
                &c,
                &cx,
                HtmlValueInterpreter::new(100),
                HtmlSemanticLimits::new(100000, 100, 100, 100, 100000, 100),
                token.clone(),
            )
            .unwrap();
            let mut out = Vec::new();
            let err = match phase {
                0 => {
                    let left = NaturalDigits::try_little(&[9, 9]).unwrap();
                    let right = NaturalDigits::try_little(&[9, 9]).unwrap();
                    s.reserve_scratch(
                        &mut out,
                        NaturalProduct::required_capacity(left, right).unwrap(),
                    )
                    .unwrap();
                    let mut cursor = NaturalProduct::new(left, right, &mut out).unwrap();
                    for _ in 0..5 {
                        assert!(!s.natural_product_step(&mut cursor).unwrap());
                    }
                    let before = cursor.scratch().to_vec();
                    token.cancel();
                    let e = s.natural_product_step(&mut cursor).unwrap_err();
                    assert_eq!(cursor.scratch(), before);
                    e
                }
                1 => {
                    let mut comparison = NaturalComparison::new(
                        NaturalDigits::try_little(&[0, 0, 0, 1]).unwrap(),
                        NaturalDigits::try_little(&[1]).unwrap(),
                    );
                    while comparison.step().is_none() {}
                    s.reserve_scratch(&mut out, 4).unwrap();
                    let mut compared = Some(comparison);
                    let mut cursor =
                        NaturalComparison::take_difference(&mut compared, &mut out).unwrap();
                    assert!(!s.natural_difference_step(&mut cursor).unwrap());
                    let before = cursor.scratch().to_vec();
                    token.cancel();
                    let e = s.natural_difference_step(&mut cursor).unwrap_err();
                    assert_eq!(cursor.scratch(), before);
                    e
                }
                _ => {
                    let mut cursor = NaturalComparison::new(
                        NaturalDigits::try_little(&[1, 2]).unwrap(),
                        NaturalDigits::try_little(&[2, 2]).unwrap(),
                    );
                    assert_eq!(s.natural_comparison_step(&mut cursor).unwrap(), None);
                    token.cancel();
                    let e = s.natural_comparison_step(&mut cursor).unwrap_err();
                    // Test-only readback proves the cancelled production step
                    // did not consume the first unequal remaining digit.
                    assert_eq!(cursor.step(), Some(CmpOrdering::Less));
                    e
                }
            };
            s.release_scratch(out.capacity());
            assert_eq!(err.kind(), &HtmlSemanticFailureKind::Cancelled);
            assert_eq!(s.usage().scratch_records(), 0);
            assert_eq!(s.poll().unwrap_err(), err);
        }
    }
}
