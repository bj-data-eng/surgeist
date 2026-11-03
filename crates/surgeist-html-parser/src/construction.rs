//! Real construction records with controlled tokenizer payload transfer.
//!
//! HTML start-tag names and no-namespace attributes retain exact immutable
//! UTF-16 backing, actual vector/shared tickets, and original token text origins.
//! Character insertion merges preceding native text at an explicitly selected
//! location. Retired merge backing remains owned and fully charged until cleanup.
//! Textarea raw backing is independent of native child text; replacement disposes
//! its old backing while retaining initialized retired slots until cleanup.
//! Prologue producers retain comments, recovered PI and doctype fields in the
//! same arena, with controlled implied-html backing and exact whitespace splits.
//! Preexisting immutable element inputs retain their earlier physical exclusion.
//! Managed MathML/SVG names and qualified attributes retain lexical originals
//! and independent adjusted backing/maps; qualified clones use the same registry.
//! Actual node-document bindings, lazy inert template documents, template patch
//! facts and whole-grammar PI maps share this registry. Other resettable form
//! metadata, source reports and public parsing remain later producers. Qualified
//! hosted context, local shadow records and captured attachment effects stay in
//! this owner; they do not implement a live DOM or public parse entry.
//! Input initialization installs source-order attributes, retaining original
//! token/value-attribute ranges and lexical maps for actual conditional writeback.
//! Script facts are retained for a sealed fresh ordinary Disabled execution;
//! preparation ends at step18 and never fabricates a loaded or executed result.

use crate::bounded::{HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationError};
use crate::keyed::{Table, slots};
#[cfg(test)]
use crate::parser::control::WalkSite;
use crate::parser::control::{
    ControlIdentity, DocumentStringAllocation, FinalReleaseError, OwnedVec, ParseControl,
    ParseStop, SharedAllocation, StandaloneUnsupported, VectorCapacity, WorkCategory,
};
use crate::parser::{
    buffer::{FrozenUtf16, Utf16Buffer},
    tokenizer::token::{Doctype, Tag, TagKind, Text, TextOrigin, TokenKind},
};
use crate::{Attribute, HtmlParseResource, HtmlTextareaFlags, HtmlTextareaRawValueAction};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::{mem, sync::Weak};
use surgeist_html::bounded;
use surgeist_html::bounded::HtmlScriptPreparationResult as DisabledScriptPreparation;
use surgeist_html::bounded::{
    HtmlFormOwnerResetDecision, HtmlFormOwnerResetTrigger, form_owner_reset_trigger,
};
use surgeist_html::*;

mod activity;
mod attribute_writer;
mod construction_owner;
mod dialog;
use crate::bounded::HtmlConstructionAttributeEditMode;
use construction_owner::{ConstructionOwner, ConstructionView, prepare_attribute_slot};
pub(crate) mod associated;
mod base;
mod clone;
mod csp;
mod css_color;
mod input_color;
mod input_image;
pub(crate) mod meta;
pub(crate) mod pending_association;
pub(crate) mod style;
#[cfg(test)]
use input_image::append_native_input_value;
mod completion;
pub(crate) mod context;
mod details;
mod effects;
#[cfg(test)]
pub(crate) use effects::{fault_effect_drain, take_effect_fault};
pub(crate) mod fragment_document;
#[cfg(test)]
mod link_history;
#[cfg(test)]
pub(crate) use context::ParserDocumentReference;
use context::{DocumentRef, FragmentContextStorage};
pub(crate) mod evidence;
use evidence::ParseEvidence;
mod foreign;
pub(crate) mod history;
#[cfg(test)]
pub(crate) mod native_view;
mod pi;
mod radio;
mod range;
#[cfg(test)]
pub(crate) use radio::InputGroupAttribute;
mod mutation;
mod nonce;
pub(crate) mod provenance;
pub(crate) mod read;
pub(crate) mod report;
mod value_sharing;
use provenance::{FieldCopy, OriginGroupRef};
#[cfg(test)]
mod no_panic_tests;
#[cfg(test)]
mod prepared_slot_tests;
#[cfg(test)]
mod read_fault_tests;
mod select;
mod svg;
mod template;
mod text_append;
#[cfg(test)]
pub(crate) mod text_append_tests;
mod tree_root;
pub use completion::{ConstructionCompletion, ConstructionReadiness, StandaloneDocumentState};
use effects::ConstructionEffects;
pub(crate) use foreign::ForeignNamespace;
#[cfg(test)]
pub(crate) use native_view::NativeView;
use select::SelectMetadata;
pub use svg::SvgScriptOrigin;
pub(crate) use svg::SvgScriptState;
pub(crate) use template::{CreationContext, CreationLocation};
#[cfg(test)]
pub(crate) use template::{ParserDocument, TemplateState};

mod cleanup;
mod formatting_creation;
mod input_values;
mod metadata_rows;
mod ordinary_view;
mod script_type;
#[cfg(test)]
mod test_support;
mod text_runs;
mod token_payload;
use cleanup::{
    cleanup_charge, cleanup_owner, keep_first, payload_write, release_logical, release_parts,
    release_vector, take_payload,
};
pub(crate) use formatting_creation::FormattingCreation;
use formatting_creation::{check_formatting_tag, copy_formatting_text};
#[cfg(test)]
use formatting_creation::{
    check_formatting_text, copy_formatting_tag, discard_formatting_tag, release_formatting_tag,
};
pub(crate) use input_values::input_tag_state;
#[cfg(test)]
use input_values::{append_input_content, find_input_sources, find_input_state};
use input_values::{
    copy_input_value, copy_preserved_input_value, copy_selected_input_value, input_type_state,
    validate_initial_input_origins,
};
use metadata_rows::{
    admitted_input_string, find_form_association_state, find_textarea_state, script_metadata,
    set_form_association_state, set_script_metadata, textarea_raw,
};
#[cfg(test)]
use ordinary_view::{append_descendant_text, find_output_state};
use ordinary_view::{
    next_ordinary_view_after_subtree, next_ordinary_view_node, ordinary_view_parent,
};
#[cfg(test)]
use script_type::script_type_charge;
use script_type::{classify_script, script_attribute};
#[cfg(test)]
pub(crate) use test_support::csp_protocol_error;
#[cfg(test)]
use test_support::{inject_walk_refusal, name_units, test_owning_depth};
#[cfg(test)]
use text_runs::discard_runs;
pub(crate) use text_runs::{
    BodyTextKind, BodyTextRuns, TextRun, partition_body_text, split_first_lf,
    split_leading_ascii_whitespace,
};
use token_payload::{
    check_disposal_text, check_text_source, exact_string_equal, literal_string,
    release_token_attributes,
};
pub(crate) use token_payload::{release_token_kind, validate_token_kind};
#[derive(Clone, Debug)]
pub(crate) struct ParserNode {
    handle: HtmlBuilderNode,
    // Immutable dense parser-metadata join; only the checked handle authorizes a node.
    index: usize,
}

/// Creation/mutation initializes from installed attributes. A Color clone
/// instead preserves its qualified original current value at clone completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputAttributeInitialization {
    Installed,
    ClonedColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionError {
    NonceMetadataUnavailable,
    CspService,
    CspProtocol(surgeist_html::HtmlCspProtocolError),
    Stop(ParseStop),
    ForeignNode,
    RootMove,
    AssociationMove,
    NotContainer,
    TemplateContentsRequired,
    WrongSibling,
    Cycle,
    Consumed,
    WrongToken,
    DuplicateAttribute,
    AliasedPayload,
    TemplateInsertionStateUnavailable,
    NotDocument,
    OriginSplitUnavailable,
    AttributeStorageUnavailable,
    ScriptMetadataUnavailable,
    SvgScriptMetadataUnavailable,
    ImageFormMetadataUnavailable,
    AssociatedFacts,
    FormAssociationMetadataUnavailable,
    TextareaMetadataUnavailable,
    OutputMetadataUnavailable,
    InputMetadataUnavailable,
    SelectMetadataUnavailable,
    NodeDocumentUnavailable,
    TemplateMetadataUnavailable,
    ProcessingInstructionMapUnavailable,
    PendingIntrinsicWork,
    CompletionMetadataUnavailable,
    CompletionIncomplete,
    ContextUnavailable,
    ContextReactionUnavailable,
    ContextFactUnavailable(crate::HtmlCaptureFact),
    CapturedDocument,
}
/// Pinned element-placement guards; generic storage moves remain independent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementInsertionIgnored {
    ParentAlreadyPresent,
    HostIncludingInclusiveAncestor,
    DocumentAlreadyHasElement,
}
impl SessionError {
    /// An impossible registry state with no controller in reach.
    pub(crate) fn registry_row() -> Self {
        Self::Stop(ParseStop::InternalInvariant(
            crate::parser::control::Invariant::RegistryRow,
        ))
    }
}
/// HTML's construction owner answered a step with an action or observation
/// that the parser's sequencing does not expect, latched as the controller's
/// first stop.
fn latch_owner_refusal(control: &mut ParseControl) -> SessionError {
    SessionError::Stop(
        control.fail_invariant(crate::parser::control::Invariant::ConstructionOwnerRefusal),
    )
}
/// A work plan that counts less than was already charged for it: no row is
/// missing, the plan and the charge disagree.
fn plan_below_charge(control: &mut ParseControl) -> SessionError {
    SessionError::Stop(control.fail_invariant(crate::parser::control::Invariant::CapacityMismatch))
}
/// An impossible registry state, latched as the controller's first stop.
fn latch_registry_row(control: &mut ParseControl) -> SessionError {
    SessionError::Stop(control.fail_invariant(crate::parser::control::Invariant::RegistryRow))
}
/// How a parse ends on a session error that no earlier stop has settled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionClass {
    /// The error is an outcome of the document, of the facts the caller
    /// supplied or of a service. The parse ends as this stop, whichever driver
    /// and stage met the error, and the failure's reason names the same state.
    Outcome(ParseStop),
    /// The error names a fault of the parser's own, met where it cannot be a
    /// document outcome. The parse ends as the invariant of its kind.
    Fault(crate::parser::control::Invariant),
}
impl SessionClass {
    /// An outcome that ends the parse as an unsupported state of the closed
    /// standalone environment.
    pub(crate) const fn unsupported(state: StandaloneUnsupported) -> Self {
        Self::Outcome(ParseStop::UnsupportedStandaloneState(state))
    }
    /// Latches the stop for this class on `control`, unless an earlier stop is
    /// already the first. Returns the first stop either way.
    pub(crate) fn latch(self, control: &mut ParseControl) -> ParseStop {
        match self {
            Self::Outcome(stop) => control.stop_outcome(stop),
            Self::Fault(invariant) => control.fail_invariant(invariant),
        }
    }
}
/// The class of a session error, by the kind of state it reports. Every
/// variant is listed, so a new one must be classified deliberately.
///
/// Outcomes, with the stop each ends a parse as:
/// - A stop is already latched by whoever raised it, and is its own.
/// - The Meta CSP service and protocol outcomes are the stops their processing
///   latches when it raises them. A processor that breaks the accounting of the
///   service it was given is a failed processing.
/// - Invalid associated facts are the associated-facts stop, as the creation
///   and Meta CSP producers latch it.
/// - Script and SVG script metadata that effect work cannot find is the
///   unsupported script document; completion metadata that is absent or
///   incomplete is the unsupported completion. The standalone environment does
///   not run scripts or document completion.
/// - A fact or reaction the caller's capture lacks is the provenance boundary:
///   the capture is the only source of the fact, so the parse cannot continue
///   from provenance it was not given.
///
/// Faults, with the invariant of their kind:
/// - An intrinsic operation that is still owed is the pending-effects stop.
/// - A uniqueness check that failed is the aliased-allocation stop.
/// - A placement the tree refuses, or a duplicated attribute, is the
///   construction owner declining a request the parser's sequencing
///   guarantees valid.
/// - A retained row that is missing, of another kind, unjoined or out of
///   place is a registry row. This is the whole of the rest of this side: every
///   remaining variant reports a lookup or a bookkeeping check that fails on a
///   handle, index or count the parser created itself (an arena node, a
///   payload row of the wrong kind, storage already taken, a reference count
///   other than one, a parent walk past its bound, a placement or association
///   the construction owner refuses to a request the parser's sequencing
///   guarantees valid). None of them reads what the caller supplied. Whatever
///   the caller's capture does not establish is
///   `ContextFactUnavailable(fact)`, a processor that breaks the accounting of
///   the service it was given is `CspService`, and the facts a request
///   supplies are `AssociatedFacts`. A raise site that would read caller
///   input must not return a fault variant.
pub(crate) fn session_class(error: &SessionError) -> SessionClass {
    use crate::parser::control::Invariant;
    match error {
        SessionError::Stop(stop) => SessionClass::Outcome(*stop),
        SessionError::CspService => SessionClass::Outcome(ParseStop::CspProcessing),
        SessionError::CspProtocol(protocol) => {
            SessionClass::Outcome(ParseStop::CspProtocol(*protocol))
        }
        SessionError::AssociatedFacts => SessionClass::Outcome(ParseStop::InvalidAssociatedFacts),
        SessionError::ScriptMetadataUnavailable | SessionError::SvgScriptMetadataUnavailable => {
            SessionClass::unsupported(StandaloneUnsupported::ScriptDocument)
        }
        SessionError::CompletionMetadataUnavailable | SessionError::CompletionIncomplete => {
            SessionClass::unsupported(StandaloneUnsupported::Completion)
        }
        SessionError::ContextFactUnavailable(_) | SessionError::ContextReactionUnavailable => {
            SessionClass::unsupported(StandaloneUnsupported::Provenance)
        }
        SessionError::PendingIntrinsicWork => SessionClass::Fault(Invariant::PendingFormEffects),
        SessionError::AliasedPayload => SessionClass::Fault(Invariant::SharedAllocationAliased),
        SessionError::RootMove
        | SessionError::AssociationMove
        | SessionError::NotContainer
        | SessionError::TemplateContentsRequired
        | SessionError::WrongSibling
        | SessionError::Cycle
        | SessionError::DuplicateAttribute => {
            SessionClass::Fault(Invariant::ConstructionOwnerRefusal)
        }
        SessionError::NonceMetadataUnavailable
        | SessionError::ForeignNode
        | SessionError::Consumed
        | SessionError::WrongToken
        | SessionError::TemplateInsertionStateUnavailable
        | SessionError::NotDocument
        | SessionError::OriginSplitUnavailable
        | SessionError::AttributeStorageUnavailable
        | SessionError::ImageFormMetadataUnavailable
        | SessionError::FormAssociationMetadataUnavailable
        | SessionError::TextareaMetadataUnavailable
        | SessionError::OutputMetadataUnavailable
        | SessionError::InputMetadataUnavailable
        | SessionError::SelectMetadataUnavailable
        | SessionError::NodeDocumentUnavailable
        | SessionError::TemplateMetadataUnavailable
        | SessionError::ProcessingInstructionMapUnavailable
        | SessionError::ContextUnavailable
        | SessionError::CapturedDocument => SessionClass::Fault(Invariant::RegistryRow),
    }
}
/// The invariant that a non-stop error names when it is met where it cannot be
/// a document outcome; an outcome has none, and the caller passes it through
/// unchanged.
fn session_fault_invariant(error: &SessionError) -> Option<crate::parser::control::Invariant> {
    match session_class(error) {
        SessionClass::Fault(invariant) => Some(invariant),
        SessionClass::Outcome(_) => None,
    }
}
/// Ends the session on `error` where it is met: latches the stop its class
/// names now. If that stop is the first one, an outcome is returned as itself,
/// which keeps its own reason, and a fault as the invariant it latched. If
/// another stop was already first, the first stop is returned for either, so
/// the error's reason and the failure's first stop cannot disagree.
fn end_session_on(control: &mut ParseControl, error: SessionError) -> SessionError {
    let class = session_class(&error);
    let first = class.latch(control);
    match class {
        SessionClass::Outcome(own) if own == first => error,
        SessionClass::Outcome(_) | SessionClass::Fault(_) => SessionError::Stop(first),
    }
}
/// The result of a non-stop error that effect work met, classified by
/// [`session_fault_invariant`]. An outcome the driver classifies itself passes
/// through unchanged and latches nothing.
fn latch_effect_fault(control: &mut ParseControl, error: SessionError) -> SessionError {
    match session_fault_invariant(&error) {
        Some(invariant) => SessionError::Stop(control.fail_invariant(invariant)),
        None => error,
    }
}
/// What a non-stop error that ended a creation after its association metadata
/// was registered becomes. A fault of the creation's own is a stop labelled by
/// what it reports, as in effect work.
///
/// An outcome the driver classifies itself passes through unchanged, as the
/// unavailable image-support fact does: the creation runs image update,
/// associated-fact (`start_img_update` and `set_hyperlink`), nonce and
/// context-fact work, whose outcomes come from the facts the caller supplied
/// and so must keep the classification the driver gives them.
///
/// The outcomes of script, SVG-wrapper and completion work are different. The
/// callees of the creation (`allocate_ordered_payload`,
/// `allocate_html_payload_prepared`, `mark_document_parser_style` and
/// `normalize_radio`, and what they call for attribute, input, select,
/// details, nonce, meta, dialog, style and associated-state set-up) were read
/// for calls into script preparation, SVG-wrapper or completion code, and
/// none was found; the form-effect drain they reach only steps the id-change
/// stage of the creation's own element. Such an outcome met here is a creation
/// that returned something that belongs to another stage, which is what the
/// creation label names.
fn latch_creation_fault(control: &mut ParseControl, error: SessionError) -> SessionError {
    use crate::parser::control::Invariant;
    if let Some(invariant) = session_fault_invariant(&error) {
        return SessionError::Stop(control.fail_invariant(invariant));
    }
    match error {
        SessionError::ScriptMetadataUnavailable
        | SessionError::SvgScriptMetadataUnavailable
        | SessionError::CompletionMetadataUnavailable
        | SessionError::CompletionIncomplete => {
            SessionError::Stop(control.fail_invariant(Invariant::IncompleteFormAssociationCreation))
        }
        other => other,
    }
}
/// The position after `cursor`. Every caller reaches it only after the work
/// list yielded an entry at `cursor`, so `cursor` is below an in-memory length
/// (at most `isize::MAX`) and the successor cannot overflow.
fn next_cursor(cursor: usize) -> usize {
    cursor.saturating_add(1)
}
/// The successor of a record count or position, or the registry-row stop when
/// it would overflow.
fn successor(control: &mut ParseControl, value: usize) -> Result<usize, SessionError> {
    value
        .checked_add(1)
        .ok_or_else(|| latch_registry_row(control))
}
/// How many of the flags are set; at most `N`.
fn count_set<const N: usize>(flags: [bool; N]) -> usize {
    flags.into_iter().filter(|flag| *flag).count()
}
impl PayloadStorage {
    /// The registry row at `slot`. A slot past the registry is an impossible
    /// registry state, latched as the controller's first stop.
    fn row(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<&RetainedPayload, SessionError> {
        self.values
            .get(slot)
            .ok_or_else(|| latch_registry_row(control))
    }
    /// As [`Self::row`], for editing.
    fn row_mut(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<&mut RetainedPayload, SessionError> {
        match self.values.get_mut(slot) {
            Some(row) => Ok(row),
            None => Err(latch_registry_row(control)),
        }
    }
}
/// The element record at `node`. A missing position or a record of another
/// kind is an impossible registry state, latched as the controller's first stop.
fn element_record<'a>(
    construction: &'a ConstructionOwner,
    control: &mut ParseControl,
    node: usize,
) -> Result<&'a HtmlElement, SessionError> {
    match construction.node_at(node).map(|record| record.kind()) {
        Some(HtmlNodeKind::Element(element)) => Ok(element),
        _ => Err(latch_registry_row(control)),
    }
}
/// The attribute at `position` of the element record at `node`, with the same
/// registry-state stop as [`element_record`].
fn element_attribute<'a>(
    construction: &'a ConstructionOwner,
    control: &mut ParseControl,
    node: usize,
    position: usize,
) -> Result<&'a Attribute, SessionError> {
    match element_record(construction, control, node)?
        .attributes()
        .as_slice()
        .get(position)
    {
        Some(attribute) => Ok(attribute),
        None => Err(latch_registry_row(control)),
    }
}
impl From<ParseStop> for SessionError {
    fn from(value: ParseStop) -> Self {
        Self::Stop(value)
    }
}

/// Admits the shared value owner's next real comparison through the original
/// parser controller. Refusal returns the same allocation for existing cleanup.
fn creation_command(kind: HtmlNodeKind) -> bounded::HtmlConstructionCommand {
    match kind {
        HtmlNodeKind::Document(mode) => bounded::HtmlConstructionCommand::CreateDocument { mode },
        HtmlNodeKind::Fragment => bounded::HtmlConstructionCommand::CreateFragment,
        kind => bounded::HtmlConstructionCommand::Create { kind },
    }
}

fn validate_parser_attributes(
    backing: Arc<Vec<Attribute>>,
    control: &mut ParseControl,
) -> Result<OrderedAttributes, (SessionError, Arc<Vec<Attribute>>)> {
    // The validation owner holds the only other count on `backing`; keeping one
    // here hands the allocation back on every refusal without recovering it
    // from the owner. On success the spare is simply dropped when this
    // function exits.
    let spare = Arc::clone(&backing);
    let mut validation = Some(HtmlAttributeStorage::from_shared_attributes(backing).validate());
    let validation_error = |error| match error {
        HtmlAttributeValidationError::Duplicate { .. } => SessionError::DuplicateAttribute,
        _ => SessionError::AttributeStorageUnavailable,
    };
    let result = (|| {
        while let Some(action) = validation
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))?
            .next_action()
            .map_err(validation_error)?
        {
            let layout = action.layout();
            if layout.record_visits() != 0 {
                control.charge(WorkCategory::RecordVisit, layout.record_visits())?;
            }
            if layout.inspected_bytes() != 0 {
                control.charge(WorkCategory::InspectByte, layout.inspected_bytes())?;
            }
            action.advance().map_err(validation_error)?;
        }
        HtmlAttributeValidation::try_finish(&mut validation)
            .map(|validated| validated.into_attributes())
            .map_err(validation_error)
    })();
    match result {
        Ok(attributes) => Ok(attributes),
        Err(error) => {
            drop(validation);
            Err((error, spare))
        }
    }
}

pub(crate) enum RootKind {
    Document(HtmlDocumentMode),
    #[cfg(test)]
    Fragment,
}
enum ImpliedHtmlName {
    Html,
    Head,
    Body,
    P,
    Colgroup,
    Tbody,
    Tr,
}
#[derive(Clone, Copy)]
enum AttributeMergeName {
    Html,
    Body,
}
#[derive(Clone, Copy)]
enum CreationProfile<'a, 'source> {
    Ordinary,
    Details(
        &'a crate::parser::tree_builder::ConstructionCapability<'source>,
        &'a crate::HtmlSourceRange,
    ),
    Input(
        &'a crate::parser::tree_builder::ConstructionCapability<'source>,
        &'a crate::HtmlSourceRange,
    ),
    Select(
        &'a crate::parser::tree_builder::ConstructionCapability<'source>,
        &'a crate::HtmlSourceRange,
    ),
}
#[derive(Clone, Copy)]
enum OrderedCreation {
    Details {
        metadata: usize,
    },
    Input {
        input: usize,
        association: usize,
        source: usize,
    },
    Select {
        metadata: Option<usize>,
    },
}
impl AttributeMergeName {
    fn local(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Body => "body",
        }
    }
}

struct PreparedAttributeMerge {
    target: usize,
    container: usize,
    stage: usize,
    start: usize,
    count: usize,
    /// Rows registered before the stage row and bound to it; the stagers that
    /// register their strings afterwards leave this empty.
    earlier: [Option<usize>; 4],
}

pub(crate) struct ParserArenaSession {
    storage: Option<SessionStorage>,
}

struct SessionStorage {
    csp_slot: Option<usize>,
    #[cfg(test)]
    cancel_after_details_shift: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    cancel_after_radio_peer: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    cancel_after_text_append_chunk: Option<crate::HtmlParseCancellation>,
    control: ControlIdentity,
    effects: ConstructionEffects,
    evidence: Option<ParseEvidence>,
    fragment: Option<FragmentContextStorage>,
    #[cfg(test)]
    cancel_after_removal: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    cancel_after_textarea_reset: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    cancel_after_output_reset: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    cancel_after_input_reset: Option<crate::HtmlParseCancellation>,
    /// Takes the form-association row the next creation has registered and
    /// not yet bound out of the registry, as a registry that lost the row
    /// would, and keeps it with its slot for the test to put back.
    #[cfg(test)]
    lose_association_row: Option<Option<(usize, RetainedPayload)>>,
    #[cfg(test)]
    cancel_after_range_append: Option<(
        Option<surgeist_html::bounded::RangeScratchRole>,
        crate::HtmlParseCancellation,
    )>,
    #[cfg(test)]
    cancel_after_range_ingress: Option<range::RangeIngressCancellation>,
    standalone_ineligibility: Option<StandaloneUnsupported>,
    construction: ConstructionOwner,
    payloads: PayloadStorage,
    failed_text_append: Option<text_append::FailedTextAppend>,
    /// The registry row listing the document tree's style elements while every
    /// change to that list since it was built has been applied to it.
    style_order: Option<pending_association::CurrentOrder>,
    /// The subtree whose insertion or removal last walked each color selection;
    /// see [`meta::MetaBatch`].
    meta_batches: meta::MetaDomainTable<Option<meta::MetaBatch>>,
}

use crate::bounded::HtmlConstructionResult;
pub(crate) struct FrozenArena {
    storage: Option<FrozenStorage>,
    finalizing: Option<value_sharing::Finalizing>,
    sharing_started: bool,
}

