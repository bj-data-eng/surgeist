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
mod node_joins;
mod payload_registry;
mod session_error;
mod string_storage;
use node_joins::{Join, NodeJoins, cell_slot, joined_cell};
pub use payload_registry::{FormAssociationEvidence, OutputDefaultValueOverride};
pub(crate) use payload_registry::{
    FormAssociationState, InputState, OutputState, ScriptMetadata, TextareaState,
};
use payload_registry::{
    KindList, KindSlots, PayloadStorage, PreparedSlot, RetainedPayload, element_attribute,
    element_record, row_list,
};
pub(crate) use session_error::{
    ElementInsertionIgnored, SessionClass, SessionError, session_class,
};
use session_error::{
    count_set, end_session_on, latch_creation_fault, latch_effect_fault, latch_owner_refusal,
    latch_registry_row, next_cursor, plan_below_charge, session_fault_invariant, successor,
};
use string_storage::{AttributeStorage, ReleasedStorage, mirrored_position};
pub(crate) use string_storage::{StringLocation, TextInsertion};
mod form_effects;
mod storage;
mod storage_allocation;
mod storage_attributes;
mod storage_input;
mod storage_insertion;
mod storage_release;
mod storage_script;
mod storage_strings;
mod storage_textarea;
mod storage_tree;
use form_effects::{FormEffectPass, FormEffectStage, FormEffects};
use storage::SessionStorage;
use storage_attributes::{AttributeMergeName, PreparedAttributeMerge};
use storage_input::InputAttributeInitialization;
use storage_insertion::InsertionCause;
#[derive(Clone, Debug)]
pub(crate) struct ParserNode {
    handle: HtmlBuilderNode,
    // Immutable dense parser-metadata join; only the checked handle authorizes a node.
    index: usize,
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

pub(crate) struct ParserArenaSession {
    storage: Option<SessionStorage>,
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
