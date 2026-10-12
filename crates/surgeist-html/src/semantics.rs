//! Capture-qualified intrinsic semantic queries, HTML 877d6146487f.
//!
//! Queries cover hidden state, disabledness, list ownership and ordinals, the
//! table model and immutable table collections, language and directionality,
//! meter and progress state, inertness,
//! form owners, form/fieldset listed collections and form length, label targets, label form owners and control labels, current control flags, global inheritance, select
//! readbacks, placeholder-label-option classification and static select missingness, datalist association and option/suggestion membership, map area membership, details/summary relationships, option form/text/label/value and select value getters, autocomplete
//! defaults, textarea/output text getters, captured time-element interpretation, image/link attribute author contexts and input/button form setting selection
//! including the source no-validate state, default buttons and static implicit
//! submission conditions, static constraint-validation candidacy, the static tabIndex getter,
//! and own/used capitalization, used autocorrection and computed writing suggestions.
//! Computed list participation, flat-tree/modal facts, document design mode and
//! default spellcheck policy are concrete capture-qualified inputs. Style, layout, focus and runtime state transitions
//! remain with their producers; queries borrow one immutable capture.

mod table;
pub use table::*;
mod language;
pub use language::*;
mod range;
pub use range::*;
mod inert;
pub use inert::*;
mod control;
pub use control::*;
mod control_text;
pub use control_text::*;
mod attribute_context;
pub use attribute_context::*;
mod time_element;
pub use time_element::*;
mod inheritance;
pub use inheritance::*;
mod text_assistance;
pub use text_assistance::*;
mod select;
pub use select::*;
mod datalist;
mod image_map;
pub use image_map::*;
mod tabindex;
pub use tabindex::*;
mod details;
pub use details::*;
mod option_text;
pub use option_text::*;
mod autocomplete;
pub use autocomplete::*;
mod form_settings;
pub use form_settings::*;
mod form_default;
pub use form_default::*;

use crate::numeric::{
    IntegerRecoveryError, IntegerUnitStep, IntegerUnitStepError, IntegerUnitStepResult,
    try_parse_integer,
};
use crate::{
    Attribute, DocumentString, HtmlBooleanAttribute, HtmlCapture, HtmlCaptureError, HtmlCaptureId,
    HtmlCaptureSubjectId, HtmlCapturedChildIds, HtmlCapturedChildren, HtmlCapturedElement,
    HtmlCapturedNode, HtmlCapturedNodeKind, HtmlCapturedParent, HtmlCustomControlAttributeContext,
    HtmlHiddenState, HtmlHostFactsRevision, HtmlInteger, HtmlValueAdmissionError,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement,
};
use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// One producer-established custom-control classification at the context revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCustomControlClassification {
    subject: HtmlCaptureSubjectId,
    form_associated: bool,
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
    subject: HtmlCaptureSubjectId,
    computed_list_item: Option<bool>,
    being_rendered: Option<bool>,
    produces_css_box: Option<bool>,
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
    facts: Vec<HtmlListParticipationFact>,
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
    classifications: Vec<HtmlCustomControlClassification>,
    participation_revision: Option<HtmlListParticipationRevision>,
    participation: Option<HtmlListParticipation>,
    language_direction: Option<HtmlLanguageDirectionFacts>,
    inert_revision: Option<HtmlFlatTreeRevision>,
    inert: Option<HtmlInertFacts>,
    controls: Option<HtmlControlFacts>,
    output_defaults: Option<HtmlOutputDefaultFacts>,
    global_inheritance_revision: Option<HtmlGlobalInheritanceRevision>,
    global_inheritance: Option<HtmlGlobalInheritanceFacts>,
}
impl HtmlSemanticContextInput {
    pub fn new(classifications: Vec<HtmlCustomControlClassification>) -> Self {
        Self {
            classifications,
            participation_revision: None,
            participation: None,
            language_direction: None,
            inert_revision: None,
            inert: None,
            controls: None,
            output_defaults: None,
            global_inheritance_revision: None,
            global_inheritance: None,
        }
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
    pub fn with_inert(
        mut self,
        expected: HtmlFlatTreeRevision,
        facts: Option<HtmlInertFacts>,
    ) -> Self {
        self.inert_revision = Some(expected);
        self.inert = facts;
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
    identity: HtmlSemanticContextId,
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    classifications: Vec<HtmlCustomControlClassification>,
    participation_revision: Option<HtmlListParticipationRevision>,
    participation: Option<HtmlListParticipation>,
    language_direction: Option<HtmlLanguageDirectionFacts>,
    inert_revision: Option<HtmlFlatTreeRevision>,
    inert: Option<HtmlInertFacts>,
    controls: Option<HtmlControlFacts>,
    output_defaults: Option<HtmlOutputDefaultFacts>,
    global_inheritance_revision: Option<HtmlGlobalInheritanceRevision>,
    global_inheritance: Option<HtmlGlobalInheritanceFacts>,
    empty_language: DocumentString,
}
/// Rejected producer context; no query result is created.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticContextError {
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
            classifications,
            participation_revision,
            participation,
            language_direction,
            inert_revision,
            inert,
            controls,
            output_defaults,
            global_inheritance_revision,
            global_inheritance,
        } = input;
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
            identity: HtmlSemanticContextId(Arc::new(())),
            capture: capture.identity().clone(),
            host_revision,
            classifications,
            participation_revision,
            participation,
            language_direction,
            inert_revision,
            inert,
            controls,
            output_defaults,
            global_inheritance_revision,
            global_inheritance,
            empty_language: DocumentString::default(),
        })
    }
    pub fn identity(&self) -> &HtmlSemanticContextId {
        &self.identity
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
    pub fn controls(&self) -> Option<&HtmlControlFacts> {
        self.controls.as_ref()
    }
    pub fn global_inheritance_revision(&self) -> Option<&HtmlGlobalInheritanceRevision> {
        self.global_inheritance_revision.as_ref()
    }
    pub fn global_inheritance(&self) -> Option<&HtmlGlobalInheritanceFacts> {
        self.global_inheritance.as_ref()
    }
    fn empty_language(&self) -> &DocumentString {
        &self.empty_language
    }
}