struct FrozenStorage {
    view: HtmlConstructionResult,
    control: ControlIdentity,
    construction: ConstructionOwner,
    lease: Option<bounded::HtmlConstructionLease>,
    payloads: PayloadStorage,
    effects: ConstructionEffects,
    /// Present until publication moves the evidence into the published history
    /// beside this storage, where only its `PublishedEvidence` form is read.
    evidence: Option<ParseEvidence>,
    fragment: Option<FragmentContextStorage>,
}
/// Actual backing and exact record-qualified ownership, not allocation proxies.
struct PayloadStorage {
    values: Vec<RetainedPayload>,
    allocation: VectorCapacity<RetainedPayload>,
    node_joins: Vec<NodeJoins>,
    node_joins_allocation: VectorCapacity<NodeJoins>,
    /// Every row of a kind that is also searched across all nodes, in registry
    /// order. A scan of these lists visits only rows of that kind.
    kind_slots: [KindSlots; KIND_LISTS],
    /// The `FormAssociation` rows of listed controls that have a `form`
    /// attribute, in registry order: the only rows an id event can affect.
    /// Created with the first such row. Only creation and cloning give a
    /// control its `form` attribute (the attribute mutators change `value`,
    /// `nonce`, `open` and `checked`, and a merge touches only html and body),
    /// so rows enter the list when they are registered.
    explicit_forms: Option<KindSlots>,
    /// Auxiliary entries admitted for rows that are not appended yet: each
    /// [`PreparedSlot`] adds its entries when it is issued and gives them back
    /// when its row is appended. A slot that is dropped instead of appended
    /// leaves its entries here, so the release that retires the registry still
    /// returns them.
    prepared_entries: u64,
    preexisting_units: usize,
    preexisting_attributes: usize,
    /// Registry rows below this slot were audited by an earlier creation-time
    /// `check_new_aliases`; the completed-freeze preflight and release audit
    /// every row again.
    audited_rows: usize,
    /// Registry rows of selectedcontent metadata. Rows are never removed, so zero
    /// proves that no select has a selectedcontent descendant to update.
    selectedcontent_rows: usize,
    /// Times a second row of a family that report preparation requires to be
    /// unique per record joined that record. Zero proves the join names the
    /// only row, so preparation need not scan the registry to count them.
    repeated_unique_rows: usize,
    /// The generation of the cached tree roots in [`NodeJoins`]; `None` once no
    /// later generation can be named, when nothing is cached.
    tree_stamp: Option<NonZeroU32>,
    /// How many topology changes have been announced, or `None` once the count
    /// cannot go on; see [`tree_root`].
    topology_changes: Option<u64>,
}
/// Derived joins into the actual payload registry, one cell per record
/// position. Dense record positions are append-only; a join never substitutes
/// for the row's location or ticket. `text` and `textarea` are rewritten as
/// their rows are merged or reset; `rows` names the first registry row of each
/// node-keyed metadata kind bound to the record, so a lookup visits one cell and
/// one row instead of scanning the registry. A join is written only for a row
/// that exists, and registry rows are appended but never removed or reordered
/// while the registry is live; rows of a joined kind are never replaced, so a
/// join always names a row of its kind (the callers still check what they get).
#[derive(Clone, Copy)]
struct NodeJoins {
    text: Option<NonZeroU32>,
    textarea: Option<NonZeroU32>,
    rows: JoinTable<Option<NonZeroU32>>,
    /// Registry slot plus one of the name of the record's last attribute as
    /// registered at creation; see `PayloadStorage::attribute_value_candidate`.
    attribute_strings: Option<NonZeroU32>,
    /// The record's tree root as last found; see [`tree_root`].
    tree_root: tree_root::TreeRootCell,
}
/// Node-keyed registry row kinds with a join in [`NodeJoins`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Join {
    Attributes,
    NodeDocument,
    Document,
    Template,
    FormAssociation,
    Script,
    Image,
    Hyperlink,
    Style,
    Dialog,
    Nonce,
    Input,
    SelectFamily,
    Base,
    BaseDocument,
    LookupRegistry,
    /// The latest style occurrence naming the record as its subject.
    StyleOccurrence,
}
impl Join {
    /// Report preparation requires exactly one row of these kinds per record.
    fn unique_per_record(self) -> bool {
        matches!(
            self,
            Self::Image | Self::Hyperlink | Self::Style | Self::Dialog
        )
    }
}
const JOIN_COUNT: usize = Join::StyleOccurrence as usize + 1;
/// One value per [`Join`]; a lookup cannot be out of range.
type JoinTable<T> = Table<Join, T, JOIN_COUNT>;
slots!(Join, JOIN_COUNT, {
    Attributes => 0,
    NodeDocument => 1,
    Document => 2,
    Template => 3,
    FormAssociation => 4,
    Script => 5,
    Image => 6,
    Hyperlink => 7,
    Style => 8,
    Dialog => 9,
    Nonce => 10,
    Input => 11,
    SelectFamily => 12,
    Base => 13,
    BaseDocument => 14,
    LookupRegistry => 15,
    StyleOccurrence => 16,
});
/// Cells hold a registry slot plus one in 32 bits; a slot beyond that is
/// refused as the registry invariant (see `joined_cell`).
const _: () = assert!(
    mem::size_of::<NodeJoins>() == 4 * (JOIN_COUNT + 3) + mem::size_of::<tree_root::TreeRootCell>()
);
impl Default for NodeJoins {
    fn default() -> Self {
        Self {
            text: None,
            textarea: None,
            rows: JoinTable::filled(None),
            attribute_strings: None,
            tree_root: tree_root::TreeRootCell::default(),
        }
    }
}
impl NodeJoins {
    fn get(&self, join: Join) -> Option<usize> {
        self.rows.copy(join).and_then(cell_slot)
    }
    /// The registry row holding the record's merged text.
    fn text(&self) -> Option<usize> {
        self.text.and_then(cell_slot)
    }
    /// The registry row holding the record's textarea state.
    fn textarea(&self) -> Option<usize> {
        self.textarea.and_then(cell_slot)
    }
    /// Names the latest registry row, the one a back-to-front scan finds.
    fn set_latest(&mut self, join: Join, slot: usize) -> Result<(), SessionError> {
        *self.rows.get_mut(join) = Some(joined_cell(slot)?);
        Ok(())
    }
    /// Keeps the earliest registry row, the one a front-to-back scan finds, and
    /// reports whether the cell already named another row.
    fn set_first(&mut self, join: Join, slot: usize) -> Result<bool, SessionError> {
        let cell = self.rows.get_mut(join);
        let slot = joined_cell(slot)?;
        let repeated = cell.is_some_and(|current| current != slot);
        if cell.is_none_or(|current| slot < current) {
            *cell = Some(slot);
        }
        Ok(repeated)
    }
}
/// Slot plus one as a nonzero 32-bit cell. A registry of more than `u32::MAX`
/// rows cannot be joined (it would hold over a terabyte of 320-byte rows), so
/// a larger slot is refused as the registry invariant, never truncated or
/// wrapped onto another row.
fn joined_cell(slot: usize) -> Result<NonZeroU32, SessionError> {
    u32::try_from(slot)
        .ok()
        .and_then(|slot| slot.checked_add(1))
        .and_then(NonZeroU32::new)
        .ok_or_else(SessionError::registry_row)
}
/// The registry slot a cell names.
fn cell_slot(cell: NonZeroU32) -> Option<usize> {
    usize::try_from(cell.get().checked_sub(1)?).ok()
}
/// Registry-wide kind lists with their own admitted backing.
#[derive(Clone, Copy, Eq, PartialEq)]
enum KindList {
    FormAssociation,
    Base,
    AssociationDocument,
}
const KIND_LISTS: usize = KindList::AssociationDocument as usize + 1;
/// The admission of one registry row: its auxiliary entry, and the entries of
/// the lists it joins, charged before the row exists. Only
/// `SessionStorage::prepare_payload_slot*` issues one, and `PayloadStorage::push_row`
/// consumes it by appending the row it admitted, so a row cannot be appended
/// unadmitted. A slot that is dropped instead leaves its entries in
/// [`PayloadStorage::prepared_entries`], which the release of the registry
/// returns with the entries of the rows.
#[must_use = "an admitted slot is consumed by appending its row with `push_row`"]
struct PreparedSlot {
    entries: u64,
    list: Option<KindList>,
    explicit_form: bool,
}
struct KindSlots {
    slots: Vec<usize>,
    allocation: VectorCapacity<usize>,
}
enum RetainedPayload {
    CspState(Option<Box<csp::State>>),
    CspPolicy(Box<csp::Policy>),
    CspTransition(Box<csp::Transition>),
    AssociationDocument(pending_association::DocumentCapture),
    AssociationOccurrence(OwnedVec<pending_association::Occurrence>),
    AssociationCapturedContext(pending_association::CapturedContext),
    AssociatedInputs {
        inputs: Arc<associated::Inputs>,
        usage: associated::AssociatedStorageUsage,
    },
    Dialog {
        node: usize,
        state: dialog::DialogMetadata,
    },
    Style {
        node: usize,
        state: style::StyleMetadata,
    },
    Hyperlink {
        node: usize,
        state: associated::HyperlinkMetadata,
    },
    Image {
        node: usize,
        state: associated::ImageMetadata,
    },
    Nonce {
        node: Option<usize>,
        raw: usize,
    },
    BaseDocument {
        node: usize,
        facts: crate::HtmlParseBaseFacts,
        first: Option<usize>,
    },
    Base {
        node: Option<usize>,
        domain: base::ResponseDomain,
        frozen: base::BaseValue,
    },
    Shadow {
        root: Option<usize>,
        state: context::ShadowState,
    },
    ConstructionCause {
        node: Option<usize>,
        cause: provenance::ConstructionCauseRef,
    },
    StandaloneDocumentCompletion {
        node: usize,
        state: StandaloneDocumentState,
    },
    PiAttributes {
        node: Option<usize>,
        values: OwnedVec<pi::PseudoAttribute>,
        ready: bool,
    },
    LookupRegistry {
        node: Option<usize>,
        registry: context::RegistryRef,
        creation: Option<context::ElementCreationState>,
    },
    NodeDocument {
        node: Option<usize>,
        document: DocumentRef,
    },
    Document {
        node: Option<usize>,
        state: template::DocumentState,
        class: crate::HtmlDocumentClass,
        meta: meta::MetaHistory,
        interaction: dialog::InteractionHistory,
    },
    MetaMissing {
        document: usize,
        subject: usize,
        kind: meta::MetaMissingKind,
        cause: Option<provenance::ConstructionCauseRef>,
    },
    Template {
        node: Option<usize>,
        state: template::TemplateMetadata,
    },
    String {
        value: DocumentString,
        allocation: DocumentStringAllocation,
        origins: Option<OriginGroupRef>,
        location: StringLocation,
    },
    HistoricalString {
        origins: OriginGroupRef,
    },
    Attributes(Option<AttributeStorage>),
    // Presence is an actual closed-recipient proof issued by ordinary creation;
    // no task was queued and the source toggle tracker remains null.
    Details {
        node: Option<usize>,
        notification: details::NotificationDomain,
    },
    SvgScript {
        node: Option<usize>,
        state: SvgScriptState,
    },
    Link {
        node: Option<usize>,
        parser_document: Option<DocumentRef>,
    },
    Script {
        node: Option<usize>,
        metadata: ScriptMetadata,
    },
    FormAssociation {
        // None is an admitted, initialized construction slot. Only an actual
        // successful allocation can bind it; a stopped slot is cleanup-only.
        node: Option<usize>,
        state: FormAssociationState,
    },
    Textarea {
        node: Option<usize>,
        state: TextareaState,
    },
    Output {
        node: Option<usize>,
        state: OutputState,
    },
    Input {
        node: Option<usize>,
        raw: usize,
        complete: bool,
        state: InputState,
    },
    InputSource {
        node: Option<usize>,
        token: crate::HtmlSourceRange,
        value_attribute: Option<crate::HtmlSourceRange>,
    },
    SelectFamily {
        node: Option<usize>,
        state: SelectMetadata,
    },
    WorkList(Option<OwnedVec<usize>>),
    /// The style elements of one tree in tree order, shared by the style
    /// occurrences recorded while the tree held exactly a prefix of them.
    StyleOrder(OwnedVec<usize>),
    // A replaced raw backing was disposed; its initialized registry slot
    // remains admitted until the registry itself is released.
    Retired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextareaState {
    raw: usize,
    pub(crate) flags: HtmlTextareaFlags,
}
/// Pinned output override issued only by actual fresh creation reset.
/// Nonnull setters/general form reset have no producer in this checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OutputDefaultValueOverride {
    Null,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OutputState {
    pub(crate) default_value_override: OutputDefaultValueOverride,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InputState {
    pub(crate) flags: crate::HtmlInputFlags,
    pub(crate) cursor: usize,
    pub(crate) selected_files_empty: bool,
    pub(crate) value_rewritten_during_creation: bool,
    /// Actual produced multiple-Email values, not a resplit of joined text.
    pub(crate) email_value_count: Option<usize>,
    image: input_image::ImageState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FormAssociationState {
    pub(crate) kind: HtmlNativeFormAssociationKind,
    owner: Option<usize>,
    pub(crate) parser_inserted: bool,
    pub(crate) evidence: FormAssociationEvidence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormAssociationEvidence {
    Created,
    ParserAssociated,
    Reset,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FormEffectStage {
    Removal,
    RemovalChildren,
    Adoption,
    Insertion,
    InsertionChildren,
    IdChange,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FormEffectPass {
    Local,
    Intrinsic,
    Global {
        /// Position in the registry's list of rows of listed controls with a
        /// form attribute.
        next_form: usize,
        tree: Option<usize>,
    },
}
/// Fixed, retained progress over the same actual tree and payload registry.
/// A stopped prefix is for cleanup only and cannot be resumed or frozen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FormEffects {
    root: usize,
    old_parent: Option<usize>,
    new_parent: Option<usize>,
    stage: FormEffectStage,
    insert: bool,
    cause: InsertionCause,
    next: Option<usize>,
    pass: FormEffectPass,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InsertionCause {
    Parser,
    NonParser,
}
/// Source processing model facts retained in the actual arena owner.
/// Owned document positions stay local; captured documents retain checked IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMetadata {
    parser_document: Option<DocumentRef>,
    preparation_document: Option<DocumentRef>,
    pub(crate) flags: HtmlScriptFlags,
    pub(crate) script_type: Option<HtmlScriptKind>,
    pub(crate) result: HtmlScriptResult,
    pub(crate) steps_when_result_ready: Option<HtmlScriptReadySteps>,
}
impl ScriptMetadata {
    fn admit_clone(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        for document in [
            self.parser_document.as_ref(),
            self.preparation_document.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            document.admit_clone(c)?;
        }
        Ok(())
    }
}
enum AttributeStorage {
    Removing {
        values: OrderedAttributes,
        allocation: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
        element: Option<usize>,
        index: usize,
        shifted: usize,
        popped: bool,
        rebound: usize,
    },
    Building {
        values: Vec<Attribute>,
        allocation: VectorCapacity<Attribute>,
        original_len: usize,
        reversed_pairs: usize,
    },
    /// The same actual staging vector, consumed in lexical order by last-pop.
    /// Moved fields still bound here resolve into the actual destination until
    /// the charged registry walk has rebound every original string location.
    Transferring {
        values: Vec<Attribute>,
        allocation: VectorCapacity<Attribute>,
        destination: usize,
        start: usize,
        original_len: usize,
        moved: usize,
    },
    Frozen {
        values: OrderedAttributes,
        allocation: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
        element: Option<usize>,
    },
    /// The staged attributes were shared for validation, validation stopped
    /// (refused by the attributes themselves, by a work stop or by a
    /// cancellation), and the controller refused to unshare them again. The
    /// same actual Arc and ticket stay here for the cleanup walk to retry the
    /// release.
    Unvalidated {
        backing: Arc<Vec<Attribute>>,
        allocation: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
        original_len: usize,
        reversed_pairs: usize,
    },
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum StringLocation {
    Detached,
    Text(usize),
    Comment(usize),
    ProcessingInstructionTarget(usize),
    ProcessingInstructionData(usize),
    DoctypeName(usize),
    DoctypePublic(usize),
    DoctypeSystem(usize),
    ElementName(usize),
    Namespace(usize),
    AttributeName(usize, usize),
    AttributeValue(usize, usize),
    AttributeNamespace(usize, usize),
    AttributePrefix(usize, usize),
    PiName(usize, usize),
    PiValue(usize, usize),
}
pub(crate) struct TextInsertion {
    pub(crate) _node: ParserNode,
    pub(crate) _offset: usize,
}
struct ReleasedStorage {
    construction: ConstructionOwner,
    payloads: PayloadStorage,
}

/// The join a registry row provides, when it is bound to a record.
fn row_join(row: &RetainedPayload) -> Option<(Join, usize)> {
    Some(match row {
        RetainedPayload::Attributes(Some(
            AttributeStorage::Frozen {
                element: Some(node),
                ..
            }
            | AttributeStorage::Removing {
                element: Some(node),
                ..
            },
        )) => (Join::Attributes, *node),
        RetainedPayload::NodeDocument {
            node: Some(node), ..
        } => (Join::NodeDocument, *node),
        RetainedPayload::Document {
            node: Some(node), ..
        } => (Join::Document, *node),
        RetainedPayload::Template {
            node: Some(node), ..
        } => (Join::Template, *node),
        RetainedPayload::FormAssociation {
            node: Some(node), ..
        } => (Join::FormAssociation, *node),
        RetainedPayload::Script {
            node: Some(node), ..
        } => (Join::Script, *node),
        RetainedPayload::Image { node, .. } => (Join::Image, *node),
        RetainedPayload::Hyperlink { node, .. } => (Join::Hyperlink, *node),
        RetainedPayload::Style { node, .. } => (Join::Style, *node),
        RetainedPayload::Dialog { node, .. } => (Join::Dialog, *node),
        RetainedPayload::Nonce {
            node: Some(node), ..
        } => (Join::Nonce, *node),
        RetainedPayload::Input {
            node: Some(node), ..
        } => (Join::Input, *node),
        RetainedPayload::SelectFamily {
            node: Some(node), ..
        } => (Join::SelectFamily, *node),
        RetainedPayload::Base {
            node: Some(node), ..
        } => (Join::Base, *node),
        RetainedPayload::BaseDocument { node, .. } => (Join::BaseDocument, *node),
        RetainedPayload::LookupRegistry {
            node: Some(node), ..
        } => (Join::LookupRegistry, *node),
        _ => return None,
    })
}
/// The registry-wide list a row belongs to, whatever its binding.
fn row_list(row: &RetainedPayload) -> Option<KindList> {
    match row {
        RetainedPayload::FormAssociation { .. } => Some(KindList::FormAssociation),
        RetainedPayload::Base { .. } => Some(KindList::Base),
        RetainedPayload::AssociationDocument(_) => Some(KindList::AssociationDocument),
        _ => None,
    }
}

impl PayloadStorage {
    /// The first registry row of `join` bound to `node`: one join cell, however
    /// many other rows the registry retains. The caller checks the row it gets.
    fn joined(
        &self,
        control: &mut ParseControl,
        node: usize,
        join: Join,
    ) -> Result<Option<usize>, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.node_joins.get(node).and_then(|cell| cell.get(join)))
    }
    /// Forgets the join of the row at `slot`, for a test that leaves the row
    /// bound to no node.
    #[cfg(test)]
    fn unjoin_for_test(&mut self, slot: usize) {
        if let Some((join, node)) = self.values.get(slot).and_then(row_join)
            && let Some(cell) = self.node_joins.get_mut(node)
        {
            *cell.rows.get_mut(join) = None;
        }
    }
    /// The same join cell without a charge, for a published report, which has
    /// no controller. The caller checks the row it gets.
    fn joined_uncharged(&self, node: usize, join: Join) -> Option<usize> {
        self.node_joins.get(node).and_then(|cell| cell.get(join))
    }
    /// Registers the row at `slot` under the record it is bound to. Binding a
    /// row and joining it are one infallible step at every binding site; the
    /// test-build check at release compares every join with the registry.
    fn join_row(&mut self, slot: usize) -> Result<(), SessionError> {
        let Some((join, node)) = self.values.get(slot).and_then(row_join) else {
            return Ok(());
        };
        let repeated = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?
            .set_first(join, slot)?;
        if repeated && join.unique_per_record() {
            self.repeated_unique_rows = self
                .repeated_unique_rows
                .checked_add(1)
                .ok_or_else(SessionError::registry_row)?;
        }
        Ok(())
    }
    /// How many rows of `join` name `node`, or `None` when only a registry scan
    /// can tell. With no repeated row of any unique family, the join names the
    /// only row, so one cell and one row decide the count.
    fn unique_row_count(
        &self,
        control: &mut ParseControl,
        node: usize,
        join: Join,
    ) -> Result<Option<usize>, SessionError> {
        if self.repeated_unique_rows != 0 || !join.unique_per_record() {
            return Ok(None);
        }
        let Some(slot) = self.joined(control, node, join)? else {
            return Ok(Some(0));
        };
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self
            .values
            .get(slot)
            .and_then(row_join)
            .filter(|&(row, owner)| row == join && owner == node)
            .map(|_| 1))
    }
    /// Names `slot` as the latest row of `join` for `node`; used by the kinds
    /// whose join follows a back-to-front scan rather than the first row.
    fn join_latest(&mut self, node: usize, join: Join, slot: usize) -> Result<(), SessionError> {
        self.node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?
            .set_latest(join, slot)
    }
    /// The registry row now holding a record's merged text.
    fn join_text(&mut self, node: usize, slot: usize) -> Result<(), SessionError> {
        let cell = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?;
        cell.text = Some(joined_cell(slot)?);
        Ok(())
    }
    /// The registry row holding a record's textarea state.
    fn join_textarea(&mut self, node: usize, slot: usize) -> Result<(), SessionError> {
        let cell = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?;
        cell.textarea = Some(joined_cell(slot)?);
        Ok(())
    }
    /// The registry slots of one registry-wide kind, in registry order.
    fn kind_list(&self, list: KindList) -> &[usize] {
        self.kind_slots
            .get(list as usize)
            .map_or(&[], |kind| kind.slots.as_slice())
    }
    /// The `FormAssociation` rows of listed controls with a `form` attribute.
    fn explicit_form_list(&self) -> &[usize] {
        self.explicit_forms
            .as_ref()
            .map_or(&[], |kind| kind.slots.as_slice())
    }
    /// Appends the row `prepared` admitted. A row already bound to a record is
    /// joined; rows of a registry-wide kind enter that kind's list, and a row
    /// of a listed control with a `form` attribute enters the explicit-form
    /// list, whose entries were admitted with the row. A row of another kind
    /// than the one admitted, or a list slot that was not admitted, refuses the
    /// row before anything is appended; the refused row stays in the registry
    /// for its release (see [`Self::retain_refused`]).
    fn push_row(
        &mut self,
        prepared: PreparedSlot,
        row: RetainedPayload,
    ) -> Result<usize, SessionError> {
        let (prepared_entries, selectedcontent_rows) = match self.admit_row(&prepared, &row) {
            Ok(admitted) => admitted,
            Err(error) => {
                self.retain_refused(row);
                return Err(error);
            }
        };
        let list = row_list(&row);
        let slot = self.values.len();
        self.prepared_entries = prepared_entries;
        self.selectedcontent_rows = selectedcontent_rows;
        self.values.push(row);
        if let Some(kind) = list.and_then(|list| self.kind_slots.get_mut(list as usize)) {
            kind.slots.push(slot);
        }
        if prepared.explicit_form
            && let Some(kind) = self.explicit_forms.as_mut()
        {
            kind.slots.push(slot);
        }
        self.join_row(slot)?;
        Ok(slot)
    }
    /// Checks that `row` is the kind of row `prepared` admitted and that
    /// everything the append writes was admitted with it, and returns the
    /// counts the append leaves. Writes nothing.
    fn admit_row(
        &self,
        prepared: &PreparedSlot,
        row: &RetainedPayload,
    ) -> Result<(u64, usize), SessionError> {
        let list = row_list(row);
        if list != prepared.list
            || prepared.explicit_form && list != Some(KindList::FormAssociation)
        {
            return Err(SessionError::registry_row());
        }
        if let Some(list) = list
            && self
                .kind_slots
                .get(list as usize)
                .is_none_or(|kind| kind.slots.len() >= kind.slots.capacity())
        {
            return Err(SessionError::registry_row());
        }
        if prepared.explicit_form
            && self
                .explicit_forms
                .as_ref()
                .is_none_or(|kind| kind.slots.len() >= kind.slots.capacity())
        {
            return Err(SessionError::registry_row());
        }
        let prepared_entries = self
            .prepared_entries
            .checked_sub(prepared.entries)
            .ok_or_else(SessionError::registry_row)?;
        let selectedcontent_rows = if matches!(
            row,
            RetainedPayload::SelectFamily {
                state: SelectMetadata::Selectedcontent(_),
                ..
            }
        ) {
            self.selectedcontent_rows
                .checked_add(1)
                .ok_or_else(SessionError::registry_row)?
        } else {
            self.selectedcontent_rows
        };
        Ok((prepared_entries, selectedcontent_rows))
    }
    /// Keeps a refused row in the registry, unlisted and unjoined, so that the
    /// release of the registry returns whatever tickets the row owns: only that
    /// release knows how to return each kind of row. The slot the prepare
    /// reserved holds the row, and the row takes over the entry of its slot.
    /// The entries admitted for its lists stay outstanding in
    /// `prepared_entries`. A refusal is an inconsistency of the registry, which
    /// the caller reports as the registry row.
    fn retain_refused(&mut self, row: RetainedPayload) {
        if self.values.len() < self.values.capacity() {
            self.prepared_entries = self.prepared_entries.saturating_sub(1);
            self.values.push(row);
        }
    }
    /// Records where creation registered the record's attribute strings: the
    /// name of its last attribute, followed by that attribute's value, then the
    /// previous attribute's name and value, and so on.
    fn record_attribute_strings(
        &mut self,
        node: usize,
        last_name_slot: usize,
    ) -> Result<(), SessionError> {
        let cell = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?;
        cell.attribute_strings = Some(joined_cell(last_name_slot)?);
        Ok(())
    }
    /// An attribute edit makes the creation-time layout unreliable.
    fn forget_attribute_strings(&mut self, node: usize) {
        if let Some(cell) = self.node_joins.get_mut(node) {
            cell.attribute_strings = None;
        }
    }
    /// The slot where `position`'s value row sits if the record's attributes
    /// are as creation registered them. Only a candidate: the caller verifies
    /// the row's identity before use.
    fn attribute_value_candidate(
        &self,
        node: usize,
        attribute_count: usize,
        position: usize,
    ) -> Option<usize> {
        let last_name = cell_slot(self.node_joins.get(node)?.attribute_strings?)?;
        let from_last = attribute_count.checked_sub(1)?.checked_sub(position)?;
        last_name
            .checked_add(from_last.checked_mul(2)?)?
            .checked_add(1)
    }
    /// The first registry string row sharing `value`'s backing: what the
    /// fallback scan finds. Test builds compare every verified hint with it.
    #[cfg(test)]
    fn first_string_sharing(&self, value: &DocumentString) -> Option<usize> {
        self.values.iter().position(|row| {
            matches!(row, RetainedPayload::String { value: stored, .. }
                if Arc::ptr_eq(value.shared_code_units(), stored.shared_code_units()))
        })
    }
    /// Every join must name exactly what a front-to-back scan of the registry
    /// finds, and every list exactly the rows of its kind.
    #[cfg(test)]
    fn assert_joins_match_registry(&self) {
        let mut expected: Vec<NodeJoins> = vec![NodeJoins::default(); self.node_joins.len()];
        let mut lists: [Vec<usize>; KIND_LISTS] = Default::default();
        for (slot, row) in self.values.iter().enumerate() {
            if let Some((join, node)) = row_join(row) {
                let cell = &mut expected[node];
                assert!(
                    cell.get(join).is_none(),
                    "two {join:?} rows are bound to record {node}"
                );
                cell.set_first(join, slot).unwrap();
            }
            if let Some(list) = row_list(row) {
                lists[list as usize].push(slot);
            }
            if let Some(subject) = pending_association::style_occurrence_subject(row) {
                expected[subject]
                    .set_latest(Join::StyleOccurrence, slot)
                    .unwrap();
            }
        }
        for (node, (cell, joined)) in expected.iter().zip(&self.node_joins).enumerate() {
            assert!(
                cell.rows == joined.rows,
                "joins of record {node} differ from the registry: {:?} against {:?}",
                cell.rows,
                joined.rows
            );
        }
        for (list, kind) in lists.iter().zip(&self.kind_slots) {
            assert!(list == &kind.slots, "a kind list differs from the registry");
        }
    }
    /// Cleanup is a bounded ownership walk over admitted entries, permitted
    /// after a terminal stop. No parser work is resumed and no work refunded.
    /// Each live location is checked against its actual immutable record.
    fn check_aliases(
        &self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.audit_aliases(records, control, 0)
    }
    /// The creation-time audit: only the rows registered since the previous one,
    /// so the cost is the rows the operations in between added, not the registry.
    /// Every row is audited again by the completed-freeze preflight and release.
    fn check_new_aliases(
        &mut self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.audit_aliases(records, control, self.audited_rows)?;
        self.audited_rows = self.values.len();
        Ok(())
    }
    fn audit_aliases(
        &self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
        first: usize,
    ) -> Result<(), SessionError> {
        for index in first..self.values.len() {
            cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
            let payload = self
                .values
                .get(index)
                .ok_or_else(|| latch_registry_row(control))?;
            match payload {
                RetainedPayload::String {
                    value,
                    allocation,
                    location,
                    ..
                } => {
                    if allocation.finalized() {
                        control.require_document_string_for_disposal(allocation, value)?;
                    }
                    let stored = location.value(records, &self.values, index, control)?;
                    cleanup_charge(control, WorkCategory::ProviderCall, 2)?;
                    let expected = if let Some(stored) = stored {
                        if !Arc::ptr_eq(stored.shared_code_units(), value.shared_code_units()) {
                            return Err(SessionError::AliasedPayload);
                        }
                        2
                    } else {
                        1
                    };
                    if !allocation.finalized()
                        && Arc::strong_count(value.shared_code_units()) != expected
                    {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(
                    AttributeStorage::Frozen {
                        values,
                        element,
                        shared,
                        ..
                    }
                    | AttributeStorage::Removing {
                        values,
                        element,
                        shared,
                        ..
                    },
                )) => {
                    let expected = if let Some(index) = element {
                        cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                        cleanup_charge(control, WorkCategory::ProviderCall, 1)?;
                        let Some(HtmlNodeKind::Element(record)) =
                            records.node_at(*index).map(|node| node.kind())
                        else {
                            return Err(SessionError::AliasedPayload);
                        };
                        if !Arc::ptr_eq(
                            values.shared_attributes(),
                            record.attributes().shared_attributes(),
                        ) {
                            return Err(SessionError::AliasedPayload);
                        }
                        2
                    } else {
                        1
                    };
                    cleanup_charge(control, WorkCategory::ProviderCall, 1)?;
                    if !shared.finalized()
                        && Arc::strong_count(values.shared_attributes()) != expected
                    {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(AttributeStorage::Unvalidated {
                    backing,
                    shared,
                    ..
                })) => {
                    cleanup_charge(control, WorkCategory::ProviderCall, 1)?;
                    if !shared.finalized() && Arc::strong_count(backing) != 1 {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(
                    AttributeStorage::Building { .. } | AttributeStorage::Transferring { .. },
                )) => {}
                RetainedPayload::Attributes(None) => return Err(SessionError::Consumed),
                RetainedPayload::CspState(_)
                | RetainedPayload::CspPolicy(_)
                | RetainedPayload::CspTransition(_)
                | RetainedPayload::BaseDocument { .. }
                | RetainedPayload::Base { .. }
                | RetainedPayload::SvgScript { .. }
                | RetainedPayload::ConstructionCause { .. }
                | RetainedPayload::Script { .. }
                | RetainedPayload::NodeDocument { .. }
                | RetainedPayload::LookupRegistry { .. }
                | RetainedPayload::Shadow { .. }
                | RetainedPayload::StandaloneDocumentCompletion { .. }
                | RetainedPayload::Document { .. }
                | RetainedPayload::MetaMissing { .. }
                | RetainedPayload::Template { .. }
                | RetainedPayload::FormAssociation { .. }
                | RetainedPayload::Textarea { .. }
                | RetainedPayload::Nonce { .. }
                | RetainedPayload::Output { .. }
                | RetainedPayload::InputSource { .. }
                | RetainedPayload::Input { .. }
                | RetainedPayload::SelectFamily { .. }
                | RetainedPayload::Details { .. }
                | RetainedPayload::Link { .. }
                | RetainedPayload::AssociatedInputs { .. }
                | RetainedPayload::Hyperlink { .. }
                | RetainedPayload::Image { .. }
                | RetainedPayload::Dialog { .. }
                | RetainedPayload::Style { .. }
                | RetainedPayload::AssociationDocument(_)
                | RetainedPayload::AssociationOccurrence(_)
                | RetainedPayload::AssociationCapturedContext(_)
                | RetainedPayload::Retired => {}
                RetainedPayload::HistoricalString { origins } => {
                    origins.require_disposal_owner(control)?
                }
                RetainedPayload::PiAttributes { values, .. } => {
                    values.require_disposal_owner(control)?
                }
                RetainedPayload::WorkList(Some(list)) => list.require_disposal_owner(control)?,
                RetainedPayload::WorkList(None) => return Err(SessionError::Consumed),
                RetainedPayload::StyleOrder(list) => list.require_disposal_owner(control)?,
            }
        }
        Ok(())
    }

    fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), SessionError> {
        #[cfg(test)]
        self.assert_joins_match_registry();
        let Self {
            mut values,
            allocation,
            node_joins,
            node_joins_allocation,
            kind_slots,
            explicit_forms,
            prepared_entries,
            ..
        } = self;
        let mut csp_storage = crate::meta_csp::Storage::ZERO;
        let join_entries = node_joins.len() as u64;
        // A refused release latches its stop in the controller (`ParseControl::fail`),
        // and cleanup continues so every other owner is still released; the first
        // refusal is returned once the walk is complete.
        let mut refused = release_vector(control, node_joins, node_joins_allocation).err();
        keep_first(
            &mut refused,
            release_logical(
                control,
                owner,
                HtmlParseResource::AuxiliaryEntries,
                join_entries,
            ),
        );
        for list in kind_slots.into_iter().chain(explicit_forms) {
            let list_entries = list.slots.len() as u64;
            if let Err(error) = release_vector(control, list.slots, list.allocation) {
                refused.get_or_insert(error);
            }
            keep_first(
                &mut refused,
                release_logical(
                    control,
                    owner,
                    HtmlParseResource::AuxiliaryEntries,
                    list_entries,
                ),
            );
        }
        // Entries admitted for rows that were never appended belong to no row.
        let entries = (values.len() as u64).saturating_add(prepared_entries);
        // Attribute containers own references to registered string backings.
        // Dispose containers first; reverse-pop then retains the real Vec until
        // its capacity ticket is consumed.
        for payload in &mut values {
            if let RetainedPayload::PiAttributes { values, .. } = payload {
                // PI maps own the second references to registered backings.
                // Remove those references before string-ticket disposal.
                loop {
                    let popped = if control.stop().is_some() {
                        values.cleanup_pop(control)
                    } else {
                        values.try_pop(control)
                    };
                    let popped = match popped {
                        Ok(popped) => popped,
                        Err(_) => match values.cleanup_pop(control) {
                            Ok(popped) => popped,
                            Err(stop) => {
                                refused.get_or_insert(SessionError::Stop(stop));
                                break;
                            }
                        },
                    };
                    match popped {
                        Some(pair) => drop(pair),
                        None => break,
                    }
                }
            }
            if let RetainedPayload::Attributes(storage) = payload {
                let Some(storage) = storage.take() else {
                    refused.get_or_insert(SessionError::Consumed);
                    continue;
                };
                let count = match &storage {
                    AttributeStorage::Building { values, .. }
                    | AttributeStorage::Transferring { values, .. } => values.len(),
                    AttributeStorage::Frozen { values, .. }
                    | AttributeStorage::Removing { values, .. } => values.len(),
                    AttributeStorage::Unvalidated { backing, .. } => backing.len(),
                };
                match storage {
                    AttributeStorage::Building {
                        values, allocation, ..
                    }
                    | AttributeStorage::Transferring {
                        values, allocation, ..
                    } => {
                        if let Err(error) = release_vector(control, values, allocation) {
                            refused.get_or_insert(error);
                        }
                    }
                    AttributeStorage::Frozen {
                        values,
                        allocation,
                        shared,
                        ..
                    }
                    | AttributeStorage::Removing {
                        values,
                        allocation,
                        shared,
                        ..
                    } => {
                        if shared.finalized() {
                            // A refused bank and tickets are dropped with the
                            // payload; the latched stop reports the skipped credit.
                            keep_first(
                                &mut refused,
                                control
                                    .release_final_attributes(values, allocation, shared)
                                    .map_err(FinalReleaseError::discard),
                            );
                            continue;
                        }
                        let backing = values.into_shared_attributes();
                        match control.release_shared(backing, shared) {
                            Ok(vector) => {
                                if let Err(error) = release_vector(control, vector, allocation) {
                                    refused.get_or_insert(error);
                                }
                            }
                            Err(_) => {
                                refused.get_or_insert(SessionError::AliasedPayload);
                            }
                        }
                    }
                    AttributeStorage::Unvalidated {
                        backing,
                        allocation,
                        shared,
                        ..
                    } => match control.release_shared(backing, shared) {
                        Ok(vector) => {
                            if let Err(error) = release_vector(control, vector, allocation) {
                                refused.get_or_insert(error);
                            }
                        }
                        Err(_) => {
                            refused.get_or_insert(SessionError::AliasedPayload);
                        }
                    },
                }
                keep_first(
                    &mut refused,
                    release_logical(
                        control,
                        owner,
                        HtmlParseResource::ManagedAttributes,
                        count as u64,
                    ),
                );
            }
        }
        while let Some(payload) = values.pop() {
            let associated_usage = match &payload {
                RetainedPayload::AssociatedInputs { usage, .. } => Some(*usage),
                RetainedPayload::Hyperlink { state, .. } => Some(state.usage),
                RetainedPayload::Image { state, .. } => Some(state.usage),
                RetainedPayload::AssociationCapturedContext(value) => {
                    Some(pending_association::captured_usage(value))
                }
                _ => None,
            };
            if let Some(usage) = associated_usage {
                drop(payload);
                keep_first(&mut refused, usage.release(control, owner));
                continue;
            }
            if let RetainedPayload::CspState(value) = payload {
                if let Some(value) = value {
                    csp_storage = value.account.live;
                    drop(value);
                }
                continue;
            }
            if let RetainedPayload::Document { mut meta, .. } = payload {
                if let Some(scheme) = meta.scheme.take() {
                    keep_first(&mut refused, scheme.release(control));
                }
                continue;
            }
            if let RetainedPayload::AssociationOccurrence(value) = payload {
                keep_first(
                    &mut refused,
                    pending_association::release_occurrence(value, control),
                );
                continue;
            }
            if let RetainedPayload::PiAttributes { values, .. } = payload {
                keep_first(&mut refused, values.release(control));
                continue;
            }
            if let RetainedPayload::WorkList(Some(list)) = payload {
                keep_first(&mut refused, list.release(control));
                continue;
            }
            if let RetainedPayload::StyleOrder(list) = payload {
                keep_first(&mut refused, list.release(control));
                continue;
            }
            if let RetainedPayload::InputSource {
                value_attribute, ..
            } = &payload
            {
                keep_first(
                    &mut refused,
                    release_logical(
                        control,
                        owner,
                        HtmlParseResource::SourceMapEntries,
                        if value_attribute.is_some() { 2 } else { 1 },
                    ),
                );
            }
            if let RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } = payload
            {
                drop(value);
                keep_first(&mut refused, control.release_document_string(allocation));
                drop(origins);
            }
        }
        keep_first(&mut refused, csp_storage.release(control));
        keep_first(&mut refused, release_vector(control, values, allocation));
        keep_first(
            &mut refused,
            release_logical(control, owner, HtmlParseResource::AuxiliaryEntries, entries),
        );
        refused.map_or(Ok(()), Err)
    }
}
/// The position `index` counts from the end of `original_len` values.
fn mirrored_position(original_len: usize, index: usize) -> Option<usize> {
    original_len.checked_sub(index.checked_add(1)?)
}
/// Where a staged attribute location currently sits in a stage that
/// `reversed_pairs` pairs have been swapped in.
fn staged_position(
    original_len: usize,
    reversed_pairs: usize,
    index: usize,
    control: &mut ParseControl,
) -> Result<usize, SessionError> {
    let Some(position) = mirrored_position(original_len, index) else {
        return Err(SessionError::AliasedPayload);
    };
    let boundary = original_len
        .checked_sub(reversed_pairs)
        .ok_or_else(|| latch_registry_row(control))?;
    if position < reversed_pairs || position >= boundary {
        mirrored_position(original_len, position).ok_or_else(|| latch_registry_row(control))
    } else {
        Ok(position)
    }
}
impl StringLocation {
    fn value<'a>(
        &self,
        records: ConstructionView<'a>,
        payloads: &'a [RetainedPayload],
        registry_index: usize,
        control: &mut ParseControl,
    ) -> Result<Option<&'a DocumentString>, SessionError> {
        let value = match *self {
            Self::PiName(map, index) | Self::PiValue(map, index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 2)?;
                match payloads.get(map) {
                    Some(RetainedPayload::PiAttributes { values, .. }) => {
                        values.as_slice().get(index).map(|pair| {
                            if matches!(self, Self::PiName(..)) {
                                &pair.name
                            } else {
                                &pair.value
                            }
                        })
                    }
                    _ => None,
                }
            }
            Self::Detached => None,
            Self::Text(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.node_at(index).map(|record| record.kind()) {
                    Some(HtmlNodeKind::Text(value)) => Some(value),
                    _ => None,
                }
            }
            Self::Comment(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.node_at(index).map(|record| record.kind()) {
                    Some(HtmlNodeKind::Comment(value)) => Some(value),
                    _ => None,
                }
            }
            Self::ProcessingInstructionTarget(index) | Self::ProcessingInstructionData(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.node_at(index).map(|record| record.kind()) {
                    Some(HtmlNodeKind::ProcessingInstruction(value)) => Some(match self {
                        Self::ProcessingInstructionTarget(_) => value.target(),
                        _ => value.data(),
                    }),
                    _ => None,
                }
            }
            Self::DoctypeName(index) | Self::DoctypePublic(index) | Self::DoctypeSystem(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.node_at(index).map(|record| record.kind()) {
                    Some(HtmlNodeKind::Doctype(value)) => match self {
                        Self::DoctypeName(_) => Some(value.name()),
                        Self::DoctypePublic(_) => value.public_id(),
                        _ => value.system_id(),
                    },
                    _ => None,
                }
            }
            Self::ElementName(index) | Self::Namespace(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.node_at(index).map(|record| record.kind()) {
                    Some(HtmlNodeKind::Element(value)) => match self {
                        Self::ElementName(..) => Some(value.name().local_name()),
                        _ => value.name().namespace().uri(),
                    },
                    _ => None,
                }
            }
            Self::AttributeName(container, index)
            | Self::AttributeValue(container, index)
            | Self::AttributeNamespace(container, index)
            | Self::AttributePrefix(container, index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 2)?;
                let attribute = match payloads.get(container) {
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Removing {
                        values,
                        index: removed,
                        shifted,
                        popped,
                        rebound,
                        ..
                    }))) => {
                        if registry_index >= *rebound && index == *removed && *popped {
                            return Ok(None);
                        }
                        let position = if registry_index < *rebound {
                            index
                        } else if index == *removed {
                            index
                                .checked_add(*shifted)
                                .ok_or_else(|| latch_registry_row(control))?
                        } else if index > *removed
                            && index
                                <= removed
                                    .checked_add(*shifted)
                                    .ok_or_else(|| latch_registry_row(control))?
                        {
                            index
                                .checked_sub(1)
                                .ok_or_else(|| latch_registry_row(control))?
                        } else {
                            index
                        };
                        values.as_slice().get(position)
                    }
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
                        values,
                        destination,
                        start,
                        original_len,
                        moved,
                        ..
                    }))) => {
                        if index < *moved {
                            cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                            let Some(position) = start.checked_add(index) else {
                                return Err(SessionError::AliasedPayload);
                            };
                            match payloads.get(*destination) {
                                Some(RetainedPayload::Attributes(Some(
                                    AttributeStorage::Frozen { values, .. },
                                ))) => values.as_slice().get(position),
                                _ => None,
                            }
                        } else {
                            let Some(position) = mirrored_position(*original_len, index) else {
                                return Err(SessionError::AliasedPayload);
                            };
                            values.get(position)
                        }
                    }
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                        values,
                        original_len,
                        reversed_pairs,
                        ..
                    }))) => {
                        let position =
                            staged_position(*original_len, *reversed_pairs, index, control)?;
                        values.get(position)
                    }
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Unvalidated {
                        backing,
                        original_len,
                        reversed_pairs,
                        ..
                    }))) => {
                        let position =
                            staged_position(*original_len, *reversed_pairs, index, control)?;
                        backing.get(position)
                    }
                    Some(RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                        values,
                        ..
                    }))) => values.as_slice().get(index),
                    _ => None,
                };
                attribute.and_then(|attribute| match self {
                    Self::AttributeName(..) => Some(attribute.name().local_name()),
                    Self::AttributeNamespace(..) => attribute.name().namespace().uri(),
                    Self::AttributePrefix(..) => attribute.name().prefix(),
                    _ => Some(attribute.value()),
                })
            }
        };
        if value.is_none() && !matches!(self, Self::Detached) {
            return Err(SessionError::AliasedPayload);
        }
        Ok(value)
    }
}

