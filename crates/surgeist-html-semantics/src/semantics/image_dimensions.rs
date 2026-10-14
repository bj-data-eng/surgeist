//! HTML img/input width/height getter interpretation, pinned HTML 877d6146487f.
//! Producers supply actual request availability, completed density/default-sizing
//! results and CSS content-box measurements. This owner selects HTML branches;
//! it does not decode images, execute CSS default sizing or perform Web IDL
//! conversion. Exact nonnegative rationals are the supported measurement domain.
//! Fact/native construction and cloning use ordinary allocation. Queries use the
//! sole semantic ledger for variable numeric copies and original raw admission;
//! atomic numeric operations have length-derived bounds, not a parser quantum.
//! Governing algorithms: [img dimensions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#img-dimensions),
//! [input dimensions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#input-img-dimensions)
//! and [determine dimensions](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#determine-dimensions).

use super::*;
use crate::{HtmlInputType, HtmlNonNegativeInteger, HtmlNonNegativeRational};
use surgeist_html::bounded::{
    IntegerPrefixScanner, NumericMaterializationError, NumericMaterializationFailure,
    NumericMaterialized, NumericParseError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionRole {
    Img,
    Input,
}

/// Independent expected keys; absence means that family was not qualified.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlImageDimensionRevision {
    resource: Option<HtmlNaturalDimensionRevision>,
    participation: Option<HtmlListParticipationRevision>,
}
impl HtmlImageDimensionRevision {
    pub fn new(
        resource: Option<HtmlNaturalDimensionRevision>,
        participation: Option<HtmlListParticipationRevision>,
    ) -> Self {
        Self {
            resource,
            participation,
        }
    }
    pub fn resource(&self) -> Option<&HtmlNaturalDimensionRevision> {
        self.resource.as_ref()
    }
    pub fn participation(&self) -> Option<&HtmlListParticipationRevision> {
        self.participation.as_ref()
    }
}

/// Known no selected resource does not by itself establish unavailability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionSelection {
    Selected(HtmlNaturalResourceKey),
    KnownNone,
    Unavailable,
}