/// Finite cumulative admission for one semantic session.
///
/// Work charges visits, result writes, inspected UTF-16 units and conservative
/// inspection envelopes for the existing intrinsic producers. Output counts
/// result records reserved across batches. Scratch bounds simultaneous retained
/// temporary vector capacity; nested output counts cumulative model capacities.
/// Allocated bytes count cumulative vector capacity growth bytes and exact
/// integer allocation capacities, including discarded intermediate values.
/// Integer digits bound each numeric output. Numeric scans/writes and allocator
/// calls are finite atomic operations with cancellation polls before and after;
/// these controls do not promise parser cancellation quanta or allocator-internal
/// memory bounds. Floating conversion and static Unicode lookup are bounded
/// atomic provider operations; work charges their visits and owned inspection
/// envelopes, without claiming internal instruction counts. Capture retention
/// and parsing have their separate controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSemanticLimits {
    work: usize,
    output_records: usize,
    scratch_records: usize,
    integer_digits: usize,
    allocated_bytes: usize,
    nested_output_records: usize,
}
impl HtmlSemanticLimits {
    pub fn new(
        work: usize,
        output_records: usize,
        scratch_records: usize,
        integer_digits: usize,
        allocated_bytes: usize,
        nested_output_records: usize,
    ) -> Self {
        Self {
            work,
            output_records,
            scratch_records,
            integer_digits,
            allocated_bytes,
            nested_output_records,
        }
    }
    pub fn work(self) -> usize {
        self.work
    }
    pub fn output_records(self) -> usize {
        self.output_records
    }
    pub fn scratch_records(self) -> usize {
        self.scratch_records
    }
    pub fn integer_digits(self) -> usize {
        self.integer_digits
    }
    pub fn allocated_bytes(self) -> usize {
        self.allocated_bytes
    }
    pub fn nested_output_records(self) -> usize {
        self.nested_output_records
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlSemanticUsage {
    work: usize,
    output_records: usize,
    scratch_records: usize,
    allocated_bytes: usize,
    peak_scratch_records: usize,
    nested_output_records: usize,
}
impl HtmlSemanticUsage {
    pub fn work(self) -> usize {
        self.work
    }
    pub fn output_records(self) -> usize {
        self.output_records
    }
    pub fn scratch_records(self) -> usize {
        self.scratch_records
    }
    pub fn allocated_bytes(self) -> usize {
        self.allocated_bytes
    }
    pub fn peak_scratch_records(self) -> usize {
        self.peak_scratch_records
    }
    pub fn nested_output_records(self) -> usize {
        self.nested_output_records
    }
}
/// Caller-owned cancellation; already returned completed reports remain valid.
#[derive(Clone, Debug, Default)]
pub struct HtmlSemanticCancellation(Arc<AtomicBool>);
impl HtmlSemanticCancellation {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticQuery {
    Hidden,
    Disabledness,
    ListOwner,
    ListStartingValue,
    ListOrdinal,
    TableModel,
    TableBodies,
    TableRows,
    TableSectionRows,
    TableRowCells,
    Language,
    Directionality,
    Meter,
    Progress,
    Inertness,
    FormOwner,
    LabeledControl,
    /// Form owner of the labeled control; this does not associate the label.
    LabelFormOwner,
    ControlLabels,
    FormControls,
    FormLength,
    FieldsetControls,
    CurrentCheckedness,
    CurrentSelectedness,
    DirtyValueFlag,
    DirtyCheckednessFlag,
    OptionDirtiness,
    Indeterminateness,
    EditingHost,
    Editable,
    IsContentEditable,
    OwnAutocapitalizationHint,
    UsedAutocapitalizationHint,
    UsedAutocorrectionState,
    ComputedWritingSuggestions,

    TranslationMode,
    Spellcheck,
    ComputedHeadingLevel,
    SelectOptions,
    SelectedOptions,
    InputSuggestionsSource,
    DatalistOptions,
    /// Ordinary HTML area descendants in source tree order.
    MapAreas,
    /// Independent img usemap author witness and first processing map.
    UsemapReference,
    ConstraintValidationCandidate,
    TabIndex,
    DetailsSummary,
    SummaryForParentDetails,
    DetailsNameGroup,
    DatalistSuggestionOptions,
    ImageMapAttribute,
    ImageControlsAttribute,
    StylesheetDisabledAttribute,
    HyperlinkReferrerPolicyAttribute,
    TimeElement,
    TextareaDefaultValue,
    OutputValue,
    OutputDefaultValue,
    SelectSelectedIndex,
    OptionIndex,
    OptionFormOwner,
    OptionText,
    OptionLabel,
    OptionValue,
    SelectValue,
    /// Source placeholder classification; this does not change selection.
    SelectPlaceholderLabelOption,
    /// Static select missingness from supplied current selectedness.
    SelectValueMissing,
    ControlAutocomplete,
    SubmitterFormMethod,
    SubmitterFormEnctype,
    SubmitterNoValidate,
    FormDefaultButton,
    FormImplicitSubmissionCondition,
}
impl HtmlSemanticQuery {
    pub fn source_clause(self) -> &'static str {
        match self {
            Self::ImageMapAttribute => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-ismap"
            }
            Self::ImageControlsAttribute => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-img-controls"
            }
            Self::StylesheetDisabledAttribute => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-link-disabled"
            }
            Self::HyperlinkReferrerPolicyAttribute => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-referrerpolicy"
            }
            Self::TimeElement => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-time-element"
            }
            Self::TextareaDefaultValue => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-textarea-defaultvalue"
            }
            Self::OutputValue => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-output-value"
            }
            Self::OutputDefaultValue => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-output-defaultvalue"
            }
            Self::InputSuggestionsSource => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-input-list"
            }
            Self::ConstraintValidationCandidate => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#candidate-for-constraint-validation"
            }
            Self::TabIndex => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tabindex"
            }
            Self::DetailsSummary => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-details-element"
            }
            Self::SummaryForParentDetails => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#summary-for-its-parent-details"
            }
            Self::DetailsNameGroup => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#details-name-group"
            }
            Self::UsemapReference => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-usemap"
            }
            Self::MapAreas => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-map-areas"
            }
            Self::DatalistOptions => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-datalist-options"
            }
            Self::DatalistSuggestionOptions => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-datalist-element"
            }
            Self::OptionFormOwner => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-option-form"
            }
            Self::OptionText => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-option-text"
            }
            Self::OptionLabel => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-option-label"
            }
            Self::OptionValue => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-option-value"
            }
            Self::SelectValueMissing => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-select-element"
            }
            Self::SelectPlaceholderLabelOption => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#placeholder-label-option"
            }
            Self::SelectValue => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-select-value"
            }
            Self::ControlLabels => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-lfe-labels"
            }
            Self::FormControls => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-form-elements"
            }
            Self::FormLength => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-form-length"
            }
            Self::FieldsetControls => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-fieldset-elements"
            }
            Self::FormDefaultButton => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#default-button"
            }
            Self::FormImplicitSubmissionCondition => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#implicit-submission"
            }
            Self::Hidden => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hidden"
            }
            Self::Disabledness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-element-disabled"
            }
            Self::ListOwner => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#list-owner"
            }
            Self::ListStartingValue => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-ol-start"
            }
            Self::ListOrdinal => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ordinal-value"
            }
            Self::TableBodies => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-table-tbodies"
            }
            Self::TableRows => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-table-rows"
            }
            Self::TableSectionRows => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tbody-rows"
            }
            Self::TableRowCells => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tr-cells"
            }
            Self::TableModel => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#table-processing-model"
            }
            Self::Language => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-lang-and-xml:lang-attributes"
            }
            Self::Directionality => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dir-attribute"
            }
            Self::Meter => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-meter-element"
            }
            Self::Progress => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-progress-element"
            }
            Self::Inertness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#inert-subtrees"
            }
            Self::FormOwner => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#form-owner"
            }
            Self::LabelFormOwner => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-label-form"
            }
            Self::LabeledControl => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#labeled-control"
            }
            Self::CurrentCheckedness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-checked"
            }
            Self::CurrentSelectedness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-option-selectedness"
            }
            Self::DirtyValueFlag => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fe-dirty"
            }
            Self::DirtyCheckednessFlag => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-input-checked-dirty-flag"
            }
            Self::OptionDirtiness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-option-dirtiness"
            }
            Self::Indeterminateness => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-input-indeterminate"
            }
            Self::EditingHost => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host"
            }
            Self::Editable => {
                "https://github.com/w3c/editing/blob/85de9ff4a9816139d06fc2dc39d5dacdc456fddc/docs/execCommand/index.html#L1053"
            }
            Self::OwnAutocapitalizationHint => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#own-autocapitalization-hint"
            }
            Self::UsedAutocapitalizationHint => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocapitalization-hint"
            }
            Self::UsedAutocorrectionState => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#used-autocorrection-state"
            }
            Self::ComputedWritingSuggestions => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#computed-writing-suggestions-value"
            }
            Self::IsContentEditable => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-iscontenteditable"
            }
            Self::TranslationMode => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-translate"
            }
            Self::Spellcheck => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-spellcheck"
            }
            Self::SelectOptions => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-select-options"
            }
            Self::SelectedOptions => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-select-selectedoptions"
            }
            Self::SelectSelectedIndex => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-select-selectedindex"
            }
            Self::SubmitterNoValidate => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fs-novalidate"
            }
            Self::SubmitterFormMethod => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fs-method"
            }
            Self::SubmitterFormEnctype => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-fs-enctype"
            }
            Self::ControlAutocomplete => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#autofill-processing-model"
            }
            Self::OptionIndex => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-option-index"
            }
            Self::ComputedHeadingLevel => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-an-element's-computed-heading-level"
            }
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticRequest {
    subject: HtmlCaptureSubjectId,
    query: HtmlSemanticQuery,
}
impl HtmlSemanticRequest {
    pub fn new(subject: HtmlCaptureSubjectId, query: HtmlSemanticQuery) -> Self {
        Self { subject, query }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn query(&self) -> HtmlSemanticQuery {
        self.query
    }
}
/// Evidence needed by an implemented query but not established by its producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticMissing {
    Parent(HtmlCaptureSubjectId),
    OrderedChildren(HtmlCaptureSubjectId),
    CustomControlClassification(HtmlCaptureSubjectId),
    ComputedListItem(HtmlCaptureSubjectId),
    BeingRendered(HtmlCaptureSubjectId),
    ProducesCssBox(HtmlCaptureSubjectId),
    LanguageFallback(HtmlCaptureSubjectId),
    CurrentValue(HtmlCaptureSubjectId),
    OutputDefaultValueOverride(HtmlCaptureSubjectId),
    SlotAssignment(HtmlCaptureSubjectId),
    ShadowHost(HtmlCaptureSubjectId),
    InertNodeFacts(HtmlCaptureSubjectId),
    InertAncestry(HtmlCaptureSubjectId),
    Connectedness(HtmlCaptureSubjectId),
    ModalBlocking(HtmlCaptureSubjectId),
    DocumentDesignMode(HtmlCaptureSubjectId),
    SpellcheckDefault(HtmlCaptureSubjectId),
    FormAssociation(HtmlCaptureSubjectId),
    ControlState {
        subject: HtmlCaptureSubjectId,
        flag: HtmlControlFlag,
    },
}
/// Hidden and disabledness do not implement foreign-language semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticUnsupported {
    ForeignNamespace,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticValue<T> {
    Known(T),
    Inapplicable,
    Missing(HtmlSemanticMissing),
    Unsupported(HtmlSemanticUnsupported),
}
impl<T> HtmlSemanticValue<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> HtmlSemanticValue<U> {
        match self {
            Self::Known(value) => HtmlSemanticValue::Known(f(value)),
            Self::Inapplicable => HtmlSemanticValue::Inapplicable,
            Self::Missing(value) => HtmlSemanticValue::Missing(value),
            Self::Unsupported(value) => HtmlSemanticValue::Unsupported(value),
        }
    }
}
/// Complete source-ordered option identities from one qualified query.
/// The query/fact selects the collection's source filter; this immutable payload
/// has no live owner or mutation behavior. Construction remains query-owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlOptionCollection {
    options: Vec<HtmlCaptureSubjectId>,
}
impl HtmlOptionCollection {
    pub fn options(&self) -> &[HtmlCaptureSubjectId] {
        &self.options
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HtmlSemanticFact {
    Hidden(HtmlSemanticValue<HtmlHiddenState>),
    Disabledness(HtmlSemanticValue<bool>),
    ListOwner(HtmlSemanticValue<Option<HtmlCaptureSubjectId>>),
    ListStartingValue(HtmlSemanticValue<HtmlInteger>),
    /// `Known(None)` is a computed list item whose source list owner is null.
    ListOrdinal(HtmlSemanticValue<Option<HtmlInteger>>),
    TableBodies(HtmlSemanticValue<HtmlTableBodyCollection>),
    TableRows(HtmlSemanticValue<HtmlTableRowCollection>),
    TableSectionRows(HtmlSemanticValue<HtmlTableRowCollection>),
    TableRowCells(HtmlSemanticValue<HtmlTableCellCollection>),
    TableModel(HtmlSemanticValue<HtmlTableModel>),
    Language(HtmlSemanticValue<HtmlLanguage>),
    Directionality(HtmlSemanticValue<HtmlDirectionality>),
    Meter(HtmlSemanticValue<HtmlMeterState>),
    Progress(HtmlSemanticValue<HtmlProgressState>),
    Inertness(HtmlSemanticValue<bool>),
    FormOwner(HtmlSemanticValue<HtmlFormOwner>),
    LabeledControl(HtmlSemanticValue<Option<HtmlCaptureSubjectId>>),
    /// Known(None) is established null; unavailable association remains Missing.
    LabelFormOwner(HtmlSemanticValue<Option<HtmlCaptureSubjectId>>),
    /// Known(None) is the source Hidden-input null; Some(empty) is a complete empty list.
    ControlLabels(HtmlSemanticValue<Option<HtmlControlLabels>>),
    FormControls(HtmlSemanticValue<HtmlListedControlCollection>),
    FormLength(HtmlSemanticValue<usize>),
    FieldsetControls(HtmlSemanticValue<HtmlListedControlCollection>),
    CurrentCheckedness(HtmlSemanticValue<bool>),
    CurrentSelectedness(HtmlSemanticValue<bool>),
    DirtyValueFlag(HtmlSemanticValue<bool>),
    DirtyCheckednessFlag(HtmlSemanticValue<bool>),
    OptionDirtiness(HtmlSemanticValue<bool>),
    Indeterminateness(HtmlSemanticValue<bool>),
    EditingHost(HtmlSemanticValue<bool>),
    Editable(HtmlSemanticValue<bool>),
    IsContentEditable(HtmlSemanticValue<bool>),
    OwnAutocapitalizationHint(HtmlSemanticValue<HtmlOwnAutocapitalizationHint>),
    UsedAutocapitalizationHint(HtmlSemanticValue<HtmlUsedAutocapitalizationHint>),
    UsedAutocorrectionState(HtmlSemanticValue<HtmlUsedAutocorrectionState>),
    ComputedWritingSuggestions(HtmlSemanticValue<HtmlComputedWritingSuggestions>),

    TranslationMode(HtmlSemanticValue<HtmlTranslationMode>),
    Spellcheck(HtmlSemanticValue<bool>),
    ComputedHeadingLevel(HtmlSemanticValue<HtmlComputedHeadingLevel>),
    SelectOptions(HtmlSemanticValue<HtmlOptionCollection>),
    SelectedOptions(HtmlSemanticValue<HtmlOptionCollection>),
    /// Known(None) establishes that the input has no source of suggestions.
    InputSuggestionsSource(HtmlSemanticValue<Option<HtmlCaptureSubjectId>>),
    DatalistOptions(HtmlSemanticValue<HtmlOptionCollection>),
    MapAreas(HtmlSemanticValue<HtmlAreaCollection>),
    UsemapReference(HtmlSemanticValue<HtmlUsemapReferenceResult>),
    ConstraintValidationCandidate(HtmlSemanticValue<bool>),
    TabIndex(HtmlSemanticValue<HtmlTabIndexReadback>),
    DetailsSummary(HtmlSemanticValue<Option<HtmlCaptureSubjectId>>),
    SummaryForParentDetails(HtmlSemanticValue<bool>),
    DetailsNameGroup(HtmlSemanticValue<HtmlDetailsNameGroup>),
    ImageMapAttribute(HtmlSemanticValue<HtmlBooleanAuthorContextReadback>),
    ImageControlsAttribute(HtmlSemanticValue<HtmlBooleanAuthorContextReadback>),
    StylesheetDisabledAttribute(HtmlSemanticValue<HtmlBooleanAuthorContextReadback>),
    HyperlinkReferrerPolicyAttribute(HtmlSemanticValue<HtmlHyperlinkReferrerPolicyReadback>),
    TimeElement(HtmlSemanticValue<HtmlTimeElementReadback>),
    TextareaDefaultValue(HtmlSemanticValue<HtmlControlTextReadback>),
    OutputValue(HtmlSemanticValue<HtmlControlTextReadback>),
    OutputDefaultValue(HtmlSemanticValue<HtmlControlTextReadback>),
    /// Membership only: source-option-enabled descendants with nonempty values.
    DatalistSuggestionOptions(HtmlSemanticValue<HtmlOptionCollection>),
    /// `Known(None)` represents the source selected index of −1.
    SelectSelectedIndex(HtmlSemanticValue<Option<usize>>),
    OptionIndex(HtmlSemanticValue<usize>),
    OptionFormOwner(HtmlSemanticValue<HtmlOptionFormOwner>),
    OptionText(HtmlSemanticValue<HtmlOptionReadback>),
    OptionLabel(HtmlSemanticValue<HtmlOptionReadback>),
    OptionValue(HtmlSemanticValue<HtmlOptionReadback>),
    SelectValue(HtmlSemanticValue<HtmlSelectValue>),
    /// Known(None) establishes that no source placeholder label option exists.
    SelectPlaceholderLabelOption(HtmlSemanticValue<Option<HtmlCaptureSubjectId>>),
    SelectValueMissing(HtmlSemanticValue<bool>),
    ControlAutocomplete(HtmlSemanticValue<HtmlControlAutocomplete>),
    SubmitterFormMethod(HtmlSemanticValue<HtmlSubmitterFormMethod>),
    SubmitterFormEnctype(HtmlSemanticValue<HtmlSubmitterFormEnctype>),
    SubmitterNoValidate(HtmlSemanticValue<HtmlSubmitterNoValidate>),
    /// Known(None) proves no owned submit button in the form's ordinary tree.
    FormDefaultButton(HtmlSemanticValue<Option<HtmlDefaultButton>>),
    FormImplicitSubmissionCondition(HtmlSemanticValue<HtmlImplicitSubmissionCondition>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlSemanticResult {
    request: HtmlSemanticRequest,
    fact: HtmlSemanticFact,
}
impl HtmlSemanticResult {
    pub fn request(&self) -> &HtmlSemanticRequest {
        &self.request
    }
    pub fn fact(&self) -> &HtmlSemanticFact {
        &self.fact
    }
}
/// Invalidation contract: recompute when any qualification changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticDependencies {
    ConservativeRecompute,
}
/// Exact Unicode data selected for HTML's first-strong directionality lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDirectionalitySource {
    unicode_version: &'static str,
    uax9_revision: u16,
    derived_bidi_class_sha256: &'static str,
}
impl HtmlDirectionalitySource {
    pub fn unicode_version(self) -> &'static str {
        self.unicode_version
    }
    pub fn uax9_revision(self) -> u16 {
        self.uax9_revision
    }
    pub fn derived_bidi_class_sha256(self) -> &'static str {
        self.derived_bidi_class_sha256
    }
}
/// Immutable qualification of one admitted session's semantic inputs and controls.
/// This stores identities and finite configuration, never tree or query payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticQualification {
    capture: HtmlCaptureId,
    context: HtmlSemanticContextId,
    host_revision: HtmlHostFactsRevision,
    limits: HtmlSemanticLimits,
    attribute_input_limit: usize,
    participation_revision: Option<HtmlListParticipationRevision>,
    directionality_source: HtmlDirectionalitySource,
    editability_source: HtmlEditabilitySource,
    global_inheritance_revision: Option<HtmlGlobalInheritanceRevision>,
    inert_revision: Option<HtmlFlatTreeRevision>,
}
impl HtmlSemanticQualification {
    pub fn source_revision(&self) -> &'static str {
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn context(&self) -> &HtmlSemanticContextId {
        &self.context
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn limits(&self) -> HtmlSemanticLimits {
        self.limits
    }
    pub fn attribute_input_limit(&self) -> usize {
        self.attribute_input_limit
    }
    pub fn participation_revision(&self) -> Option<&HtmlListParticipationRevision> {
        self.participation_revision.as_ref()
    }
    pub fn inert_revision(&self) -> Option<&HtmlFlatTreeRevision> {
        self.inert_revision.as_ref()
    }
    pub fn global_inheritance_revision(&self) -> Option<&HtmlGlobalInheritanceRevision> {
        self.global_inheritance_revision.as_ref()
    }
    pub fn editability_source(&self) -> HtmlEditabilitySource {
        self.editability_source
    }
    pub fn directionality_source(&self) -> HtmlDirectionalitySource {
        self.directionality_source
    }
}
/// Complete ordered results, qualified by every influencing input.
#[derive(Debug)]
pub struct HtmlSemanticReport {
    qualification: Arc<HtmlSemanticQualification>,
    usage: HtmlSemanticUsage,
    results: Vec<HtmlSemanticResult>,
}
impl HtmlSemanticReport {
    pub fn qualification(&self) -> &HtmlSemanticQualification {
        &self.qualification
    }
    pub fn source_revision(&self) -> &'static str {
        self.qualification.source_revision()
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        self.qualification.capture()
    }
    pub fn context(&self) -> &HtmlSemanticContextId {
        self.qualification.context()
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        self.qualification.host_revision()
    }
    pub fn limits(&self) -> HtmlSemanticLimits {
        self.qualification.limits()
    }
    pub fn attribute_input_limit(&self) -> usize {
        self.qualification.attribute_input_limit()
    }
    pub fn usage(&self) -> HtmlSemanticUsage {
        self.usage
    }
    pub fn results(&self) -> &[HtmlSemanticResult] {
        &self.results
    }
    pub fn dependencies(&self) -> HtmlSemanticDependencies {
        HtmlSemanticDependencies::ConservativeRecompute
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSemanticResource {
    Work,
    OutputRecords,
    ScratchRecords,
    IntegerDigits,
    AllocatedBytes,
    NestedOutputRecords,
}
/// Terminal failure returns usage and no requested result prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSemanticFailureKind {
    Cancelled,
    LimitExceeded {
        resource: HtmlSemanticResource,
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    AllocationFailed,
    Capture(HtmlCaptureError),
    ContextCaptureMismatch,
    Attribute(HtmlValueAdmissionError),
    IntrinsicUnavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSemanticFailure {
    kind: HtmlSemanticFailureKind,
    usage: HtmlSemanticUsage,
    qualification: Option<Arc<HtmlSemanticQualification>>,
}
impl HtmlSemanticFailure {
    pub fn kind(&self) -> &HtmlSemanticFailureKind {
        &self.kind
    }
    pub fn usage(&self) -> HtmlSemanticUsage {
        self.usage
    }
    /// Every terminal query failure retains its original admitted session key.
    /// `None` denotes a constructor rejection before any valid session existed.
    pub fn qualification(&self) -> Option<&HtmlSemanticQualification> {
        self.qualification.as_deref()
    }
}
impl fmt::Display for HtmlSemanticFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
impl Error for HtmlSemanticFailure {}

/// Reusable borrowed boundary for the implemented capture-qualified queries.
/// Supplements provide current control, document design mode, default policy,
/// flat-tree and modal evidence; this session neither mutates those states nor
/// computes their producer transitions.
/// The first terminal failure is sticky and retains the original qualification.
pub struct HtmlSemanticSession<'a> {
    capture: &'a HtmlCapture,
    context: &'a HtmlSemanticContext,
    interpreter: HtmlValueInterpreter,
    limits: HtmlSemanticLimits,
    cancellation: HtmlSemanticCancellation,
    qualification: Arc<HtmlSemanticQualification>,
    usage: HtmlSemanticUsage,
    failure: Option<HtmlSemanticFailure>,
}
#[derive(Clone, Copy)]
enum HtmlParticipationField {
    ListItem,
    Rendered,
    CssBox,
}
impl<'a> HtmlSemanticSession<'a> {
    pub fn new(
        capture: &'a HtmlCapture,
        context: &'a HtmlSemanticContext,
        interpreter: HtmlValueInterpreter,
        limits: HtmlSemanticLimits,
        cancellation: HtmlSemanticCancellation,
    ) -> Result<Self, HtmlSemanticFailure> {
        if capture.identity() != context.capture() {
            return Err(HtmlSemanticFailure {
                kind: HtmlSemanticFailureKind::ContextCaptureMismatch,
                usage: HtmlSemanticUsage::default(),
                qualification: None,
            });
        }
        Ok(Self {
            capture,
            context,
            interpreter,
            limits,
            cancellation,
            // Prepare this fixed-size key before execution. Terminal stops and
            // repeated failures only clone its Arc; they do not allocate.
            qualification: Arc::new(HtmlSemanticQualification {
                capture: capture.identity().clone(),
                context: context.identity.clone(),
                host_revision: context.host_revision.clone(),
                limits,
                attribute_input_limit: interpreter.max_input_code_units(),
                participation_revision: context.participation_revision.clone(),
                inert_revision: context.inert_revision.clone(),
                global_inheritance_revision: context.global_inheritance_revision.clone(),
                editability_source: HtmlEditabilitySource,
                directionality_source: HtmlDirectionalitySource {
                    unicode_version: crate::bidi::UCD_VERSION,
                    uax9_revision: crate::bidi::UAX9_REVISION,
                    derived_bidi_class_sha256: crate::bidi::DERIVED_BIDI_CLASS_SHA256,
                },
            }),
            usage: HtmlSemanticUsage::default(),
            failure: None,
        })
    }
    pub fn usage(&self) -> HtmlSemanticUsage {
        self.usage
    }
    pub fn failure(&self) -> Option<&HtmlSemanticFailure> {
        self.failure.as_ref()
    }
    fn stop(&mut self, kind: HtmlSemanticFailureKind) -> HtmlSemanticFailure {
        self.failure
            .get_or_insert(HtmlSemanticFailure {
                kind,
                usage: self.usage,
                qualification: Some(self.qualification.clone()),
            })
            .clone()
    }
    fn ordinary_tree_root(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Result<HtmlCaptureSubjectId, HtmlSemanticMissing>, HtmlSemanticFailure> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            match self.parent(node)? {
                Ok(Some(parent)) => current = parent,
                Ok(None) => return Ok(Ok(current)),
                Err(missing) => return Ok(Err(missing)),
            }
        }
    }

    // Descend only after inspecting the current node. This lets a first match
    // succeed without reading its children or any later branch. Host/slot edges
    // are never ordinary-tree traversal edges.
    fn ordinary_tree_next(
        &mut self,
        node: HtmlCapturedNode<'a>,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        if matches!(
            node.kind(),
            HtmlCapturedNodeKind::Element(_)
                | HtmlCapturedNodeKind::Document(_)
                | HtmlCapturedNodeKind::Fragment(_)
        ) {
            self.charge(1)?;
            let children = match node.children() {
                HtmlCapturedChildren::Complete(children) => children,
                HtmlCapturedChildren::Unavailable => {
                    return Ok(Err(HtmlSemanticMissing::OrderedChildren(node.id())));
                }
            };
            self.reserve_scratch(frames, 1)?;
            self.charge(1)?;
            frames.push(children);
        }
        Ok(Ok(self.ordinary_tree_resume(frames)?))
    }

