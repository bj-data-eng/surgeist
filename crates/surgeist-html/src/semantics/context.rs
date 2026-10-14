//! Admission and immutable identity of capture-qualified semantic inputs.

use super::*;

/// One producer-established custom-control classification at the context revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCustomControlClassification {
    pub(super) subject: HtmlCaptureSubjectId,
    pub(super) form_associated: bool,
}
impl HtmlCustomControlClassification {
    /// `false` is established nonparticipation, rather than missing evidence.
    pub fn new(subject: HtmlCaptureSubjectId, form_associated: bool) -> Self {
        Self {
            subject,
            form_associated,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn form_associated(&self) -> bool {
        self.form_associated
    }
}

/// Identity of immutable supplied classifications; equal records do not imply identity.
#[derive(Clone, Debug)]
pub struct HtmlSemanticContextId(Arc<()>);
impl PartialEq for HtmlSemanticContextId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlSemanticContextId {}

/// Abstract producer identity, independent of Style, Layout or host types.
#[derive(Clone, Debug)]
pub struct HtmlParticipationProducerId(Arc<()>);
impl HtmlParticipationProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlParticipationProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlParticipationProducerId {}

/// Exact computed-classification and CSS/SVG/equivalent box-production inputs.
/// Configuration and revision tokens are supplied by the coordinating host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlListParticipationRevision {
    style_producer: HtmlParticipationProducerId,
    style_configuration: u64,
    style_revision: u64,
    box_producer: HtmlParticipationProducerId,
    box_configuration: u64,
    box_revision: u64,
}
impl HtmlListParticipationRevision {
    pub fn new(
        style_producer: HtmlParticipationProducerId,
        style_configuration: u64,
        style_revision: u64,
        box_producer: HtmlParticipationProducerId,
        box_configuration: u64,
        box_revision: u64,
    ) -> Self {
        Self {
            style_producer,
            style_configuration,
            style_revision,
            box_producer,
            box_configuration,
            box_revision,
        }
    }
    pub fn style_producer(&self) -> &HtmlParticipationProducerId {
        &self.style_producer
    }
    pub fn style_configuration(&self) -> u64 {
        self.style_configuration
    }
    pub fn style_revision(&self) -> u64 {
        self.style_revision
    }
    pub fn box_producer(&self) -> &HtmlParticipationProducerId {
        &self.box_producer
    }
    pub fn box_configuration(&self) -> u64 {
        self.box_configuration
    }
    pub fn box_revision(&self) -> u64 {
        self.box_revision
    }
}

/// Producer-established facts. `None` is unavailable; `Some(false)` is evidence.
/// Being rendered means associated CSS, SVG or equivalent boxes, regardless of
/// viewport, hidden content attributes, positive geometry or marker geometry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlListParticipationFact {
    pub(super) subject: HtmlCaptureSubjectId,
    pub(super) computed_list_item: Option<bool>,
    pub(super) being_rendered: Option<bool>,
    pub(super) produces_css_box: Option<bool>,
}
impl HtmlListParticipationFact {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        computed_list_item: Option<bool>,
        being_rendered: Option<bool>,
        produces_css_box: Option<bool>,
    ) -> Self {
        Self {
            subject,
            computed_list_item,
            being_rendered,
            produces_css_box,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn computed_list_item(&self) -> Option<bool> {
        self.computed_list_item
    }
    pub fn being_rendered(&self) -> Option<bool> {
        self.being_rendered
    }
    pub fn produces_css_box(&self) -> Option<bool> {
        self.produces_css_box
    }
}

/// Checked immutable participation supplement for one exact ordinary topology.
#[derive(Debug)]
pub struct HtmlListParticipation {
    capture: HtmlCaptureId,
    revision: HtmlListParticipationRevision,
    pub(super) facts: Vec<HtmlListParticipationFact>,
}
impl HtmlListParticipation {
    /// Retains the supplied vector, validating membership and uniqueness under
    /// its explicit capacity cap. No tree projection or inferred defaults.
    pub fn new(
        capture: &HtmlCapture,
        revision: HtmlListParticipationRevision,
        facts: Vec<HtmlListParticipationFact>,
        max_facts: usize,
    ) -> Result<Self, HtmlSemanticContextError> {
        if facts.capacity() > max_facts {
            return Err(HtmlSemanticContextError::ParticipationLimit {
                limit: max_facts,
                retained_capacity: facts.capacity(),
            });
        }
        for (i, fact) in facts.iter().enumerate() {
            capture
                .element_id(&fact.subject)
                .map_err(HtmlSemanticContextError::Capture)?;
            if facts[..i].iter().any(|other| other.subject == fact.subject) {
                return Err(HtmlSemanticContextError::DuplicateSubject(
                    fact.subject.clone(),
                ));
            }
            if fact.produces_css_box == Some(true) && fact.being_rendered == Some(false) {
                return Err(HtmlSemanticContextError::ContradictoryParticipation(
                    fact.subject.clone(),
                ));
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            revision,
            facts,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn revision(&self) -> &HtmlListParticipationRevision {
        &self.revision
    }
    pub fn facts(&self) -> &[HtmlListParticipationFact] {
        &self.facts
    }
}

/// Moved inputs for the concrete implemented semantic consumers.
/// Attaching an optional supplement does not certify coherence; the context
/// checks capture and host qualification before minting its immutable identity.
#[derive(Debug)]
pub struct HtmlSemanticContextInput {
    media_revision: Option<HtmlMediaContextRevision>,
    media: Option<HtmlMediaFacts>,
    rel_revisions: HtmlRelExpectedRevisions,
    rel_facts: Option<HtmlRelFacts>,
    image_dimension_revision: Option<HtmlImageDimensionRevision>,
    image_dimensions: Option<HtmlImageDimensionFacts>,
    natural_dimension_revision: Option<HtmlNaturalDimensionRevision>,
    natural_dimensions: Option<HtmlNaturalDimensions>,
    classifications: Vec<HtmlCustomControlClassification>,
    participation_revision: Option<HtmlListParticipationRevision>,
    participation: Option<HtmlListParticipation>,
    language_direction: Option<HtmlLanguageDirectionFacts>,
    inert_revision: Option<HtmlFlatTreeRevision>,
    inert: Option<HtmlInertFacts>,
    controls: Option<HtmlControlFacts>,
    element_references: Option<HtmlElementReferenceFacts>,
    output_defaults: Option<HtmlOutputDefaultFacts>,
    microdata_revision: Option<HtmlMicrodataRevision>,
    global_inheritance_revision: Option<HtmlGlobalInheritanceRevision>,
    microdata: Option<HtmlMicrodataFacts>,
    global_inheritance: Option<HtmlGlobalInheritanceFacts>,
    base_revision: Option<HtmlBaseFactsRevision>,
    base_facts: Option<HtmlCapturedBaseFacts>,
}
impl HtmlSemanticContextInput {
    pub fn new(classifications: Vec<HtmlCustomControlClassification>) -> Self {
        Self {
            media_revision: None,
            media: None,
            rel_revisions: HtmlRelExpectedRevisions::new(),
            rel_facts: None,
            natural_dimension_revision: None,
            image_dimension_revision: None,
            image_dimensions: None,
            natural_dimensions: None,
            classifications,
            participation_revision: None,
            participation: None,
            language_direction: None,
            inert_revision: None,
            inert: None,
            controls: None,
            element_references: None,
            output_defaults: None,
            microdata_revision: None,
            microdata: None,
            global_inheritance_revision: None,
            global_inheritance: None,
            base_revision: None,
            base_facts: None,
        }
    }
    pub fn with_microdata(
        mut self,
        expected: HtmlMicrodataRevision,
        facts: Option<HtmlMicrodataFacts>,
    ) -> Self {
        self.microdata_revision = Some(expected);
        self.microdata = facts;
        self
    }
    pub fn with_base_facts(
        mut self,
        expected: HtmlBaseFactsRevision,
        facts: Option<HtmlCapturedBaseFacts>,
    ) -> Self {
        self.base_revision = Some(expected);
        self.base_facts = facts;
        self
    }
    pub fn with_list_participation(
        mut self,
        expected: HtmlListParticipationRevision,
        participation: Option<HtmlListParticipation>,
    ) -> Self {
        self.participation_revision = Some(expected);
        self.participation = participation;
        self
    }
    pub fn with_media(
        mut self,
        expected: HtmlMediaContextRevision,
        facts: Option<HtmlMediaFacts>,
    ) -> Self {
        self.media_revision = Some(expected);
        self.media = facts;
        self
    }
    pub fn with_link_relationships(
        mut self,
        expected: HtmlRelExpectedRevisions,
        facts: Option<HtmlRelFacts>,
    ) -> Self {
        self.rel_revisions = expected;
        self.rel_facts = facts;
        self
    }
    pub fn with_natural_dimensions(
        mut self,
        expected: HtmlNaturalDimensionRevision,
        facts: Option<HtmlNaturalDimensions>,
    ) -> Self {
        self.natural_dimension_revision = Some(expected);
        self.natural_dimensions = facts;
        self
    }
    pub fn with_inert(
        mut self,
        expected: HtmlFlatTreeRevision,
        facts: Option<HtmlInertFacts>,
    ) -> Self {
        self.inert_revision = Some(expected);
        self.inert = facts;
        self
    }
    pub fn with_image_dimensions(
        mut self,
        expected: HtmlImageDimensionRevision,
        facts: Option<HtmlImageDimensionFacts>,
    ) -> Self {
        self.image_dimension_revision = Some(expected);
        self.image_dimensions = facts;
        self
    }
    pub fn with_global_inheritance(
        mut self,
        expected: HtmlGlobalInheritanceRevision,
        facts: Option<HtmlGlobalInheritanceFacts>,
    ) -> Self {
        self.global_inheritance_revision = Some(expected);
        self.global_inheritance = facts;
        self
    }
    pub fn with_controls(mut self, facts: HtmlControlFacts) -> Self {
        self.controls = Some(facts);
        self
    }
    pub fn with_element_references(mut self, facts: HtmlElementReferenceFacts) -> Self {
        self.element_references = Some(facts);
        self
    }
    pub fn with_output_defaults(mut self, facts: HtmlOutputDefaultFacts) -> Self {
        self.output_defaults = Some(facts);
        self
    }
    pub fn with_language_direction(mut self, facts: HtmlLanguageDirectionFacts) -> Self {
        self.language_direction = Some(facts);
        self
    }
}

/// Checked immutable inputs, shared by queries over one exact capture.
#[derive(Debug)]
pub struct HtmlSemanticContext {
    pub(super) media_revision: Option<HtmlMediaContextRevision>,
    pub(super) media: Option<HtmlMediaFacts>,
    pub(super) rel_revisions: HtmlRelExpectedRevisions,
    pub(super) rel_facts: Option<HtmlRelFacts>,
    pub(super) image_dimension_revision: Option<HtmlImageDimensionRevision>,
    pub(super) image_dimensions: Option<HtmlImageDimensionFacts>,
    pub(super) natural_dimension_revision: Option<HtmlNaturalDimensionRevision>,
    pub(super) natural_dimensions: Option<HtmlNaturalDimensions>,
    pub(super) identity: HtmlSemanticContextId,
    capture: HtmlCaptureId,
    pub(super) host_revision: HtmlHostFactsRevision,
    pub(super) classifications: Vec<HtmlCustomControlClassification>,
    pub(super) participation_revision: Option<HtmlListParticipationRevision>,
    pub(super) participation: Option<HtmlListParticipation>,
    language_direction: Option<HtmlLanguageDirectionFacts>,
    pub(super) inert_revision: Option<HtmlFlatTreeRevision>,
    inert: Option<HtmlInertFacts>,
    controls: Option<HtmlControlFacts>,
    element_references: Option<HtmlElementReferenceFacts>,
    output_defaults: Option<HtmlOutputDefaultFacts>,
    pub(super) microdata_revision: Option<HtmlMicrodataRevision>,
    pub(super) global_inheritance_revision: Option<HtmlGlobalInheritanceRevision>,
    pub(super) microdata: Option<HtmlMicrodataFacts>,
    global_inheritance: Option<HtmlGlobalInheritanceFacts>,
    pub(super) base_revision: Option<HtmlBaseFactsRevision>,
    pub(super) base_facts: Option<HtmlCapturedBaseFacts>,
    pub(super) empty_language: DocumentString,
}
/// Rejected producer context; no query result is created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticContextError {
    MediaCaptureMismatch,
    MediaHostRevisionMismatch,
    MediaRevisionMismatch,
    MicrodataFactsMismatch,
    BaseCaptureMismatch,
    BaseHostRevisionMismatch,
    BaseRevisionMismatch,
    LinkRelationshipsCaptureMismatch,
    LinkRelationshipsHostRevisionMismatch,
    LinkRelationshipsRevisionMismatch,
    ImageDimensionsCaptureMismatch,
    ImageDimensionsHostRevisionMismatch,
    ImageDimensionsRevisionMismatch,
    ImageDimensionsFacts(HtmlImageDimensionFactsError),
    NaturalDimensionsCaptureMismatch,
    NaturalDimensionsHostRevisionMismatch,
    NaturalDimensionsRevisionMismatch,
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    ClassificationLimit {
        limit: usize,
        retained_capacity: usize,
    },
    DuplicateSubject(HtmlCaptureSubjectId),
    ParticipationLimit {
        limit: usize,
        retained_capacity: usize,
    },
    ParticipationCaptureMismatch,
    ParticipationRevisionMismatch,
    ContradictoryParticipation(HtmlCaptureSubjectId),
    LanguageDirectionCaptureMismatch,
    LanguageDirectionHostRevisionMismatch,
    InertCaptureMismatch,
    InertHostRevisionMismatch,
    InertRevisionMismatch,
    ControlsCaptureMismatch,
    ControlsHostRevisionMismatch,
    ElementReferencesCaptureMismatch,
    ElementReferencesHostRevisionMismatch,
    OutputDefaultsCaptureMismatch,
    OutputDefaultsHostRevisionMismatch,
    GlobalInheritanceCaptureMismatch,
    GlobalInheritanceHostRevisionMismatch,
    GlobalInheritanceRevisionMismatch,
    ContradictoryCustomControlAssociation(HtmlCaptureSubjectId),
    ControlsFacts(HtmlControlFactsError),
}
impl fmt::Display for HtmlSemanticContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlSemanticContextError {}
impl HtmlSemanticContext {
    pub fn media_revision(&self) -> Option<&HtmlMediaContextRevision> {
        self.media_revision.as_ref()
    }
    pub fn media(&self) -> Option<&HtmlMediaFacts> {
        self.media.as_ref()
    }
    /// Moves already supplied records without projecting or copying the tree.
    /// `max_classifications` bounds classification vector capacity and duplicate
    /// validation. Each optional supplement carries its own checked, finite
    /// producer construction. Every input must match the capture and host key.
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        input: HtmlSemanticContextInput,
        max_classifications: usize,
    ) -> Result<Self, HtmlSemanticContextError> {
        if &host_revision != capture.host_facts_revision() {
            return Err(HtmlSemanticContextError::HostRevisionMismatch);
        }
        let HtmlSemanticContextInput {
            media_revision,
            media,
            rel_revisions,
            rel_facts,
            image_dimension_revision,
            mut image_dimensions,
            natural_dimension_revision,
            natural_dimensions,
            classifications,
            participation_revision,
            participation,
            language_direction,
            inert_revision,
            inert,
            controls,
            element_references,
            output_defaults,
            microdata_revision,
            microdata,
            global_inheritance_revision,
            global_inheritance,
            base_revision,
            base_facts,
        } = input;
        if let Some(facts) = &media {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::MediaCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::MediaHostRevisionMismatch);
            }
            if media_revision.as_ref() != Some(facts.revision()) {
                return Err(HtmlSemanticContextError::MediaRevisionMismatch);
            }
        }
        if microdata
            .as_ref()
            .is_some_and(|facts| !facts.qualifies(capture, microdata_revision.as_ref()))
        {
            return Err(HtmlSemanticContextError::MicrodataFactsMismatch);
        }
        if let Some(facts) = &base_facts {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::BaseCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::BaseHostRevisionMismatch);
            }
            if base_revision.as_ref() != Some(facts.revision()) {
                return Err(HtmlSemanticContextError::BaseRevisionMismatch);
            }
        }
        if let Some(facts) = &rel_facts {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::LinkRelationshipsCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::LinkRelationshipsHostRevisionMismatch);
            }
            if facts.revisions() != &rel_revisions {
                return Err(HtmlSemanticContextError::LinkRelationshipsRevisionMismatch);
            }
        }
        if let Some(facts) = &mut image_dimensions {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::ImageDimensionsCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::ImageDimensionsHostRevisionMismatch);
            }
            if image_dimension_revision.as_ref() != Some(facts.revision()) {
                return Err(HtmlSemanticContextError::ImageDimensionsRevisionMismatch);
            }
            if facts.revision().participation().is_some()
                && facts.revision().participation() != participation_revision.as_ref()
            {
                return Err(HtmlSemanticContextError::ImageDimensionsRevisionMismatch);
            }
            facts
                .validate_joins(participation.as_ref(), natural_dimensions.as_ref())
                .map_err(HtmlSemanticContextError::ImageDimensionsFacts)?;
        }
        if let Some(facts) = &natural_dimensions {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::NaturalDimensionsCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::NaturalDimensionsHostRevisionMismatch);
            }
            if natural_dimension_revision.as_ref() != Some(facts.revision()) {
                return Err(HtmlSemanticContextError::NaturalDimensionsRevisionMismatch);
            }
        }
        if let Some(facts) = &global_inheritance {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::GlobalInheritanceCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::GlobalInheritanceHostRevisionMismatch);
            }
            if global_inheritance_revision.as_ref() != Some(facts.revision()) {
                return Err(HtmlSemanticContextError::GlobalInheritanceRevisionMismatch);
            }
        }
        if let Some(facts) = &inert {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::InertCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::InertHostRevisionMismatch);
            }
            if inert_revision.as_ref() != Some(facts.flat_revision()) {
                return Err(HtmlSemanticContextError::InertRevisionMismatch);
            }
        }
        if let Some(facts) = &output_defaults {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::OutputDefaultsCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::OutputDefaultsHostRevisionMismatch);
            }
        }
        if let Some(facts) = &element_references {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::ElementReferencesCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::ElementReferencesHostRevisionMismatch);
            }
        }
        if let Some(facts) = &controls {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::ControlsCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::ControlsHostRevisionMismatch);
            }
        }
        if let Some(facts) = &language_direction {
            if facts.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::LanguageDirectionCaptureMismatch);
            }
            if facts.host_revision() != &host_revision {
                return Err(HtmlSemanticContextError::LanguageDirectionHostRevisionMismatch);
            }
        }
        if let Some(input) = &participation {
            if input.capture() != capture.identity() {
                return Err(HtmlSemanticContextError::ParticipationCaptureMismatch);
            }
            if participation_revision.as_ref() != Some(input.revision()) {
                return Err(HtmlSemanticContextError::ParticipationRevisionMismatch);
            }
        }
        if classifications.capacity() > max_classifications {
            return Err(HtmlSemanticContextError::ClassificationLimit {
                limit: max_classifications,
                retained_capacity: classifications.capacity(),
            });
        }
        for (i, fact) in classifications.iter().enumerate() {
            capture
                .element_id(&fact.subject)
                .map_err(HtmlSemanticContextError::Capture)?;
            if classifications[..i]
                .iter()
                .any(|other| other.subject == fact.subject)
            {
                return Err(HtmlSemanticContextError::DuplicateSubject(
                    fact.subject.clone(),
                ));
            }
        }
        let controls = controls
            .map(|facts| facts.validate_classifications(capture, &classifications))
            .transpose()
            .map_err(|error| match error {
                HtmlControlFactsError::ContradictoryCustomControlAssociation(subject) => {
                    HtmlSemanticContextError::ContradictoryCustomControlAssociation(subject)
                }
                other => HtmlSemanticContextError::ControlsFacts(other),
            })?;
        Ok(Self {
            rel_revisions,
            rel_facts,
            identity: HtmlSemanticContextId(Arc::new(())),
            media_revision,
            media,
            image_dimension_revision,
            image_dimensions,
            natural_dimension_revision,
            natural_dimensions,
            capture: capture.identity().clone(),
            host_revision,
            classifications,
            participation_revision,
            participation,
            language_direction,
            inert_revision,
            inert,
            controls,
            element_references,
            output_defaults,
            microdata_revision,
            microdata,
            global_inheritance_revision,
            global_inheritance,
            base_revision,
            base_facts,
            empty_language: DocumentString::default(),
        })
    }
    pub fn identity(&self) -> &HtmlSemanticContextId {
        &self.identity
    }
    pub fn natural_dimension_revision(&self) -> Option<&HtmlNaturalDimensionRevision> {
        self.natural_dimension_revision.as_ref()
    }
    pub fn image_dimension_revision(&self) -> Option<&HtmlImageDimensionRevision> {
        self.image_dimension_revision.as_ref()
    }
    pub fn image_dimensions(&self) -> Option<&HtmlImageDimensionFacts> {
        self.image_dimensions.as_ref()
    }
    pub fn natural_dimensions(&self) -> Option<&HtmlNaturalDimensions> {
        self.natural_dimensions.as_ref()
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn classifications(&self) -> &[HtmlCustomControlClassification] {
        &self.classifications
    }
    pub fn participation_revision(&self) -> Option<&HtmlListParticipationRevision> {
        self.participation_revision.as_ref()
    }
    pub fn participation(&self) -> Option<&HtmlListParticipation> {
        self.participation.as_ref()
    }
    pub fn language_direction(&self) -> Option<&HtmlLanguageDirectionFacts> {
        self.language_direction.as_ref()
    }
    pub fn inert_revision(&self) -> Option<&HtmlFlatTreeRevision> {
        self.inert_revision.as_ref()
    }
    pub fn inert(&self) -> Option<&HtmlInertFacts> {
        self.inert.as_ref()
    }
    pub fn output_defaults(&self) -> Option<&HtmlOutputDefaultFacts> {
        self.output_defaults.as_ref()
    }
    pub fn element_references(&self) -> Option<&HtmlElementReferenceFacts> {
        self.element_references.as_ref()
    }
    pub fn controls(&self) -> Option<&HtmlControlFacts> {
        self.controls.as_ref()
    }
    pub fn microdata_revision(&self) -> Option<&HtmlMicrodataRevision> {
        self.microdata_revision.as_ref()
    }
    pub fn base_revision(&self) -> Option<&HtmlBaseFactsRevision> {
        self.base_revision.as_ref()
    }
    pub fn base_facts(&self) -> Option<&HtmlCapturedBaseFacts> {
        self.base_facts.as_ref()
    }
    pub fn global_inheritance_revision(&self) -> Option<&HtmlGlobalInheritanceRevision> {
        self.global_inheritance_revision.as_ref()
    }
    pub fn global_inheritance(&self) -> Option<&HtmlGlobalInheritanceFacts> {
        self.global_inheritance.as_ref()
    }
    pub(super) fn empty_language(&self) -> &DocumentString {
        &self.empty_language
    }
}
