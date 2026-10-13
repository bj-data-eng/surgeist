//! Ordinary-Document relationship selectors. Resource use remains external.
use super::*;
use crate::attributes::token_declarations::{IconSizeCursor, IconSizeShape};
use crate::tokens::{SpaceTokenCursor, SpaceTokenStep};
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDefaultFeedApplicability {
    Applicable,
    Inapplicable,
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDefaultFeedInput {
    capture: HtmlCaptureId,
    context: HtmlSemanticContextId,
    host: HtmlHostFactsRevision,
    document: HtmlCaptureSubjectId,
    applicability: HtmlDefaultFeedApplicability,
}
impl HtmlDefaultFeedInput {
    pub fn new(
        capture: &HtmlCapture,
        context: &HtmlSemanticContext,
        document: HtmlCaptureSubjectId,
        applicability: HtmlDefaultFeedApplicability,
        limits: HtmlRelFactsLimits,
    ) -> Result<Self, HtmlRelFactsError> {
        let mut admission = selector_admission(limits, std::mem::size_of::<Self>())?;
        validate_document(capture, context, &document, &mut admission)?;
        Ok(Self {
            capture: capture.identity().clone(),
            context: context.identity().clone(),
            host: context.host_revision().clone(),
            document,
            applicability,
        })
    }
    pub fn document(&self) -> &HtmlCaptureSubjectId {
        &self.document
    }
}
fn selector_admission(
    limits: HtmlRelFactsLimits,
    bytes: usize,
) -> Result<FactsAdmission, HtmlRelFactsError> {
    if limits.records < 1 {
        return Err(HtmlRelFactsError::RecordLimit);
    }
    if bytes > limits.bytes {
        return Err(HtmlRelFactsError::ByteLimit);
    }
    let mut a = FactsAdmission {
        limits,
        records: 1,
        bytes,
        work: 0,
    };
    a.work(1)?;
    Ok(a)
}
fn validate_document(
    capture: &HtmlCapture,
    context: &HtmlSemanticContext,
    document: &HtmlCaptureSubjectId,
    a: &mut FactsAdmission,
) -> Result<(), HtmlRelFactsError> {
    a.work(3)?;
    if context.capture() != capture.identity() {
        return Err(HtmlRelFactsError::SelectorBindingMismatch);
    }
    if context.host_revision() != capture.host_facts_revision() {
        return Err(HtmlRelFactsError::HostRevisionMismatch);
    }
    if !matches!(
        capture
            .node(document)
            .map_err(HtmlRelFactsError::Capture)?
            .kind(),
        HtmlCapturedNodeKind::Document(_)
    ) {
        return Err(HtmlRelFactsError::InvalidRole);
    }
    Ok(())
}

/// Exact original declaration field, including absence and attribute position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconAttribute {
    ordinal: usize,
    raw: DocumentString,
}
impl HtmlIconAttribute {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn raw(&self) -> &DocumentString {
        &self.raw
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIconSizeDeclaration {
    Scalable,
    Bitmap {
        width: Range<usize>,
        height: Range<usize>,
    },
    Invalid(crate::HtmlIconSizeError),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconSizeOccurrence {
    range: Range<usize>,
    declaration: HtmlIconSizeDeclaration,
    duplicate_of: Option<usize>,
}
impl HtmlIconSizeOccurrence {
    pub fn code_unit_range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn declaration(&self) -> &HtmlIconSizeDeclaration {
        &self.declaration
    }
    pub fn duplicate_of(&self) -> Option<usize> {
        self.duplicate_of
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIconSizesAuthor {
    NotSpecified,
    Conforming,
    Invalid {
        token_index: usize,
        reason: crate::HtmlIconSizeError,
    },
    Duplicate {
        first_index: usize,
        duplicate_index: usize,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconDeclaration {
    subject: HtmlCaptureSubjectId,
    rel: HtmlIconAttribute,
    type_hint: Option<HtmlIconAttribute>,
    media: Option<HtmlIconAttribute>,
    sizes: Option<HtmlIconAttribute>,
    href: Option<HtmlIconAttribute>,
    media_rules: HtmlMediaAttributeReadback,
    dimensions: Vec<HtmlIconSizeOccurrence>,
    sizes_author: HtmlIconSizesAuthor,
}
impl HtmlIconDeclaration {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn rel(&self) -> &HtmlIconAttribute {
        &self.rel
    }
    pub fn type_hint(&self) -> Option<&HtmlIconAttribute> {
        self.type_hint.as_ref()
    }
    pub fn media(&self) -> Option<&HtmlIconAttribute> {
        self.media.as_ref()
    }
    pub fn sizes(&self) -> Option<&HtmlIconAttribute> {
        self.sizes.as_ref()
    }
    pub fn href(&self) -> Option<&HtmlIconAttribute> {
        self.href.as_ref()
    }
    pub fn media_rules(&self) -> &HtmlMediaAttributeReadback {
        &self.media_rules
    }
    pub fn size_occurrences(&self) -> &[HtmlIconSizeOccurrence] {
        &self.dimensions
    }
    pub fn sizes_author(&self) -> &HtmlIconSizesAuthor {
        &self.sizes_author
    }
    /// Returns exact arbitrary-precision authored digits, without materialization.
    pub fn digits(&self, range: Range<usize>) -> Option<&[u16]> {
        self.sizes.as_ref()?.raw.code_units().get(range)
    }
}
/// Only the canonical complete traversal can construct this declaration basis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentIconCollection {
    capture: HtmlCaptureId,
    context: HtmlSemanticContextId,
    host: HtmlHostFactsRevision,
    document: HtmlCaptureSubjectId,
    media_revision: Option<HtmlMediaContextRevision>,
    candidates: Vec<HtmlIconDeclaration>,
}
impl HtmlDocumentIconCollection {
    pub fn document(&self) -> &HtmlCaptureSubjectId {
        &self.document
    }
    pub fn candidates(&self) -> &[HtmlIconDeclaration] {
        &self.candidates
    }
    pub fn media_revision(&self) -> Option<&HtmlMediaContextRevision> {
        self.media_revision.as_ref()
    }
}

macro_rules! icon_identity {
    ($name:ident) => {
        #[derive(Clone, Debug)]
        pub struct $name(Arc<()>);
        impl $name {
            pub fn fresh() -> Self {
                Self(Arc::new(()))
            }
        }
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                Arc::ptr_eq(&self.0, &other.0)
            }
        }
        impl Eq for $name {}
    };
}
icon_identity!(HtmlIconPresentationId);
icon_identity!(HtmlIconResourceId);
icon_identity!(HtmlIconExaminationId);
/// No universal icon score: policy and format observations belong to the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconExpectedRevisions {
    presentation: HtmlIconPresentationId,
    policy: HtmlRelRevision,
    resource: HtmlRelRevision,
    media: Option<HtmlMediaContextRevision>,
}
impl HtmlIconExpectedRevisions {
    pub fn new(
        presentation: HtmlIconPresentationId,
        policy: HtmlRelRevision,
        resource: HtmlRelRevision,
        media: Option<HtmlMediaContextRevision>,
    ) -> Self {
        Self {
            presentation,
            policy,
            resource,
            media,
        }
    }
    pub fn presentation(&self) -> &HtmlIconPresentationId {
        &self.presentation
    }
    pub fn policy(&self) -> &HtmlRelRevision {
        &self.policy
    }
    pub fn resource(&self) -> &HtmlRelRevision {
        &self.resource
    }
    pub fn media(&self) -> Option<&HtmlMediaContextRevision> {
        self.media.as_ref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconResourceKey {
    identity: HtmlIconResourceId,
    revision: HtmlRelRevision,
}
impl HtmlIconResourceKey {
    pub fn new(identity: HtmlIconResourceId, revision: HtmlRelRevision) -> Self {
        Self { identity, revision }
    }
    pub fn identity(&self) -> &HtmlIconResourceId {
        &self.identity
    }
    pub fn revision(&self) -> &HtmlRelRevision {
        &self.revision
    }
}
/// Each definitive external reason is independently decisive for this presentation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIconUnsuitableReason {
    UnsupportedType,
    UnsuitableSizes,
    UnusableResource,
    MediaDoesNotMatch,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIconSuitability {
    Suitable,
    Unsuitable(HtmlIconUnsuitableReason),
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconSuitabilityObservation {
    collection: IconCollectionKey,
    subject: HtmlCaptureSubjectId,
    resource: Option<HtmlIconResourceKey>,
    expected: HtmlIconExpectedRevisions,
    suitability: HtmlIconSuitability,
}
impl HtmlIconSuitabilityObservation {
    /// An attestation over the exact declaration in the collection supplied to
    /// formation; it does not fetch, decode, install or compare image formats.
    pub fn new(
        collection: &HtmlDocumentIconCollection,
        subject: HtmlCaptureSubjectId,
        resource: HtmlIconResourceKey,
        expected: HtmlIconExpectedRevisions,
        suitability: HtmlIconSuitability,
    ) -> Self {
        Self {
            collection: collection.key(),
            subject,
            resource: Some(resource),
            expected,
            suitability,
        }
    }
    /// An unobserved candidate needs no invented resource identity.
    pub fn unavailable(
        collection: &HtmlDocumentIconCollection,
        subject: HtmlCaptureSubjectId,
        expected: HtmlIconExpectedRevisions,
    ) -> Self {
        Self {
            collection: collection.key(),
            subject,
            resource: None,
            expected,
            suitability: HtmlIconSuitability::Unavailable,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn resource(&self) -> Option<&HtmlIconResourceKey> {
        self.resource.as_ref()
    }
    pub fn suitability(&self) -> HtmlIconSuitability {
        self.suitability
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct IconCollectionKey {
    capture: HtmlCaptureId,
    context: HtmlSemanticContextId,
    host: HtmlHostFactsRevision,
    document: HtmlCaptureSubjectId,
}
impl HtmlDocumentIconCollection {
    fn key(&self) -> IconCollectionKey {
        IconCollectionKey {
            capture: self.capture.clone(),
            context: self.context.clone(),
            host: self.host.clone(),
            document: self.document.clone(),
        }
    }
}
/// Ordered equivalence groups: best first. Member order never replaces tree order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIconOrdering {
    Known(Vec<Vec<HtmlCaptureSubjectId>>),
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentIconInput {
    data: Arc<IconInputData>,
}
#[derive(Debug, Eq, PartialEq)]
struct IconInputData {
    collection: HtmlDocumentIconCollection,
    expected: HtmlIconExpectedRevisions,
    observations: Vec<HtmlIconSuitabilityObservation>,
    ordering: HtmlIconOrdering,
    retained_bytes: usize,
    retained_records: usize,
}
impl HtmlDocumentIconInput {
    /// Moves the canonical complete collection and all finite external records.
    /// Expected revisions are independent of the observations being admitted.
    pub fn new(
        capture: &HtmlCapture,
        context: &HtmlSemanticContext,
        collection: HtmlDocumentIconCollection,
        expected: HtmlIconExpectedRevisions,
        mut observations: Vec<HtmlIconSuitabilityObservation>,
        ordering: HtmlIconOrdering,
        limits: HtmlRelFactsLimits,
    ) -> Result<Self, HtmlRelFactsError> {
        let mut a = selector_admission(
            limits,
            std::mem::size_of::<IconInputData>() + 2 * std::mem::size_of::<usize>(),
        )?;
        validate_document(capture, context, &collection.document, &mut a)?;
        a.work(5)?;
        if collection.capture != *capture.identity()
            || collection.context != *context.identity()
            || collection.host != *context.host_revision()
            || collection.media_revision != expected.media
            || context.media_revision() != expected.media.as_ref()
        {
            return Err(HtmlRelFactsError::SelectorBindingMismatch);
        }
        a.vector(&collection.candidates)?;
        a.vector(&observations)?;
        for declaration in &collection.candidates {
            a.work(1)?;
            a.vector(&declaration.dimensions)?;
            for field in [
                Some(&declaration.rel),
                declaration.type_hint.as_ref(),
                declaration.media.as_ref(),
                declaration.sizes.as_ref(),
                declaration.href.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                a.raw(&field.raw)?;
            }
            // Canonical media readback owns its own nested fixed record.
            let (records, bytes) = declaration.media_rules.owned_backing();
            a.records = a
                .records
                .checked_add(records)
                .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
            if a.records > a.limits.records {
                return Err(HtmlRelFactsError::RecordLimit);
            }
            a.bytes = a
                .bytes
                .checked_add(bytes)
                .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
            if a.bytes > a.limits.bytes {
                return Err(HtmlRelFactsError::ByteLimit);
            }
            let mut count = 0;
            for observation in &observations {
                a.work(1)?;
                if observation.subject == declaration.subject {
                    count += 1;
                }
            }
            if count != 1 {
                return Err(HtmlRelFactsError::IconObservationCoverage);
            }
        }
        for observation in &observations {
            a.work(4)?;
            if observation.collection != collection.key()
                || observation.expected != expected
                || observation
                    .resource
                    .as_ref()
                    .is_some_and(|r| r.revision != expected.resource)
            {
                return Err(HtmlRelFactsError::SelectorBindingMismatch);
            }
            let mut matched = None;
            for candidate in &collection.candidates {
                a.work(1)?;
                if candidate.subject == observation.subject {
                    matched = Some(candidate);
                    break;
                }
            }
            let Some(candidate) = matched else {
                return Err(HtmlRelFactsError::IconObservationCoverage);
            };
            let HtmlMediaProcessing::Link(rules) = candidate.media_rules.processing() else {
                return Err(HtmlRelFactsError::InvalidRole);
            };
            a.work(2)?;
            let mismatch =
                rules.external_resource_gate().condition() == &HtmlMediaCondition::CssDoesNotMatch;
            if mismatch && observation.suitability == HtmlIconSuitability::Suitable
                || !mismatch
                    && observation.suitability
                        == HtmlIconSuitability::Unsuitable(
                            HtmlIconUnsuitableReason::MediaDoesNotMatch,
                        )
            {
                return Err(HtmlRelFactsError::IconMediaContradiction);
            }
        }
        if let HtmlIconOrdering::Known(groups) = &ordering {
            a.vector(groups)?;
            for group in groups {
                a.vector(group)?;
                a.work(1)?;
                if group.is_empty() {
                    return Err(HtmlRelFactsError::IconOrderingInvalid);
                }
                for member in group {
                    let mut suitable = false;
                    for observation in &observations {
                        a.work(2)?;
                        if observation.subject == *member
                            && observation.suitability == HtmlIconSuitability::Suitable
                        {
                            suitable = true;
                            break;
                        }
                    }
                    if !suitable {
                        return Err(HtmlRelFactsError::IconOrderingInvalid);
                    }
                }
            }
            for observation in &observations {
                let mut count = 0;
                for group in groups {
                    for member in group {
                        a.work(1)?;
                        if *member == observation.subject {
                            count += 1;
                        }
                    }
                }
                a.work(1)?;
                if count != usize::from(observation.suitability == HtmlIconSuitability::Suitable) {
                    return Err(HtmlRelFactsError::IconOrderingInvalid);
                }
            }
        }
        // Normalize record position against actual tree order, without allocating.
        for (index, declaration) in collection.candidates.iter().enumerate() {
            for position in index..observations.len() {
                a.work(1)?;
                if observations[position].subject == declaration.subject {
                    a.work(1)?;
                    observations.swap(index, position);
                    break;
                }
            }
        }
        a.work(1)?;
        let retained_bytes = a.bytes;
        let retained_records = a.records;
        Ok(Self {
            data: Arc::new(IconInputData {
                collection,
                expected,
                observations,
                ordering,
                retained_bytes,
                retained_records,
            }),
        })
    }
    pub fn collection(&self) -> &HtmlDocumentIconCollection {
        &self.data.collection
    }
    pub fn expected(&self) -> &HtmlIconExpectedRevisions {
        &self.data.expected
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentIconChoice {
    input: HtmlDocumentIconInput,
    index: usize,
    rejections: Vec<HtmlIconRejectionRecord>,
}
impl HtmlDocumentIconChoice {
    pub fn declaration(&self) -> &HtmlIconDeclaration {
        &self.input.data.collection.candidates[self.index]
    }
    pub fn resource(&self) -> &HtmlIconResourceKey {
        self.input.data.observations[self.index]
            .resource
            .as_ref()
            .expect("selected suitable observation owns an actual resource key")
    }
    pub fn input(&self) -> &HtmlDocumentIconInput {
        &self.input
    }
    pub fn rejected_count(&self) -> usize {
        self.rejections.len()
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIconCloserExamination {
    AttemptedThenInappropriate,
    NotAttempted,
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconExaminationObservation {
    identity: HtmlIconExaminationId,
    subject: HtmlCaptureSubjectId,
    resource: HtmlIconResourceKey,
    revision: HtmlRelRevision,
    outcome: HtmlIconCloserExamination,
}
impl HtmlIconExaminationObservation {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        resource: HtmlIconResourceKey,
        identity: HtmlIconExaminationId,
        revision: HtmlRelRevision,
        outcome: HtmlIconCloserExamination,
    ) -> Self {
        Self {
            identity,
            subject,
            resource,
            revision,
            outcome,
        }
    }
    pub fn identity(&self) -> &HtmlIconExaminationId {
        &self.identity
    }
    pub fn revision(&self) -> &HtmlRelRevision {
        &self.revision
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct HtmlIconRejectionRecord {
    index: usize,
    observation: HtmlIconExaminationObservation,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlNextDocumentIconInput {
    data: Arc<NextIconData>,
}
#[derive(Debug, Eq, PartialEq)]
struct NextIconData {
    previous: HtmlDocumentIconChoice,
    examination: HtmlIconExaminationObservation,
}
impl HtmlNextDocumentIconInput {
    /// Only an actual selected result can authorize the next-most operation.
    pub fn new(
        previous: HtmlDocumentIconChoice,
        expected_examination: &HtmlRelRevision,
        examination: HtmlIconExaminationObservation,
        limits: HtmlRelFactsLimits,
    ) -> Result<Self, HtmlRelFactsError> {
        let mut a = selector_admission(
            limits,
            std::mem::size_of::<NextIconData>() + 2 * std::mem::size_of::<usize>(),
        )?;
        a.vector(&previous.rejections)?;
        a.work(4)?;
        if previous.declaration().subject != examination.subject
            || previous.resource() != &examination.resource
            || &examination.revision != expected_examination
        {
            return Err(HtmlRelFactsError::SelectorBindingMismatch);
        }
        if examination.outcome == HtmlIconCloserExamination::NotAttempted {
            return Err(HtmlRelFactsError::IconNotAttempted);
        }
        for rejection in &previous.rejections {
            a.work(1)?;
            if rejection.observation.identity == examination.identity {
                return Err(HtmlRelFactsError::DuplicateObservation);
            }
        }
        // The already-formed input is retained, rather than copying its vectors.
        a.bytes = a
            .bytes
            .checked_add(previous.input.data.retained_bytes)
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        a.records = a
            .records
            .checked_add(previous.input.data.retained_records)
            .ok_or(HtmlRelFactsError::RepresentationExhausted)?;
        if a.records > a.limits.records {
            return Err(HtmlRelFactsError::RecordLimit);
        }
        if a.bytes > a.limits.bytes {
            return Err(HtmlRelFactsError::ByteLimit);
        }
        Ok(Self {
            data: Arc::new(NextIconData {
                previous,
                examination,
            }),
        })
    }
    pub fn document(&self) -> &HtmlCaptureSubjectId {
        self.data.previous.input.collection().document()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlDocumentIconReadback {
    Declarations {
        collection: HtmlDocumentIconCollection,
        selection: HtmlSemanticValue<Option<HtmlDocumentIconChoice>>,
    },
    Selection(HtmlSemanticValue<Option<HtmlDocumentIconChoice>>),
}

impl<'a> HtmlSemanticSession<'a> {
    fn icon_input_binding(
        &mut self,
        document: &HtmlCaptureSubjectId,
        input: &HtmlDocumentIconInput,
    ) -> Result<(), HtmlSemanticFailure> {
        self.charge(5)?;
        let c = input.collection();
        if c.capture != *self.capture.identity()
            || c.context != *self.context.identity()
            || c.host != *self.context.host_revision()
            || c.document != *document
            || input.expected().media() != self.context.media_revision()
        {
            return Err(self.stop(HtmlSemanticFailureKind::RelationshipSelectorRequestMismatch));
        }
        // Request retention is already finite and accounted at formation. Recheck
        // actual original selected values against this session's raw admission.
        for candidate in &c.candidates {
            for field in [
                Some(&candidate.rel),
                candidate.type_hint.as_ref(),
                candidate.media.as_ref(),
                candidate.sizes.as_ref(),
                candidate.href.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                self.charge(1)?;
                self.interpreter
                    .admit_attribute_raw(Some(&field.raw))
                    .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
            }
        }
        Ok(())
    }
    fn choose_icon(
        &mut self,
        document: &HtmlCaptureSubjectId,
        input: &HtmlDocumentIconInput,
        rejections: Vec<HtmlIconRejectionRecord>,
    ) -> Result<HtmlSemanticValue<Option<HtmlDocumentIconChoice>>, HtmlSemanticFailure> {
        self.icon_input_binding(document, input)?;
        for (index, observation) in input.data.observations.iter().enumerate() {
            self.charge(1)?;
            let HtmlMediaProcessing::Link(rules) = input.data.collection.candidates[index]
                .media_rules
                .processing()
            else {
                unreachable!("formed actual Link")
            };
            self.charge(1)?;
            let gate = rules.external_resource_gate();
            // A definitive canonical media exclusion makes external suitability
            // and relative ordering noninfluencing for this declaration.
            if gate.condition() == &HtmlMediaCondition::CssDoesNotMatch {
                continue;
            }
            if observation.suitability == HtmlIconSuitability::Unavailable {
                return Ok(HtmlSemanticValue::Missing(Self::rel_missing(
                    &observation.subject,
                    HtmlRelMissingFact::IconSuitability,
                )));
            }
            if observation.suitability == HtmlIconSuitability::Suitable {
                self.charge(1)?;
                match gate.condition() {
                    HtmlMediaCondition::Missing(m) => {
                        return Ok(HtmlSemanticValue::Missing(m.clone()));
                    }
                    HtmlMediaCondition::Unsupported(c) => {
                        return Ok(HtmlSemanticValue::Unsupported(
                            HtmlSemanticUnsupported::IconMedia(*c),
                        ));
                    }
                    HtmlMediaCondition::CssDoesNotMatch => {
                        return Err(
                            self.stop(HtmlSemanticFailureKind::RelationshipSelectorRequestMismatch)
                        );
                    }
                    _ => {}
                }
            }
        }
        let mut any_suitable = false;
        for observation in &input.data.observations {
            self.charge(1)?;
            any_suitable |= observation.suitability == HtmlIconSuitability::Suitable;
        }
        if !any_suitable {
            return Ok(HtmlSemanticValue::Known(None));
        }
        let HtmlIconOrdering::Known(groups) = &input.data.ordering else {
            return Ok(HtmlSemanticValue::Missing(Self::rel_missing(
                document,
                HtmlRelMissingFact::IconOrdering,
            )));
        };
        for group in groups {
            let mut selected = None;
            for (index, declaration) in input.data.collection.candidates.iter().enumerate() {
                let mut excluded = false;
                for rejection in &rejections {
                    self.charge(3)?;
                    if rejection.index == index {
                        // Every record arose from an actual previous winner and
                        // admitted attempted-use rejection; original input is fixed.
                        if rejection.observation.subject != declaration.subject
                            || Some(&rejection.observation.resource)
                                != input.data.observations[index].resource.as_ref()
                            || rejection.observation.outcome
                                != HtmlIconCloserExamination::AttemptedThenInappropriate
                        {
                            return Err(self.stop(
                                HtmlSemanticFailureKind::RelationshipSelectorRequestMismatch,
                            ));
                        }
                        excluded = true;
                        break;
                    }
                }
                if excluded {
                    continue;
                }
                for member in group {
                    self.charge(1)?;
                    if *member == declaration.subject {
                        selected = Some(index);
                        break;
                    }
                }
            }
            if let Some(index) = selected {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(Some(HtmlDocumentIconChoice {
                    input: input.clone(),
                    index,
                    rejections,
                })));
            }
        }
        Ok(HtmlSemanticValue::Known(None))
    }
    pub(in super::super) fn document_icon(
        &mut self,
        document: &HtmlCaptureSubjectId,
        input: Option<&HtmlDocumentIconInput>,
    ) -> Result<HtmlSemanticValue<HtmlDocumentIconReadback>, HtmlSemanticFailure> {
        if let Some(input) = input {
            return Ok(HtmlSemanticValue::Known(
                HtmlDocumentIconReadback::Selection(self.choose_icon(
                    document,
                    input,
                    Vec::new(),
                )?),
            ));
        }
        Ok(self.document_icon_collection(document)?.map(|collection| {
            let selection = if collection.candidates.is_empty() {
                HtmlSemanticValue::Known(None)
            } else {
                HtmlSemanticValue::Missing(Self::rel_missing(
                    document,
                    HtmlRelMissingFact::IconSuitability,
                ))
            };
            HtmlDocumentIconReadback::Declarations {
                collection,
                selection,
            }
        }))
    }
    pub(in super::super) fn next_document_icon(
        &mut self,
        document: &HtmlCaptureSubjectId,
        input: Option<&HtmlNextDocumentIconInput>,
    ) -> Result<HtmlSemanticValue<HtmlDocumentIconReadback>, HtmlSemanticFailure> {
        if !matches!(
            self.node(document)?.kind(),
            HtmlCapturedNodeKind::Document(_)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let Some(input) = input else {
            return Ok(HtmlSemanticValue::Missing(Self::rel_missing(
                document,
                HtmlRelMissingFact::IconCloserExamination,
            )));
        };
        let previous = &input.data.previous;
        self.icon_input_binding(document, &previous.input)?;
        self.charge(3)?;
        if input.data.examination.subject != previous.declaration().subject
            || input.data.examination.resource != *previous.resource()
        {
            return Err(self.stop(HtmlSemanticFailureKind::RelationshipSelectorRequestMismatch));
        }
        if input.data.examination.outcome == HtmlIconCloserExamination::Unavailable {
            return Ok(HtmlSemanticValue::Known(
                HtmlDocumentIconReadback::Selection(HtmlSemanticValue::Missing(Self::rel_missing(
                    previous.declaration().subject(),
                    HtmlRelMissingFact::IconCloserExamination,
                ))),
            ));
        }
        let mut rejections = Vec::new();
        self.reserve_nested_output(&mut rejections, previous.rejections.len())?;
        for rejection in &previous.rejections {
            self.charge(1)?;
            rejections.push(rejection.clone());
        }
        self.reserve_nested_output(&mut rejections, 1)?;
        self.charge(1)?;
        rejections.push(HtmlIconRejectionRecord {
            index: previous.index,
            observation: input.data.examination.clone(),
        });
        Ok(HtmlSemanticValue::Known(
            HtmlDocumentIconReadback::Selection(self.choose_icon(
                document,
                &previous.input,
                rejections,
            )?),
        ))
    }
    fn selector_raw(
        &mut self,
        element: &'a HtmlCapturedElement,
        name: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        let raw = self.attribute(element, name)?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        Ok(raw)
    }
    fn icon_field(
        &mut self,
        element: &HtmlCapturedElement,
        raw: Option<&Attribute>,
    ) -> Result<Option<HtmlIconAttribute>, HtmlSemanticFailure> {
        let Some(raw) = raw else { return Ok(None) };
        for (ordinal, field) in element.attributes().iter().enumerate() {
            self.charge(1)?;
            if std::ptr::eq(field, raw) {
                return Ok(Some(HtmlIconAttribute {
                    ordinal,
                    raw: raw.value().clone(),
                }));
            }
        }
        unreachable!("canonical attribute lookup returns actual field")
    }
    fn selector_membership(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        raw: Option<&DocumentString>,
        requested: HtmlRelKeyword,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let Some(raw) = raw else {
            return Ok(HtmlSemanticValue::Known(false));
        };
        self.charge(SpaceTokenCursor::BEGIN_WORK)?;
        let mut cursor = SpaceTokenCursor::new(raw.code_units());
        let mut unknown = false;
        loop {
            self.charge(SpaceTokenCursor::STEP_WORK)?;
            match cursor.step() {
                SpaceTokenStep::Continue => {}
                SpaceTokenStep::Done => break,
                SpaceTokenStep::Token { start, end } => {
                    let (mut keyword, spelling) = rel::classify(
                        &raw.code_units()[start..end],
                        &mut RelControl { session: self },
                    )?;
                    let mut unresolved = spelling == HtmlRelSpelling::Unknown;
                    if unresolved && let Some(facts) = self.context.rel_facts.as_ref() {
                        for record in &facts.input.occurrences {
                            self.charge(3)?;
                            if record.subject != *subject
                                || record.start != start
                                || record.end != end
                            {
                                continue;
                            }
                            match &record.state {
                                RegistryOccurrence::Unavailable => {}
                                RegistryOccurrence::ProvenAbsent => unresolved = false,
                                RegistryOccurrence::Definition { key, .. } => {
                                    for definition in &facts.input.definitions {
                                        self.charge(1)?;
                                        if definition.key == *key {
                                            keyword = definition.standard;
                                            unresolved = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                    self.charge(2)?;
                    if keyword == Some(requested) {
                        return Ok(HtmlSemanticValue::Known(true));
                    }
                    unknown |= unresolved;
                }
            }
        }
        Ok(if unknown {
            HtmlSemanticValue::Missing(Self::rel_missing(subject, HtmlRelMissingFact::Registry))
        } else {
            HtmlSemanticValue::Known(false)
        })
    }
    fn selector_exact_type(
        &mut self,
        raw: Option<&Attribute>,
        expected: &[u8],
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(raw) = raw else { return Ok(false) };
        if raw.value().len() != expected.len() {
            return Ok(false);
        }
        for (a, b) in raw.value().code_units().iter().zip(expected) {
            self.charge(1)?;
            if *a != u16::from(*b) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn selector_cleanup<T>(
        &mut self,
        frames: Vec<HtmlCapturedChildIds<'a>>,
        outcome: Result<T, HtmlSemanticFailure>,
    ) -> Result<T, HtmlSemanticFailure> {
        self.release_scratch(frames.capacity());
        drop(frames);
        if outcome.is_err()
            && let Some(first) = self.failure.as_mut()
        {
            first.usage.scratch_records = self.usage.scratch_records;
            return Err(first.clone());
        }
        outcome
    }
    pub(in super::super) fn document_relationship_candidates(
        &mut self,
        document: &HtmlCaptureSubjectId,
        keyword: HtmlRelKeyword,
        first: bool,
    ) -> Result<HtmlSemanticValue<Vec<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        if !matches!(
            self.node(document)?.kind(),
            HtmlCapturedNodeKind::Document(_)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut frames = Vec::new();
        let mut found = Vec::new();
        let outcome = (|| {
            let mut current = Some(document.clone());
            while let Some(subject) = current {
                let node = self.node(&subject)?;
                if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                    let raw = self.selector_raw(element, "rel")?;
                    let type_hint = if keyword == HtmlRelKeyword::Alternate {
                        self.selector_raw(element, "type")?
                    } else {
                        None
                    };
                    if self.known(element)? == Some(KnownHtmlElement::Link) {
                        let feed_type = keyword != HtmlRelKeyword::Alternate
                            || self.selector_exact_type(type_hint, b"application/rss+xml")?
                            || self.selector_exact_type(type_hint, b"application/atom+xml")?;
                        if feed_type {
                            match self.selector_membership(
                                &subject,
                                raw.map(Attribute::value),
                                keyword,
                            )? {
                                HtmlSemanticValue::Known(true) => {
                                    self.reserve_nested_output(&mut found, 1)?;
                                    self.charge(1)?;
                                    found.push(subject.clone());
                                    if first {
                                        return Ok(HtmlSemanticValue::Known(found));
                                    }
                                }
                                HtmlSemanticValue::Missing(m) => {
                                    return Ok(HtmlSemanticValue::Missing(m));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                current = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(next) => next,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                };
            }
            Ok(HtmlSemanticValue::Known(found))
        })();
        self.selector_cleanup(frames, outcome)
    }
    pub(in super::super) fn default_feed(
        &mut self,
        document: &HtmlCaptureSubjectId,
        input: Option<&HtmlDefaultFeedInput>,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        if let Some(input) = input {
            self.charge(5)?;
            if input.capture != *self.capture.identity()
                || input.context != *self.context.identity()
                || input.host != *self.context.host_revision()
                || input.document != *document
            {
                return Err(self.stop(HtmlSemanticFailureKind::RelationshipSelectorRequestMismatch));
            }
        }
        if !matches!(
            self.node(document)?.kind(),
            HtmlCapturedNodeKind::Document(_)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        match input.map(|i| i.applicability) {
            Some(HtmlDefaultFeedApplicability::Inapplicable) => Ok(HtmlSemanticValue::Inapplicable),
            None | Some(HtmlDefaultFeedApplicability::Unavailable) => {
                Ok(HtmlSemanticValue::Missing(Self::rel_missing(
                    document,
                    HtmlRelMissingFact::DefaultFeedApplicability,
                )))
            }
            Some(HtmlDefaultFeedApplicability::Applicable) => Ok(self
                .document_relationship_candidates(document, HtmlRelKeyword::Alternate, true)?
                .map(|v| v.into_iter().next())),
        }
    }
    pub(in super::super) fn document_icon_collection(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlDocumentIconCollection>, HtmlSemanticFailure> {
        if !matches!(
            self.node(document)?.kind(),
            HtmlCapturedNodeKind::Document(_)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut frames = Vec::new();
        let mut candidates = Vec::new();
        let outcome = (|| {
            let mut current = Some(document.clone());
            while let Some(subject) = current {
                let node = self.node(&subject)?;
                if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                    let rel = self.selector_raw(element, "rel")?;
                    let type_hint = self.selector_raw(element, "type")?;
                    let media = self.selector_raw(element, "media")?;
                    let sizes = self.selector_raw(element, "sizes")?;
                    let href = self.selector_raw(element, "href")?;
                    if self.known(element)? == Some(KnownHtmlElement::Link) {
                        match self.selector_membership(
                            &subject,
                            rel.map(Attribute::value),
                            HtmlRelKeyword::Icon,
                        )? {
                            HtmlSemanticValue::Known(true) => {
                                let media_rules = match self.media_attribute_rules(&subject)? {
                                    HtmlSemanticValue::Known(value) => value,
                                    HtmlSemanticValue::Missing(m) => {
                                        return Ok(HtmlSemanticValue::Missing(m));
                                    }
                                    _ => unreachable!("actual link media role"),
                                };
                                let mut dimensions: Vec<HtmlIconSizeOccurrence> = Vec::new();
                                let mut first_invalid = None;
                                let mut first_duplicate = None;
                                if let Some(sizes) = sizes {
                                    self.charge(SpaceTokenCursor::BEGIN_WORK)?;
                                    let mut tokens =
                                        SpaceTokenCursor::new(sizes.value().code_units());
                                    loop {
                                        self.charge(SpaceTokenCursor::STEP_WORK)?;
                                        match tokens.step() {
                                            SpaceTokenStep::Done => break,
                                            SpaceTokenStep::Continue => {}
                                            SpaceTokenStep::Token { start, end } => {
                                                self.charge(1)?;
                                                let mut grammar = IconSizeCursor::new(
                                                    &sizes.value().code_units()[start..end],
                                                );
                                                while !grammar.is_complete() {
                                                    self.charge(IconSizeCursor::STEP_WORK)?;
                                                    grammar.step();
                                                }
                                                let declaration = match grammar
                                                    .result()
                                                    .expect("complete canonical sizes token")
                                                {
                                                    Ok(IconSizeShape::Scalable) => {
                                                        HtmlIconSizeDeclaration::Scalable
                                                    }
                                                    Ok(IconSizeShape::Bitmap { width, height }) => {
                                                        HtmlIconSizeDeclaration::Bitmap {
                                                            width: start + width.start
                                                                ..start + width.end,
                                                            height: start + height.start
                                                                ..start + height.end,
                                                        }
                                                    }
                                                    Err(error) => {
                                                        HtmlIconSizeDeclaration::Invalid(*error)
                                                    }
                                                };
                                                let mut duplicate_of = None;
                                                for (index, previous) in
                                                    dimensions.iter().enumerate()
                                                {
                                                    self.charge(1)?;
                                                    if previous.range.len() != end - start {
                                                        continue;
                                                    }
                                                    let mut equal = true;
                                                    for (a, b) in sizes.value().code_units()
                                                        [previous.range.clone()]
                                                    .iter()
                                                    .zip(&sizes.value().code_units()[start..end])
                                                    {
                                                        self.charge(3)?;
                                                        if rel::fold(*a) != rel::fold(*b) {
                                                            equal = false;
                                                            break;
                                                        }
                                                    }
                                                    if equal {
                                                        duplicate_of = Some(index);
                                                        break;
                                                    }
                                                }
                                                self.charge(2)?;
                                                if first_invalid.is_none()
                                                    && let HtmlIconSizeDeclaration::Invalid(reason) =
                                                        declaration
                                                {
                                                    first_invalid =
                                                        Some((dimensions.len(), reason));
                                                }
                                                if first_duplicate.is_none()
                                                    && let Some(index) = duplicate_of
                                                {
                                                    first_duplicate =
                                                        Some((index, dimensions.len()));
                                                }
                                                self.reserve_nested_output(&mut dimensions, 1)?;
                                                self.charge(1)?;
                                                dimensions.push(HtmlIconSizeOccurrence {
                                                    range: start..end,
                                                    declaration,
                                                    duplicate_of,
                                                });
                                            }
                                        }
                                    }
                                }
                                let sizes_author =
                                    match (sizes.is_some(), first_invalid, first_duplicate) {
                                        (_, Some((token_index, reason)), _) => {
                                            HtmlIconSizesAuthor::Invalid {
                                                token_index,
                                                reason,
                                            }
                                        }
                                        (_, _, Some((first_index, duplicate_index))) => {
                                            HtmlIconSizesAuthor::Duplicate {
                                                first_index,
                                                duplicate_index,
                                            }
                                        }
                                        (true, _, _) => HtmlIconSizesAuthor::Conforming,
                                        (false, _, _) => HtmlIconSizesAuthor::NotSpecified,
                                    };
                                let candidate = HtmlIconDeclaration {
                                    subject: subject.clone(),
                                    rel: self
                                        .icon_field(element, rel)?
                                        .expect("icon membership proves rel"),
                                    type_hint: self.icon_field(element, type_hint)?,
                                    media: self.icon_field(element, media)?,
                                    sizes: self.icon_field(element, sizes)?,
                                    href: self.icon_field(element, href)?,
                                    media_rules,
                                    dimensions,
                                    sizes_author,
                                };
                                self.reserve_nested_output(&mut candidates, 1)?;
                                self.charge(1)?;
                                candidates.push(candidate);
                            }
                            HtmlSemanticValue::Missing(m) => {
                                return Ok(HtmlSemanticValue::Missing(m));
                            }
                            _ => {}
                        }
                    }
                }
                current = match self.ordinary_tree_next(node, &mut frames)? {
                    Ok(next) => next,
                    Err(m) => return Ok(HtmlSemanticValue::Missing(m)),
                };
            }
            Ok(HtmlSemanticValue::Known(HtmlDocumentIconCollection {
                capture: self.capture.identity().clone(),
                context: self.context.identity().clone(),
                host: self.context.host_revision().clone(),
                document: document.clone(),
                media_revision: self.context.media_revision().cloned(),
                candidates,
            }))
        })();
        self.selector_cleanup(frames, outcome)
    }
}