    // Resume only established ancestor iterators. A caller that skips unavailable
    // children must retain that dependency unless an independent proof resolves it.
    fn ordinary_tree_resume(
        &mut self,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<Option<HtmlCaptureSubjectId>, HtmlSemanticFailure> {
        loop {
            self.charge(1)?;
            let Some(frame) = frames.last_mut() else {
                return Ok(None);
            };
            if let Some(subject) = frame.next() {
                return Ok(Some(subject));
            }
            frames.pop();
        }
    }

    fn poll(&mut self) -> Result<(), HtmlSemanticFailure> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        if self.cancellation.is_cancelled() {
            return Err(self.stop(HtmlSemanticFailureKind::Cancelled));
        }
        Ok(())
    }
    fn charge(&mut self, work: usize) -> Result<(), HtmlSemanticFailure> {
        self.poll()?;
        let Some(required) = self.usage.work.checked_add(work) else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        if required > self.limits.work {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: self.limits.work,
                required,
            }));
        }
        self.usage.work = required;
        Ok(())
    }
    fn node(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, HtmlSemanticFailure> {
        self.charge(1)?;
        self.capture
            .node(subject)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Capture(e)))
    }
    fn known(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, HtmlSemanticFailure> {
        // Envelope covers every candidate spelling and all supplied-name units
        // before the existing vocabulary owner performs its bounded recognition.
        let units = element.name().local_name().len();
        let ns_units = element.name().namespace().uri().map_or(0, |s| s.len());
        let envelope = units
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(ns_units));
        let Some(envelope) = envelope else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        self.charge(envelope)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
    fn attribute(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        self.attribute_ns(element, None, local)
    }
    fn attribute_ns(
        &mut self,
        element: &'a HtmlCapturedElement,
        namespace: Option<&str>,
        local: &str,
    ) -> Result<Option<&'a Attribute>, HtmlSemanticFailure> {
        for attribute in element.attributes().iter() {
            self.charge(1)?;
            let namespace_matches = match (attribute.name().namespace().uri(), namespace) {
                (None, None) => true,
                (Some(uri), Some(expected)) if uri.len() == expected.len() => {
                    let mut equal = true;
                    for (unit, byte) in uri.code_units().iter().zip(expected.bytes()) {
                        self.charge(1)?;
                        if *unit != u16::from(byte) {
                            equal = false;
                            break;
                        }
                    }
                    equal
                }
                _ => false,
            };
            if !namespace_matches {
                continue;
            }
            let units = attribute.name().local_name().code_units();
            if units.len() != local.len() {
                continue;
            }
            let mut equal = true;
            for (unit, byte) in units.iter().zip(local.bytes()) {
                self.charge(1)?;
                if *unit != u16::from(byte) {
                    equal = false;
                    break;
                }
            }
            if equal {
                return Ok(Some(attribute));
            }
        }
        Ok(None)
    }
    fn html_namespace(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.namespace_is(element, "http://www.w3.org/1999/xhtml")
    }
    fn namespace_is(
        &mut self,
        element: &HtmlCapturedElement,
        expected: &str,
    ) -> Result<bool, HtmlSemanticFailure> {
        self.charge(1)?;
        let Some(uri) = element.name().namespace().uri() else {
            return Ok(false);
        };
        if uri.len() != expected.len() {
            return Ok(false);
        }
        for (unit, byte) in uri.code_units().iter().zip(expected.bytes()) {
            self.charge(1)?;
            if *unit != u16::from(byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn parent(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        self.charge(1)?;
        Ok(match node.parent() {
            HtmlCapturedParent::Root => Ok(None),
            HtmlCapturedParent::Subject(id) => Ok(Some(id)),
            HtmlCapturedParent::Unavailable => Err(HtmlSemanticMissing::Parent(node.id())),
        })
    }
    fn disabled_attribute(
        &mut self,
        element: &'a HtmlCapturedElement,
        custom_control: bool,
    ) -> Result<bool, HtmlSemanticFailure> {
        let raw = self.attribute(element, "disabled")?;
        self.admit_intrinsic(element, raw, 1)?;
        // The intrinsic owner supplies presence, conformance and applicability.
        let report = if custom_control {
            self.interpreter.disabled_for_custom_control(
                element.name(),
                raw,
                HtmlCustomControlAttributeContext::FormAssociated,
            )
        } else {
            self.interpreter.disabled(element.name(), raw)
        }
        .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn admit_intrinsic(
        &mut self,
        element: &HtmlCapturedElement,
        raw: Option<&Attribute>,
        keyword_count: usize,
    ) -> Result<(), HtmlSemanticFailure> {
        // Existing intrinsic rules own recognition and defaults. This envelope
        // admits their finite vocabulary searches and keyword inspections as
        // one atomic bounded operation; it is not parser byte accounting.
        let element_units = element.name().local_name().len();
        let namespace_units = element.name().namespace().uri().map_or(0, |v| v.len());
        let attribute_units = raw.map_or(0, |a| a.name().local_name().len());
        let value_units = raw.map_or(0, |a| a.value().len());
        let envelope = element_units
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| {
                attribute_units
                    .checked_add(1)
                    .and_then(|a| a.checked_mul(KnownHtmlAttribute::ALL.len()))
                    .and_then(|a| n.checked_add(a))
            })
            .and_then(|n| {
                value_units
                    .checked_mul(keyword_count)
                    .and_then(|v| n.checked_add(v))
            })
            .and_then(|n| {
                namespace_units
                    .checked_add(1)
                    .and_then(|s| s.checked_mul(3))
                    .and_then(|s| n.checked_add(s))
            });
        let Some(envelope) = envelope else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        self.charge(envelope)
    }
    fn form_disabled(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        custom_control: bool,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if self.disabled_attribute(element, custom_control)? {
            return Ok(HtmlSemanticValue::Known(true));
        }
        let mut current = node;
        let mut branch = node.id();
        let mut missing_children = None;
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                Ok(None) => {
                    return Ok(missing_children
                        .map_or(HtmlSemanticValue::Known(false), HtmlSemanticValue::Missing));
                }
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            let ancestor = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = ancestor.kind()
                && self.known(e)? == Some(KnownHtmlElement::Fieldset)
                && self.disabled_attribute(e, false)?
            {
                // A descendant can be inside the first legend only through a
                // direct HTML legend child. Other branches need no child-order
                // evidence to establish this fieldset's disabling effect.
                let branch_node = self.node(&branch)?;
                let legend_branch = match branch_node.kind() {
                    HtmlCapturedNodeKind::Element(e) => {
                        self.known(e)? == Some(KnownHtmlElement::Legend)
                    }
                    _ => false,
                };
                if !legend_branch {
                    return Ok(HtmlSemanticValue::Known(true));
                }
                self.charge(1)?;
                match ancestor.children() {
                    HtmlCapturedChildren::Unavailable => {
                        missing_children = Some(HtmlSemanticMissing::OrderedChildren(id.clone()))
                    }
                    HtmlCapturedChildren::Complete(children) => {
                        let mut first_legend = None;
                        let mut children = children;
                        loop {
                            self.charge(1)?;
                            let Some(child) = children.next() else {
                                break;
                            };
                            let child_node = self.node(&child)?;
                            if let HtmlCapturedNodeKind::Element(e) = child_node.kind()
                                && self.known(e)? == Some(KnownHtmlElement::Legend)
                            {
                                first_legend = Some(child);
                                break;
                            }
                        }
                        if first_legend.as_ref() != Some(&branch) {
                            return Ok(HtmlSemanticValue::Known(true));
                        }
                    }
                }
            }
            branch = id;
            current = ancestor;
        }
    }
    fn nearest_select(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Result<Option<HtmlCaptureSubjectId>, HtmlSemanticMissing>, HtmlSemanticFailure>
    {
        let mut current = self.node(subject)?;
        let mut optgroup = false;
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                other => return Ok(other),
            };
            current = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = current.kind() {
                match self.known(e)? {
                    Some(
                        KnownHtmlElement::Datalist
                        | KnownHtmlElement::Hr
                        | KnownHtmlElement::Option,
                    ) => return Ok(Ok(None)),
                    Some(KnownHtmlElement::Optgroup) if optgroup => return Ok(Ok(None)),
                    Some(KnownHtmlElement::Optgroup) => optgroup = true,
                    Some(KnownHtmlElement::Select) => return Ok(Ok(Some(id))),
                    _ => {}
                }
            }
        }
    }
    fn option_disabled(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = self.node(subject)?;
        if let HtmlCapturedNodeKind::Element(e) = current.kind()
            && self.disabled_attribute(e, false)?
        {
            return Ok(HtmlSemanticValue::Known(true));
        }
        loop {
            let id = match self.parent(current)? {
                Ok(Some(id)) => id,
                Ok(None) => return Ok(HtmlSemanticValue::Known(false)),
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
            current = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(e) = current.kind() {
                match self.known(e)? {
                    Some(
                        KnownHtmlElement::Select
                        | KnownHtmlElement::Hr
                        | KnownHtmlElement::Datalist
                        | KnownHtmlElement::Option,
                    ) => return Ok(HtmlSemanticValue::Known(false)),
                    Some(KnownHtmlElement::Optgroup) => {
                        return Ok(HtmlSemanticValue::Known(self.disabled_attribute(e, false)?));
                    }
                    _ => {}
                }
            }
        }
    }
    fn disabledness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        match self.known(element)? {
            Some(
                KnownHtmlElement::Button
                | KnownHtmlElement::Input
                | KnownHtmlElement::Select
                | KnownHtmlElement::Textarea
                | KnownHtmlElement::Fieldset,
            ) => self.form_disabled(subject, false),
            Some(kind @ (KnownHtmlElement::Option | KnownHtmlElement::Optgroup)) => {
                let local = if kind == KnownHtmlElement::Option {
                    self.option_disabled(subject)?
                } else {
                    HtmlSemanticValue::Known(self.disabled_attribute(element, false)?)
                };
                if local == HtmlSemanticValue::Known(true) {
                    return Ok(local);
                }
                let select = match self.nearest_select(subject)? {
                    Ok(Some(id)) => self.form_disabled(&id, false)?,
                    Ok(None) => HtmlSemanticValue::Known(false),
                    Err(missing) => HtmlSemanticValue::Missing(missing),
                };
                Ok(if select == HtmlSemanticValue::Known(true) {
                    select
                } else if matches!(local, HtmlSemanticValue::Missing(_)) {
                    local
                } else {
                    select
                })
            }
            Some(_) => Ok(HtmlSemanticValue::Inapplicable),
            None => {
                let context = self.context;
                for fact in &context.classifications {
                    self.charge(1)?;
                    if &fact.subject == subject {
                        return if fact.form_associated {
                            self.form_disabled(subject, true)
                        } else {
                            Ok(HtmlSemanticValue::Inapplicable)
                        };
                    }
                }
                Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::CustomControlClassification(subject.clone()),
                ))
            }
        }
    }
    fn reserve_records<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        resource: HtmlSemanticResource,
    ) -> Result<(), HtmlSemanticFailure> {
        self.poll()?;
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        if needed <= values.capacity() {
            return if additional == 0 {
                Ok(())
            } else {
                self.charge(1)
            };
        }
        let (retained, limit) = match resource {
            HtmlSemanticResource::ScratchRecords => {
                (self.usage.scratch_records, self.limits.scratch_records)
            }
            HtmlSemanticResource::NestedOutputRecords => (
                self.usage.nested_output_records,
                self.limits.nested_output_records,
            ),
            _ => (self.usage.output_records, self.limits.output_records),
        };
        let growth = needed - values.capacity();
        let required = retained
            .checked_add(growth)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        if required > limit {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource,
                limit,
                required,
            }));
        }
        let requested_bytes = growth
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|n| n.checked_add(self.usage.allocated_bytes))
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        if requested_bytes > self.limits.allocated_bytes {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::AllocatedBytes,
                limit: self.limits.allocated_bytes,
                required: requested_bytes,
            }));
        }
        let before = values.capacity();
        self.charge(1)?;
        values
            .try_reserve_exact(needed - values.len())
            .map_err(|_| self.stop(HtmlSemanticFailureKind::AllocationFailed))?;
        let actual_growth = values.capacity() - before;
        let actual = retained
            .checked_add(actual_growth)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        let bytes = actual_growth
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|n| n.checked_add(self.usage.allocated_bytes))
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.usage.allocated_bytes = bytes;
        match resource {
            HtmlSemanticResource::ScratchRecords => {
                self.usage.scratch_records = actual;
                self.usage.peak_scratch_records = self.usage.peak_scratch_records.max(actual);
            }
            HtmlSemanticResource::NestedOutputRecords => self.usage.nested_output_records = actual,
            _ => self.usage.output_records = actual,
        }
        if actual > limit {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource,
                limit,
                required: actual,
            }));
        }
        if bytes > self.limits.allocated_bytes {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::AllocatedBytes,
                limit: self.limits.allocated_bytes,
                required: bytes,
            }));
        }
        self.poll()
    }
    fn reserve_scratch<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), HtmlSemanticFailure> {
        self.reserve_records(values, additional, HtmlSemanticResource::ScratchRecords)
    }
    fn reserve_nested_output<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), HtmlSemanticFailure> {
        self.reserve_records(
            values,
            additional,
            HtmlSemanticResource::NestedOutputRecords,
        )
    }
    fn release_scratch(&mut self, capacity: usize) {
        // Only capacities previously recorded by reserve_scratch are released.
        // Cleanup is unconditional, including after a sticky terminal stop.
        self.usage.scratch_records -= capacity;
    }
    fn numeric_budget(&mut self, digits: usize) -> Result<usize, HtmlSemanticFailure> {
        // One bounded atomic numeric operation: scan, canonical write, and
        // allocation. Poll immediately before and after, without claiming the
        // parser's 256-unit cancellation quantum for String operations.
        let work = digits
            .checked_add(1)
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| n.checked_add(8))
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)?;
        Ok(self.remaining_allocated_bytes())
    }
    fn remaining_allocated_bytes(&self) -> usize {
        self.limits
            .allocated_bytes
            .saturating_sub(self.usage.allocated_bytes)
    }
    /// Records actual capacity after an admitted allocation, including an
    /// allocator overshoot. Accounting precedes the next cancellation poll.
    fn record_allocated_bytes(&mut self, capacity: usize) -> Result<(), HtmlSemanticFailure> {
        self.usage.allocated_bytes = self
            .usage
            .allocated_bytes
            .checked_add(capacity)
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        if self.usage.allocated_bytes > self.limits.allocated_bytes {
            return Err(self.stop(HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::AllocatedBytes,
                limit: self.limits.allocated_bytes,
                required: self.usage.allocated_bytes,
            }));
        }
        Ok(())
    }
    fn numeric_result(
        &mut self,
        result: Result<IntegerUnitStepResult, IntegerUnitStepError>,
    ) -> Result<HtmlInteger, HtmlSemanticFailure> {
        let capacity = match &result {
            Ok(result) => result.capacity_bytes,
            Err(IntegerUnitStepError::ActualCapacityLimitExceeded { capacity_bytes, .. }) => {
                *capacity_bytes
            }
            _ => 0,
        };
        self.record_allocated_bytes(capacity)?;
        let result = result.map_err(|error| {
            let kind = match error {
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
                IntegerUnitStepError::RequestedCapacityLimitExceeded { required, .. } => {
                    HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::AllocatedBytes,
                        limit: self.limits.allocated_bytes,
                        required: self.usage.allocated_bytes.saturating_add(required),
                    }
                }
                IntegerUnitStepError::ActualCapacityLimitExceeded { .. } => {
                    HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::AllocatedBytes,
                        limit: self.limits.allocated_bytes,
                        required: self.usage.allocated_bytes,
                    }
                }
            };
            self.stop(kind)
        })?;
        self.poll()?;
        Ok(result.value)
    }
    fn integer_count(&mut self, count: usize) -> Result<HtmlInteger, HtmlSemanticFailure> {
        let remaining = self.numeric_budget(usize::BITS as usize)?;
        self.numeric_result(HtmlInteger::try_from_usize(
            count,
            self.limits.integer_digits,
            remaining,
        ))
    }
    fn step_integer(
        &mut self,
        value: &HtmlInteger,
        reversed: bool,
    ) -> Result<HtmlInteger, HtmlSemanticFailure> {
        let remaining = self.numeric_budget(value.magnitude().decimal_digits().len())?;
        self.numeric_result(value.try_unit_step(
            if reversed {
                IntegerUnitStep::Decrement
            } else {
                IntegerUnitStep::Increment
            },
            self.limits.integer_digits,
            remaining,
        ))
    }
    fn parsed_integer(
        &mut self,
        element: &'a HtmlCapturedElement,
        attribute: &str,
    ) -> Result<Option<HtmlInteger>, HtmlSemanticFailure> {
        let Some(raw) = self.attribute(element, attribute)? else {
            return Ok(None);
        };
        self.interpreter
            .admit_attribute_raw(Some(raw.value()))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        let remaining = self.numeric_budget(raw.value().len())?;
        match try_parse_integer(raw.value(), self.limits.integer_digits, remaining) {
            Ok(value) => self.numeric_result(Ok(value)).map(Some),
            Err(IntegerRecoveryError::Syntax(_)) => {
                self.poll()?;
                Ok(None)
            }
            Err(IntegerRecoveryError::Allocation(error)) => {
                self.numeric_result(Err(error)).map(Some)
            }
        }
    }

    fn reversed(&mut self, element: &'a HtmlCapturedElement) -> Result<bool, HtmlSemanticFailure> {
        let raw = self.attribute(element, "reversed")?;
        self.admit_intrinsic(element, raw, 1)?;
        let report = self
            .interpreter
            .boolean_attribute(HtmlBooleanAttribute::Reversed, element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn participation(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        field: HtmlParticipationField,
    ) -> Result<Result<bool, HtmlSemanticMissing>, HtmlSemanticFailure> {
        if let Some(input) = &self.context.participation {
            for fact in &input.facts {
                self.charge(1)?;
                if &fact.subject == subject {
                    self.charge(1)?;
                    let value = match field {
                        HtmlParticipationField::ListItem => fact.computed_list_item,
                        HtmlParticipationField::Rendered => fact.being_rendered,
                        HtmlParticipationField::CssBox => fact.produces_css_box,
                    };
                    if let Some(value) = value {
                        return Ok(Ok(value));
                    }
                    break;
                }
            }
        }
        Ok(Err(match field {
            HtmlParticipationField::ListItem => {
                HtmlSemanticMissing::ComputedListItem(subject.clone())
            }
            HtmlParticipationField::Rendered => HtmlSemanticMissing::BeingRendered(subject.clone()),
            HtmlParticipationField::CssBox => HtmlSemanticMissing::ProducesCssBox(subject.clone()),
        }))
    }
    fn list_owner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        match self.participation(subject, HtmlParticipationField::ListItem)? {
            Ok(false) => return Ok(HtmlSemanticValue::Inapplicable),
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            Ok(true) => {}
        }
        match self.participation(subject, HtmlParticipationField::Rendered)? {
            Ok(false) => return Ok(HtmlSemanticValue::Known(None)),
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            Ok(true) => {}
        }
        let node = self.node(subject)?;
        let first = match self.parent(node)? {
            Ok(parent) => parent,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let mut ancestor = first.clone();
        let mut selected = first;
        while let Some(id) = ancestor {
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && matches!(
                    self.known(element)?,
                    Some(KnownHtmlElement::Ol | KnownHtmlElement::Ul | KnownHtmlElement::Menu)
                )
            {
                selected = Some(id);
                break;
            }
            ancestor = match self.parent(node)? {
                Ok(parent) => parent,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
        }
        while let Some(id) = selected {
            let node = self.node(&id)?;
            if matches!(node.kind(), HtmlCapturedNodeKind::Element(_)) {
                match self.participation(&id, HtmlParticipationField::CssBox)? {
                    Ok(true) => return Ok(HtmlSemanticValue::Known(Some(id))),
                    Ok(false) => {}
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                }
            }
            selected = match self.parent(node)? {
                Ok(parent) => parent,
                Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            };
        }
        Ok(HtmlSemanticValue::Known(None))
    }
    fn owned_items(
        &mut self,
        owner: &HtmlCaptureSubjectId,
        html_li_only: bool,
    ) -> Result<HtmlSemanticValue<Vec<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let mut stack: Vec<HtmlCapturedChildIds<'a>> = Vec::new();
        let mut items = Vec::new();
        let outcome = (|| {
            let node = self.node(owner)?;
            self.charge(1)?;
            match node.children() {
                HtmlCapturedChildren::Complete(children) => {
                    self.reserve_scratch(&mut stack, 1)?;
                    stack.push(children);
                }
                HtmlCapturedChildren::Unavailable => {
                    return Ok(Some(HtmlSemanticMissing::OrderedChildren(owner.clone())));
                }
            }
            while !stack.is_empty() {
                self.charge(1)?;
                let next = stack.last_mut().and_then(Iterator::next);
                let Some(id) = next else {
                    stack.pop();
                    continue;
                };
                let node = self.node(&id)?;
                let mut skip_children = false;
                if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                    let known = self.known(element)?;
                    if !html_li_only || known == Some(KnownHtmlElement::Li) {
                        match self.list_owner(&id)? {
                            HtmlSemanticValue::Known(Some(candidate)) if &candidate == owner => {
                                self.reserve_scratch(&mut items, 1)?;
                                items.push(id.clone());
                            }
                            HtmlSemanticValue::Missing(missing) => {
                                return Ok(Some(missing));
                            }
                            _ => {}
                        }
                    }
                    // A nested HTML list that itself produces a CSS box owns
                    // every descendant candidate selected through that list;
                    // none can fall through to the outer owner. Its unavailable
                    // subtree need not be consulted for this owner's items.
                    if matches!(
                        known,
                        Some(KnownHtmlElement::Ol | KnownHtmlElement::Ul | KnownHtmlElement::Menu)
                    ) {
                        skip_children = matches!(
                            self.participation(&id, HtmlParticipationField::CssBox)?,
                            Ok(true)
                        );
                    }
                }
                if skip_children {
                    continue;
                }
                self.charge(1)?;
                match node.children() {
                    HtmlCapturedChildren::Complete(children) if children.len() > 0 => {
                        self.reserve_scratch(&mut stack, 1)?;
                        stack.push(children);
                    }
                    HtmlCapturedChildren::Complete(_) => {}
                    HtmlCapturedChildren::Unavailable => {
                        return Ok(Some(HtmlSemanticMissing::OrderedChildren(id)));
                    }
                }
            }
            Ok(None)
        })();
        self.release_scratch(stack.capacity());
        match outcome {
            Ok(None) => Ok(HtmlSemanticValue::Known(items)),
            Ok(Some(missing)) => {
                self.release_scratch(items.capacity());
                Ok(HtmlSemanticValue::Missing(missing))
            }
            Err(error) => {
                self.release_scratch(items.capacity());
                Err(error)
            }
        }
    }
    fn count_owned_li(
        &mut self,
        items: &[HtmlCaptureSubjectId],
    ) -> Result<HtmlInteger, HtmlSemanticFailure> {
        let mut count = 0usize;
        for id in items {
            let node = self.node(id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Li)
            {
                count = count
                    .checked_add(1)
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
            }
        }
        self.integer_count(count)
    }
    fn starting_value(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        owned: Option<&[HtmlCaptureSubjectId]>,
    ) -> Result<HtmlSemanticValue<HtmlInteger>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::Ol) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        if let Some(value) = self.parsed_integer(element, "start")? {
            return Ok(HtmlSemanticValue::Known(value));
        }
        if !self.reversed(element)? {
            return self.integer_count(1).map(HtmlSemanticValue::Known);
        }
        if let Some(items) = owned {
            return self.count_owned_li(items).map(HtmlSemanticValue::Known);
        }
        match self.owned_items(subject, true)? {
            HtmlSemanticValue::Known(items) => {
                let result = self.count_owned_li(&items).map(HtmlSemanticValue::Known);
                self.release_scratch(items.capacity());
                result
            }
            HtmlSemanticValue::Missing(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(unsupported) => {
                Ok(HtmlSemanticValue::Unsupported(unsupported))
            }
        }
    }
    fn ordinal(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlInteger>>, HtmlSemanticFailure> {
        let owner = match self.list_owner(subject)? {
            HtmlSemanticValue::Known(Some(owner)) => owner,
            value => return Ok(value.map(|_| None)),
        };
        // Complete ownership coverage is established before any ordinal is
        // published; an unavailable later subtree cannot produce a prefix.
        let items = match self.owned_items(&owner, false)? {
            HtmlSemanticValue::Known(items) => items,
            value => return Ok(value.map(|_| None)),
        };
        let outcome = (|| {
            let node = self.node(&owner)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
            };
            let ol = self.known(element)? == Some(KnownHtmlElement::Ol);
            let mut numbering = if ol {
                match self.starting_value(&owner, element, Some(&items))? {
                    HtmlSemanticValue::Known(value) => value,
                    value => return Ok(value.map(Some)),
                }
            } else {
                self.integer_count(1)?
            };
            let reversed = ol && self.reversed(element)?;
            for id in &items {
                let node = self.node(id)?;
                if let HtmlCapturedNodeKind::Element(element) = node.kind()
                    && self.known(element)? == Some(KnownHtmlElement::Li)
                    && let Some(value) = self.parsed_integer(element, "value")?
                {
                    numbering = value;
                }
                self.charge(1)?;
                if id == subject {
                    return Ok(HtmlSemanticValue::Known(Some(numbering)));
                }
                numbering = self.step_integer(&numbering, reversed)?;
            }
            Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
        })();
        self.release_scratch(items.capacity());
        outcome
    }
    /// Requests are retained in input order, including duplicates. Any terminal
    /// failure discards this whole batch; unavailable semantic evidence is a
    /// complete typed fact, not an interrupted query.
    pub fn query(
        &mut self,
        requests: &[HtmlSemanticRequest],
    ) -> Result<HtmlSemanticReport, HtmlSemanticFailure> {
        self.poll()?;
        let mut results = Vec::new();
        self.reserve_records(
            &mut results,
            requests.len(),
            HtmlSemanticResource::OutputRecords,
        )?;
        for request in requests {
            let node = self.node(&request.subject)?;
            let fact = match node.kind() {
                HtmlCapturedNodeKind::Element(element) => {
                    let foreign = !self.html_namespace(element)?;
                    match request.query {
                        HtmlSemanticQuery::Hidden => {
                            let value = if foreign {
                                HtmlSemanticValue::Unsupported(
                                    HtmlSemanticUnsupported::ForeignNamespace,
                                )
                            } else {
                                let raw = self.attribute(element, "hidden")?;
                                self.admit_intrinsic(element, raw, 2)?;
                                let state =
                                    self.interpreter.hidden(element.name(), raw).map_err(|e| {
                                        self.stop(HtmlSemanticFailureKind::Attribute(e))
                                    })?;
                                HtmlSemanticValue::Known(state.intrinsic().copied().ok_or_else(
                                    || self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable),
                                )?)
                            };
                            HtmlSemanticFact::Hidden(value)
                        }
                        HtmlSemanticQuery::Disabledness => {
                            HtmlSemanticFact::Disabledness(if foreign {
                                HtmlSemanticValue::Unsupported(
                                    HtmlSemanticUnsupported::ForeignNamespace,
                                )
                            } else {
                                self.disabledness(&request.subject, element)?
                            })
                        }
                        HtmlSemanticQuery::ListOwner => {
                            HtmlSemanticFact::ListOwner(self.list_owner(&request.subject)?)
                        }
                        HtmlSemanticQuery::ListStartingValue => {
                            HtmlSemanticFact::ListStartingValue(self.starting_value(
                                &request.subject,
                                element,
                                None,
                            )?)
                        }
                        HtmlSemanticQuery::ListOrdinal => {
                            HtmlSemanticFact::ListOrdinal(self.ordinal(&request.subject)?)
                        }
                        HtmlSemanticQuery::TableBodies => {
                            HtmlSemanticFact::TableBodies(self.table_bodies(&request.subject)?)
                        }
                        HtmlSemanticQuery::TableRows => {
                            HtmlSemanticFact::TableRows(self.table_rows(&request.subject)?)
                        }
                        HtmlSemanticQuery::TableSectionRows => HtmlSemanticFact::TableSectionRows(
                            self.table_section_rows(&request.subject)?,
                        ),
                        HtmlSemanticQuery::TableRowCells => {
                            HtmlSemanticFact::TableRowCells(self.table_row_cells(&request.subject)?)
                        }
                        HtmlSemanticQuery::TableModel => {
                            HtmlSemanticFact::TableModel(self.table_model(&request.subject)?)
                        }
                        HtmlSemanticQuery::Language => {
                            HtmlSemanticFact::Language(self.language(&request.subject)?)
                        }
                        HtmlSemanticQuery::Directionality => {
                            HtmlSemanticFact::Directionality(self.directionality(&request.subject)?)
                        }
                        HtmlSemanticQuery::Meter => {
                            HtmlSemanticFact::Meter(self.meter_state(element)?)
                        }
                        HtmlSemanticQuery::Progress => {
                            HtmlSemanticFact::Progress(self.progress_state(element)?)
                        }
                        HtmlSemanticQuery::EditingHost => {
                            HtmlSemanticFact::EditingHost(self.editing_host(&request.subject)?)
                        }
                        HtmlSemanticQuery::Editable => {
                            HtmlSemanticFact::Editable(self.editable(&request.subject)?)
                        }
                        HtmlSemanticQuery::OwnAutocapitalizationHint => {
                            HtmlSemanticFact::OwnAutocapitalizationHint(
                                self.own_autocapitalization_hint(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::UsedAutocapitalizationHint => {
                            HtmlSemanticFact::UsedAutocapitalizationHint(
                                self.used_autocapitalization_hint(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::UsedAutocorrectionState => {
                            HtmlSemanticFact::UsedAutocorrectionState(
                                self.used_autocorrection_state(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::ComputedWritingSuggestions => {
                            HtmlSemanticFact::ComputedWritingSuggestions(
                                self.computed_writing_suggestions(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::IsContentEditable => {
                            HtmlSemanticFact::IsContentEditable(
                                self.is_content_editable(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::TranslationMode => HtmlSemanticFact::TranslationMode(
                            self.translation_mode(&request.subject)?,
                        ),
                        HtmlSemanticQuery::Spellcheck => {
                            HtmlSemanticFact::Spellcheck(self.spellcheck(&request.subject)?)
                        }
                        HtmlSemanticQuery::ImageMapAttribute => {
                            HtmlSemanticFact::ImageMapAttribute(
                                self.image_map_attribute(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::ImageControlsAttribute => {
                            HtmlSemanticFact::ImageControlsAttribute(
                                self.image_controls_attribute(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::StylesheetDisabledAttribute => {
                            HtmlSemanticFact::StylesheetDisabledAttribute(
                                self.stylesheet_disabled_attribute(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute => {
                            HtmlSemanticFact::HyperlinkReferrerPolicyAttribute(
                                self.hyperlink_referrerpolicy_attribute(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::TimeElement => {
                            HtmlSemanticFact::TimeElement(self.time_element(&request.subject)?)
                        }
                        HtmlSemanticQuery::TextareaDefaultValue => {
                            HtmlSemanticFact::TextareaDefaultValue(self.control_text_readback(
                                &request.subject,
                                HtmlControlTextOrigin::ChildText,
                            )?)
                        }
                        HtmlSemanticQuery::OutputValue => {
                            HtmlSemanticFact::OutputValue(self.control_text_readback(
                                &request.subject,
                                HtmlControlTextOrigin::DescendantText,
                            )?)
                        }
                        HtmlSemanticQuery::OutputDefaultValue => {
                            HtmlSemanticFact::OutputDefaultValue(self.control_text_readback(
                                &request.subject,
                                HtmlControlTextOrigin::OutputDefaultOverride,
                            )?)
                        }
                        HtmlSemanticQuery::InputSuggestionsSource => {
                            HtmlSemanticFact::InputSuggestionsSource(
                                self.input_suggestions_source(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::ConstraintValidationCandidate => {
                            HtmlSemanticFact::ConstraintValidationCandidate(
                                self.constraint_validation_candidate(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::TabIndex => {
                            HtmlSemanticFact::TabIndex(self.tab_index(&request.subject)?)
                        }
                        HtmlSemanticQuery::DetailsSummary => HtmlSemanticFact::DetailsSummary(
                            self.details_summary(&request.subject)?,
                        ),
                        HtmlSemanticQuery::SummaryForParentDetails => {
                            HtmlSemanticFact::SummaryForParentDetails(
                                self.summary_for_parent_details(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::DetailsNameGroup => HtmlSemanticFact::DetailsNameGroup(
                            self.details_name_group(&request.subject)?,
                        ),
                        HtmlSemanticQuery::UsemapReference => HtmlSemanticFact::UsemapReference(
                            self.usemap_reference(&request.subject)?,
                        ),
                        HtmlSemanticQuery::MapAreas => {
                            HtmlSemanticFact::MapAreas(self.map_areas(&request.subject)?)
                        }
                        HtmlSemanticQuery::DatalistOptions => HtmlSemanticFact::DatalistOptions(
                            self.datalist_options(&request.subject, false)?,
                        ),
                        HtmlSemanticQuery::DatalistSuggestionOptions => {
                            HtmlSemanticFact::DatalistSuggestionOptions(
                                self.datalist_options(&request.subject, true)?,
                            )
                        }
                        HtmlSemanticQuery::SelectOptions => {
                            HtmlSemanticFact::SelectOptions(self.select_options(&request.subject)?)
                        }
                        HtmlSemanticQuery::SelectedOptions => HtmlSemanticFact::SelectedOptions(
                            self.selected_options(&request.subject)?,
                        ),
                        HtmlSemanticQuery::SelectSelectedIndex => {
                            HtmlSemanticFact::SelectSelectedIndex(
                                self.select_selected_index(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::SubmitterNoValidate => {
                            HtmlSemanticFact::SubmitterNoValidate(
                                self.submitter_no_validate(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::FormDefaultButton => {
                            HtmlSemanticFact::FormDefaultButton(
                                self.form_default_button(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::FormImplicitSubmissionCondition => {
                            HtmlSemanticFact::FormImplicitSubmissionCondition(
                                self.form_implicit_submission_condition(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::SubmitterFormMethod => {
                            HtmlSemanticFact::SubmitterFormMethod(
                                self.submitter_form_method(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::SubmitterFormEnctype => {
                            HtmlSemanticFact::SubmitterFormEnctype(
                                self.submitter_form_enctype(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::ControlAutocomplete => {
                            HtmlSemanticFact::ControlAutocomplete(
                                self.control_autocomplete(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::OptionIndex => {
                            HtmlSemanticFact::OptionIndex(self.option_index(&request.subject)?)
                        }
                        HtmlSemanticQuery::OptionFormOwner => HtmlSemanticFact::OptionFormOwner(
                            self.option_form_owner(&request.subject)?,
                        ),
                        HtmlSemanticQuery::OptionText => {
                            HtmlSemanticFact::OptionText(self.option_readback(
                                &request.subject,
                                HtmlOptionReadbackOrigin::DescendantText,
                            )?)
                        }
                        HtmlSemanticQuery::OptionLabel => {
                            HtmlSemanticFact::OptionLabel(self.option_readback(
                                &request.subject,
                                HtmlOptionReadbackOrigin::LabelAttribute,
                            )?)
                        }
                        HtmlSemanticQuery::OptionValue => {
                            HtmlSemanticFact::OptionValue(self.option_readback(
                                &request.subject,
                                HtmlOptionReadbackOrigin::ValueAttribute,
                            )?)
                        }
                        HtmlSemanticQuery::SelectValueMissing => {
                            HtmlSemanticFact::SelectValueMissing(
                                self.select_value_missing(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::SelectPlaceholderLabelOption => {
                            HtmlSemanticFact::SelectPlaceholderLabelOption(
                                self.select_placeholder_label_option(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::SelectValue => {
                            HtmlSemanticFact::SelectValue(self.select_value(&request.subject)?)
                        }
                        HtmlSemanticQuery::ComputedHeadingLevel => {
                            HtmlSemanticFact::ComputedHeadingLevel(
                                self.computed_heading_level(&request.subject)?,
                            )
                        }
                        HtmlSemanticQuery::Inertness => {
                            HtmlSemanticFact::Inertness(self.inertness(&request.subject)?)
                        }
                        HtmlSemanticQuery::FormOwner => {
                            HtmlSemanticFact::FormOwner(self.form_owner(&request.subject)?)
                        }
                        HtmlSemanticQuery::LabelFormOwner => HtmlSemanticFact::LabelFormOwner(
                            self.label_form_owner(&request.subject)?,
                        ),
                        HtmlSemanticQuery::LabeledControl => HtmlSemanticFact::LabeledControl(
                            self.labeled_control(&request.subject)?,
                        ),
                        HtmlSemanticQuery::ControlLabels => {
                            HtmlSemanticFact::ControlLabels(self.control_labels(&request.subject)?)
                        }
                        HtmlSemanticQuery::FormControls => {
                            HtmlSemanticFact::FormControls(self.form_controls(&request.subject)?)
                        }
                        HtmlSemanticQuery::FormLength => {
                            HtmlSemanticFact::FormLength(self.form_length(&request.subject)?)
                        }
                        HtmlSemanticQuery::FieldsetControls => HtmlSemanticFact::FieldsetControls(
                            self.fieldset_controls(&request.subject)?,
                        ),
                        HtmlSemanticQuery::CurrentCheckedness => {
                            HtmlSemanticFact::CurrentCheckedness(
                                self.control_flag(&request.subject, HtmlControlFlag::Checkedness)?,
                            )
                        }
                        HtmlSemanticQuery::CurrentSelectedness => {
                            HtmlSemanticFact::CurrentSelectedness(
                                self.control_flag(&request.subject, HtmlControlFlag::Selectedness)?,
                            )
                        }
                        HtmlSemanticQuery::DirtyValueFlag => HtmlSemanticFact::DirtyValueFlag(
                            self.control_flag(&request.subject, HtmlControlFlag::DirtyValue)?,
                        ),
                        HtmlSemanticQuery::DirtyCheckednessFlag => {
                            HtmlSemanticFact::DirtyCheckednessFlag(self.control_flag(
                                &request.subject,
                                HtmlControlFlag::DirtyCheckedness,
                            )?)
                        }
                        HtmlSemanticQuery::OptionDirtiness => HtmlSemanticFact::OptionDirtiness(
                            self.control_flag(&request.subject, HtmlControlFlag::OptionDirtiness)?,
                        ),
                        HtmlSemanticQuery::Indeterminateness => {
                            HtmlSemanticFact::Indeterminateness(self.control_flag(
                                &request.subject,
                                HtmlControlFlag::Indeterminateness,
                            )?)
                        }
                    }
                }
                _ => match request.query {
                    HtmlSemanticQuery::Hidden => {
                        HtmlSemanticFact::Hidden(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Disabledness => {
                        HtmlSemanticFact::Disabledness(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ListOwner => {
                        HtmlSemanticFact::ListOwner(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ListStartingValue => {
                        HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ListOrdinal => {
                        HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TableBodies => {
                        HtmlSemanticFact::TableBodies(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TableRows => {
                        HtmlSemanticFact::TableRows(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TableSectionRows => {
                        HtmlSemanticFact::TableSectionRows(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TableRowCells => {
                        HtmlSemanticFact::TableRowCells(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TableModel => {
                        HtmlSemanticFact::TableModel(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Language => {
                        HtmlSemanticFact::Language(self.language(&request.subject)?)
                    }
                    HtmlSemanticQuery::Directionality => {
                        HtmlSemanticFact::Directionality(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Meter => {
                        HtmlSemanticFact::Meter(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Progress => {
                        HtmlSemanticFact::Progress(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::EditingHost => {
                        HtmlSemanticFact::EditingHost(self.editing_host(&request.subject)?)
                    }
                    HtmlSemanticQuery::Editable => {
                        HtmlSemanticFact::Editable(self.editable(&request.subject)?)
                    }
                    HtmlSemanticQuery::OwnAutocapitalizationHint => {
                        HtmlSemanticFact::OwnAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::UsedAutocapitalizationHint => {
                        HtmlSemanticFact::UsedAutocapitalizationHint(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::UsedAutocorrectionState => {
                        HtmlSemanticFact::UsedAutocorrectionState(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ComputedWritingSuggestions => {
                        HtmlSemanticFact::ComputedWritingSuggestions(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::IsContentEditable => {
                        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TranslationMode => {
                        HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Spellcheck => {
                        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ImageMapAttribute => {
                        HtmlSemanticFact::ImageMapAttribute(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ImageControlsAttribute => {
                        HtmlSemanticFact::ImageControlsAttribute(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::StylesheetDisabledAttribute => {
                        HtmlSemanticFact::StylesheetDisabledAttribute(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute => {
                        HtmlSemanticFact::HyperlinkReferrerPolicyAttribute(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::TimeElement => {
                        HtmlSemanticFact::TimeElement(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::TextareaDefaultValue => {
                        HtmlSemanticFact::TextareaDefaultValue(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OutputValue => {
                        HtmlSemanticFact::OutputValue(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OutputDefaultValue => {
                        HtmlSemanticFact::OutputDefaultValue(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::InputSuggestionsSource => {
                        HtmlSemanticFact::InputSuggestionsSource(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ConstraintValidationCandidate => {
                        HtmlSemanticFact::ConstraintValidationCandidate(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::TabIndex => {
                        HtmlSemanticFact::TabIndex(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::DetailsSummary => {
                        HtmlSemanticFact::DetailsSummary(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SummaryForParentDetails => {
                        HtmlSemanticFact::SummaryForParentDetails(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::DetailsNameGroup => {
                        HtmlSemanticFact::DetailsNameGroup(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::UsemapReference => {
                        HtmlSemanticFact::UsemapReference(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::MapAreas => {
                        HtmlSemanticFact::MapAreas(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::DatalistOptions => {
                        HtmlSemanticFact::DatalistOptions(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::DatalistSuggestionOptions => {
                        HtmlSemanticFact::DatalistSuggestionOptions(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SelectOptions => {
                        HtmlSemanticFact::SelectOptions(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SelectedOptions => {
                        HtmlSemanticFact::SelectedOptions(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SelectSelectedIndex => {
                        HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SubmitterNoValidate => {
                        HtmlSemanticFact::SubmitterNoValidate(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::FormDefaultButton => {
                        HtmlSemanticFact::FormDefaultButton(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::FormImplicitSubmissionCondition => {
                        HtmlSemanticFact::FormImplicitSubmissionCondition(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::SubmitterFormMethod => {
                        HtmlSemanticFact::SubmitterFormMethod(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SubmitterFormEnctype => {
                        HtmlSemanticFact::SubmitterFormEnctype(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ControlAutocomplete => {
                        HtmlSemanticFact::ControlAutocomplete(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OptionIndex => {
                        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OptionFormOwner => {
                        HtmlSemanticFact::OptionFormOwner(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OptionText => {
                        HtmlSemanticFact::OptionText(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OptionLabel => {
                        HtmlSemanticFact::OptionLabel(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OptionValue => {
                        HtmlSemanticFact::OptionValue(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SelectValueMissing => {
                        HtmlSemanticFact::SelectValueMissing(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::SelectPlaceholderLabelOption => {
                        HtmlSemanticFact::SelectPlaceholderLabelOption(
                            HtmlSemanticValue::Inapplicable,
                        )
                    }
                    HtmlSemanticQuery::SelectValue => {
                        HtmlSemanticFact::SelectValue(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ComputedHeadingLevel => {
                        HtmlSemanticFact::ComputedHeadingLevel(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Inertness => {
                        HtmlSemanticFact::Inertness(self.inertness(&request.subject)?)
                    }
                    HtmlSemanticQuery::FormOwner => {
                        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::LabelFormOwner => {
                        HtmlSemanticFact::LabelFormOwner(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::LabeledControl => {
                        HtmlSemanticFact::LabeledControl(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::ControlLabels => {
                        HtmlSemanticFact::ControlLabels(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::FormControls => {
                        HtmlSemanticFact::FormControls(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::FormLength => {
                        HtmlSemanticFact::FormLength(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::FieldsetControls => {
                        HtmlSemanticFact::FieldsetControls(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::CurrentCheckedness => {
                        HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::CurrentSelectedness => {
                        HtmlSemanticFact::CurrentSelectedness(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::DirtyValueFlag => {
                        HtmlSemanticFact::DirtyValueFlag(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::DirtyCheckednessFlag => {
                        HtmlSemanticFact::DirtyCheckednessFlag(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::OptionDirtiness => {
                        HtmlSemanticFact::OptionDirtiness(HtmlSemanticValue::Inapplicable)
                    }
                    HtmlSemanticQuery::Indeterminateness => {
                        HtmlSemanticFact::Indeterminateness(HtmlSemanticValue::Inapplicable)
                    }
                },
            };
            self.charge(1)?;
            results.push(HtmlSemanticResult {
                request: request.clone(),
                fact,
            });
        }
        self.poll()?;
        Ok(HtmlSemanticReport {
            qualification: self.qualification.clone(),
            usage: self.usage,
            results,
        })
    }
}