impl SessionStorage {
    fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        let effects = ConstructionEffects::new(control)?;
        let construction = ConstructionOwner::new(control)?;
        let (values, allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (node_joins, node_joins_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (forms, forms_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, node_joins, node_joins_allocation)?;
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (bases, bases_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, forms, forms_allocation)?;
                release_vector(control, node_joins, node_joins_allocation)?;
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (documents, documents_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, bases, bases_allocation)?;
                release_vector(control, forms, forms_allocation)?;
                release_vector(control, node_joins, node_joins_allocation)?;
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        // Indexed by `KindList`.
        let kind_slots = [
            KindSlots {
                slots: forms,
                allocation: forms_allocation,
            },
            KindSlots {
                slots: bases,
                allocation: bases_allocation,
            },
            KindSlots {
                slots: documents,
                allocation: documents_allocation,
            },
        ];
        let mut session = Self {
            csp_slot: None,
            #[cfg(test)]
            cancel_after_details_shift: None,
            #[cfg(test)]
            cancel_after_radio_peer: None,
            #[cfg(test)]
            cancel_after_text_append_chunk: None,
            control: control.identity(),
            effects,
            evidence: None,
            fragment: None,
            #[cfg(test)]
            cancel_after_removal: None,
            #[cfg(test)]
            cancel_after_textarea_reset: None,
            #[cfg(test)]
            cancel_after_output_reset: None,
            #[cfg(test)]
            cancel_after_input_reset: None,
            #[cfg(test)]
            lose_association_row: None,
            #[cfg(test)]
            cancel_after_range_append: None,
            #[cfg(test)]
            cancel_after_range_ingress: None,
            standalone_ineligibility: None,
            construction,
            failed_text_append: None,
            style_order: Some(pending_association::CurrentOrder::empty_main_tree()),
            meta_batches: meta::MetaDomainTable::filled(None),
            payloads: PayloadStorage {
                values,
                allocation,
                node_joins,
                node_joins_allocation,
                kind_slots,
                explicit_forms: None,
                prepared_entries: 0,
                preexisting_units: 0,
                preexisting_attributes: 0,
                audited_rows: 0,
                selectedcontent_rows: 0,
                repeated_unique_rows: 0,
                tree_stamp: NonZeroU32::new(1),
                topology_changes: Some(0),
            },
        };
        let kind = match root {
            RootKind::Document(mode) => HtmlNodeKind::Document(mode),
            #[cfg(test)]
            RootKind::Fragment => HtmlNodeKind::Fragment,
        };
        let allocated = payload_write::<Option<FormEffects>>(control, 1)
            .and_then(|()| session.allocate_native(control, kind, false, 0, 0));
        if let Err(error) = allocated {
            // The allocation failure is the error reported; a refused release has
            // latched its own stop, which is not reported over the first.
            let _ = session.release(control);
            return Err(error);
        }
        if matches!(
            session
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            let facts = (|| {
                let prepared = session.prepare_payload_slot_owned(control)?;
                session
                    .payloads
                    .push_row(
                        prepared,
                        RetainedPayload::Document {
                            node: Some(0),
                            state: template::DocumentState::Ordinary { inert: None },
                            class: crate::HtmlDocumentClass::Html,
                            meta: meta::MetaHistory::offline(),
                            interaction: dialog::InteractionHistory::new(),
                        },
                    )
                    .map_err(|_| latch_registry_row(control))?;
                let prepared = session.prepare_payload_slot_owned(control)?;
                session
                    .payloads
                    .push_row(
                        prepared,
                        RetainedPayload::NodeDocument {
                            node: Some(0),
                            document: DocumentRef::Owned(0),
                        },
                    )
                    .map_err(|_| latch_registry_row(control))?;
                Ok::<_, SessionError>(())
            })();
            if let Err(error) = facts {
                let _ = session.release(control);
                return Err(error);
            }
        }
        Ok(session)
    }

    fn require(&self, control: &mut ParseControl) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.effects.form.is_some() {
            return Err(control
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        }
        self.effects.require_idle(control)?;
        self.require_context_ready(control)?;
        Ok(())
    }
    fn node(&self, control: &mut ParseControl, index: usize) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        self.node_owned(control, index)
    }
    fn node_owned(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<ParserNode, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<ParserNode>() as u64)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let view = self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?;
        Ok(ParserNode {
            handle: view.handle(),
            index: view.position(),
        })
    }
    fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.node(control, 0)
    }
    fn index(&self, control: &mut ParseControl, node: &ParserNode) -> Result<usize, SessionError> {
        self.require(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        self.construction
            .storage(control)?
            .position(&node.handle)
            .map_err(|_| SessionError::ForeignNode)
    }

    /// Consumes preexisting immutable payloads, not tokenizer-created backing.
    #[cfg(test)]
    fn allocate_element(
        &mut self,
        control: &mut ParseControl,
        element: HtmlElement,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        // Actual preexisting input permanently leaves the closed producer
        // inventory. Admission precedes the monotone provenance transition.
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.standalone_ineligibility.is_none() {
            payload_write::<Option<StandaloneUnsupported>>(control, 1)?;
            self.standalone_ineligibility = Some(StandaloneUnsupported::Provenance);
        }
        let name = element.name();
        let template = exact_ascii(
            control,
            name.namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? && exact_ascii(control, Some(name.local_name()), "template")?;
        let mut units = name_units(control, name)?;
        let attributes = element.attributes().len();
        for index in 0..attributes {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let attribute = &element.attributes().as_slice()[index];
            let Some(next) = units
                .checked_add(name_units(control, attribute.name())?)
                .and_then(|sum| sum.checked_add(attribute.value().len()))
            else {
                return control
                    .representation(HtmlParseResource::ManagedUtf16Units)
                    .map_err(Into::into);
            };
            units = next;
        }
        let node = self.allocate_native(
            control,
            HtmlNodeKind::Element(element),
            template,
            attributes,
            units,
        )?;
        self.payloads.preexisting_units += units;
        self.payloads.preexisting_attributes += attributes;
        Ok(node)
    }

    fn allocate(
        &mut self,
        control: &mut ParseControl,
        kind: HtmlNodeKind,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        self.allocate_owned(control, kind, template, attributes, units)
    }
    fn allocate_owned(
        &mut self,
        control: &mut ParseControl,
        kind: HtmlNodeKind,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let document = matches!(
            self.construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        )
        .then_some(DocumentRef::Owned(0));
        self.allocate_in_document_owned(control, kind, template, attributes, units, document)
    }
    fn allocate_native(
        &mut self,
        control: &mut ParseControl,
        kind: HtmlNodeKind,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_native_record(control, creation_command(kind), template, attributes, units)
    }
    fn allocate_native_record(
        &mut self,
        control: &mut ParseControl,
        command: bounded::HtmlConstructionCommand,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        control.require_identity(&self.control)?;
        // A template is a record and its content fragment.
        let (count, entries): (usize, u64) = if template { (2, 4) } else { (1, 2) };
        let cause_slots = self.prepare_node_causes(control, count)?;
        // Each record retains its original child-ticket slot and one derived
        // node-join cell; payload metadata rows remain separately admitted.
        let requests = [
            (HtmlParseResource::ManagedNodes, count as u64),
            (HtmlParseResource::ManagedEdges, u64::from(template)),
            (HtmlParseResource::ManagedAttributes, attributes as u64),
            (HtmlParseResource::ManagedUtf16Units, units as u64),
            (HtmlParseResource::AuxiliaryEntries, entries),
        ];
        control.check_acquire_many(&requests)?;
        // Preserve the original node/edge/logical preflight before preparing
        // any new nonce backing; successful elements still bind the slot
        // before their native record can be published.
        let element_name = match &command {
            bounded::HtmlConstructionCommand::Create {
                kind: HtmlNodeKind::Element(element),
            } => Some(element.name()),
            bounded::HtmlConstructionCommand::CreateElement { name, .. } => Some(name),
            _ => None,
        };
        let nonce_slot = self.prepare_nonce(control, element_name)?;
        if nonce_slot.is_some() {
            payload_write::<Option<usize>>(control, 1)?;
        }
        let index = self.construction.len(control)?;
        control.reserve_vector_geometric(
            &mut self.payloads.node_joins,
            &mut self.payloads.node_joins_allocation,
            count,
        )?;
        control.admit_atomic_layout(mem::size_of::<NodeJoins>())?;
        control.charge(
            WorkCategory::CopyByte,
            count.saturating_mul(mem::size_of::<NodeJoins>()) as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, entries)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<ParserNode>() as u64)?;
        control.charge(WorkCategory::RecordVisit, 3)?;
        let Some(handle) = self.construction.apply(control, command, &requests)? else {
            return Err(latch_owner_refusal(control));
        };
        let Ok(position) = self
            .construction
            .storage(control)?
            .node(&handle)
            .map(|record| record.position())
        else {
            return Err(latch_registry_row(control));
        };
        let handle = ParserNode {
            handle,
            index: position,
        };
        for _ in 0..count {
            self.payloads.node_joins.push(NodeJoins::default());
        }
        if let Some(slot) = nonce_slot {
            let Some(RetainedPayload::Nonce { node, .. }) = self.payloads.values.get_mut(slot)
            else {
                return Err(latch_registry_row(control));
            };
            *node = Some(index);
            self.payloads
                .join_row(slot)
                .map_err(|_| latch_registry_row(control))?;
        }
        for (offset, slot) in cause_slots.into_iter().enumerate().take(count) {
            if let Some(slot) = slot {
                let Some(RetainedPayload::ConstructionCause { node, .. }) =
                    self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(control));
                };
                *node = Some(
                    index
                        .checked_add(offset)
                        .ok_or_else(|| latch_registry_row(control))?,
                );
            }
        }
        Ok(handle)
    }

    #[cfg_attr(test, track_caller)]
    fn prepare_payload_slot(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<PreparedSlot, SessionError> {
        self.require(control)?;
        self.prepare_payload_slot_owned(control)
    }
    // Only the synchronous effects owner may admit a raw replacement while
    // public/session entry points are blocked by the retained effects phase.
    #[cfg_attr(test, track_caller)]
    fn prepare_payload_slot_owned(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<PreparedSlot, SessionError> {
        self.prepare_payload_slot_for(control, None)
    }
    /// Admits one registry row and, for a registry-wide kind, the entry of its
    /// list; `push_row` then appends both.
    #[cfg_attr(test, track_caller)]
    fn prepare_payload_slot_for(
        &mut self,
        control: &mut ParseControl,
        list: Option<KindList>,
    ) -> Result<PreparedSlot, SessionError> {
        self.prepare_payload_slot_with(control, list, false)
    }
    /// As `prepare_payload_slot_for`, also admitting the entry of the
    /// explicit-form list when the row is of a listed control with a `form`
    /// attribute. All reservations precede the one acquisition.
    #[cfg_attr(test, track_caller)]
    fn prepare_payload_slot_with(
        &mut self,
        control: &mut ParseControl,
        list: Option<KindList>,
        explicit_form: bool,
    ) -> Result<PreparedSlot, SessionError> {
        control.require_identity(&self.control)?;
        control.reserve_vector_geometric(
            &mut self.payloads.values,
            &mut self.payloads.allocation,
            1,
        )?;
        if explicit_form {
            if self.payloads.explicit_forms.is_none() {
                let (slots, allocation) = control.new_vector()?;
                self.payloads.explicit_forms = Some(KindSlots { slots, allocation });
            }
            let Some(KindSlots { slots, allocation }) = self.payloads.explicit_forms.as_mut()
            else {
                return Err(latch_registry_row(control));
            };
            control.reserve_vector_geometric(slots, allocation, 1)?;
            control.admit_atomic_layout(mem::size_of::<usize>())?;
            control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
            control.charge(WorkCategory::RecordWrite, 1)?;
        }
        if let Some(list) = list {
            let Some(KindSlots { slots, allocation }) =
                self.payloads.kind_slots.get_mut(list as usize)
            else {
                return Err(latch_registry_row(control));
            };
            control.reserve_vector_geometric(slots, allocation, 1)?;
            control.admit_atomic_layout(mem::size_of::<usize>())?;
            control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
            control.charge(WorkCategory::RecordWrite, 1)?;
        }
        control.admit_atomic_layout(mem::size_of::<RetainedPayload>())?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<RetainedPayload>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        let entries = 1u64
            .saturating_add(u64::from(list.is_some()))
            .saturating_add(u64::from(explicit_form));
        let prepared_entries = self
            .payloads
            .prepared_entries
            .checked_add(entries)
            .ok_or_else(|| latch_registry_row(control))?;
        control.acquire(HtmlParseResource::AuxiliaryEntries, entries)?;
        self.payloads.prepared_entries = prepared_entries;
        #[cfg(test)]
        control.note_prepared_slot(std::panic::Location::caller());
        Ok(PreparedSlot {
            entries,
            list,
            explicit_form,
        })
    }
    fn register_raw(
        &mut self,
        control: &mut ParseControl,
        raw: FrozenUtf16,
    ) -> Result<usize, SessionError> {
        let origins = if let Some(evidence) = &mut self.evidence {
            match evidence.rule_group(control, raw.text().len()) {
                Ok(group) => Some(group),
                Err(error) => {
                    raw.release(control)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        let prepared = match self.prepare_payload_slot_owned(control) {
            Ok(prepared) => prepared,
            Err(error) => {
                raw.release(control)?;
                return Err(error);
            }
        };
        let (value, allocation) = raw.into_parts();
        let index = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location: StringLocation::Detached,
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(latch_registry_row(control));
        };
        control.move_document_string_to_managed(allocation, value)?;
        if let Some(origins) = origins {
            self.evidence
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .bind_leaf_value(control, origins, index)?;
        }
        Ok(index)
    }

    /// Actual blank creation followed by source-order native attribute appends.
    fn allocate_ordered_payload(
        &mut self,
        control: &mut ParseControl,
        name: Text,
        mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
        prepared: Option<template::PreparedDocuments>,
        creation: OrderedCreation,
    ) -> Result<ParserNode, SessionError> {
        let result = (|| {
            if let Err(error) =
                payload_write::<OwnedVec<crate::parser::tokenizer::token::Attribute>>(control, 1)
            {
                name.release(control)?;
                return Err(error);
            }
            let empty = match OwnedVec::new(HtmlParseResource::TokenAttributes) {
                Ok(value) => value,
                Err(stop) => {
                    name.release(control)?;
                    return Err(stop.into());
                }
            };
            let node = self.allocate_html_payload_prepared(
                control,
                name.value,
                Some(name.origins),
                empty,
                prepared,
            )?;
            match creation {
                OrderedCreation::Details { metadata } => {
                    payload_write::<Option<usize>>(control, 1)?;
                    let Some(RetainedPayload::Details { node: bound, .. }) =
                        self.payloads.values.get_mut(metadata)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                }
                OrderedCreation::Input {
                    input,
                    association,
                    source,
                } => {
                    payload_write::<[Option<usize>; 3]>(control, 3)?;
                    let Some(RetainedPayload::InputSource { node: bound, .. }) =
                        self.payloads.values.get_mut(source)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    let Some(RetainedPayload::Input { node: bound, .. }) =
                        self.payloads.values.get_mut(input)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(input)
                        .map_err(|_| latch_registry_row(control))?;
                    let Some(RetainedPayload::FormAssociation { node: bound, .. }) =
                        self.payloads.values.get_mut(association)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(association)
                        .map_err(|_| latch_registry_row(control))?;
                }
                OrderedCreation::Select {
                    metadata: Some(slot),
                } => {
                    payload_write::<Option<usize>>(control, 1)?;
                    let Some(RetainedPayload::SelectFamily { node: bound, .. }) =
                        self.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(slot)
                        .map_err(|_| latch_registry_row(control))?;
                }
                OrderedCreation::Select { metadata: None } => {}
            }
            let container = self.element_attribute_container(control, node.index)?;
            let (values, allocation) = control.new_vector()?;
            let prepared = match self.prepare_payload_slot(control) {
                Ok(prepared) => prepared,
                Err(error) => {
                    release_vector(control, values, allocation)?;
                    return Err(error);
                }
            };
            let count = attributes.len();
            let stage = self
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Attributes(Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len: count,
                        reversed_pairs: 0,
                    })),
                )
                .map_err(|_| latch_registry_row(control))?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let mut remaining = count;
            while let Some(attribute) = attributes.try_pop(control)? {
                let (local, local_ticket) = match self.register_text(control, attribute.name) {
                    Ok(value) => value,
                    Err(error) => {
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                };
                let (value, value_ticket) = self.register_text(control, attribute.value)?;
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                }))) = self.payloads.values.get_mut(stage)
                else {
                    return Err(latch_registry_row(control));
                };
                control.reserve_vector_geometric_toward(values, allocation, 1, count)?;
                payload_write::<Attribute>(control, 1)?;
                payload_write::<[StringLocation; 2]>(control, 2)?;
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                values.push(Attribute::new(
                    QualifiedName::new(Namespace::none(), None, local),
                    value,
                ));
                remaining = remaining
                    .checked_sub(1)
                    .ok_or_else(|| latch_registry_row(control))?;
                self.bind_string(
                    local_ticket,
                    StringLocation::AttributeName(stage, remaining),
                )
                .map_err(|_| latch_registry_row(control))?;
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(stage, remaining),
                )
                .map_err(|_| latch_registry_row(control))?;
            }
            self.edit_attributes(control, node.index, container, stage, count, false)?;
            self.begin_attribute_transfer(control, stage, container, 0)?;
            for position in 0..count {
                self.edit_attributes(control, node.index, container, stage, 1, true)?;
                if self.effects.form.is_some() {
                    self.drain_form_effects(control)?;
                }
                self.initial_nonce_attribute(control, node.index, position)?;
                match creation {
                    OrderedCreation::Details { .. } => {
                        self.initial_details_attribute(control, node.index, position)?
                    }
                    OrderedCreation::Input {
                        input, association, ..
                    } => self.initial_input_attribute(
                        control,
                        node.index,
                        input,
                        association,
                        (container, position),
                        InputAttributeInitialization::Installed,
                    )?,
                    OrderedCreation::Select { .. } => {
                        self.initial_select_attribute(control, node.index, position)?
                    }
                }
            }
            // Close the real source redirect after every installed slot/effect.
            payload_write::<PreparedAttributeMerge>(control, 1)?;
            self.finish_attribute_merge(
                control,
                PreparedAttributeMerge {
                    target: node.index,
                    container,
                    stage,
                    start: 0,
                    count: 0,
                    earlier: [None; 4],
                },
            )?;

            match creation {
                OrderedCreation::Details { .. } => {}
                OrderedCreation::Input { input, .. } => {
                    self.reset_input(control, node.index, input)?
                }
                OrderedCreation::Select { .. } => self.initial_select_reset(control, node.index)?,
            }
            Ok(node)
        })();
        let cleanup = release_token_attributes(attributes, control);
        result.and_then(|node| cleanup.map(|()| node))
    }

    fn input_raw_slot(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<usize, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::Input { raw, .. }) = self.payloads.values.get(slot) else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
        Ok(*raw)
    }
    fn read_input_state(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<InputState, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<InputState>() as u64)?;
        let Some(RetainedPayload::Input { state, .. }) = self.payloads.values.get(slot) else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        Ok(*state)
    }
    fn write_input_state(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        next: InputState,
    ) -> Result<(), SessionError> {
        payload_write::<InputState>(control, 1)?;
        let Some(RetainedPayload::Input { state, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        *state = next;
        Ok(())
    }
    fn replace_input_raw(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        text: FieldCopy,
    ) -> Result<(), SessionError> {
        let previous = match self.input_raw_slot(control, slot) {
            Ok(previous) => previous,
            Err(error) => {
                text.release(control)?;
                return Err(error);
            }
        };
        // Both fresh initialization and clone initialization already run inside
        // the same qualified owner. A clone's retained construction frame is
        // legitimate useful work, so do not reenter the external idle gate.
        let dispose_previous = match self.releasable_string(control, previous, false) {
            Ok(value) => value,
            Err(error) => {
                text.release(control)?;
                return Err(error);
            }
        };
        let (value, next) = self.register_field_copy(control, text)?;
        drop(value); // no native alias: actual independent raw owner only
        payload_write::<usize>(control, 1)?;
        payload_write::<RetainedPayload>(control, 2)?;
        self.retire_field_group(control, previous)?;
        let Some(RetainedPayload::Input { raw, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(control));
        };
        *raw = next;
        if dispose_previous {
            self.dispose_aggregate_string(control, previous)?;
        }
        Ok(())
    }
    fn attribute_value_slot(
        &self,
        control: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<usize, SessionError> {
        control.charge(WorkCategory::RecordVisit, 2)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let attributes = element.attributes().as_slice();
        let value = attributes
            .get(position)
            .ok_or(SessionError::AttributeStorageUnavailable)?
            .value();
        // Valid-or-fallback hint: creation registered this element's attribute
        // names and values as adjacent registry rows, last attribute first. The
        // candidate row is used only after its identity matches the attribute
        // value; any mismatch, or a hint cleared by an attribute edit, falls
        // back to the scan below, which returns the same row.
        if let Some(candidate) =
            self.payloads
                .attribute_value_candidate(node, attributes.len(), position)
        {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::String { value: stored, .. }) =
                self.payloads.values.get(candidate)
            {
                control.charge(WorkCategory::ProviderCall, 1)?;
                if Arc::ptr_eq(value.shared_code_units(), stored.shared_code_units()) {
                    #[cfg(test)]
                    assert_eq!(
                        self.payloads.first_string_sharing(value),
                        Some(candidate),
                        "a verified hint names the row a scan finds"
                    );
                    return Ok(candidate);
                }
            }
        }
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::String {
                value: candidate, ..
            } = payload
            {
                control.charge(WorkCategory::ProviderCall, 1)?;
                if Arc::ptr_eq(value.shared_code_units(), candidate.shared_code_units()) {
                    return Ok(slot);
                }
            }
        }
        Err(SessionError::AttributeStorageUnavailable)
    }
    fn install_sanitized_input_value(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        source: usize,
        policy: crate::HtmlInputValueSanitization,
    ) -> Result<(), SessionError> {
        let policy = match policy {
            crate::HtmlInputValueSanitization::String(policy) => policy,
            crate::HtmlInputValueSanitization::Range => {
                return self.install_range_input_value(control, node, slot, source);
            }
            crate::HtmlInputValueSanitization::Color => {
                return self.update_input_color(control, node, slot);
            }
        };
        if matches!(
            policy,
            crate::HtmlInputSanitization::Email
                | crate::HtmlInputSanitization::StripCrLfAndTrim
                | crate::HtmlInputSanitization::Temporal(_)
                | crate::HtmlInputSanitization::Number
        ) {
            let mode = if policy == crate::HtmlInputSanitization::Email
                && self.form_attribute(control, node, "multiple")?.is_some()
            {
                crate::HtmlInputEmailMode::Multiple
            } else {
                crate::HtmlInputEmailMode::Single
            };
            let (field, count) = copy_selected_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                source,
                policy,
                mode,
            )?;
            self.replace_input_raw(control, slot, field)?;
            let mut state = self.read_input_state(control, slot)?;
            state.email_value_count = count;
            self.write_input_state(control, slot, state)?;
        } else {
            let field = if policy == crate::HtmlInputSanitization::Preserve {
                copy_preserved_input_value(&self.payloads, self.evidence.as_ref(), control, source)?
            } else {
                copy_input_value(
                    &self.payloads,
                    self.evidence.as_ref(),
                    control,
                    source,
                    policy,
                )?
            };
            self.replace_input_raw(control, slot, field)?;
        }
        Ok(())
    }
    fn initial_input_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        association: usize,
        attribute: (usize, usize),
        initialization: InputAttributeInitialization,
    ) -> Result<(), SessionError> {
        let (container, position) = attribute;
        control.charge(WorkCategory::RecordVisit, 3)?;
        let named = element_attribute(&self.construction, control, node, position)?;
        let name = named.name().local_name();
        let value_attr = exact_ascii(control, Some(name), "value")?;
        let type_attr = exact_ascii(control, Some(name), "type")?;
        let checked_attr = exact_ascii(control, Some(name), "checked")?;
        let form_attr = exact_ascii(control, Some(name), "form")?;
        let name_attr =
            named.name().namespace().uri().is_none() && exact_ascii(control, Some(name), "name")?;
        let state = self.read_input_state(control, slot)?;
        let src_attr = state.flags.input_type() == crate::HtmlInputType::Image
            && named.name().namespace().uri().is_none()
            && exact_ascii(control, Some(name), "src")?;
        let multiple_attr = state.flags.input_type() == crate::HtmlInputType::Email
            && exact_ascii(control, Some(name), "multiple")?;
        if value_attr
            && initialization != InputAttributeInitialization::ClonedColor
            && let crate::HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(policy) =
                state.flags.value_attribute_changed()
        {
            let source = self.attribute_value_slot(control, node, position)?;
            self.install_sanitized_input_value(control, node, slot, source, policy)?;
        }
        if checked_attr
            && let Some(flags) = state
                .flags
                .checked_attribute_changed(crate::HtmlInputCheckedAttributeChange::Added)
        {
            let mut next = state;
            next.flags = flags;
            self.write_input_state(control, slot, next)?;
            if flags.radio_normalization_required(crate::HtmlRadioTrigger::Checkedness) {
                self.normalize_radio(control, node, None)?;
            }
        }
        if type_attr {
            let attribute = element_attribute(&self.construction, control, node, position)?;
            let next_kind = input_type_state(control, attribute.value())?;
            self.change_input_type(control, node, slot, container, state, next_kind)?;
        }
        if src_attr {
            self.image_source_changed(control, node, slot)?;
        }
        if name_attr
            && state
                .flags
                .radio_normalization_required(crate::HtmlRadioTrigger::Name)
        {
            self.normalize_radio(control, node, None)?;
        }
        if state.flags.input_type() == crate::HtmlInputType::Email && multiple_attr {
            let raw = self.input_raw_slot(control, slot)?;
            self.install_sanitized_input_value(
                control,
                node,
                slot,
                raw,
                crate::HtmlInputValueSanitization::String(crate::HtmlInputSanitization::Email),
            )?;
        }
        if state.flags.input_type() == crate::HtmlInputType::Range {
            let attribute = element_attribute(&self.construction, control, node, position)?;
            let relevant = attribute.name().namespace().uri().is_none()
                && (exact_ascii(control, Some(attribute.name().local_name()), "min")?
                    || exact_ascii(control, Some(attribute.name().local_name()), "max")?
                    || exact_ascii(control, Some(attribute.name().local_name()), "step")?);
            if relevant {
                let raw = self.input_raw_slot(control, slot)?;
                self.install_range_input_value(control, node, slot, raw)?;
            }
        }
        if state.flags.input_type() == crate::HtmlInputType::Color
            && initialization != InputAttributeInitialization::ClonedColor
        {
            let attribute = element_attribute(&self.construction, control, node, position)?;
            let relevant = attribute.name().namespace().uri().is_none()
                && (exact_ascii(control, Some(attribute.name().local_name()), "alpha")?
                    || exact_ascii(control, Some(attribute.name().local_name()), "colorspace")?);
            if relevant {
                self.update_input_color(control, node, slot)?;
            }
        }
        if form_attr {
            let state = find_form_association_state(&self.payloads, control, node)?
                .ok_or(SessionError::FormAssociationMetadataUnavailable)?
                .1;
            if form_owner_reset_trigger(HtmlFormOwnerResetTrigger::FormAttributeChanged {
                listed: state.kind.listed(),
            }) == HtmlFormOwnerResetDecision::Reset
            {
                self.reset_form_association(control, association, node, state, None)?;
            }
        }
        Ok(())
    }
    // The same actual state transition is consumed by ordered creation and
    // managed type removal/change. Storage and source effects stay here.
    fn change_input_type(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        container: usize,
        state: InputState,
        next_kind: crate::HtmlInputType,
    ) -> Result<(), SessionError> {
        if next_kind != state.flags.input_type() {
            let mut next = state;
            next.flags = state.flags.type_changed(next_kind);
            if next_kind != crate::HtmlInputType::Email {
                next.email_value_count = None;
            }
            self.write_input_state(control, slot, next)?;
            let raw = self.input_raw_slot(control, slot)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let current_len = textarea_raw(&self.payloads, control, raw)?.len();
            let mut change =
                crate::HtmlInputTypeChange::new(state.flags.input_type(), next_kind, current_len);
            while let Some(effect) = change.next_effect() {
                if effect == crate::HtmlInputTypeEffect::SetContentValueFromCurrent {
                    let mut value_position = None;
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    let element = element_record(&self.construction, control, node)?;
                    for (index, attribute) in element.attributes().as_slice().iter().enumerate() {
                        control.charge(WorkCategory::RecordVisit, 1)?;
                        if exact_ascii(control, Some(attribute.name().local_name()), "value")? {
                            value_position = Some(index);
                            break;
                        }
                    }
                    let generated = matches!(
                        state.flags.input_type(),
                        crate::HtmlInputType::Range | crate::HtmlInputType::Color
                    );
                    let position = match value_position {
                        Some(position) => {
                            self.writeback_input_value(
                                control, node, slot, container, position, raw,
                            )?;
                            position
                        }
                        None => self.append_input_value_from_current(
                            control, node, slot, container, raw, generated,
                        )?,
                    };
                    // Nested dirty=false value change in Default or DefaultOn:
                    // copy the actual replaced native field without a sanitizer.
                    let source = self.attribute_value_slot(control, node, position)?;
                    let text = if generated {
                        provenance::copy_generated_input_value(
                            &self.payloads,
                            self.evidence.as_ref(),
                            control,
                            source,
                        )?
                    } else {
                        copy_input_value(
                            &self.payloads,
                            self.evidence.as_ref(),
                            control,
                            source,
                            crate::HtmlInputSanitization::Preserve,
                        )?
                    };
                    self.replace_input_raw(control, slot, text)?;
                }
                if effect == crate::HtmlInputTypeEffect::SetCurrentFromDefaultAndClearDirty {
                    let mut source = None;
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    let element = element_record(&self.construction, control, node)?;
                    for (position, attribute) in element.attributes().as_slice().iter().enumerate()
                    {
                        control.charge(WorkCategory::RecordVisit, 2)?;
                        if attribute.name().namespace().uri().is_none()
                            && exact_ascii(control, Some(attribute.name().local_name()), "value")?
                        {
                            source = Some(self.attribute_value_slot(control, node, position)?);
                            break;
                        }
                    }
                    let field = match source {
                        Some(source) => copy_preserved_input_value(
                            &self.payloads,
                            self.evidence.as_ref(),
                            control,
                            source,
                        )?,
                        None => FieldCopy {
                            value: literal_string(control, b"")?,
                            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                            original: None,
                        },
                    };
                    self.replace_input_raw(control, slot, field)?;
                    let mut current = self.read_input_state(control, slot)?;
                    current.flags = current.flags.type_default_installed();
                    self.write_input_state(control, slot, current)?;
                }
                if effect == crate::HtmlInputTypeEffect::SetCurrentEmptyForFilename {
                    let field = FieldCopy {
                        value: literal_string(control, b"")?,
                        origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                        original: None,
                    };
                    self.replace_input_raw(control, slot, field)?;
                }
                if let crate::HtmlInputTypeEffect::Sanitize(policy) = effect
                    && policy
                        != crate::HtmlInputValueSanitization::String(
                            crate::HtmlInputSanitization::Preserve,
                        )
                {
                    let raw = self.input_raw_slot(control, slot)?;
                    self.install_sanitized_input_value(control, node, slot, raw, policy)?;
                }
                if effect == crate::HtmlInputTypeEffect::SignalTypeChange
                    && next_kind == crate::HtmlInputType::Radio
                {
                    self.normalize_radio(control, node, None)?;
                }
                // Rendering has no host recipient in
                // fresh immutable construction. Initial cursor is already0.
            }
            if next_kind == crate::HtmlInputType::Image {
                self.image_type_entered(control, node, slot)?;
            }
        }
        Ok(())
    }
    fn append_input_value_from_current(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        container: usize,
        raw: usize,
        generated: bool,
    ) -> Result<usize, SessionError> {
        // "value" is a source-required generated name, not a tokenizer field.
        // The current cause is the actual type transition; no lexical atom is
        // invented. The value separately retains its real prior group.
        let name = FieldCopy {
            value: literal_string(control, b"value")?,
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
            original: None,
        };
        let (name, name_slot) = self.register_field_copy(control, name)?;
        let value = if generated {
            provenance::copy_generated_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                raw,
            )?
        } else {
            copy_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                raw,
                crate::HtmlInputSanitization::Preserve,
            )?
        };
        let (value, value_slot) = self.register_field_copy(control, value)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let position = element.attributes().len();
        let mut state = self.read_input_state(control, slot)?;
        state.value_rewritten_during_creation = true;
        payload_write::<InputState>(control, 1)?;
        payload_write::<Attribute>(control, 1)?;
        let incoming = Attribute::new(QualifiedName::new(Namespace::none(), None, name), value);
        self.append_registered_attribute(
            control,
            node,
            container,
            incoming,
            [name_slot, value_slot],
        )?;
        let Some(RetainedPayload::Input {
            state: destination, ..
        }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        *destination = state;
        Ok(position)
    }
    fn writeback_input_value(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        container: usize,
        position: usize,
        raw: usize,
    ) -> Result<(), SessionError> {
        let old_slot = self.attribute_value_slot(control, node, position)?;
        let text =
            copy_preserved_input_value(&self.payloads, self.evidence.as_ref(), control, raw)?;
        let (value, new_slot) = self.register_field_copy(control, text)?;
        (|| {
            control.charge(WorkCategory::RecordVisit, 2)?;
            control.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<QualifiedName>(control, 1)?;
            let attribute = element_attribute(&self.construction, control, node, position)?;
            payload_write::<Attribute>(control, 1)?;
            payload_write::<Option<Attribute>>(control, 1)?;
            let mut incoming = Some(Attribute::new(attribute.name().clone(), value));
            // Registry redirects and known shared-wrapper restoration are
            // preadmitted. The actual slot producer then commits all or none.
            payload_write::<[StringLocation; 2]>(control, 2)?;
            let mut next_state = self.read_input_state(control, slot)?;
            next_state.value_rewritten_during_creation = true;
            payload_write::<InputState>(control, 1)?;
            payload_write::<[Option<AttributeStorage>; 2]>(control, 2)?;
            control.charge(WorkCategory::ProviderCall, 2)?;
            payload_write::<OrderedAttributes>(control, 1)?;
            let result = self.replace_attribute_storage(
                control,
                node,
                container,
                position,
                &mut incoming,
                &[Some(old_slot)],
            );
            match result {
                Ok(old) => {
                    self.bind_string(old_slot, StringLocation::Detached)
                        .map_err(|_| latch_registry_row(control))?;
                    self.bind_string(
                        new_slot,
                        StringLocation::AttributeValue(container, position),
                    )
                    .map_err(|_| latch_registry_row(control))?;
                    let Some(RetainedPayload::Input { state, .. }) =
                        self.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *state = next_state;
                    drop(old);
                    Ok(())
                }
                Err(stop) => {
                    drop(incoming);
                    Err(stop)
                }
            }
        })()
    }
    fn reset_input(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
    ) -> Result<(), SessionError> {
        let mut state = self.read_input_state(control, slot)?;
        // Only fresh construction/clone initialization reaches this adapter.
        // Checked presence is already installed, files are already empty and
        // no partial input is observable before complete=true. Default copy
        // and sanitization may therefore fuse; generic reset remains ordered.
        state.flags = state.flags.reset_begin();
        self.write_input_state(control, slot, state)?;
        let mut checked = false;
        let mut value = None;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let element = element_record(&self.construction, control, node)?;
        for (position, attribute) in element.attributes().as_slice().iter().enumerate() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(attribute.name().local_name()), "checked")? {
                checked = true;
            }
            if exact_ascii(control, Some(attribute.name().local_name()), "value")? {
                value = Some(position);
            }
        }
        if let Some(position) = value {
            let source = self.attribute_value_slot(control, node, position)?;
            self.install_sanitized_input_value(
                control,
                node,
                slot,
                source,
                state.flags.input_type().sanitization(),
            )?;
        } else {
            let field = FieldCopy {
                value: literal_string(control, b"")?,
                origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                original: None,
            };
            self.replace_input_raw(control, slot, field)?;
            if state.flags.input_type() == crate::HtmlInputType::Range {
                let raw = self.input_raw_slot(control, slot)?;
                self.install_range_input_value(control, node, slot, raw)?;
            }
            if state.flags.input_type() == crate::HtmlInputType::Color {
                self.update_input_color(control, node, slot)?;
            }
            if state.flags.input_type() == crate::HtmlInputType::Email {
                state.email_value_count =
                    if self.form_attribute(control, node, "multiple")?.is_some() {
                        Some(0)
                    } else {
                        None
                    };
                self.write_input_state(control, slot, state)?;
            }
        }
        if matches!(
            state.flags.input_type(),
            crate::HtmlInputType::Email | crate::HtmlInputType::Url
        ) {
            state = self.read_input_state(control, slot)?;
        }
        state.flags = state.flags.reset_checkedness(checked);
        self.write_input_state(control, slot, state)?;
        if state
            .flags
            .radio_normalization_required(crate::HtmlRadioTrigger::Checkedness)
        {
            self.normalize_radio(control, node, None)?;
        }
        state.selected_files_empty = true;
        self.write_input_state(control, slot, state)?;
        payload_write::<bool>(control, 1)?;
        let Some(RetainedPayload::Input { complete, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        *complete = true;
        #[cfg(test)]
        if let Some(cancellation) = self.cancel_after_input_reset.take() {
            cancellation.cancel();
        }
        Ok(())
    }

    fn textarea_slot(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<Option<(usize, TextareaState)>, SessionError> {
        find_textarea_state(&self.payloads, control, node)
    }

    /// Child text is direct Text children in the actual source view. A moved
    /// root is excluded for removal, even if the fixed final links reinsert it.
    fn refresh_textarea(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        reset: bool,
    ) -> Result<(), SessionError> {
        let Some((slot, mut state)) = self.textarea_slot(control, node)? else {
            return Ok(());
        };
        let old = textarea_raw(&self.payloads, control, state.raw)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        if Arc::strong_count(old.shared_code_units()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        if reset {
            payload_write::<TextareaState>(control, 1)?;
            state.flags.reset();
            let Some(RetainedPayload::Textarea { state: stored, .. }) =
                self.payloads.values.get_mut(slot)
            else {
                return Err(latch_registry_row(control));
            };
            *stored = state;
        } else if state.flags.children_changed() == HtmlTextareaRawValueAction::Keep {
            return Ok(());
        }
        let dispose_previous = self.releasable_string(control, state.raw, false)?;
        let mut raw = Utf16Buffer::new(control)?;
        let mut group = None;
        let mut units = 0usize;
        let copied = (|| {
            control.charge(WorkCategory::RecordVisit, 1)?;
            for position in 0..self
                .construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(control))?
                .child_count()
            {
                control.charge(WorkCategory::RecordVisit, 2)?;
                let child = self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_at(position)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                if Some(child) != detached
                    && let HtmlNodeKind::Text(value) = &self
                        .construction
                        .node_at(child)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                {
                    raw.extend_units(value.code_units(), control)?;
                    let added = value.len();
                    let end = units.checked_add(added).ok_or_else(|| {
                        control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                    })?;
                    let source = self.field_slot(control, StringLocation::Text(child))?;
                    let mut incoming = self.copied_slot_group(control, source)?;
                    if incoming.is_none() && self.evidence.is_some() {
                        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
                        let fresh = evidence.source_independent_group(control, added)?;
                        evidence.bind_leaf_value(control, &fresh, source)?;
                        incoming = Some(fresh);
                    }
                    if let Some(incoming) = incoming {
                        group = Some(if let Some(previous) = group.take() {
                            self.evidence
                                .as_mut()
                                .ok_or(SessionError::WrongToken)?
                                .merge_groups(control, &previous, &incoming, units, end)?
                        } else {
                            incoming
                        });
                    }
                    units = end;
                }
            }
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = copied {
            raw.release(control)?;
            return Err(error);
        }
        let group = match self.reset_group(control, state.raw, group, units) {
            Ok(group) => group,
            Err(error) => {
                raw.release(control)?;
                return Err(error);
            }
        };
        let raw = raw.freeze(control)?;
        let (native, next) = self.register_group_string_owned(control, raw, group)?;
        drop(native);
        // All useful work and both actual record moves precede a fixed commit.
        // The old raw has no native/metadata Arc alias: metadata holds an index.
        payload_write::<TextareaState>(control, 1)?;
        payload_write::<RetainedPayload>(control, 2)?;
        self.retire_field_group(control, state.raw)?;
        let Some(RetainedPayload::Textarea { state: stored, .. }) =
            self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        let previous = state.raw;
        state.raw = next;
        *stored = state;
        if dispose_previous {
            self.dispose_aggregate_string(control, previous)?;
        }
        Ok(())
    }
    fn check_text(control: &mut ParseControl, text: &mut Text) -> Result<(), SessionError> {
        control.require_document_string(text.value.allocation(), text.value.text())?;
        text.origins.require_owner(control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        if Arc::strong_count(text.value.text().shared_code_units()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        Ok(())
    }
    fn register_string(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<TextOrigin>>,
    ) -> Result<(DocumentString, usize), SessionError> {
        if let Err(error) = self.require(control) {
            value.release(control)?;
            if let Some(origins) = origins {
                origins.release(control)?;
            }
            return Err(error);
        }
        self.register_string_owned(control, value, origins)
    }
    fn register_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<TextOrigin>>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let origins = if let Some(origins) = origins {
            let evidence = match self.origin_evidence(control) {
                Ok(evidence) => evidence,
                Err(error) => {
                    origins.release(control)?;
                    value.release(control)?;
                    return Err(error);
                }
            };
            match evidence.retain_group(control, origins, value.text().len()) {
                Ok(group) => Some(group),
                Err(error) => {
                    value.release(control)?;
                    return Err(error);
                }
            }
        } else if let Some(evidence) = &mut self.evidence {
            match evidence.rule_group(control, value.text().len()) {
                Ok(group) => Some(group),
                Err(error) => {
                    value.release(control)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        self.register_group_string_owned(control, value, origins)
    }
    /// The group is committed in the sole evidence owner before the actual
    /// native backing slot is reserved. A refused slot leaves no logical slot
    /// reservation and the original backing is reclaimed by this owner.
    fn register_group_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OriginGroupRef>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let prepared = match self.prepare_payload_slot_owned(control) {
            Ok(prepared) => prepared,
            Err(error) => {
                value.release(control)?;
                return Err(error);
            }
        };
        let (value, allocation) = value.into_parts();
        let index = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location: StringLocation::Detached,
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(latch_registry_row(control));
        };
        control.move_document_string_to_managed(allocation, value)?;
        if let Some(origins) = origins {
            self.evidence
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .bind_leaf_value(control, origins, index)?;
        }
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<DocumentString>() as u64,
        )?;
        Ok((value.clone(), index))
    }
    fn register_text(
        &mut self,
        control: &mut ParseControl,
        text: Text,
    ) -> Result<(DocumentString, usize), SessionError> {
        self.register_string(control, text.value, Some(text.origins))
    }
    fn allocate_html_tag(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        Self::check_html_tag(control, tag)?;
        let Some(Tag {
            name, attributes, ..
        }) = input.take()
        else {
            return Err(SessionError::Consumed);
        };
        self.allocate_html_payload(control, name.value, Some(name.origins), attributes)
    }
    /// The existing generic borrowed validation, shared without transferring
    /// fields or reserving parser-created metadata.
    fn check_html_tag(control: &mut ParseControl, tag: &mut Tag) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        Self::check_text(control, &mut tag.name)?;
        lowercase_token_name(control, tag.name.value.text())?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(attribute) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            Self::check_text(control, &mut attribute.name)?;
            Self::check_text(control, &mut attribute.value)?;
            lowercase_token_name(control, attribute.name.value.text())?;
        }
        Ok(())
    }
    /// Shared real storage transfer for explicit and source-implied elements.
    /// An implied name has no fabricated token or source-origin vector.
    fn allocate_html_payload(
        &mut self,
        control: &mut ParseControl,
        name: FrozenUtf16,
        origins: Option<OwnedVec<TextOrigin>>,
        attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
    ) -> Result<ParserNode, SessionError> {
        let preparation = (|| {
            let template = exact_ascii(control, Some(name.text()), "template")?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let document = matches!(
                self.construction
                    .node_at(0)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                HtmlNodeKind::Document(_)
            )
            .then_some(DocumentRef::Owned(0));
            let ordinary_document = self.fragment.is_none() && document.is_some();
            let mut prepared = self.prepare_document_allocation(control, document, template)?;
            if ordinary_document {
                let mut is_value = None;
                for attribute in attributes.as_slice() {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    if exact_ascii(control, Some(attribute.name.value.text()), "is")? {
                        is_value = Some(&attribute.value);
                        break;
                    }
                }
                // This lexical helper creates in the actual ordinary root
                // Document. Its original token, not later merged attributes,
                // supplies the independent saved-is creation value.
                self.prepare_creation_registry(
                    control,
                    0,
                    name.text(),
                    is_value,
                    true,
                    &mut prepared,
                )?;
            }
            Ok(prepared)
        })();
        let prepared = match preparation {
            Ok(p) => p,
            Err(error) => {
                name.release(control)?;
                if let Some(origins) = origins {
                    origins.release(control)?;
                }
                release_token_attributes(attributes, control)?;
                return Err(error);
            }
        };
        self.allocate_html_payload_prepared(control, name, origins, attributes, prepared)
    }
    fn allocate_html_payload_prepared(
        &mut self,
        control: &mut ParseControl,
        name: FrozenUtf16,
        origins: Option<OwnedVec<TextOrigin>>,
        mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
        prepared: Option<template::PreparedDocuments>,
    ) -> Result<ParserNode, SessionError> {
        let result = (|| {
            let (name, name_ticket) = self.register_string(control, name, origins)?;
            // The selected HTML namespace is a defining literal, copied through
            // the same admitted exact buffer as token-created units.
            let namespace = literal_string(control, b"http://www.w3.org/1999/xhtml")?;
            let (namespace, namespace_ticket) = self.register_string(control, namespace, None)?;
            control.charge(WorkCategory::RecordVisit, 2)?;
            let mut units = name
                .len()
                .checked_add(namespace.len())
                .ok_or_else(|| control.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
            let (values, allocation) = control.new_vector()?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let original_len = attributes.len();
            let admission = match self.prepare_payload_slot(control) {
                Ok(admission) => admission,
                Err(error) => {
                    release_vector(control, values, allocation)?;
                    return Err(error);
                }
            };
            let container = self
                .payloads
                .push_row(
                    admission,
                    RetainedPayload::Attributes(Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len,
                        reversed_pairs: 0,
                    })),
                )
                .map_err(|_| latch_registry_row(control))?;
            // Names and values register as adjacent rows, last attribute first.
            let mut last_name_slot = None;
            while let Some(attribute) = attributes.try_pop(control)? {
                let attribute_name = self.register_text(control, attribute.name);
                let (local, local_ticket) = match attribute_name {
                    Ok(value) => value,
                    Err(error) => {
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                };
                last_name_slot.get_or_insert(local_ticket);
                let (value, value_ticket) = self.register_text(control, attribute.value)?;
                control.charge(WorkCategory::RecordVisit, 2)?;
                units = units
                    .checked_add(local.len())
                    .and_then(|sum| sum.checked_add(value.len()))
                    .ok_or_else(|| {
                        control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                    })?;
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                }))) = self.payloads.values.get_mut(container)
                else {
                    return Err(latch_registry_row(control));
                };
                control.reserve_vector_geometric_toward(values, allocation, 1, original_len)?;
                control.charge(WorkCategory::CopyByte, mem::size_of::<Attribute>() as u64)?;
                control.charge(WorkCategory::RecordWrite, 3)?;
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                let position = attributes.len();
                values.push(Attribute::new(
                    QualifiedName::new(Namespace::none(), None, local),
                    value,
                ));
                self.bind_string(
                    local_ticket,
                    StringLocation::AttributeName(container, position),
                )
                .map_err(|_| latch_registry_row(control))?;
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(container, position),
                )
                .map_err(|_| latch_registry_row(control))?;
            }
            self.reverse_attributes(control, container)?;
            let Some(RetainedPayload::Attributes(storage)) =
                self.payloads.values.get_mut(container)
            else {
                return Err(latch_registry_row(control));
            };
            let taken = storage.take();
            let Some(AttributeStorage::Building {
                values,
                allocation,
                original_len,
                reversed_pairs,
            }) = taken
            else {
                *storage = taken;
                return Err(latch_registry_row(control));
            };
            let (backing, shared) = match control.share_owned(values) {
                Ok(value) => value,
                Err((stop, values)) => {
                    *storage = Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len,
                        reversed_pairs,
                    });
                    return Err(stop.into());
                }
            };
            let ordered = match validate_parser_attributes(backing, control) {
                Ok(values) => values,
                Err((error, backing)) => {
                    // Preserve the admitted shared wrapper in the owning phase
                    // even when intrinsic duplicate checking or work stops.
                    let values = match control.release_shared(backing, shared) {
                        Ok(values) => values,
                        Err(refused) => {
                            // Unpublished backing is uniquely owned, so a
                            // refusal is an invariant stop. The refused Arc and
                            // ticket stay in the row for cleanup to retry.
                            let (_, backing, shared) = refused.into_parts();
                            *storage = Some(AttributeStorage::Unvalidated {
                                backing,
                                allocation,
                                shared,
                                original_len,
                                reversed_pairs,
                            });
                            return Err(latch_registry_row(control));
                        }
                    };
                    *storage = Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len,
                        reversed_pairs,
                    });
                    return Err(error);
                }
            };
            *storage = Some(AttributeStorage::Frozen {
                values: ordered,
                allocation,
                shared,
                element: None,
            });
            control.charge(WorkCategory::ProviderCall, 1)?;
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<OrderedAttributes>() as u64,
            )?;
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                values, ..
            }))) = self.payloads.values.get(container)
            else {
                return Err(latch_registry_row(control));
            };
            let attributes = values.clone();
            let template = exact_ascii(control, Some(&name), "template")?;
            let link_flags =
                exact_ascii(control, Some(&name), "link")?.then(crate::HtmlLinkFlags::fresh);
            control.charge(WorkCategory::RecordWrite, 6)?;
            let node = self.allocate_prepared_owned(
                control,
                bounded::HtmlConstructionCommand::CreateElement {
                    link_flags,
                    name: QualifiedName::new(Namespace::from_uri(namespace), None, name),
                    attributes,
                },
                template,
                0,
                0,
                prepared,
            )?;
            self.bind_string(name_ticket, StringLocation::ElementName(node.index))
                .map_err(|_| latch_registry_row(control))?;
            self.bind_string(namespace_ticket, StringLocation::Namespace(node.index))
                .map_err(|_| latch_registry_row(control))?;
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                element, ..
            }))) = self.payloads.values.get_mut(container)
            else {
                return Err(latch_registry_row(control));
            };
            *element = Some(node.index);
            self.payloads
                .join_row(container)
                .map_err(|_| latch_registry_row(control))?;
            if let Some(slot) = last_name_slot {
                self.payloads
                    .record_attribute_strings(node.index, slot)
                    .map_err(|_| latch_registry_row(control))?;
            }
            self.initial_nonce_attributes(control, node.index)?;
            self.initial_associated_state(control, node.index)?;
            self.initial_meta_attributes(control, node.index)?;
            self.association_initial_attributes(control, node.index)?;
            self.register_base(control, node.index, base::ResponseDomain::Unavailable)?;
            Ok(node)
        })();
        release_token_attributes(attributes, control)?;
        result
    }
    /// Implements the pinned character-insertion operation at an already
    /// selected insertion location. Foster-parent selection belongs to the
    /// later tree builder. The returned offset locates the original token text
    /// inside the native text node; retained origins remain exact token facts.
    fn insert_text(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        before: Option<&ParserNode>,
        input: &mut Option<Text>,
    ) -> Result<Option<TextInsertion>, SessionError> {
        let parent_index = self.index(control, parent)?;
        let before_index = before.map(|node| self.index(control, node)).transpose()?;
        self.container(control, parent_index)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(before) = before_index {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if self
                .construction
                .node_at(before)
                .ok_or_else(|| latch_registry_row(control))?
                .parent()
                .map(|node| node.position())
                != Some(parent_index)
            {
                return Err(SessionError::WrongSibling);
            }
        }
        let text = input.as_mut().ok_or(SessionError::Consumed)?;
        Self::check_text(control, text)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        if text.value.text().is_empty()
            || matches!(
                self.construction
                    .node_at(parent_index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                HtmlNodeKind::Document(_)
            )
        {
            let Some(text) = input.take() else {
                return Err(SessionError::Consumed);
            };
            text.release(control)?;
            return Ok(None);
        }
        let mut preceding = None;
        if before_index.is_none() {
            // Appending: the preceding sibling is the last child, which needs
            // no walk over the parent's other children.
            control.charge(WorkCategory::RecordVisit, 1)?;
            let parent = self
                .construction
                .node_at(parent_index)
                .ok_or_else(|| latch_registry_row(control))?;
            if let Some(last) = parent.child_count().checked_sub(1) {
                preceding = Some(
                    parent
                        .child_at(last)
                        .ok_or_else(|| latch_registry_row(control))?
                        .position(),
                );
            }
        } else {
            // Walk back from the last child to the reference sibling, so the
            // work is the children behind it and not the ones in front of it;
            // the entry just before it is the preceding sibling.
            let parent = self
                .construction
                .node_at(parent_index)
                .ok_or_else(|| latch_registry_row(control))?;
            let mut position = parent.child_count();
            loop {
                let Some(found) = position.checked_sub(1) else {
                    return Err(latch_registry_row(control));
                };
                control.charge(WorkCategory::RecordVisit, 1)?;
                let child = parent
                    .child_at(found)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                position = found;
                if Some(child) == before_index {
                    break;
                }
            }
            if let Some(previous) = position.checked_sub(1) {
                control.charge(WorkCategory::RecordVisit, 1)?;
                preceding = Some(
                    parent
                        .child_at(previous)
                        .ok_or_else(|| latch_registry_row(control))?
                        .position(),
                );
            }
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        let preceding = match preceding {
            Some(index) => match self.construction.node_at(index).map(|record| record.kind()) {
                Some(HtmlNodeKind::Text(_)) => Some(index),
                Some(_) => None,
                None => return Err(latch_registry_row(control)),
            },
            None => None,
        };
        // The preceding Text record and the registry row holding its string.
        let preceding = match preceding {
            Some(index) => Some((
                index,
                self.field_slot(control, StringLocation::Text(index))?,
            )),
            None => None,
        };
        let dispose_old = match preceding {
            Some((_, slot)) => self.releasable_string(control, slot, true)?,
            None => false,
        };
        let prepared = if preceding.is_none() {
            let document = match self.node_document_ref(control, parent_index) {
                Ok(document) => Some(document),
                Err(SessionError::NodeDocumentUnavailable)
                    if matches!(
                        self.construction
                            .node_at(0)
                            .ok_or_else(|| latch_registry_row(control))?
                            .kind(),
                        HtmlNodeKind::Fragment
                    ) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            self.prepare_document_allocation(control, document, false)?
        } else {
            None
        };
        let Some(text) = input.take() else {
            return Err(SessionError::Consumed);
        };
        let (value, ticket) = self.register_text(control, text)?;
        if let Some((index, old_slot)) = preceding {
            self.append_registered_text(
                control,
                text_append::RegisteredTextAppend {
                    parent: parent_index,
                    index,
                    old_slot,
                    incoming_slot: ticket,
                    incoming: value,
                    dispose_old,
                },
            )
            .map(Some)
        } else {
            control.charge(WorkCategory::RecordWrite, 2)?;
            payload_write::<Option<usize>>(control, 1)?;
            let node = self.allocate_prepared_owned(
                control,
                creation_command(HtmlNodeKind::Text(value)),
                false,
                0,
                0,
                prepared,
            )?;
            self.bind_string(ticket, StringLocation::Text(node.index))
                .map_err(|_| latch_registry_row(control))?;
            self.insert(control, parent, &node, before, InsertionCause::NonParser)?;
            Ok(Some(TextInsertion {
                _node: node,
                _offset: 0,
            }))
        }
    }
    fn releasable_string(
        &self,
        control: &mut ParseControl,
        slot: usize,
        native: bool,
    ) -> Result<bool, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get(slot)
        else {
            return Err(latch_registry_row(control));
        };
        control.require_document_string(allocation, value)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        if Arc::strong_count(value.shared_code_units()) != if native { 2 } else { 1 } {
            return Err(SessionError::AliasedPayload);
        }
        if let Some(group) = origins {
            let evidence = self.evidence.as_ref().ok_or(SessionError::WrongToken)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.charge(
                WorkCategory::InspectByte,
                mem::size_of::<provenance::OriginGroup>() as u64,
            )?;
            let group = evidence.require_group(group)?;
            Ok(!matches!(group.body, provenance::GroupBody::Leaf { .. }) || group.units == 0)
        } else {
            Ok(true)
        }
    }
    /// Called only after the admitted native/raw slot commit has removed every
    /// live alias. Leaf backings remain owned; aggregate history retains IDs,
    /// not another materialized prefix. This deallocation never polls useful
    /// work or refunds cumulative work/high-water observations.
    fn dispose_aggregate_string(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let Some(row) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(control));
        };
        let (value, allocation, origins) = match mem::replace(row, RetainedPayload::Retired) {
            RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } => (value, allocation, origins),
            // A row of another kind is handed back where it was found.
            other => {
                *row = other;
                return Err(SessionError::WrongToken);
            }
        };
        drop(value);
        control.release_document_string(allocation)?;
        if let Some(origins) = origins {
            *row = RetainedPayload::HistoricalString { origins };
        }
        Ok(())
    }
    /// Retired raw fields may be unique immutable history leaves. Keep those
    /// actual allocations; only reconstructed aggregate backings are redundant.
    fn retire_registered_string(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let dispose = self.releasable_string(control, slot, false)?;
        if dispose {
            payload_write::<RetainedPayload>(control, 2)?;
        }
        self.retire_field_group(control, slot)?;
        if dispose {
            self.dispose_aggregate_string(control, slot)?;
        }
        Ok(())
    }
    fn bind_string(&mut self, index: usize, location: StringLocation) -> Result<(), SessionError> {
        let Some(RetainedPayload::String {
            location: current, ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(SessionError::registry_row());
        };
        *current = location;
        if let StringLocation::Text(node) = location {
            self.payloads.join_text(node, index)?;
        }
        Ok(())
    }
    /// Same parser-owned HTML root, real slots and original lexical origins.
    /// Imported/native attribute backing requires a later admitted importer.
    fn add_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        if let Some(prepared) = self.prepare_attribute_merge(control, target, input)? {
            self.finish_attribute_merge(control, prepared)?;
        }
        Ok(())
    }
    /// Real borrowed-input admission and owned staging shared by the complete
    /// operation and its bounded commit. Rejection before take preserves Tag.
    fn prepare_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<Option<PreparedAttributeMerge>, SessionError> {
        self.prepare_named_attribute_merge(control, target, input, AttributeMergeName::Html)
    }
    fn prepare_named_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
        name: AttributeMergeName,
    ) -> Result<Option<PreparedAttributeMerge>, SessionError> {
        let target = self.index(control, target)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(target)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        if !exact_ascii(
            control,
            element.name().namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? || !exact_ascii(control, Some(element.name().local_name()), name.local())?
        {
            return Err(SessionError::WrongToken);
        }
        let container = self.element_attribute_container(control, target)?;
        self.payloads.check_new_aliases(
            ConstructionView::Active(self.construction.storage(control)?),
            control,
        )?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        if Arc::weak_count(element.attributes().shared_attributes()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        Self::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), name.local())? {
            return Err(SessionError::WrongToken);
        }
        check_text_source(control, &tag.name)?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(a) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            Self::check_text(control, &mut a.name)?;
            Self::check_text(control, &mut a.value)?;
            lowercase_token_name(control, a.name.value.text())?;
            check_text_source(control, &a.name)?;
            check_text_source(control, &a.value)?;
            control.require_source_range(&a.source)?;
        }
        self.begin_attribute_edit(
            control,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge {
                candidate_count: tag.attributes.len(),
            },
        )?;
        let planned = (|| {
            for index in 0..tag.attributes.len() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                self.construction.plan_attribute_candidate(
                    control,
                    index,
                    tag.attributes.as_slice(),
                )?;
            }
            self.construction
                .attribute_edit(control)?
                .selected_count()
                .map_err(|error| construction_owner::construction_error(control, error))
        })();
        let count = match planned {
            Ok(count) => count,
            Err(error) => {
                self.construction.abort_attributes(control)?;
                return Err(error);
            }
        };
        // Final borrowed-input admission leaves the complete Tag caller-owned.
        if let Err(error) = payload_write::<Option<Tag>>(control, 1) {
            self.construction.abort_attributes(control)?;
            return Err(error);
        }
        let Some(Tag {
            name,
            mut attributes,
            ..
        }) = input.take()
        else {
            // Nothing was taken: only the edit begun above needs ending.
            self.construction.abort_attributes(control)?;
            return Err(SessionError::Consumed);
        };
        let result = (|| {
            name.release(control)?;
            let (values, allocation) = control.new_vector()?;
            let prepared = match self.prepare_payload_slot(control) {
                Ok(prepared) => prepared,
                Err(error) => {
                    release_vector(control, values, allocation)?;
                    return Err(error);
                }
            };
            let stage = self
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Attributes(Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len: count,
                        reversed_pairs: 0,
                    })),
                )
                .map_err(|_| latch_registry_row(control))?;
            let mut remaining = count;
            while !attributes.as_slice().is_empty() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                let decision_index = attributes
                    .len()
                    .checked_sub(1)
                    .ok_or_else(|| latch_registry_row(control))?;
                let keep = self
                    .construction
                    .attribute_edit(control)?
                    .decision(decision_index)
                    .map_err(|error| construction_owner::construction_error(control, error))?;
                let Some(attribute) = attributes.try_pop(control)? else {
                    return Err(latch_registry_row(control));
                };
                if !keep {
                    let retained = (|| {
                        if let Some(evidence) = self.evidence.as_mut() {
                            evidence
                                .lexical_attribute_ignored(control, attribute.lexical.as_ref())?;
                        }
                        Ok::<_, SessionError>(())
                    })();
                    if let Err(error) = retained {
                        attribute.name.release(control)?;
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                    attribute.name.release(control)?;
                    attribute.value.release(control)?;
                    continue;
                }
                let (local, name_ticket) = match self.register_text(control, attribute.name) {
                    Ok(value) => value,
                    Err(error) => {
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                };
                let (value, value_ticket) = self.register_text(control, attribute.value)?;
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                }))) = self.payloads.values.get_mut(stage)
                else {
                    return Err(latch_registry_row(control));
                };
                control.reserve_vector_geometric_toward(values, allocation, 1, count)?;
                control.charge(
                    WorkCategory::CopyByte,
                    (const { mem::size_of::<Attribute>() + 2 * mem::size_of::<StringLocation>() })
                        as u64,
                )?;
                control.charge(WorkCategory::RecordWrite, 3)?;
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                values.push(Attribute::new(
                    QualifiedName::new(Namespace::none(), None, local),
                    value,
                ));
                remaining = remaining
                    .checked_sub(1)
                    .ok_or_else(|| latch_registry_row(control))?;
                self.bind_string(name_ticket, StringLocation::AttributeName(stage, remaining))
                    .map_err(|_| latch_registry_row(control))?;
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(stage, remaining),
                )
                .map_err(|_| latch_registry_row(control))?;
            }
            if count == 0 {
                self.construction.finish_attributes(control)?;
                return Ok(None);
            }
            // The actual stage remains reversed: last-pop yields source order
            // without a variable shift or a second attribute representation.
            self.edit_attributes(control, target, container, stage, count, false)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let old_len = element_record(&self.construction, control, target)?
                .attributes()
                .len();
            self.begin_attribute_transfer(control, stage, container, old_len)?;
            payload_write::<PreparedAttributeMerge>(control, 1)?;
            Ok(Some(PreparedAttributeMerge {
                target,
                container,
                stage,
                start: old_len,
                count,
                earlier: [None; 4],
            }))
        })();
        let cleanup = release_token_attributes(attributes, control);
        result.and_then(|prepared| cleanup.map(|()| prepared))
    }
    fn finish_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        prepared: PreparedAttributeMerge,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        let PreparedAttributeMerge {
            target,
            container,
            stage,
            start,
            count,
            earlier,
        } = prepared;
        for _ in 0..count {
            self.edit_attributes(control, target, container, stage, 1, true)?;
            if self.effects.form.is_some() {
                self.drain_form_effects(control)?;
            }
        }
        // The strings staged for this merge are the registry rows from the stage on
        // and the few rows registered before it that the stager names.
        #[cfg(test)]
        for (index, row) in self.payloads.values[..stage].iter().enumerate() {
            if let RetainedPayload::String { location, .. } = row {
                assert!(
                    !matches!(
                        location,
                        StringLocation::AttributeName(owner, _)
                            | StringLocation::AttributeValue(owner, _)
                            | StringLocation::AttributeNamespace(owner, _)
                            | StringLocation::AttributePrefix(owner, _)
                            if *owner == stage
                    ) || earlier.contains(&Some(index)),
                    "row {index} before stage {stage} names the stage unlisted"
                );
            }
        }
        for index in earlier
            .into_iter()
            .flatten()
            .chain(stage..self.payloads.values.len())
        {
            self.rebind_attribute_location(control, index, stage, container, start)?;
        }
        // Clearing the redirect is safe only after every original location was
        // rebound. A stop above retains the exact transfer phase.
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        payload_write::<usize>(control, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(stage) else {
            return Err(latch_registry_row(control));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Transferring {
            values, allocation, ..
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(control));
        };
        *storage = Some(AttributeStorage::Building {
            values,
            allocation,
            original_len: 0,
            reversed_pairs: 0,
        });
        if self.construction.has_attribute_edit() {
            self.construction.finish_attributes(control)?;
        }
        let end = start
            .checked_add(count)
            .ok_or_else(|| latch_registry_row(control))?;
        for position in start..end {
            self.initial_nonce_attribute(control, target, position)?;
            self.initial_meta_attribute(control, target, position)?;
            self.association_attribute_hook(control, target, position)?;
            self.initial_dialog_attribute(control, target, position)?;
        }
        if base::is_base(
            self.construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?,
        ) {
            // Only an actual appended no-namespace href changes the pinned
            // first-Base href trigger. Other fields share this merge owner.
            let mut href_changed = false;
            for position in start..end {
                control.charge(WorkCategory::RecordVisit, 1)?;
                let attribute = element_attribute(&self.construction, control, target, position)?;
                if attribute.name().namespace().uri().is_none()
                    && exact_ascii(control, Some(attribute.name().local_name()), "href")?
                {
                    href_changed = true;
                    break;
                }
            }
            self.base_change(control, target, None, href_changed)?;
        }
        Ok(())
    }

    fn begin_attribute_transfer(
        &mut self,
        control: &mut ParseControl,
        stage: usize,
        destination: usize,
        start: usize,
    ) -> Result<(), SessionError> {
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(stage) else {
            return Err(latch_registry_row(control));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Building {
            values,
            allocation,
            original_len,
            reversed_pairs,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(control));
        };
        // Last-pop consumes the unchanged reverse stage; a stage that was
        // already reversed is handed back to the row untouched.
        if reversed_pairs != 0 {
            *storage = Some(AttributeStorage::Building {
                values,
                allocation,
                original_len,
                reversed_pairs,
            });
            return Err(latch_registry_row(control));
        }
        *storage = Some(AttributeStorage::Transferring {
            values,
            allocation,
            destination,
            start,
            original_len,
            moved: 0,
        });
        Ok(())
    }

    fn rebind_attribute_location(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        stage: usize,
        destination: usize,
        start: usize,
    ) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        #[cfg(test)]
        control.observe_registry_iteration();
        if let Some(RetainedPayload::String { location, .. }) = self.payloads.values.get_mut(index)
        {
            control.charge(
                WorkCategory::CopyByte,
                (const { 2 * mem::size_of::<StringLocation>() }) as u64,
            )?;
            control.charge(WorkCategory::RecordWrite, 1)?;
            #[cfg(test)]
            control.observe_registry_location_write(mem::size_of::<StringLocation>());
            *location = match *location {
                StringLocation::AttributeName(owner, index) if owner == stage => {
                    StringLocation::AttributeName(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                StringLocation::AttributeValue(owner, index) if owner == stage => {
                    StringLocation::AttributeValue(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                StringLocation::AttributeNamespace(owner, index) if owner == stage => {
                    StringLocation::AttributeNamespace(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                StringLocation::AttributePrefix(owner, index) if owner == stage => {
                    StringLocation::AttributePrefix(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedAttributes)
                        })?,
                    )
                }
                other => other,
            };
        }
        Ok(())
    }

    fn prepare_id_attribute_effects(
        &self,
        control: &mut ParseControl,
        target: usize,
        stage: usize,
    ) -> Result<Option<FormEffects>, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
            values, ..
        }))) = self.payloads.values.get(stage)
        else {
            return Err(latch_registry_row(control));
        };
        let attribute = values.last().ok_or_else(|| latch_registry_row(control))?;
        if attribute.name().namespace().uri().is_some()
            || !exact_ascii(control, Some(attribute.name().local_name()), "id")?
        {
            return Ok(None);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        if attribute.value().is_empty() {
            return Ok(None);
        }
        #[cfg(test)]
        self.assert_explicit_forms_exact();
        for &index in self.payloads.explicit_form_list() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            }) = self.payloads.values.get(index)
                && state.kind.listed()
                && self.form_attribute(control, *node, "form")?.is_some()
            {
                return Ok(Some(FormEffects {
                    root: target,
                    old_parent: None,
                    new_parent: None,
                    stage: FormEffectStage::IdChange,
                    insert: false,
                    cause: InsertionCause::NonParser,
                    next: Some(target),
                    pass: FormEffectPass::Local,
                }));
            }
        }
        Ok(None)
    }

    /// Every attribute edit begins here, so the creation-time registry hint for
    /// the record's attribute strings is dropped before the first change.
    fn begin_attribute_edit(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        mode: HtmlConstructionAttributeEditMode,
    ) -> Result<(), SessionError> {
        self.payloads.forget_attribute_strings(target);
        self.construction.begin_attributes(control, target, mode)
    }
    /// Removes/restores only the registry's known strong owner. Both restoration
    /// paths are admitted before removal; no new wrapper allocation is needed.
    fn edit_attributes(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        container: usize,
        stage: usize,
        count: usize,
        append: bool,
    ) -> Result<(), SessionError> {
        self.payloads.forget_attribute_strings(target);
        let form_effects = if append {
            self.prepare_id_attribute_effects(control, target, stage)?
        } else {
            None
        };
        if form_effects.is_some() {
            payload_write::<Option<FormEffects>>(control, 1)?;
        }
        control.charge(WorkCategory::ProviderCall, 2)?;
        control.charge(WorkCategory::RecordVisit, 3)?;
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<OrderedAttributes>() as u64,
        )?;
        if append {
            // No fallible operation may intervene between the actual slot move
            // and this fixed progress write; cleanup resolves it immediately.
            payload_write::<usize>(control, 1)?;
        }
        // Everything that can refuse is checked before ownership leaves the
        // container; the target stays an element while its attributes change.
        if !matches!(
            self.construction
                .storage(control)?
                .node_at(target)
                .map(|node| node.kind()),
            Some(HtmlNodeKind::Element(_))
        ) || stage > self.payloads.values.len()
        {
            return Err(latch_registry_row(control));
        }
        let (earlier, later) = self.payloads.values.split_at_mut(stage);
        let Some(RetainedPayload::Attributes(storage)) = earlier.get_mut(container) else {
            return Err(latch_registry_row(control));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Frozen {
            values,
            mut allocation,
            mut shared,
            element,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(control));
        };
        drop(values);
        let was_named = self.construction.has_attribute_edit();
        let started = if was_named {
            Ok(())
        } else {
            self.construction.begin_attributes(
                control,
                target,
                HtmlConstructionAttributeEditMode::Slots,
            )
        };
        let result = (|| {
            started?;
            let group = self.construction.attribute_edit(control)?;
            let slot = if append {
                let Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
                    values,
                    allocation: incoming_allocation,
                    ..
                }))) = later.first_mut()
                else {
                    return Err(latch_registry_row(control));
                };
                control.require_incoming_attribute_vector(values, incoming_allocation)?;
                group.append(values)
            } else {
                group.reserve(count)
            }
            .map_err(|error| construction_owner::construction_error(control, error))?;
            let ready = prepare_attribute_slot(control, slot, &mut shared, &mut allocation)?;
            #[cfg(test)]
            if append {
                control.observe_construction_attribute_append();
            }
            ready.commit();
            Ok::<_, SessionError>(())
        })();
        // Restore the sole ticket before creating the registry's known alias.
        let Some(HtmlNodeKind::Element(record)) = self
            .construction
            .storage(control)?
            .node_at(target)
            .map(|node| node.kind())
        else {
            return Err(latch_registry_row(control));
        };
        control.restore_attribute_weak(record.attributes().shared_attributes(), &mut shared);
        let values = record.attributes().clone();
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        let mut result = result;
        if append && result.is_ok() {
            match later.first_mut() {
                Some(RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
                    moved,
                    ..
                }))) => match moved.checked_add(1) {
                    Some(next) => {
                        *moved = next;
                        if form_effects.is_some() {
                            self.effects.form = form_effects;
                        }
                    }
                    None => result = Err(latch_registry_row(control)),
                },
                _ => result = Err(latch_registry_row(control)),
            }
        }
        if result.is_err() {
            self.construction.abort_attributes(control)?;
        } else if !was_named {
            self.construction.finish_attributes(control)?;
        }
        result
    }
    fn reverse_attributes(
        &mut self,
        control: &mut ParseControl,
        container: usize,
    ) -> Result<(), SessionError> {
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. }))) =
            self.payloads.values.get(container)
        else {
            return Err(latch_registry_row(control));
        };
        let len = values.len();
        for left in 0..len / 2 {
            let right = mirrored_position(len, left).ok_or_else(|| latch_registry_row(control))?;
            control.admit_atomic_layout(const { mem::size_of::<Attribute>() * 2 })?;
            control.charge(
                WorkCategory::CopyByte,
                (const { mem::size_of::<Attribute>() * 2 }) as u64,
            )?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.charge(WorkCategory::RecordWrite, 3)?;
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                reversed_pairs,
                ..
            }))) = self.payloads.values.get_mut(container)
            else {
                return Err(latch_registry_row(control));
            };
            values.swap(left, right);
            *reversed_pairs = reversed_pairs
                .checked_add(1)
                .ok_or_else(|| latch_registry_row(control))?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let index = self.index(control, element)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(index)
            .ok_or_else(|| latch_registry_row(control))?
            .template_contents()
            .map(|contents| self.node(control, contents.position()))
            .transpose()
    }

    fn container(&self, control: &mut ParseControl, index: usize) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .check_container()
            .map_err(|error| construction_owner::construction_error(control, error))
    }

    /// Qualifies the intended parent of an element being created. Creation
    /// reads only the parent's node document and registry and never adds a
    /// child, so a template element qualifies, unlike for [`Self::container`]:
    /// the adoption agency algorithm creates replacements with the template
    /// as the intended parent and later inserts them into its contents.
    fn creation_parent(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        match self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        {
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_) => Ok(()),
            _ => Err(SessionError::NotContainer),
        }
    }

    /// Actual entry into an associated document leaves the closed ordinary
    /// producer inventory. This qualifies topology mutation, not preparation's
    /// source-ordered ordinary connectivity condition.
    fn associated_document_parent(
        &mut self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<bool, SessionError> {
        let storage = self.construction.storage(control)?;
        let root = self.payloads.tree_root(control, storage, index)?;
        self.associated_root_in(control, storage, root)
    }
    fn associated_document_parent_in(
        &mut self,
        control: &mut ParseControl,
        storage: &bounded::HtmlConstructionStorage,
        index: usize,
    ) -> Result<bool, SessionError> {
        let root = self.payloads.tree_root(control, storage, index)?;
        self.associated_root_in(control, storage, root)
    }
    /// Whether the tree rooted at `root`, found along parent edges, is a
    /// fragment that an associated document owns. Only a fragment has an
    /// owner, and an owned fragment has no parent, so the root decides.
    fn associated_root_in(
        &self,
        control: &mut ParseControl,
        storage: &bounded::HtmlConstructionStorage,
        root: usize,
    ) -> Result<bool, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let record = storage.node_at(root).ok_or(SessionError::ForeignNode)?;
        match record.fragment_owner() {
            None => Ok(false),
            Some(HtmlFragmentOwner::ShadowHost(_)) => Ok(self.fragment.is_none()),
            Some(HtmlFragmentOwner::Template(template)) => {
                control.charge(WorkCategory::ProviderCall, 1)?;
                let template_position = storage
                    .node_by_result_id(template)
                    .map_err(|_| SessionError::ForeignNode)?
                    .position();
                let (_, state) = self.template_slot_in(control, storage, template_position)?;
                if state.contents != root {
                    return Err(SessionError::TemplateMetadataUnavailable);
                }
                Ok(false)
            }
        }
    }

    /// Only ordinary parents participate in form association. The single
    /// override represents the source removal intermediate after a move's
    /// coherent final topology has committed; it is not another tree.
    fn ordinary_parent(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        ordinary_view_parent(
            ConstructionView::Active(self.construction.storage(control)?),
            control,
            node,
            detached,
        )
    }

    fn ordinary_root(
        &self,
        control: &mut ParseControl,
        mut node: usize,
        detached: Option<usize>,
    ) -> Result<usize, SessionError> {
        if detached.is_none() {
            let storage = self.construction.storage(control)?;
            return self.payloads.tree_root_uncached(control, storage, node);
        }
        for _ in 0..self.construction.storage(control)?.len() {
            match self.ordinary_parent(control, node, detached)? {
                Some(parent) => node = parent,
                None => return Ok(node),
            }
        }
        Err(SessionError::Cycle)
    }

    fn html_local(
        &self,
        control: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<bool, SessionError> {
        let storage = self.construction.storage(control)?;
        Self::html_local_in(control, storage, node, local)
    }
    fn html_local_in(
        control: &mut ParseControl,
        storage: &bounded::HtmlConstructionStorage,
        node: usize,
        local: &str,
    ) -> Result<bool, SessionError> {
        control.charge(WorkCategory::RecordVisit, 3)?;
        let record = storage.node_at(node).ok_or(SessionError::ForeignNode)?;
        let HtmlNodeKind::Element(element) = record.kind() else {
            return Ok(false);
        };
        Ok(exact_ascii(
            control,
            element.name().namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? && exact_ascii(control, Some(element.name().local_name()), local)?)
    }

    fn nearest_form(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let mut next = self.ordinary_parent(control, node, detached)?;
        for _ in 0..self.construction.storage(control)?.len() {
            let Some(node) = next else { return Ok(None) };
            if self.html_local(control, node, "form")? {
                return Ok(Some(node));
            }
            next = self.ordinary_parent(control, node, detached)?;
        }
        Err(SessionError::Cycle)
    }

    fn prepare_form_effects(
        &self,
        control: &mut ParseControl,
        root: usize,
        remove: bool,
        insert: bool,
        new_parent: Option<usize>,
    ) -> Result<Option<FormEffects>, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let old_parent = self
            .construction
            .node_at(root)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .map(|node| node.position());
        let mut children_changed = false;
        for parent in [old_parent.filter(|_| remove), new_parent]
            .into_iter()
            .flatten()
        {
            if self.textarea_slot(control, parent)?.is_some()
                || self.html_local(control, parent, "script")?
                || self.html_local(control, parent, "style")?
            {
                children_changed = true;
            }
        }
        let adoption = if let Some(parent) = new_parent {
            match (
                self.node_document_ref(control, root),
                self.node_document_ref(control, parent),
            ) {
                (Ok(old), Ok(new)) => old != new,
                (Err(SessionError::NodeDocumentUnavailable), _)
                | (_, Err(SessionError::NodeDocumentUnavailable)) => false,
                (Err(error), _) | (_, Err(error)) => return Err(error),
            }
        } else {
            false
        };
        let mut local = adoption;
        // The subtree is local when it holds a Base or form association row's
        // node, or an id while some listed control has a form attribute (an id
        // can then resolve that attribute). One walk of the subtree finds the
        // rows through their joins and whether it holds an id, so only a
        // subtree with an id scans the registry's form associations.
        let mut joined = false;
        let mut identified = false;
        let mut next = Some(root);
        while let Some(node) = next {
            for name in [
                "option",
                "optgroup",
                "selectedcontent",
                "script",
                "details",
                "dialog",
                "img",
                "source",
                "meta",
                "style",
                "link",
            ] {
                if self.html_local(control, node, name)? {
                    local = true;
                    if name == "option" || name == "selectedcontent" {
                        self.select_metadata(control, node)?;
                    }
                    if name == "script" {
                        script_metadata(&self.payloads, control, node)?;
                    }
                }
            }
            if self.svg_script_local(control, node)? {
                self.svg_state_at(control, node)?;
                local = true;
            }
            for join in [Join::Base, Join::FormAssociation] {
                joined |= self.payloads.joined(control, node, join)?.is_some();
            }
            identified |= self.has_id(control, node)?;
            next = self.next_effect_node(control, root, node)?;
        }
        let mut listed_explicit = false;
        if !joined && identified {
            #[cfg(test)]
            self.assert_explicit_forms_exact();
            for &slot in self.payloads.explicit_form_list() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if let Some(RetainedPayload::FormAssociation {
                    node: Some(node),
                    state,
                }) = self.payloads.values.get(slot)
                    && state.kind.listed()
                    && self.form_attribute(control, *node, "form")?.is_some()
                {
                    listed_explicit = true;
                    break;
                }
            }
        }
        #[cfg(test)]
        assert_eq!(
            joined || listed_explicit,
            self.uncharged_form_subtree_is_local(root),
            "form locality of the subtree at {root}"
        );
        local |= joined || listed_explicit;
        if !local && !children_changed {
            return Ok(None);
        }
        Ok(Some(FormEffects {
            root,
            old_parent,
            new_parent,
            stage: if remove {
                FormEffectStage::Removal
            } else if adoption {
                FormEffectStage::Adoption
            } else {
                FormEffectStage::Insertion
            },
            insert,
            cause: InsertionCause::NonParser,
            next: Some(root),
            pass: FormEffectPass::Local,
        }))
    }

    /// Borrowed exact attributes over the actual managed element. Neither the
    /// value nor the installed attribute container is cloned for resolution.
    fn form_attribute<'a>(
        &'a self,
        control: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<Option<&'a DocumentString>, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let record = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        let HtmlNodeKind::Element(element) = record.kind() else {
            return Ok(None);
        };
        for attribute in element.attributes().as_slice() {
            control.charge(WorkCategory::RecordVisit, 2)?;
            if attribute.name().namespace().uri().is_none()
                && exact_ascii(control, Some(attribute.name().local_name()), local)?
            {
                return Ok(Some(attribute.value()));
            }
        }
        Ok(None)
    }

    fn has_id(&self, control: &mut ParseControl, node: usize) -> Result<bool, SessionError> {
        let id = self.form_attribute(control, node, "id")?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(id.is_some_and(|id| !id.is_empty()))
    }

    /// Source tree order without traversal storage. The virtual removal view
    /// both detaches the root's parent and hides that child from outside trees.
    /// Its own subtree retains its exact internal child order.
    fn next_view_node(
        &self,
        control: &mut ParseControl,
        root: usize,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        next_ordinary_view_node(
            ConstructionView::Active(self.construction.storage(control)?),
            control,
            root,
            node,
            detached,
        )
    }

    /// Whether no node follows `node` in `root`'s subtree in tree order, apart
    /// from `node`'s own descendants: it ends every ancestor's children up to
    /// `root`.
    fn ends_subtree(
        &self,
        c: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<bool, SessionError> {
        let mut node = node;
        for _ in 0..self.construction.storage(c)?.len() {
            let Some(parent) = self.ordinary_parent(c, node, None)? else {
                return Ok(false);
            };
            c.charge(WorkCategory::RecordVisit, 2)?;
            let record = self
                .construction
                .node_at(parent)
                .ok_or_else(|| latch_registry_row(c))?;
            let last = record
                .child_count()
                .checked_sub(1)
                .and_then(|last| record.child_at(last))
                .map(|child| child.position());
            if last != Some(node) {
                return Ok(false);
            }
            if parent == root {
                return Ok(true);
            }
            node = parent;
        }
        Err(SessionError::Cycle)
    }
    fn next_effect_node(
        &self,
        control: &mut ParseControl,
        root: usize,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        self.next_view_node(control, root, node, None)
    }

    fn connected(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        let root = self.ordinary_root(control, node, detached)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(matches!(
            self.construction
                .node_at(root)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            HtmlNodeKind::Document(_)
        ))
    }

    /// DOM3071 connected is a shadow-including root, not ordinary tree membership.
    /// The virtual removed root applies to every owned host hop as well.
    fn shadow_including_connected(
        &self,
        control: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        let mut root = self.ordinary_root(control, node, detached)?;
        'roots: for _ in 0..self.construction.storage(control)?.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(
                self.construction
                    .node_at(root)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                HtmlNodeKind::Document(_)
            ) {
                return Ok(true);
            }
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(HtmlFragmentOwner::ShadowHost(host)) = &self
                .construction
                .node_at(root)
                .ok_or_else(|| latch_registry_row(control))?
                .fragment_owner()
            {
                control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                control.charge(WorkCategory::ProviderCall, 1)?;
                let host_position = self
                    .construction
                    .storage(control)?
                    .node_by_result_id(host)
                    .map_err(|_| SessionError::ForeignNode)?
                    .position();
                root = self.ordinary_root(control, host_position, detached)?;
                continue;
            }
            // A retained captured attachment is an influencing host, not proof
            // of disconnection. The adapter cannot invent its connectedness.
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(fragment) = &self.fragment {
                for effect in fragment.effects.as_slice() {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    if let context::ContextEffect::AttachShadow {
                        contents: Some(contents),
                        host,
                        ..
                    } = effect
                        && *contents == root
                    {
                        match host {
                            context::ShadowHostKey::Owned(host) => {
                                control.charge(
                                    WorkCategory::CopyByte,
                                    mem::size_of::<usize>() as u64,
                                )?;
                                root = self.ordinary_root(control, *host, detached)?;
                                continue 'roots;
                            }
                            context::ShadowHostKey::Captured(host) => {
                                return match self.captured_shadow_including_document(control, host)
                                {
                                    Ok(document) => Ok(document.is_some()),
                                    Err(error) => Err(end_session_on(control, error)),
                                };
                            }
                        }
                    }
                }
            }
            return Ok(false);
        }
        Err(SessionError::Cycle)
    }

    fn first_id_form(
        &self,
        control: &mut ParseControl,
        node: usize,
        value: &DocumentString,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let root = self.ordinary_root(control, node, detached)?;
        let mut next = Some(root);
        while let Some(candidate) = next {
            if let Some(id) = self.form_attribute(control, candidate, "id")? {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if !id.is_empty() && exact_string_equal(control, id, value)? {
                    return Ok(self
                        .html_local(control, candidate, "form")?
                        .then_some(candidate));
                }
            }
            next = self.next_view_node(control, root, candidate, detached)?;
        }
        Ok(None)
    }

    fn reset_form_association(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        node: usize,
        mut state: FormAssociationState,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        use bounded::{
            HtmlFormOwnerResetAction as Action, HtmlFormOwnerResetCursor,
            HtmlFormOwnerResetReply as Reply,
        };
        // Admit the actual constant-state rule before retaining it. This cursor
        // has no identity or tree: every effect below mutates the same owner.
        payload_write::<HtmlFormOwnerResetCursor>(control, 1)?;
        let mut cursor = HtmlFormOwnerResetCursor::new(state.kind.listed(), state.owner.is_some());
        payload_write::<Option<usize>>(control, 1)?;
        let mut found = None;
        loop {
            let reply = match cursor.action() {
                Action::ClearParserInserted => {
                    // Source step 1 precedes every attribute/ancestor lookup.
                    state.parser_inserted = false;
                    state.evidence = FormAssociationEvidence::Reset;
                    set_form_association_state(self, control, slot, state)?;
                    Reply::Applied
                }
                Action::ReadFormAttributePresence => Reply::FormAttributePresent(
                    self.form_attribute(control, node, "form")?.is_some(),
                ),
                Action::ReadCurrentOwnerIsNearestAncestor => {
                    let Some(owner) = state.owner else {
                        return Err(latch_registry_row(control));
                    };
                    Reply::CurrentOwnerIsNearestAncestor(
                        self.nearest_form(control, node, detached)? == Some(owner),
                    )
                }
                Action::ClearOwner => {
                    state.owner = None;
                    set_form_association_state(self, control, slot, state)?;
                    Reply::Applied
                }
                Action::ApplyOwnerChangeEffects => {
                    if state.kind == HtmlNativeFormAssociationKind::Input {
                        self.normalize_radio(control, node, detached)?;
                    }
                    Reply::Applied
                }
                Action::ReadConnectedness => {
                    Reply::Connected(self.shadow_including_connected(control, node, detached)?)
                }
                Action::FindFirstIdMatch => {
                    let value = self
                        .form_attribute(control, node, "form")?
                        .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                    let candidate = self.first_id_form(control, node, value, detached)?;
                    payload_write::<Option<usize>>(control, 1)?;
                    found = candidate;
                    Reply::FormFound(found.is_some())
                }
                Action::FindNearestAncestor => {
                    // The earlier retention probe cannot supply this candidate:
                    // lookup happens afresh after the owner-clear effects.
                    let candidate = self.nearest_form(control, node, detached)?;
                    payload_write::<Option<usize>>(control, 1)?;
                    found = candidate;
                    Reply::FormFound(found.is_some())
                }
                Action::AssociateFoundOwner => {
                    let Some(owner) = found else {
                        return Err(latch_registry_row(control));
                    };
                    state.owner = Some(owner);
                    set_form_association_state(self, control, slot, state)?;
                    Reply::Applied
                }
                Action::Complete => return Ok(()),
            };
            payload_write::<HtmlFormOwnerResetCursor>(control, 1)?;
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        }
    }

    /// Called synchronously only by the mutation that installed this phase.
    /// No consumer resume method exists. After a stop only disposal is allowed.
    fn step_form_effects(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        self.step_form_effects_with_meta_csp(control, &mut None)
    }
    fn step_form_effects_with_meta_csp(
        &mut self,
        control: &mut ParseControl,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let result = (|| {
            control.require_identity(&self.control)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<Option<FormEffects>>() as u64,
            )?;
            let Some(mut effects) = self.effects.form else {
                return Ok(());
            };
            if effects.stage == FormEffectStage::Adoption {
                let parent = effects.new_parent.ok_or(SessionError::NotContainer)?;
                let document = match self.node_document_ref(control, parent) {
                    Ok(document) => Some(document),
                    Err(SessionError::NodeDocumentUnavailable) => None,
                    Err(error) => return Err(error),
                };
                let old = match self.node_document_ref(control, effects.root) {
                    Ok(document) => Some(document),
                    Err(SessionError::NodeDocumentUnavailable) => None,
                    Err(error) => return Err(error),
                };
                effects.stage = FormEffectStage::Insertion;
                effects.next = Some(effects.root);
                effects.pass = FormEffectPass::Local;
                payload_write::<Option<FormEffects>>(control, 1)?;
                self.effects.form = Some(effects);
                if let (Some(old), Some(document)) = (old, document)
                    && old != document
                {
                    self.start_effect(
                        control,
                        effects::EffectFrame::AdoptSubtree {
                            root: effects.root,
                            next: Some(effects.root),
                            document,
                        },
                    )?;
                }
                return Ok(());
            }
            if matches!(
                effects.stage,
                FormEffectStage::RemovalChildren | FormEffectStage::InsertionChildren
            ) {
                let removal = effects.stage == FormEffectStage::RemovalChildren;
                // A fragment's retained insertion owner completes every sibling
                // hook before its single parent hook/static-list continuation.
                if !removal && self.fragment_insertion_parent(control, effects.new_parent)? {
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = None;
                    return Ok(());
                }
                let parent = if removal {
                    effects.old_parent
                } else {
                    effects.new_parent
                };
                if let Some(parent) = parent {
                    self.refresh_textarea(control, parent, removal.then_some(effects.root), false)?;
                    self.association_children(control, parent, removal.then_some(effects.root))?;
                }
                if removal && effects.insert {
                    effects.stage = FormEffectStage::Adoption;
                    effects.next = Some(effects.root);
                    effects.pass = FormEffectPass::Local;
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    #[cfg(test)]
                    if let Some(cancellation) = self.cancel_after_removal.take() {
                        cancellation.cancel();
                    }
                    if let Some(parent) = parent
                        && self.html_local(control, parent, "script")?
                        && self.connected(control, parent, Some(effects.root))?
                    {
                        self.start_effect(
                            control,
                            effects::EffectFrame::ScriptPreparation {
                                node: parent,
                                detached: Some(effects.root),
                            },
                        )?;
                    }
                    return Ok(());
                }
                payload_write::<Option<FormEffects>>(control, 1)?;
                self.effects.form = None;
                if !removal && effects.insert {
                    // Retain only the finite continuation now. Actual static
                    // collection runs after the source-prior parent script hook.
                    self.start_effect(
                        control,
                        effects::EffectFrame::InsertionPostConnection {
                            roots: effects::PostConnectionRoots::Single(effects.root),
                            cause: effects.cause,
                        },
                    )?;
                }
                if let Some(parent) = parent
                    && self.html_local(control, parent, "script")?
                    && self.connected(control, parent, removal.then_some(effects.root))?
                {
                    self.start_effect(
                        control,
                        effects::EffectFrame::ScriptPreparation {
                            node: parent,
                            detached: removal.then_some(effects.root),
                        },
                    )?;
                }
                return Ok(());
            }
            let Some(node) = effects.next else {
                if effects.stage == FormEffectStage::Removal
                    || effects.stage == FormEffectStage::Insertion
                {
                    effects.stage = if effects.stage == FormEffectStage::Removal {
                        FormEffectStage::RemovalChildren
                    } else {
                        FormEffectStage::InsertionChildren
                    };
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    return Ok(());
                }
                payload_write::<Option<FormEffects>>(control, 1)?;
                self.effects.form = None;
                return Ok(());
            };
            let detached = (effects.stage == FormEffectStage::Removal).then_some(effects.root);
            match effects.pass {
                FormEffectPass::Local => {
                    if let Some((slot, state)) =
                        find_form_association_state(&self.payloads, control, node)?
                    {
                        match effects.stage {
                            FormEffectStage::Removal => {
                                if form_owner_reset_trigger(
                                    HtmlFormOwnerResetTrigger::RemovingOrMoving {
                                        has_owner: state.owner.is_some(),
                                    },
                                ) == HtmlFormOwnerResetDecision::ResetIfOwnerInDifferentTree
                                {
                                    let node_root = self.ordinary_root(control, node, detached)?;
                                    let Some(owner) = state.owner else {
                                        return Err(latch_registry_row(control));
                                    };
                                    if node_root != self.ordinary_root(control, owner, detached)? {
                                        self.reset_form_association(
                                            control, slot, node, state, detached,
                                        )?;
                                    }
                                }
                            }
                            FormEffectStage::Insertion
                                if form_owner_reset_trigger(
                                    HtmlFormOwnerResetTrigger::Insertion {
                                        parser_inserted: state.parser_inserted,
                                    },
                                ) == HtmlFormOwnerResetDecision::Reset =>
                            {
                                self.reset_form_association(control, slot, node, state, None)?;
                            }
                            FormEffectStage::Insertion | FormEffectStage::IdChange => {}
                            // The children stages returned above.
                            FormEffectStage::Adoption
                            | FormEffectStage::RemovalChildren
                            | FormEffectStage::InsertionChildren => {
                                return Err(latch_registry_row(control));
                            }
                        }
                    }
                    effects.pass = FormEffectPass::Intrinsic;
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    return Ok(());
                }
                FormEffectPass::Intrinsic => {
                    let id_event = self.has_id(control, node)?
                        && match effects.stage {
                            // The selected source predicate requires an actual
                            // removing hook and post-removal non-Document tree;
                            // there is no old-connected prerequisite.
                            FormEffectStage::Removal => !self.connected(control, node, detached)?,
                            FormEffectStage::Insertion => self.connected(control, node, None)?,
                            FormEffectStage::IdChange => true,
                            // The children stages returned above.
                            FormEffectStage::Adoption
                            | FormEffectStage::RemovalChildren
                            | FormEffectStage::InsertionChildren => {
                                return Err(latch_registry_row(control));
                            }
                        };
                    if id_event {
                        effects.pass = FormEffectPass::Global {
                            next_form: 0,
                            tree: if effects.stage == FormEffectStage::IdChange {
                                Some(self.ordinary_root(control, node, None)?)
                            } else {
                                None
                            },
                        };
                    } else {
                        effects.pass = FormEffectPass::Local;
                        effects.next = if effects.stage == FormEffectStage::IdChange {
                            None
                        } else {
                            self.next_effect_node(control, effects.root, node)?
                        };
                    }
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                    if effects.stage != FormEffectStage::IdChange {
                        self.image_tree_changed(
                            control,
                            node,
                            if effects.stage == FormEffectStage::Removal {
                                surgeist_html::HtmlImageTreeChange::Removed
                            } else {
                                surgeist_html::HtmlImageTreeChange::Inserted
                            },
                            effects.root,
                            effects.old_parent,
                        )?;
                        self.meta_tree_hook(control, node, detached, &effects, service)?;
                        self.association_tree_hook(
                            control,
                            node,
                            detached,
                            effects.stage == FormEffectStage::Removal,
                            effects.old_parent,
                            effects.root,
                        )?;
                        self.intrinsic_tree_hook(
                            control,
                            node,
                            detached,
                            effects.stage == FormEffectStage::Removal,
                        )?;
                    }
                    return Ok(());
                }
                FormEffectPass::Global { next_form, tree } => {
                    #[cfg(test)]
                    self.assert_explicit_forms_exact();
                    let next_slot = self.payloads.explicit_form_list().get(next_form).copied();
                    if let Some(next_slot) = next_slot {
                        control.charge(WorkCategory::RecordVisit, 1)?;
                        if let Some(RetainedPayload::FormAssociation {
                            node: Some(candidate),
                            state,
                        }) = self.payloads.values.get(next_slot)
                        {
                            control.charge(
                                WorkCategory::CopyByte,
                                (const {
                                    mem::size_of::<usize>() + mem::size_of::<FormAssociationState>()
                                }) as u64,
                            )?;
                            let candidate = *candidate;
                            let state = *state;
                            // These records can only be issued by the same
                            // fresh ordinary Document creation owner. A
                            // Document ID event also affects its detached
                            // controls; connectivity is tested by reset.
                            if self.node_document_ref(control, candidate)?
                                == self.node_document_ref(control, node)?
                                && form_owner_reset_trigger(HtmlFormOwnerResetTrigger::IdEvent {
                                    listed: state.kind.listed(),
                                }) == HtmlFormOwnerResetDecision::ResetIfFormAttributePresent
                                && self.form_attribute(control, candidate, "form")?.is_some()
                                && match tree {
                                    Some(tree) => {
                                        self.ordinary_root(control, candidate, detached)? == tree
                                    }
                                    None => true,
                                }
                            {
                                self.reset_form_association(
                                    control, next_slot, candidate, state, detached,
                                )?;
                            }
                        }
                        effects.pass = FormEffectPass::Global {
                            next_form: next_form.saturating_add(1),
                            tree,
                        };
                    } else {
                        effects.pass = FormEffectPass::Local;
                        effects.next = if effects.stage == FormEffectStage::IdChange {
                            None
                        } else {
                            self.next_effect_node(control, effects.root, node)?
                        };
                    }
                }
            }
            payload_write::<Option<FormEffects>>(control, 1)?;
            self.effects.form = Some(effects);
            Ok(())
        })();
        match result {
            Err(
                error @ (SessionError::Stop(_)
                | SessionError::CspService
                | SessionError::CspProtocol(_)),
            ) => Err(error),
            Err(error) => Err(latch_effect_fault(control, error)),
            Ok(()) => Ok(()),
        }
    }

    /// Prepared child order. The real replacement vector is admitted and every
    /// entry is read/copied before use. Commit later consists only of vector swaps.
    fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        cause: InsertionCause,
    ) -> Result<(), SessionError> {
        self.insert_with_meta_csp(control, parent, node, before, cause, &mut None)
    }
    fn insert_with_meta_csp(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        cause: InsertionCause,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let parent = self.index(control, parent)?;
        let node = self.index(control, node)?;
        let before = before.map(|node| self.index(control, node)).transpose()?;
        self.insert_indices_with_cause(control, parent, node, before, cause)?;
        self.drain_construction_effects_with_meta_csp(control, service)
    }

    fn insert_indices(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
        node: usize,
        before: Option<usize>,
    ) -> Result<(), SessionError> {
        self.insert_indices_with_cause(control, parent, node, before, InsertionCause::NonParser)
    }
    fn insert_indices_with_cause(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
        node: usize,
        before: Option<usize>,
        cause: InsertionCause,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        self.container(control, parent)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .check_detachment()
            .map_err(|error| construction_owner::construction_error(control, error))?;
        if let Some(before) = before {
            control.charge(WorkCategory::RecordVisit, 1)?;
            self.construction
                .node_at(before)
                .ok_or(SessionError::ForeignNode)?
                .check_reference_child_of(
                    self.construction
                        .node_at(parent)
                        .ok_or(SessionError::ForeignNode)?,
                )
                .map_err(|error| construction_owner::construction_error(control, error))?;
            if before == node {
                // Pre-insert adjusts the reference child but still adopts
                // (removes) and inserts the root. Final links/order are equal;
                // actual managed image facts nevertheless receive both hooks.
                if let Some(mut effects) =
                    self.prepare_form_effects(control, node, true, true, Some(parent))?
                {
                    payload_write::<InsertionCause>(control, 1)?;
                    effects.cause = cause;
                    payload_write::<Option<FormEffects>>(control, 1)?;
                    self.effects.form = Some(effects);
                }
                self.nonce_inserted_tree(control, node)?;
                return Ok(());
            }
        }
        let old_parent = self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .map(|node| node.position());
        let mut form_effects =
            self.prepare_form_effects(control, node, old_parent.is_some(), true, Some(parent))?;
        if let Some(effects) = &mut form_effects {
            payload_write::<InsertionCause>(control, 1)?;
            effects.cause = cause;
        }
        let reroot = self.begin_reroot(control, node, Some(parent))?;
        control.charge(
            WorkCategory::ProviderCall,
            if before.is_some() { 3 } else { 2 },
        )?;
        let node_handle = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let parent_handle = self
            .construction
            .node_at(parent)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let before_handle = before
            .map(|position| {
                self.construction
                    .node_at(position)
                    .map(|record| record.handle())
                    .ok_or_else(|| latch_registry_row(control))
            })
            .transpose()?;
        let requests = &[(
            HtmlParseResource::ManagedEdges,
            u64::from(old_parent.is_none()),
        )];
        let mut pending = self.construction.start(
            control,
            bounded::HtmlConstructionCommand::Insert {
                node: node_handle,
                parent: parent_handle,
                before: before_handle,
            },
        )?;
        let mut revoke_standalone = false;
        let result = (|| {
            loop {
                let before_edges = pending.before_edges();
                let Some(action) = pending.action(control)? else {
                    break;
                };
                match action.description() {
                    bounded::HtmlConstructionActionDescription::Preflight { usage } => {
                        // Source semantics stay between the original depth and
                        // edge checks, before any actual scratch-bank provider.
                        control.observe_tree_depth(usage.depth() as u64)?;
                        control.charge(WorkCategory::RecordVisit, 1)?;
                        let Some(view) = action.storage_view() else {
                            return Err(latch_owner_refusal(control));
                        };
                        revoke_standalone = self.standalone_ineligibility.is_none()
                            && self.associated_document_parent_in(control, view, parent)?;
                        control.check_acquire_many(requests)?;
                        action
                            .perform()
                            .map_err(|failure| {
                                construction_owner::construction_error(
                                    control,
                                    failure.into_parts().0,
                                )
                            })?
                            .accept();
                    }
                    bounded::HtmlConstructionActionDescription::Commit { .. } => {
                        // These fixed source payloads precede the atomic commit,
                        // after the original actual child-order preparation.
                        self.construction.perform_action_with_commit(
                            control,
                            action,
                            requests,
                            before_edges,
                            |control| {
                                if revoke_standalone {
                                    payload_write::<Option<StandaloneUnsupported>>(control, 1)?;
                                }
                                if form_effects.is_some() {
                                    payload_write::<Option<FormEffects>>(control, 1)?;
                                }
                                Ok(())
                            },
                        )?;
                        break;
                    }
                    _ => {
                        self.construction.perform_action(
                            control,
                            action,
                            requests,
                            before_edges,
                        )?;
                    }
                }
            }
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = result {
            let aborted = self.construction.abort_pending(control, pending);
            self.end_reroot(reroot);
            aborted?;
            return Err(error);
        }
        let finished = self.construction.finish_pending(control, pending);
        self.end_reroot(reroot);
        finished?;
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        if revoke_standalone {
            self.standalone_ineligibility = Some(StandaloneUnsupported::ScriptDocument);
        }
        self.nonce_inserted_tree(control, node)?;
        Ok(())
    }

    fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        let node = self.index(control, node)?;
        self.remove_index(control, node)?;
        self.drain_construction_effects(control)
    }

    fn remove_index(
        &mut self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .check_detachment()
            .map_err(|error| construction_owner::construction_error(control, error))?;
        if self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .is_none()
        {
            return Ok(());
        }
        let form_effects = self.prepare_form_effects(control, node, true, false, None)?;
        if form_effects.is_some() {
            payload_write::<Option<FormEffects>>(control, 1)?;
        }
        control.charge(WorkCategory::ProviderCall, 1)?;
        let handle = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let reroot = self.begin_reroot(control, node, None)?;
        let detached = self.construction.apply(
            control,
            bounded::HtmlConstructionCommand::Detach { node: handle },
            &[],
        );
        self.end_reroot(reroot);
        detached?;
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        Ok(())
    }

    fn prepare_script_owned(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        detached: Option<usize>,
    ) -> Result<DisabledScriptPreparation, SessionError> {
        use bounded::{HtmlScriptPreparationAction as Action, HtmlScriptPreparationReply as Reply};
        let (slot, mut metadata) = script_metadata(&self.payloads, control, index)?;
        payload_write::<bounded::HtmlScriptPreparationCursor>(control, 1)?;
        let mut cursor = bounded::HtmlScriptPreparationCursor::new();
        let mut saved_document = None;
        loop {
            let reply = match cursor.action() {
                Action::ReadAlreadyStarted => {
                    Reply::AlreadyStarted(metadata.flags.already_started())
                }
                Action::SaveAndClearParserDocument => {
                    saved_document = metadata.parser_document.take();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::ParserDocumentSaved(saved_document.is_some())
                }
                Action::ReadAsyncPresence => {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    let HtmlNodeKind::Element(element) = &self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                    else {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    };
                    Reply::AsyncPresent(script_attribute(control, element, "async")?.is_some())
                }
                Action::SetForceAsync => {
                    metadata.flags.parser_document_cleared_without_async();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::ReadChildTextEmpty => {
                    let mut empty = true;
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    let children = self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .child_count();
                    for position in 0..children {
                        control.charge(WorkCategory::RecordVisit, 3)?;
                        let child = self
                            .construction
                            .node_at(index)
                            .ok_or_else(|| latch_registry_row(control))?
                            .child_at(position)
                            .ok_or_else(|| latch_registry_row(control))?
                            .position();
                        let record = self
                            .construction
                            .node_at(child)
                            .ok_or(SessionError::ForeignNode)?;
                        if Some(child) != detached
                            && let HtmlNodeKind::Text(text) = record.kind()
                            && !text.is_empty()
                        {
                            empty = false;
                            break;
                        }
                    }
                    // Preserve the original element read after source-text
                    // inspection, including when nonempty text skips src lookup.
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    if !matches!(
                        self.construction
                            .node_at(index)
                            .ok_or_else(|| latch_registry_row(control))?
                            .kind(),
                        HtmlNodeKind::Element(_)
                    ) {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    }
                    Reply::ChildTextEmpty(empty)
                }
                Action::ReadSrcPresence => {
                    let HtmlNodeKind::Element(element) = &self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                    else {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    };
                    Reply::SrcPresent(script_attribute(control, element, "src")?.is_some())
                }
                Action::ReadConnected => {
                    // Connectivity follows the parser-document and empty-source
                    // writes. The actual detached construction view stays here.
                    let mut ancestor = Some(index);
                    let mut connected = false;
                    for _ in 0..self.construction.storage(control)?.len() {
                        let Some(current) = ancestor else { break };
                        control.charge(WorkCategory::RecordVisit, 1)?;
                        let record = self
                            .construction
                            .node_at(current)
                            .ok_or(SessionError::ForeignNode)?;
                        if matches!(record.kind(), HtmlNodeKind::Document(_)) {
                            connected = true;
                            ancestor = None;
                            break;
                        }
                        ancestor = if Some(current) == detached {
                            None
                        } else {
                            record.parent().map(|node| node.position())
                        };
                    }
                    if ancestor.is_some() {
                        return Err(SessionError::Cycle);
                    }
                    Reply::Connected(connected)
                }
                Action::ClassifyType => {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    let HtmlNodeKind::Element(element) = &self
                        .construction
                        .node_at(index)
                        .ok_or_else(|| latch_registry_row(control))?
                        .kind()
                    else {
                        return Err(SessionError::ScriptMetadataUnavailable);
                    };
                    Reply::Classified(classify_script(control, element)?)
                }
                Action::StoreType(kind) => {
                    metadata.script_type = Some(kind);
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::RestoreParserDocument => {
                    metadata.parser_document = saved_document.take();
                    metadata.flags.parser_document_restored();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::MarkAlreadyStarted => {
                    metadata.flags.mark_already_started();
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::StorePreparationDocument => {
                    metadata.preparation_document = Some(self.node_document_ref(control, index)?);
                    set_script_metadata(self, control, slot, &metadata)?;
                    Reply::Applied
                }
                Action::CompareDocuments => {
                    Reply::DocumentsEqual(metadata.parser_document == metadata.preparation_document)
                }
                Action::RequireDisabledScripting => {
                    // The actual ordinary/qualified-fragment construction
                    // capability admits only this closed Disabled execution
                    // domain. This is not inferred from flags or XML class.
                    Reply::Disabled
                }
                Action::Complete(result) => return Ok(result),
            };
            // Account the actual new owned state before each transition. The
            // metadata write boundaries and first reached failure remain intact.
            payload_write::<bounded::HtmlScriptPreparationCursor>(control, 1)?;
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        }
    }

    fn prepare_freeze(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<usize>, SessionError> {
        control.require_identity(&self.control)?;
        self.effects.require_freezable(control)?;
        self.require(control)?;
        self.validate_template_facts(control)?;
        self.validate_svg_facts(control)?;
        self.validate_completion_facts(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let document = matches!(
            self.construction
                .node_at(0)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            HtmlNodeKind::Document(_)
        );
        if let Some(fragment) = &self.fragment {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let output = fragment.output.ok_or(SessionError::ContextUnavailable)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            if !matches!(
                self.construction
                    .node_at(output)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                HtmlNodeKind::Fragment
            ) {
                return Err(SessionError::WrongToken);
            }
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<Option<usize>>() as u64,
            )?;
            Ok(Some(output))
        } else {
            Ok(if document { None } else { Some(0) })
        }
    }

    /// Called only after the owning transition verifies cleanup identity. This
    /// consumes actual storage; exact private tickets prove the release invariant.
    fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        let Self {
            csp_slot: _,
            control: owner,
            effects,
            evidence,
            fragment,
            #[cfg(test)]
                cancel_after_details_shift: _,
            #[cfg(test)]
                cancel_after_radio_peer: _,
            #[cfg(test)]
                cancel_after_removal: _,
            #[cfg(test)]
                cancel_after_textarea_reset: _,
            #[cfg(test)]
                cancel_after_output_reset: _,
            #[cfg(test)]
                cancel_after_input_reset: _,
            #[cfg(test)]
                lose_association_row: _,
            #[cfg(test)]
                cancel_after_range_append: _,
            #[cfg(test)]
                cancel_after_range_ingress: _,
            standalone_ineligibility: _,
            construction,
            payloads,
            failed_text_append,
            style_order: _,
            meta_batches: _,
            #[cfg(test)]
                cancel_after_text_append_chunk: _,
        } = self;
        // Every owner is released even after a refusal; the first one is
        // returned at the end.
        let mut refused = None;
        if let Some(failed) = failed_text_append {
            keep_first(&mut refused, failed.release(control));
        }
        if let Some(fragment) = fragment {
            keep_first(&mut refused, fragment.release(control, &owner));
        }
        if let Some(evidence) = evidence {
            keep_first(&mut refused, evidence.release(control));
        }
        keep_first(&mut refused, effects.release(control));
        keep_first(
            &mut refused,
            release_parts(
                control,
                &owner,
                ReleasedStorage {
                    construction,
                    payloads,
                },
            ),
        );
        refused.map_or(Ok(()), Err)
    }
}

/// Borrowed transitions retain the owner on every failure, including terminal
/// stops. Storage is absent only after successful freeze or explicit cleanup.
impl ParserArenaSession {
    pub(crate) fn validate_standalone_document(
        &self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        if let Some(reason) = storage.standalone_ineligibility {
            return Err(control.stop_unsupported_standalone(reason).into());
        }
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            return Err(control
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into());
        }
        Ok(())
    }
    /// Borrowed sealed-proof qualification, including legitimate checkpoint
    /// reentry. Useful external producers independently enforce readiness.
    pub(crate) fn validate_standalone_proof(
        &self,
        c: &mut ParseControl,
        document: &ParserNode,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        c.require_identity(&storage.control)?;
        c.charge(WorkCategory::RecordVisit, 4)?;
        if let Some(reason) = storage.standalone_ineligibility {
            return Err(c.stop_unsupported_standalone(reason).into());
        }
        if storage
            .construction
            .storage(c)?
            .position(&document.handle)
            .ok()
            != Some(0)
            || !matches!(
                storage
                    .construction
                    .node_at(0)
                    .ok_or_else(|| latch_registry_row(c))?
                    .kind(),
                HtmlNodeKind::Document(_)
            )
        {
            return Err(c
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into());
        }
        Ok(())
    }
    /// The sealed execution owner, not a generic Document constructor, proves
    /// the selected ordinary Disabled environment before any payload transfer.
    pub(crate) fn allocate_disabled_parser_script(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: Option<&ParserNode>,
        context: CreationContext<'_>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        let tag = input.as_ref().ok_or(SessionError::Consumed)?;
        check_disposal_text(control, &tag.name)?;
        tag.attributes.require_disposal_owner(control)?;
        check_text_source(control, &tag.name)?;
        for attribute in tag.attributes.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            check_disposal_text(control, &attribute.name)?;
            check_disposal_text(control, &attribute.value)?;
            control.require_source_range(&attribute.source)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
        }
        if !exact_ascii(control, Some(tag.name.value.text()), "script")? {
            return Err(SessionError::WrongToken);
        }
        self.qualify_creation_context(control, &context)?;
        let parent = match intended_parent {
            Some(parent) => {
                let index = self.active()?.index(control, parent)?;
                self.active()?.node(control, index)?
            }
            None => self.root(control)?,
        };
        self.active()?.node_document_ref(control, parent.index)?;
        let storage = self.active_mut()?;
        storage.document_slot(control, 0)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let inert = storage
            .fragment
            .as_ref()
            .is_some_and(|fragment| fragment.mode == crate::HtmlParserScriptingMode::Inert);
        let prepared = storage.prepare_payload_slot(control)?;
        payload_write::<Option<usize>>(control, 1)?;
        let script = storage
            .payloads
            .push_row(
                prepared,
                RetainedPayload::Script {
                    node: None,
                    metadata: ScriptMetadata {
                        parser_document: Some(DocumentRef::Owned(0)),
                        preparation_document: None,
                        flags: {
                            let mut flags = HtmlScriptFlags::fresh();
                            flags.mark_parser_inserted();
                            if inert {
                                flags.mark_already_started();
                            }
                            flags
                        },
                        script_type: None,
                        result: HtmlScriptResult::Uninitialized,
                        steps_when_result_ready: None,
                    },
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        let node = self.allocate_for_creation(
            control,
            input,
            &parent,
            None,
            CreationProfile::Ordinary,
            context,
        )?;
        let Some(RetainedPayload::Script { node: bound, .. }) =
            self.active_mut()?.payloads.values.get_mut(script)
        else {
            return Err(latch_registry_row(control));
        };
        *bound = Some(node.index);
        self.active_mut()?
            .payloads
            .join_row(script)
            .map_err(|_| latch_registry_row(control))?;
        Ok(node)
    }
    #[cfg(test)]
    pub(crate) fn script_metadata(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ScriptMetadata, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        script_metadata(&storage.payloads, control, node).map(|(_, metadata)| metadata)
    }
    #[cfg(test)]
    pub(crate) fn script_parser_document(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.script_metadata(control, node)?
            .parser_document
            .map(|index| self.active()?.node(control, index.owned()?))
            .transpose()
    }
    #[cfg(test)]
    pub(crate) fn script_preparation_document(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.script_metadata(control, node)?
            .preparation_document
            .map(|index| self.active()?.node(control, index.owned()?))
            .transpose()
    }
    /// Text-mode EOF marks the current script started; it does not prepare it.
    pub(crate) fn mark_script_eof(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        capability.validate_session(control, self)?;
        let storage = self.active_mut()?;
        let index = storage.index(control, node)?;
        let (slot, mut metadata) = script_metadata(&storage.payloads, control, index)?;
        metadata.flags.mark_already_started();
        set_script_metadata(storage, control, slot, &metadata)
    }
    /// Exact preparation prefix through HTML's scripting-disabled step18.
    /// No loader, compilation, result or execution callback is invoked.
    pub(crate) fn prepare_disabled_script(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<DisabledScriptPreparation, SessionError> {
        capability.validate_session(control, self)?;
        let storage = self.active_mut()?;
        let index = storage.index(control, node)?;
        storage.prepare_script_owned(control, index, None)
    }
    pub(crate) fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        Ok(Self {
            storage: Some(SessionStorage::new(control, root)?),
        })
    }
    fn active(&self) -> Result<&SessionStorage, SessionError> {
        self.storage.as_ref().ok_or(SessionError::Consumed)
    }
    fn active_mut(&mut self) -> Result<&mut SessionStorage, SessionError> {
        self.storage.as_mut().ok_or(SessionError::Consumed)
    }
    pub(crate) fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.active()?.root(control)
    }
    /// Preexisting immutable input only; this does not construct parser foreign
    /// names or admit the input backing physically.
    #[cfg(test)]
    pub(crate) fn allocate_element(
        &mut self,
        control: &mut ParseControl,
        element: HtmlElement,
    ) -> Result<ParserNode, SessionError> {
        self.active_mut()?.allocate_element(control, element)
    }
    pub(crate) fn allocate_html_tag(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
    ) -> Result<ParserNode, SessionError> {
        self.active_mut()?.allocate_html_tag(control, input)
    }
    /// Actual fresh ordinary parser creation context. The source form pointer
    /// remains driver-owned; exact admitted form-associated kinds own metadata.
    pub(crate) fn allocate_html_tag_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: &ParserNode,
        current_form: Option<&ParserNode>,
        context: CreationContext<'_>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_for_creation(
            control,
            input,
            intended_parent,
            current_form,
            CreationProfile::Ordinary,
            context,
        )
    }
    pub(crate) fn allocate_input_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        location: CreationLocation<'_>,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        self.allocate_for_creation(
            control,
            input,
            location.intended_parent,
            location.current_form,
            CreationProfile::Input(capability, token_source),
            location.context,
        )
    }
    pub(crate) fn allocate_details_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        location: CreationLocation<'_>,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        self.allocate_for_creation(
            control,
            input,
            location.intended_parent,
            location.current_form,
            CreationProfile::Details(capability, token_source),
            location.context,
        )
    }
    pub(crate) fn allocate_select_family_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        location: CreationLocation<'_>,
        token_source: &crate::HtmlSourceRange,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<ParserNode, SessionError> {
        capability.validate_session(control, self)?;
        self.allocate_for_creation(
            control,
            input,
            location.intended_parent,
            location.current_form,
            CreationProfile::Select(capability, token_source),
            location.context,
        )
    }
    fn allocate_for_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        intended_parent: &ParserNode,
        current_form: Option<&ParserNode>,
        profile: CreationProfile<'_, '_>,
        context: CreationContext<'_>,
    ) -> Result<ParserNode, SessionError> {
        self.validate_standalone_document(control)?;
        let parsing_template = self.qualify_creation_context(control, &context)?;
        let storage = self.active()?;
        let parent = storage.index(control, intended_parent)?;
        storage.container(control, parent)?;
        if self
            .active_mut()?
            .associated_document_parent(control, parent)?
        {
            return Err(control
                .stop_unsupported_standalone(StandaloneUnsupported::ScriptDocument)
                .into());
        }
        let storage = self.active()?;
        let document = storage.node_document_ref(control, parent)?;
        let form = current_form
            .map(|form| storage.index(control, form))
            .transpose()?;
        if let Some(form) = form
            && !storage.html_local(control, form, "form")?
        {
            return Err(SessionError::WrongToken);
        }
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_html_tag(control, tag)?;
        check_text_source(control, &tag.name)?;
        for (index, attribute) in tag.attributes.as_slice().iter().enumerate() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.require_source_range(&attribute.source)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
            // No recoverable duplicate rejection may follow an initialized
            // metadata slot. Compare actual unique no-namespace token names.
            for earlier in tag.attributes.as_slice().iter().take(index) {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if exact_string_equal(
                    control,
                    earlier.name.value.text(),
                    attribute.name.value.text(),
                )? {
                    return Err(SessionError::DuplicateAttribute);
                }
            }
        }
        let mut kind = bounded::native_form_association_with(|candidate| {
            // These two metadata families are admitted only below, after the
            // actual profile/capability/origin checks. Avoid inapplicable reads.
            if matches!(
                candidate,
                KnownHtmlElement::Input | KnownHtmlElement::Select
            ) {
                return Ok(false);
            }
            control.charge(WorkCategory::RecordVisit, 1)?;
            exact_ascii(control, Some(tag.name.value.text()), candidate.local_name())
        })?;
        let token_source = match profile {
            CreationProfile::Ordinary => None,
            CreationProfile::Details(capability, source)
            | CreationProfile::Input(capability, source)
            | CreationProfile::Select(capability, source) => {
                // Concrete borrowed capability, not a caller-supplied eligibility flag.
                capability.validate_session(control, self)?;
                Some(source)
            }
        };
        let details_domain = if let CreationProfile::Details(capability, _) = profile {
            if !exact_ascii(control, Some(tag.name.value.text()), "details")? {
                return Err(SessionError::WrongToken);
            }
            let closed = capability.details_notification_domain(control, self)?;
            for attr in tag.attributes.as_slice() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attr.name.value.text()), "open")? && !closed {
                    return Err(control
                        .stop_unsupported_standalone(StandaloneUnsupported::DetailsNotification)
                        .into());
                }
            }
            Some(if closed {
                details::NotificationDomain::ClosedOwned
            } else {
                details::NotificationDomain::Unavailable
            })
        } else {
            None
        };
        let select_metadata = if matches!(profile, CreationProfile::Select(..)) {
            select::initial_metadata(control, tag)?
        } else {
            None
        };
        if matches!(profile, CreationProfile::Select(..))
            && exact_ascii(control, Some(tag.name.value.text()), "select")?
        {
            kind = Some(HtmlNativeFormAssociationKind::Select);
        }
        // Every creation audits the rows registered since the previous one, so no
        // creation pays for the registry its predecessors built.
        let storage = self.active_mut()?;
        storage.payloads.check_new_aliases(
            ConstructionView::Active(storage.construction.storage(control)?),
            control,
        )?;
        if !matches!(profile, CreationProfile::Ordinary) {
            if matches!(profile, CreationProfile::Input(..))
                && !exact_ascii(control, Some(tag.name.value.text()), "input")?
            {
                return Err(SessionError::WrongToken);
            }
            let source = token_source.ok_or(SessionError::InputMetadataUnavailable)?;
            control.require_source_range(source)?;
            // Validate every original field against the actual token, including
            // empty-value attributes, before retaining any construction slot.
            check_formatting_tag(control, tag, source)?;
            if matches!(profile, CreationProfile::Input(..)) {
                validate_initial_input_origins(control, tag)?;
                kind = Some(HtmlNativeFormAssociationKind::Input);
            }
        }
        let mut explicit_form = false;
        if kind.is_some_and(HtmlNativeFormAssociationKind::listed) {
            for attribute in tag.attributes.as_slice() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attribute.name.value.text()), "form")? {
                    explicit_form = true;
                    break;
                }
            }
        }
        // §13.2.6.1's fragment-context-null guard is independent of a retained
        // nearest-form pointer used by insertion-mode recovery.
        let owner = if kind.is_some()
            && !explicit_form
            && !parsing_template
            && storage.fragment.is_none()
        {
            match form {
                Some(form)
                    if storage.ordinary_root(control, parent, None)?
                        == storage.ordinary_root(control, form, None)? =>
                {
                    Some(form)
                }
                _ => None,
            }
        } else {
            None
        };
        let value_source = if kind == Some(HtmlNativeFormAssociationKind::Input) {
            let mut selected = None;
            for attribute in tag.attributes.as_slice() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attribute.name.value.text()), "value")? {
                    selected = Some(&attribute.source);
                    break;
                }
            }
            selected
        } else {
            None
        };
        let input_sources = if let Some(token) =
            token_source.filter(|_| kind == Some(HtmlNativeFormAssociationKind::Input))
        {
            control.charge(
                WorkCategory::ProviderCall,
                if value_source.is_some() { 2 } else { 1 },
            )?;
            payload_write::<(crate::HtmlSourceRange, Option<crate::HtmlSourceRange>)>(control, 1)?;
            Some((token.clone(), value_source.cloned()))
        } else {
            None
        };
        let template = exact_ascii(control, Some(tag.name.value.text()), "template")?;
        let storage = self.active_mut()?;
        let mut prepared =
            storage.prepare_document_allocation(control, Some(document), template)?;
        let mut is_value = None;
        for attr in tag.attributes.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(attr.name.value.text()), "is")? {
                is_value = Some(&attr.value);
                break;
            }
        }
        storage.prepare_creation_registry(
            control,
            parent,
            tag.name.value.text(),
            is_value,
            true,
            &mut prepared,
        )?;
        let link_slot = if exact_ascii(control, Some(tag.name.value.text()), "link")? {
            storage.document_slot(control, 0)?;
            let prepared = storage.prepare_payload_slot(control)?;
            payload_write::<(Option<usize>, Option<DocumentRef>)>(control, 1)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Link {
                        node: None,
                        parser_document: Some(DocumentRef::Owned(0)),
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let details_slot = if let Some(notification) = details_domain {
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Details {
                        node: None,
                        notification,
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let select_slot = if let Some(state) = select_metadata {
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::SelectFamily { node: None, state },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        // Reserve actual independent empty raw backing and initialized metadata
        // BEFORE lexical ownership transfer. A stopped unbound slot is cleanup-only.
        let textarea_slot = if kind == Some(HtmlNativeFormAssociationKind::Textarea) {
            let empty = literal_string(control, b"")?;
            let raw = storage.register_raw(control, empty)?;
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Textarea {
                        node: None,
                        state: TextareaState {
                            raw,
                            flags: HtmlTextareaFlags::default(),
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let output_slot = if kind == Some(HtmlNativeFormAssociationKind::Output) {
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Output {
                        node: None,
                        state: OutputState {
                            default_value_override: OutputDefaultValueOverride::Null,
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let input_slot = if kind == Some(HtmlNativeFormAssociationKind::Input) {
            let empty = literal_string(control, b"")?;
            let raw = storage.register_raw(control, empty)?;
            let prepared = storage.prepare_payload_slot(control)?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::Input {
                        node: None,
                        raw,
                        complete: false,
                        state: InputState {
                            flags: crate::HtmlInputFlags::fresh(),
                            cursor: 0,
                            selected_files_empty: true,
                            value_rewritten_during_creation: false,
                            email_value_count: None,
                            image: input_image::ImageState::NeverEntered,
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let source_slot = if let Some((token, value_attribute)) = input_sources {
            let entries = if value_attribute.is_some() { 2 } else { 1 };
            control.acquire(HtmlParseResource::SourceMapEntries, entries)?;
            let prepared = match storage.prepare_payload_slot(control) {
                Ok(prepared) => prepared,
                Err(error) => {
                    // The slot refusal is the error reported; a refused logical
                    // release has latched its own stop in the controller.
                    let _ = release_logical(
                        control,
                        &storage.control,
                        HtmlParseResource::SourceMapEntries,
                        entries,
                    );
                    return Err(error);
                }
            };
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::InputSource {
                        node: None,
                        token,
                        value_attribute,
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let reset_before_association = textarea_slot.is_some()
            || output_slot.is_some()
            || input_slot.is_some()
            || kind == Some(HtmlNativeFormAssociationKind::Select);
        let slot = if let Some(kind) = kind {
            storage.require(control)?;
            let prepared = storage.prepare_payload_slot_with(
                control,
                Some(KindList::FormAssociation),
                explicit_form,
            )?;
            let slot = storage
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::FormAssociation {
                        node: None,
                        state: FormAssociationState {
                            kind,
                            owner: if reset_before_association {
                                None
                            } else {
                                owner
                            },
                            parser_inserted: !reset_before_association && owner.is_some(),
                            evidence: if !reset_before_association && owner.is_some() {
                                FormAssociationEvidence::ParserAssociated
                            } else {
                                FormAssociationEvidence::Created
                            },
                        },
                    },
                )
                .map_err(|_| latch_registry_row(control))?;
            Some(slot)
        } else {
            None
        };
        let result = (|| {
            // The creation metadata is resolved before the tag leaves its
            // caller, so an impossible state keeps the token owned there.
            let creation = if let Some(metadata) = details_slot {
                Some(OrderedCreation::Details { metadata })
            } else if let Some(input) = input_slot {
                let (Some(association), Some(source)) = (slot, source_slot) else {
                    return Err(latch_registry_row(control));
                };
                Some(OrderedCreation::Input {
                    input,
                    association,
                    source,
                })
            } else if matches!(profile, CreationProfile::Select(..)) {
                Some(OrderedCreation::Select {
                    metadata: select_slot,
                })
            } else {
                None
            };
            let Tag {
                name, attributes, ..
            } = take_payload(input, control)?;
            let node = match creation {
                Some(creation) => storage
                    .allocate_ordered_payload(control, name, attributes, prepared, creation)?,
                None => storage.allocate_html_payload_prepared(
                    control,
                    name.value,
                    Some(name.origins),
                    attributes,
                    prepared,
                )?,
            };
            storage.mark_document_parser_style(control, node.index)?;
            if let Some(link) = link_slot {
                payload_write::<Option<usize>>(control, 1)?;
                let Some(RetainedPayload::Link { node: bound, .. }) =
                    storage.payloads.values.get_mut(link)
                else {
                    return Err(latch_registry_row(control));
                };
                *bound = Some(node.index);
            }
            if let Some(textarea) = textarea_slot {
                control.charge(WorkCategory::RecordVisit, 2)?;
                if storage
                    .construction
                    .node_at(node.index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_count()
                    != 0
                {
                    return Err(SessionError::WrongToken);
                }
                // Actual fresh node has no children. Its already admitted empty
                // raw is therefore exactly child text; reset precedes association.
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<TextareaState>(control, 1)?;
                let Some(RetainedPayload::Textarea { node: bound, state }) =
                    storage.payloads.values.get_mut(textarea)
                else {
                    return Err(latch_registry_row(control));
                };
                *bound = Some(node.index);
                state.flags.reset();
                storage
                    .payloads
                    .join_textarea(node.index, textarea)
                    .map_err(|_| latch_registry_row(control))?;
                #[cfg(test)]
                if let Some(cancellation) = storage.cancel_after_textarea_reset.take() {
                    cancellation.cancel();
                }
            }
            if let Some(output) = output_slot {
                // Syntax3537 reset: actual freshly allocated output has a null
                // override and empty descendants. DOM1593/716 therefore replaces
                // all with null: zero removed/added children, no mutation record.
                control.charge(WorkCategory::RecordVisit, 2)?;
                if storage
                    .construction
                    .node_at(node.index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_count()
                    != 0
                {
                    return Err(SessionError::WrongToken);
                }
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<OutputState>(control, 1)?;
                let Some(RetainedPayload::Output { node: bound, state }) =
                    storage.payloads.values.get_mut(output)
                else {
                    return Err(latch_registry_row(control));
                };
                *bound = Some(node.index);
                state.default_value_override = OutputDefaultValueOverride::Null;
                #[cfg(test)]
                if let Some(cancellation) = storage.cancel_after_output_reset.take() {
                    cancellation.cancel();
                }
            }
            #[cfg(test)]
            if let Some(lost) = storage.lose_association_row.as_mut()
                && let Some(slot) = slot
                && let Some(row) = storage.payloads.values.get_mut(slot)
            {
                *lost = Some((slot, mem::replace(row, RetainedPayload::Retired)));
            }
            if let Some(slot) = slot {
                control.charge(WorkCategory::RecordVisit, 1)?;
                payload_write::<Option<usize>>(control, 1)?;
                let Some(RetainedPayload::FormAssociation { node: bound, .. }) =
                    storage.payloads.values.get_mut(slot)
                else {
                    return Err(SessionError::FormAssociationMetadataUnavailable);
                };
                *bound = Some(node.index);
                storage
                    .payloads
                    .join_row(slot)
                    .map_err(|_| latch_registry_row(control))?;
                if reset_before_association {
                    payload_write::<FormAssociationState>(control, 1)?;
                    let Some(RetainedPayload::FormAssociation { state, .. }) =
                        storage.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    state.owner = owner;
                    state.parser_inserted = owner.is_some();
                    state.evidence = if owner.is_some() {
                        FormAssociationEvidence::ParserAssociated
                    } else {
                        FormAssociationEvidence::Created
                    };
                }
                if reset_before_association && owner.is_some() && input_slot.is_some() {
                    storage.normalize_radio(control, node.index, None)?;
                }
            }
            Ok(node)
        })();
        match result {
            Err(error)
                if slot.is_some()
                    || link_slot.is_some()
                    || matches!(
                        profile,
                        CreationProfile::Select(..) | CreationProfile::Details(..)
                    ) =>
            {
                Err(latch_creation_fault(control, error))
            }
            other => other,
        }
    }

    #[cfg(test)]
    pub(crate) fn input_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<InputState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        Ok(find_input_state(&storage.payloads, control, node)?.1)
    }
    /// Returns original token/value-attribute ranges only when creation actually
    /// rewrote the native value. A completed no-writeback input returns None;
    /// unavailable/imported/incomplete metadata returns its typed error.
    #[cfg(test)]
    pub(crate) fn input_writeback_sources<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError>
    {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        find_input_sources(&storage.payloads, control, index)
    }
    #[cfg(test)]
    pub(crate) fn append_input_internal_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        let raw = find_input_state(&storage.payloads, control, node)?.0;
        destination.extend_units(
            textarea_raw(&storage.payloads, control, raw)?.code_units(),
            control,
        )?;
        Ok(())
    }
    #[cfg(test)]
    /// Native input value readback, distinct from internal and default value.
    pub(crate) fn append_input_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let state = self.input_state(control, node)?;
        let storage = self.active()?;
        append_native_input_value(
            ConstructionView::Active(storage.construction.test_storage()),
            &storage.payloads,
            control,
            node.index,
            state,
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_input_default_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.input_state(control, node)?;
        append_input_content(
            ConstructionView::Active(self.active()?.construction.test_storage()),
            control,
            node.index,
            destination,
            &[],
        )
    }

    #[cfg(test)]
    pub(crate) fn image_form_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<FormAssociationState, SessionError> {
        let state = self
            .form_association_state(control, node)
            .map_err(|error| match error {
                SessionError::FormAssociationMetadataUnavailable => {
                    SessionError::ImageFormMetadataUnavailable
                }
                other => other,
            })?;
        if state.kind != HtmlNativeFormAssociationKind::Image {
            return Err(SessionError::ImageFormMetadataUnavailable);
        }
        Ok(state)
    }
    #[cfg(test)]
    pub(crate) fn output_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<OutputState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        find_output_state(&storage.payloads, control, node)
    }
    #[cfg(test)]
    pub(crate) fn append_output_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.output_state(control, node)?;
        append_descendant_text(
            ConstructionView::Active(self.active()?.construction.test_storage()),
            control,
            node.index,
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_output_default_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        // Actual retained Null override delegates to descendant text. There is
        // no nonnull override/setter producer or guessed stored default.
        self.append_output_value(control, node, destination)
    }
    #[cfg(test)]
    pub(crate) fn textarea_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<TextareaState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        storage
            .textarea_slot(control, node)?
            .map(|(_, state)| state)
            .ok_or(SessionError::TextareaMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn textarea_raw_value<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<&'a DocumentString, SessionError> {
        let state = self.textarea_state(control, node)?;
        textarea_raw(&self.active()?.payloads, control, state.raw)
    }
    /// Called after a real parser stack removal, never during terminal cleanup.
    pub(crate) fn parser_stack_removed(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        self.qualify_stack_removal(control, node, capability)?;
        let storage = self.active_mut()?;
        let node = storage.index(control, node)?;
        storage.style_stack_changed(control, node, false)?;
        if storage.html_local(control, node, "option")? {
            storage.update_for_option(control, node)?;
            return storage.drain_construction_effects(control);
        }
        if !storage.html_local(control, node, "textarea")? {
            return Ok(());
        }
        match storage.textarea_slot(control, node) {
            Ok(Some(_)) => {}
            // No state row, or a row of another kind, for a textarea.
            Ok(None) | Err(SessionError::WrongToken | SessionError::ForeignNode) => {
                return Err(latch_registry_row(control));
            }
            Err(error) => return Err(error),
        }
        match storage.refresh_textarea(control, node, None, true) {
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(error) => Err(latch_effect_fault(control, error)),
            Ok(()) => Ok(()),
        }
    }
    pub(crate) fn qualify_stack_removal(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        let index = storage.index(c, node)?;
        if storage.html_local(c, index, "option")? {
            let capability = capability.ok_or_else(|| {
                SessionError::Stop(c.stop_unsupported_standalone(StandaloneUnsupported::Provenance))
            })?;
            capability.validate_session(c, self)?;
            storage.select_metadata(c, index)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn image_form_owner(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.image_form_state(control, node)?
            .owner
            .map(|owner| self.active()?.node(control, owner))
            .transpose()
    }
    #[cfg(test)]
    pub(crate) fn form_association_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<FormAssociationState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        find_form_association_state(&storage.payloads, control, node)?
            .map(|(_, state)| state)
            .ok_or(SessionError::FormAssociationMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn form_association_owner(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.form_association_state(control, node)?
            .owner
            .map(|owner| self.active()?.node(control, owner))
            .transpose()
    }
    pub(crate) fn add_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        self.active_mut()?
            .add_attributes_if_missing(control, target, input)
    }
    pub(crate) fn allocate_implicit_html(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, None, ImpliedHtmlName::Html)
    }
    pub(crate) fn allocate_implicit_head(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Head)
    }
    pub(crate) fn allocate_implicit_body(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Body)
    }
    pub(crate) fn allocate_implicit_p(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::P)
    }
    pub(crate) fn allocate_implicit_colgroup(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Colgroup)
    }
    pub(crate) fn allocate_implicit_tbody(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Tbody)
    }
    pub(crate) fn allocate_implicit_tr(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, intended_parent, ImpliedHtmlName::Tr)
    }
    pub(crate) fn add_body_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        if let Some(prepared) = storage.prepare_named_attribute_merge(
            control,
            target,
            input,
            AttributeMergeName::Body,
        )? {
            storage.finish_attribute_merge(control, prepared)?;
        }
        Ok(())
    }
    /// Pinned InBody `image` recovery rewrites the actual token name only.
    /// The exact original name range is retained as RecoverySyntax; all other
    /// lexical fields and their owners remain in their original slots.
    pub(crate) fn recover_image_tag(
        &mut self,
        control: &mut ParseControl,
        source: &crate::HtmlSource,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        self.active()?.require(control)?;
        control.require_source(source)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        SessionStorage::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), "image")? {
            return Err(SessionError::WrongToken);
        }
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(attribute) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            SessionStorage::check_text(control, &mut attribute.name)?;
            SessionStorage::check_text(control, &mut attribute.value)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
            control.require_source_range(&attribute.source)?;
        }
        let mut covered = 0;
        let mut start = None;
        let mut end = None;
        for origin in tag.name.origins.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.require_source_range(&origin.source)?;
            if origin.start != covered
                || origin.end <= origin.start
                || origin.end > tag.name.value.text().len()
                || end.is_some_and(|end| end != origin.source.start())
            {
                return Err(SessionError::OriginSplitUnavailable);
            }
            start.get_or_insert(origin.source.start());
            end = Some(origin.source.end());
            covered = origin.end;
        }
        if covered != tag.name.value.text().len() {
            return Err(SessionError::OriginSplitUnavailable);
        }
        let start = start.ok_or(SessionError::OriginSplitUnavailable)?;
        let end = end.ok_or(SessionError::OriginSplitUnavailable)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<crate::HtmlSourceRange>(control, 1)?;
        // HtmlSource::range reads each interior UTF8 endpoint once. Pre-admit
        // both reads together immediately before that bounded native helper.
        if source.as_utf8().is_some() {
            let reads = [start, end]
                .into_iter()
                .filter(|&endpoint| endpoint != 0 && endpoint != source.len())
                .count() as u64;
            control.charge(WorkCategory::InputUnitRead, reads)?;
        }
        let range = source
            .range(start, end)
            .map_err(|_| SessionError::OriginSplitUnavailable)?;
        let value = literal_string(control, b"img")?;
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        let prepared = (|| {
            origins.try_push(
                control,
                TextOrigin {
                    start: 0,
                    end: 3,
                    source: range,
                    kind: crate::parser::tokenizer::token::OriginKind::RecoverySyntax,
                },
            )?;
            payload_write::<Text>(control, 1)
        })();
        if let Err(error) = prepared {
            value.release(control)?;
            origins.release(control)?;
            return Err(error);
        }
        let original = mem::replace(&mut tag.name, Text { value, origins });
        original.release(control)?;
        Ok(())
    }
    /// Pinned end-br recovery discards actual attributes before start-tag
    /// reprocessing. Name text/origins stay lexical: their units did not change;
    /// the driver retains the original end-token range and recovery diagnostic.
    pub(crate) fn recover_br_end_tag(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        self.active()?.require(control)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::End {
            return Err(SessionError::WrongToken);
        }
        SessionStorage::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), "br")? {
            return Err(SessionError::WrongToken);
        }
        check_text_source(control, &tag.name)?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(attribute) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            SessionStorage::check_text(control, &mut attribute.name)?;
            SessionStorage::check_text(control, &mut attribute.value)?;
            check_text_source(control, &attribute.name)?;
            check_text_source(control, &attribute.value)?;
            control.require_source_range(&attribute.source)?;
        }
        let empty = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
        payload_write::<OwnedVec<crate::parser::tokenizer::token::Attribute>>(control, 1)?;
        payload_write::<TagKind>(control, 1)?;
        let original = mem::replace(&mut tag.attributes, empty);
        // This disposal finishes all nested reclamation even on a newly latched
        // stop. The retained Tag then owns its unchanged name and empty attrs;
        // kind remains End unless the complete disposal succeeded.
        release_token_attributes(original, control)?;
        tag.kind = TagKind::Start;
        Ok(())
    }
    fn allocate_implied_html(
        &mut self,
        control: &mut ParseControl,
        intended_parent: Option<&ParserNode>,
        local: ImpliedHtmlName,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let document = storage.creation_document(control, intended_parent)?;
        let ordinary_document = storage.fragment.is_none() && document.is_some();
        let mut prepared = storage.prepare_document_allocation(control, document, false)?;
        let attributes = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
        let name = literal_string(
            control,
            match local {
                ImpliedHtmlName::Html => b"html",
                ImpliedHtmlName::Head => b"head",
                ImpliedHtmlName::Body => b"body",
                ImpliedHtmlName::P => b"p",
                ImpliedHtmlName::Colgroup => b"colgroup",
                ImpliedHtmlName::Tbody => b"tbody",
                ImpliedHtmlName::Tr => b"tr",
            },
        )?;
        // Fragment bootstrap establishes its captured root registry separately;
        // an ordinary implied root instead belongs to the actual root Document.
        let parent = intended_parent
            .map(|parent| parent.index)
            .or_else(|| ordinary_document.then_some(0));
        if let Some(parent) = parent
            && let Err(error) = storage.prepare_creation_registry(
                control,
                parent,
                name.text(),
                None,
                true,
                &mut prepared,
            )
        {
            name.release(control)?;
            attributes.release(control)?;
            return Err(error);
        }
        storage.allocate_html_payload_prepared(control, name, None, attributes, prepared)
    }
    pub(crate) fn allocate_comment(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Text>,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        SessionStorage::check_text(control, input.as_mut().ok_or(SessionError::Consumed)?)?;
        let document = storage.creation_document(control, intended_parent)?;
        let prepared = storage.prepare_document_allocation(control, document, false)?;
        let text = take_payload(input, control)?;
        let (value, ticket) = storage.register_text(control, text)?;
        control.charge(WorkCategory::RecordWrite, 2)?;
        let node = storage.allocate_prepared_owned(
            control,
            creation_command(HtmlNodeKind::Comment(value)),
            false,
            0,
            0,
            prepared,
        )?;
        storage
            .bind_string(ticket, StringLocation::Comment(node.index))
            .map_err(|_| latch_registry_row(control))?;
        Ok(node)
    }
    pub(crate) fn allocate_processing_instruction(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<(Text, Text)>,
        intended_parent: Option<&ParserNode>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let (target, data) = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_text(control, target)?;
        SessionStorage::check_text(control, data)?;
        let document = storage.creation_document(control, intended_parent)?;
        let prepared = storage.prepare_document_allocation(control, document, false)?;
        let map = storage.prepare_pi_map(control)?;
        payload_write::<Option<usize>>(control, 1)?;
        let (target, data) = take_payload(input, control)?;
        let (target, target_ticket) = match storage.register_text(control, target) {
            Ok(value) => value,
            Err(error) => {
                data.release(control)?;
                return Err(error);
            }
        };
        let (data, data_ticket) = storage.register_text(control, data)?;
        control.charge(WorkCategory::RecordWrite, 3)?;
        // Recovered PI data is exact storage, not checked native authoring input.
        let node = storage.allocate_prepared_owned(
            control,
            creation_command(HtmlNodeKind::ProcessingInstruction(
                HtmlProcessingInstruction::new(target, data),
            )),
            false,
            0,
            0,
            prepared,
        )?;
        storage
            .bind_string(
                target_ticket,
                StringLocation::ProcessingInstructionTarget(node.index),
            )
            .map_err(|_| latch_registry_row(control))?;
        storage
            .bind_string(
                data_ticket,
                StringLocation::ProcessingInstructionData(node.index),
            )
            .map_err(|_| latch_registry_row(control))?;
        storage.bind_pi_map(control, map, node.index)?;
        storage.parse_pi_map(control, node.index, map)?;
        Ok(node)
    }
    pub(crate) fn allocate_doctype(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Doctype>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let doctype = input.as_mut().ok_or(SessionError::Consumed)?;
        for text in [
            &mut doctype.name,
            &mut doctype.public_id,
            &mut doctype.system_id,
        ]
        .into_iter()
        .flatten()
        {
            SessionStorage::check_text(control, text)?;
        }
        let Doctype {
            name,
            public_id,
            system_id,
            ..
        } = take_payload(input, control)?;
        let mut incoming = [name, public_id, system_id];
        let mut registered: [Option<(DocumentString, usize)>; 3] = [None, None, None];
        let result = (|| {
            let mut units = 0usize;
            for (field, slot) in incoming.iter_mut().zip(registered.iter_mut()) {
                payload_write::<Option<Text>>(control, 2)?;
                let value = if let Some(text) = field.take() {
                    storage.register_text(control, text)?
                } else {
                    let empty = literal_string(control, b"")?;
                    storage.register_string(control, empty, None)?
                };
                control.charge(WorkCategory::RecordVisit, 1)?;
                units = units.checked_add(value.0.len()).ok_or_else(|| {
                    control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                })?;
                payload_write::<Option<(DocumentString, usize)>>(control, 2)?;
                *slot = Some(value);
            }
            payload_write::<[Option<(DocumentString, usize)>; 3]>(control, 3)?;
            let [Some((name, a)), Some((public_id, b)), Some((system_id, d))] = registered else {
                return Err(latch_registry_row(control));
            };
            control.charge(WorkCategory::RecordWrite, 4)?;
            // HTML's DOM creation step lowers missing token fields to empty
            // strings. Native authored Option data keeps its separate meaning.
            let node = storage.allocate(
                control,
                HtmlNodeKind::Doctype(HtmlDoctype::new(name, Some(public_id), Some(system_id))),
                false,
                0,
                0,
            )?;
            storage
                .bind_string(a, StringLocation::DoctypeName(node.index))
                .map_err(|_| latch_registry_row(control))?;
            storage
                .bind_string(b, StringLocation::DoctypePublic(node.index))
                .map_err(|_| latch_registry_row(control))?;
            storage
                .bind_string(d, StringLocation::DoctypeSystem(node.index))
                .map_err(|_| latch_registry_row(control))?;
            Ok(node)
        })();
        for text in incoming.into_iter().flatten() {
            text.release(control)?;
        }
        result
    }
    pub(crate) fn document_accepts_doctype(
        &self,
        control: &mut ParseControl,
    ) -> Result<bool, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            return Err(SessionError::NotDocument);
        }
        for position in 0..storage
            .construction
            .node_at(0)
            .ok_or_else(|| latch_registry_row(control))?
            .child_count()
        {
            control.charge(WorkCategory::RecordVisit, 2)?;
            let child = storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .child_at(position)
                .ok_or_else(|| latch_registry_row(control))?
                .position();
            if matches!(
                storage
                    .construction
                    .node_at(child)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                HtmlNodeKind::Doctype(_) | HtmlNodeKind::Element(_)
            ) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub(crate) fn document_mode(
        &self,
        control: &mut ParseControl,
    ) -> Result<HtmlDocumentMode, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Document(mode) = &storage
            .construction
            .node_at(0)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::NotDocument);
        };
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<HtmlDocumentMode>() as u64,
        )?;
        Ok(*mode)
    }
    pub(crate) fn set_document_mode(
        &mut self,
        control: &mut ParseControl,
        mode: HtmlDocumentMode,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            return Err(SessionError::NotDocument);
        }
        payload_write::<HtmlDocumentMode>(control, 1)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        let Some(document) = storage.construction.node_at(0).map(|root| root.handle()) else {
            return Err(latch_registry_row(control));
        };
        storage.construction.apply(
            control,
            bounded::HtmlConstructionCommand::SetDocumentMode { document, mode },
            &[],
        )?;
        Ok(())
    }
    pub(crate) fn insert_text(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        before: Option<&ParserNode>,
        input: &mut Option<Text>,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<Option<TextInsertion>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, parent)?;
        if storage.html_local(control, index, "script")?
            && script_metadata(&storage.payloads, control, index)?
                .1
                .parser_document
                .is_none()
        {
            let capability = capability.ok_or_else(|| {
                SessionError::Stop(
                    control.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
                )
            })?;
            capability.validate_session(control, self)?;
        }
        self.active_mut()?
            .insert_text(control, parent, before, input)
    }
    pub(crate) fn fragment_root(
        &self,
        control: &mut ParseControl,
    ) -> Result<Option<ParserNode>, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Fragment
        ) {
            Ok(Some(storage.root(control)?))
        } else {
            Ok(None)
        }
    }
    pub(crate) fn validate_node(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        self.active()?.index(control, node)?;
        Ok(())
    }
    /// The validated arena position that is the node's identity: two nodes are
    /// the same node exactly when their positions are equal.
    pub(crate) fn node_position(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<usize, SessionError> {
        self.active()?.index(control, node)
    }
    pub(crate) fn same_node(
        &self,
        control: &mut ParseControl,
        left: &ParserNode,
        right: &ParserNode,
    ) -> Result<bool, SessionError> {
        Ok(self.active()?.index(control, left)? == self.active()?.index(control, right)?)
    }
    pub(crate) fn parent(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        storage
            .construction
            .node_at(index)
            .ok_or_else(|| latch_registry_row(control))?
            .parent()
            .map(|node| storage.node(control, node.position()))
            .transpose()
    }
    pub(crate) fn element_name<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<&'a QualifiedName>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(
            match &storage
                .construction
                .node_at(index)
                .ok_or_else(|| latch_registry_row(control))?
                .kind()
            {
                HtmlNodeKind::Element(value) => Some(value.name()),
                _ => None,
            },
        )
    }
    /// The facts required by HTML's insert-an-element operation (§13.2.6.1).
    /// This observes the existing arena, without allocating or moving a node.
    /// Association ancestry comes only from an actual template/shadow owner;
    /// an associated fragment's absent ordinary parent is not its ancestry.
    pub(crate) fn element_insertion_ignored(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
        target_parent: &ParserNode,
    ) -> Result<Option<ElementInsertionIgnored>, SessionError> {
        let storage = self.active()?;
        let element = storage.index(control, element)?;
        let target = storage.index(control, target_parent)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let record = storage
            .construction
            .node_at(element)
            .ok_or(SessionError::ForeignNode)?;
        if !matches!(record.kind(), HtmlNodeKind::Element(_)) {
            return Err(SessionError::WrongToken);
        }
        if record.parent().is_some() {
            return Ok(Some(ElementInsertionIgnored::ParentAlreadyPresent));
        }
        // An element with no children, no shadow root and, if it owns template
        // contents, empty and shadowless ones has nothing below it but those
        // contents. It is then a host-including inclusive ancestor of the
        // target only as the target itself or as the owner of the target that
        // is its contents. Every element the parser inserts is such an element.
        let contents = match record.template_contents_kind() {
            Some(HtmlTemplateContentsKind::Owned) => record.template_contents(),
            _ => None,
        };
        control.charge(WorkCategory::RecordVisit, 1)?;
        let shallow = record.child_count() == 0
            && record.shadow_root().is_none()
            && contents.is_none_or(|contents| {
                contents.child_count() == 0 && contents.shadow_root().is_none()
            });
        if shallow {
            if target == element || contents.is_some_and(|contents| contents.position() == target) {
                return Ok(Some(
                    ElementInsertionIgnored::HostIncludingInclusiveAncestor,
                ));
            }
        } else {
            let mut ancestor = Some(target);
            for _ in 0..storage.construction.storage(control)?.len() {
                let Some(index) = ancestor else { break };
                control.charge(WorkCategory::RecordVisit, 1)?;
                if index == element {
                    return Ok(Some(
                        ElementInsertionIgnored::HostIncludingInclusiveAncestor,
                    ));
                }
                let record = storage
                    .construction
                    .node_at(index)
                    .ok_or(SessionError::ForeignNode)?;
                ancestor = if let Some(owner) = &record.fragment_owner() {
                    let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
                        owner;
                    control.charge(WorkCategory::ProviderCall, 1)?;
                    let owner_position = storage
                        .construction
                        .storage(control)?
                        .node_by_result_id(owner)
                        .map_err(|_| SessionError::ForeignNode)?
                        .position();
                    Some(owner_position)
                } else {
                    record.parent().map(|node| node.position())
                };
            }
            if ancestor.is_some() {
                return Err(SessionError::Cycle);
            }
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        if matches!(
            storage
                .construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            for position in 0..storage
                .construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?
                .child_count()
            {
                // One actual child-index read and one referenced kind read.
                control.charge(WorkCategory::RecordVisit, 2)?;
                let child = storage
                    .construction
                    .node_at(target)
                    .ok_or_else(|| latch_registry_row(control))?
                    .child_at(position)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                let record = storage
                    .construction
                    .node_at(child)
                    .ok_or(SessionError::ForeignNode)?;
                if matches!(record.kind(), HtmlNodeKind::Element(_)) {
                    return Ok(Some(ElementInsertionIgnored::DocumentAlreadyHasElement));
                }
            }
        }
        Ok(None)
    }
    /// Only the actual token-created template path in this checkpoint proves
    /// its insertion target and end marker are initially null. Preexisting
    /// templates carry no such parser creation provenance.
    #[cfg(test)]
    pub(crate) fn unpatched_template_contents(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        let contents = storage.template_contents(control, node)?;
        if contents.is_none() {
            return Ok(None);
        }
        for payload in &storage.payloads.values {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(payload, RetainedPayload::String { location: StringLocation::ElementName(owner), .. } if *owner == index)
            {
                return Ok(contents);
            }
        }
        Err(SessionError::TemplateInsertionStateUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.active()?.template_contents(control, element)
    }
    pub(crate) fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        self.qualify_intrinsic_mutation(control, node, capability)?;
        self.active_mut()?
            .insert(control, parent, node, before, InsertionCause::NonParser)
    }
    /// Actual parser insertion; qualification precedes topology/effect mutation.
    #[cfg(test)]
    pub(crate) fn insert_parser(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        self.insert_parser_with_meta_csp(control, parent, node, before, capability, &mut None)
    }
    pub(crate) fn insert_parser_with_meta_csp(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        capability.validate_session(control, self)?;
        self.qualify_base_response(control, node, capability)?;
        self.qualify_intrinsic_mutation(control, node, Some(capability))?;
        self.active_mut()?.insert_with_meta_csp(
            control,
            parent,
            node,
            before,
            InsertionCause::Parser,
            service,
        )
    }
    pub(crate) fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        self.qualify_intrinsic_mutation(control, node, capability)?;
        self.active_mut()?.remove(control, node)
    }
    fn qualify_intrinsic_mutation(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        let root = storage.index(c, node)?;
        let mut next = Some(root);
        while let Some(index) = next {
            if storage.svg_script_local(c, index)? {
                storage.svg_state_at(c, index)?;
                let capability = capability.ok_or_else(|| {
                    SessionError::Stop(
                        c.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
                    )
                })?;
                capability.validate_session(c, self)?;
            }
            for name in ["select", "option", "selectedcontent", "script"] {
                if storage.html_local(c, index, name)? {
                    if name == "script"
                        && script_metadata(&storage.payloads, c, index)?
                            .1
                            .parser_document
                            .is_some()
                    {
                        continue;
                    }
                    let capability = capability.ok_or_else(|| {
                        SessionError::Stop(
                            c.stop_unsupported_standalone(StandaloneUnsupported::Provenance),
                        )
                    })?;
                    capability.validate_session(c, self)?;
                    return Ok(());
                }
            }
            next = storage.next_effect_node(c, root, index)?;
        }
        Ok(())
    }
    pub(crate) fn freeze(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<FrozenArena, SessionError> {
        let fragment_root = self.active_mut()?.prepare_freeze(control)?;
        self.active_mut()?.prepare_association_documents(control)?;
        // A stopped append keeps its buffer for cleanup and cannot publish.
        // The check precedes publication, so a refusal consumes nothing.
        if self.active()?.failed_text_append.is_some() {
            return Err(latch_registry_row(control));
        }
        // Only a parser-initialized session carries this actual variable owner.
        // Prepay its active-to-frozen move before storage is taken; a stopped
        // share restores that same owner without useful recovery work.
        if self.active()?.evidence.is_some() {
            payload_write::<Option<ParseEvidence>>(control, 2)?;
        }
        control.charge(WorkCategory::RecordVisit, 2)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        let root_position = fragment_root.unwrap_or(0);
        let root = self
            .active()?
            .construction
            .node_at(root_position)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        let (result, lease) = self.active_mut()?.construction.publish(control, root)?;
        let SessionStorage {
            csp_slot: _,
            control: owner,
            effects,
            evidence,
            fragment,
            #[cfg(test)]
                cancel_after_details_shift: _,
            #[cfg(test)]
                cancel_after_radio_peer: _,
            #[cfg(test)]
                cancel_after_removal: _,
            #[cfg(test)]
                cancel_after_textarea_reset: _,
            #[cfg(test)]
                cancel_after_output_reset: _,
            #[cfg(test)]
                cancel_after_input_reset: _,
            #[cfg(test)]
                lose_association_row: _,
            #[cfg(test)]
                cancel_after_range_append: _,
            #[cfg(test)]
                cancel_after_range_ingress: _,
            standalone_ineligibility: _,
            construction,
            payloads,
            failed_text_append: _,
            style_order: _,
            meta_batches: _,
            #[cfg(test)]
                cancel_after_text_append_chunk: _,
        } = self.storage.take().ok_or(SessionError::Consumed)?;
        let view = match result {
            bounded::HtmlConstructionResult::Document(value) => {
                HtmlConstructionResult::Document(value)
            }
            bounded::HtmlConstructionResult::Fragment(value) => {
                HtmlConstructionResult::Fragment(value)
            }
        };
        Ok(FrozenArena {
            finalizing: None,
            sharing_started: false,
            storage: Some(FrozenStorage {
                view,
                control: owner,
                construction,
                lease: Some(lease),
                payloads,
                effects,
                evidence,
                fragment,
            }),
        })
    }
    /// Strict private completed-assembly preflight. Generic native freezing
    /// remains separate; no backing is cloned to test actual ownership.
    pub(crate) fn preflight_completed_freeze(
        &self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        storage.payloads.check_aliases(
            ConstructionView::Active(storage.construction.storage(control)?),
            control,
        )?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if storage
            .evidence
            .as_ref()
            .is_none_or(|evidence| !evidence.retains_diagnostics())
        {
            return Err(SessionError::WrongToken);
        }
        Ok(())
    }
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(control, &self.active()?.control)?;
        self.active()?.payloads.check_aliases(
            ConstructionView::Active(self.active()?.construction.storage(control)?),
            control,
        )?;
        self.storage
            .take()
            .ok_or(SessionError::Consumed)?
            .release(control)
    }
}