/// Identity/configuration/revision of an actual completed external operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlImageSizingRevision {
    producer: HtmlParticipationProducerId,
    configuration: u64,
    revision: u64,
}
impl HtmlImageSizingRevision {
    pub fn new(producer: HtmlParticipationProducerId, configuration: u64, revision: u64) -> Self {
        Self {
            producer,
            configuration,
            revision,
        }
    }
    pub fn producer(&self) -> &HtmlParticipationProducerId {
        &self.producer
    }
    pub fn configuration(&self) -> u64 {
        self.configuration
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

/// The actual completed role and every influencing request/density/preferred
/// and CSS-sizing revision. These are supplied results, not geometry hints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionOperationKey {
    ImgPreferredDensityCorrected {
        operation: HtmlImageSizingRevision,
        request: u64,
        density: u64,
        preferred: u64,
    },
    ImgDefaultSized {
        operation: HtmlImageSizingRevision,
        request: u64,
        density: u64,
        preferred: u64,
        css: HtmlImageSizingRevision,
    },
    InputDefaultSized {
        operation: HtmlImageSizingRevision,
        request: u64,
        css: HtmlImageSizingRevision,
    },
}
impl HtmlImageDimensionOperationKey {
    fn role(&self) -> HtmlImageDimensionRole {
        match self {
            Self::InputDefaultSized { .. } => HtmlImageDimensionRole::Input,
            _ => HtmlImageDimensionRole::Img,
        }
    }
    fn request(&self) -> u64 {
        match self {
            Self::ImgPreferredDensityCorrected { request, .. }
            | Self::ImgDefaultSized { request, .. }
            | Self::InputDefaultSized { request, .. } => *request,
        }
    }
    fn origin(&self) -> HtmlImageDimensionOrigin {
        match self {
            Self::ImgPreferredDensityCorrected { .. } => {
                HtmlImageDimensionOrigin::ImgPreferredDensityCorrected
            }
            Self::ImgDefaultSized { .. } => HtmlImageDimensionOrigin::ImgDefaultSized,
            Self::InputDefaultSized { .. } => HtmlImageDimensionOrigin::InputDefaultSized,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlImageDimensionExpectation {
    subject: HtmlCaptureSubjectId,
    role: HtmlImageDimensionRole,
    selection: HtmlImageDimensionSelection,
    request: u64,
    operation: Option<HtmlImageDimensionOperationKey>,
}
impl HtmlImageDimensionExpectation {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        role: HtmlImageDimensionRole,
        selection: HtmlImageDimensionSelection,
        request: u64,
        operation: Option<HtmlImageDimensionOperationKey>,
    ) -> Self {
        Self {
            subject,
            role,
            selection,
            request,
            operation,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn role(&self) -> HtmlImageDimensionRole {
        self.role
    }
    pub fn selection(&self) -> &HtmlImageDimensionSelection {
        &self.selection
    }
    pub fn request_revision(&self) -> u64 {
        self.request
    }
    pub fn operation(&self) -> Option<&HtmlImageDimensionOperationKey> {
        self.operation.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImgRequestState {
    Unavailable,
    PartiallyAvailable,
    CompletelyAvailable,
    Broken,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputImageAvailability {
    Available,
    NotAvailable,
}
/// A completed axis, distinct from an absent natural intrinsic dimension.
#[derive(Clone, Debug)]
pub enum HtmlImageDimensionMeasurement {
    Known(HtmlNonNegativeRational),
    Unavailable,
    Unsupported(HtmlNaturalDimensionUnsupported),
}
impl PartialEq for HtmlImageDimensionMeasurement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Known(a), Self::Known(b)) => a.same_representation(b),
            (Self::Unavailable, Self::Unavailable) => true,
            (Self::Unsupported(a), Self::Unsupported(b)) => a == b,
            _ => false,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlImageDimensionCompleted {
    key: HtmlImageDimensionOperationKey,
    width: HtmlImageDimensionMeasurement,
    height: HtmlImageDimensionMeasurement,
}
impl HtmlImageDimensionCompleted {
    pub fn new(
        key: HtmlImageDimensionOperationKey,
        width: HtmlImageDimensionMeasurement,
        height: HtmlImageDimensionMeasurement,
    ) -> Self {
        Self { key, width, height }
    }
    pub fn key(&self) -> &HtmlImageDimensionOperationKey {
        &self.key
    }
    pub fn width(&self) -> &HtmlImageDimensionMeasurement {
        &self.width
    }
    pub fn height(&self) -> &HtmlImageDimensionMeasurement {
        &self.height
    }
}
#[derive(Debug)]
enum ImageAvailability {
    Img(Option<HtmlImgRequestState>),
    Input(Option<HtmlInputImageAvailability>),
}
impl ImageAvailability {
    fn role(&self) -> HtmlImageDimensionRole {
        match self {
            Self::Img(_) => HtmlImageDimensionRole::Img,
            Self::Input(_) => HtmlImageDimensionRole::Input,
        }
    }
    fn available(&self) -> Option<bool> {
        match self {
            Self::Img(state) => state.map(|s| {
                matches!(
                    s,
                    HtmlImgRequestState::PartiallyAvailable
                        | HtmlImgRequestState::CompletelyAvailable
                )
            }),
            Self::Input(state) => state.map(|s| s == HtmlInputImageAvailability::Available),
        }
    }
}
#[derive(Debug)]
pub struct HtmlImageDimensionResourceObservation {
    revision: HtmlNaturalDimensionRevision,
    expected: HtmlImageDimensionExpectation,
    availability: ImageAvailability,
    completed: Option<HtmlImageDimensionCompleted>,
}
impl HtmlImageDimensionResourceObservation {
    pub fn img(
        revision: HtmlNaturalDimensionRevision,
        expected: HtmlImageDimensionExpectation,
        state: Option<HtmlImgRequestState>,
        completed: Option<HtmlImageDimensionCompleted>,
    ) -> Self {
        Self {
            revision,
            expected,
            availability: ImageAvailability::Img(state),
            completed,
        }
    }
    pub fn input(
        revision: HtmlNaturalDimensionRevision,
        expected: HtmlImageDimensionExpectation,
        availability: Option<HtmlInputImageAvailability>,
        completed: Option<HtmlImageDimensionCompleted>,
    ) -> Self {
        Self {
            revision,
            expected,
            availability: ImageAvailability::Input(availability),
            completed,
        }
    }
    pub fn expectation(&self) -> &HtmlImageDimensionExpectation {
        &self.expected
    }
    pub fn revision(&self) -> &HtmlNaturalDimensionRevision {
        &self.revision
    }
    pub fn img_request_state(&self) -> Option<HtmlImgRequestState> {
        match self.availability {
            ImageAvailability::Img(v) => v,
            ImageAvailability::Input(_) => None,
        }
    }
    pub fn input_availability(&self) -> Option<HtmlInputImageAvailability> {
        match self.availability {
            ImageAvailability::Input(v) => v,
            ImageAvailability::Img(_) => None,
        }
    }
    pub fn completed(&self) -> Option<&HtmlImageDimensionCompleted> {
        self.completed.as_ref()
    }
}
/// Actual CSS content-box observation, not a second rendering classification.
/// The participation producer remains the authority for branch selection.
#[derive(Debug)]
pub struct HtmlImageContentBoxObservation {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    subject: HtmlCaptureSubjectId,
    revision: HtmlListParticipationRevision,
    width: HtmlImageDimensionMeasurement,
    height: HtmlImageDimensionMeasurement,
}
impl HtmlImageContentBoxObservation {
    pub fn new(
        capture: HtmlCaptureId,
        host: HtmlHostFactsRevision,
        subject: HtmlCaptureSubjectId,
        revision: HtmlListParticipationRevision,
        width: HtmlImageDimensionMeasurement,
        height: HtmlImageDimensionMeasurement,
    ) -> Self {
        Self {
            capture,
            host,
            subject,
            revision,
            width,
            height,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn revision(&self) -> &HtmlListParticipationRevision {
        &self.revision
    }
    pub fn width(&self) -> &HtmlImageDimensionMeasurement {
        &self.width
    }
    pub fn height(&self) -> &HtmlImageDimensionMeasurement {
        &self.height
    }
}

/// Incoming vector capacities, rational digit capacities and validation work.
/// These bound fact admission, independently of the sole query ledger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlImageDimensionFactsLimits {
    records: usize,
    bytes: usize,
    work: usize,
}
impl HtmlImageDimensionFactsLimits {
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
pub enum HtmlImageDimensionFactsError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    CaptureMismatch,
    RoleMismatch(HtmlCaptureSubjectId),
    DuplicateSubject(HtmlCaptureSubjectId),
    ResourceRevisionMissing,
    ResourceRevisionMismatch,
    ParticipationRevisionMismatch,
    UnexpectedObservation(HtmlCaptureSubjectId),
    BindingMismatch(HtmlCaptureSubjectId),
    InvalidOperation(HtmlCaptureSubjectId),
    InvalidAvailability(HtmlCaptureSubjectId),
    ContradictoryBox(HtmlCaptureSubjectId),
    ContradictoryResource(HtmlCaptureSubjectId),
    RecordLimit,
    ByteLimit,
    WorkLimit,
    RepresentationExhausted,
}
impl fmt::Display for HtmlImageDimensionFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlImageDimensionFactsError {}

/// Immutable joined getter evidence. No intrinsic-author fact is silently used
/// as a completed getter result. Missing families never fabricate a null image.
#[derive(Debug)]
pub struct HtmlImageDimensionFacts {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    revision: HtmlImageDimensionRevision,
    expected: Vec<HtmlImageDimensionExpectation>,
    resources: Vec<HtmlImageDimensionResourceObservation>,
    boxes: Vec<HtmlImageContentBoxObservation>,
    limits: HtmlImageDimensionFactsLimits,
    validation_work: usize,
}
struct ImageAdmission {
    work: usize,
    limit: usize,
}
impl ImageAdmission {
    fn charge(&mut self, n: usize) -> Result<(), HtmlImageDimensionFactsError> {
        self.work = self
            .work
            .checked_add(n)
            .ok_or(HtmlImageDimensionFactsError::RepresentationExhausted)?;
        if self.work > self.limit {
            Err(HtmlImageDimensionFactsError::WorkLimit)
        } else {
            Ok(())
        }
    }
    fn role(
        &mut self,
        capture: &HtmlCapture,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlImageDimensionRole, HtmlImageDimensionFactsError> {
        self.charge(1)?;
        let node = capture
            .node(subject)
            .map_err(HtmlImageDimensionFactsError::Capture)?;
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            return Err(HtmlImageDimensionFactsError::RoleMismatch(subject.clone()));
        };
        let n = e
            .name()
            .local_name()
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(e.name().namespace().uri().map_or(0, DocumentString::len)))
            .ok_or(HtmlImageDimensionFactsError::RepresentationExhausted)?;
        self.charge(n)?;
        match KnownHtmlElement::from_name(e.name()) {
            Some(KnownHtmlElement::Img) => Ok(HtmlImageDimensionRole::Img),
            Some(KnownHtmlElement::Input) => Ok(HtmlImageDimensionRole::Input),
            _ => Err(HtmlImageDimensionFactsError::RoleMismatch(subject.clone())),
        }
    }
}
fn measurement_bytes(
    v: &HtmlImageDimensionMeasurement,
) -> Result<usize, HtmlImageDimensionFactsError> {
    match v {
        HtmlImageDimensionMeasurement::Known(v) => v
            .retained_bytes()
            .ok_or(HtmlImageDimensionFactsError::RepresentationExhausted),
        _ => Ok(0),
    }
}
impl HtmlImageDimensionFacts {
    /// Moves all three vectors without tree or measurement clones. Admission
    /// counts their complete incoming capacities and every rational digit-buffer
    /// capacity. Duplicate/name/binding loops consume the validation-work cap.
    /// The same cap continues through participation/natural joins at context
    /// admission; this constructor alone does not certify those separate inputs.
    pub fn new(
        capture: &HtmlCapture,
        host: HtmlHostFactsRevision,
        revision: HtmlImageDimensionRevision,
        expected: Vec<HtmlImageDimensionExpectation>,
        resources: Vec<HtmlImageDimensionResourceObservation>,
        boxes: Vec<HtmlImageContentBoxObservation>,
        limits: HtmlImageDimensionFactsLimits,
    ) -> Result<Self, HtmlImageDimensionFactsError> {
        use HtmlImageDimensionFactsError as E;
        if &host != capture.host_facts_revision() {
            return Err(E::HostRevisionMismatch);
        }
        let capacities = [expected.capacity(), resources.capacity(), boxes.capacity()];
        let records = capacities
            .into_iter()
            .try_fold(0usize, |n, v| n.checked_add(v))
            .ok_or(E::RepresentationExhausted)?;
        if records > limits.records {
            return Err(E::RecordLimit);
        }
        let mut bytes = expected
            .capacity()
            .checked_mul(std::mem::size_of::<HtmlImageDimensionExpectation>())
            .and_then(|n| {
                resources
                    .capacity()
                    .checked_mul(std::mem::size_of::<HtmlImageDimensionResourceObservation>())
                    .and_then(|v| n.checked_add(v))
            })
            .and_then(|n| {
                boxes
                    .capacity()
                    .checked_mul(std::mem::size_of::<HtmlImageContentBoxObservation>())
                    .and_then(|v| n.checked_add(v))
            })
            .ok_or(E::RepresentationExhausted)?;
        if bytes > limits.bytes {
            return Err(E::ByteLimit);
        }
        if (!expected.is_empty() || !resources.is_empty()) && revision.resource.is_none() {
            return Err(E::ResourceRevisionMissing);
        }
        let mut a = ImageAdmission {
            work: 0,
            limit: limits.work,
        };
        for (i, e) in expected.iter().enumerate() {
            if a.role(capture, &e.subject)? != e.role {
                return Err(E::RoleMismatch(e.subject.clone()));
            }
            for prior in &expected[..i] {
                a.charge(1)?;
                if prior.subject == e.subject {
                    return Err(E::DuplicateSubject(e.subject.clone()));
                }
            }
            a.charge(1)?;
            if e.operation
                .as_ref()
                .is_some_and(|op| op.role() != e.role || op.request() != e.request)
            {
                return Err(E::InvalidOperation(e.subject.clone()));
            }
        }
        for (i, r) in resources.iter().enumerate() {
            a.charge(1)?;
            if revision.resource.as_ref() != Some(&r.revision) {
                return Err(E::ResourceRevisionMismatch);
            }
            for prior in &resources[..i] {
                a.charge(1)?;
                if prior.expected.subject == r.expected.subject {
                    return Err(E::DuplicateSubject(r.expected.subject.clone()));
                }
            }
            let mut binding = None;
            for e in &expected {
                a.charge(1)?;
                if e.subject == r.expected.subject {
                    binding = Some(e);
                    break;
                }
            }
            let Some(e) = binding else {
                return Err(E::UnexpectedObservation(r.expected.subject.clone()));
            };
            a.charge(1)?;
            if e != &r.expected {
                return Err(E::BindingMismatch(e.subject.clone()));
            }
            if r.availability.role() != e.role {
                return Err(E::RoleMismatch(e.subject.clone()));
            }
            if matches!(e.selection, HtmlImageDimensionSelection::Unavailable) {
                return Err(E::BindingMismatch(e.subject.clone()));
            }
            if r.availability.available() == Some(true)
                && !matches!(e.selection, HtmlImageDimensionSelection::Selected(_))
            {
                return Err(E::InvalidAvailability(e.subject.clone()));
            }
            if let Some(result) = &r.completed {
                a.charge(1)?;
                if r.availability.available() != Some(true) {
                    return Err(E::InvalidAvailability(e.subject.clone()));
                }
                if e.operation.as_ref() != Some(&result.key) {
                    return Err(E::InvalidOperation(e.subject.clone()));
                }
                for axis in [&result.width, &result.height] {
                    a.charge(1)?;
                    bytes = bytes
                        .checked_add(measurement_bytes(axis)?)
                        .ok_or(E::RepresentationExhausted)?;
                }
            }
            if bytes > limits.bytes {
                return Err(E::ByteLimit);
            }
        }
        for (i, b) in boxes.iter().enumerate() {
            a.role(capture, &b.subject)?;
            a.charge(3)?;
            if &b.capture != capture.identity() {
                return Err(E::CaptureMismatch);
            }
            if b.host != host {
                return Err(E::HostRevisionMismatch);
            }
            if revision.participation.as_ref() != Some(&b.revision) {
                return Err(E::ParticipationRevisionMismatch);
            }
            for prior in &boxes[..i] {
                a.charge(1)?;
                if prior.subject == b.subject {
                    return Err(E::DuplicateSubject(b.subject.clone()));
                }
            }
            for axis in [&b.width, &b.height] {
                a.charge(1)?;
                bytes = bytes
                    .checked_add(measurement_bytes(axis)?)
                    .ok_or(E::RepresentationExhausted)?;
            }
            if bytes > limits.bytes {
                return Err(E::ByteLimit);
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host,
            revision,
            expected,
            resources,
            boxes,
            limits,
            validation_work: a.work,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host
    }
    pub fn revision(&self) -> &HtmlImageDimensionRevision {
        &self.revision
    }
    pub fn expectations(&self) -> &[HtmlImageDimensionExpectation] {
        &self.expected
    }
    pub fn resource_observations(&self) -> &[HtmlImageDimensionResourceObservation] {
        &self.resources
    }
    pub fn content_boxes(&self) -> &[HtmlImageContentBoxObservation] {
        &self.boxes
    }
    pub(super) fn validate_joins(
        &mut self,
        participation: Option<&HtmlListParticipation>,
        natural: Option<&HtmlNaturalDimensions>,
    ) -> Result<(), HtmlImageDimensionFactsError> {
        use HtmlImageDimensionFactsError as E;
        let mut a = ImageAdmission {
            work: self.validation_work,
            limit: self.limits.work,
        };
        if let Some(p) = participation {
            a.charge(2)?;
            if p.capture() != &self.capture {
                return Err(E::CaptureMismatch);
            }
            if self
                .revision
                .participation
                .as_ref()
                .is_some_and(|expected| expected != p.revision())
            {
                return Err(E::ParticipationRevisionMismatch);
            }
            for b in &self.boxes {
                for f in p.facts() {
                    a.charge(1)?;
                    if f.subject() == &b.subject
                        && (f.being_rendered() == Some(false)
                            || f.produces_css_box() == Some(false))
                    {
                        return Err(E::ContradictoryBox(b.subject.clone()));
                    }
                }
            }
        }
        if let Some(n) = natural {
            for e in &self.expected {
                for f in n.facts() {
                    a.charge(1)?;
                    if f.subject() == &e.subject
                        && let (HtmlImageDimensionSelection::Selected(key), Some(other)) =
                            (&e.selection, f.resource())
                    {
                        a.charge(1)?;
                        if key != other {
                            return Err(E::ContradictoryResource(e.subject.clone()));
                        }
                    }
                    if f.subject() == &e.subject
                        && matches!(e.selection, HtmlImageDimensionSelection::KnownNone)
                        && f.resource().is_some()
                    {
                        return Err(E::ContradictoryResource(e.subject.clone()));
                    }
                }
            }
        }
        self.validation_work = a.work;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionOrigin {
    NonImageInputZero,
    ContentBox,
    ContentAttribute,
    UnavailableImageZero,
    ImgPreferredDensityCorrected,
    ImgDefaultSized,
    InputDefaultSized,
}
#[derive(Clone, Debug)]
pub struct HtmlImageDimensionValue {
    css_pixels: HtmlNonNegativeRational,
    origin: HtmlImageDimensionOrigin,
}
impl HtmlImageDimensionValue {
    pub fn css_pixels(&self) -> &HtmlNonNegativeRational {
        &self.css_pixels
    }
    pub fn origin(&self) -> HtmlImageDimensionOrigin {
        self.origin
    }
}
impl PartialEq for HtmlImageDimensionValue {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.css_pixels.same_representation(&other.css_pixels)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionMissingReason {
    ResourceObservation,
    Availability,
    CompletedWidth,
    CompletedHeight,
    ContentBoxWidth,
    ContentBoxHeight,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlImageDimensionMissing {
    subject: HtmlCaptureSubjectId,
    reason: HtmlImageDimensionMissingReason,
}
impl HtmlImageDimensionMissing {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn reason(&self) -> HtmlImageDimensionMissingReason {
        self.reason
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlImageDimensionAxisResult {
    Known(HtmlImageDimensionValue),
    Missing(HtmlImageDimensionMissing),
    Unsupported(HtmlNaturalDimensionUnsupported),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionRecovery {
    Recovered,
    InvalidSyntax,
    Negative,
}
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlImageDimensionAttributeObservation {
    Uninspected,
    Missing,
    Present {
        attribute: Attribute,
        recovery: HtmlImageDimensionRecovery,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageDimensionBranch {
    NonImageInput,
    RenderedContentBox,
    Unrendered,
}
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlImageDimensionReadback {
    role: HtmlImageDimensionRole,
    branch: HtmlImageDimensionBranch,
    width: HtmlImageDimensionAxisResult,
    height: HtmlImageDimensionAxisResult,
    width_attribute: HtmlImageDimensionAttributeObservation,
    height_attribute: HtmlImageDimensionAttributeObservation,
    operation: Option<HtmlImageDimensionOperationKey>,
}
impl HtmlImageDimensionReadback {
    pub fn role(&self) -> HtmlImageDimensionRole {
        self.role
    }
    pub fn branch(&self) -> HtmlImageDimensionBranch {
        self.branch
    }
    pub fn width(&self) -> &HtmlImageDimensionAxisResult {
        &self.width
    }
    pub fn height(&self) -> &HtmlImageDimensionAxisResult {
        &self.height
    }
    pub fn width_attribute(&self) -> &HtmlImageDimensionAttributeObservation {
        &self.width_attribute
    }
    pub fn height_attribute(&self) -> &HtmlImageDimensionAttributeObservation {
        &self.height_attribute
    }
    /// Actual completed operation used for an unoverridden axis. Bypassed
    /// resource facts and a known-null argument retain no operation here.
    pub fn operation(&self) -> Option<&HtmlImageDimensionOperationKey> {
        self.operation.as_ref()
    }
    pub fn source_clause(&self) -> &'static str {
        match self.role {
            HtmlImageDimensionRole::Img => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#img-dimensions"
            }
            HtmlImageDimensionRole::Input => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#input-img-dimensions"
            }
        }
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn image_dimensions(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlImageDimensionReadback>, HtmlSemanticFailure> {
        let role = match self.known(element)? {
            Some(KnownHtmlElement::Img) => HtmlImageDimensionRole::Img,
            Some(KnownHtmlElement::Input) => HtmlImageDimensionRole::Input,
            _ => return Ok(HtmlSemanticValue::Inapplicable),
        };
        let uninspected = HtmlImageDimensionAttributeObservation::Uninspected;
        if role == HtmlImageDimensionRole::Input {
            let state = self
                .control_input_type_report(element)?
                .intrinsic()
                .copied()
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            if state != HtmlInputType::Image {
                let width = self.image_zero(HtmlImageDimensionOrigin::NonImageInputZero)?;
                let height = self.image_zero(HtmlImageDimensionOrigin::NonImageInputZero)?;
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(HtmlImageDimensionReadback {
                    role,
                    branch: HtmlImageDimensionBranch::NonImageInput,
                    width,
                    height,
                    width_attribute: uninspected.clone(),
                    height_attribute: uninspected,
                    operation: None,
                }));
            }
        }
        let rendered = match self.participation(subject, HtmlParticipationField::Rendered)? {
            Ok(v) => v,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        if rendered {
            let mut measurement = None;
            if let Some(facts) = &self.context.image_dimensions {
                for b in &facts.boxes {
                    self.charge(1)?;
                    if &b.subject == subject {
                        measurement = Some(b);
                        break;
                    }
                }
            }
            let width = self.image_axis(
                subject,
                measurement.map(|b| &b.width),
                HtmlImageDimensionOrigin::ContentBox,
                HtmlImageDimensionMissingReason::ContentBoxWidth,
            )?;
            let height = self.image_axis(
                subject,
                measurement.map(|b| &b.height),
                HtmlImageDimensionOrigin::ContentBox,
                HtmlImageDimensionMissingReason::ContentBoxHeight,
            )?;
            self.charge(1)?;
            return Ok(HtmlSemanticValue::Known(HtmlImageDimensionReadback {
                role,
                branch: HtmlImageDimensionBranch::RenderedContentBox,
                width,
                height,
                width_attribute: uninspected.clone(),
                height_attribute: uninspected,
                operation: None,
            }));
        }
        let (width_override, width_attribute) = self.image_dimension_attribute(element, "width")?;
        let (height_override, height_attribute) =
            self.image_dimension_attribute(element, "height")?;
        // Once both content attributes recover, no unavailable resource evidence
        // can influence either result. Missing rendering was handled above.
        let resource = if width_override.is_some() && height_override.is_some() {
            None
        } else {
            let mut resource = None;
            if let Some(facts) = &self.context.image_dimensions {
                for r in &facts.resources {
                    self.charge(1)?;
                    if &r.expected.subject == subject {
                        resource = Some(r);
                        break;
                    }
                }
            }
            resource
        };
        let width = match width_override {
            Some(v) => v,
            None => self.image_resource_axis(subject, resource, true)?,
        };
        let height = match height_override {
            Some(v) => v,
            None => self.image_resource_axis(subject, resource, false)?,
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlImageDimensionReadback {
            role,
            branch: HtmlImageDimensionBranch::Unrendered,
            width,
            height,
            width_attribute,
            height_attribute,
            operation: resource
                .and_then(|r| r.completed.as_ref())
                .map(|c| c.key.clone()),
        }))
    }

    fn image_resource_axis(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        resource: Option<&HtmlImageDimensionResourceObservation>,
        width: bool,
    ) -> Result<HtmlImageDimensionAxisResult, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(resource) = resource else {
            return Ok(Self::image_missing(
                subject,
                HtmlImageDimensionMissingReason::ResourceObservation,
            ));
        };
        match resource.availability.available() {
            None => Ok(Self::image_missing(
                subject,
                HtmlImageDimensionMissingReason::Availability,
            )),
            Some(false) => self.image_zero(HtmlImageDimensionOrigin::UnavailableImageZero),
            Some(true) => {
                let reason = if width {
                    HtmlImageDimensionMissingReason::CompletedWidth
                } else {
                    HtmlImageDimensionMissingReason::CompletedHeight
                };
                let Some(completed) = &resource.completed else {
                    return Ok(Self::image_missing(subject, reason));
                };
                self.image_axis(
                    subject,
                    Some(if width {
                        &completed.width
                    } else {
                        &completed.height
                    }),
                    completed.key.origin(),
                    reason,
                )
            }
        }
    }
    fn image_missing(
        subject: &HtmlCaptureSubjectId,
        reason: HtmlImageDimensionMissingReason,
    ) -> HtmlImageDimensionAxisResult {
        HtmlImageDimensionAxisResult::Missing(HtmlImageDimensionMissing {
            subject: subject.clone(),
            reason,
        })
    }
    fn image_axis(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        axis: Option<&HtmlImageDimensionMeasurement>,
        origin: HtmlImageDimensionOrigin,
        reason: HtmlImageDimensionMissingReason,
    ) -> Result<HtmlImageDimensionAxisResult, HtmlSemanticFailure> {
        self.charge(1)?;
        match axis {
            Some(HtmlImageDimensionMeasurement::Known(v)) => {
                self.image_work(v.copy_work_bound())?;
                let result = v.try_copy(self.limits.integer_digits, self.image_remaining_bytes());
                let css_pixels = self.image_numeric_result(result)?;
                Ok(HtmlImageDimensionAxisResult::Known(
                    HtmlImageDimensionValue { css_pixels, origin },
                ))
            }
            Some(HtmlImageDimensionMeasurement::Unsupported(reason)) => {
                Ok(HtmlImageDimensionAxisResult::Unsupported(*reason))
            }
            _ => Ok(Self::image_missing(subject, reason)),
        }
    }
    fn image_zero(
        &mut self,
        origin: HtmlImageDimensionOrigin,
    ) -> Result<HtmlImageDimensionAxisResult, HtmlSemanticFailure> {
        self.image_work(HtmlNonNegativeInteger::from_u32_work_bound())?;
        let result = HtmlNonNegativeInteger::try_from_u32(
            0,
            self.limits.integer_digits,
            self.image_remaining_bytes(),
        );
        let integer = self.image_numeric_result(result)?;
        self.image_integer(integer, origin)
    }
    fn image_integer(
        &mut self,
        integer: HtmlNonNegativeInteger,
        origin: HtmlImageDimensionOrigin,
    ) -> Result<HtmlImageDimensionAxisResult, HtmlSemanticFailure> {
        self.image_work(HtmlNonNegativeRational::integer_work_bound())?;
        let result = HtmlNonNegativeRational::try_from_integer(
            integer,
            self.limits.integer_digits,
            self.image_remaining_bytes(),
        )
        .map_err(|failure| {
            // A sticky semantic stop abandons this unpublished numerator; its
            // allocation was already charged by its original materializer.
            let (failure, _numerator) = failure.into_parts();
            failure
        });
        let css_pixels = self.image_numeric_result(result)?;
        Ok(HtmlImageDimensionAxisResult::Known(
            HtmlImageDimensionValue { css_pixels, origin },
        ))
    }
    fn image_dimension_attribute(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<
        (
            Option<HtmlImageDimensionAxisResult>,
            HtmlImageDimensionAttributeObservation,
        ),
        HtmlSemanticFailure,
    > {
        let Some(attribute) = self.attribute(element, local)? else {
            return Ok((None, HtmlImageDimensionAttributeObservation::Missing));
        };
        self.interpreter
            .admit_attribute_raw(Some(attribute.value()))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let units = attribute.value().code_units();
        let mut scanner = IntegerPrefixScanner::new(true);
        for unit in units {
            self.charge(1)?;
            if !scanner.feed(*unit) {
                break;
            }
        }
        self.charge(1)?;
        let prefix = scanner.finish();
        let digits = match prefix {
            Ok(prefix) => {
                let length = prefix.end() - prefix.start();
                // The public operation checks the separately supplied digits,
                // then performs the original negative-zero inspection. Admit
                // fixed range/sign work and at most two visits per digit.
                self.charge(3)?;
                self.charge(length)?;
                if prefix.is_negative() {
                    self.charge(length)?;
                }
                prefix.try_nonnegative_digits(units)
            }
            Err(e) => Err(e),
        };
        let (value, recovery) = match digits {
            Ok(digits) => {
                self.image_work(HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                    digits.len(),
                ))?;
                let result = HtmlNonNegativeInteger::try_from_borrowed_digits(
                    digits,
                    self.limits.integer_digits,
                    self.image_remaining_bytes(),
                );
                let integer = self.image_numeric_result(result)?;
                (
                    Some(self.image_integer(integer, HtmlImageDimensionOrigin::ContentAttribute)?),
                    HtmlImageDimensionRecovery::Recovered,
                )
            }
            Err(NumericParseError::Negative) => (None, HtmlImageDimensionRecovery::Negative),
            Err(_) => (None, HtmlImageDimensionRecovery::InvalidSyntax),
        };
        self.charge(1)?;
        Ok((
            value,
            HtmlImageDimensionAttributeObservation::Present {
                attribute: attribute.clone(),
                recovery,
            },
        ))
    }
    fn image_remaining_bytes(&self) -> usize {
        self.limits
            .allocated_bytes
            .saturating_sub(self.usage.allocated_bytes)
    }
    fn image_work(&mut self, work: Option<usize>) -> Result<(), HtmlSemanticFailure> {
        let work =
            work.ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)
    }
    fn image_numeric_result<T>(
        &mut self,
        result: Result<NumericMaterialized<T>, NumericMaterializationFailure>,
    ) -> Result<T, HtmlSemanticFailure> {
        let baseline = self.usage.allocated_bytes;
        let observed = match &result {
            Ok(v) => v.capacity_bytes(),
            Err(e) => e.capacity_bytes(),
        };
        self.record_allocated_bytes(observed)?;
        let value = result
            .map_err(|e| {
                let kind = match e.kind() {
                    NumericMaterializationError::Digits(_) => {
                        HtmlSemanticFailureKind::IntrinsicUnavailable
                    }
                    NumericMaterializationError::Boundary(e) => match e {
                        IntegerUnitStepError::RepresentationExhausted => {
                            HtmlSemanticFailureKind::RepresentationExhausted
                        }
                        IntegerUnitStepError::AllocationFailed { .. } => {
                            HtmlSemanticFailureKind::AllocationFailed
                        }
                        IntegerUnitStepError::DigitLimitExceeded { limit, required } => {
                            HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::IntegerDigits,
                                limit,
                                required,
                            }
                        }
                        IntegerUnitStepError::RequestedCapacityLimitExceeded {
                            required, ..
                        } => match baseline.checked_add(required) {
                            Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::AllocatedBytes,
                                limit: self.limits.allocated_bytes,
                                required,
                            },
                            None => HtmlSemanticFailureKind::RepresentationExhausted,
                        },
                        IntegerUnitStepError::ActualCapacityLimitExceeded { .. } => {
                            HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::AllocatedBytes,
                                limit: self.limits.allocated_bytes,
                                required: self.usage.allocated_bytes,
                            }
                        }
                    },
                };
                self.stop(kind)
            })?
            .into_parts()
            .0;
        self.poll()?;
        Ok(value)
    }
}