impl FrozenArena {
    /// Fixed lender of the real committed store, not a report projection.
    pub(crate) fn retained_evidence(&self) -> Result<&ParseEvidence, SessionError> {
        self.storage
            .as_ref()
            .ok_or(SessionError::Consumed)?
            .evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)
    }
    /// Partition the stopped frozen store through its original evidence issuer.
    pub(crate) fn take_stopped_evidence(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<evidence::FailureDiagnosticEvidence>, SessionError> {
        if let Some(finalizing) = &mut self.finalizing {
            return finalizing.take_stopped_evidence(control);
        }
        let storage = self.storage.as_mut().ok_or(SessionError::Consumed)?;
        control.cleanup_release(&storage.control, HtmlParseResource::AuxiliaryEntries, 0)?;
        ParseEvidence::partition_failed(&mut storage.evidence, control)
    }
    #[cfg(test)]
    pub(crate) fn input_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<InputState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        Ok(find_input_state(&storage.payloads, control, position)?.1)
    }
    /// Same conditional availability as the active owner, qualified to this arena.
    #[cfg(test)]
    pub(crate) fn input_writeback_sources<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError>
    {
        self.input_state(control, node)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let arena = &self.storage.as_ref().ok_or(SessionError::Consumed)?.view;
        find_input_sources(
            &self
                .storage
                .as_ref()
                .ok_or(SessionError::Consumed)?
                .payloads,
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
        )
    }
    #[cfg(test)]
    pub(crate) fn append_input_internal_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let arena = &storage.view;
        let raw = find_input_state(
            &storage.payloads,
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
        )?
        .0;
        destination.extend_units(
            textarea_raw(&storage.payloads, control, raw)?.code_units(),
            control,
        )?;
        Ok(())
    }
    #[cfg(test)]
    /// Native input value readback, distinct from internal and default value.
    pub(crate) fn append_input_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let state = self.input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        append_native_input_value(
            ConstructionView::Frozen(arena),
            &storage.payloads,
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
            state,
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_input_default_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        append_input_content(
            ConstructionView::Frozen(arena),
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
            destination,
            &[],
        )
    }

    #[cfg(test)]
    pub(crate) fn output_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<OutputState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        find_output_state(&storage.payloads, control, position)
    }
    #[cfg(test)]
    pub(crate) fn append_output_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.output_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        append_descendant_text(
            ConstructionView::Frozen(arena),
            control,
            arena
                .node_by_result_id(node)
                .map_err(|_| SessionError::ForeignNode)?
                .position(),
            destination,
        )
    }
    #[cfg(test)]
    pub(crate) fn append_output_default_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        self.append_output_value(control, node, destination)
    }
    #[cfg(test)]
    pub(crate) fn textarea_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<TextareaState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        find_textarea_state(&storage.payloads, control, position)?
            .map(|(_, state)| state)
            .ok_or(SessionError::TextareaMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn textarea_raw_value<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<&'a DocumentString, SessionError> {
        let state = self.textarea_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        textarea_raw(&storage.payloads, control, state.raw)
    }
    #[cfg(test)]
    pub(crate) fn image_form_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<FormAssociationState, SessionError> {
        let state = self
            .form_association_state(control, node)
            .map_err(|error| match error {
                SessionError::FormAssociationMetadataUnavailable => {
                    SessionError::ImageFormMetadataUnavailable
                }
                other => other,
            })?;
        if state.kind != HtmlNativeFormAssociationKind::Image {
            return Err(SessionError::ImageFormMetadataUnavailable);
        }
        Ok(state)
    }
    #[cfg(test)]
    pub(crate) fn image_form_owner(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let state = self.image_form_state(control, node)?;
        self.resolve_form_owner(control, state.owner)
    }
    #[cfg(test)]
    pub(crate) fn form_association_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<FormAssociationState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = &storage.view;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        find_form_association_state(&storage.payloads, control, position)?
            .map(|(_, state)| state)
            .ok_or(SessionError::FormAssociationMetadataUnavailable)
    }
    #[cfg(test)]
    pub(crate) fn form_association_owner(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let state = self.form_association_state(control, node)?;
        self.resolve_form_owner(control, state.owner)
    }
    #[cfg(test)]
    fn resolve_form_owner(
        &self,
        control: &mut ParseControl,
        owner: Option<usize>,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let Some(owner) = owner else {
            return Ok(None);
        };
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = &storage.view;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if owner >= arena.len() {
            return Err(SessionError::ForeignNode);
        }
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<HtmlNodeId>() as u64)?;
        Ok(Some(
            arena
                .node_at(owner)
                .expect("qualified form owner")
                .as_node()
                .id(),
        ))
    }
    #[cfg(test)]
    pub(crate) fn script_metadata(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ScriptMetadata, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let position = storage
            .view
            .position(&node.handle)
            .map_err(|_| SessionError::ForeignNode)?;
        script_metadata(&storage.payloads, control, position).map(|(_, metadata)| metadata)
    }
    pub(crate) fn document(&self) -> Option<&HtmlDocument> {
        match &self.storage.as_ref()?.view {
            HtmlConstructionResult::Document(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn fragment(&self) -> Option<&HtmlFragment> {
        match &self.storage.as_ref()?.view {
            HtmlConstructionResult::Fragment(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        if let Some(finalizing) = &self.finalizing {
            finalizing.require_cleanup(control)?;
        }
        if let Some(finalizing) = self.finalizing.take() {
            return finalizing.release(control);
        }
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        cleanup_owner(control, &storage.control)?;
        storage
            .payloads
            .check_aliases(ConstructionView::Frozen(&storage.view), control)?;
        // A missing lease is refused before anything is taken, so the owner
        // stays where it was.
        let Some(publication) = storage.lease.as_ref() else {
            return Err(latch_registry_row(control));
        };
        storage
            .construction
            .require_recovery(control, publication)?;
        let Some(FrozenStorage {
            view,
            control: owner,
            mut construction,
            mut lease,
            payloads,
            effects,
            evidence,
            fragment,
        }) = self.storage.take()
        else {
            return Err(SessionError::Consumed);
        };
        let mut result = Some(match view {
            HtmlConstructionResult::Document(value) => {
                bounded::HtmlConstructionResult::Document(value)
            }
            HtmlConstructionResult::Fragment(value) => {
                bounded::HtmlConstructionResult::Fragment(value)
            }
        });
        let recovery = match bounded::HtmlConstructionLease::try_recover(&mut result, &mut lease) {
            Ok(recovery) => recovery,
            Err(error) => {
                // `try_recover` refuses before it takes the result, so the slot
                // filled above is still occupied.
                let view = match result.take() {
                    Some(bounded::HtmlConstructionResult::Document(value)) => {
                        HtmlConstructionResult::Document(value)
                    }
                    Some(bounded::HtmlConstructionResult::Fragment(value)) => {
                        HtmlConstructionResult::Fragment(value)
                    }
                    None => return Err(latch_registry_row(control)),
                };
                self.storage = Some(FrozenStorage {
                    view,
                    control: owner,
                    construction,
                    lease,
                    payloads,
                    effects,
                    evidence,
                    fragment,
                });
                return Err(construction_owner::construction_error(control, error));
            }
        };
        // Every owner is released even after a refusal; the first one is
        // returned at the end. The recovery put the storage back before it
        // could refuse, so the owner below can still be retired.
        let mut refused = None;
        let recovered = construction.recovered(control, recovery);
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, recovered);
        if let Some(fragment) = fragment {
            let released = fragment.release(control, &owner);
            #[cfg(test)]
            inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
            keep_first(&mut refused, released);
        }
        if let Some(evidence) = evidence {
            let released = evidence.release(control);
            #[cfg(test)]
            inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
            keep_first(&mut refused, released);
        }
        let released = effects.release(control);
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, released);
        let released = release_parts(
            control,
            &owner,
            ReleasedStorage {
                construction,
                payloads,
            },
        );
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, released);
        refused.map_or(Ok(()), Err)
    }
}

fn lowercase_token_name(
    control: &mut ParseControl,
    name: &DocumentString,
) -> Result<(), SessionError> {
    for &unit in name.code_units() {
        control.charge(WorkCategory::InspectByte, 2)?;
        if matches!(unit, 0x41..=0x5a) {
            return Err(SessionError::WrongToken);
        }
    }
    Ok(())
}

fn exact_ascii(
    control: &mut ParseControl,
    value: Option<&DocumentString>,
    expected: &str,
) -> Result<bool, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let Some(value) = value else { return Ok(false) };
    if value.len() != expected.len() {
        return Ok(false);
    }
    for (&unit, byte) in value.code_units().iter().zip(expected.bytes()) {
        control.charge(WorkCategory::InspectByte, 3)?;
        if unit != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests;
