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
//! Hidden initialization installs source-order attributes, retaining original
//! token/value-attribute ranges and lexical maps for actual conditional writeback.
//! Script facts are retained for a sealed fresh ordinary Disabled execution;
//! preparation ends at step18 and never fabricates a loaded or executed result.

use super::*;
use crate::parser::control::{
    ControlIdentity, DocumentStringAllocation, OwnedVec, ParseControl, ParseStop, SharedAllocation,
    StandaloneUnsupported, VectorCapacity, WorkCategory,
};
use crate::parser::{
    buffer::{FrozenUtf16, Utf16Buffer},
    tokenizer::token::{Doctype, Tag, TagKind, Text, TextOrigin, TokenKind},
};
use crate::values::ParserAttributeError;
use crate::{Attribute, HtmlParseResource};
use std::{mem, sync::Weak};

mod clone;
mod completion;
pub(crate) mod context;
pub(crate) use context::ParserDocumentReference;
use context::{DocumentRef, FragmentContextStorage};
pub(crate) mod evidence;
use evidence::ParseEvidence;
mod foreign;
pub(crate) mod native_view;
mod pi;
pub(crate) mod provenance;
use provenance::{FieldCopy, OriginGroupRef};
mod select;
mod svg;
mod template;
pub(crate) use completion::{
    ConstructionCompletion, ConstructionReadiness, StandaloneDocumentState,
};
pub(crate) use foreign::ForeignNamespace;
pub(crate) use native_view::NativeView;
use select::{ConstructionEffects, SelectMetadata};
pub(crate) use svg::{SvgScriptOrigin, SvgScriptState};
pub(crate) use template::{CreationContext, CreationLocation, ParserDocument, TemplateState};

#[derive(Clone, Debug)]
pub(crate) struct ParserNode {
    owner: Weak<Identity>,
    index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionError {
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
    FormAssociationMetadataUnavailable,
    TextareaMetadataUnavailable,
    OutputMetadataUnavailable,
    HiddenInputMetadataUnavailable,
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
impl From<ParseStop> for SessionError {
    fn from(value: ParseStop) -> Self {
        Self::Stop(value)
    }
}

pub(crate) enum RootKind {
    Document(HtmlDocumentMode),
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
    Hidden(
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
    Hidden {
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
    units: usize,
}

pub(crate) struct ParserArenaSession {
    storage: Option<SessionStorage>,
}

/// Original lexical creation record, not an alias of installed arena backing.
/// Only this arena can issue it; every replay copies into independent tickets.
pub(crate) struct FormattingCreation {
    token: Option<Tag>,
    source: crate::HtmlSourceRange,
    control: ControlIdentity,
    arena: Weak<Identity>,
    cause: Option<provenance::ConstructionCauseRef>,
}

impl FormattingCreation {
    /// Borrowed disposal retains originals on any preflight rejection. Once a
    /// destructive disposal begins, newly observed stops complete reclamation.
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_owner(control, &self.control)?;
        release_formatting_tag(&mut self.token, control)
    }
}

struct SessionStorage {
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
    cancel_after_hidden_reset: Option<crate::HtmlParseCancellation>,
    standalone_ineligibility: Option<StandaloneUnsupported>,
    identity: Arc<Identity>,
    identity_allocation: SharedAllocation<Identity>,
    records: Vec<Record>,
    records_allocation: VectorCapacity<Record>,
    children_allocations: Vec<VectorCapacity<usize>>,
    tickets_allocation: VectorCapacity<VectorCapacity<usize>>,
    usage: HtmlConstructionUsage,
    payloads: PayloadStorage,
}

enum FrozenView {
    Document(HtmlDocument),
    Fragment(HtmlFragment),
}
pub(crate) struct FrozenArena {
    storage: Option<FrozenStorage>,
}

struct FrozenStorage {
    view: FrozenView,
    control: ControlIdentity,
    arena_allocation: SharedAllocation<Arena>,
    identity_allocation: SharedAllocation<Identity>,
    records_allocation: VectorCapacity<Record>,
    children_allocations: Vec<VectorCapacity<usize>>,
    tickets_allocation: VectorCapacity<VectorCapacity<usize>>,
    payloads: PayloadStorage,
    effects: ConstructionEffects,
    evidence: Option<ParseEvidence>,
    fragment: Option<FragmentContextStorage>,
}
/// Actual backing and exact record-qualified ownership, not allocation proxies.
struct PayloadStorage {
    values: Vec<RetainedPayload>,
    allocation: VectorCapacity<RetainedPayload>,
    preexisting_units: usize,
    preexisting_attributes: usize,
}
enum RetainedPayload {
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
    SvgScript {
        node: Option<usize>,
        state: SvgScriptState,
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
    HiddenInput {
        node: Option<usize>,
        raw: usize,
        hidden: bool,
        complete: bool,
        state: HiddenInputState,
    },
    HiddenInputSource {
        node: Option<usize>,
        token: crate::HtmlSourceRange,
        value_attribute: Option<crate::HtmlSourceRange>,
    },
    SelectFamily {
        node: Option<usize>,
        state: SelectMetadata,
    },
    WorkList(Option<OwnedVec<usize>>),
    // A replaced raw backing was disposed; its initialized registry slot
    // remains admitted until the registry itself is released.
    Retired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextareaState {
    raw: usize,
    pub(crate) dirty_value: bool,
    pub(crate) user_validity: bool,
}
/// Pinned output override issued only by actual fresh creation reset.
/// Nonnull setters/general form reset have no producer in this checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum OutputDefaultValueOverride {
    Null,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OutputState {
    pub(crate) default_value_override: OutputDefaultValueOverride,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HiddenInputState {
    pub(crate) dirty_value: bool,
    pub(crate) dirty_checkedness: bool,
    pub(crate) checkedness: bool,
    pub(crate) indeterminate: bool,
    pub(crate) user_validity: bool,
    pub(crate) selected_files_empty: bool,
    pub(crate) value_rewritten_during_creation: bool,
}
/// Actual parser-created association, distinct from structural owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormAssociationKind {
    Image,
    Button,
    Fieldset,
    Object,
    Textarea,
    Output,
    HiddenInput,
    Select,
}
impl FormAssociationKind {
    fn listed(self) -> bool {
        self != Self::Image
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FormAssociationState {
    pub(crate) kind: FormAssociationKind,
    owner: Option<usize>,
    pub(crate) parser_inserted: bool,
    pub(crate) evidence: FormAssociationEvidence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormAssociationEvidence {
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
        next_slot: usize,
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
/// Document indices are private and resolved only through qualified handles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMetadata {
    parser_document: Option<DocumentRef>,
    preparation_document: Option<DocumentRef>,
    pub(crate) force_async: bool,
    pub(crate) already_started: bool,
    pub(crate) script_type: Option<ScriptType>,
    pub(crate) external_file: bool,
    pub(crate) ready_to_be_parser_executed: bool,
    pub(crate) delaying_load: bool,
    pub(crate) result: ScriptResult,
    pub(crate) steps_when_result_ready: Option<ScriptReadySteps>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptType {
    Classic,
    Module,
    ImportMap,
    SpeculationRules,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptResult {
    Uninitialized,
}
/// No preparation beyond the Disabled return installs result-ready steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptReadySteps {}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisabledScriptPreparation {
    AlreadyStarted,
    Empty,
    Disconnected,
    UnrecognizedType,
    Disabled(ScriptType),
    ParserDocumentMismatch,
}
enum AttributeStorage {
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
}
#[derive(Clone, Copy)]
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
    pub(crate) node: ParserNode,
    pub(crate) offset: usize,
}
struct ReleasedStorage {
    arena: Arena,
    records: VectorCapacity<Record>,
    children: Vec<VectorCapacity<usize>>,
    tickets: VectorCapacity<VectorCapacity<usize>>,
    identity: SharedAllocation<Identity>,
    payloads: PayloadStorage,
}

impl PayloadStorage {
    /// Cleanup is a bounded ownership walk over admitted entries, permitted
    /// after a terminal stop. No parser work is resumed and no work refunded.
    /// Each live location is checked against its actual immutable record.
    fn check_aliases(
        &self,
        records: &[Record],
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        for index in 0..self.values.len() {
            cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
            let payload = &self.values[index];
            match payload {
                RetainedPayload::String {
                    value, location, ..
                } => {
                    let stored = location.value(records, &self.values, control)?;
                    cleanup_charge(control, WorkCategory::ProviderCall, 2)?;
                    let expected = if let Some(stored) = stored {
                        if !Arc::ptr_eq(stored.backing(), value.backing()) {
                            return Err(SessionError::AliasedPayload);
                        }
                        2
                    } else {
                        1
                    };
                    if Arc::strong_count(value.backing()) != expected {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(AttributeStorage::Frozen {
                    values,
                    element,
                    ..
                })) => {
                    let expected = if let Some(index) = element {
                        cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                        cleanup_charge(control, WorkCategory::ProviderCall, 1)?;
                        let Some(Record {
                            kind: HtmlNodeKind::Element(record),
                            ..
                        }) = records.get(*index)
                        else {
                            return Err(SessionError::AliasedPayload);
                        };
                        if !Arc::ptr_eq(values.backing(), record.attributes().backing()) {
                            return Err(SessionError::AliasedPayload);
                        }
                        2
                    } else {
                        1
                    };
                    cleanup_charge(control, WorkCategory::ProviderCall, 1)?;
                    if Arc::strong_count(values.backing()) != expected {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(
                    AttributeStorage::Building { .. } | AttributeStorage::Transferring { .. },
                )) => {}
                RetainedPayload::Attributes(None) => return Err(SessionError::Consumed),
                RetainedPayload::SvgScript { .. }
                | RetainedPayload::ConstructionCause { .. }
                | RetainedPayload::Script { .. }
                | RetainedPayload::NodeDocument { .. }
                | RetainedPayload::LookupRegistry { .. }
                | RetainedPayload::Shadow { .. }
                | RetainedPayload::StandaloneDocumentCompletion { .. }
                | RetainedPayload::Document { .. }
                | RetainedPayload::Template { .. }
                | RetainedPayload::FormAssociation { .. }
                | RetainedPayload::Textarea { .. }
                | RetainedPayload::Output { .. }
                | RetainedPayload::HiddenInputSource { .. }
                | RetainedPayload::HiddenInput { .. }
                | RetainedPayload::SelectFamily { .. }
                | RetainedPayload::Retired => {}
                RetainedPayload::HistoricalString { origins } => {
                    origins.require_disposal_owner(control)?
                }
                RetainedPayload::PiAttributes { values, .. } => {
                    values.require_disposal_owner(control)?
                }
                RetainedPayload::WorkList(Some(list)) => list.require_disposal_owner(control)?,
                RetainedPayload::WorkList(None) => return Err(SessionError::Consumed),
            }
        }
        Ok(())
    }

    fn release(self, control: &mut ParseControl, owner: &ControlIdentity) {
        let Self {
            mut values,
            allocation,
            ..
        } = self;
        let entries = values.len() as u64;
        // Attribute containers own references to registered string backings.
        // Dispose containers first; reverse-pop then retains the real Vec until
        // its capacity ticket is consumed.
        for payload in &mut values {
            if let RetainedPayload::PiAttributes { values, .. } = payload {
                // PI maps own the second references to registered backings.
                // Remove those references before string-ticket disposal.
                while let Some(pair) = if control.stop().is_some() {
                    values.cleanup_pop(control)
                } else {
                    values.try_pop(control)
                }
                .unwrap_or_else(|_| {
                    values
                        .cleanup_pop(control)
                        .expect("terminal PI pair disposal")
                }) {
                    drop(pair);
                }
            }
            if let RetainedPayload::Attributes(storage) = payload {
                let storage = storage.take().expect("retained attribute phase");
                let count = match &storage {
                    AttributeStorage::Building { values, .. }
                    | AttributeStorage::Transferring { values, .. } => values.len(),
                    AttributeStorage::Frozen { values, .. } => values.len(),
                };
                match storage {
                    AttributeStorage::Building {
                        values, allocation, ..
                    }
                    | AttributeStorage::Transferring {
                        values, allocation, ..
                    } => {
                        release_vector(control, values, allocation)
                            .expect("actual attribute vector ticket");
                    }
                    AttributeStorage::Frozen {
                        values,
                        allocation,
                        shared,
                        ..
                    } => {
                        let backing = values.into_parser_storage();
                        let vector = control.release_shared(backing, shared).unwrap_or_else(|_| {
                            panic!("prechecked attribute backing has no external aliases")
                        });
                        release_vector(control, vector, allocation)
                            .expect("actual attribute vector ticket");
                    }
                }
                release_logical(
                    control,
                    owner,
                    HtmlParseResource::ManagedAttributes,
                    count as u64,
                );
            }
        }
        while let Some(payload) = values.pop() {
            if let RetainedPayload::PiAttributes { values, .. } = payload {
                values
                    .release(control)
                    .expect("actual PI map vector ownership");
                continue;
            }
            if let RetainedPayload::WorkList(Some(list)) = payload {
                list.release(control)
                    .expect("actual retained work-list ticket");
                continue;
            }
            if let RetainedPayload::HiddenInputSource {
                value_attribute, ..
            } = &payload
            {
                release_logical(
                    control,
                    owner,
                    HtmlParseResource::SourceMapEntries,
                    1 + u64::from(value_attribute.is_some()),
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
                control
                    .release_document_string(allocation)
                    .expect("actual managed string ticket");
                drop(origins);
            }
        }
        release_vector(control, values, allocation).expect("actual payload registry capacity");
        release_logical(control, owner, HtmlParseResource::AuxiliaryEntries, entries);
    }
}
impl StringLocation {
    fn value<'a>(
        &self,
        records: &'a [Record],
        payloads: &'a [RetainedPayload],
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
                match records.get(index).map(|record| &record.kind) {
                    Some(HtmlNodeKind::Text(value)) => Some(value),
                    _ => None,
                }
            }
            Self::Comment(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.get(index).map(|record| &record.kind) {
                    Some(HtmlNodeKind::Comment(value)) => Some(value),
                    _ => None,
                }
            }
            Self::ProcessingInstructionTarget(index) | Self::ProcessingInstructionData(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.get(index).map(|record| &record.kind) {
                    Some(HtmlNodeKind::ProcessingInstruction(value)) => Some(match self {
                        Self::ProcessingInstructionTarget(_) => value.target(),
                        _ => value.data(),
                    }),
                    _ => None,
                }
            }
            Self::DoctypeName(index) | Self::DoctypePublic(index) | Self::DoctypeSystem(index) => {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                match records.get(index).map(|record| &record.kind) {
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
                match records.get(index).map(|record| &record.kind) {
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
                            let Some(position) = original_len.checked_sub(index + 1) else {
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
                        let Some(position) = original_len.checked_sub(index + 1) else {
                            return Err(SessionError::AliasedPayload);
                        };
                        let position = if position < *reversed_pairs
                            || position >= original_len - reversed_pairs
                        {
                            original_len - 1 - position
                        } else {
                            position
                        };
                        values.get(position)
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
/// Running cleanup admits its actual bounded metadata/provider inspections.
/// After a latched stop, only ownership disposal over already-admitted entries
/// is allowed; this does not resume execution or refund any work/high-water.
fn cleanup_charge(
    control: &mut ParseControl,
    category: WorkCategory,
    units: u64,
) -> Result<(), SessionError> {
    if control.stop().is_none() {
        control.charge(category, units)?;
    }
    Ok(())
}

fn release_logical(
    control: &mut ParseControl,
    owner: &ControlIdentity,
    resource: HtmlParseResource,
    units: u64,
) {
    if control.stop().is_none() && control.release(resource, units).is_ok() {
        return;
    }
    control
        .cleanup_release(owner, resource, units)
        .expect("exact admitted logical ownership");
}

impl SessionStorage {
    fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        let effects = ConstructionEffects::new(control)?;
        let (records, records_allocation) = control.new_vector()?;
        let (children_allocations, tickets_allocation) = control.new_vector()?;
        let (values, allocation) = control.new_vector()?;
        let (identity, identity_allocation) = control
            .share_owned(Identity)
            .map_err(|(stop, _)| SessionError::Stop(stop))?;
        let mut session = Self {
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
            cancel_after_hidden_reset: None,
            standalone_ineligibility: None,
            identity,
            identity_allocation,
            records,
            records_allocation,
            children_allocations,
            tickets_allocation,
            usage: HtmlConstructionUsage::default(),
            payloads: PayloadStorage {
                values,
                allocation,
                preexisting_units: 0,
                preexisting_attributes: 0,
            },
        };
        let kind = match root {
            RootKind::Document(mode) => HtmlNodeKind::Document(mode),
            RootKind::Fragment => HtmlNodeKind::Fragment,
        };
        let allocated = payload_write::<Option<FormEffects>>(control, 1)
            .and_then(|()| session.allocate_native(control, kind, false, 0, 0));
        if let Err(error) = allocated {
            session.release(control);
            return Err(error);
        }
        if matches!(session.records[0].kind, HtmlNodeKind::Document(_)) {
            let facts = (|| {
                session.prepare_payload_slot_owned(control)?;
                session.payloads.values.push(RetainedPayload::Document {
                    node: Some(0),
                    state: template::DocumentState::Ordinary { inert: None },
                    class: crate::HtmlDocumentClass::Html,
                });
                session.prepare_payload_slot_owned(control)?;
                session.payloads.values.push(RetainedPayload::NodeDocument {
                    node: Some(0),
                    document: DocumentRef::Owned(0),
                });
                Ok::<_, SessionError>(())
            })();
            if let Err(error) = facts {
                session.release(control);
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
        Ok(ParserNode {
            owner: Arc::downgrade(&self.identity),
            index,
        })
    }
    fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.node(control, 0)
    }
    fn index(&self, control: &mut ParseControl, node: &ParserNode) -> Result<usize, SessionError> {
        self.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node.owner.as_ptr() != Arc::as_ptr(&self.identity) || node.index >= self.records.len() {
            return Err(SessionError::ForeignNode);
        }
        Ok(node.index)
    }

    /// Consumes preexisting immutable payloads, not tokenizer-created backing.
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
        let document = matches!(self.records[0].kind, HtmlNodeKind::Document(_))
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
        control.require_identity(&self.control)?;
        let count = 1 + usize::from(template);
        let cause_slots = self.prepare_node_causes(control, count)?;
        let requests = [
            (HtmlParseResource::ManagedNodes, count as u64),
            (HtmlParseResource::ManagedEdges, u64::from(template)),
            (HtmlParseResource::ManagedAttributes, attributes as u64),
            (HtmlParseResource::ManagedUtf16Units, units as u64),
            (HtmlParseResource::AuxiliaryEntries, count as u64),
        ];
        control.check_acquire_many(&requests)?;
        control.reserve_vector(&mut self.records, &mut self.records_allocation, count)?;
        control.reserve_vector(
            &mut self.children_allocations,
            &mut self.tickets_allocation,
            count,
        )?;
        let (children, children_allocation) = control.new_vector()?;
        let associated = if template {
            Some(control.new_vector::<usize>()?)
        } else {
            None
        };
        let index = self.records.len();
        let handle = self.node_owned(control, index)?;
        let owner = if template {
            control.charge(WorkCategory::ProviderCall, 1)?;
            Some(HtmlFragmentOwner::Template(HtmlNodeId {
                identity: self.identity.clone(),
                index,
            }))
        } else {
            None
        };
        control.observe_tree_depth(self.usage.depth.max(usize::from(template)) as u64)?;
        control.admit_atomic_layout(mem::size_of::<Record>() * count)?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<Record>() as u64 * count as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, (count * 2) as u64)?;
        // All allocations/checks precede this final logical commit. The bounded
        // append writes at most two records and two tickets, without further polls.
        control.acquire_many(&requests)?;
        self.records.push(Record {
            kind,
            parent: None,
            children,
            depth: 0,
            template_contents: template.then_some(index + 1),
            shadow: None,
            owner: None,
        });
        self.children_allocations.push(children_allocation);
        if let Some((children, allocation)) = associated {
            self.records.push(Record {
                kind: HtmlNodeKind::Fragment,
                parent: None,
                children,
                depth: 1,
                template_contents: None,
                shadow: None,
                owner,
            });
            self.children_allocations.push(allocation);
        }
        for (offset, slot) in cause_slots.into_iter().enumerate().take(count) {
            if let Some(slot) = slot {
                let RetainedPayload::ConstructionCause { node, .. } =
                    &mut self.payloads.values[slot]
                else {
                    unreachable!()
                };
                *node = Some(index + offset);
            }
        }
        self.usage.nodes += count;
        self.usage.edges += usize::from(template);
        self.usage.attributes += attributes;
        self.usage.utf16_units += units;
        self.usage.depth = self.usage.depth.max(usize::from(template));
        Ok(handle)
    }

    fn prepare_payload_slot(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        self.require(control)?;
        self.prepare_payload_slot_owned(control)
    }
    // Only the synchronous effects owner may admit a raw replacement while
    // public/session entry points are blocked by the retained effects phase.
    fn prepare_payload_slot_owned(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        control.reserve_vector_geometric(
            &mut self.payloads.values,
            &mut self.payloads.allocation,
            1,
        )?;
        control.admit_atomic_layout(mem::size_of::<RetainedPayload>())?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<RetainedPayload>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        control.acquire(HtmlParseResource::AuxiliaryEntries, 1)?;
        Ok(())
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
        if let Err(error) = self.prepare_payload_slot_owned(control) {
            raw.release(control)?;
            return Err(error);
        }
        let (value, allocation) = raw.into_parts();
        let index = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::String {
            value,
            allocation,
            origins,
            location: StringLocation::Detached,
        });
        let RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        } = &mut self.payloads.values[index]
        else {
            unreachable!()
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
                OrderedCreation::Hidden {
                    input,
                    association,
                    source,
                } => {
                    payload_write::<[Option<usize>; 3]>(control, 3)?;
                    let RetainedPayload::HiddenInputSource { node: bound, .. } =
                        &mut self.payloads.values[source]
                    else {
                        unreachable!()
                    };
                    *bound = Some(node.index);
                    let RetainedPayload::HiddenInput { node: bound, .. } =
                        &mut self.payloads.values[input]
                    else {
                        unreachable!()
                    };
                    *bound = Some(node.index);
                    let RetainedPayload::FormAssociation { node: bound, .. } =
                        &mut self.payloads.values[association]
                    else {
                        unreachable!()
                    };
                    *bound = Some(node.index);
                }
                OrderedCreation::Select {
                    metadata: Some(slot),
                } => {
                    payload_write::<Option<usize>>(control, 1)?;
                    let RetainedPayload::SelectFamily { node: bound, .. } =
                        &mut self.payloads.values[slot]
                    else {
                        unreachable!()
                    };
                    *bound = Some(node.index);
                }
                OrderedCreation::Select { metadata: None } => {}
            }
            let mut container = None;
            for index in 0..self.payloads.values.len() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if matches!(&self.payloads.values[index], RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element: Some(owner), .. })) if *owner == node.index)
                {
                    container = Some(index);
                    break;
                }
            }
            let container = container.ok_or(SessionError::AttributeStorageUnavailable)?;
            let (values, allocation) = control.new_vector()?;
            if let Err(error) = self.prepare_payload_slot(control) {
                release_vector(control, values, allocation)?;
                return Err(error);
            }
            control.charge(WorkCategory::RecordVisit, 1)?;
            let count = attributes.len();
            let stage = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Attributes(Some(
                AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: count,
                    reversed_pairs: 0,
                },
            )));
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
                let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                })) = &mut self.payloads.values[stage]
                else {
                    unreachable!()
                };
                control.reserve_vector(values, allocation, 1)?;
                payload_write::<Attribute>(control, 1)?;
                payload_write::<[StringLocation; 2]>(control, 2)?;
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                values.push(Attribute::new(
                    QualifiedName::new(Namespace::none(), None, local),
                    value,
                ));
                remaining -= 1;
                self.bind_string(
                    local_ticket,
                    StringLocation::AttributeName(stage, remaining),
                );
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(stage, remaining),
                );
            }
            self.edit_attributes(control, node.index, container, stage, count, false)?;
            self.begin_attribute_transfer(control, stage, container, 0)?;
            for position in 0..count {
                self.edit_attributes(control, node.index, container, stage, 1, true)?;
                if self.effects.form.is_some() {
                    self.drain_form_effects(control)?;
                }
                match creation {
                    OrderedCreation::Hidden {
                        input, association, ..
                    } => self.initial_input_attribute(
                        control,
                        node.index,
                        input,
                        association,
                        container,
                        position,
                    )?,
                    OrderedCreation::Select { .. } => {
                        self.initial_select_attribute(control, node.index, position)?
                    }
                }
            }
            // Every original lexical backing remains retained, including values
            // detached by a semantic writeback. Native usage counts actual fields.
            let mut units = 0usize;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let HtmlNodeKind::Element(element) = &self.records[node.index].kind else {
                unreachable!()
            };
            for attribute in element.attributes().as_slice() {
                control.charge(WorkCategory::RecordVisit, 2)?;
                units = units
                    .checked_add(attribute.name().local_name().len())
                    .and_then(|v| v.checked_add(attribute.value().len()))
                    .ok_or_else(|| {
                        control
                            .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                            .unwrap_err()
                    })?;
            }
            // Zero remaining appends: close the same real redirect and charge
            // logical native usage only after all source effects have completed.
            payload_write::<PreparedAttributeMerge>(control, 1)?;
            self.finish_attribute_merge(
                control,
                PreparedAttributeMerge {
                    target: node.index,
                    container,
                    stage,
                    start: 0,
                    count: 0,
                    units,
                },
            )?;
            self.usage.attributes += count;
            match creation {
                OrderedCreation::Hidden { input, .. } => {
                    self.reset_hidden_input(control, node.index, input)?
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
        let RetainedPayload::HiddenInput { raw, .. } = &self.payloads.values[slot] else {
            return Err(SessionError::HiddenInputMetadataUnavailable);
        };
        control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
        Ok(*raw)
    }
    fn read_input_state(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<HiddenInputState, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<HiddenInputState>() as u64,
        )?;
        let RetainedPayload::HiddenInput { state, .. } = &self.payloads.values[slot] else {
            return Err(SessionError::HiddenInputMetadataUnavailable);
        };
        Ok(*state)
    }
    fn write_input_state(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        next: HiddenInputState,
    ) -> Result<(), SessionError> {
        payload_write::<HiddenInputState>(control, 1)?;
        let RetainedPayload::HiddenInput { state, .. } = &mut self.payloads.values[slot] else {
            return Err(SessionError::HiddenInputMetadataUnavailable);
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
        let RetainedPayload::HiddenInput { raw, .. } = &mut self.payloads.values[slot] else {
            unreachable!()
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
        let HtmlNodeKind::Element(element) = &self.records[node].kind else {
            return Err(SessionError::WrongToken);
        };
        let value = element.attributes().as_slice()[position].value();
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::String {
                value: candidate, ..
            } = payload
            {
                control.charge(WorkCategory::ProviderCall, 1)?;
                if Arc::ptr_eq(value.backing(), candidate.backing()) {
                    return Ok(slot);
                }
            }
        }
        Err(SessionError::AttributeStorageUnavailable)
    }
    fn initial_input_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        association: usize,
        container: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 3)?;
        let HtmlNodeKind::Element(element) = &self.records[node].kind else {
            unreachable!()
        };
        let name = element.attributes().as_slice()[position]
            .name()
            .local_name();
        let value_attr = exact_ascii(control, Some(name), "value")?;
        let type_attr = exact_ascii(control, Some(name), "type")?;
        let checked_attr = exact_ascii(control, Some(name), "checked")?;
        let form_attr = exact_ascii(control, Some(name), "form")?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<bool>() as u64)?;
        let RetainedPayload::HiddenInput { hidden, .. } = &self.payloads.values[slot] else {
            unreachable!()
        };
        let hidden = *hidden;
        let state = self.read_input_state(control, slot)?;
        let dirty = state.dirty_value;
        if value_attr && !dirty {
            let source = self.attribute_value_slot(control, node, position)?;
            let text = copy_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                source,
                !hidden,
            )?;
            self.replace_input_raw(control, slot, text)?;
        }
        if checked_attr && !state.dirty_checkedness {
            let mut next = state;
            next.checkedness = true;
            self.write_input_state(control, slot, next)?;
        }
        if type_attr {
            // Final exact Hidden admission was checked before take. The real
            // type is changed before the nested value-attribute change.
            payload_write::<bool>(control, 1)?;
            let RetainedPayload::HiddenInput { hidden, .. } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *hidden = true;
            let raw = self.input_raw_slot(control, slot)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let nonempty = !textarea_raw(&self.payloads, control, raw)?.is_empty();
            if nonempty {
                let mut value_position = None;
                control.charge(WorkCategory::RecordVisit, 1)?;
                let HtmlNodeKind::Element(element) = &self.records[node].kind else {
                    unreachable!()
                };
                for (index, attribute) in element.attributes().as_slice().iter().enumerate() {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    if exact_ascii(control, Some(attribute.name().local_name()), "value")? {
                        value_position = Some(index);
                        break;
                    }
                }
                let position = value_position.ok_or(SessionError::WrongToken)?;
                self.writeback_input_value(control, node, slot, container, position, raw)?;
                // Nested dirty=false value change under current Hidden: copy
                // the actual replaced native field, with no Text sanitization.
                let source = self.attribute_value_slot(control, node, position)?;
                let text = copy_input_value(
                    &self.payloads,
                    self.evidence.as_ref(),
                    control,
                    source,
                    false,
                )?;
                self.replace_input_raw(control, slot, text)?;
            }
        }
        if form_attr {
            let state = find_form_association_state(&self.payloads, control, node)?
                .ok_or(SessionError::FormAssociationMetadataUnavailable)?
                .1;
            self.reset_form_association(control, association, node, state, None)?;
        }
        Ok(())
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
        let text = copy_input_value(&self.payloads, self.evidence.as_ref(), control, raw, false)?;
        let (value, new_slot) = self.register_field_copy(control, text)?;
        (|| {
            control.charge(WorkCategory::RecordVisit, 2)?;
            control.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<QualifiedName>(control, 1)?;
            let HtmlNodeKind::Element(element) = &self.records[node].kind else {
                unreachable!()
            };
            payload_write::<Attribute>(control, 1)?;
            payload_write::<Option<Attribute>>(control, 1)?;
            let mut incoming = Some(Attribute::new(
                element.attributes().as_slice()[position].name().clone(),
                value,
            ));
            // Registry redirects and known shared-wrapper restoration are
            // preadmitted. The actual slot producer then commits all or none.
            payload_write::<[StringLocation; 2]>(control, 2)?;
            let mut next_state = self.read_input_state(control, slot)?;
            next_state.value_rewritten_during_creation = true;
            payload_write::<HiddenInputState>(control, 1)?;
            payload_write::<[Option<AttributeStorage>; 2]>(control, 2)?;
            control.charge(WorkCategory::ProviderCall, 2)?;
            payload_write::<OrderedAttributes>(control, 1)?;
            self.retire_field_group(control, old_slot)?;
            let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
                unreachable!()
            };
            let AttributeStorage::Frozen {
                values,
                allocation,
                mut shared,
                element,
            } = storage.take().expect("actual native attributes")
            else {
                unreachable!()
            };
            drop(values);
            let HtmlNodeKind::Element(record) = &mut self.records[node].kind else {
                unreachable!()
            };
            let result = control.replace_attribute_slot(
                record.attributes.parser_backing_mut(),
                &mut shared,
                &allocation,
                position,
                &mut incoming,
            );
            *storage = Some(AttributeStorage::Frozen {
                values: record.attributes.clone(),
                allocation,
                shared,
                element,
            });
            match result {
                Ok(old) => {
                    self.bind_string(old_slot, StringLocation::Detached);
                    self.bind_string(
                        new_slot,
                        StringLocation::AttributeValue(container, position),
                    );
                    let RetainedPayload::HiddenInput { state, .. } =
                        &mut self.payloads.values[slot]
                    else {
                        unreachable!()
                    };
                    *state = next_state;
                    drop(old);
                    Ok(())
                }
                Err(stop) => {
                    drop(incoming);
                    Err(stop.into())
                }
            }
        })()
    }
    fn reset_hidden_input(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
    ) -> Result<(), SessionError> {
        let mut state = self.read_input_state(control, slot)?;
        state.dirty_value = false;
        state.dirty_checkedness = false;
        state.user_validity = false;
        self.write_input_state(control, slot, state)?;
        let mut checked = false;
        let mut value = None;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self.records[node].kind else {
            unreachable!()
        };
        for (position, attribute) in element.attributes().as_slice().iter().enumerate() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(attribute.name().local_name()), "checked")? {
                checked = true;
            }
            if exact_ascii(control, Some(attribute.name().local_name()), "value")? {
                value = Some(position);
            }
        }
        let text = if let Some(position) = value {
            let source = self.attribute_value_slot(control, node, position)?;
            copy_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                source,
                false,
            )?
        } else {
            FieldCopy {
                value: literal_string(control, b"")?,
                origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                original: None,
            }
        };
        self.replace_input_raw(control, slot, text)?;
        state.checkedness = checked;
        self.write_input_state(control, slot, state)?;
        state.selected_files_empty = true;
        self.write_input_state(control, slot, state)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::HiddenInput { hidden, .. } = &self.payloads.values[slot] else {
            unreachable!()
        };
        if !*hidden {
            return Err(SessionError::WrongToken);
        }
        payload_write::<bool>(control, 1)?;
        let RetainedPayload::HiddenInput { complete, .. } = &mut self.payloads.values[slot] else {
            unreachable!()
        };
        *complete = true;
        #[cfg(test)]
        if let Some(cancellation) = self.cancel_after_hidden_reset.take() {
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
        if Arc::strong_count(old.backing()) != 1 {
            return Err(SessionError::AliasedPayload);
        }
        if reset {
            payload_write::<TextareaState>(control, 1)?;
            state.dirty_value = false;
            state.user_validity = false;
            let RetainedPayload::Textarea { state: stored, .. } = &mut self.payloads.values[slot]
            else {
                unreachable!()
            };
            *stored = state;
        } else if state.dirty_value {
            return Ok(());
        }
        let dispose_previous = self.releasable_string(control, state.raw, false)?;
        let mut raw = Utf16Buffer::new(control)?;
        let mut group = None;
        let mut units = 0usize;
        let copied = (|| {
            control.charge(WorkCategory::RecordVisit, 1)?;
            for position in 0..self.records[node].children.len() {
                control.charge(WorkCategory::RecordVisit, 2)?;
                let child = self.records[node].children[position];
                if Some(child) != detached
                    && let HtmlNodeKind::Text(value) = &self.records[child].kind
                {
                    raw.extend_units(value.code_units(), control)?;
                    let added = value.len();
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
                                .merge_groups(control, &previous, &incoming, units, units + added)?
                        } else {
                            incoming
                        });
                    }
                    units += added;
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
        let RetainedPayload::Textarea { state: stored, .. } = &mut self.payloads.values[slot]
        else {
            unreachable!()
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
        if Arc::strong_count(text.value.text().backing()) != 1 {
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
        if let Err(error) = self.prepare_payload_slot_owned(control) {
            value.release(control)?;
            return Err(error);
        }
        let (value, allocation) = value.into_parts();
        let index = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::String {
            value,
            allocation,
            origins,
            location: StringLocation::Detached,
        });
        let RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        } = &mut self.payloads.values[index]
        else {
            unreachable!()
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
        let Tag {
            name, attributes, ..
        } = input.take().expect("checked start tag");
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
            let attribute = tag
                .attributes
                .get_mut(index, control)?
                .expect("indexed token attribute");
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
            let document = matches!(self.records[0].kind, HtmlNodeKind::Document(_))
                .then_some(DocumentRef::Owned(0));
            self.prepare_document_allocation(control, document, template)
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
            let mut units = name.len().checked_add(namespace.len()).ok_or_else(|| {
                control
                    .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                    .unwrap_err()
            })?;
            let (values, allocation) = control.new_vector()?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let original_len = attributes.len();
            if let Err(error) = self.prepare_payload_slot(control) {
                release_vector(control, values, allocation)?;
                return Err(error);
            }
            let container = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Attributes(Some(
                AttributeStorage::Building {
                    values,
                    allocation,
                    original_len,
                    reversed_pairs: 0,
                },
            )));
            while let Some(attribute) = attributes.try_pop(control)? {
                let attribute_name = self.register_text(control, attribute.name);
                let (local, local_ticket) = match attribute_name {
                    Ok(value) => value,
                    Err(error) => {
                        attribute.value.release(control)?;
                        return Err(error);
                    }
                };
                let (value, value_ticket) = self.register_text(control, attribute.value)?;
                control.charge(WorkCategory::RecordVisit, 2)?;
                units = units
                    .checked_add(local.len())
                    .and_then(|sum| sum.checked_add(value.len()))
                    .ok_or_else(|| {
                        control
                            .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                            .unwrap_err()
                    })?;
                let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                })) = &mut self.payloads.values[container]
                else {
                    unreachable!()
                };
                control.reserve_vector(values, allocation, 1)?;
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
                );
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(container, position),
                );
            }
            self.reverse_attributes(control, container)?;
            let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
                unreachable!()
            };
            let AttributeStorage::Building {
                values,
                allocation,
                original_len,
                reversed_pairs,
            } = storage.take().expect("attribute builder phase")
            else {
                unreachable!()
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
            let ordered = match OrderedAttributes::from_parser_storage(backing, control) {
                Ok(values) => values,
                Err((error, backing)) => {
                    // Preserve the admitted shared wrapper in the owning phase
                    // even when intrinsic duplicate checking or work stops.
                    let values = control.release_shared(backing, shared).unwrap_or_else(|_| {
                        panic!("unpublished attribute builder is uniquely owned")
                    });
                    *storage = Some(AttributeStorage::Building {
                        values,
                        allocation,
                        original_len,
                        reversed_pairs,
                    });
                    return Err(match error {
                        ParserAttributeError::Stop(stop) => stop.into(),
                        ParserAttributeError::Duplicate { .. } => SessionError::DuplicateAttribute,
                    });
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
            let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { values, .. })) =
                &self.payloads.values[container]
            else {
                unreachable!()
            };
            let attributes = values.clone();
            let count = attributes.len();
            let template = exact_ascii(control, Some(&name), "template")?;
            control.charge(WorkCategory::RecordWrite, 6)?;
            let node = self.allocate_prepared_owned(
                control,
                HtmlNodeKind::Element(HtmlElement {
                    name: QualifiedName::new(Namespace::from_uri(namespace), None, name),
                    attributes,
                }),
                template,
                0,
                0,
                prepared,
            )?;
            self.bind_string(name_ticket, StringLocation::ElementName(node.index));
            self.bind_string(namespace_ticket, StringLocation::Namespace(node.index));
            let RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element, .. })) =
                &mut self.payloads.values[container]
            else {
                unreachable!()
            };
            *element = Some(node.index);
            self.usage.attributes += count;
            self.usage.utf16_units += units;
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
            if self.records[before].parent != Some(parent_index) {
                return Err(SessionError::WrongSibling);
            }
        }
        let text = input.as_mut().ok_or(SessionError::Consumed)?;
        Self::check_text(control, text)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        if text.value.text().is_empty()
            || matches!(self.records[parent_index].kind, HtmlNodeKind::Document(_))
        {
            input.take().expect("verified text").release(control)?;
            return Ok(None);
        }
        let mut preceding = None;
        for position in 0..self.records[parent_index].children.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let child = self.records[parent_index].children[position];
            if Some(child) == before_index {
                break;
            }
            preceding = Some(child);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        let preceding =
            preceding.filter(|&index| matches!(self.records[index].kind, HtmlNodeKind::Text(_)));
        let old_ticket = if let Some(index) = preceding {
            let mut ticket = None;
            for position in 0..self.payloads.values.len() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if matches!(&self.payloads.values[position], RetainedPayload::String { location: StringLocation::Text(owner), .. } if *owner == index)
                {
                    ticket = Some(position);
                    break;
                }
            }
            Some(ticket.ok_or(SessionError::WrongToken)?)
        } else {
            None
        };
        let dispose_old = match old_ticket {
            Some(slot) => self.releasable_string(control, slot, true)?,
            None => false,
        };
        let prepared = if preceding.is_none() {
            let document = match self.node_document_ref(control, parent_index) {
                Ok(document) => Some(document),
                Err(SessionError::NodeDocumentUnavailable)
                    if matches!(self.records[0].kind, HtmlNodeKind::Fragment) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            self.prepare_document_allocation(control, document, false)?
        } else {
            None
        };
        let text = input.take().expect("verified text");
        let (value, ticket) = self.register_text(control, text)?;
        if let Some(index) = preceding {
            let HtmlNodeKind::Text(old) = &self.records[index].kind else {
                unreachable!()
            };
            control.charge(WorkCategory::RecordVisit, 2)?;
            let offset = old.len();
            let added = value.len();
            let mut merged = Utf16Buffer::new(control)?;
            if let Err(stop) = merged
                .extend_units(old.code_units(), control)
                .and_then(|()| merged.extend_units(value.code_units(), control))
            {
                merged.release(control)?;
                return Err(stop.into());
            }
            let merged = merged.freeze(control)?;
            let group = match self.merge_field_groups(
                control,
                old_ticket.expect("registered preceding text"),
                ticket,
                offset,
                offset + added,
            ) {
                Ok(group) => group,
                Err(error) => {
                    merged.release(control)?;
                    return Err(error);
                }
            };
            let (merged, merged_ticket) =
                self.register_group_string_owned(control, merged, group)?;
            let node = self.node(control, index)?;
            let children_changed = self.textarea_slot(control, parent_index)?.is_some()
                || self.html_local(control, parent_index, "script")?;
            if children_changed {
                payload_write::<Option<FormEffects>>(control, 1)?;
            }
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<DocumentString>() as u64,
            )?;
            control.charge(WorkCategory::ProviderCall, 1)?;
            control.charge(WorkCategory::RecordWrite, 4)?;
            if dispose_old {
                payload_write::<RetainedPayload>(control, 2)?;
            }
            self.retire_field_group(control, old_ticket.expect("registered preceding text"))?;
            // Fixed commit after all fallible work. The old and incoming backing
            // remain owned and fully charged, detached from the live native slot.
            self.bind_string(
                old_ticket.expect("registered preceding text"),
                StringLocation::Detached,
            );
            self.bind_string(merged_ticket, StringLocation::Text(index));
            self.records[index].kind = HtmlNodeKind::Text(merged);
            if dispose_old {
                self.dispose_aggregate_string(
                    control,
                    old_ticket.expect("registered preceding text"),
                )?;
            }
            self.usage.utf16_units += added;
            if children_changed {
                self.effects.form = Some(FormEffects {
                    root: parent_index,
                    old_parent: None,
                    new_parent: Some(parent_index),
                    stage: FormEffectStage::InsertionChildren,
                    insert: false,
                    cause: InsertionCause::NonParser,
                    next: None,
                    pass: FormEffectPass::Local,
                });
                self.drain_form_effects(control)?;
            }
            Ok(Some(TextInsertion { node, offset }))
        } else {
            let units = value.len();
            control.charge(WorkCategory::RecordWrite, 2)?;
            let node = self.allocate_prepared_owned(
                control,
                HtmlNodeKind::Text(value),
                false,
                0,
                0,
                prepared,
            )?;
            self.bind_string(ticket, StringLocation::Text(node.index));
            self.usage.utf16_units += units;
            self.insert(control, parent, &node, before, InsertionCause::NonParser)?;
            Ok(Some(TextInsertion { node, offset: 0 }))
        }
    }
    fn releasable_string(
        &self,
        control: &mut ParseControl,
        slot: usize,
        native: bool,
    ) -> Result<bool, SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        } = &self.payloads.values[slot]
        else {
            return Err(SessionError::WrongToken);
        };
        control.require_document_string(allocation, value)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        if Arc::strong_count(value.backing()) != if native { 2 } else { 1 } {
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
        let RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        } = mem::replace(&mut self.payloads.values[slot], RetainedPayload::Retired)
        else {
            return Err(SessionError::WrongToken);
        };
        drop(value);
        control.release_document_string(allocation)?;
        if let Some(origins) = origins {
            self.payloads.values[slot] = RetainedPayload::HistoricalString { origins };
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
    fn bind_string(&mut self, index: usize, location: StringLocation) {
        let RetainedPayload::String {
            location: current, ..
        } = &mut self.payloads.values[index]
        else {
            unreachable!()
        };
        *current = location;
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
        let HtmlNodeKind::Element(element) = &self.records[target].kind else {
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
        let mut container = None;
        for index in 0..self.payloads.values.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(&self.payloads.values[index], RetainedPayload::Attributes(Some(AttributeStorage::Frozen { element: Some(owner), .. })) if *owner == target)
            {
                container = Some(index);
                break;
            }
        }
        let container = container.ok_or(SessionError::AttributeStorageUnavailable)?;
        self.payloads.check_aliases(&self.records, control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        if Arc::weak_count(element.attributes().backing()) != 1 {
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
            let a = tag
                .attributes
                .get_mut(index, control)?
                .expect("actual attribute");
            Self::check_text(control, &mut a.name)?;
            Self::check_text(control, &mut a.value)?;
            lowercase_token_name(control, a.name.value.text())?;
            check_text_source(control, &a.name)?;
            check_text_source(control, &a.value)?;
            control.require_source_range(&a.source)?;
        }
        let mut missing = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let planned = (|| {
            let mut count = 0;
            for index in 0..tag.attributes.len() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                let name = tag.attributes.as_slice()[index].name.value.text();
                let mut found = false;
                for old in element.attributes().as_slice() {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    if old.name().namespace().uri().is_none()
                        && exact_string_equal(control, old.name().local_name(), name)?
                    {
                        found = true;
                        break;
                    }
                }
                // A real tokenizer removes lexical duplicates. Independently
                // retain the first even if another concrete producer supplies
                // duplicate names: no trust in a tokenizer boolean/proxy proof.
                for prior in 0..index {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    if exact_string_equal(
                        control,
                        tag.attributes.as_slice()[prior].name.value.text(),
                        name,
                    )? {
                        found = true;
                        break;
                    }
                }
                missing.try_push(control, !found)?;
                count += usize::from(!found);
            }
            Ok::<_, SessionError>(count)
        })();
        let count = match planned {
            Ok(count) => count,
            Err(error) => {
                missing.release(control)?;
                return Err(error);
            }
        };
        // Final borrowed-input admission; every rejected preflight leaves Tag
        // in the caller. Staging after this point owns all partially moved data.
        if let Err(error) = payload_write::<Option<Tag>>(control, 1) {
            missing.release(control)?;
            return Err(error);
        }
        let Tag {
            name,
            mut attributes,
            ..
        } = input.take().expect("checked start html");
        let result = (|| {
            name.release(control)?;
            let (values, allocation) = control.new_vector()?;
            if let Err(error) = self.prepare_payload_slot(control) {
                release_vector(control, values, allocation)?;
                return Err(error);
            }
            let stage = self.payloads.values.len();
            self.payloads.values.push(RetainedPayload::Attributes(Some(
                AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: count,
                    reversed_pairs: 0,
                },
            )));
            let mut remaining = count;
            let mut units = 0usize;
            while !attributes.as_slice().is_empty() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                let keep = missing
                    .try_pop(control)?
                    .expect("one real decision per attribute");
                let attribute = attributes.try_pop(control)?.expect("actual attribute");
                if !keep {
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
                control.charge(WorkCategory::RecordVisit, 2)?;
                units = units
                    .checked_add(local.len())
                    .and_then(|v| v.checked_add(value.len()))
                    .ok_or_else(|| {
                        control
                            .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                            .unwrap_err()
                    })?;
                let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    ..
                })) = &mut self.payloads.values[stage]
                else {
                    unreachable!()
                };
                control.reserve_vector(values, allocation, 1)?;
                control.charge(
                    WorkCategory::CopyByte,
                    (mem::size_of::<Attribute>() + 2 * mem::size_of::<StringLocation>()) as u64,
                )?;
                control.charge(WorkCategory::RecordWrite, 3)?;
                control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
                values.push(Attribute::new(
                    QualifiedName::new(Namespace::none(), None, local),
                    value,
                ));
                remaining -= 1;
                self.bind_string(name_ticket, StringLocation::AttributeName(stage, remaining));
                self.bind_string(
                    value_ticket,
                    StringLocation::AttributeValue(stage, remaining),
                );
            }
            if count == 0 {
                return Ok(None);
            }
            // The actual stage remains reversed: last-pop yields source order
            // without a variable shift or a second attribute representation.
            self.edit_attributes(control, target, container, stage, count, false)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            let old_len = match &self.records[target].kind {
                HtmlNodeKind::Element(v) => v.attributes().len(),
                _ => unreachable!(),
            };
            self.begin_attribute_transfer(control, stage, container, old_len)?;
            payload_write::<PreparedAttributeMerge>(control, 1)?;
            Ok(Some(PreparedAttributeMerge {
                target,
                container,
                stage,
                start: old_len,
                count,
                units,
            }))
        })();
        missing.release(control)?;
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
            units,
        } = prepared;
        for _ in 0..count {
            self.edit_attributes(control, target, container, stage, 1, true)?;
            if self.effects.form.is_some() {
                self.drain_form_effects(control)?;
            }
        }
        for index in 0..self.payloads.values.len() {
            self.rebind_attribute_location(control, index, stage, container, start)?;
        }
        // Clearing the redirect is safe only after every original location was
        // rebound. A stop above retains the exact transfer phase.
        payload_write::<Option<AttributeStorage>>(control, 2)?;
        payload_write::<usize>(control, 2)?;
        let RetainedPayload::Attributes(storage) = &mut self.payloads.values[stage] else {
            unreachable!()
        };
        let AttributeStorage::Transferring {
            values, allocation, ..
        } = storage.take().expect("retained transfer phase")
        else {
            unreachable!()
        };
        *storage = Some(AttributeStorage::Building {
            values,
            allocation,
            original_len: 0,
            reversed_pairs: 0,
        });
        self.usage.attributes += count;
        self.usage.utf16_units += units;
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
        let RetainedPayload::Attributes(storage) = &mut self.payloads.values[stage] else {
            unreachable!()
        };
        let AttributeStorage::Building {
            values,
            allocation,
            original_len,
            reversed_pairs,
        } = storage.take().expect("real staged attributes")
        else {
            unreachable!()
        };
        assert_eq!(
            reversed_pairs, 0,
            "last-pop consumes the unchanged reverse stage"
        );
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
        if let RetainedPayload::String { location, .. } = &mut self.payloads.values[index] {
            control.charge(
                WorkCategory::CopyByte,
                (2 * mem::size_of::<StringLocation>()) as u64,
            )?;
            control.charge(WorkCategory::RecordWrite, 1)?;
            #[cfg(test)]
            control.observe_registry_location_write(mem::size_of::<StringLocation>());
            *location = match *location {
                StringLocation::AttributeName(owner, index) if owner == stage => {
                    StringLocation::AttributeName(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control
                                .representation::<()>(HtmlParseResource::ManagedAttributes)
                                .unwrap_err()
                        })?,
                    )
                }
                StringLocation::AttributeValue(owner, index) if owner == stage => {
                    StringLocation::AttributeValue(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control
                                .representation::<()>(HtmlParseResource::ManagedAttributes)
                                .unwrap_err()
                        })?,
                    )
                }
                StringLocation::AttributeNamespace(owner, index) if owner == stage => {
                    StringLocation::AttributeNamespace(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control
                                .representation::<()>(HtmlParseResource::ManagedAttributes)
                                .unwrap_err()
                        })?,
                    )
                }
                StringLocation::AttributePrefix(owner, index) if owner == stage => {
                    StringLocation::AttributePrefix(
                        destination,
                        start.checked_add(index).ok_or_else(|| {
                            control
                                .representation::<()>(HtmlParseResource::ManagedAttributes)
                                .unwrap_err()
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
        let RetainedPayload::Attributes(Some(AttributeStorage::Transferring { values, .. })) =
            &self.payloads.values[stage]
        else {
            return Err(control
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        };
        let attribute = values.last().ok_or_else(|| {
            SessionError::Stop(
                control.fail_invariant(crate::parser::control::Invariant::PendingFormEffects),
            )
        })?;
        if attribute.name().namespace().uri().is_some()
            || !exact_ascii(control, Some(attribute.name().local_name()), "id")?
        {
            return Ok(None);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        if attribute.value().is_empty() {
            return Ok(None);
        }
        for index in 0..self.payloads.values.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            } = &self.payloads.values[index]
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
        let (earlier, later) = self.payloads.values.split_at_mut(stage);
        let RetainedPayload::Attributes(storage) = &mut earlier[container] else {
            unreachable!()
        };
        let AttributeStorage::Frozen {
            values,
            mut allocation,
            mut shared,
            element,
        } = storage.take().expect("real attribute phase")
        else {
            unreachable!()
        };
        drop(values);
        let HtmlNodeKind::Element(record) = &mut self.records[target].kind else {
            unreachable!()
        };
        let result = if append {
            let RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
                values,
                allocation: incoming_allocation,
                ..
            })) = &mut later[0]
            else {
                unreachable!()
            };
            control.append_attribute_slot(
                record.attributes.parser_backing_mut(),
                &mut shared,
                &allocation,
                values,
                incoming_allocation,
            )
        } else {
            control.reserve_attribute_storage(
                record.attributes.parser_backing_mut(),
                &mut shared,
                &mut allocation,
                count,
            )
        };
        let values = record.attributes.clone();
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        if append && result.is_ok() {
            let RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved, .. })) =
                &mut later[0]
            else {
                unreachable!()
            };
            *moved += 1;
            if form_effects.is_some() {
                self.effects.form = form_effects;
            }
        }
        result.map_err(Into::into)
    }
    fn reverse_attributes(
        &mut self,
        control: &mut ParseControl,
        container: usize,
    ) -> Result<(), SessionError> {
        let RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. })) =
            &self.payloads.values[container]
        else {
            unreachable!()
        };
        let len = values.len();
        for left in 0..len / 2 {
            let right = len - 1 - left;
            control.admit_atomic_layout(mem::size_of::<Attribute>() * 2)?;
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<Attribute>() * 2) as u64,
            )?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.charge(WorkCategory::RecordWrite, 3)?;
            let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                reversed_pairs,
                ..
            })) = &mut self.payloads.values[container]
            else {
                unreachable!()
            };
            values.swap(left, right);
            *reversed_pairs += 1;
        }
        Ok(())
    }

    fn template_contents(
        &self,
        control: &mut ParseControl,
        element: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        let index = self.index(control, element)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.records[index]
            .template_contents
            .map(|index| self.node(control, index))
            .transpose()
    }

    fn container(&self, control: &mut ParseControl, index: usize) -> Result<(), SessionError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.records[index].template_contents.is_some() {
            return Err(SessionError::TemplateContentsRequired);
        }
        match self.records[index].kind {
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_) => Ok(()),
            _ => Err(SessionError::NotContainer),
        }
    }

    /// Follow ordinary or association parents, with one proposed ordinary move.
    /// No recursive tree or temporary ancestry allocation is needed.
    fn depth(
        &self,
        control: &mut ParseControl,
        mut index: usize,
        change: Option<(usize, Option<usize>)>,
    ) -> Result<usize, SessionError> {
        let mut depth = 0;
        loop {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let record = &self.records[index];
            let parent = if let Some((_, parent)) = change.filter(|(node, _)| *node == index) {
                parent
            } else if let Some(owner) = &record.owner {
                let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
                    owner;
                Some(owner.index)
            } else {
                record.parent
            };
            let Some(parent) = parent else {
                return Ok(depth);
            };
            depth += 1;
            if depth > self.records.len() {
                return Err(SessionError::Cycle);
            }
            index = parent;
        }
    }
    fn maximum_depth(
        &self,
        control: &mut ParseControl,
        change: Option<(usize, Option<usize>)>,
    ) -> Result<usize, SessionError> {
        let mut maximum = 0;
        for index in 0..self.records.len() {
            maximum = maximum.max(self.depth(control, index, change)?);
        }
        Ok(maximum)
    }

    /// Actual entry into an associated document leaves the closed ordinary
    /// producer inventory. This qualifies topology mutation, not preparation's
    /// source-ordered ordinary connectivity condition.
    fn associated_document_parent(
        &self,
        control: &mut ParseControl,
        mut index: usize,
    ) -> Result<bool, SessionError> {
        for _ in 0..self.records.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let record = self.records.get(index).ok_or(SessionError::ForeignNode)?;
            if let Some(owner) = &record.owner {
                match owner {
                    HtmlFragmentOwner::ShadowHost(_) => return Ok(self.fragment.is_none()),
                    HtmlFragmentOwner::Template(template) => {
                        let (_, state) = self.template_slot(control, template.index)?;
                        if state.contents != index {
                            return Err(SessionError::TemplateMetadataUnavailable);
                        }
                        return Ok(false);
                    }
                }
            }
            let Some(parent) = record.parent else {
                return Ok(false);
            };
            index = parent;
        }
        Err(SessionError::Cycle)
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
        ordinary_view_parent(&self.records, control, node, detached)
    }

    fn ordinary_root(
        &self,
        control: &mut ParseControl,
        mut node: usize,
        detached: Option<usize>,
    ) -> Result<usize, SessionError> {
        for _ in 0..self.records.len() {
            match self.ordinary_parent(control, node, detached)? {
                Some(parent) => node = parent,
                None => return Ok(node),
            }
        }
        Err(SessionError::Cycle)
    }

    fn ordinary_descendant(
        &self,
        control: &mut ParseControl,
        mut node: usize,
        root: usize,
    ) -> Result<bool, SessionError> {
        for _ in 0..self.records.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if node == root {
                return Ok(true);
            }
            match self.ordinary_parent(control, node, None)? {
                Some(parent) => node = parent,
                None => return Ok(false),
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
        control.charge(WorkCategory::RecordVisit, 3)?;
        let record = self.records.get(node).ok_or(SessionError::ForeignNode)?;
        let HtmlNodeKind::Element(element) = &record.kind else {
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
        for _ in 0..self.records.len() {
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
        let old_parent = self.records[root].parent;
        let mut children_changed = false;
        for parent in [old_parent.filter(|_| remove), new_parent]
            .into_iter()
            .flatten()
        {
            if self.textarea_slot(control, parent)?.is_some()
                || self.html_local(control, parent, "script")?
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
        let mut listed_explicit = false;
        for slot in 0..self.payloads.values.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            } = &self.payloads.values[slot]
            {
                control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                if self.ordinary_descendant(control, *node, root)? {
                    local = true;
                    break;
                }
                if state.kind.listed() && self.form_attribute(control, *node, "form")?.is_some() {
                    listed_explicit = true;
                }
            }
        }
        if !local && listed_explicit {
            let mut next = Some(root);
            while let Some(node) = next {
                if self.has_id(control, node)? {
                    local = true;
                    break;
                }
                next = self.next_effect_node(control, root, node)?;
            }
        }
        let mut next = Some(root);
        while let Some(node) = next {
            for name in ["option", "optgroup", "selectedcontent", "script"] {
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
            next = self.next_effect_node(control, root, node)?;
        }
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
        let record = self.records.get(node).ok_or(SessionError::ForeignNode)?;
        let HtmlNodeKind::Element(element) = &record.kind else {
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
        next_ordinary_view_node(&self.records, control, root, node, detached)
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
            self.records
                .get(root)
                .ok_or(SessionError::ForeignNode)?
                .kind,
            HtmlNodeKind::Document(_)
        ))
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
        // Source reset step1 is an actual effect before any lookup work.
        state.parser_inserted = false;
        state.evidence = FormAssociationEvidence::Reset;
        set_form_association_state(self, control, slot, state)?;
        let explicit = if state.kind.listed() {
            self.form_attribute(control, node, "form")?.is_some()
        } else {
            false
        };
        if !explicit
            && let Some(owner) = state.owner
            && self.nearest_form(control, node, detached)? == Some(owner)
        {
            return Ok(());
        }
        state.owner = None;
        set_form_association_state(self, control, slot, state)?;
        state.owner = if explicit && self.connected(control, node, detached)? {
            let value = self
                .form_attribute(control, node, "form")?
                .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
            self.first_id_form(control, node, value, detached)?
        } else {
            self.nearest_form(control, node, detached)?
        };
        if state.owner.is_some() {
            set_form_association_state(self, control, slot, state)?;
        }
        Ok(())
    }

    /// Called synchronously only by the mutation that installed this phase.
    /// No consumer resume method exists. After a stop only disposal is allowed.
    fn step_form_effects(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
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
                        select::EffectFrame::AdoptSubtree {
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
                            select::EffectFrame::ScriptPreparation {
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
                        select::EffectFrame::InsertionPostConnection {
                            roots: select::PostConnectionRoots::Single(effects.root),
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
                        select::EffectFrame::ScriptPreparation {
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
                                if let Some(owner) = state.owner
                                    && self.ordinary_root(control, node, detached)?
                                        != self.ordinary_root(control, owner, detached)?
                                {
                                    self.reset_form_association(
                                        control, slot, node, state, detached,
                                    )?;
                                }
                            }
                            FormEffectStage::Insertion if !state.parser_inserted => {
                                self.reset_form_association(control, slot, node, state, None)?;
                            }
                            FormEffectStage::Insertion | FormEffectStage::IdChange => {}
                            FormEffectStage::Adoption
                            | FormEffectStage::RemovalChildren
                            | FormEffectStage::InsertionChildren => {
                                unreachable!("children stage handled above")
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
                            FormEffectStage::Adoption
                            | FormEffectStage::RemovalChildren
                            | FormEffectStage::InsertionChildren => {
                                unreachable!("children stage handled above")
                            }
                        };
                    if id_event {
                        effects.pass = FormEffectPass::Global {
                            next_slot: 0,
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
                        self.intrinsic_tree_hook(
                            control,
                            node,
                            detached,
                            effects.stage == FormEffectStage::Removal,
                        )?;
                    }
                    return Ok(());
                }
                FormEffectPass::Global { next_slot, tree } => {
                    if next_slot < self.payloads.values.len() {
                        control.charge(WorkCategory::RecordVisit, 1)?;
                        if let RetainedPayload::FormAssociation {
                            node: Some(candidate),
                            state,
                        } = &self.payloads.values[next_slot]
                        {
                            control.charge(
                                WorkCategory::CopyByte,
                                (mem::size_of::<usize>() + mem::size_of::<FormAssociationState>())
                                    as u64,
                            )?;
                            let candidate = *candidate;
                            let state = *state;
                            // These records can only be issued by the same
                            // fresh ordinary Document creation owner. A
                            // Document ID event also affects its detached
                            // controls; connectivity is tested by reset.
                            if self.node_document_ref(control, candidate)?
                                == self.node_document_ref(control, node)?
                                && state.kind.listed()
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
                            next_slot: next_slot + 1,
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
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) => Err(control
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into()),
            Ok(()) => Ok(()),
        }
    }

    /// Prepared child order. The real replacement vector is admitted and every
    /// entry is read/copied before use. Commit later consists only of vector swaps.
    fn child_order(
        &self,
        control: &mut ParseControl,
        parent: usize,
        remove: usize,
        insert: Option<(usize, Option<usize>)>,
    ) -> Result<(Vec<usize>, VectorCapacity<usize>), SessionError> {
        let (mut order, mut allocation) = control.new_vector()?;
        let prepare = (|| {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.reserve_vector(
                &mut order,
                &mut allocation,
                self.records[parent].children.len() + 1,
            )?;
            let mut inserted = false;
            for index in 0..self.records[parent].children.len() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                let child = self.records[parent].children[index];
                if child == remove {
                    continue;
                }
                if let Some((node, Some(before))) = insert
                    && child == before
                {
                    control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                    control.charge(WorkCategory::RecordWrite, 1)?;
                    control.acquire(HtmlParseResource::AuxiliaryEntries, 1)?;
                    order.push(node);
                    inserted = true;
                }
                control.charge(WorkCategory::RecordWrite, 1)?;
                control.acquire(HtmlParseResource::AuxiliaryEntries, 1)?;
                order.push(child);
            }
            if let Some((node, before)) = insert
                && !inserted
            {
                if before.is_some() {
                    return Err(SessionError::WrongSibling);
                }
                control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
                control.charge(WorkCategory::RecordWrite, 1)?;
                control.acquire(HtmlParseResource::AuxiliaryEntries, 1)?;
                order.push(node);
            }
            Ok(())
        })();
        if let Err(error) = prepare {
            release_order(control, &self.control, order, allocation)?;
            return Err(error);
        }
        Ok((order, allocation))
    }

    fn insert(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        cause: InsertionCause,
    ) -> Result<(), SessionError> {
        let parent = self.index(control, parent)?;
        let node = self.index(control, node)?;
        let before = before.map(|node| self.index(control, node)).transpose()?;
        self.insert_indices_with_cause(control, parent, node, before, cause)?;
        self.drain_construction_effects(control)
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
        if node == 0 || matches!(self.records[node].kind, HtmlNodeKind::Document(_)) {
            return Err(SessionError::RootMove);
        }
        if self.records[node].owner.is_some() {
            return Err(SessionError::AssociationMove);
        }
        if let Some(before) = before {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if self.records[before].parent != Some(parent) {
                return Err(SessionError::WrongSibling);
            }
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
                return Ok(());
            }
        }
        let old_parent = self.records[node].parent;
        let mut form_effects =
            self.prepare_form_effects(control, node, old_parent.is_some(), true, Some(parent))?;
        if let Some(effects) = &mut form_effects {
            payload_write::<InsertionCause>(control, 1)?;
            effects.cause = cause;
        }
        let depth = self.maximum_depth(control, Some((node, Some(parent))))?;
        control.observe_tree_depth(depth as u64)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let revoke_standalone = self.standalone_ineligibility.is_none()
            && self.associated_document_parent(control, parent)?;
        let requests = [(
            HtmlParseResource::ManagedEdges,
            u64::from(old_parent.is_none()),
        )];
        control.check_acquire_many(&requests)?;
        let (order, allocation) = self.child_order(control, parent, node, Some((node, before)))?;
        let old_order = if let Some(old) = old_parent.filter(|old| *old != parent) {
            match self.child_order(control, old, node, None) {
                Ok(order) => Some((old, order)),
                Err(error) => {
                    release_order(control, &self.control, order, allocation)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        let admission = (|| {
            control.charge(WorkCategory::RecordWrite, 8)?;
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<Vec<usize>>() * 4) as u64,
            )?;
            if revoke_standalone {
                payload_write::<Option<StandaloneUnsupported>>(control, 1)?;
            }
            if form_effects.is_some() {
                payload_write::<Option<FormEffects>>(control, 1)?;
            }
            let scratch_entries = order.len() as u64
                + old_order
                    .as_ref()
                    .map_or(0, |(_, (order, _))| order.len() as u64);
            control.commit_child_orders(&self.control, scratch_entries, requests[0].1, 0)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = admission {
            release_order(control, &self.control, order, allocation)?;
            if let Some((_, (order, allocation))) = old_order {
                release_order(control, &self.control, order, allocation)?;
            }
            return Err(error);
        }
        let mut retired = self.swap_order(parent, order, allocation);
        let mut retired_old =
            old_order.map(|(old, (order, allocation))| self.swap_order(old, order, allocation));
        self.records[node].parent = Some(parent);
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        if revoke_standalone {
            self.standalone_ineligibility = Some(StandaloneUnsupported::ScriptDocument);
        }
        self.usage.edges += usize::from(old_parent.is_none());
        self.usage.depth = depth;
        // Complete the same fixed commit by retiring the old initialized
        // references. usize has no destructor, so clear is a bounded length
        // metadata write, not a useful-work traversal or fallible provider call.
        // No initialized retired scratch survives to the capacity cleanup below.
        retired.0.clear();
        if let Some((order, _)) = &mut retired_old {
            order.clear();
        }
        release_vector(control, retired.0, retired.1)?;
        if let Some((order, allocation)) = retired_old {
            release_vector(control, order, allocation)?;
        }
        Ok(())
    }

    fn swap_order(
        &mut self,
        index: usize,
        order: Vec<usize>,
        allocation: VectorCapacity<usize>,
    ) -> (Vec<usize>, VectorCapacity<usize>) {
        (
            mem::replace(&mut self.records[index].children, order),
            mem::replace(&mut self.children_allocations[index], allocation),
        )
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
        if node == 0 || matches!(self.records[node].kind, HtmlNodeKind::Document(_)) {
            return Err(SessionError::RootMove);
        }
        if self.records[node].owner.is_some() {
            return Err(SessionError::AssociationMove);
        }
        let Some(parent) = self.records[node].parent else {
            return Ok(());
        };
        let form_effects = self.prepare_form_effects(control, node, true, false, None)?;
        let depth = self.maximum_depth(control, Some((node, None)))?;
        control.observe_tree_depth(depth as u64)?;
        let (order, allocation) = self.child_order(control, parent, node, None)?;
        let admission = (|| {
            control.charge(WorkCategory::RecordWrite, 4)?;
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<Vec<usize>>() * 2) as u64,
            )?;
            if form_effects.is_some() {
                payload_write::<Option<FormEffects>>(control, 1)?;
            }
            control.commit_child_orders(&self.control, order.len() as u64, 0, 1)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = admission {
            release_order(control, &self.control, order, allocation)?;
            return Err(error);
        }
        let (mut order, allocation) = self.swap_order(parent, order, allocation);
        self.records[node].parent = None;
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        self.usage.edges -= 1;
        self.usage.depth = depth;
        order.clear();
        release_vector(control, order, allocation)?;
        Ok(())
    }

    fn prepare_script_owned(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        detached: Option<usize>,
    ) -> Result<DisabledScriptPreparation, SessionError> {
        let (slot, mut metadata) = script_metadata(&self.payloads, control, index)?;
        if metadata.already_started {
            return Ok(DisabledScriptPreparation::AlreadyStarted);
        }
        let saved_document = metadata.parser_document;
        metadata.parser_document = None;
        set_script_metadata(self, control, slot, metadata)?;
        if saved_document.is_some() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let HtmlNodeKind::Element(element) = &self.records[index].kind else {
                return Err(SessionError::ScriptMetadataUnavailable);
            };
            if script_attribute(control, element, "async")?.is_none() {
                metadata.force_async = true;
                set_script_metadata(self, control, slot, metadata)?;
            }
        }
        let mut empty = true;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let children = self.records[index].children.len();
        for position in 0..children {
            control.charge(WorkCategory::RecordVisit, 3)?;
            let child = self.records[index].children[position];
            let record = self.records.get(child).ok_or(SessionError::ForeignNode)?;
            if Some(child) != detached
                && let HtmlNodeKind::Text(text) = &record.kind
                && !text.is_empty()
            {
                empty = false;
                break;
            }
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self.records[index].kind else {
            unreachable!()
        };
        if empty && script_attribute(control, element, "src")?.is_none() {
            return Ok(DisabledScriptPreparation::Empty);
        }
        // HTML step 7 follows the parser-document/force-async and empty-source
        // prefix. Associated-domain eligibility was qualified at the actual
        // topology producer and rejected by the capability before any effects.
        let mut ancestor = Some(index);
        let mut connected = false;
        for _ in 0..self.records.len() {
            let Some(current) = ancestor else { break };
            control.charge(WorkCategory::RecordVisit, 1)?;
            let record = self.records.get(current).ok_or(SessionError::ForeignNode)?;
            if matches!(record.kind, HtmlNodeKind::Document(_)) {
                connected = true;
                ancestor = None;
                break;
            }
            ancestor = if Some(current) == detached {
                None
            } else {
                record.parent
            };
        }
        if ancestor.is_some() {
            return Err(SessionError::Cycle);
        }
        if !connected {
            return Ok(DisabledScriptPreparation::Disconnected);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self.records[index].kind else {
            unreachable!()
        };
        let Some(kind) = classify_script(control, element)? else {
            return Ok(DisabledScriptPreparation::UnrecognizedType);
        };
        metadata.script_type = Some(kind);
        set_script_metadata(self, control, slot, metadata)?;
        if saved_document.is_some() {
            metadata.parser_document = saved_document;
            metadata.force_async = false;
            set_script_metadata(self, control, slot, metadata)?;
        }
        metadata.already_started = true;
        set_script_metadata(self, control, slot, metadata)?;
        metadata.preparation_document = Some(self.node_document_ref(control, index)?);
        set_script_metadata(self, control, slot, metadata)?;
        if metadata.parser_document.is_some()
            && metadata.parser_document != metadata.preparation_document
        {
            return Ok(DisabledScriptPreparation::ParserDocumentMismatch);
        }
        Ok(DisabledScriptPreparation::Disabled(kind))
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
        let depth = self.maximum_depth(control, None)?;
        control.observe_tree_depth(depth as u64)?;
        for index in 0..self.records.len() {
            let depth = self.depth(control, index, None)?;
            control.charge(WorkCategory::RecordWrite, 1)?;
            self.records[index].depth = depth;
            control.charge(WorkCategory::RecordVisit, 1)?;
            for position in 0..self.records[index].children.len() {
                control.charge(WorkCategory::RecordVisit, 2)?;
                let child = self.records[index].children[position];
                if self.records[child].parent != Some(index) {
                    return Err(SessionError::WrongSibling);
                }
            }
        }
        self.usage.depth = depth;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let document = matches!(self.records[0].kind, HtmlNodeKind::Document(_));
        control.charge(WorkCategory::CopyByte, mem::size_of::<Arena>() as u64)?;
        if let Some(fragment) = &self.fragment {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let output = fragment.output.ok_or(SessionError::ContextUnavailable)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            if !matches!(self.records[output].kind, HtmlNodeKind::Fragment) {
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
    fn release(self, control: &mut ParseControl) {
        let Self {
            control: owner,
            effects,
            evidence,
            fragment,
            #[cfg(test)]
                cancel_after_removal: _,
            #[cfg(test)]
                cancel_after_textarea_reset: _,
            #[cfg(test)]
                cancel_after_output_reset: _,
            #[cfg(test)]
                cancel_after_hidden_reset: _,
            standalone_ineligibility: _,
            identity,
            identity_allocation,
            records,
            records_allocation,
            children_allocations,
            tickets_allocation,
            usage,
            payloads,
        } = self;
        if let Some(fragment) = fragment {
            fragment
                .release(control, &owner)
                .expect("actual context effects owner");
        }
        if let Some(evidence) = evidence {
            evidence.release(control).expect("actual diagnostic owner");
        }
        effects
            .release(control)
            .expect("actual construction effect owners");
        release_parts(
            control,
            &owner,
            ReleasedStorage {
                arena: Arena {
                    identity,
                    records,
                    usage,
                },
                records: records_allocation,
                children: children_allocations,
                tickets: tickets_allocation,
                identity: identity_allocation,
                payloads,
            },
        );
    }
}

/// Borrowed transitions retain the owner on every failure, including terminal
/// stops. Storage is absent only after successful freeze or explicit cleanup.
impl ParserArenaSession {
    /// Retains the actual fourteen ordinary formatting-start token families.
    /// Marker wrappers use ordinary allocation and never manufacture a recipe.
    pub(crate) fn retain_formatting_creation(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        source: crate::HtmlSourceRange,
    ) -> Result<FormattingCreation, SessionError> {
        self.validate_standalone_document(control)?;
        control.require_source_range(&source)?;
        let tag = input.as_ref().ok_or(SessionError::Consumed)?;
        check_formatting_tag(control, tag, &source)?;
        let mut formatting = false;
        for local in [
            "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong",
            "tt", "u",
        ] {
            if exact_ascii(control, Some(tag.name.value.text()), local)? {
                formatting = true;
                break;
            }
        }
        if !formatting {
            return Err(SessionError::WrongToken);
        }
        // Every fallible admission precedes the original token take.
        control.charge(WorkCategory::ProviderCall, 2)?;
        payload_write::<FormattingCreation>(control, 1)?;
        let arena = Arc::downgrade(&self.active()?.identity);
        let owner = control.identity();
        let cause = self
            .active()?
            .evidence
            .as_ref()
            .map(|evidence| evidence.current_cause(control))
            .transpose()?
            .flatten();
        let tag = take_payload(input, control)?;
        Ok(FormattingCreation {
            token: Some(tag),
            source,
            control: owner,
            arena,
            cause,
        })
    }

    fn require_formatting_creation<'a>(
        &self,
        control: &mut ParseControl,
        creation: &'a FormattingCreation,
    ) -> Result<&'a Tag, SessionError> {
        let storage = self.active()?;
        storage.require(control)?;
        control.require_identity(&creation.control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if creation.arena.as_ptr() != Arc::as_ptr(&storage.identity) {
            return Err(SessionError::ForeignNode);
        }
        control.require_source_range(&creation.source)?;
        let tag = creation.token.as_ref().ok_or(SessionError::Consumed)?;
        check_formatting_tag(control, tag, &creation.source)?;
        Ok(tag)
    }

    /// Creates a detached native element for the original token. The caller
    /// supplies the actual source-required intended parent, not current-node
    /// inference. Partial copied payloads remain in this arena or are disposed.
    pub(crate) fn create_formatting_from_creation(
        &mut self,
        control: &mut ParseControl,
        creation: &FormattingCreation,
        intended_parent: &ParserNode,
    ) -> Result<ParserNode, SessionError> {
        self.validate_standalone_document(control)?;
        let storage = self.active()?;
        let parent = storage.index(control, intended_parent)?;
        storage.container(control, parent)?;
        if storage.associated_document_parent(control, parent)? {
            return Err(control
                .stop_unsupported_standalone(StandaloneUnsupported::ScriptDocument)
                .into());
        }
        let tag = self.require_formatting_creation(control, creation)?;
        let document = storage.node_document_ref(control, parent)?;
        let mut prepared =
            self.active_mut()?
                .prepare_document_allocation(control, Some(document), false)?;
        let mut is_value = None;
        if self.active()?.fragment.is_some() {
            for attr in tag.attributes.as_slice() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if exact_ascii(control, Some(attr.name.value.text()), "is")? {
                    is_value = Some(&attr.value);
                    break;
                }
            }
        }
        self.active_mut()?.prepare_creation_registry(
            control,
            parent,
            tag.name.value.text(),
            is_value,
            true,
            &mut prepared,
        )?;
        let mut copy = Some(copy_formatting_tag(control, tag)?);
        let copied = match take_payload(&mut copy, control) {
            Ok(copied) => copied,
            Err(error) => {
                discard_formatting_tag(&mut copy, control)?;
                return Err(error);
            }
        };
        let previous = match self.active_mut()?.begin_formatting_replay(
            control,
            creation.cause.as_ref(),
            Some(parent),
        ) {
            Ok(previous) => previous,
            Err(error) => {
                copied.name.release(control)?;
                copied.attributes.release(control)?;
                return Err(error);
            }
        };
        let result = self.active_mut()?.allocate_html_payload_prepared(
            control,
            copied.name.value,
            Some(copied.name.origins),
            copied.attributes,
            prepared,
        );
        if control.stop().is_none() {
            self.active_mut()?
                .end_node_copy(control, previous.as_ref())?;
        }
        match result {
            Ok(node) => Ok(node),
            Err(error) => {
                // Preconsumption failure still owns the actual independent
                // copy; postconsumption failure owns it in the arena registry.
                discard_formatting_tag(&mut copy, control)?;
                Err(error)
            }
        }
    }

    /// Noah equality compares parsed attributes as created, independent of
    /// order. This closed issuer uses HTML elements and no-namespace parsed
    /// attribute names; prefixes/colon units are not reinterpreted namespaces.
    pub(crate) fn same_formatting_creation(
        &self,
        control: &mut ParseControl,
        left: &FormattingCreation,
        right: &FormattingCreation,
    ) -> Result<bool, SessionError> {
        let left = self.require_formatting_creation(control, left)?;
        let right = self.require_formatting_creation(control, right)?;
        if !exact_string_equal(control, left.name.value.text(), right.name.value.text())? {
            return Ok(false);
        }
        control.charge(WorkCategory::RecordVisit, 2)?;
        if left.attributes.len() != right.attributes.len() {
            return Ok(false);
        }
        for left_attribute in left.attributes.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let mut found = false;
            for right_attribute in right.attributes.as_slice() {
                control.charge(WorkCategory::RecordVisit, 1)?;
                if exact_string_equal(
                    control,
                    left_attribute.name.value.text(),
                    right_attribute.name.value.text(),
                )? {
                    if !exact_string_equal(
                        control,
                        left_attribute.value.value.text(),
                        right_attribute.value.value.text(),
                    )? {
                        return Ok(false);
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Ordered, individually atomic moves. Each committed prefix has coherent
    /// backlinks; a stopped suffix remains owned by its original parent. The
    /// existing move scans/orders/depth are repeated, so cost can be quadratic.
    pub(crate) fn reparent_children(
        &mut self,
        control: &mut ParseControl,
        from: &ParserNode,
        to: &ParserNode,
        capability: Option<&crate::parser::tree_builder::ConstructionCapability<'_>>,
    ) -> Result<(), SessionError> {
        let storage = self.active()?;
        let source = storage.index(control, from)?;
        let destination = storage.index(control, to)?;
        storage.container(control, source)?;
        storage.container(control, destination)?;
        if source == destination {
            return Err(SessionError::Cycle);
        }
        // A destination inside a source child cannot accept all those children.
        // Detect that impossible transfer before moving any earlier siblings.
        storage.depth(control, destination, Some((source, Some(destination))))?;
        loop {
            let storage = self.active()?;
            storage.require(control)?;
            control.charge(WorkCategory::RecordVisit, 1)?;
            if storage.records[source].children.is_empty() {
                return Ok(());
            }
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.charge(WorkCategory::CopyByte, mem::size_of::<usize>() as u64)?;
            let index = storage.records[source].children[0];
            let child = storage.node(control, index)?;
            self.insert(control, to, &child, None, capability)?;
        }
    }

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
        if !matches!(storage.records[0].kind, HtmlNodeKind::Document(_)) {
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
        if document.owner.as_ptr() != Arc::as_ptr(&storage.identity)
            || document.index != 0
            || !matches!(storage.records[0].kind, HtmlNodeKind::Document(_))
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
        for position in 0..tag.attributes.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let attribute = &tag.attributes.as_slice()[position];
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
        storage.prepare_payload_slot(control)?;
        let script = storage.payloads.values.len();
        payload_write::<Option<usize>>(control, 1)?;
        storage.payloads.values.push(RetainedPayload::Script {
            node: None,
            metadata: ScriptMetadata {
                parser_document: Some(DocumentRef::Owned(0)),
                preparation_document: None,
                force_async: false,
                already_started: inert,
                script_type: None,
                external_file: false,
                ready_to_be_parser_executed: false,
                delaying_load: false,
                result: ScriptResult::Uninitialized,
                steps_when_result_ready: None,
            },
        });
        let node = self.allocate_for_creation(
            control,
            input,
            &parent,
            None,
            CreationProfile::Ordinary,
            context,
        )?;
        let RetainedPayload::Script { node: bound, .. } =
            &mut self.active_mut()?.payloads.values[script]
        else {
            unreachable!()
        };
        *bound = Some(node.index);
        Ok(node)
    }
    pub(crate) fn script_metadata(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ScriptMetadata, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        script_metadata(&storage.payloads, control, node).map(|(_, metadata)| metadata)
    }
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
        metadata.already_started = true;
        set_script_metadata(storage, control, slot, metadata)
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
    pub(crate) fn allocate_hidden_input_for_creation(
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
            CreationProfile::Hidden(capability, token_source),
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
        if storage.associated_document_parent(control, parent)? {
            return Err(control
                .stop_unsupported_standalone(StandaloneUnsupported::ScriptDocument)
                .into());
        }
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
            for earlier in &tag.attributes.as_slice()[..index] {
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
        let mut kind = None;
        for (local, candidate) in [
            ("img", FormAssociationKind::Image),
            ("button", FormAssociationKind::Button),
            ("fieldset", FormAssociationKind::Fieldset),
            ("object", FormAssociationKind::Object),
            ("textarea", FormAssociationKind::Textarea),
            ("output", FormAssociationKind::Output),
        ] {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(tag.name.value.text()), local)? {
                kind = Some(candidate);
                break;
            }
        }
        let token_source = match profile {
            CreationProfile::Ordinary => None,
            CreationProfile::Hidden(capability, source)
            | CreationProfile::Select(capability, source) => {
                // Concrete borrowed capability, not a caller-supplied eligibility flag.
                capability.validate_session(control, self)?;
                Some(source)
            }
        };
        let select_metadata = if matches!(profile, CreationProfile::Select(..)) {
            select::initial_metadata(control, tag)?
        } else {
            None
        };
        if matches!(profile, CreationProfile::Select(..))
            && exact_ascii(control, Some(tag.name.value.text()), "select")?
        {
            kind = Some(FormAssociationKind::Select);
        }
        if !matches!(profile, CreationProfile::Ordinary) {
            storage.payloads.check_aliases(&storage.records, control)?;
            if matches!(profile, CreationProfile::Hidden(..))
                && (!exact_ascii(control, Some(tag.name.value.text()), "input")?
                    || !hidden_input_tag(control, tag)?)
            {
                return Err(SessionError::WrongToken);
            }
            let source = token_source.ok_or(SessionError::HiddenInputMetadataUnavailable)?;
            control.require_source_range(source)?;
            // Validate every original field against the actual token, including
            // empty-value attributes, before retaining any construction slot.
            check_formatting_tag(control, tag, source)?;
            if matches!(profile, CreationProfile::Hidden(..)) {
                validate_initial_input_origins(control, tag)?;
                kind = Some(FormAssociationKind::HiddenInput);
            }
        }
        let mut explicit_form = false;
        if kind.is_some_and(FormAssociationKind::listed) {
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
        let value_source = if kind == Some(FormAssociationKind::HiddenInput) {
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
        let hidden_sources = if let Some(token) =
            token_source.filter(|_| kind == Some(FormAssociationKind::HiddenInput))
        {
            control.charge(
                WorkCategory::ProviderCall,
                1 + u64::from(value_source.is_some()),
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
        let select_slot = if let Some(state) = select_metadata {
            storage.prepare_payload_slot(control)?;
            let slot = storage.payloads.values.len();
            storage
                .payloads
                .values
                .push(RetainedPayload::SelectFamily { node: None, state });
            Some(slot)
        } else {
            None
        };
        // Reserve actual independent empty raw backing and initialized metadata
        // BEFORE lexical ownership transfer. A stopped unbound slot is cleanup-only.
        let textarea_slot = if kind == Some(FormAssociationKind::Textarea) {
            let empty = literal_string(control, b"")?;
            let raw = storage.register_raw(control, empty)?;
            storage.prepare_payload_slot(control)?;
            let slot = storage.payloads.values.len();
            storage.payloads.values.push(RetainedPayload::Textarea {
                node: None,
                state: TextareaState {
                    raw,
                    dirty_value: false,
                    user_validity: false,
                },
            });
            Some(slot)
        } else {
            None
        };
        let output_slot = if kind == Some(FormAssociationKind::Output) {
            storage.prepare_payload_slot(control)?;
            let slot = storage.payloads.values.len();
            storage.payloads.values.push(RetainedPayload::Output {
                node: None,
                state: OutputState {
                    default_value_override: OutputDefaultValueOverride::Null,
                },
            });
            Some(slot)
        } else {
            None
        };
        let hidden_slot = if kind == Some(FormAssociationKind::HiddenInput) {
            let empty = literal_string(control, b"")?;
            let raw = storage.register_raw(control, empty)?;
            storage.prepare_payload_slot(control)?;
            let slot = storage.payloads.values.len();
            storage.payloads.values.push(RetainedPayload::HiddenInput {
                node: None,
                raw,
                hidden: false,
                complete: false,
                state: HiddenInputState {
                    dirty_value: false,
                    dirty_checkedness: false,
                    checkedness: false,
                    indeterminate: false,
                    user_validity: false,
                    selected_files_empty: true,
                    value_rewritten_during_creation: false,
                },
            });
            Some(slot)
        } else {
            None
        };
        let source_slot = if let Some((token, value_attribute)) = hidden_sources {
            let entries = 1 + u64::from(value_attribute.is_some());
            control.acquire(HtmlParseResource::SourceMapEntries, entries)?;
            if let Err(error) = storage.prepare_payload_slot(control) {
                release_logical(
                    control,
                    &storage.control,
                    HtmlParseResource::SourceMapEntries,
                    entries,
                );
                return Err(error);
            }
            let slot = storage.payloads.values.len();
            storage
                .payloads
                .values
                .push(RetainedPayload::HiddenInputSource {
                    node: None,
                    token,
                    value_attribute,
                });
            Some(slot)
        } else {
            None
        };
        let reset_before_association = textarea_slot.is_some()
            || output_slot.is_some()
            || hidden_slot.is_some()
            || kind == Some(FormAssociationKind::Select);
        let slot = if let Some(kind) = kind {
            storage.prepare_payload_slot(control)?;
            let slot = storage.payloads.values.len();
            storage
                .payloads
                .values
                .push(RetainedPayload::FormAssociation {
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
                });
            Some(slot)
        } else {
            None
        };
        let result = (|| {
            let Tag {
                name, attributes, ..
            } = take_payload(input, control)?;
            let node = if let Some(hidden_slot) = hidden_slot {
                storage.allocate_ordered_payload(
                    control,
                    name,
                    attributes,
                    prepared,
                    OrderedCreation::Hidden {
                        input: hidden_slot,
                        association: slot.expect("canonical hidden association"),
                        source: source_slot.expect("original hidden sources"),
                    },
                )?
            } else if matches!(profile, CreationProfile::Select(..)) {
                storage.allocate_ordered_payload(
                    control,
                    name,
                    attributes,
                    prepared,
                    OrderedCreation::Select {
                        metadata: select_slot,
                    },
                )?
            } else {
                storage.allocate_html_payload_prepared(
                    control,
                    name.value,
                    Some(name.origins),
                    attributes,
                    prepared,
                )?
            };
            if let Some(textarea) = textarea_slot {
                control.charge(WorkCategory::RecordVisit, 2)?;
                if !storage.records[node.index].children.is_empty() {
                    return Err(SessionError::WrongToken);
                }
                // Actual fresh node has no children. Its already admitted empty
                // raw is therefore exactly child text; reset precedes association.
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<TextareaState>(control, 1)?;
                let RetainedPayload::Textarea { node: bound, state } =
                    &mut storage.payloads.values[textarea]
                else {
                    unreachable!()
                };
                *bound = Some(node.index);
                state.dirty_value = false;
                state.user_validity = false;
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
                if !storage.records[node.index].children.is_empty() {
                    return Err(SessionError::WrongToken);
                }
                payload_write::<Option<usize>>(control, 1)?;
                payload_write::<OutputState>(control, 1)?;
                let RetainedPayload::Output { node: bound, state } =
                    &mut storage.payloads.values[output]
                else {
                    unreachable!()
                };
                *bound = Some(node.index);
                state.default_value_override = OutputDefaultValueOverride::Null;
                #[cfg(test)]
                if let Some(cancellation) = storage.cancel_after_output_reset.take() {
                    cancellation.cancel();
                }
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
                if reset_before_association {
                    payload_write::<FormAssociationState>(control, 1)?;
                    let RetainedPayload::FormAssociation { state, .. } =
                        &mut storage.payloads.values[slot]
                    else {
                        unreachable!()
                    };
                    state.owner = owner;
                    state.parser_inserted = owner.is_some();
                    state.evidence = if owner.is_some() {
                        FormAssociationEvidence::ParserAssociated
                    } else {
                        FormAssociationEvidence::Created
                    };
                }
            }
            Ok(node)
        })();
        match result {
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) if slot.is_some() || matches!(profile, CreationProfile::Select(..)) => {
                Err(control
                    .fail_invariant(
                        crate::parser::control::Invariant::IncompleteFormAssociationCreation,
                    )
                    .into())
            }
            other => other,
        }
    }

    pub(crate) fn hidden_input_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<HiddenInputState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        Ok(find_hidden_input_state(&storage.payloads, control, node)?.1)
    }
    /// Returns original token/value-attribute ranges only when creation actually
    /// rewrote the native value. A completed no-writeback input returns None;
    /// unavailable/imported/incomplete metadata returns its typed error.
    pub(crate) fn hidden_input_writeback_sources<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError>
    {
        let storage = self.active()?;
        let index = storage.index(control, node)?;
        find_hidden_input_sources(&storage.payloads, control, index)
    }
    pub(crate) fn append_hidden_input_internal_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        let raw = find_hidden_input_state(&storage.payloads, control, node)?.0;
        destination.extend_units(
            textarea_raw(&storage.payloads, control, raw)?.code_units(),
            control,
        )?;
        Ok(())
    }
    pub(crate) fn append_hidden_input_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.hidden_input_state(control, node)?;
        append_input_default(&self.active()?.records, control, node.index, destination)
    }

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
        if state.kind != FormAssociationKind::Image {
            return Err(SessionError::ImageFormMetadataUnavailable);
        }
        Ok(state)
    }
    pub(crate) fn output_state(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<OutputState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(control, node)?;
        find_output_state(&storage.payloads, control, node)
    }
    pub(crate) fn append_output_value(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.output_state(control, node)?;
        append_descendant_text(&self.active()?.records, control, node.index, destination)
    }
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
        if storage.html_local(control, node, "option")? {
            storage.update_for_option(control, node)?;
            return storage.drain_construction_effects(control);
        }
        if !storage.html_local(control, node, "textarea")? {
            return Ok(());
        }
        if storage.textarea_slot(control, node)?.is_none() {
            return Err(control
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        }
        match storage.refresh_textarea(control, node, None, true) {
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) => Err(control
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into()),
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
            let attribute = tag
                .attributes
                .get_mut(index, control)?
                .expect("actual attribute");
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
            let reads = u64::from(start != 0 && start != source.len())
                + u64::from(end != 0 && end != source.len());
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
            let attribute = tag
                .attributes
                .get_mut(index, control)?
                .expect("actual attribute");
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
        if let Some(parent) = intended_parent
            && let Err(error) = storage.prepare_creation_registry(
                control,
                parent.index,
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
        control.charge(WorkCategory::RecordVisit, 1)?;
        let units = value.len();
        control.charge(WorkCategory::RecordWrite, 2)?;
        let node = storage.allocate_prepared_owned(
            control,
            HtmlNodeKind::Comment(value),
            false,
            0,
            0,
            prepared,
        )?;
        storage.bind_string(ticket, StringLocation::Comment(node.index));
        storage.usage.utf16_units += units;
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
        control.charge(WorkCategory::RecordVisit, 2)?;
        let units = target.len().checked_add(data.len()).ok_or_else(|| {
            control
                .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                .unwrap_err()
        })?;
        control.charge(WorkCategory::RecordWrite, 3)?;
        // Recovered PI data is exact storage, not checked native authoring input.
        let node = storage.allocate_prepared_owned(
            control,
            HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction { target, data }),
            false,
            0,
            0,
            prepared,
        )?;
        storage.bind_string(
            target_ticket,
            StringLocation::ProcessingInstructionTarget(node.index),
        );
        storage.bind_string(
            data_ticket,
            StringLocation::ProcessingInstructionData(node.index),
        );
        storage.usage.utf16_units += units;
        storage.bind_pi_map(map, node.index);
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
            for index in 0..3 {
                payload_write::<Option<Text>>(control, 2)?;
                let value = if let Some(text) = incoming[index].take() {
                    storage.register_text(control, text)?
                } else {
                    let empty = literal_string(control, b"")?;
                    storage.register_string(control, empty, None)?
                };
                control.charge(WorkCategory::RecordVisit, 1)?;
                units = units.checked_add(value.0.len()).ok_or_else(|| {
                    control
                        .representation::<()>(HtmlParseResource::ManagedUtf16Units)
                        .unwrap_err()
                })?;
                payload_write::<Option<(DocumentString, usize)>>(control, 2)?;
                registered[index] = Some(value);
            }
            payload_write::<[Option<(DocumentString, usize)>; 3]>(control, 3)?;
            let (name, a) = registered[0].take().expect("registered name");
            let (public_id, b) = registered[1].take().expect("registered public identifier");
            let (system_id, d) = registered[2].take().expect("registered system identifier");
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
            storage.bind_string(a, StringLocation::DoctypeName(node.index));
            storage.bind_string(b, StringLocation::DoctypePublic(node.index));
            storage.bind_string(d, StringLocation::DoctypeSystem(node.index));
            storage.usage.utf16_units += units;
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
        if !matches!(storage.records[0].kind, HtmlNodeKind::Document(_)) {
            return Err(SessionError::NotDocument);
        }
        for position in 0..storage.records[0].children.len() {
            control.charge(WorkCategory::RecordVisit, 2)?;
            let child = storage.records[0].children[position];
            if matches!(
                storage.records[child].kind,
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
        let HtmlNodeKind::Document(mode) = &storage.records[0].kind else {
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
        let HtmlNodeKind::Document(current) = &mut storage.records[0].kind else {
            return Err(SessionError::NotDocument);
        };
        payload_write::<HtmlDocumentMode>(control, 1)?;
        *current = mode;
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
        if matches!(storage.records[0].kind, HtmlNodeKind::Fragment) {
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
        storage.records[index]
            .parent
            .map(|index| storage.node(control, index))
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
        Ok(match &storage.records[index].kind {
            HtmlNodeKind::Element(value) => Some(value.name()),
            _ => None,
        })
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
        let record = &storage.records[element];
        if !matches!(record.kind, HtmlNodeKind::Element(_)) {
            return Err(SessionError::WrongToken);
        }
        if record.parent.is_some() {
            return Ok(Some(ElementInsertionIgnored::ParentAlreadyPresent));
        }
        let mut ancestor = Some(target);
        for _ in 0..storage.records.len() {
            let Some(index) = ancestor else { break };
            control.charge(WorkCategory::RecordVisit, 1)?;
            if index == element {
                return Ok(Some(
                    ElementInsertionIgnored::HostIncludingInclusiveAncestor,
                ));
            }
            let record = storage
                .records
                .get(index)
                .ok_or(SessionError::ForeignNode)?;
            ancestor = if let Some(owner) = &record.owner {
                let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) =
                    owner;
                control.charge(WorkCategory::ProviderCall, 1)?;
                if !Arc::ptr_eq(&owner.identity, &storage.identity) {
                    return Err(SessionError::ForeignNode);
                }
                Some(owner.index)
            } else {
                record.parent
            };
        }
        if ancestor.is_some() {
            return Err(SessionError::Cycle);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        if matches!(storage.records[target].kind, HtmlNodeKind::Document(_)) {
            for position in 0..storage.records[target].children.len() {
                // One actual child-index read and one referenced kind read.
                control.charge(WorkCategory::RecordVisit, 2)?;
                let child = storage.records[target].children[position];
                let record = storage
                    .records
                    .get(child)
                    .ok_or(SessionError::ForeignNode)?;
                if matches!(record.kind, HtmlNodeKind::Element(_)) {
                    return Ok(Some(ElementInsertionIgnored::DocumentAlreadyHasElement));
                }
            }
        }
        Ok(None)
    }
    /// Only the actual token-created template path in this checkpoint proves
    /// its insertion target and end marker are initially null. Preexisting
    /// templates carry no such parser creation provenance.
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
    pub(crate) fn insert_parser(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        capability.validate_session(control, self)?;
        self.qualify_intrinsic_mutation(control, node, Some(capability))?;
        self.active_mut()?
            .insert(control, parent, node, before, InsertionCause::Parser)
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
        // Only a parser-initialized session carries this actual variable owner.
        // Prepay its active-to-frozen move before storage is taken; a stopped
        // share restores that same owner without useful recovery work.
        if self.active()?.evidence.is_some() {
            payload_write::<Option<ParseEvidence>>(control, 2)?;
        }
        let SessionStorage {
            control: owner,
            effects,
            evidence,
            fragment,
            #[cfg(test)]
                cancel_after_removal: _,
            #[cfg(test)]
                cancel_after_textarea_reset: _,
            #[cfg(test)]
                cancel_after_output_reset: _,
            #[cfg(test)]
                cancel_after_hidden_reset: _,
            standalone_ineligibility,
            identity,
            identity_allocation,
            records,
            records_allocation,
            children_allocations,
            tickets_allocation,
            usage,
            payloads,
        } = self
            .storage
            .take()
            .expect("successful preparation retains active storage");
        let arena = Arena {
            identity,
            records,
            usage,
        };
        let (arena, arena_allocation) = match control.share_owned(arena) {
            Ok(result) => result,
            Err((stop, arena)) => {
                self.storage = Some(SessionStorage {
                    control: owner,
                    effects,
                    evidence,
                    fragment,
                    #[cfg(test)]
                    cancel_after_removal: None,
                    #[cfg(test)]
                    cancel_after_textarea_reset: None,
                    #[cfg(test)]
                    cancel_after_output_reset: None,
                    #[cfg(test)]
                    cancel_after_hidden_reset: None,
                    standalone_ineligibility,
                    identity: arena.identity,
                    identity_allocation,
                    records: arena.records,
                    records_allocation,
                    children_allocations,
                    tickets_allocation,
                    usage: arena.usage,
                    payloads,
                });
                return Err(stop.into());
            }
        };
        let view = match fragment_root {
            None => FrozenView::Document(HtmlDocument { arena }),
            Some(root) => FrozenView::Fragment(HtmlFragment { arena, root }),
        };
        Ok(FrozenArena {
            storage: Some(FrozenStorage {
                view,
                control: owner,
                arena_allocation,
                identity_allocation,
                records_allocation,
                children_allocations,
                tickets_allocation,
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
        storage.payloads.check_aliases(&storage.records, control)?;
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
        self.active()?
            .payloads
            .check_aliases(&self.active()?.records, control)?;
        self.storage
            .take()
            .expect("verified active owner retains storage")
            .release(control);
        Ok(())
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
        let storage = self.storage.as_mut().ok_or(SessionError::Consumed)?;
        control.cleanup_release(&storage.control, HtmlParseResource::AuxiliaryEntries, 0)?;
        ParseEvidence::partition_failed(&mut storage.evidence, control)
    }
    pub(crate) fn hidden_input_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<HiddenInputState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        Ok(find_hidden_input_state(&storage.payloads, control, node.index)?.1)
    }
    /// Same conditional availability as the active owner, qualified to this arena.
    pub(crate) fn hidden_input_writeback_sources<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError>
    {
        self.hidden_input_state(control, node)?;
        find_hidden_input_sources(
            &self
                .storage
                .as_ref()
                .ok_or(SessionError::Consumed)?
                .payloads,
            control,
            node.index,
        )
    }
    pub(crate) fn append_hidden_input_internal_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.hidden_input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let raw = find_hidden_input_state(&storage.payloads, control, node.index)?.0;
        destination.extend_units(
            textarea_raw(&storage.payloads, control, raw)?.code_units(),
            control,
        )?;
        Ok(())
    }
    pub(crate) fn append_hidden_input_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.hidden_input_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        append_input_default(&arena.records, control, node.index, destination)
    }

    pub(crate) fn output_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<OutputState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        find_output_state(&storage.payloads, control, node.index)
    }
    pub(crate) fn append_output_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        destination.require_owner(control)?;
        self.output_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        append_descendant_text(&arena.records, control, node.index, destination)
    }
    pub(crate) fn append_output_default_value(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
        destination: &mut Utf16Buffer,
    ) -> Result<(), SessionError> {
        self.append_output_value(control, node, destination)
    }
    pub(crate) fn textarea_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<TextareaState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        find_textarea_state(&storage.payloads, control, node.index)?
            .map(|(_, state)| state)
            .ok_or(SessionError::TextareaMetadataUnavailable)
    }
    pub(crate) fn textarea_raw_value<'a>(
        &'a self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<&'a DocumentString, SessionError> {
        let state = self.textarea_state(control, node)?;
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        textarea_raw(&storage.payloads, control, state.raw)
    }
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
        if state.kind != FormAssociationKind::Image {
            return Err(SessionError::ImageFormMetadataUnavailable);
        }
        Ok(state)
    }
    pub(crate) fn image_form_owner(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let state = self.image_form_state(control, node)?;
        self.resolve_form_owner(control, state.owner)
    }
    pub(crate) fn form_association_state(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<FormAssociationState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if !Arc::ptr_eq(&node.identity, &arena.identity) || node.index >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        find_form_association_state(&storage.payloads, control, node.index)?
            .map(|(_, state)| state)
            .ok_or(SessionError::FormAssociationMetadataUnavailable)
    }
    pub(crate) fn form_association_owner(
        &self,
        control: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let state = self.form_association_state(control, node)?;
        self.resolve_form_owner(control, state.owner)
    }
    fn resolve_form_owner(
        &self,
        control: &mut ParseControl,
        owner: Option<usize>,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let Some(owner) = owner else {
            return Ok(None);
        };
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        control.charge(WorkCategory::RecordVisit, 1)?;
        if owner >= arena.records.len() {
            return Err(SessionError::ForeignNode);
        }
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<HtmlNodeId>() as u64)?;
        Ok(Some(HtmlNodeId {
            identity: arena.identity.clone(),
            index: owner,
        }))
    }
    pub(crate) fn script_metadata(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<ScriptMetadata, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        control.require_identity(&storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node.owner.as_ptr() != Arc::as_ptr(&arena.identity) || node.index >= arena.records.len()
        {
            return Err(SessionError::ForeignNode);
        }
        script_metadata(&storage.payloads, control, node.index).map(|(_, metadata)| metadata)
    }
    pub(crate) fn document(&self) -> Option<&HtmlDocument> {
        match &self.storage.as_ref()?.view {
            FrozenView::Document(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn fragment(&self) -> Option<&HtmlFragment> {
        match &self.storage.as_ref()?.view {
            FrozenView::Fragment(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        cleanup_owner(control, &storage.control)?;
        let arena = match &storage.view {
            FrozenView::Document(view) => &view.arena,
            FrozenView::Fragment(view) => &view.arena,
        };
        storage.payloads.check_aliases(&arena.records, control)?;
        let FrozenStorage {
            view,
            control: owner,
            arena_allocation,
            identity_allocation,
            records_allocation,
            children_allocations,
            tickets_allocation,
            payloads,
            effects,
            evidence,
            fragment,
        } = self
            .storage
            .take()
            .expect("verified frozen owner retains storage");
        let fragment_root = match &view {
            FrozenView::Document(_) => None,
            FrozenView::Fragment(view) => Some(view.root),
        };
        let arena = match view {
            FrozenView::Document(view) => view.arena,
            FrozenView::Fragment(view) => view.arena,
        };
        let arena = match control.release_shared(arena, arena_allocation) {
            Ok(arena) => arena,
            Err(error) => {
                let (stop, arena, arena_allocation) = error.into_parts();
                let view = match fragment_root {
                    None => FrozenView::Document(HtmlDocument { arena }),
                    Some(root) => FrozenView::Fragment(HtmlFragment { arena, root }),
                };
                self.storage = Some(FrozenStorage {
                    view,
                    control: owner,
                    arena_allocation,
                    identity_allocation,
                    records_allocation,
                    children_allocations,
                    tickets_allocation,
                    payloads,
                    effects,
                    evidence,
                    fragment,
                });
                return Err(stop.into());
            }
        };
        if let Some(fragment) = fragment {
            fragment
                .release(control, &owner)
                .expect("actual context effects owner");
        }
        if let Some(evidence) = evidence {
            evidence.release(control)?;
        }
        effects.release(control)?;
        release_parts(
            control,
            &owner,
            ReleasedStorage {
                arena,
                records: records_allocation,
                children: children_allocations,
                tickets: tickets_allocation,
                identity: identity_allocation,
                payloads,
            },
        );
        Ok(())
    }
}

fn release_vector<T>(
    control: &mut ParseControl,
    vector: Vec<T>,
    allocation: VectorCapacity<T>,
) -> Result<(), SessionError> {
    control.release_vector(vector, allocation).map_err(|error| {
        let (stop, _, _) = error.into_parts();
        SessionError::Stop(stop)
    })
}

/// Disposal consumes the actual initialized scratch entries, independently of
/// their reserved capacity. A terminal stop permits only this ownership cleanup.
fn release_order(
    control: &mut ParseControl,
    owner: &ControlIdentity,
    order: Vec<usize>,
    allocation: VectorCapacity<usize>,
) -> Result<(), SessionError> {
    let entries = order.len() as u64;
    release_vector(control, order, allocation)?;
    if control.stop().is_none()
        && control
            .release(HtmlParseResource::AuxiliaryEntries, entries)
            .is_ok()
    {
        return Ok(());
    }
    control.cleanup_release(owner, HtmlParseResource::AuxiliaryEntries, entries)?;
    Ok(())
}
fn release_parts(control: &mut ParseControl, owner: &ControlIdentity, parts: ReleasedStorage) {
    let ReleasedStorage {
        mut arena,
        records: records_allocation,
        mut children,
        tickets: tickets_allocation,
        identity: identity_allocation,
        payloads,
    } = parts;
    let children_allocations = &mut children;
    for record in arena.records.iter_mut().rev() {
        let allocation = children_allocations
            .pop()
            .expect("each actual record owns a child-vector ticket");
        release_vector(control, mem::take(&mut record.children), allocation)
            .expect("private record retains its exact child-vector capacity ticket and control");
    }
    // Release drops association IDs before reclaiming the identity shared payload.
    release_vector(control, arena.records, records_allocation)
        .expect("private session retains its exact record-vector capacity ticket and control");
    release_vector(control, children, tickets_allocation)
        .expect("private session retains its exact ticket-vector capacity ticket and control");
    for (resource, units) in [
        (HtmlParseResource::ManagedEdges, arena.usage.edges as u64),
        (
            HtmlParseResource::ManagedAttributes,
            payloads.preexisting_attributes as u64,
        ),
        (
            HtmlParseResource::ManagedUtf16Units,
            payloads.preexisting_units as u64,
        ),
        (
            HtmlParseResource::AuxiliaryEntries,
            arena.usage.nodes as u64,
        ),
    ] {
        if control.stop().is_none() && control.release(resource, units).is_ok() {
            continue;
        }
        control
            .cleanup_release(owner, resource, units)
            .expect("private session's admitted logical storage has not been released elsewhere");
    }
    payloads.release(control, owner);
    // Identity has a zero-sized known payload. Public IDs may retain its actual
    // Arc header, which is explicitly excluded from measured payload capacity.
    // The controller checks the exact ticket; this makes no deallocation claim.
    if control
        .release_zero_sized_shared(arena.identity, identity_allocation)
        .is_err()
    {
        panic!("private session retains the exact zero-payload identity ticket and control");
    }
}

fn cleanup_owner(control: &mut ParseControl, owner: &ControlIdentity) -> Result<(), SessionError> {
    if control.stop().is_some() {
        control.cleanup_release(owner, HtmlParseResource::AuxiliaryEntries, 0)?;
    } else if let Err(stop) = control.require_identity(owner) {
        // A cancellation can latch during the normal identity checkpoint. The
        // terminal cleanup check still verifies ownership before any consumption.
        control
            .cleanup_release(owner, HtmlParseResource::AuxiliaryEntries, 0)
            .map_err(|_| SessionError::Stop(stop))?;
    }
    Ok(())
}

fn release_token_attributes(
    mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    let mut disposal_stop = None;
    loop {
        let attribute = if control.stop().is_some() {
            attributes.cleanup_pop(control)?
        } else {
            match attributes.try_pop(control) {
                Ok(attribute) => attribute,
                Err(stop) => {
                    // A stop first observed by this pop must not drop the only
                    // owner of lexical capacity or of the remaining fields.
                    // All incoming owners were checked before token consumption.
                    disposal_stop = Some(stop);
                    attributes.cleanup_pop(control)?
                }
            }
        };
        let Some(attribute) = attribute else { break };
        attribute.name.release(control)?;
        attribute.value.release(control)?;
    }
    attributes.release(control)?;
    disposal_stop.map_or(Ok(()), |stop| Err(SessionError::Stop(stop)))
}
fn payload_write<T>(control: &mut ParseControl, writes: u64) -> Result<(), SessionError> {
    control.admit_atomic_layout(mem::size_of::<T>())?;
    control.charge(WorkCategory::CopyByte, mem::size_of::<T>() as u64)?;
    control.charge(WorkCategory::RecordWrite, writes)?;
    Ok(())
}
fn take_payload<T>(input: &mut Option<T>, control: &mut ParseControl) -> Result<T, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    if input.is_none() {
        return Err(SessionError::Consumed);
    }
    payload_write::<Option<T>>(control, 2)?;
    Ok(input.take().expect("preflighted payload"))
}
fn literal_string(
    control: &mut ParseControl,
    literal: &'static [u8],
) -> Result<FrozenUtf16, SessionError> {
    let mut buffer = Utf16Buffer::new(control)?;
    let result = (|| {
        let mut index = 0;
        while index < literal.len() {
            control.charge(WorkCategory::InspectByte, 1)?;
            buffer.push_unit(u16::from(literal[index]), control)?;
            index += 1;
        }
        Ok::<_, ParseStop>(())
    })();
    if let Err(stop) = result {
        buffer.release(control)?;
        return Err(stop.into());
    }
    Ok(buffer.freeze(control)?)
}

fn check_formatting_text(
    control: &mut ParseControl,
    text: &Text,
    source: &crate::HtmlSourceRange,
) -> Result<(), SessionError> {
    control.require_document_string(text.value.allocation(), text.value.text())?;
    text.origins.require_disposal_owner(control)?;
    control.charge(WorkCategory::ProviderCall, 1)?;
    if Arc::strong_count(text.value.text().backing()) != 1 {
        return Err(SessionError::AliasedPayload);
    }
    control.charge(WorkCategory::RecordVisit, 1)?;
    let len = text.value.text().len();
    let mut covered = 0;
    for origin in text.origins.as_slice() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.require_source_range(&origin.source)?;
        if origin.start != covered
            || origin.end <= origin.start
            || origin.end > len
            || origin.source.start() < source.start()
            || origin.source.end() > source.end()
        {
            return Err(SessionError::OriginSplitUnavailable);
        }
        covered = origin.end;
    }
    if covered != len {
        return Err(SessionError::OriginSplitUnavailable);
    }
    Ok(())
}
fn check_formatting_tag(
    control: &mut ParseControl,
    tag: &Tag,
    source: &crate::HtmlSourceRange,
) -> Result<(), SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    if tag.kind != TagKind::Start {
        return Err(SessionError::WrongToken);
    }
    check_formatting_text(control, &tag.name, source)?;
    lowercase_token_name(control, tag.name.value.text())?;
    tag.attributes.require_disposal_owner(control)?;
    for (index, attribute) in tag.attributes.as_slice().iter().enumerate() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        check_formatting_text(control, &attribute.name, source)?;
        check_formatting_text(control, &attribute.value, source)?;
        lowercase_token_name(control, attribute.name.value.text())?;
        control.require_source_range(&attribute.source)?;
        if attribute.source.start() < source.start() || attribute.source.end() > source.end() {
            return Err(SessionError::OriginSplitUnavailable);
        }
        // Pair matching for Noah relies on actual unique parsed names, not the
        // token's duplicate-recovery flag or an untrusted counter substitute.
        for earlier in &tag.attributes.as_slice()[..index] {
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
    Ok(())
}

fn copy_formatting_text(control: &mut ParseControl, text: &Text) -> Result<Text, SessionError> {
    let mut value = Utf16Buffer::new(control)?;
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let copied = (|| {
        // The concrete buffer copies/polls bounded real slices; no scalar or
        // lossy conversion, lexical scan or implicit Arc alias is introduced.
        value.extend_units(text.value.text().code_units(), control)?;
        for origin in text.origins.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<TextOrigin>(control, 1)?;
            origins.try_push(
                control,
                TextOrigin {
                    start: origin.start,
                    end: origin.end,
                    source: origin.source.clone(),
                    kind: origin.kind,
                },
            )?;
        }
        payload_write::<Text>(control, 1)
    })();
    if let Err(error) = copied {
        value.release(control)?;
        origins.release(control)?;
        return Err(error);
    }
    match value.freeze(control) {
        Ok(value) => Ok(Text { value, origins }),
        Err(stop) => {
            origins.release(control)?;
            Err(stop.into())
        }
    }
}
fn copy_formatting_tag(control: &mut ParseControl, tag: &Tag) -> Result<Tag, SessionError> {
    let name = copy_formatting_text(control, &tag.name)?;
    let mut attributes = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
    let copied = (|| {
        for original in tag.attributes.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let name = copy_formatting_text(control, &original.name)?;
            let value = match copy_formatting_text(control, &original.value) {
                Ok(value) => value,
                Err(error) => {
                    name.release(control)?;
                    return Err(error);
                }
            };
            let admission = (|| {
                control.charge(WorkCategory::ProviderCall, 1)?;
                payload_write::<crate::parser::tokenizer::token::Attribute>(control, 1)
            })();
            if let Err(error) = admission {
                name.release(control)?;
                value.release(control)?;
                return Err(error);
            }
            let attribute = crate::parser::tokenizer::token::Attribute {
                name,
                value,
                source: original.source.clone(),
            };
            if let Err((stop, attribute)) = attributes.try_push_recover(control, attribute) {
                attribute.name.release(control)?;
                attribute.value.release(control)?;
                return Err(stop.into());
            }
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        payload_write::<Tag>(control, 1)
    })();
    if let Err(error) = copied {
        name.release(control)?;
        release_token_attributes(attributes, control)?;
        return Err(error);
    }
    Ok(Tag {
        kind: TagKind::Start,
        name,
        self_closing: tag.self_closing,
        attributes,
        had_duplicate_attributes: tag.had_duplicate_attributes,
    })
}

fn release_formatting_tag(
    input: &mut Option<Tag>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
    let Some(tag) = input.as_ref() else {
        return Ok(());
    };
    check_disposal_text(control, &tag.name)?;
    tag.attributes.require_disposal_owner(control)?;
    for attribute in tag.attributes.as_slice() {
        cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
        check_disposal_text(control, &attribute.name)?;
        check_disposal_text(control, &attribute.value)?;
    }
    if control.stop().is_none() {
        payload_write::<Option<Tag>>(control, 2)?;
    }
    let tag = input.take().expect("checked original formatting token");
    tag.name.release(control)?;
    release_token_attributes(tag.attributes, control)
}
fn discard_formatting_tag(
    input: &mut Option<Tag>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    match release_formatting_tag(input, control) {
        Err(_) if control.stop().is_some() && input.is_some() => {
            release_formatting_tag(input, control)
        }
        Err(_) if control.stop().is_some() && input.is_none() => Ok(()),
        result => result,
    }
}
fn check_text_source(control: &mut ParseControl, text: &Text) -> Result<(), SessionError> {
    for origin in text.origins.as_slice() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.require_source_range(&origin.source)?;
    }
    Ok(())
}
fn exact_string_equal(
    control: &mut ParseControl,
    left: &DocumentString,
    right: &DocumentString,
) -> Result<bool, SessionError> {
    control.charge(WorkCategory::RecordVisit, 2)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for index in 0..left.len() {
        control.charge(WorkCategory::InspectByte, 4)?;
        if left.code_units()[index] != right.code_units()[index] {
            return Ok(false);
        }
    }
    Ok(true)
}
fn check_disposal_text(control: &mut ParseControl, text: &Text) -> Result<(), SessionError> {
    control.require_document_string_for_disposal(text.value.allocation(), text.value.text())?;
    text.origins.require_disposal_owner(control)?;
    cleanup_charge(control, WorkCategory::ProviderCall, 1)?;
    if Arc::strong_count(text.value.text().backing()) != 1 {
        return Err(SessionError::AliasedPayload);
    }
    Ok(())
}
/// Consumes only a checked, uniquely owned lexical payload. Terminal checks
/// inspect ownership solely for disposal and never resume token processing.
fn check_token_kind(token: &TokenKind, control: &mut ParseControl) -> Result<(), SessionError> {
    match token {
        TokenKind::Text(text) | TokenKind::Comment(text) => check_disposal_text(control, text)?,
        TokenKind::ProcessingInstruction { target, data } => {
            check_disposal_text(control, target)?;
            check_disposal_text(control, data)?;
        }
        TokenKind::Doctype(value) => {
            for field in [&value.name, &value.public_id, &value.system_id] {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                if let Some(text) = field {
                    check_disposal_text(control, text)?;
                }
            }
        }
        TokenKind::Tag(value) => {
            check_disposal_text(control, &value.name)?;
            value.attributes.require_disposal_owner(control)?;
            for index in 0..value.attributes.len() {
                cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
                let attribute = &value.attributes.as_slice()[index];
                check_disposal_text(control, &attribute.name)?;
                check_disposal_text(control, &attribute.value)?;
            }
        }
        TokenKind::Eof => {
            if control.stop().is_none() {
                control.checkpoint()?;
            }
        }
    }
    Ok(())
}
/// Active borrowed admission of every owned lexical field, not a source-ID
/// substitute or permission to resume a terminal execution. EOF owns no fields.
pub(crate) fn validate_token_kind(
    token: &TokenKind,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    control.checkpoint()?;
    control.charge(WorkCategory::RecordVisit, 1)?;
    check_token_kind(token, control)
}
pub(crate) fn release_token_kind(
    input: &mut Option<TokenKind>,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
    let token = input.as_ref().ok_or(SessionError::Consumed)?;
    check_token_kind(token, control)?;
    if control.stop().is_none() {
        // Admit the finite destructive field inspection before consuming the
        // borrowed owner, so disposal cannot strand fields on a late work stop.
        let fields = match token {
            TokenKind::Text(_) | TokenKind::Comment(_) => 1,
            TokenKind::ProcessingInstruction { .. } | TokenKind::Tag(_) => 2,
            TokenKind::Doctype(_) => 3,
            TokenKind::Eof => 0,
        };
        control.charge(WorkCategory::RecordVisit, fields)?;
        payload_write::<Option<TokenKind>>(control, 2)?;
    }
    match input.take().expect("preflighted disposal payload") {
        TokenKind::Text(text) | TokenKind::Comment(text) => text.release(control)?,
        TokenKind::ProcessingInstruction { target, data } => {
            target.release(control)?;
            data.release(control)?;
        }
        TokenKind::Doctype(value) => {
            // The finite owned fields are being disposed, not parsed. Admission
            // above covers active metadata; after a stop cleanup is uncharged.
            for text in [value.name, value.public_id, value.system_id]
                .into_iter()
                .flatten()
            {
                text.release(control)?;
            }
        }
        TokenKind::Tag(value) => {
            value.name.release(control)?;
            release_token_attributes(value.attributes, control)?;
        }
        TokenKind::Eof => {}
    }
    Ok(())
}
/// Splits only a leading run of the five HTML ASCII whitespace code points.
/// The returned prefix and retained suffix have real owned backing and exact
/// origin ranges. No source-coordinate range is guessed from output offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BodyTextKind {
    Null,
    Whitespace,
    Other,
}
pub(crate) struct TextRun {
    pub(crate) kind: BodyTextKind,
    pub(crate) text: Text,
}
/// Actual lexical owners in original order. Consumed slots remain admitted
/// until collection disposal; no mutable vector escapes.
pub(crate) struct BodyTextRuns {
    entries: Option<OwnedVec<Option<TextRun>>>,
    next: usize,
}
impl BodyTextRuns {
    pub(crate) fn take_next(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<TextRun>, SessionError> {
        let entries = self.entries.as_mut().ok_or(SessionError::Consumed)?;
        entries.require_owner(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.next == entries.len() {
            return Ok(None);
        }
        payload_write::<usize>(control, 1)?;
        payload_write::<Option<TextRun>>(control, 1)?;
        let slot = entries
            .get_mut(self.next, control)?
            .expect("retained run slot");
        let value = slot.take().expect("next slot has not been transferred");
        self.next += 1;
        Ok(Some(value))
    }
    /// Preflight precedes disposal. Once it begins, newly latched work or
    /// cancellation stops cannot strand later nested owners.
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
        let Some(entries) = self.entries.as_ref() else {
            return Ok(());
        };
        entries.require_disposal_owner(control)?;
        for entry in entries.as_slice() {
            cleanup_charge(control, WorkCategory::RecordVisit, 1)?;
            if let Some(run) = entry {
                check_disposal_text(control, &run.text)?;
            }
        }
        if control.stop().is_none() {
            payload_write::<Option<OwnedVec<Option<TextRun>>>>(control, 1)?;
        }
        let mut entries = self.entries.take().expect("preflight retained entries");
        let mut failure = None;
        loop {
            let entry = if control.stop().is_some() {
                entries.cleanup_pop(control)?
            } else {
                match entries.try_pop(control) {
                    Ok(entry) => entry,
                    Err(stop) => {
                        failure = Some(stop);
                        entries.cleanup_pop(control)?
                    }
                }
            };
            let Some(entry) = entry else { break };
            if let Some(run) = entry {
                run.text.release(control)?;
            }
        }
        entries.release(control)?;
        match failure {
            Some(stop) => Err(stop.into()),
            None => Ok(()),
        }
    }
}
struct TextRunDraft {
    kind: BodyTextKind,
    value: Utf16Buffer,
    origins: OwnedVec<TextOrigin>,
}
impl TextRunDraft {
    fn new(kind: BodyTextKind, control: &mut ParseControl) -> Result<Self, SessionError> {
        Ok(Self {
            kind,
            value: Utf16Buffer::new(control)?,
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
        })
    }
    fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        self.value.release(control)?;
        self.origins.release(control)?;
        Ok(())
    }
    fn freeze(self, control: &mut ParseControl) -> Result<TextRun, SessionError> {
        let Self {
            kind,
            value,
            origins,
        } = self;
        if let Err(error) = payload_write::<TextRun>(control, 2) {
            value.release(control)?;
            origins.release(control)?;
            return Err(error);
        }
        match value.freeze(control) {
            Ok(value) => Ok(TextRun {
                kind,
                text: Text { value, origins },
            }),
            Err(stop) => {
                origins.release(control)?;
                Err(stop.into())
            }
        }
    }
}
#[derive(Clone, Copy)]
enum TextPartition {
    Body,
    FirstLf,
}
fn body_text_kind(unit: u16) -> BodyTextKind {
    match unit {
        0 => BodyTextKind::Null,
        9 | 10 | 12 | 13 | 32 => BodyTextKind::Whitespace,
        _ => BodyTextKind::Other,
    }
}
fn discard_runs(runs: &mut BodyTextRuns, control: &mut ParseControl) -> Result<(), SessionError> {
    match runs.release(control) {
        Err(_) if control.stop().is_some() && runs.entries.is_some() => runs.release(control),
        Err(_) if control.stop().is_some() && runs.entries.is_none() => Ok(()),
        result => result,
    }
}
fn push_text_run(
    entries: &mut OwnedVec<Option<TextRun>>,
    draft: TextRunDraft,
    control: &mut ParseControl,
) -> Result<(), SessionError> {
    let run = draft.freeze(control)?;
    if let Err((stop, run)) = entries.try_push_recover(control, Some(run)) {
        run.expect("submitted run").text.release(control)?;
        return Err(stop.into());
    }
    Ok(())
}
/// One origin traversal and one classification read per code unit. Every exact
/// origin is copied to one admitted owner, never repeatedly to suffixes.
/// An origin spanning a requested cut is refused rather than guessed/sliced.
fn stage_text_runs(
    text: &Text,
    partition: TextPartition,
    control: &mut ParseControl,
) -> Result<BodyTextRuns, SessionError> {
    let mut runs = BodyTextRuns {
        entries: Some(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?),
        next: 0,
    };
    let mut draft: Option<TextRunDraft> = None;
    let result = (|| {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let len = text.value.text().len();
        let mut covered = 0;
        for origin in text.origins.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.require_source_range(&origin.source)?;
            if origin.start != covered || origin.end <= origin.start || origin.end > len {
                return Err(SessionError::OriginSplitUnavailable);
            }
            let mut kind = None;
            for index in origin.start..origin.end {
                control.charge(WorkCategory::InspectByte, 2)?;
                let unit = text.value.text().code_units()[index];
                let current = match partition {
                    TextPartition::Body => body_text_kind(unit),
                    TextPartition::FirstLf if index == 0 && unit == 10 => BodyTextKind::Whitespace,
                    TextPartition::FirstLf => BodyTextKind::Other,
                };
                if kind.is_some_and(|old| old != current) {
                    return Err(SessionError::OriginSplitUnavailable);
                }
                kind = Some(current);
            }
            let kind = kind.expect("nonempty checked origin");
            if draft.as_ref().is_some_and(|draft| draft.kind != kind) {
                push_text_run(
                    runs.entries.as_mut().expect("staging entries"),
                    draft.take().expect("staged draft"),
                    control,
                )?;
            }
            if draft.is_none() {
                draft = Some(TextRunDraft::new(kind, control)?);
            }
            let current = draft.as_mut().expect("current origin output");
            let start = current.value.len();
            current.value.extend_units(
                &text.value.text().code_units()[origin.start..origin.end],
                control,
            )?;
            control.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<crate::HtmlSourceRange>(control, 1)?;
            current.origins.try_push(
                control,
                TextOrigin {
                    start,
                    end: current.value.len(),
                    source: origin.source.clone(),
                    kind: origin.kind,
                },
            )?;
            covered = origin.end;
        }
        if covered != len {
            return Err(SessionError::OriginSplitUnavailable);
        }
        if let Some(draft) = draft.take() {
            push_text_run(
                runs.entries.as_mut().expect("staging entries"),
                draft,
                control,
            )?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        if let Some(draft) = draft.take() {
            draft.release(control)?;
        }
        discard_runs(&mut runs, control)?;
        return Err(error);
    }
    Ok(runs)
}
pub(crate) fn partition_body_text(
    control: &mut ParseControl,
    input: &mut Option<Text>,
) -> Result<BodyTextRuns, SessionError> {
    let text = input.as_mut().ok_or(SessionError::Consumed)?;
    SessionStorage::check_text(control, text)?;
    let mut runs = stage_text_runs(text, TextPartition::Body, control)?;
    let original = match take_payload(input, control) {
        Ok(original) => original,
        Err(error) => {
            discard_runs(&mut runs, control)?;
            return Err(error);
        }
    };
    original.release(control)?;
    Ok(runs)
}
/// Source-required first LF omission, retaining its complete original origin
/// (including CRLF preprocessing) and every later unit in the suffix.
pub(crate) fn split_first_lf(
    control: &mut ParseControl,
    input: &mut Option<Text>,
) -> Result<Option<Text>, SessionError> {
    let text = input.as_mut().ok_or(SessionError::Consumed)?;
    SessionStorage::check_text(control, text)?;
    control.charge(WorkCategory::RecordVisit, 1)?;
    if text.value.text().is_empty() {
        return Ok(None);
    }
    control.charge(WorkCategory::InspectByte, 2)?;
    if text.value.text().code_units()[0] != 10 {
        return Ok(None);
    }
    if text.value.text().len() == 1 {
        check_text_source(control, text)?;
        return Ok(Some(take_payload(input, control)?));
    }
    let mut runs = stage_text_runs(text, TextPartition::FirstLf, control)?;
    let prefix = match runs.take_next(control) {
        Ok(Some(run)) => run.text,
        Ok(None) => unreachable!("nonempty LF prefix"),
        Err(error) => {
            discard_runs(&mut runs, control)?;
            return Err(error);
        }
    };
    let suffix = match runs.take_next(control) {
        Ok(Some(run)) => run.text,
        Ok(None) => unreachable!("nonempty suffix"),
        Err(error) => {
            prefix.release(control)?;
            discard_runs(&mut runs, control)?;
            return Err(error);
        }
    };
    if let Err(error) = payload_write::<Option<Text>>(control, 2) {
        prefix.release(control)?;
        suffix.release(control)?;
        discard_runs(&mut runs, control)?;
        return Err(error);
    }
    // Scratch disposal is still fallible while input remains in its caller.
    if let Err(error) = runs.release(control) {
        prefix.release(control)?;
        suffix.release(control)?;
        discard_runs(&mut runs, control)?;
        return Err(error);
    }
    let original = input.replace(suffix).expect("borrowed original retained");
    original.release(control)?;
    Ok(Some(prefix))
}

pub(crate) fn split_leading_ascii_whitespace(
    control: &mut ParseControl,
    input: &mut Option<Text>,
) -> Result<Option<Text>, SessionError> {
    let text = input.as_mut().ok_or(SessionError::Consumed)?;
    SessionStorage::check_text(control, text)?;
    let mut cut = 0;
    for index in 0..text.value.text().len() {
        control.charge(WorkCategory::InspectByte, 2)?;
        if !matches!(
            text.value.text().code_units()[index],
            0x09 | 0x0a | 0x0c | 0x0d | 0x20
        ) {
            break;
        }
        cut += 1;
    }
    if cut == 0 {
        return Ok(None);
    }
    if cut == text.value.text().len() {
        return Ok(Some(take_payload(input, control)?));
    }
    let mut covered = 0;
    for origin in text.origins.as_slice() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if origin.start != covered
            || origin.end <= origin.start
            || origin.end > text.value.text().len()
        {
            return Err(SessionError::OriginSplitUnavailable);
        }
        if origin.start < cut && cut < origin.end {
            return Err(SessionError::OriginSplitUnavailable);
        }
        covered = origin.end;
    }
    if covered != text.value.text().len() {
        return Err(SessionError::OriginSplitUnavailable);
    }
    let prefix = slice_text(text, 0, cut, control)?;
    let suffix = match slice_text(text, cut, text.value.text().len(), control) {
        Ok(text) => text,
        Err(error) => {
            prefix.release(control)?;
            return Err(error);
        }
    };
    if let Err(error) = payload_write::<Option<Text>>(control, 2) {
        prefix.release(control)?;
        suffix.release(control)?;
        return Err(error);
    }
    let original = input
        .replace(suffix)
        .expect("original input retained through all fallible work");
    original.release(control)?;
    Ok(Some(prefix))
}
fn slice_text(
    text: &Text,
    start: usize,
    end: usize,
    control: &mut ParseControl,
) -> Result<Text, SessionError> {
    let mut buffer = Utf16Buffer::new(control)?;
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let result = (|| {
        buffer.extend_units(&text.value.text().code_units()[start..end], control)?;
        for origin in text.origins.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            if origin.start < start || origin.end > end {
                continue;
            }
            control.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<crate::HtmlSourceRange>(control, 1)?;
            origins.try_push(
                control,
                TextOrigin {
                    start: origin.start - start,
                    end: origin.end - start,
                    source: origin.source.clone(),
                    kind: origin.kind,
                },
            )?;
        }
        payload_write::<Text>(control, 2)?;
        Ok::<_, SessionError>(())
    })();
    if let Err(error) = result {
        buffer.release(control)?;
        origins.release(control)?;
        return Err(error);
    }
    let value = match buffer.freeze(control) {
        Ok(value) => value,
        Err(stop) => {
            origins.release(control)?;
            return Err(stop.into());
        }
    };
    Ok(Text { value, origins })
}

pub(crate) fn hidden_input_tag(
    control: &mut ParseControl,
    tag: &Tag,
) -> Result<bool, SessionError> {
    control.require_document_string(tag.name.value.allocation(), tag.name.value.text())?;
    tag.attributes.require_disposal_owner(control)?;
    for attribute in tag.attributes.as_slice() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if exact_ascii(control, Some(attribute.name.value.text()), "type")? {
            let value = attribute.value.value.text();
            control.require_document_string(attribute.value.value.allocation(), value)?;
            return script_match(control, b"", value, 0, value.len(), b"hidden");
        }
    }
    Ok(false)
}
fn validate_initial_input_origins(
    control: &mut ParseControl,
    tag: &Tag,
) -> Result<(), SessionError> {
    let mut hidden = false;
    for attribute in tag.attributes.as_slice() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if exact_ascii(control, Some(attribute.name.value.text()), "type")? {
            hidden = true;
        }
        if !exact_ascii(control, Some(attribute.name.value.text()), "value")? {
            continue;
        }
        let text = &attribute.value;
        let mut covered = 0;
        for origin in text.origins.as_slice() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            control.require_source_range(&origin.source)?;
            if origin.start != covered
                || origin.end <= origin.start
                || origin.end > text.value.text().len()
            {
                return Err(SessionError::OriginSplitUnavailable);
            }
            if !hidden {
                let mut omitted = 0;
                for position in origin.start..origin.end {
                    control.charge(WorkCategory::InspectByte, 2)?;
                    if matches!(text.value.text().code_units()[position], 0x0d | 0x0a) {
                        omitted += 1;
                    }
                }
                if omitted != 0 && omitted != origin.end - origin.start {
                    return Err(SessionError::OriginSplitUnavailable);
                }
            }
            covered = origin.end;
        }
        if covered != text.value.text().len() {
            return Err(SessionError::OriginSplitUnavailable);
        }
    }
    Ok(())
}
fn find_hidden_input_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<(usize, HiddenInputState), SessionError> {
    for payload in &payloads.values {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::HiddenInput {
            node: Some(owner),
            raw,
            complete: true,
            state,
            ..
        } = payload
            && *owner == node
        {
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<usize>() + mem::size_of::<HiddenInputState>()) as u64,
            )?;
            return Ok((*raw, *state));
        }
    }
    Err(SessionError::HiddenInputMetadataUnavailable)
}
fn find_hidden_input_sources<'a>(
    payloads: &'a PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<Option<(&'a crate::HtmlSourceRange, &'a crate::HtmlSourceRange)>, SessionError> {
    let (_, state) = find_hidden_input_state(payloads, control, node)?;
    if !state.value_rewritten_during_creation {
        return Ok(None);
    }
    for payload in &payloads.values {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::HiddenInputSource {
            node: Some(owner),
            token,
            value_attribute: Some(value),
        } = payload
            && *owner == node
        {
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<(&crate::HtmlSourceRange, &crate::HtmlSourceRange)>() as u64,
            )?;
            return Ok(Some((token, value)));
        }
    }
    Err(SessionError::HiddenInputMetadataUnavailable)
}
fn append_input_default(
    records: &[Record],
    control: &mut ParseControl,
    node: usize,
    destination: &mut Utf16Buffer,
) -> Result<(), SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let HtmlNodeKind::Element(element) = &records[node].kind else {
        return Err(SessionError::HiddenInputMetadataUnavailable);
    };
    for attribute in element.attributes().as_slice() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if attribute.name().namespace().uri().is_none()
            && exact_ascii(control, Some(attribute.name().local_name()), "value")?
        {
            destination.extend_units(attribute.value().code_units(), control)?;
            break;
        }
    }
    Ok(())
}
/// Copy actual surviving origin groups. A mixed aggregate cannot be split by
/// guessing source offsets; per-codepoint tokenizer origins establish the cut.
fn copy_input_value(
    payloads: &PayloadStorage,
    evidence: Option<&ParseEvidence>,
    control: &mut ParseControl,
    slot: usize,
    sanitize: bool,
) -> Result<FieldCopy, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let RetainedPayload::String {
        value,
        origins,
        allocation,
        ..
    } = &payloads.values[slot]
    else {
        return Err(SessionError::HiddenInputMetadataUnavailable);
    };
    control.require_document_string(allocation, value)?;
    let mut buffer = Utf16Buffer::new(control)?;
    let mut copied = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
    let result = (|| {
        let mut covered = 0;
        if let Some(origins) = origins {
            origins.require_disposal_owner(control)?;
            let evidence = evidence.ok_or(SessionError::WrongToken)?;
            let mut cursor = provenance::GroupOriginCursor::new(evidence, origins, control)?;
            let read = (|| {
                while let Some(origin) = cursor.next(evidence, control)? {
                    control.charge(WorkCategory::RecordVisit, 1)?;
                    control.require_source_range(&origin.source)?;
                    if origin.start != covered
                        || origin.end <= origin.start
                        || origin.end > value.len()
                    {
                        return Err(SessionError::OriginSplitUnavailable);
                    }
                    let mut omitted = 0;
                    for position in origin.start..origin.end {
                        control.charge(WorkCategory::InspectByte, 2)?;
                        if sanitize && matches!(value.code_units()[position], 0x0d | 0x0a) {
                            omitted += 1;
                        }
                    }
                    if omitted != 0 && omitted != origin.end - origin.start {
                        return Err(SessionError::OriginSplitUnavailable);
                    }
                    if omitted == 0 {
                        let start = buffer.len();
                        buffer
                            .extend_units(&value.code_units()[origin.start..origin.end], control)?;
                        control.charge(WorkCategory::ProviderCall, 1)?;
                        payload_write::<TextOrigin>(control, 1)?;
                        copied.try_push(
                            control,
                            TextOrigin {
                                start,
                                end: buffer.len(),
                                source: origin.source.clone(),
                                kind: origin.kind,
                            },
                        )?;
                    }
                    covered = origin.end;
                }
                Ok::<_, SessionError>(())
            })();
            cursor.release(control)?;
            read?;
        }
        if covered != value.len() {
            return Err(SessionError::OriginSplitUnavailable);
        }
        payload_write::<Text>(control, 1)?;
        Ok::<_, SessionError>(())
    })();
    if let Err(error) = result {
        buffer.release(control)?;
        copied.release(control)?;
        return Err(error);
    }
    let original = match origins
        .as_ref()
        .map(|group| {
            evidence
                .ok_or(SessionError::WrongToken)?
                .copy_group_reference(control, group)
        })
        .transpose()
    {
        Ok(original) => original,
        Err(error) => {
            buffer.release(control)?;
            copied.release(control)?;
            return Err(error);
        }
    };
    match buffer.freeze(control) {
        Ok(value) => Ok(FieldCopy {
            value,
            origins: copied,
            original,
        }),
        Err(stop) => {
            copied.release(control)?;
            Err(stop.into())
        }
    }
}

fn lowercase_token_name(
    control: &mut ParseControl,
    name: &DocumentString,
) -> Result<(), SessionError> {
    for index in 0..name.len() {
        control.charge(WorkCategory::InspectByte, 2)?;
        if matches!(name.code_units()[index], 0x41..=0x5a) {
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
    for (index, byte) in expected.bytes().enumerate() {
        control.charge(WorkCategory::InspectByte, 3)?;
        if value.code_units()[index] != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}
// Shared exact physical traversal used by form views and output reads.
fn ordinary_view_parent(
    records: &[Record],
    control: &mut ParseControl,
    node: usize,
    detached: Option<usize>,
) -> Result<Option<usize>, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let record = records.get(node).ok_or(SessionError::ForeignNode)?;
    Ok(if detached == Some(node) {
        None
    } else {
        record.parent
    })
}
fn next_ordinary_view_node(
    records: &[Record],
    control: &mut ParseControl,
    root: usize,
    node: usize,
    detached: Option<usize>,
) -> Result<Option<usize>, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let record = records.get(node).ok_or(SessionError::ForeignNode)?;
    for position in 0..record.children.len() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let child = record.children[position];
        if detached != Some(child) || root == child {
            return Ok(Some(child));
        }
    }
    next_ordinary_view_after_subtree(records, control, root, node, detached)
}
fn next_ordinary_view_after_subtree(
    records: &[Record],
    control: &mut ParseControl,
    root: usize,
    mut node: usize,
    detached: Option<usize>,
) -> Result<Option<usize>, SessionError> {
    for _ in 0..records.len() {
        if node == root {
            return Ok(None);
        }
        let parent = ordinary_view_parent(records, control, node, detached)?
            .ok_or(SessionError::WrongSibling)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let children = &records
            .get(parent)
            .ok_or(SessionError::ForeignNode)?
            .children;
        let mut found = false;
        let mut position = 0;
        while position < children.len() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let child = children[position];
            if found && (detached != Some(child) || root == child) {
                return Ok(Some(child));
            }
            found |= child == node;
            position += 1;
        }
        if !found {
            return Err(SessionError::WrongSibling);
        }
        node = parent;
    }
    Err(SessionError::Cycle)
}
fn find_output_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<OutputState, SessionError> {
    for slot in 0..payloads.values.len() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::Output {
            node: Some(owner),
            state,
        } = &payloads.values[slot]
            && *owner == node
        {
            control.charge(WorkCategory::CopyByte, mem::size_of::<OutputState>() as u64)?;
            return Ok(*state);
        }
    }
    Err(SessionError::OutputMetadataUnavailable)
}
fn append_descendant_text(
    records: &[Record],
    control: &mut ParseControl,
    root: usize,
    destination: &mut Utf16Buffer,
) -> Result<(), SessionError> {
    let mut next = next_ordinary_view_node(records, control, root, root, None)?;
    while let Some(node) = next {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let HtmlNodeKind::Text(value) = &records.get(node).ok_or(SessionError::ForeignNode)?.kind
        {
            destination.extend_units(value.code_units(), control)?;
        }
        next = next_ordinary_view_node(records, control, root, node, None)?;
    }
    Ok(())
}
fn find_textarea_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<Option<(usize, TextareaState)>, SessionError> {
    for slot in 0..payloads.values.len() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::Textarea {
            node: Some(owner),
            state,
        } = &payloads.values[slot]
            && *owner == node
        {
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<TextareaState>() as u64,
            )?;
            return Ok(Some((slot, *state)));
        }
    }
    Ok(None)
}

fn textarea_raw<'a>(
    payloads: &'a PayloadStorage,
    control: &mut ParseControl,
    raw: usize,
) -> Result<&'a DocumentString, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value,
        allocation,
        location: StringLocation::Detached,
        ..
    }) = payloads.values.get(raw)
    else {
        return Err(SessionError::TextareaMetadataUnavailable);
    };
    control.require_document_string(allocation, value)?;
    Ok(value)
}

fn find_form_association_state(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<Option<(usize, FormAssociationState)>, SessionError> {
    for slot in 0..payloads.values.len() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::FormAssociation { node: owner, state } = &payloads.values[slot]
            && *owner == Some(node)
        {
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<FormAssociationState>() as u64,
            )?;
            return Ok(Some((slot, *state)));
        }
    }
    Ok(None)
}
fn set_form_association_state(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    slot: usize,
    state: FormAssociationState,
) -> Result<(), SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    payload_write::<FormAssociationState>(control, 1)?;
    let Some(RetainedPayload::FormAssociation { state: current, .. }) =
        storage.payloads.values.get_mut(slot)
    else {
        return Err(SessionError::ImageFormMetadataUnavailable);
    };
    *current = state;
    Ok(())
}
fn script_metadata(
    payloads: &PayloadStorage,
    control: &mut ParseControl,
    node: usize,
) -> Result<(usize, ScriptMetadata), SessionError> {
    for position in 0..payloads.values.len() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::Script {
            node: owner,
            metadata,
        } = &payloads.values[position]
            && *owner == Some(node)
        {
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<ScriptMetadata>() as u64,
            )?;
            return Ok((position, *metadata));
        }
    }
    Err(SessionError::ScriptMetadataUnavailable)
}
fn set_script_metadata(
    storage: &mut SessionStorage,
    control: &mut ParseControl,
    slot: usize,
    value: ScriptMetadata,
) -> Result<(), SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    payload_write::<ScriptMetadata>(control, 1)?;
    let RetainedPayload::Script { metadata, .. } = storage
        .payloads
        .values
        .get_mut(slot)
        .ok_or(SessionError::ScriptMetadataUnavailable)?
    else {
        return Err(SessionError::ScriptMetadataUnavailable);
    };
    *metadata = value;
    Ok(())
}
fn script_attribute<'a>(
    control: &mut ParseControl,
    element: &'a HtmlElement,
    local: &str,
) -> Result<Option<&'a DocumentString>, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    for position in 0..element.attributes().len() {
        control.charge(WorkCategory::RecordVisit, 3)?;
        let attribute = &element.attributes().as_slice()[position];
        if attribute.name().namespace().uri().is_none()
            && exact_ascii(control, Some(attribute.name().local_name()), local)?
        {
            return Ok(Some(attribute.value()));
        }
    }
    Ok(None)
}
fn script_type_charge(
    control: &mut ParseControl,
    work: crate::script_type::ScriptTypeWork,
) -> Result<(), SessionError> {
    match work {
        crate::script_type::ScriptTypeWork::RecordVisit(units) => {
            control.charge(WorkCategory::RecordVisit, units)?
        }
        crate::script_type::ScriptTypeWork::InspectUnit(units) => {
            control.charge(WorkCategory::InspectByte, units)?
        }
    }
    Ok(())
}
// Actual hidden-input comparison and classification share the one borrowed
// literal matcher. The owning ParseControl admits every step before unit reads.
fn script_match(
    control: &mut ParseControl,
    prefix: &'static [u8],
    value: &DocumentString,
    start: usize,
    end: usize,
    expected: &'static [u8],
) -> Result<bool, SessionError> {
    let mut cursor =
        crate::script_type::ScriptTypeMatchCursor::new(prefix, value, start, end, expected);
    while let Some(work) = cursor.required_work() {
        script_type_charge(control, work)?;
        cursor.advance();
    }
    Ok(cursor
        .result()
        .expect("all admitted comparison actions completed"))
}
fn classify_script(
    control: &mut ParseControl,
    element: &HtmlElement,
) -> Result<Option<ScriptType>, SessionError> {
    let explicit = script_attribute(control, element, "type")?;
    let language = script_attribute(control, element, "language")?;
    let mut cursor = crate::script_type::ScriptTypeCursor::preparation(explicit, language);
    while let Some(work) = cursor.required_work() {
        script_type_charge(control, work)?;
        cursor.advance();
    }
    // Current-content classification is distinct from this existing saved
    // preparation-metadata type. No metadata field or effect is installed here.
    Ok(cursor
        .result()
        .expect("all admitted classification actions completed")
        .map(|kind| match kind {
            crate::HtmlScriptKind::Classic => ScriptType::Classic,
            crate::HtmlScriptKind::Module => ScriptType::Module,
            crate::HtmlScriptKind::ImportMap => ScriptType::ImportMap,
            crate::HtmlScriptKind::SpeculationRules => ScriptType::SpeculationRules,
        }))
}
fn name_units(control: &mut ParseControl, name: &QualifiedName) -> Result<usize, SessionError> {
    control.charge(WorkCategory::RecordVisit, 4)?;
    let units = name
        .local_name()
        .len()
        .checked_add(name.namespace().uri().map_or(0, DocumentString::len))
        .and_then(|sum| sum.checked_add(name.prefix().map_or(0, DocumentString::len)));
    match units {
        Some(units) => Ok(units),
        None => control
            .representation(HtmlParseResource::ManagedUtf16Units)
            .map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_owner<'a>(
        source: &'a crate::HtmlSource,
        caps: HtmlParseLimits,
    ) -> (
        ParseControl,
        ParserArenaSession,
        ParserNode,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
    ) {
        output_owner_cancellable(source, caps, HtmlParseCancellation::new())
    }

    fn output_owner_cancellable<'a>(
        source: &'a crate::HtmlSource,
        caps: HtmlParseLimits,
        cancellation: HtmlParseCancellation,
    ) -> (
        ParseControl,
        ParserArenaSession,
        ParserNode,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let mut c = ParseControl::new(caps, cancellation);
        c.admit_source(source).unwrap();
        let mut tokenizer = LexerFixture::new(source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut tag = Some(next_fixture_tag(
            &mut tokenizer,
            &mut c,
            &[111, 117, 116, 112, 117, 116],
        ));
        let output = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut tag,
                &root,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        assert!(tag.is_none());
        session.insert(&mut c, &root, &output, None, None).unwrap();
        (c, session, output, tokenizer)
    }

    #[test]
    fn output_reads_actual_descendants_after_merge_move_and_freeze_without_raw_cache() {
        let source = crate::HtmlSource::utf8(
            "<output>A<span>B&amp;C</span>D<!--ignored--><?ok ignored?><span>E</span>F",
        );
        let (mut c, mut session, output, mut lexer) = output_owner(&source, limits());
        let mut a = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &output, None, &mut a, None)
            .unwrap();
        let mut tag = Some(next_fixture_tag(&mut lexer, &mut c, &[115, 112, 97, 110]));
        let span = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut tag,
                &output,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &output, &span, None, None).unwrap();
        // Data emits one code point per Text token, including the reference.
        for _ in 0..3 {
            let mut b = next_text(&mut lexer, &mut c);
            session
                .insert_text(&mut c, &span, None, &mut b, None)
                .unwrap();
        }
        release_token_kind(&mut Some(lexical_kind(&mut lexer, &mut c)), &mut c).unwrap();
        let mut d = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &output, None, &mut d, None)
            .unwrap();
        let TokenKind::Comment(comment) = lexical_kind(&mut lexer, &mut c) else {
            panic!("real comment")
        };
        let comment = session
            .allocate_comment(&mut c, &mut Some(comment), None)
            .unwrap();
        session
            .insert(&mut c, &output, &comment, None, None)
            .unwrap();
        let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c)
        else {
            panic!("real PI")
        };
        let pi = session
            .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
            .unwrap();
        session.insert(&mut c, &output, &pi, None, None).unwrap();
        let mut tag = Some(next_fixture_tag(&mut lexer, &mut c, &[115, 112, 97, 110]));
        let last = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut tag,
                &output,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &output, &last, None, None).unwrap();
        let mut e = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &last, None, &mut e, None)
            .unwrap();
        release_token_kind(&mut Some(lexical_kind(&mut lexer, &mut c)), &mut c).unwrap();
        let mut f = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &last, None, &mut f, None)
            .unwrap();
        lexer.release(&mut c).unwrap();

        assert_eq!(
            session
                .output_state(&mut c, &output)
                .unwrap()
                .default_value_override,
            OutputDefaultValueOverride::Null
        );
        let mut current = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_output_value(&mut c, &output, &mut current)
            .unwrap();
        assert_eq!(current.units(), [65, 66, 38, 67, 68, 69, 70]);
        current.clear(&mut c).unwrap();
        session
            .append_output_default_value(&mut c, &output, &mut current)
            .unwrap();
        assert_eq!(current.units(), [65, 66, 38, 67, 68, 69, 70]);
        session.remove(&mut c, &span, None).unwrap();
        current.clear(&mut c).unwrap();
        session
            .append_output_value(&mut c, &output, &mut current)
            .unwrap();
        assert_eq!(current.units(), [65, 68, 69, 70]);
        session.insert(&mut c, &last, &span, None, None).unwrap();
        current.clear(&mut c).unwrap();
        session
            .append_output_value(&mut c, &output, &mut current)
            .unwrap();
        assert_eq!(current.units(), [65, 68, 69, 70, 66, 38, 67]);
        current.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let id = frozen.document().unwrap().root().children().next().unwrap();
        let mut value = Utf16Buffer::new(&mut c).unwrap();
        frozen.append_output_value(&mut c, &id, &mut value).unwrap();
        assert_eq!(value.units(), [65, 68, 69, 70, 66, 38, 67]);
        value.clear(&mut c).unwrap();
        frozen
            .append_output_default_value(&mut c, &id, &mut value)
            .unwrap();
        assert_eq!(value.units(), [65, 68, 69, 70, 66, 38, 67]);
        assert_eq!(
            frozen
                .output_state(&mut c, &id)
                .unwrap()
                .default_value_override,
            OutputDefaultValueOverride::Null
        );
        value.release(&mut c).unwrap();
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn output_observation_exact_40_unit_cap_or_sticky_20_unit_owned_prefix() {
        for cap in [39, 40] {
            let literal = format!(
                "<output><span>{}</span><span>{}</span>",
                "A".repeat(20),
                "B".repeat(20)
            );
            let source = crate::HtmlSource::utf8(literal);
            let (mut c, mut session, output, mut lexer) = output_owner(
                &source,
                limits()
                    .into_builder()
                    .buffered_utf16_units(cap)
                    // Forty genuine insertion batches retain their merge and
                    // retirement histories; this test still isolates the exact
                    // forty-unit observation buffer, not a work-credit cap.
                    .work_credits(20_000_000)
                    .finish()
                    .unwrap(),
            );
            for _ in 0..2 {
                let mut tag = Some(next_fixture_tag(&mut lexer, &mut c, &[115, 112, 97, 110]));
                let span = session
                    .allocate_html_tag_for_creation(
                        &mut c,
                        &mut tag,
                        &output,
                        None,
                        crate::construction::parser_session::CreationContext::Ordinary,
                    )
                    .unwrap();
                session.insert(&mut c, &output, &span, None, None).unwrap();
                for _ in 0..20 {
                    let mut text = next_text(&mut lexer, &mut c);
                    session
                        .insert_text(&mut c, &span, None, &mut text, None)
                        .unwrap();
                }
                release_token_kind(&mut Some(lexical_kind(&mut lexer, &mut c)), &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            let mut buffer = Utf16Buffer::new(&mut c).unwrap();
            let result = session.append_output_value(&mut c, &output, &mut buffer);
            if cap == 40 {
                result.unwrap();
                assert_eq!(buffer.units(), [&[65; 20][..], &[66; 20][..]].concat());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    40
                );
            } else {
                let stop = c.stop().unwrap();
                assert_eq!(result, Err(SessionError::Stop(stop)));
                assert_eq!(
                    stop,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::BufferedUtf16Units,
                        limit: 39,
                        current: 20,
                        requested_delta: 20,
                    }
                );
                assert_eq!(buffer.units(), &[65; 20]);
                assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(s)) if s==stop));
                assert_eq!(
                    session.append_output_default_value(&mut c, &output, &mut buffer),
                    Err(SessionError::Stop(stop))
                );
            }
            buffer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn output_observations_reject_foreign_destination_and_unavailable_or_foreign_nodes() {
        let source = crate::HtmlSource::utf8("<output>A");
        let (mut c, mut session, output, mut lexer) = output_owner(&source, limits());
        let mut text = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &output, None, &mut text, None)
            .unwrap();
        lexer.release(&mut c).unwrap();
        let mut other = control();
        let mut foreign = Utf16Buffer::new(&mut other).unwrap();
        foreign.push_unit(81, &mut other).unwrap();
        assert!(matches!(
            session.append_output_value(&mut c, &output, &mut foreign),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(foreign.units(), [81]);
        assert!(c.stop().is_some());
        foreign.release(&mut other).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        storage_zero(&other);

        let (mut c, mut session, output, lexer) = output_owner(&source, limits());
        lexer.release(&mut c).unwrap();
        let mut second =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let wrong = second.root(&mut c).unwrap();
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        assert_eq!(
            session.append_output_value(&mut c, &wrong, &mut buffer),
            Err(SessionError::ForeignNode)
        );
        assert!(buffer.units().is_empty());
        let imported = session
            .allocate_element(&mut c, element(Namespace::html(), "output"))
            .unwrap();
        assert_eq!(
            session.output_state(&mut c, &imported),
            Err(SessionError::OutputMetadataUnavailable)
        );
        assert_eq!(
            session
                .output_state(&mut c, &output)
                .unwrap()
                .default_value_override,
            OutputDefaultValueOverride::Null
        );
        buffer.release(&mut c).unwrap();
        second.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn output_metadata_slots_reject_before_lexical_take_with_exact_auxiliary_boundary() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for cap in [13, 14] {
            let source = crate::HtmlSource::utf8("<output>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = session.root(&mut c).unwrap();
            let html = session.allocate_implicit_html(&mut c).unwrap();
            session.insert(&mut c, &root, &html, None, None).unwrap();
            let mut tag = Some(next_fixture_tag(
                &mut lexer,
                &mut c,
                &[111, 117, 116, 112, 117, 116],
            ));
            session.allocate_implicit_body(&mut c, None).unwrap();
            // Actual root/html/body registry + record/ticket entries are9;
            // After its document binding, Output and canonical association require2 more initialized slots.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                13
            );
            let backing = Arc::as_ptr(tag.as_ref().unwrap().name.value.text().backing());
            assert!(matches!(
                session.allocate_html_tag_for_creation(
                    &mut c,
                    &mut tag,
                    &html,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                ),
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    ..
                }))
            ));
            assert_eq!(
                Arc::as_ptr(tag.as_ref().unwrap().name.value.text().backing()),
                backing
            );
            release_token_kind(&mut Some(TokenKind::Tag(tag.take().unwrap())), &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn output_cancellation_preserves_owned_scratch_and_blocks_metadata_freeze_without_reset() {
        let source = crate::HtmlSource::utf8("<output>A");
        let cancellation = HtmlParseCancellation::new();
        let (mut c, mut session, output, mut lexer) =
            output_owner_cancellable(&source, limits(), cancellation.clone());
        let mut text = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &output, None, &mut text, None)
            .unwrap();
        lexer.release(&mut c).unwrap();
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        buffer.push_unit(81, &mut c).unwrap();
        cancellation.cancel();
        assert_eq!(
            session.append_output_value(&mut c, &output, &mut buffer),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(buffer.units(), [81]);
        assert_eq!(
            session.output_state(&mut c, &output),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert!(matches!(
            session.freeze(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        buffer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        storage_zero(&c);
    }

    #[test]
    fn output_initial_empty_reset_precedes_parser_association_and_cancellation_retains_both_slots()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><output>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut form = Some(next_fixture_tag(&mut lexer, &mut c, &[102, 111, 114, 109]));
        let form = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut form,
                &root,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &root, &form, None, None).unwrap();
        let mut tag = Some(next_fixture_tag(
            &mut lexer,
            &mut c,
            &[111, 117, 116, 112, 117, 116],
        ));
        session.active_mut().unwrap().cancel_after_output_reset = Some(cancellation);
        assert!(matches!(
            session.allocate_html_tag_for_creation(
                &mut c,
                &mut tag,
                &form,
                Some(&form),
                crate::construction::parser_session::CreationContext::Ordinary
            ),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert!(tag.is_none());
        let storage = session.active().unwrap();
        let output_slot = storage
            .payloads
            .values
            .iter()
            .find_map(|p| match p {
                RetainedPayload::Output {
                    node: Some(node),
                    state,
                } => Some((*node, *state)),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            output_slot.1.default_value_override,
            OutputDefaultValueOverride::Null
        );
        assert!(storage.records[output_slot.0].children.is_empty());
        assert!(storage.records[output_slot.0].parent.is_none());
        assert!(storage.payloads.values.iter().any(|p| matches!(p,
            RetainedPayload::FormAssociation{node:None,state}
            if state.kind==FormAssociationKind::Output && state.owner.is_none()
               && !state.parser_inserted && state.evidence==FormAssociationEvidence::Created
        )));
        assert!(matches!(
            session.freeze(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        storage_zero(&c);
    }

    #[test]
    fn output_single_text_read_has_exact_physical_work_and_one_short_terminal_cleanup() {
        for remaining in [20, 21] {
            let source = crate::HtmlSource::utf8("<output>A");
            let (mut c, mut session, output, mut lexer) = output_owner(&source, limits());
            let mut text = next_text(&mut lexer, &mut c);
            session
                .insert_text(&mut c, &output, None, &mut text, None)
                .unwrap();
            lexer.release(&mut c).unwrap();
            let mut destination = Utf16Buffer::new(&mut c).unwrap();
            destination.push_unit(81, &mut c).unwrap();
            destination.clear(&mut c).unwrap(); // capacity for1 exists before measured copying.
            // Getter visits: active readiness+index7, root Document/self-binding2 + Output node-binding1 + Output slot1,
            // root+child2, Text1, leaf/parent/child-list/sibling4 =18.
            // Copies are one finite state byte plus one UTF16 unit =3.
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let maximum = limits().work_credits();
            c.charge(WorkCategory::StateStep, maximum - used - remaining)
                .unwrap();
            let before = c.usage().clone();
            let result = session.append_output_value(&mut c, &output, &mut destination);
            assert_eq!(destination.units(), [65]);
            if remaining == 21 {
                result.unwrap();
                assert_eq!(
                    c.usage().work(WorkCategory::RecordVisit)
                        - before.work(WorkCategory::RecordVisit),
                    18
                );
                assert_eq!(
                    c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
                    3
                );
                assert_eq!(
                    c.usage().work(WorkCategory::InspectByte)
                        - before.work(WorkCategory::InspectByte),
                    0
                );
                assert_eq!(
                    c.usage().work(WorkCategory::RecordWrite)
                        - before.work(WorkCategory::RecordWrite),
                    0
                );
                assert_eq!(
                    c.usage().work(WorkCategory::StateStep) - before.work(WorkCategory::StateStep),
                    0
                );
                assert!(c.stop().is_none());
                c.charge(WorkCategory::StateStep, 1).unwrap_err(); // terminal cleanup uses no useful work.
            } else {
                assert_eq!(result, Err(SessionError::Stop(c.stop().unwrap())));
            }
            let stop = c.stop().unwrap();
            assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(s)) if s==stop));
            destination.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), Some(stop));
            storage_zero(&c);
        }
    }

    fn textarea_owner<'a>(
        source: &'a crate::HtmlSource,
        caps: HtmlParseLimits,
    ) -> (
        ParseControl,
        ParserArenaSession,
        ParserNode,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
    ) {
        textarea_owner_cancellable(source, caps, HtmlParseCancellation::new())
    }
    fn textarea_owner_cancellable<'a>(
        source: &'a crate::HtmlSource,
        caps: HtmlParseLimits,
        cancellation: HtmlParseCancellation,
    ) -> (
        ParseControl,
        ParserArenaSession,
        ParserNode,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let mut c = ParseControl::new(caps, cancellation);
        c.admit_source(source).unwrap();
        let mut tokenizer = LexerFixture::new(source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let tag = next_fixture_tag(
            &mut tokenizer,
            &mut c,
            &[116, 101, 120, 116, 97, 114, 101, 97],
        );
        let mut input = Some(tag);
        let textarea = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &root,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        assert!(input.is_none());
        session
            .insert(&mut c, &root, &textarea, None, None)
            .unwrap();
        (c, session, textarea, tokenizer)
    }

    fn next_text(
        tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> Option<Text> {
        let TokenKind::Text(text) = lexical_kind(tokenizer, c) else {
            panic!("actual lexical text")
        };
        Some(text)
    }

    #[test]
    fn textarea_raw_is_independent_exact_child_text_and_retired_backing_is_disposed() {
        let source = crate::HtmlSource::utf8("<textarea>AB<span>C");
        let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, limits());
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &node)
                .unwrap()
                .code_units(),
            []
        );
        let state = session.textarea_state(&mut c, &node).unwrap();
        assert!(!state.dirty_value && !state.user_validity);
        assert_eq!(
            session.form_association_state(&mut c, &node).unwrap().kind,
            FormAssociationKind::Textarea
        );
        let mut a = next_text(&mut tokenizer, &mut c);
        let text = session
            .insert_text(&mut c, &node, None, &mut a, None)
            .unwrap()
            .unwrap()
            .node;
        let first_raw = session.textarea_state(&mut c, &node).unwrap().raw;
        let native = native_text(&session, &text);
        let raw = session
            .textarea_raw_value(&mut c, &node)
            .unwrap()
            .code_units()
            .to_vec();
        assert_eq!(native, [65]);
        assert_eq!(raw, [65]);
        let HtmlNodeKind::Text(native) = &session.active().unwrap().records[text.index].kind else {
            panic!("text")
        };
        assert!(!Arc::ptr_eq(
            native.backing(),
            session.textarea_raw_value(&mut c, &node).unwrap().backing()
        ));
        let mut b = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &node, None, &mut b, None)
            .unwrap();
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &node)
                .unwrap()
                .code_units(),
            [65, 66]
        );
        assert!(matches!(
            session.active().unwrap().payloads.values[first_raw],
            RetainedPayload::HistoricalString { .. }
        ));
        let before_reset = c
            .usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current;
        for _ in 0..4 {
            session.parser_stack_removed(&mut c, &node, None).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                before_reset
            );
        }
        // A descendant Text is excluded: only immediate Text children contribute.
        let span = next_fixture_tag(&mut tokenizer, &mut c, &[115, 112, 97, 110]);
        let child = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(span),
                &node,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &node, &child, None, None).unwrap();
        let mut text_c = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &child, None, &mut text_c, None)
            .unwrap();
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &node)
                .unwrap()
                .code_units(),
            [65, 66]
        );
        tokenizer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let id = document.root().children().next().unwrap();
        assert_eq!(
            frozen.textarea_raw_value(&mut c, &id).unwrap().code_units(),
            [65, 66]
        );
        let state = frozen.textarea_state(&mut c, &id).unwrap();
        assert!(!state.dirty_value && !state.user_validity);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn textarea_same_parent_reference_removal_runs_old_view_before_insertion() {
        let source = crate::HtmlSource::utf8("<textarea>AB");
        let cancel = HtmlParseCancellation::new();
        let (mut c, mut session, node, mut tokenizer) =
            textarea_owner_cancellable(&source, limits(), cancel.clone());
        let mut a = next_text(&mut tokenizer, &mut c);
        let text = session
            .insert_text(&mut c, &node, None, &mut a, None)
            .unwrap()
            .unwrap()
            .node;
        let mut b = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &node, None, &mut b, None)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        // Cancellation between hook phases proves the removal raw prefix was
        // applied, although final before==node topology is deliberately equal.
        session.active_mut().unwrap().cancel_after_removal = Some(cancel.clone());
        let Err(SessionError::Stop(first)) =
            session.insert(&mut c, &node, &text, Some(&text), None)
        else {
            panic!("actual phase cancellation")
        };
        let storage = session.active().unwrap();
        assert_eq!(storage.records[text.index].parent, Some(node.index));
        assert_eq!(native_text(&session, &text), [65, 66]);
        let raw = storage
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::Textarea {
                    node: Some(owner),
                    state,
                } if *owner == node.index => Some(state.raw),
                _ => None,
            })
            .unwrap();
        let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
            panic!("raw owner")
        };
        assert_eq!(value.code_units(), []);
        assert!(storage.effects.form.is_some());
        assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(stop)) if stop==first));
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.release(&mut c).unwrap();
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(c.stop(), Some(first));
        storage_zero(&c);
    }

    #[test]
    fn textarea_raw_coexistence_has_exact_43_unit_admission_and_retained_terminal_prefix() {
        // textarea name8 + HTML URI28; A(native1,raw1)=38. B adds lexical1,
        // merged2, new raw2 while old raw1 coexists: exact peak43, then current42.
        for cap in [42, 43] {
            let source = crate::HtmlSource::utf8("<textarea>AB");
            let caps = limits()
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap();
            let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, caps);
            let mut a = next_text(&mut tokenizer, &mut c);
            let text = session
                .insert_text(&mut c, &node, None, &mut a, None)
                .unwrap()
                .unwrap()
                .node;
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                38
            );
            let old_raw = session.textarea_state(&mut c, &node).unwrap().raw;
            let mut b = next_text(&mut tokenizer, &mut c);
            let result = session.insert_text(&mut c, &node, None, &mut b, None);
            assert!(b.is_none());
            assert_eq!(native_text(&session, &text), [65, 66]);
            if cap == 43 {
                assert!(result.is_ok());
                assert_eq!(
                    session
                        .textarea_raw_value(&mut c, &node)
                        .unwrap()
                        .code_units(),
                    [65, 66]
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    42
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .high_water,
                    43
                );
                assert!(matches!(
                    session.active().unwrap().payloads.values[old_raw],
                    RetainedPayload::HistoricalString { .. }
                ));
            } else {
                let Err(SessionError::Stop(first)) = result else {
                    panic!("actual coexistence stop")
                };
                assert!(session.active().unwrap().effects.form.is_some());
                let RetainedPayload::String { value, .. } =
                    &session.active().unwrap().payloads.values[old_raw]
                else {
                    panic!("old owner retained")
                };
                assert_eq!(value.code_units(), [65]);
                assert!(
                    matches!(session.textarea_state(&mut c,&node),Err(SessionError::Stop(s)) if s==first)
                );
                assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(s)) if s==first));
            }
            tokenizer.release(&mut c).unwrap();
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
        }
    }

    #[test]
    fn textarea_repeated_reset_retires_backing_but_keeps_initialized_slots_bounded() {
        // Registry slots: initial raw/textarea/form3, name/namespace/attrs3,
        // actual native A1 and first children-change raw1 plus two retired static postconnection list slots plus four real document facts =14. Three actual nodes
        // add three auxiliary slots, so baseline13. The finite cap18
        // admits initial lexical/topology staging, then five reset slots;
        // the next reset retains the current raw and latches the slot cap.
        let source = crate::HtmlSource::utf8("<textarea>A");
        let caps = limits()
            .into_builder()
            .auxiliary_entries(22)
            .finish()
            .unwrap();
        let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, caps);
        let mut text = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &node, None, &mut text, None)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        assert_eq!(session.active().unwrap().payloads.values.len(), 14);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            17
        );
        let old = Arc::downgrade(session.textarea_raw_value(&mut c, &node).unwrap().backing());
        for _ in 0..5 {
            session.parser_stack_removed(&mut c, &node, None).unwrap();
        }
        assert!(
            old.upgrade().is_none(),
            "retired backing is actually disposed"
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            22
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            38
        );
        let current = Arc::downgrade(session.textarea_raw_value(&mut c, &node).unwrap().backing());
        let current_slot = session.textarea_state(&mut c, &node).unwrap().raw;
        let result = session.parser_stack_removed(&mut c, &node, None);
        let first = c.stop().unwrap();
        assert_eq!(result, Err(SessionError::Stop(first)));
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                ..
            }
        ));
        let storage = session.active().unwrap();
        assert_eq!(storage.payloads.values.len(), 19);
        let RetainedPayload::String { value, .. } = &storage.payloads.values[current_slot] else {
            panic!("original current raw retained")
        };
        assert_eq!(value.code_units(), [65]);
        assert!(current.upgrade().is_some());
        assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(s)) if s==first));
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.release(&mut c).unwrap();
        assert!(current.upgrade().is_none());
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(c.stop(), Some(first));
        storage_zero(&c);
    }

    #[test]
    fn textarea_raw_and_metadata_slots_are_admitted_before_lexical_take() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for cap in [15, 16, 17] {
            let source = crate::HtmlSource::utf8("<textarea>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = session.root(&mut c).unwrap();
            let html = session.allocate_implicit_html(&mut c).unwrap();
            session.insert(&mut c, &root, &html, None, None).unwrap();
            let tag = next_fixture_tag(
                &mut tokenizer,
                &mut c,
                &[116, 101, 120, 116, 97, 114, 101, 97],
            );
            let backing = Arc::as_ptr(tag.name.value.text().backing());
            let mut input = Some(tag);
            session.allocate_implicit_body(&mut c, None).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                13
            );
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &html,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            );
            assert!(matches!(result, Err(SessionError::Stop(_))));
            if cap < 17 {
                assert_eq!(
                    Arc::as_ptr(input.as_ref().unwrap().name.value.text().backing()),
                    backing
                );
            } else {
                assert!(input.is_none());
            }
            if let Some(tag) = input.take() {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn textarea_initial_reset_precedes_parser_association_commit_and_stop_owns_every_slot() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><textarea>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let form_tag = next_fixture_tag(&mut tokenizer, &mut c, &[102, 111, 114, 109]);
        let form = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(form_tag),
                &root,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &root, &form, None, None).unwrap();
        let tag = next_fixture_tag(
            &mut tokenizer,
            &mut c,
            &[116, 101, 120, 116, 97, 114, 101, 97],
        );
        session.active_mut().unwrap().cancel_after_textarea_reset = Some(cancellation);
        let mut input = Some(tag);
        assert!(matches!(
            session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &form,
                Some(&form),
                crate::construction::parser_session::CreationContext::Ordinary
            ),
            Err(SessionError::Stop(_))
        ));
        assert!(input.is_none());
        let storage = session.active().unwrap();
        let state = storage
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::Textarea {
                    node: Some(_),
                    state,
                } => Some(*state),
                _ => None,
            })
            .unwrap();
        assert!(!state.dirty_value && !state.user_validity);
        let RetainedPayload::String { value, .. } = &storage.payloads.values[state.raw] else {
            panic!("initial raw owner")
        };
        assert_eq!(value.code_units(), []);
        assert!(storage.payloads.values.iter().any(|entry|matches!(entry,
            RetainedPayload::FormAssociation{node:None,state} if state.kind==FormAssociationKind::Textarea && state.owner.is_none() && !state.parser_inserted && state.evidence==FormAssociationEvidence::Created
        )));
        let first = c.stop().unwrap();
        assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(stop)) if stop==first));
        tokenizer.release(&mut c).unwrap();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.release(&mut c).unwrap();
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(c.stop(), Some(first));
        storage_zero(&c);
    }

    #[test]
    fn textarea_raw_child_moves_update_both_parents_and_ignore_descendant_text() {
        let source = crate::HtmlSource::utf8("<textarea>A<textarea>B");
        let (mut c, mut session, left, mut tokenizer) = textarea_owner(&source, limits());
        let mut a = next_text(&mut tokenizer, &mut c);
        let text = session
            .insert_text(&mut c, &left, None, &mut a, None)
            .unwrap()
            .unwrap()
            .node;
        let tag = next_fixture_tag(
            &mut tokenizer,
            &mut c,
            &[116, 101, 120, 116, 97, 114, 101, 97],
        );
        let root = session.root(&mut c).unwrap();
        let right = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &root,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &root, &right, None, None).unwrap();
        let mut b = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &right, None, &mut b, None)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        session.insert(&mut c, &right, &text, None, None).unwrap();
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &left)
                .unwrap()
                .code_units(),
            []
        );
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &right)
                .unwrap()
                .code_units(),
            [66, 65]
        );
        session.insert(&mut c, &left, &right, None, None).unwrap();
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &left)
                .unwrap()
                .code_units(),
            []
        );
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &right)
                .unwrap()
                .code_units(),
            [66, 65]
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn textarea_getters_qualify_actual_arena_control_and_preserve_alias_cleanup() {
        let source = crate::HtmlSource::utf8("<textarea>A");
        let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, limits());
        let mut a = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &node, None, &mut a, None)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        let mut unrelated =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let other = unrelated.root(&mut c).unwrap();
        assert!(matches!(
            session.textarea_state(&mut c, &other),
            Err(SessionError::ForeignNode)
        ));
        let root = session.root(&mut c).unwrap();
        assert!(matches!(
            session.textarea_state(&mut c, &root),
            Err(SessionError::TextareaMetadataUnavailable)
        ));
        let alias = session.textarea_raw_value(&mut c, &node).unwrap().clone();
        assert!(matches!(
            session.release(&mut c),
            Err(SessionError::AliasedPayload)
        ));
        assert!(session.storage.is_some());
        drop(alias);
        let mut foreign = control();
        assert!(matches!(
            session.textarea_raw_value(&mut foreign, &node),
            Err(SessionError::Stop(_))
        ));
        assert!(c.stop().is_none());
        unrelated.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        storage_zero(&foreign);
    }

    fn formatting_lexical_tag(
        tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> (Option<Tag>, crate::HtmlSourceRange) {
        loop {
            match tokenizer.next(c, false).unwrap().expect("actual token") {
                crate::parser::tokenizer::token::Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    let TokenKind::Tag(tag) = token.kind else {
                        panic!("expected real tag")
                    };
                    return (Some(tag), token.source);
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    fn formatting_parent(c: &mut ParseControl, session: &mut ParserArenaSession) -> ParserNode {
        let root = session.root(c).unwrap();
        let html = session.allocate_implicit_html(c).unwrap();
        session.insert(c, &root, &html, None, None).unwrap();
        let body = session.allocate_implicit_body(c, None).unwrap();
        session.insert(c, &html, &body, None, None).unwrap();
        body
    }
    #[test]
    fn template_owned_html_wrapper_pre_registration_stops_reclaim_lexical_fields() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // require: 6 visits; kind: 1; name owner+alias: 2; lowercase
        // template: 16; indexed attribute: 1; its two owner+alias checks: 4;
        // lowercase a: 2. This is the actual borrowed preflight, before take.
        const PREFLIGHT: u64 = 32;
        // The newly owned wrapper reads length (1), eight units (3 each),
        // then root kind (1). Cut each credit before document preparation.
        for wrapper_credit in 0..26 {
            let source = crate::HtmlSource::utf8("<template a='V&#65;'>");
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut input, _) = formatting_lexical_tag(&mut lexer, &mut c);
            lexer.release(&mut c).unwrap();
            let tag = input.as_ref().unwrap();
            assert_eq!(
                tag.name.value.text().code_units(),
                [116, 101, 109, 112, 108, 97, 116, 101]
            );
            assert_eq!(
                tag.attributes.as_slice()[0].value.value.text().code_units(),
                [86, 65]
            );
            assert!(!tag.name.origins.as_slice().is_empty());
            assert_eq!(tag.attributes.as_slice()[0].value.origins.len(), 2);
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - PREFLIGHT - wrapper_credit,
            )
            .unwrap();
            let result = session.allocate_html_tag(&mut c, &mut input);
            let first = c.stop().expect("wrapper work stop");
            assert!(matches!(
                first,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }
            ));
            assert_eq!(result.err(), Some(SessionError::Stop(first)));
            assert!(
                input.is_none(),
                "lexical transfer precedes wrapper, credit={wrapper_credit}"
            );
            assert_eq!(session.active().unwrap().records.len(), 1);
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            let work = c.usage().resource(HtmlParseResource::WorkCredits);
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), Some(first));
            assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
            storage_zero(&c);
        }
    }

    #[test]
    fn template_owned_html_wrapper_cancellation_reclaims_transferred_lexical_fields() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<template a='V&#65;'>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, _) = formatting_lexical_tag(&mut lexer, &mut c);
        lexer.release(&mut c).unwrap();
        // Exercise the real owned wrapper with fields from a validated actual
        // lexical tag. Cancellation is observable at its first admission.
        SessionStorage::check_html_tag(&mut c, input.as_mut().unwrap()).unwrap();
        let Tag {
            name, attributes, ..
        } = input.take().unwrap();
        cancellation.cancel();
        assert_eq!(
            session
                .active_mut()
                .unwrap()
                .allocate_html_payload(&mut c, name.value, Some(name.origins), attributes,)
                .err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }

    fn formatting_complete_copy_prefix(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        creation: &FormattingCreation,
        parent: &ParserNode,
    ) -> u64 {
        // A separate real prefix locates the admission window; it is not the
        // cleanup oracle. Independently check all copied fields/storage below.
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.validate_standalone_document(c).unwrap();
        let storage = session.active().unwrap();
        let parent = storage.index(c, parent).unwrap();
        storage.container(c, parent).unwrap();
        assert!(!storage.associated_document_parent(c, parent).unwrap());
        session.require_formatting_creation(c, creation).unwrap();
        let document = storage.node_document_ref(c, parent).unwrap();
        session
            .active_mut()
            .unwrap()
            .prepare_document_allocation(c, Some(document), false)
            .unwrap();
        let original = creation.token.as_ref().unwrap();
        let mut copy = Some(copy_formatting_tag(c, original).unwrap());
        let complete = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        let copied = copy.as_ref().unwrap();
        assert_eq!(copied.name.value.text().code_units(), [98]);
        assert_eq!(
            copied.attributes.as_slice()[0]
                .name
                .value
                .text()
                .code_units(),
            [97]
        );
        assert_eq!(
            copied.attributes.as_slice()[0]
                .value
                .value
                .text()
                .code_units(),
            [86, 65]
        );
        assert_ne!(
            copied.name.value.text().code_units().as_ptr(),
            original.name.value.text().code_units().as_ptr()
        );
        let origins = |text: &Text| {
            text.origins
                .as_slice()
                .iter()
                .map(|o| (o.start, o.end, o.source.clone(), o.kind))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            origins(&copied.attributes.as_slice()[0].value),
            origins(&original.attributes.as_slice()[0].value)
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            2
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            8
        );
        discard_formatting_tag(&mut copy, c).unwrap();
        complete
    }

    #[test]
    fn formatting_completed_copy_take_stops_keep_recipe_and_reclaim_independent_fields() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        fn fixture(
            source: &crate::HtmlSource,
            c: &mut ParseControl,
        ) -> (ParserArenaSession, ParserNode, FormattingCreation) {
            c.admit_source(source).unwrap();
            let mut session =
                ParserArenaSession::new(c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
            let parent = formatting_parent(c, &mut session);
            let mut lexer = LexerFixture::new(source, State::Data, None, c).unwrap();
            let (mut input, range) = formatting_lexical_tag(&mut lexer, c);
            lexer.release(c).unwrap();
            let creation = session
                .retain_formatting_creation(c, &mut input, range)
                .unwrap();
            assert!(input.is_none());
            (session, parent, creation)
        }
        let source = crate::HtmlSource::utf8("<b a='V&#65;'>");
        let mut measured = control();
        let (mut session, parent, mut creation) = fixture(&source, &mut measured);
        let prefix =
            formatting_complete_copy_prefix(&mut session, &mut measured, &creation, &parent);
        creation.release(&mut measured).unwrap();
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        // take_payload: one actual Option visit, then the fixed Option<Tag>
        // copy, then two writes. Cut each admitted action after complete copy.
        for take_credit in [
            0,
            1,
            1 + mem::size_of::<Option<Tag>>() as u64,
            2 + mem::size_of::<Option<Tag>>() as u64,
        ] {
            let mut c = control();
            let (mut session, parent, mut creation) = fixture(&source, &mut c);
            let pointer = creation
                .token
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr();
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - prefix - take_credit,
            )
            .unwrap();
            let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
            let first = c.stop().expect("copied token take work stop");
            assert!(matches!(
                first,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }
            ));
            assert_eq!(result.err(), Some(SessionError::Stop(first)));
            assert_eq!(session.active().unwrap().records.len(), 3);
            assert_eq!(
                creation
                    .token
                    .as_ref()
                    .unwrap()
                    .name
                    .value
                    .text()
                    .code_units()
                    .as_ptr(),
                pointer
            );
            assert_eq!(creation.token.as_ref().unwrap().attributes.len(), 1);
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            let work = c.usage().resource(HtmlParseResource::WorkCredits);
            creation.release(&mut c).unwrap();
            assert!(creation.token.is_none());
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), Some(first));
            assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
            storage_zero(&c);
        }
    }
    #[test]
    fn formatting_replay_copies_exact_utf16_attributes_and_original_origins_into_distinct_nodes() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<b z='A&#x1F600;\r\n"
                .encode_utf16()
                .chain([0xd800])
                .chain("' a=x>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = formatting_parent(&mut c, &mut session);
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let original_pointer = input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let original_origins = input.as_ref().unwrap().attributes.as_slice()[0]
            .value
            .origins
            .as_slice()
            .iter()
            .map(|o| (o.start, o.end, o.source.clone(), o.kind))
            .collect::<Vec<_>>();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        assert!(input.is_none());
        let first = session
            .create_formatting_from_creation(&mut c, &creation, &parent)
            .unwrap();
        let second = session
            .create_formatting_from_creation(&mut c, &creation, &parent)
            .unwrap();
        assert!(!session.same_node(&mut c, &first, &second).unwrap());
        session.insert(&mut c, &parent, &first, None, None).unwrap();
        session
            .insert(&mut c, &parent, &second, None, None)
            .unwrap();
        for node in [&first, &second] {
            let storage = session.active().unwrap();
            let HtmlNodeKind::Element(element) = &storage.records[node.index].kind else {
                panic!("element")
            };
            assert_ne!(
                element.name().local_name().code_units().as_ptr(),
                original_pointer
            );
            assert_eq!(element.name().namespace(), &Namespace::html());
            let attributes = element.attributes().as_slice();
            assert_eq!(attributes[0].name().local_name().code_units(), [122]);
            assert_eq!(attributes[0].name().namespace(), &Namespace::none());
            assert_eq!(
                attributes[0].value().code_units(),
                [65, 0xd83d, 0xde00, 10, 0xd800]
            );
            assert_eq!(attributes[1].name().local_name().code_units(), [97]);
            let origins = storage.payloads.values.iter().find_map(|payload| match payload {
                RetainedPayload::String { origins:Some(origins),location:StringLocation::AttributeValue(container,0),.. }
                    if matches!(&storage.payloads.values[*container],RetainedPayload::Attributes(Some(AttributeStorage::Frozen {element:Some(index),..})) if *index==node.index) => Some(storage.observed_origins(origins, &mut c).unwrap()),
                _ => None,
            }).expect("actual attribute origins");
            let observed = origins
                .iter()
                .map(|o| (o.start, o.end, o.source.clone(), o.kind))
                .collect::<Vec<_>>();
            assert_eq!(observed, original_origins);
        }
        creation.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        let mut arena = session.freeze(&mut c).unwrap();
        assert!(arena.document().is_some());
        arena.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            5
        );
    }

    #[test]
    fn formatting_noah_ignores_attribute_order_duplicate_recovery_and_self_closing_but_preserves_exact_values()
     {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source =
            crate::HtmlSource::utf8("<b a=x z=y><b z=y a=x a=ignored /><b a=x z=Y><i z=y a=x>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut first, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        let mut a = session
            .retain_formatting_creation(&mut c, &mut first, range)
            .unwrap();
        let (mut second, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        assert!(second.as_ref().unwrap().had_duplicate_attributes);
        assert!(second.as_ref().unwrap().self_closing);
        let mut b = session
            .retain_formatting_creation(&mut c, &mut second, range)
            .unwrap();
        let (mut third, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        let mut different_value = session
            .retain_formatting_creation(&mut c, &mut third, range)
            .unwrap();
        let (mut fourth, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        let mut different_name = session
            .retain_formatting_creation(&mut c, &mut fourth, range)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        assert!(session.same_formatting_creation(&mut c, &a, &b).unwrap());
        assert!(
            !session
                .same_formatting_creation(&mut c, &a, &different_value)
                .unwrap()
        );
        assert!(
            !session
                .same_formatting_creation(&mut c, &a, &different_name)
                .unwrap()
        );
        a.release(&mut c).unwrap();
        b.release(&mut c).unwrap();
        different_value.release(&mut c).unwrap();
        different_name.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn formatting_capture_rejects_alias_and_wrong_family_without_consuming_original() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<b a=x><applet>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        let alias = input.as_ref().unwrap().name.value.text().clone();
        assert!(matches!(
            session.retain_formatting_creation(&mut c, &mut input, range.clone()),
            Err(SessionError::AliasedPayload)
        ));
        assert!(input.is_some());
        assert_eq!(c.stop(), None);
        drop(alias);
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        let (mut wrapper, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        assert!(matches!(
            session.retain_formatting_creation(&mut c, &mut wrapper, range),
            Err(SessionError::WrongToken)
        ));
        assert!(wrapper.is_some());
        discard_formatting_tag(&mut wrapper, &mut c).unwrap();
        creation.release(&mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn formatting_replay_rejects_foreign_arena_and_parent_before_new_payload_admission() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<b a=x>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = formatting_parent(&mut c, &mut session);
        let mut other =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let other_parent = other.root(&mut c).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        let before = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        assert!(matches!(
            other.create_formatting_from_creation(&mut c, &creation, &other_parent),
            Err(SessionError::ForeignNode)
        ));
        assert!(matches!(
            session.create_formatting_from_creation(&mut c, &creation, &other_parent),
            Err(SessionError::ForeignNode)
        ));
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            before
        );
        assert!(creation.token.is_some());
        let node = session
            .create_formatting_from_creation(&mut c, &creation, &parent)
            .unwrap();
        session.insert(&mut c, &parent, &node, None, None).unwrap();
        creation.release(&mut c).unwrap();
        other.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn formatting_replay_token_attribute_coexistence_exact_and_one_short_reclaims_only_copies() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for cap in [3, 4] {
            let source = crate::HtmlSource::utf8("<b a=x z=y>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .token_attributes(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let parent = formatting_parent(&mut c, &mut session);
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
            tokenizer.release(&mut c).unwrap();
            let mut creation = session
                .retain_formatting_creation(&mut c, &mut input, range)
                .unwrap();
            let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
            if cap == 3 {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::TokenAttributes,
                        limit: 3,
                        current: 3,
                        requested_delta: 1
                    }))
                ));
                assert!(session.freeze(&mut c).is_err());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    5
                );
                assert_eq!(
                    c.usage().resource(HtmlParseResource::ManagedNodes).current,
                    3
                );
            } else {
                let node = result.unwrap();
                session.insert(&mut c, &parent, &node, None, None).unwrap();
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::TokenAttributes)
                        .high_water,
                    4
                );
            }
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::TokenAttributes)
                    .current,
                2
            );
            let stop = c.stop();
            creation.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }

    #[test]
    fn formatting_recipe_terminal_cancel_and_foreign_disposal_preserve_owner_then_reclaim_once() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<b a=x>");
        let cancel = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancel.clone());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = formatting_parent(&mut c, &mut session);
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        let mut foreign = control();
        foreign.admit_source(&source).unwrap();
        assert!(creation.release(&mut foreign).is_err());
        assert!(creation.token.is_some());
        cancel.cancel();
        assert!(matches!(
            session.create_formatting_from_creation(&mut c, &creation, &parent),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        let alias = creation.token.as_ref().unwrap().name.value.text().clone();
        assert_eq!(creation.release(&mut c), Err(SessionError::AliasedPayload));
        assert!(creation.token.is_some());
        drop(alias);
        creation.release(&mut c).unwrap();
        creation.release(&mut c).unwrap();
        assert!(session.freeze(&mut c).is_err());
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn formatting_same_source_foreign_control_replay_preserves_both_arenas_and_original_recipe() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<b a=x>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        let mut foreign = control();
        foreign.admit_source(&source).unwrap();
        let mut other =
            ParserArenaSession::new(&mut foreign, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let other_root = other.root(&mut foreign).unwrap();
        assert!(matches!(
            other.create_formatting_from_creation(&mut foreign, &creation, &other_root),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert_eq!(
            foreign
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            foreign
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            1
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            3
        );
        assert_eq!(c.stop(), None);
        assert!(creation.token.is_some());
        other.release(&mut foreign).unwrap();
        storage_zero(&foreign);
        session
            .create_formatting_from_creation(&mut c, &creation, &root)
            .unwrap();
        creation.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn formatting_child_transfer_preserves_existing_destination_order_and_backlinks() {
        let mut c = control();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = formatting_parent(&mut c, &mut session);
        let from = session.allocate_implicit_p(&mut c, None).unwrap();
        let to = session.allocate_implicit_p(&mut c, None).unwrap();
        session.insert(&mut c, &parent, &from, None, None).unwrap();
        session.insert(&mut c, &parent, &to, None, None).unwrap();
        let existing = session.allocate_implicit_p(&mut c, None).unwrap();
        session.insert(&mut c, &to, &existing, None, None).unwrap();
        let one = session.allocate_implicit_p(&mut c, None).unwrap();
        let two = session.allocate_implicit_p(&mut c, None).unwrap();
        session.insert(&mut c, &from, &one, None, None).unwrap();
        session.insert(&mut c, &from, &two, None, None).unwrap();
        let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
        session.reparent_children(&mut c, &from, &to, None).unwrap();
        assert!(
            session.active().unwrap().records[from.index]
                .children
                .is_empty()
        );
        assert_eq!(
            session.active().unwrap().records[to.index].children,
            [existing.index, one.index, two.index]
        );
        for child in [&existing, &one, &two] {
            let actual_parent = session.parent(&mut c, child).unwrap().unwrap();
            assert!(session.same_node(&mut c, &actual_parent, &to).unwrap());
        }
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedEdges).current,
            edges
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            session.active().unwrap().records.len() as u64
                + session.active().unwrap().payloads.values.len() as u64
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn formatting_namespace_copy_exact_and_one_short_buffer_limit_retains_original_and_staged_name()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Original b needs one unit. Replay's copied name moves to Managed
        // before the defining 28-unit HTML namespace is buffered: peak 1+28.
        for cap in [28, 29] {
            let source = crate::HtmlSource::utf8("<b>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .buffered_utf16_units(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let parent = session.root(&mut c).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
            tokenizer.release(&mut c).unwrap();
            let mut creation = session
                .retain_formatting_creation(&mut c, &mut input, range)
                .unwrap();
            let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
            if cap == 28 {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::BufferedUtf16Units,
                        limit: 28,
                        current: 28,
                        requested_delta: 1
                    }))
                ));
                assert_eq!(
                    c.usage().resource(HtmlParseResource::ManagedNodes).current,
                    1
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    1
                );
                assert!(session.freeze(&mut c).is_err());
            } else {
                assert!(result.is_ok());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    29
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .high_water,
                    29
                );
            }
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                1
            );
            let stop = c.stop();
            creation.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }

    #[test]
    fn formatting_capture_one_short_ordinary_document_qualification_preserves_original_before_take()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Readiness first inspects the real form phase plus five intrinsic readiness fields (six visits).
        // Qualification then admits eligibility/root-kind together (two).
        // One short leaves only readiness admitted; exactly eight completes
        // qualification but cannot admit the next source-range visit.
        for remaining in [7, 8] {
            let source = crate::HtmlSource::utf8("<b>");
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
            tokenizer.release(&mut c).unwrap();
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - work - remaining,
            )
            .unwrap();
            let before = c.usage().work(WorkCategory::RecordVisit);
            assert!(matches!(
                session.retain_formatting_creation(&mut c, &mut input, range),
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            assert!(input.is_some());
            assert_eq!(
                c.usage().work(WorkCategory::RecordVisit) - before,
                if remaining == 8 { 8 } else { 6 }
            );
            let stop = c.stop();
            discard_formatting_tag(&mut input, &mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }

    #[test]
    fn formatting_wide_replay_source_map_coexistence_exact_and_one_short_reclaims_partial_copies() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Real b/v/512 literal x values have 514 original codepoint origins.
        // Independent replay retains them and needs 514 new origin records.
        // Native retention additionally owns four groups and four explicit
        // source-independent field causes (name, URI, attribute name/value).
        for cap in [1035, 1036] {
            let source = crate::HtmlSource::utf8(format!("<b v='{}'>", "x".repeat(512)));
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .buffered_utf16_units(2000)
                    .source_map_entries(cap)
                    .work_credits(100_000_000)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let parent = session.root(&mut c).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
            tokenizer.release(&mut c).unwrap();
            let mut creation = session
                .retain_formatting_creation(&mut c, &mut input, range)
                .unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                514
            );
            let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
            if cap == 1035 {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::SourceMapEntries,
                        limit: 1035,
                        current: 1035,
                        requested_delta: 1
                    }))
                ));
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current,
                    1035
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    514
                );
                assert_eq!(
                    c.usage().resource(HtmlParseResource::ManagedNodes).current,
                    1
                );
                assert!(session.freeze(&mut c).is_err());
            } else {
                assert!(result.is_ok());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current,
                    1036
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .high_water,
                    1036
                );
            }
            let stop = c.stop();
            creation.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }

    #[test]
    fn formatting_capture_foreign_source_and_replay_import_refusal_leave_original_recipe_owned() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<b a=x>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = session.root(&mut c).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        session.allocate_element(&mut c, html("div")).unwrap();
        assert!(matches!(
            session.create_formatting_from_creation(&mut c, &creation, &parent),
            Err(SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            )))
        ));
        assert!(creation.token.is_some());
        creation.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);

        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut input, _range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let foreign = crate::HtmlSource::utf8("<b a=x>").range(0, 7).unwrap();
        assert!(
            session
                .retain_formatting_creation(&mut c, &mut input, foreign)
                .is_err()
        );
        assert!(input.is_some());
        discard_formatting_tag(&mut input, &mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn formatting_child_transfer_stops_with_an_ordered_coherent_prefix_and_reclaims_all_scratch() {
        let mut observed_partial = false;
        for remaining in (0..=1536).step_by(32) {
            let mut c = control();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let parent = formatting_parent(&mut c, &mut session);
            let from = session.allocate_implicit_p(&mut c, None).unwrap();
            let to = session.allocate_implicit_p(&mut c, None).unwrap();
            session.insert(&mut c, &parent, &from, None, None).unwrap();
            session.insert(&mut c, &parent, &to, None, None).unwrap();
            let one = session.allocate_implicit_p(&mut c, None).unwrap();
            let two = session.allocate_implicit_p(&mut c, None).unwrap();
            session.insert(&mut c, &from, &one, None, None).unwrap();
            session.insert(&mut c, &from, &two, None, None).unwrap();
            let before_nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
            let before_edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - work - remaining,
            )
            .unwrap();
            let result = session.reparent_children(&mut c, &from, &to, None);
            let storage = session.active().unwrap();
            let moved = storage.records[to.index].children.len();
            let ordered = [one.index, two.index];
            assert_eq!(storage.records[to.index].children, &ordered[..moved]);
            assert_eq!(storage.records[from.index].children, &ordered[moved..]);
            for (index, child) in ordered.iter().enumerate() {
                assert_eq!(
                    storage.records[*child].parent,
                    Some(if index < moved { to.index } else { from.index })
                );
            }
            observed_partial |= moved == 1;
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                before_nodes
            );
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedEdges).current,
                before_edges
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                storage.records.len() as u64 + storage.payloads.values.len() as u64
            );
            if result.is_err() {
                assert!(session.freeze(&mut c).is_err());
            }
            let stop = c.stop();
            if session.release(&mut c).is_err() {
                assert!(c.stop().is_some());
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            if let Some(stop) = stop {
                assert_eq!(c.stop(), Some(stop));
            }
        }
        assert!(
            observed_partial,
            "a real first-child commit must survive a later work stop"
        );
    }
    use crate::{Attribute, HtmlParseResource};
    use crate::{HtmlParseCancellation, HtmlParseLimits};

    fn body_lexical_value(source: &crate::HtmlSource, c: &mut ParseControl) -> Text {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        c.admit_source(source).unwrap();
        let mut tokenizer = LexerFixture::new(source, State::Data, None, c).unwrap();
        let (value, tag, attribute) = lexical_attribute_text(&mut tokenizer, c);
        tag.release(c).unwrap();
        attribute.release(c).unwrap();
        tokenizer.release(c).unwrap();
        value
    }
    #[test]
    fn recovered_end_br_discards_actual_attributes_and_keeps_name_origin_and_native_empty_attrs() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("</BR x=kept x=ignored>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        assert_eq!(tag.kind, TagKind::End);
        assert!(tag.had_duplicate_attributes);
        let name = tag.name.value.text().backing().as_ptr();
        let origins = tag
            .name
            .origins
            .as_slice()
            .iter()
            .map(|o| (o.start, o.end, o.source.clone(), o.kind))
            .collect::<Vec<_>>();
        let mut input = Some(tag);
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.recover_br_end_tag(&mut c, &mut input).unwrap();
        let tag = input.as_ref().unwrap();
        assert_eq!(tag.kind, TagKind::Start);
        assert_eq!(tag.name.value.text().backing().as_ptr(), name);
        assert_eq!(
            tag.name
                .origins
                .as_slice()
                .iter()
                .map(|o| (o.start, o.end, o.source.clone(), o.kind))
                .collect::<Vec<_>>(),
            origins
        );
        assert_eq!(tag.attributes.len(), 0);
        assert!(tag.had_duplicate_attributes);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            0
        );
        let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
        let root = session.root(&mut c).unwrap();
        session.insert(&mut c, &root, &node, None, None).unwrap();
        tokenizer.release(&mut c).unwrap();
        let mut arena = session.freeze(&mut c).unwrap();
        assert_eq!(locals(arena.fragment().unwrap()), ["br"]);
        let fragment = arena.fragment().unwrap();
        let id = fragment.root().children().next().unwrap();
        let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
            panic!("br")
        };
        assert!(element.attributes().is_empty());
        arena.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn end_br_alias_foreign_and_cancel_reject_before_discarding_any_actual_field() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("</br x=kept>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let alias = input.as_ref().unwrap().attributes.as_slice()[0]
            .value
            .value
            .text()
            .clone();
        let capacity = c.usage().actual_capacity_bytes;
        assert_eq!(
            session.recover_br_end_tag(&mut c, &mut input),
            Err(SessionError::AliasedPayload)
        );
        assert_eq!(input.as_ref().unwrap().kind, TagKind::End);
        assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
        assert_eq!(c.usage().actual_capacity_bytes, capacity);
        drop(alias);
        let mut foreign = control();
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            session.recover_br_end_tag(&mut foreign, &mut input),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
        cancellation.cancel();
        assert_eq!(
            session.recover_br_end_tag(&mut c, &mut input),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(input.as_ref().unwrap().kind, TagKind::End);
        assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
        release_token_kind(&mut input.take().map(TokenKind::Tag), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn end_br_work_stops_retain_preflight_input_or_finish_all_disposal_without_start_reprocessing()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("</br x=one y=two>");
        let mut required = None;
        for remaining in [
            None,
            Some(0),
            Some(1),
            Some(100),
            Some(u64::MAX - 1),
            Some(u64::MAX),
        ] {
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("tag")
            };
            let name = tag.name.value.text().backing().as_ptr();
            let mut input = Some(tag);
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let remaining = remaining.map(|value| {
                if value >= u64::MAX - 1 {
                    required.unwrap() - (u64::MAX - value)
                } else {
                    value
                }
            });
            if let Some(remaining) = remaining {
                let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
                c.charge(
                    WorkCategory::StateStep,
                    limits().work_credits() - used - remaining,
                )
                .unwrap();
            }
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let result = session.recover_br_end_tag(&mut c, &mut input);
            assert_eq!(
                input.as_ref().unwrap().name.value.text().backing().as_ptr(),
                name
            );
            if remaining.is_none() {
                result.unwrap();
                required =
                    Some(c.usage().resource(HtmlParseResource::WorkCredits).current - before);
            } else if remaining == required {
                result.unwrap();
                assert_eq!(input.as_ref().unwrap().kind, TagKind::Start);
                assert_eq!(input.as_ref().unwrap().attributes.len(), 0);
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert_eq!(input.as_ref().unwrap().kind, TagKind::End);
                if remaining == required.map(|work| work - 1) {
                    // Final empty pop observes the first stop after both attrs
                    // were disposed. Empty lexical capacity still gets released.
                    assert_eq!(input.as_ref().unwrap().attributes.len(), 0);
                    assert_eq!(
                        c.usage()
                            .resource(HtmlParseResource::TokenAttributes)
                            .current,
                        0
                    );
                } else if remaining.is_some_and(|credits| credits <= 1) {
                    assert_eq!(input.as_ref().unwrap().attributes.len(), 2);
                } else {
                    // An intermediate stop may precede the slot transfer or
                    // occur in its complete terminal disposal. It cannot leave
                    // one partially disposed attribute in the lexical owner.
                    let attributes = input.as_ref().unwrap().attributes.len();
                    assert!(matches!(attributes, 0 | 2));
                    assert_eq!(
                        c.usage()
                            .resource(HtmlParseResource::TokenAttributes)
                            .current,
                        attributes as u64
                    );
                }
                let stop = c.stop().unwrap();
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            }
            let stop = c.stop();
            let mut kind = input.take().map(TokenKind::Tag);
            if release_token_kind(&mut kind, &mut c).is_err() {
                release_token_kind(&mut kind, &mut c).unwrap();
            }
            tokenizer.release(&mut c).unwrap();
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            if stop.is_some() {
                assert_eq!(c.stop(), stop);
            }
        }
    }
    #[test]
    fn body_merge_qualifies_both_actual_local_names_and_preserves_import_refusal() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<body a=x>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let original = input.as_ref().unwrap().name.value.text().backing().as_ptr();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        let body = session.allocate_implicit_body(&mut c, None).unwrap();
        assert_eq!(
            session.add_body_attributes_if_missing(&mut c, &html, &mut input),
            Err(SessionError::WrongToken)
        );
        assert_eq!(
            session.add_attributes_if_missing(&mut c, &body, &mut input),
            Err(SessionError::WrongToken)
        );
        assert_eq!(
            input.as_ref().unwrap().name.value.text().backing().as_ptr(),
            original
        );
        let imported = session
            .allocate_element(&mut c, super::tests::html("body"))
            .unwrap();
        assert_eq!(
            session.active().unwrap().standalone_ineligibility,
            Some(StandaloneUnsupported::Provenance)
        );
        assert_eq!(
            session.add_body_attributes_if_missing(&mut c, &imported, &mut input),
            Err(SessionError::AttributeStorageUnavailable)
        );
        assert_eq!(
            input.as_ref().unwrap().name.value.text().backing().as_ptr(),
            original
        );
        session.allocate_implicit_p(&mut c, None).unwrap();
        session
            .add_body_attributes_if_missing(&mut c, &body, &mut input)
            .unwrap();
        assert_eq!(
            session.active().unwrap().standalone_ineligibility,
            Some(StandaloneUnsupported::Provenance)
        );
        assert!(input.is_none());
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn body_merge_actual_attribute_cap_preserves_old_order_and_reclaims_staged_payloads() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<body id=old><body a=one b=two>");
        for cap in [2, 3] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .managed_attributes(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("tag")
            };
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let body = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
            let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("tag")
            };
            let mut input = Some(second);
            let result = session.add_body_attributes_if_missing(&mut c, &body, &mut input);
            let HtmlNodeKind::Element(element) =
                &session.active().unwrap().records[body.index].kind
            else {
                panic!("body")
            };
            assert_eq!(
                element.attributes().as_slice()[0]
                    .name()
                    .local_name()
                    .code_units(),
                [105, 100]
            );
            assert_eq!(
                element.attributes().as_slice()[0].value().code_units(),
                [111, 108, 100]
            );
            if cap == 3 {
                result.unwrap();
                assert_eq!(element.attributes().len(), 3);
                assert!(input.is_none());
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedAttributes,
                        ..
                    }))
                ));
                assert_eq!(element.attributes().len(), 1);
                let stop = c.stop().unwrap();
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            }
            if let Some(tag) = input.take() {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn body_partition_wide_alternation_classifies_each_actual_unit_once() {
        let source = crate::HtmlSource::utf8(format!("<a v='{}'>", "A ".repeat(256)));
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(2000)
                .source_map_entries(2000)
                .work_credits(100_000_000)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let mut input = Some(body_lexical_value(&source, &mut c));
        let reads = c.usage().work(WorkCategory::InspectByte);
        let mut runs = partition_body_text(&mut c, &mut input).unwrap();
        assert_eq!(c.usage().work(WorkCategory::InspectByte) - reads, 1024);
        for index in 0..512 {
            let run = runs.take_next(&mut c).unwrap().unwrap();
            assert_eq!(
                run.kind,
                if index % 2 == 0 {
                    BodyTextKind::Other
                } else {
                    BodyTextKind::Whitespace
                }
            );
            assert_eq!(
                run.text.value.text().code_units(),
                [if index % 2 == 0 { 65 } else { 32 }]
            );
            assert_eq!(
                (
                    run.text.origins.as_slice()[0].source.start(),
                    run.text.origins.as_slice()[0].source.end()
                ),
                (index + 6, index + 7)
            );
            run.text.release(&mut c).unwrap();
        }
        assert!(runs.take_next(&mut c).unwrap().is_none());
        runs.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn body_partition_actual_origin_and_run_slots_require_their_coexisting_caps() {
        let source = crate::HtmlSource::utf8("<a v='A B C D'>");
        // Root child-storage plus four slots each for real html/body/p owners
        // coexist with seven output runs. These caps also admit the actual
        // lexer's seventeen-node attribute trie and its returned token record.
        for (maps, auxiliary, succeeds) in [(13, 20, false), (14, 19, false), (14, 20, true)] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .source_map_entries(maps)
                    .auxiliary_entries(auxiliary)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            let mut input = Some(body_lexical_value(&source, &mut c));
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let html = session.allocate_implicit_html(&mut c).unwrap();
            let body = session.allocate_implicit_body(&mut c, None).unwrap();
            let p = session.allocate_implicit_p(&mut c, None).unwrap();
            let root = session.root(&mut c).unwrap();
            session.insert(&mut c, &root, &html, None, None).unwrap();
            session.insert(&mut c, &html, &body, None, None).unwrap();
            session.insert(&mut c, &body, &p, None, None).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                13
            );
            let capacity = c.usage().actual_capacity_bytes;
            let result = partition_body_text(&mut c, &mut input);
            assert_eq!(result.is_ok(), succeeds);
            if let Ok(mut runs) = result {
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .high_water,
                    14
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .high_water,
                    20
                );
                runs.release(&mut c).unwrap();
            } else {
                assert_eq!(
                    input.as_ref().unwrap().value.text().code_units(),
                    [65, 32, 66, 32, 67, 32, 68]
                );
                assert_eq!(c.usage().actual_capacity_bytes, capacity);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current,
                    7
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    13
                );
                release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
            }
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn body_runs_last_cleanup_stop_reclaims_all_storage_and_exposes_completed_ownership() {
        let source = crate::HtmlSource::utf8("<a v='A B'>");
        let mut probe = control();
        let mut input = Some(body_lexical_value(&source, &mut probe));
        let mut runs = partition_body_text(&mut probe, &mut input).unwrap();
        let before = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        runs.release(&mut probe).unwrap();
        let release_work = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        let mut c = control();
        let mut input = Some(body_lexical_value(&source, &mut c));
        let mut runs = partition_body_text(&mut c, &mut input).unwrap();
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            WorkCategory::StateStep,
            limits().work_credits() - used - release_work + 1,
        )
        .unwrap();
        assert!(matches!(
            runs.release(&mut c),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        assert!(runs.entries.is_none());
        runs.release(&mut c).unwrap();
        storage_zero(&c);
        assert!(c.stop().is_some());
    }
    #[test]
    fn first_lf_work_and_unit_stops_keep_original_input_and_release_every_staged_owner() {
        let source = crate::HtmlSource::utf8("<a v='\nA B'>");
        let mut probe = control();
        let mut input = Some(body_lexical_value(&source, &mut probe));
        let before = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let prefix = split_first_lf(&mut probe, &mut input).unwrap().unwrap();
        let required = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        prefix.release(&mut probe).unwrap();
        input.take().unwrap().release(&mut probe).unwrap();
        storage_zero(&probe);
        for remaining in [0, 1, required / 2, required - 1, required] {
            let mut c = control();
            let mut input = Some(body_lexical_value(&source, &mut c));
            let original = input.as_ref().unwrap().value.text().backing().as_ptr();
            let capacity = c.usage().actual_capacity_bytes;
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - remaining,
            )
            .unwrap();
            let result = split_first_lf(&mut c, &mut input);
            assert_eq!(result.is_ok(), remaining == required);
            if let Ok(Some(prefix)) = result {
                prefix.release(&mut c).unwrap();
                input.take().unwrap().release(&mut c).unwrap();
            } else {
                assert_eq!(
                    input.as_ref().unwrap().value.text().backing().as_ptr(),
                    original
                );
                assert_eq!(c.usage().actual_capacity_bytes, capacity);
                release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
            }
            storage_zero(&c);
        }
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(7)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let mut input = Some(body_lexical_value(&source, &mut c));
        assert!(matches!(
            split_first_lf(&mut c, &mut input),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::BufferedUtf16Units,
                ..
            }))
        ));
        assert_eq!(
            input.as_ref().unwrap().value.text().code_units(),
            [10, 65, 32, 66]
        );
        release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn recovered_image_keeps_actual_attributes_and_exact_original_name_span() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::OriginKind};
        let source = crate::HtmlSource::utf8("<ImAgE id=x />");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let value = input.as_ref().unwrap().attributes.as_slice()[0]
            .value
            .value
            .text()
            .backing()
            .as_ptr();
        let self_closing = input.as_ref().unwrap().self_closing;
        let duplicate = input.as_ref().unwrap().had_duplicate_attributes;
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let reads = c.usage().work(WorkCategory::InputUnitRead);
        session
            .recover_image_tag(&mut c, &source, &mut input)
            .unwrap();
        assert_eq!(c.usage().work(WorkCategory::InputUnitRead) - reads, 2);
        let tag = input.as_ref().unwrap();
        assert_eq!(tag.name.value.text().code_units(), [105, 109, 103]);
        assert_eq!(
            (tag.kind, tag.self_closing, tag.had_duplicate_attributes),
            (TagKind::Start, self_closing, duplicate)
        );
        assert_eq!(
            tag.attributes.as_slice()[0]
                .value
                .value
                .text()
                .backing()
                .as_ptr(),
            value
        );
        let origin = &tag.name.origins.as_slice()[0];
        assert_eq!(
            (
                origin.start,
                origin.end,
                origin.source.start(),
                origin.source.end(),
                origin.kind
            ),
            (0, 3, 1, 6, OriginKind::RecoverySyntax)
        );
        let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
        assert!(input.is_none());
        let root = session.root(&mut c).unwrap();
        session.insert(&mut c, &root, &node, None, None).unwrap();
        assert!(session.active().unwrap().standalone_ineligibility.is_none());
        tokenizer.release(&mut c).unwrap();
        let mut arena = session.freeze(&mut c).unwrap();
        assert_eq!(locals(arena.fragment().unwrap()), ["img"]);
        arena.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn recovered_image_literal_coexistence_cap_preserves_old_backing_before_commit() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<image id=x>");
        // Actual incoming image(5), id(2), x(1), and tokenizer last-start(5)
        // occupy thirteen units; the replacement requires three more live.
        for cap in [15, 16] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .buffered_utf16_units(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("tag")
            };
            let mut input = Some(tag);
            let original = input.as_ref().unwrap().name.value.text().backing().as_ptr();
            let bytes = c.usage().actual_capacity_bytes;
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let before = c.usage().actual_capacity_bytes;
            let result = session.recover_image_tag(&mut c, &source, &mut input);
            if cap == 16 {
                result.unwrap();
                assert_eq!(
                    input.as_ref().unwrap().name.value.text().code_units(),
                    [105, 109, 103]
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .high_water,
                    16
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::BufferedUtf16Units,
                        ..
                    }))
                ));
                assert_eq!(
                    input.as_ref().unwrap().name.value.text().backing().as_ptr(),
                    original
                );
                assert_eq!(c.usage().actual_capacity_bytes, before);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    13
                );
            }
            assert!(before >= bytes);
            release_token_kind(&mut input.take().map(TokenKind::Tag), &mut c).unwrap();
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn image_alias_foreign_and_cancellation_leave_actual_name_and_attributes_in_input() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<image id=x>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let alias = input.as_ref().unwrap().attributes.as_slice()[0]
            .value
            .value
            .text()
            .clone();
        assert_eq!(
            session.recover_image_tag(&mut c, &source, &mut input),
            Err(SessionError::AliasedPayload)
        );
        assert_eq!(
            input.as_ref().unwrap().name.value.text().code_units(),
            [105, 109, 97, 103, 101]
        );
        drop(alias);
        let mut foreign = control();
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            session.recover_image_tag(&mut foreign, &source, &mut input),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert!(session.active().unwrap().standalone_ineligibility.is_none());
        cancellation.cancel();
        assert_eq!(
            session.recover_image_tag(&mut c, &source, &mut input),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
        release_token_kind(&mut input.take().map(TokenKind::Tag), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn image_work_exact_and_partial_stops_preserve_or_commit_one_actual_name_owner() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<image id=x>");
        let mut required = None;
        for remaining in [
            None,
            Some(0),
            Some(1),
            Some(100),
            Some(u64::MAX - 1),
            Some(u64::MAX),
        ] {
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("tag")
            };
            let mut input = Some(tag);
            let original = input.as_ref().unwrap().name.value.text().backing().as_ptr();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let bytes = c.usage().actual_capacity_bytes;
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let remaining = remaining.map(|value| {
                if value >= u64::MAX - 1 {
                    required.unwrap() - (u64::MAX - value)
                } else {
                    value
                }
            });
            if let Some(remaining) = remaining {
                c.charge(
                    WorkCategory::StateStep,
                    limits().work_credits() - before - remaining,
                )
                .unwrap();
            }
            let start = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let result = session.recover_image_tag(&mut c, &source, &mut input);
            if remaining.is_none() {
                result.unwrap();
                required = Some(c.usage().resource(HtmlParseResource::WorkCredits).current - start);
            } else if remaining == required {
                result.unwrap();
                assert_eq!(
                    input.as_ref().unwrap().name.value.text().code_units(),
                    [105, 109, 103]
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert_eq!(
                    input.as_ref().unwrap().name.value.text().backing().as_ptr(),
                    original
                );
                assert_eq!(c.usage().actual_capacity_bytes, bytes);
            }
            let stop = c.stop();
            let mut token = input.take().map(TokenKind::Tag);
            if release_token_kind(&mut token, &mut c).is_err() {
                release_token_kind(&mut token, &mut c).unwrap();
            }
            tokenizer.release(&mut c).unwrap();
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            if stop.is_some() {
                assert_eq!(c.stop(), stop);
            }
        }
    }
    #[test]
    fn body_runs_keep_exact_utf16_kinds_order_and_newline_reference_origins() {
        use crate::parser::tokenizer::token::OriginKind;
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<a v='A \r\n&#x20;"
                .encode_utf16()
                .chain([0xd83d, 0xde00, 0xd800, 0xfeff, 0x0b])
                .chain(" B'>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        let mut input = Some(body_lexical_value(&source, &mut c));
        let len = input.as_ref().unwrap().value.text().len();
        let reads = c.usage().work(WorkCategory::InspectByte);
        let mut runs = partition_body_text(&mut c, &mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(
            c.usage().work(WorkCategory::InspectByte) - reads,
            2 * len as u64
        );
        let expected = [
            (BodyTextKind::Other, vec![65]),
            (BodyTextKind::Whitespace, vec![32, 10, 32]),
            (
                BodyTextKind::Other,
                vec![0xd83d, 0xde00, 0xd800, 0xfeff, 0x0b],
            ),
            (BodyTextKind::Whitespace, vec![32]),
            (BodyTextKind::Other, vec![66]),
        ];
        for (kind, units) in expected {
            let run = runs.take_next(&mut c).unwrap().unwrap();
            assert_eq!(run.kind, kind);
            assert_eq!(run.text.value.text().code_units(), units);
            for origin in run.text.origins.as_slice() {
                assert_eq!(origin.source.source(), source.identity());
            }
            if kind == BodyTextKind::Whitespace && units.len() == 3 {
                let origins = run.text.origins.as_slice();
                assert_eq!(
                    (
                        origins[1].start,
                        origins[1].end,
                        origins[1].source.start(),
                        origins[1].source.end(),
                        origins[1].kind
                    ),
                    (1, 2, 8, 10, OriginKind::Newline)
                );
                assert_eq!(
                    (
                        origins[2].start,
                        origins[2].end,
                        origins[2].source.start(),
                        origins[2].source.end(),
                        origins[2].kind
                    ),
                    (2, 3, 10, 16, OriginKind::CharacterReference)
                );
            }
            run.text.release(&mut c).unwrap();
        }
        assert!(runs.take_next(&mut c).unwrap().is_none());
        runs.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn body_null_run_retains_the_actual_data_null_source() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("\0");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Text(text) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("text")
        };
        let mut input = Some(text);
        let mut runs = partition_body_text(&mut c, &mut input).unwrap();
        let run = runs.take_next(&mut c).unwrap().unwrap();
        assert_eq!(run.kind, BodyTextKind::Null);
        assert_eq!(run.text.value.text().code_units(), [0]);
        assert_eq!(
            (
                run.text.origins.as_slice()[0].source.start(),
                run.text.origins.as_slice()[0].source.end()
            ),
            (0, 1)
        );
        run.text.release(&mut c).unwrap();
        runs.release(&mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn body_partition_full_coexisting_units_exact_and_one_short_preserve_input_on_failure() {
        let source = crate::HtmlSource::utf8("<a v='A B '>");
        for cap in [7, 8] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .buffered_utf16_units(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            let mut input = Some(body_lexical_value(&source, &mut c));
            let bytes = c.usage().actual_capacity_bytes;
            let result = partition_body_text(&mut c, &mut input);
            if cap == 8 {
                let mut runs = result.unwrap();
                assert!(input.is_none());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .high_water,
                    8
                );
                runs.release(&mut c).unwrap();
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::BufferedUtf16Units,
                        ..
                    }))
                ));
                assert_eq!(
                    input.as_ref().unwrap().value.text().code_units(),
                    [65, 32, 66, 32]
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    4
                );
                assert_eq!(c.usage().actual_capacity_bytes, bytes);
                release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
            }
            storage_zero(&c);
        }
    }
    #[test]
    fn body_partition_alias_foreign_and_cancel_preflight_preserve_original_owner() {
        let source = crate::HtmlSource::utf8("<a v='A B'>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut input = Some(body_lexical_value(&source, &mut c));
        let alias = input.as_ref().unwrap().value.text().clone();
        assert!(matches!(
            partition_body_text(&mut c, &mut input),
            Err(SessionError::AliasedPayload)
        ));
        assert!(input.is_some());
        drop(alias);
        let mut foreign = control();
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            partition_body_text(&mut foreign, &mut input),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert!(input.is_some());
        cancellation.cancel();
        assert!(matches!(
            partition_body_text(&mut c, &mut input),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert_eq!(
            input.as_ref().unwrap().value.text().code_units(),
            [65, 32, 66]
        );
        release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn body_partition_work_boundaries_cleanup_all_staged_real_owners() {
        let source = crate::HtmlSource::utf8("<a v='A B C'>");
        let mut probe = control();
        let mut input = Some(body_lexical_value(&source, &mut probe));
        let before = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut runs = partition_body_text(&mut probe, &mut input).unwrap();
        let required = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        runs.release(&mut probe).unwrap();
        storage_zero(&probe);
        for remaining in [0, 1, required / 3, required * 2 / 3, required - 1, required] {
            let mut c = control();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let mut input = Some(body_lexical_value(&source, &mut c));
            let bytes = c.usage().actual_capacity_bytes;
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - remaining,
            )
            .unwrap();
            let result = partition_body_text(&mut c, &mut input);
            assert_eq!(result.is_ok(), remaining == required);
            if let Ok(mut runs) = result {
                assert!(input.is_none());
                discard_runs(&mut runs, &mut c).unwrap();
            } else {
                assert!(input.is_some());
                assert_eq!(c.usage().actual_capacity_bytes, bytes);
                let stop = c.stop().unwrap();
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
                release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
                assert_eq!(c.stop(), Some(stop));
            }
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
        }
    }
    #[test]
    fn body_runs_partial_take_cancel_and_alias_disposal_retain_actual_slots() {
        let source = crate::HtmlSource::utf8("<a v='A B'>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut input = Some(body_lexical_value(&source, &mut c));
        let mut runs = partition_body_text(&mut c, &mut input).unwrap();
        let first = runs.take_next(&mut c).unwrap().unwrap();
        let alias = runs.entries.as_ref().unwrap().as_slice()[1]
            .as_ref()
            .unwrap()
            .text
            .value
            .text()
            .clone();
        assert_eq!(runs.release(&mut c), Err(SessionError::AliasedPayload));
        assert!(runs.entries.is_some());
        drop(alias);
        cancellation.cancel();
        assert!(matches!(
            runs.take_next(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        first.text.release(&mut c).unwrap();
        runs.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }
    #[test]
    fn first_lf_retains_crlf_and_character_reference_sources_and_exact_suffix() {
        use crate::parser::tokenizer::token::OriginKind;
        for (literal, end, kind) in [
            ("\r\nA B", 8, OriginKind::Newline),
            ("&#10;A B", 11, OriginKind::CharacterReference),
        ] {
            let source = crate::HtmlSource::utf8(format!("<a v='{literal}'>"));
            let mut c = control();
            let mut input = Some(body_lexical_value(&source, &mut c));
            let prefix = split_first_lf(&mut c, &mut input).unwrap().unwrap();
            assert_eq!(prefix.value.text().code_units(), [10]);
            let origin = &prefix.origins.as_slice()[0];
            assert_eq!(
                (
                    origin.start,
                    origin.end,
                    origin.source.start(),
                    origin.source.end(),
                    origin.kind
                ),
                (0, 1, 6, end, kind)
            );
            assert_eq!(
                input.as_ref().unwrap().value.text().code_units(),
                [65, 32, 66]
            );
            assert_eq!(
                input.as_ref().unwrap().origins.as_slice()[0].source.start(),
                end
            );
            prefix.release(&mut c).unwrap();
            input.take().unwrap().release(&mut c).unwrap();
            storage_zero(&c);
        }
        let source = crate::HtmlSource::utf8("<a v='\n'>");
        let mut c = control();
        let mut input = Some(body_lexical_value(&source, &mut c));
        let original = input.as_ref().unwrap().value.text().backing().as_ptr();
        let prefix = split_first_lf(&mut c, &mut input).unwrap().unwrap();
        assert!(input.is_none());
        assert_eq!(prefix.value.text().backing().as_ptr(), original);
        prefix.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn body_and_p_literals_and_both_body_merge_paths_freeze_same_native_arena() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for implied in [false, true] {
            let source =
                crate::HtmlSource::utf8("<body id=old><body id=ignored x:id=p a=one a=last z=end>");
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
                panic!("tag")
            };
            let body = if implied {
                let body = session.allocate_implicit_body(&mut c, None).unwrap();
                session
                    .add_body_attributes_if_missing(&mut c, &body, &mut Some(first))
                    .unwrap();
                body
            } else {
                session.allocate_html_tag(&mut c, &mut Some(first)).unwrap()
            };
            let root = session.root(&mut c).unwrap();
            session.insert(&mut c, &root, &body, None, None).unwrap();
            let p = session.allocate_implicit_p(&mut c, None).unwrap();
            session.insert(&mut c, &body, &p, None, None).unwrap();
            let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
                panic!("tag")
            };
            session
                .add_body_attributes_if_missing(&mut c, &body, &mut Some(second))
                .unwrap();
            let storage = session.active().unwrap();
            assert!(storage.standalone_ineligibility.is_none());
            let HtmlNodeKind::Element(element) = &storage.records[body.index].kind else {
                panic!("body")
            };
            let attributes = element.attributes().as_slice();
            assert_eq!(
                attributes
                    .iter()
                    .map(|a| (
                        a.name().local_name().to_utf8().unwrap(),
                        a.value().to_utf8().unwrap()
                    ))
                    .collect::<Vec<_>>(),
                [("id", "old"), ("x:id", "p"), ("a", "one"), ("z", "end")]
                    .map(|(a, b)| (a.to_owned(), b.to_owned()))
            );
            assert!(
                attributes
                    .iter()
                    .all(|a| a.name().namespace().uri().is_none())
            );
            t.release(&mut c).unwrap();
            let mut arena = session.freeze(&mut c).unwrap();
            let fragment = arena.fragment().unwrap();
            assert_eq!(locals(fragment), ["body"]);
            let body_id = fragment.root().children().next().unwrap();
            let p_id = fragment.node(&body_id).unwrap().children().next().unwrap();
            let HtmlNodeKind::Element(p) = fragment.node(&p_id).unwrap().kind() else {
                panic!("p")
            };
            assert_eq!(p.name().local_name().code_units(), [112]);
            arena.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    fn hidden_owner<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> (
        ParseControl,
        ParserArenaSession,
        crate::parser::tree_builder::ConstructionCapability<'a>,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
        ParserNode,
        Option<Tag>,
        crate::HtmlSourceRange,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let (mut c, session, capability) = fresh_script_owner(request);
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let (tag, source) = loop {
            match lexer.next(&mut c, false).unwrap().unwrap() {
                crate::parser::tokenizer::token::Event::Token(event) => {
                    let token = event.into_token(&mut c).unwrap();
                    if matches!(&token.kind, TokenKind::Tag(tag) if tag.name.value.text().code_units()==[105,110,112,117,116])
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
        let parent = session
            .active()
            .unwrap()
            .node(&mut c, session.active().unwrap().records[0].children[0])
            .unwrap();
        (c, session, capability, lexer, parent, tag, source)
    }
    #[test]
    fn hidden_input_ordered_initialization_reset_and_frozen_values_are_independent() {
        for (attrs, expected, rewritten) in [
            (
                "value='A&#13;B&#10;C' type=hidden checked",
                vec![65, 66, 67],
                true,
            ),
            (
                "type=HiDdEn value='A&#13;B&#10;C' checked",
                vec![65, 13, 66, 10, 67],
                false,
            ),
            ("value='&#13;&#10;' type=hidden", vec![13, 10], false),
            (
                "value='A&#13;B' type=hidden value=ignored type=text",
                vec![65, 66],
                true,
            ),
            ("type=hidden", vec![], false),
        ] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!("<html><head></head><input {attrs}>")),
                limits(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
                hidden_owner(&request);
            let original_value_source = input
                .as_ref()
                .unwrap()
                .attributes
                .as_slice()
                .iter()
                .find(|a| a.name.value.text().code_units() == [118, 97, 108, 117, 101])
                .map(|a| a.source.clone());
            let node = session
                .allocate_hidden_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::parser_session::CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: crate::construction::parser_session::CreationContext::Ordinary,
                    },
                    &token_source,
                    &capability,
                )
                .unwrap();
            assert!(input.is_none());
            match session
                .hidden_input_writeback_sources(&mut c, &node)
                .unwrap()
            {
                Some((token, value)) => {
                    assert!(rewritten);
                    assert_eq!(token, &token_source);
                    assert_eq!(Some(value), original_value_source.as_ref());
                    assert_eq!(token.coordinates(), crate::HtmlSourceCoordinates::Utf8Bytes);
                }
                None => assert!(!rewritten),
            }
            let state = session.hidden_input_state(&mut c, &node).unwrap();
            assert!(
                !state.dirty_value
                    && !state.dirty_checkedness
                    && !state.indeterminate
                    && !state.user_validity
            );
            assert!(state.selected_files_empty);
            assert_eq!(state.checkedness, attrs.contains("checked"));
            assert_eq!(state.value_rewritten_during_creation, rewritten);
            let association = session.form_association_state(&mut c, &node).unwrap();
            assert_eq!(association.kind, FormAssociationKind::HiddenInput);
            assert!(
                session
                    .form_association_owner(&mut c, &node)
                    .unwrap()
                    .is_none()
            );
            let mut value = Utf16Buffer::new(&mut c).unwrap();
            session
                .append_hidden_input_internal_value(&mut c, &node, &mut value)
                .unwrap();
            assert_eq!(value.units(), expected);
            value.clear(&mut c).unwrap();
            session
                .append_hidden_input_value(&mut c, &node, &mut value)
                .unwrap();
            assert_eq!(value.units(), expected);
            value.release(&mut c).unwrap();
            let storage = session.active().unwrap();
            let raw = find_hidden_input_state(&storage.payloads, &mut c, node.index)
                .unwrap()
                .0;
            let RetainedPayload::String {
                value: internal, ..
            } = &storage.payloads.values[raw]
            else {
                unreachable!()
            };
            if let Some(native) = storage.form_attribute(&mut c, node.index, "value").unwrap() {
                assert!(!Arc::ptr_eq(native.backing(), internal.backing()));
            }
            assert!(storage.payloads.values.iter().any(|p| matches!(
                p,
                RetainedPayload::HistoricalString { .. } | RetainedPayload::Retired
            )));
            if rewritten {
                assert!(storage.payloads.values.iter().any(|p| matches!(p, RetainedPayload::String { value, origins:Some(_), location:StringLocation::Detached, .. } if value.code_units() == [65,13,66,10,67] || value.code_units() == [65,13,66])));
            }
            session.insert(&mut c, &parent, &node, None, None).unwrap();
            lexer.release(&mut c).unwrap();
            let mut frozen = session.freeze(&mut c).unwrap();
            let document = frozen.document().unwrap();
            let html = document.root().children().next().unwrap();
            let id = document.node(&html).unwrap().children().last().unwrap();
            assert_eq!(frozen.hidden_input_state(&mut c, &id).unwrap(), state);
            match frozen.hidden_input_writeback_sources(&mut c, &id).unwrap() {
                Some((token, value)) => {
                    assert!(rewritten);
                    assert_eq!(token, &token_source);
                    assert_eq!(Some(value), original_value_source.as_ref());
                }
                None => assert!(!rewritten),
            }
            let mut value = Utf16Buffer::new(&mut c).unwrap();
            frozen
                .append_hidden_input_internal_value(&mut c, &id, &mut value)
                .unwrap();
            assert_eq!(value.units(), expected);
            value.clear(&mut c).unwrap();
            frozen
                .append_hidden_input_value(&mut c, &id, &mut value)
                .unwrap();
            assert_eq!(value.units(), expected);
            value.release(&mut c).unwrap();
            drop(id);
            drop(html);
            frozen.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn hidden_input_sanitization_preserves_exact_utf16_origin_groups_and_original_evidence() {
        let mut units: Vec<u16> = "<html><head></head><input value='".encode_utf16().collect();
        units.extend([0xd83d, 0xde00, 0xd800, 65]);
        units.extend("&#13;B&#10;C' type=hidden>".encode_utf16());
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            hidden_owner(&request);
        let original_value_source = input.as_ref().unwrap().attributes.as_slice()[0]
            .source
            .clone();
        let expected = [0xd83d, 0xde00, 0xd800, 65, 66, 67];
        let node = session
            .allocate_hidden_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::parser_session::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap();
        let (token, value) = session
            .hidden_input_writeback_sources(&mut c, &node)
            .unwrap()
            .unwrap();
        assert_eq!(token, &token_source);
        assert_eq!(value, &original_value_source);
        assert_eq!(
            token.coordinates(),
            crate::HtmlSourceCoordinates::Utf16CodeUnits
        );
        let source_slot = session
            .active()
            .unwrap()
            .attribute_value_slot(&mut c, node.index, 0)
            .unwrap();
        let RetainedPayload::String {
            value,
            origins: Some(origins),
            ..
        } = &session.active().unwrap().payloads.values[source_slot]
        else {
            unreachable!()
        };
        assert_eq!(value.code_units(), expected);
        assert_eq!(
            session
                .active()
                .unwrap()
                .observed_origins(origins, &mut c)
                .unwrap()
                .iter()
                .map(|o| (o.start, o.end))
                .collect::<Vec<_>>(),
            [(0, 2), (2, 3), (3, 4), (4, 5), (5, 6)]
        );
        assert!(
            session
                .active()
                .unwrap()
                .observed_origins(origins, &mut c)
                .unwrap()
                .iter()
                .all(|o| o.kind != crate::parser::tokenizer::token::OriginKind::RecoverySyntax)
        );
        let originals = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|p| match p {
                RetainedPayload::String {
                    value,
                    origins: Some(origins),
                    location: StringLocation::Detached,
                    ..
                } if value.code_units() == [0xd83d, 0xde00, 0xd800, 65, 13, 66, 10, 67] => Some(
                    session
                        .active()
                        .unwrap()
                        .observed_origins(origins, &mut c)
                        .unwrap(),
                ),
                _ => None,
            })
            .unwrap();
        assert_eq!(originals.len(), 7);
        assert_eq!(
            originals[4].source,
            session
                .active()
                .unwrap()
                .observed_origins(origins, &mut c)
                .unwrap()[3]
                .source
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn hidden_input_foreign_proof_alias_and_nonhidden_refuse_before_token_transfer() {
        for case in 0..3 {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head></head><input type=hidden value=x>"),
                limits(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
                hidden_owner(&request);
            if case == 0 {
                let alias = input.as_ref().unwrap().attributes.as_slice()[1]
                    .value
                    .value
                    .text()
                    .clone();
                assert_eq!(
                    session
                        .allocate_hidden_input_for_creation(
                            &mut c,
                            &mut input,
                            crate::construction::parser_session::CreationLocation {
                                intended_parent: &parent,
                                current_form: None,
                                context:
                                    crate::construction::parser_session::CreationContext::Ordinary
                            },
                            &token_source,
                            &capability
                        )
                        .err(),
                    Some(SessionError::AliasedPayload)
                );
                assert!(input.is_some());
                drop(alias);
            } else if case == 1 {
                let mut other =
                    ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                        .unwrap();
                let root = other.root(&mut c).unwrap();
                assert!(matches!(
                    other.allocate_hidden_input_for_creation(
                        &mut c,
                        &mut input,
                        crate::construction::parser_session::CreationLocation {
                            intended_parent: &root,
                            current_form: None,
                            context: crate::construction::parser_session::CreationContext::Ordinary
                        },
                        &token_source,
                        &capability
                    ),
                    Err(SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                        StandaloneUnsupported::Provenance
                    )))
                ));
                assert!(input.is_some());
                other.release(&mut c).unwrap();
            } else {
                let generic = session
                    .allocate_html_tag_for_creation(
                        &mut c,
                        &mut input,
                        &parent,
                        None,
                        crate::construction::parser_session::CreationContext::Ordinary,
                    )
                    .unwrap();
                assert_eq!(
                    session.hidden_input_state(&mut c, &generic),
                    Err(SessionError::HiddenInputMetadataUnavailable)
                );
                assert!(matches!(
                    session.hidden_input_writeback_sources(&mut c, &generic),
                    Err(SessionError::HiddenInputMetadataUnavailable)
                ));
            }
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=text>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            hidden_owner(&request);
        assert_eq!(
            session
                .allocate_hidden_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::parser_session::CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: crate::construction::parser_session::CreationContext::Ordinary
                    },
                    &token_source,
                    &capability
                )
                .err(),
            Some(SessionError::WrongToken)
        );
        assert!(input.is_some());
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn hidden_input_every_stopped_writeback_tail_retains_alias_valid_owners_and_first_stop() {
        let source = "<html><head></head><input value='A&#13;B&#10;C' type=hidden checked>";
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            hidden_owner(&request);
        session
            .allocate_hidden_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::parser_session::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap();
        let total = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        let mut partial = false;
        for tail in (1..=2500).step_by(25) {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source),
                limits()
                    .into_builder()
                    .work_credits(total - tail)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
                hidden_owner(&request);
            assert!(matches!(
                session.allocate_hidden_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::parser_session::CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: crate::construction::parser_session::CreationContext::Ordinary
                    },
                    &token_source,
                    &capability
                ),
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            let first = c.stop().unwrap();
            partial |= session.active().unwrap().payloads.values.iter().any(|p| matches!(p,RetainedPayload::HiddenInput { complete:false,state,.. } if state.value_rewritten_during_creation));
            assert!(
                matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
            );
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        }
        assert!(
            partial,
            "a real stopped writeback/reset prefix was exercised"
        );
    }

    #[test]
    fn hidden_input_raw_rewrite_coexistence_exact_104_units_and_one_short_reclaims_every_backing() {
        for cap in [103, 104] {
            // Fresh html=4+28. New input=5+28; source attribute names=5+4+7
            // and values=5+6+0. The retained lexical total is92. Sanitized raw,
            // replacement native value and a coexisting next raw each add3.
            // The first transformed ABC leaf remains reconstructible when its
            // raw slot is replaced; one later raw therefore adds3 over old101.
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(
                    "<html><head></head><input value='A&#13;B&#10;C' type=hidden checked>",
                ),
                limits()
                    .into_builder()
                    .managed_utf16_units(cap)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
                hidden_owner(&request);
            let result = session.allocate_hidden_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::parser_session::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            );
            if cap == 104 {
                let node = result.unwrap();
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .high_water,
                    104
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    101
                );
                assert_eq!(session.active().unwrap().usage.utf16_units, 90);
                assert!(
                    session
                        .hidden_input_state(&mut c, &node)
                        .unwrap()
                        .checkedness
                );
                assert_eq!(
                    session
                        .active()
                        .unwrap()
                        .payloads
                        .values
                        .iter()
                        .filter(|v| matches!(
                            v,
                            RetainedPayload::Retired | RetainedPayload::HistoricalString { .. }
                        ))
                        .count(),
                    2
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedUtf16Units,
                        ..
                    }))
                ));
                assert!(session.freeze(&mut c).is_err());
            }
            let stop = c.stop();
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }
    #[test]
    fn hidden_input_cancel_after_real_reset_precedes_parser_association() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><form><input value='A&#13;B' type=hidden checked>",
            ),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            hidden_owner(&request);
        let mut form_lexer =
            LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let mut form = Some(next_fixture_tag(
            &mut form_lexer,
            &mut c,
            &[102, 111, 114, 109],
        ));
        let form = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut form,
                &parent,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &parent, &form, None, None).unwrap();
        form_lexer.release(&mut c).unwrap();
        session.active_mut().unwrap().cancel_after_hidden_reset =
            Some(request.cancellation().clone());
        assert!(matches!(
            session.allocate_hidden_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &form,
                    current_form: Some(&form),
                    context: crate::construction::parser_session::CreationContext::Ordinary
                },
                &token_source,
                &capability
            ),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert!(input.is_none());
        let storage = session.active().unwrap();
        let (node, raw, state) = storage
            .payloads
            .values
            .iter()
            .find_map(|v| match v {
                RetainedPayload::HiddenInput {
                    node: Some(node),
                    raw,
                    complete: true,
                    state,
                    ..
                } => Some((*node, *raw, *state)),
                _ => None,
            })
            .unwrap();
        assert!(!state.dirty_value && !state.dirty_checkedness && !state.user_validity);
        assert!(state.checkedness && state.value_rewritten_during_creation);
        assert_eq!(
            textarea_raw(&storage.payloads, &mut c, raw).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
            unreachable!()
        };
        assert_eq!(value.code_units(), [65, 66]);
        let association = storage
            .payloads
            .values
            .iter()
            .find_map(|v| match v {
                RetainedPayload::FormAssociation {
                    node: Some(owner),
                    state,
                } if *owner == node => Some(*state),
                _ => None,
            })
            .unwrap();
        assert_eq!(association.owner, None);
        assert!(!association.parser_inserted);
        assert_eq!(association.evidence, FormAssociationEvidence::Created);
        assert!(storage.records[node].parent.is_none());
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn hidden_input_value_getters_qualify_destination_and_refuse_imported_and_cancelled_state() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=hidden value=x>"),
            limits(),
        );
        let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
            hidden_owner(&request);
        let node = session
            .allocate_hidden_input_for_creation(
                &mut c,
                &mut input,
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::parser_session::CreationContext::Ordinary,
                },
                &token_source,
                &capability,
            )
            .unwrap();
        let mut foreign = control();
        let mut buffer = Utf16Buffer::new(&mut foreign).unwrap();
        buffer.push_unit(81, &mut foreign).unwrap();
        assert!(matches!(
            session.append_hidden_input_value(&mut c, &node, &mut buffer),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert_eq!(buffer.units(), [81]);
        let first = c.stop();
        assert_eq!(
            session.hidden_input_state(&mut c, &node).err(),
            first.map(SessionError::Stop)
        );
        assert_eq!(session.freeze(&mut c).err(), first.map(SessionError::Stop));
        buffer.release(&mut foreign).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        storage_zero(&foreign);
    }

    pub(super) fn fresh_script_owner<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> (
        ParseControl,
        ParserArenaSession,
        crate::parser::tree_builder::ConstructionCapability<'a>,
    ) {
        use crate::parser::tree_builder::StandaloneDocumentDriver;
        let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
            request,
        ));
        for _ in 0..128 {
            let owner = driver.as_mut().unwrap();
            if owner.standalone_parts().is_some() {
                break;
            }
            owner.step().unwrap();
        }
        assert!(driver.as_mut().unwrap().standalone_parts().is_some());
        let parts = StandaloneDocumentDriver::take_standalone_fixture(&mut driver).unwrap();
        assert!(driver.is_none());
        parts
    }
    fn fresh_script<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
        connected: bool,
        text: bool,
    ) -> (
        ParseControl,
        ParserArenaSession,
        crate::parser::tree_builder::ConstructionCapability<'a>,
        ParserNode,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let (mut c, mut session, capability) = fresh_script_owner(request);
        let mut tokenizer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let mut incoming = loop {
            let kind = lexical_kind(&mut tokenizer, &mut c);
            if matches!(&kind, TokenKind::Tag(tag) if tag.name.value.text().code_units() == [0x73,0x63,0x72,0x69,0x70,0x74])
            {
                let TokenKind::Tag(tag) = kind else {
                    unreachable!()
                };
                break Some(tag);
            }
            assert!(!matches!(kind, TokenKind::Eof));
            release_token_kind(&mut Some(kind), &mut c).unwrap();
        };
        let node = session
            .allocate_disabled_parser_script(
                &mut c,
                &mut incoming,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
                &capability,
            )
            .unwrap();
        assert!(incoming.is_none());
        if connected {
            let html_index = session.active().unwrap().records[0].children[0];
            let html = session.active().unwrap().node(&mut c, html_index).unwrap();
            session.insert(&mut c, &html, &node, None, None).unwrap();
        }
        if text {
            let TokenKind::Text(value) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("literal text")
            };
            session
                .insert_text(&mut c, &node, None, &mut Some(value), None)
                .unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        (c, session, capability, node)
    }

    #[test]
    fn hidden_input_original_sources_reject_foreign_or_incomplete_token_before_take() {
        for case in 0..3 {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head></head><input value='A&#13;B' type=hidden>"),
                limits(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
                hidden_owner(&request);
            let wrong = if case == 1 {
                crate::HtmlSource::utf8("different").range(0, 9).unwrap()
            } else if case == 2 {
                crate::HtmlSource::utf16(crate::DocumentString::from("different"))
                    .range(0, 9)
                    .unwrap()
            } else {
                request
                    .source()
                    .range(token_source.start(), token_source.end() - 3)
                    .unwrap()
            };
            let slots = c
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current;
            let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
            let ptr = input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr();
            let error = session
                .allocate_hidden_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::parser_session::CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: crate::construction::parser_session::CreationContext::Ordinary,
                    },
                    &wrong,
                    &capability,
                )
                .unwrap_err();
            if case != 0 {
                assert_eq!(
                    error,
                    SessionError::Stop(ParseStop::InternalInvariant(
                        crate::parser::control::Invariant::ForeignSource
                    ))
                );
            } else {
                // The trailing token delimiter has no field mapping; narrowing
                // into the final type attribute must exclude its actual range.
                assert_eq!(error, SessionError::OriginSplitUnavailable);
            }
            assert_eq!(
                input
                    .as_ref()
                    .unwrap()
                    .name
                    .value
                    .text()
                    .code_units()
                    .as_ptr(),
                ptr
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                slots
            );
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                nodes
            );
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn hidden_input_original_ranges_charge_two_entries_before_transfer_and_cleanup_terminal_slots()
    {
        let source = "<html><head></head><input value='A&#13;B' type=hidden>";
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut c, mut session, _capability, lexer, _parent, mut input, _token_source) =
            hidden_owner(&request);
        let baseline = c
            .usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current;
        release_formatting_tag(&mut input, &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        // One actual initialized raw group now precedes the unchanged pair
        // of original token/value-attribute source ranges.
        for extra in [2, 3] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source),
                limits()
                    .into_builder()
                    .source_map_entries(baseline + extra)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability, lexer, parent, mut input, token_source) =
                hidden_owner(&request);
            let error = session
                .allocate_hidden_input_for_creation(
                    &mut c,
                    &mut input,
                    crate::construction::parser_session::CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: crate::construction::parser_session::CreationContext::Ordinary,
                    },
                    &token_source,
                    &capability,
                )
                .unwrap_err();
            assert!(matches!(
                error,
                SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::SourceMapEntries,
                    ..
                })
            ));
            if extra == 2 {
                assert!(input.is_some());
                assert!(
                    !session
                        .active()
                        .unwrap()
                        .payloads
                        .values
                        .iter()
                        .any(|p| matches!(p, RetainedPayload::HiddenInputSource { .. }))
                );
            } else {
                // The fixed two original ranges fit. Later actual text-origin
                // coexistence exhausts the cap, with the source record retained.
                assert!(input.is_none());
                assert!(session.active().unwrap().payloads.values.iter().any(|p| matches!(p, RetainedPayload::HiddenInputSource { token, value_attribute: Some(_), .. } if token==&token_source)));
            }
            let first = c.stop();
            assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(_))));
            release_formatting_tag(&mut input, &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), first);
            storage_zero(&c);
        }
    }

    fn next_fixture_tag(
        tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
        expected: &[u16],
    ) -> Tag {
        loop {
            let kind = lexical_kind(tokenizer, c);
            if matches!(&kind, TokenKind::Tag(tag) if tag.kind == TagKind::Start && tag.name.value.text().code_units() == expected)
            {
                let TokenKind::Tag(tag) = kind else {
                    unreachable!()
                };
                return tag;
            }
            assert!(!matches!(kind, TokenKind::Eof));
            release_token_kind(&mut Some(kind), c).unwrap();
        }
    }

    fn actual_template_and_div(
        source: &crate::HtmlSource,
        c: &mut ParseControl,
        session: &mut ParserArenaSession,
    ) -> (ParserNode, ParserNode) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let mut tokenizer = LexerFixture::new(source, State::Data, None, c).unwrap();
        let template = next_fixture_tag(
            &mut tokenizer,
            c,
            &[0x74, 0x65, 0x6d, 0x70, 0x6c, 0x61, 0x74, 0x65],
        );
        let template = session.allocate_html_tag(c, &mut Some(template)).unwrap();
        let contents = session.template_contents(c, &template).unwrap().unwrap();
        let div = next_fixture_tag(&mut tokenizer, c, &[0x64, 0x69, 0x76]);
        let div = session.allocate_html_tag(c, &mut Some(div)).unwrap();
        tokenizer.release(c).unwrap();
        (contents, div)
    }

    #[test]
    fn disabled_preparation_retains_actual_documents_and_uninitialized_result_through_freeze() {
        for (attributes, kind) in [
            ("src=''", ScriptType::Classic),
            ("src='' type=module async", ScriptType::Module),
            ("src='' type=importmap", ScriptType::ImportMap),
            ("src='' type=speculationrules", ScriptType::SpeculationRules),
        ] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!("<html><head><script {attributes}>")),
                limits(),
            );
            let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
            let initial = session.script_metadata(&mut c, &node).unwrap();
            assert!(!initial.force_async && !initial.already_started);
            assert_eq!(initial.script_type, None);
            let root = session.root(&mut c).unwrap();
            let parser_document = session
                .script_parser_document(&mut c, &node)
                .unwrap()
                .unwrap();
            assert!(session.same_node(&mut c, &root, &parser_document).unwrap());
            assert!(
                session
                    .script_preparation_document(&mut c, &node)
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                session
                    .prepare_disabled_script(&mut c, &node, &capability)
                    .unwrap(),
                DisabledScriptPreparation::Disabled(kind)
            );
            let prepared = session.script_metadata(&mut c, &node).unwrap();
            assert!(prepared.already_started && !prepared.force_async);
            assert_eq!(prepared.script_type, Some(kind));
            assert!(
                !prepared.external_file
                    && !prepared.ready_to_be_parser_executed
                    && !prepared.delaying_load
            );
            assert_eq!(prepared.result, ScriptResult::Uninitialized);
            assert!(prepared.steps_when_result_ready.is_none());
            let preparation_document = session
                .script_preparation_document(&mut c, &node)
                .unwrap()
                .unwrap();
            assert!(
                session
                    .same_node(&mut c, &root, &preparation_document)
                    .unwrap()
            );
            assert_eq!(
                session
                    .prepare_disabled_script(&mut c, &node, &capability)
                    .unwrap(),
                DisabledScriptPreparation::AlreadyStarted
            );
            let mut frozen = session.freeze(&mut c).unwrap();
            assert_eq!(frozen.script_metadata(&mut c, &node).unwrap(), prepared);
            let document = frozen.document().unwrap();
            assert_eq!(document.root().children().count(), 1);
            frozen.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn empty_disconnected_and_unknown_returns_keep_source_prefix_metadata_effects() {
        for (attributes, connected, text, outcome, force_async) in [
            ("", true, false, DisabledScriptPreparation::Empty, true),
            (
                "async",
                true,
                false,
                DisabledScriptPreparation::Empty,
                false,
            ),
            (
                "src=''",
                false,
                false,
                DisabledScriptPreparation::Disconnected,
                true,
            ),
            (
                "src='' type=' module '",
                true,
                false,
                DisabledScriptPreparation::UnrecognizedType,
                true,
            ),
        ] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!("<html><head><script {attributes}>")),
                limits(),
            );
            let (mut c, mut session, capability, node) = fresh_script(&request, connected, text);
            assert_eq!(
                session
                    .prepare_disabled_script(&mut c, &node, &capability)
                    .unwrap(),
                outcome
            );
            let metadata = session.script_metadata(&mut c, &node).unwrap();
            assert_eq!(metadata.force_async, force_async);
            assert!(!metadata.already_started);
            assert_eq!(metadata.script_type, None);
            assert!(
                session
                    .script_parser_document(&mut c, &node)
                    .unwrap()
                    .is_none()
            );
            assert!(
                session
                    .script_preparation_document(&mut c, &node)
                    .unwrap()
                    .is_none()
            );
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script>x"),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, true, true);
        assert_eq!(
            session
                .prepare_disabled_script(&mut c, &node, &capability)
                .unwrap(),
            DisabledScriptPreparation::Disabled(ScriptType::Classic)
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn script_eof_marks_started_without_preparation_and_cancelled_marker_has_no_effect() {
        for cancel in [false, true] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head><script>"),
                limits(),
            );
            let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
            let initial = session.script_metadata(&mut c, &node).unwrap();
            if cancel {
                request.cancellation().cancel();
            }
            let result = session.mark_script_eof(&mut c, &node, &capability);
            if cancel {
                assert_eq!(result, Err(SessionError::Stop(ParseStop::Cancelled)));
                let (_, retained) = session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .find_map(|value| {
                        if let RetainedPayload::Script { node, metadata } = value {
                            Some((*node, *metadata))
                        } else {
                            None
                        }
                    })
                    .unwrap();
                assert_eq!(retained, initial);
                assert!(matches!(
                    session.freeze(&mut c),
                    Err(SessionError::Stop(ParseStop::Cancelled))
                ));
            } else {
                result.unwrap();
                let marked = session.script_metadata(&mut c, &node).unwrap();
                assert!(marked.already_started && !marked.force_async);
                assert_eq!(marked.script_type, None);
                assert_eq!(marked.preparation_document, None);
                assert_eq!(marked.parser_document, initial.parser_document);
                assert_eq!(
                    session
                        .prepare_disabled_script(&mut c, &node, &capability)
                        .unwrap(),
                    DisabledScriptPreparation::AlreadyStarted
                );
            }
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn real_standalone_capability_cannot_be_reused_for_another_arena_on_the_same_control() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script>"),
            limits(),
        );
        let (mut c, mut original, capability, node) = fresh_script(&request, true, false);
        let mut unrelated =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance);
        let result = unrelated.mark_script_eof(&mut c, &node, &capability);
        assert_eq!(result, Err(SessionError::Stop(stop)));
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(unrelated.active().unwrap().records.len(), 1);
        assert!(original.active().unwrap().payloads.values.iter().any(|entry| matches!(entry, RetainedPayload::Script { metadata, .. } if !metadata.already_started)));
        assert!(
            matches!(unrelated.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
        );
        unrelated.release(&mut c).unwrap();
        original.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn preparation_clears_parser_document_before_ordinary_connectivity_work_can_stop() {
        let source = "<html><head><script src=x>";
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut baseline, mut session, capability, node) = fresh_script(&request, true, false);
        let setup_work = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        // Establish only the actual unchanged owner/node/metadata-read cost.
        // This performs no preparation or ordinary ancestry walk.
        capability
            .validate_session(&mut baseline, &session)
            .unwrap();
        let storage = session.active().unwrap();
        let index = storage.index(&mut baseline, &node).unwrap();
        let (_, initial) = script_metadata(&storage.payloads, &mut baseline, index).unwrap();
        assert_eq!(initial.parser_document, Some(DocumentRef::Owned(0)));
        let qualification_work = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - setup_work;
        // One metadata-slot visit, the actual fixed payload copy, and its write
        // are sufficient for HTML preparation step 3. The source connectivity
        // condition is step 7, after this required parser-document clear.
        let first_write_work = 1 + mem::size_of::<ScriptMetadata>() as u64 + 1;
        let cap = setup_work + qualification_work + first_write_work;
        session.release(&mut baseline).unwrap();
        storage_zero(&baseline);

        let limits = limits().into_builder().work_credits(cap).finish().unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits);
        let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            setup_work
        );
        let result = session.prepare_disabled_script(&mut c, &node, &capability);
        assert!(matches!(
            result,
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        let retained = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::Script { metadata, .. } => Some(*metadata),
                _ => None,
            })
            .unwrap();
        let first_stop = c.stop().unwrap();
        assert!(
            matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first_stop)
        );
        let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let high_water = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first_stop));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            stopped_work
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            high_water
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            3
        );
        assert_eq!(
            retained.parser_document, None,
            "HTML step 3 must be retained before ordinary connectivity work at step 7"
        );
        assert!(!retained.already_started && !retained.force_async);
        assert_eq!(retained.script_type, None);
        assert_eq!(retained.preparation_document, None);
    }

    #[test]
    fn empty_preparation_work_is_independent_of_actual_ordinary_ancestor_depth() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = format!("<html><head><script>{}", "<div>".repeat(16));
        let mut observed_work = Vec::new();
        for depth in [0, 16] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source.clone()),
                limits(),
            );
            let (mut c, mut session, capability, node) = fresh_script(&request, false, false);
            let html_index = session.active().unwrap().records[0].children[0];
            let mut parent = session.active().unwrap().node(&mut c, html_index).unwrap();
            let mut tokenizer =
                LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            for _ in 0..depth {
                let tag = next_fixture_tag(&mut tokenizer, &mut c, &[0x64, 0x69, 0x76]);
                let child = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
                session.insert(&mut c, &parent, &child, None, None).unwrap();
                parent = child;
            }
            session.insert(&mut c, &parent, &node, None, None).unwrap();
            tokenizer.release(&mut c).unwrap();
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            assert_eq!(
                session.prepare_disabled_script(&mut c, &node, &capability),
                Ok(DisabledScriptPreparation::Empty)
            );
            observed_work.push(c.usage().resource(HtmlParseResource::WorkCredits).current - before);
            let retained = session.script_metadata(&mut c, &node).unwrap();
            assert_eq!(retained.parser_document, None);
            assert!(retained.force_async && !retained.already_started);
            assert_eq!(retained.script_type, None);
            assert_eq!(retained.preparation_document, None);
            assert_eq!(session.active().unwrap().records.len(), 3 + depth);
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
        // HTML step 6 returns Empty; ordinary connectedness at step 7 is never
        // needed. Both actual trees have the same script payload/qualifications.
        assert_eq!(observed_work[0], observed_work[1]);
    }

    #[test]
    fn actual_template_domain_adoption_qualifies_document_and_only_import_revokes_inventory() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
            limits(),
        );
        let (mut c, mut session, capability, script) = fresh_script(&request, false, false);
        let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
        session.insert(&mut c, &div, &script, None, None).unwrap();
        let initial = session.script_metadata(&mut c, &script).unwrap();
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        let mut foreign_control = control();
        assert!(matches!(
            session.insert(&mut foreign_control, &contents, &div, None, None),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        let mut foreign_arena =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let foreign_root = foreign_arena.root(&mut c).unwrap();
        assert_eq!(
            session.insert(&mut c, &contents, &foreign_root, None, None),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session.insert(&mut c, &contents, &div, Some(&script), None),
            Err(SessionError::WrongSibling)
        );
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        assert_eq!(session.active().unwrap().records[div.index].parent, None);
        foreign_arena.release(&mut c).unwrap();
        // The ordinary div carries the actual script descendant into the real
        // template-owned fragment. No script-name guess or callback flag is used.
        session.insert(&mut c, &contents, &div, None, None).unwrap();
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        let document = session
            .node_document(&mut c, &script)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        let contents_document = session
            .node_document(&mut c, &contents)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        assert!(
            session
                .same_node(&mut c, &document, &contents_document)
                .unwrap()
        );
        let root = session.root(&mut c).unwrap();
        assert!(!session.same_node(&mut c, &root, &document).unwrap());
        assert_eq!(
            session.active().unwrap().records[script.index].parent,
            Some(div.index)
        );
        assert_eq!(session.script_metadata(&mut c, &script).unwrap(), initial);
        session.remove(&mut c, &div, None).unwrap();
        assert_eq!(session.active().unwrap().records[div.index].parent, None);
        session.allocate_element(&mut c, html("span")).unwrap();
        assert_eq!(
            session.active().unwrap().standalone_ineligibility,
            Some(StandaloneUnsupported::Provenance)
        );
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance);
        assert_eq!(
            session.prepare_disabled_script(&mut c, &script, &capability),
            Err(SessionError::Stop(stop))
        );
        let retained = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|entry| {
                if let RetainedPayload::Script { metadata, .. } = entry {
                    Some(*metadata)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(retained, initial);
        assert!(
            matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn cancelled_domain_insert_preserves_ordinary_eligibility_and_all_actual_owners() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
            limits(),
        );
        let (mut c, mut session, _, script) = fresh_script(&request, false, false);
        let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
        session.insert(&mut c, &div, &script, None, None).unwrap();
        request.cancellation().cancel();
        assert_eq!(
            session.insert(&mut c, &contents, &div, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        let storage = session.active().unwrap();
        assert_eq!(storage.standalone_ineligibility, None);
        assert_eq!(storage.records[div.index].parent, None);
        assert_eq!(storage.records[script.index].parent, Some(div.index));
        assert!(storage.records[contents.index].children.is_empty());
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        assert!(matches!(
            session.freeze(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            7
        );
    }

    #[test]
    fn exact_and_one_short_domain_insert_work_admit_eligibility_only_with_topology() {
        let source = "<html><head><script src=x><template><div>";
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut baseline, mut session, _, script) = fresh_script(&request, false, false);
        let (contents, div) =
            actual_template_and_div(request.source(), &mut baseline, &mut session);
        session
            .insert(&mut baseline, &div, &script, None, None)
            .unwrap();
        session.validate_node(&mut baseline, &contents).unwrap();
        session.validate_node(&mut baseline, &div).unwrap();
        session
            .active_mut()
            .unwrap()
            .insert_indices(&mut baseline, contents.index, div.index, None)
            .unwrap();
        let exact = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session.release(&mut baseline).unwrap();
        storage_zero(&baseline);
        for cap in [exact - 1, exact] {
            let limits = limits().into_builder().work_credits(cap).finish().unwrap();
            let request =
                crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits);
            let (mut c, mut session, _, script) = fresh_script(&request, false, false);
            let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
            session.insert(&mut c, &div, &script, None, None).unwrap();
            // Both actual handles are already qualified by setup. Exercise the
            // fixed commit owner separately from its subsequent useful drain.
            session.validate_node(&mut c, &contents).unwrap();
            session.validate_node(&mut c, &div).unwrap();
            let result = session.active_mut().unwrap().insert_indices(
                &mut c,
                contents.index,
                div.index,
                None,
            );
            let storage = session.active().unwrap();
            assert_eq!(storage.records[script.index].parent, Some(div.index));
            if cap == exact {
                assert_eq!(result, Ok(()));
                assert_eq!(storage.records[div.index].parent, Some(contents.index));
                assert_eq!(storage.records[contents.index].children, [div.index]);
                assert_eq!(storage.standalone_ineligibility, None);
                assert_eq!(c.stop(), None);
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert_eq!(storage.records[div.index].parent, None);
                assert!(storage.records[contents.index].children.is_empty());
                assert_eq!(storage.standalone_ineligibility, None);
                let first = c.stop().unwrap();
                assert!(
                    matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                );
            }
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                7
            );
        }
    }

    #[test]
    fn public_associated_domain_insert_stop_after_commit_never_publishes_the_completed_links() {
        let source = "<html><head><script src=x><template><div>";
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut measured, mut session, _, script) = fresh_script(&request, false, false);
        let (contents, div) =
            actual_template_and_div(request.source(), &mut measured, &mut session);
        session
            .insert(&mut measured, &div, &script, None, None)
            .unwrap();
        let start = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .insert(&mut measured, &contents, &div, None, None)
            .unwrap();
        let whole = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - start;
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        let (mut c, mut session, _, script) = fresh_script(&request, false, false);
        let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
        session.insert(&mut c, &div, &script, None, None).unwrap();
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, unused - whole + 1)
            .unwrap();
        let Err(SessionError::Stop(first)) = session.insert(&mut c, &contents, &div, None, None)
        else {
            panic!("real interrupted postcommit drain")
        };
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 1,
                ..
            }
        ));
        let storage = session.active().unwrap();
        assert_eq!(storage.records[div.index].parent, Some(contents.index));
        assert_eq!(storage.records[contents.index].children, [div.index]);
        assert_eq!(storage.records[script.index].parent, Some(div.index));
        assert_eq!(storage.standalone_ineligibility, None);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(first))
        );
        assert_eq!(session.root(&mut c).err(), Some(SessionError::Stop(first)));
        assert_eq!(
            session.insert(&mut c, &contents, &div, None, None),
            Err(SessionError::Stop(first))
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
    }

    #[test]
    fn exact_preparation_work_reaches_step18_and_one_short_retains_step15_for_cleanup() {
        let source = "<html><head><script src=''>";
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
        let (mut baseline, mut session, capability, node) = fresh_script(&request, true, false);
        session
            .prepare_disabled_script(&mut baseline, &node, &capability)
            .unwrap();
        let exact = baseline
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session.release(&mut baseline).unwrap();
        storage_zero(&baseline);
        for cap in [exact - 1, exact] {
            let limits = limits().into_builder().work_credits(cap).finish().unwrap();
            let request =
                crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits);
            let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
            let result = session.prepare_disabled_script(&mut c, &node, &capability);
            let retained = session
                .active()
                .unwrap()
                .payloads
                .values
                .iter()
                .find_map(|entry| {
                    if let RetainedPayload::Script { metadata, .. } = entry {
                        Some(*metadata)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert!(retained.already_started && !retained.force_async);
            assert_eq!(retained.script_type, Some(ScriptType::Classic));
            assert_eq!(retained.parser_document, Some(DocumentRef::Owned(0)));
            if cap == exact {
                assert_eq!(
                    result,
                    Ok(DisabledScriptPreparation::Disabled(ScriptType::Classic))
                );
                assert_eq!(retained.preparation_document, Some(DocumentRef::Owned(0)));
                assert_eq!(c.stop(), None);
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert_eq!(retained.preparation_document, None);
                let first = c.stop().unwrap();
                assert!(
                    matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                );
            }
            // Exact useful work can leave no credit for active disposal. The
            // same retained owner then performs terminal cleanup, without work
            // refund or a usable partially prepared immutable tree.
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                3
            );
        }
    }

    #[test]
    fn the_actual_script_metadata_entry_requires_its_own_slot_and_failed_admission_is_reclaimable()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Root/html plus completion use9 retained slots, with prologue stack coexistence10.
        // Script metadata is reserved BEFORE transfer, followed by its actual
        // document binding, name/URI/attrs and native ticket: completed15.
        // Ten stops after metadata but before document/native preflight;
        // fourteen stops at native-ticket admission after lexical transfer.
        // Root, html and script each also retain their actual node cause.
        for cap in [12, 17, 18] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<html><head><script>"),
                limits()
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
            );
            let (mut c, mut session, capability) = fresh_script_owner(&request);
            let mut tokenizer =
                LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let mut incoming = loop {
                let kind = lexical_kind(&mut tokenizer, &mut c);
                if matches!(&kind, TokenKind::Tag(tag) if tag.name.value.text().code_units() == [0x73,0x63,0x72,0x69,0x70,0x74])
                {
                    let TokenKind::Tag(tag) = kind else {
                        unreachable!()
                    };
                    break Some(tag);
                }
                release_token_kind(&mut Some(kind), &mut c).unwrap();
            };
            let result = session.allocate_disabled_parser_script(
                &mut c,
                &mut incoming,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
                &capability,
            );
            assert_eq!(incoming.is_some(), cap == 12);
            assert_eq!(
                session.active().unwrap().records.len(),
                if cap == 18 { 3 } else { 2 }
            );
            let count = session
                .active()
                .unwrap()
                .payloads
                .values
                .iter()
                .filter(|entry| matches!(entry, RetainedPayload::Script { .. }))
                .count();
            if cap == 18 {
                result.unwrap();
                assert_eq!(count, 1);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    18
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        ..
                    }))
                ));
                assert_eq!(count, 1);
                let stop = c.stop().unwrap();
                assert!(
                    matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
                );
            }
            let first = c.stop();
            if let Some(tag) = incoming {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), first);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                if cap == 18 { 3 } else { 2 }
            );
        }
    }

    #[test]
    fn script_allocation_alias_rejection_preserves_the_actual_lexical_owner_and_arena() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script src=''>"),
            limits(),
        );
        let (mut c, mut session, capability, _) = fresh_script(&request, true, false);
        let mut tokenizer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let tag = loop {
            let kind = lexical_kind(&mut tokenizer, &mut c);
            if let TokenKind::Tag(tag) = kind {
                if tag.name.value.text().code_units() == [0x73, 0x63, 0x72, 0x69, 0x70, 0x74] {
                    break tag;
                }
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            } else {
                panic!("only tags before script")
            }
        };
        let alias = tag.name.value.text().clone();
        let before = session.active().unwrap().records.len();
        let mut input = Some(tag);
        assert!(matches!(
            session.allocate_disabled_parser_script(
                &mut c,
                &mut input,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
                &capability
            ),
            Err(SessionError::AliasedPayload)
        ));
        assert!(input.is_some());
        assert_eq!(session.active().unwrap().records.len(), before);
        assert_eq!(c.stop(), None);
        drop(alias);
        release_token_kind(&mut input.map(TokenKind::Tag), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    fn lexical_script_type(attributes: &str) -> Option<ScriptType> {
        lexical_script_type_source(crate::HtmlSource::utf8(format!("<script {attributes}>")))
    }
    fn lexical_script_type_source(source: crate::HtmlSource) -> Option<ScriptType> {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("script tag")
        };
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let node = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[node.index].kind
        else {
            panic!("script")
        };
        let kind = classify_script(&mut c, element).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        kind
    }

    #[test]
    fn shared_literal_match_keeps_repeated_hidden_comparison_credits_and_stops() {
        let mut c = control();
        let before = c.usage().clone();
        for (value, expected) in [("HiDdEn", true), ("hiddEX", false), ("other", false)] {
            let value = DocumentString::from(value);
            assert_eq!(
                script_match(&mut c, b"", &value, 0, value.len(), b"hidden").unwrap(),
                expected
            );
        }
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            3
        );
        assert_eq!(
            c.usage().work(WorkCategory::InspectByte) - before.work(WorkCategory::InspectByte),
            36
        );
        let value = DocumentString::from("hidden");
        for available in [0, 18] {
            let mut c = control();
            c.charge(WorkCategory::StateStep, limits().work_credits() - available)
                .unwrap();
            assert!(script_match(&mut c, b"", &value, 0, value.len(), b"hidden").is_err());
            assert!(c.stop().is_some());
        }
        let cancellation = HtmlParseCancellation::new();
        cancellation.cancel();
        let mut c = ParseControl::new(limits(), cancellation);
        assert!(script_match(&mut c, b"", &value, 0, value.len(), b"hidden").is_err());
        assert_eq!(c.usage().work(WorkCategory::InspectByte), 0);
    }

    #[test]
    fn script_type_classification_preserves_preexisting_charge_totals() {
        for (explicit, language, expected, visits_expected, inspections_expected) in [
            (None, None, Some(ScriptType::Classic), 4, 0),
            (Some(""), Some("unknown"), Some(ScriptType::Classic), 16, 36),
            (
                Some("text/javascript"),
                None,
                Some(ScriptType::Classic),
                24,
                75,
            ),
            (
                Some(" \tTEXT/JAVASCRIPT\n"),
                None,
                Some(ScriptType::Classic),
                56,
                103,
            ),
            (Some("MoDuLe"), None, Some(ScriptType::Module), 78, 34),
            (Some(" module "), None, None, 82, 20),
            (None, Some("javascript"), Some(ScriptType::Classic), 24, 77),
            (None, Some(" javascript "), None, 50, 50),
            (Some("unknown"), Some("javascript"), None, 86, 40),
            (Some("application/json"), None, None, 82, 19),
        ] {
            let mut element = html("script");
            element.attributes = OrderedAttributes::new(
                explicit
                    .map(|v| Attribute::new(crate::KnownHtmlAttribute::Type.qualified_name(), v))
                    .into_iter()
                    .chain(language.map(|v| {
                        Attribute::new(QualifiedName::new(Namespace::none(), None, "language"), v)
                    })),
            )
            .unwrap();
            let mut c = control();
            let before = c.usage().clone();
            assert_eq!(classify_script(&mut c, &element).unwrap(), expected);
            let visits =
                c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit);
            let inspections =
                c.usage().work(WorkCategory::InspectByte) - before.work(WorkCategory::InspectByte);
            assert_eq!(
                (visits, inspections),
                (visits_expected, inspections_expected),
                "{explicit:?}/{language:?}"
            );
            for available in [0, visits_expected + inspections_expected - 1] {
                let mut short = control();
                short
                    .charge(WorkCategory::StateStep, limits().work_credits() - available)
                    .unwrap();
                assert!(classify_script(&mut short, &element).is_err());
                assert!(short.stop().is_some());
            }
        }
    }

    #[test]
    fn isolated_surrogates_and_supplementary_units_are_not_mime_normalized() {
        for extra in [vec![0xd800], vec![0xdc00], vec![0xd83d, 0xde00]] {
            let units = "<script type='text/javascript"
                .encode_utf16()
                .chain(extra)
                .chain("'>".encode_utf16())
                .collect::<Vec<_>>();
            assert_eq!(
                lexical_script_type_source(crate::HtmlSource::utf16(DocumentString::from_utf16(
                    units
                ))),
                None
            );
        }
    }

    #[test]
    fn every_javascript_essence_and_ascii_case_variant_is_classic_without_parameters() {
        // Independently enumerated selected MIME §4.6 expectations.
        for essence in [
            "application/ecmascript",
            "application/javascript",
            "application/x-ecmascript",
            "application/x-javascript",
            "text/ecmascript",
            "text/javascript",
            "text/javascript1.0",
            "text/javascript1.1",
            "text/javascript1.2",
            "text/javascript1.3",
            "text/javascript1.4",
            "text/javascript1.5",
            "text/jscript",
            "text/livescript",
            "text/x-ecmascript",
            "text/x-javascript",
        ] {
            for value in [essence.to_owned(), essence.to_ascii_uppercase()] {
                assert_eq!(
                    lexical_script_type(&format!("type='{value}'")),
                    Some(ScriptType::Classic)
                );
            }
        }
        for value in [
            "text/javascript;charset=utf-8",
            "text/javascript1.6",
            "application/json",
            "text/javaſcript",
            "text/javascript&#xFFFD;",
        ] {
            assert_eq!(
                lexical_script_type(&format!("type='{value}'")),
                None,
                "{value}"
            );
        }
    }

    #[test]
    fn explicit_type_and_legacy_language_follow_distinct_whitespace_and_default_branches() {
        for attributes in [
            "",
            "type=''",
            "language=''",
            "type='' language=unknown",
            "language=javascript",
            "type=' \tTEXT/JAVASCRIPT\n\r\x0c '",
        ] {
            assert_eq!(
                lexical_script_type(attributes),
                Some(ScriptType::Classic),
                "{attributes}"
            );
        }
        for (value, kind) in [
            ("MoDuLe", ScriptType::Module),
            ("IMPORTMAP", ScriptType::ImportMap),
            ("speculationrules", ScriptType::SpeculationRules),
        ] {
            assert_eq!(lexical_script_type(&format!("type={value}")), Some(kind));
            assert_eq!(lexical_script_type(&format!("type=' {value} '")), None);
        }
        for attributes in [
            "language=' javascript '",
            "language=module",
            "type=unknown language=javascript",
            "type='   '",
            "type='&#xA0;text/javascript&#xA0;'",
            "type=text/javascript1.0x",
        ] {
            assert_eq!(lexical_script_type(attributes), None, "{attributes}");
        }
    }

    #[test]
    fn actual_import_permanently_revokes_standalone_inventory_but_foreign_control_does_not() {
        let mut c = control();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.validate_standalone_document(&mut c).unwrap();
        let mut foreign = control();
        assert!(matches!(
            session.allocate_element(&mut foreign, html("div")),
            Err(SessionError::Stop(_))
        ));
        assert!(session.active().unwrap().standalone_ineligibility.is_none());
        session.validate_standalone_document(&mut c).unwrap();
        session.allocate_element(&mut c, html("div")).unwrap();
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance);
        assert_eq!(
            session.validate_standalone_document(&mut c),
            Err(SessionError::Stop(stop))
        );
        assert!(
            matches!(session.allocate_implicit_html(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
        );
        assert_eq!(
            session.validate_standalone_document(&mut c),
            Err(SessionError::Stop(stop))
        );
        assert!(
            matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn generic_fragment_and_preexisting_input_cannot_establish_ordinary_document_provenance() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        assert_eq!(
            session.validate_standalone_document(&mut c),
            Err(SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            )))
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn actual_template_contents_adoption_qualifies_inert_document_and_preserves_disconnected_script_prefix()
     {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><template><script src=''>"),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, false, false);
        let initial = session.script_metadata(&mut c, &node).unwrap();
        let mut tokenizer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let tag = loop {
            let kind = lexical_kind(&mut tokenizer, &mut c);
            if let TokenKind::Tag(tag) = kind {
                if tag.name.value.text().code_units()
                    == [0x74, 0x65, 0x6d, 0x70, 0x6c, 0x61, 0x74, 0x65]
                {
                    break tag;
                }
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            } else {
                panic!("template tag")
            }
        };
        let template = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
        let contents = session
            .template_contents(&mut c, &template)
            .unwrap()
            .unwrap();
        session
            .insert(&mut c, &contents, &node, None, None)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        let parser = session
            .script_parser_document(&mut c, &node)
            .unwrap()
            .unwrap();
        let document = session
            .node_document(&mut c, &node)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        let contents_document = session
            .node_document(&mut c, &contents)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        assert!(!session.same_node(&mut c, &parser, &document).unwrap());
        assert!(
            session
                .same_node(&mut c, &document, &contents_document)
                .unwrap()
        );
        assert_eq!(
            session.prepare_disabled_script(&mut c, &node, &capability),
            Ok(DisabledScriptPreparation::Disconnected)
        );
        drop((parser, document, contents_document));
        let retained = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|entry| {
                if let RetainedPayload::Script { metadata, .. } = entry {
                    Some(*metadata)
                } else {
                    None
                }
            })
            .unwrap();
        let mut expected = initial;
        expected.parser_document = None;
        expected.force_async = true;
        assert_eq!(retained, expected);
        assert!(retained.preparation_document.is_none());
        drop((node, contents, template));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert!(c.stop().is_none());
    }

    fn form_session(c: &mut ParseControl) -> (ParserArenaSession, ParserNode) {
        let mut session =
            ParserArenaSession::new(c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
        let root = session.root(c).unwrap();
        let html = session.allocate_implicit_html(c).unwrap();
        session.insert(c, &root, &html, None, None).unwrap();
        let body = session.allocate_implicit_body(c, None).unwrap();
        session.insert(c, &html, &body, None, None).unwrap();
        (session, body)
    }
    fn form_creation(
        session: &mut ParserArenaSession,
        tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
        parent: &ParserNode,
        form: Option<&ParserNode>,
    ) -> ParserNode {
        let TokenKind::Tag(tag) = lexical_kind(tokenizer, c) else {
            panic!("actual tag")
        };
        let mut input = Some(tag);
        let node = session
            .allocate_html_tag_for_creation(
                c,
                &mut input,
                parent,
                form,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        assert!(input.is_none());
        node
    }
    fn assert_form_owner(
        session: &ParserArenaSession,
        c: &mut ParseControl,
        image: &ParserNode,
        owner: Option<&ParserNode>,
        inserted: bool,
    ) {
        let actual = session.image_form_owner(c, image).unwrap();
        match (actual, owner) {
            (Some(actual), Some(expected)) => {
                assert!(session.same_node(c, &actual, expected).unwrap())
            }
            (None, None) => {}
            _ => panic!("wrong actual form owner"),
        }
        assert_eq!(
            session.image_form_state(c, image).unwrap().parser_inserted,
            inserted
        );
    }
    fn assert_listed_owner(
        session: &ParserArenaSession,
        c: &mut ParseControl,
        node: &ParserNode,
        kind: FormAssociationKind,
        owner: Option<&ParserNode>,
        parser_inserted: bool,
    ) {
        let state = session.form_association_state(c, node).unwrap();
        assert_eq!(state.kind, kind);
        assert_eq!(state.parser_inserted, parser_inserted);
        match (session.form_association_owner(c, node).unwrap(), owner) {
            (Some(actual), Some(expected)) => {
                assert!(session.same_node(c, &actual, expected).unwrap())
            }
            (None, None) => {}
            _ => panic!("wrong actual listed form owner"),
        }
    }

    #[test]
    fn listed_creation_suppresses_pointer_for_present_empty_form_and_retains_qualified_kinds() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source =
            crate::HtmlSource::utf8("<form id=f><button><fieldset form=''><object form=f>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, Some(&form));
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&form),
            true,
        );
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let fieldset = form_creation(&mut session, &mut lexer, &mut c, &body, Some(&form));
        assert_listed_owner(
            &session,
            &mut c,
            &fieldset,
            FormAssociationKind::Fieldset,
            None,
            false,
        );
        session
            .insert(&mut c, &body, &fieldset, None, None)
            .unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &fieldset,
            FormAssociationKind::Fieldset,
            None,
            false,
        );
        let object = form_creation(&mut session, &mut lexer, &mut c, &body, Some(&form));
        assert_listed_owner(
            &session,
            &mut c,
            &object,
            FormAssociationKind::Object,
            None,
            false,
        );
        session.insert(&mut c, &body, &object, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &object,
            FormAssociationKind::Object,
            Some(&form),
            false,
        );
        assert_eq!(
            session.image_form_state(&mut c, &button),
            Err(SessionError::ImageFormMetadataUnavailable)
        );
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().next().unwrap();
        let id = document.node(&body).unwrap().children().nth(3).unwrap();
        assert_eq!(
            frozen.form_association_state(&mut c, &id).unwrap().kind,
            FormAssociationKind::Object
        );
        assert!(
            frozen
                .form_association_owner(&mut c, &id)
                .unwrap()
                .is_some()
        );
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_first_id_tree_order_blocks_later_form_and_id_only_removal_reassociates() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<button form=f><div id=f><form id=f><form id=f>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let blocker = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &blocker, None, None).unwrap();
        let first = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &first, None, None).unwrap();
        let second = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &second, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            None,
            false,
        );
        session.remove(&mut c, &blocker, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&first),
            false,
        );
        session.remove(&mut c, &first, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&second),
            false,
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_removal_id_event_resets_detached_same_document_recipient_after_local_preservation() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<div><button form=f><form id=f>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let group = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &group, None, None).unwrap();
        let button = form_creation(&mut session, &mut lexer, &mut c, &group, None);
        session.insert(&mut c, &group, &button, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, &mut c, &group, None);
        session.insert(&mut c, &group, &form, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        session.remove(&mut c, &group, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            None,
            false,
        );
        assert_eq!(
            session.active().unwrap().records[button.index].parent,
            Some(group.index)
        );
        assert_eq!(
            session.active().unwrap().records[form.index].parent,
            Some(group.index)
        );
        // An actual hook inside an already detached tree still runs the global
        // reset. A parentless removal afterwards has no hook and no phase.
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.remove(&mut c, &form, None).unwrap();
        let hook = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.remove(&mut c, &form, None).unwrap();
        let noop = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        assert!(hook > noop);
        assert!(session.active().unwrap().effects.form.is_none());
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            None,
            false,
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_connected_explicit_form_has_no_ancestor_fallback_but_detached_reset_does() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form id=f><fieldset form=missing><object form=''>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        let fieldset = form_creation(&mut session, &mut lexer, &mut c, &form, Some(&form));
        session
            .insert(&mut c, &form, &fieldset, None, None)
            .unwrap();
        let object = form_creation(&mut session, &mut lexer, &mut c, &form, Some(&form));
        session.insert(&mut c, &form, &object, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &fieldset,
            FormAssociationKind::Fieldset,
            None,
            false,
        );
        assert_listed_owner(
            &session,
            &mut c,
            &object,
            FormAssociationKind::Object,
            None,
            false,
        );
        session.remove(&mut c, &form, None).unwrap();
        // ID removal globally resets every explicit-form recipient in this
        // originating document, each against its real detached ordinary tree.
        assert_listed_owner(
            &session,
            &mut c,
            &fieldset,
            FormAssociationKind::Fieldset,
            Some(&form),
            false,
        );
        assert_listed_owner(
            &session,
            &mut c,
            &object,
            FormAssociationKind::Object,
            Some(&form),
            false,
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_late_body_id_merge_resets_exact_target_before_next_attribute_and_freezes() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<button form=f><form id=f><body id=f title=after>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("body")
        };
        let mut input = Some(tag);
        session
            .add_body_attributes_if_missing(&mut c, &body, &mut input)
            .unwrap();
        assert!(input.is_none());
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            None,
            false,
        );
        assert!(session.active().unwrap().effects.form.is_none());
        let HtmlNodeKind::Element(body_record) =
            &session.active().unwrap().records[body.index].kind
        else {
            panic!("body")
        };
        assert_eq!(
            body_record
                .attributes()
                .as_slice()
                .iter()
                .map(|a| a.name().local_name().code_units().to_vec())
                .collect::<Vec<_>>(),
            vec![vec![105, 100], vec![116, 105, 116, 108, 101]]
        );
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().next().unwrap();
        let id = document.node(&body).unwrap().children().next().unwrap();
        assert_eq!(frozen.form_association_owner(&mut c, &id).unwrap(), None);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_metadata_cap_rejects_all_three_kinds_before_any_lexical_take() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for text in ["<button>", "<fieldset>", "<object>"] {
            let source = crate::HtmlSource::utf8(text);
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .auxiliary_entries(13)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = session.root(&mut c).unwrap();
            let html = session.allocate_implicit_html(&mut c).unwrap();
            session.insert(&mut c, &root, &html, None, None).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
                panic!("tag")
            };
            // Lexical pending events precede the final retained allocation.
            // Body stays detached: no unneeded child-order scratch is admitted.
            let _body = session.allocate_implicit_body(&mut c, None).unwrap();
            let mut input = Some(tag);
            let original = input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .to_vec();
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &html,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            );
            let stop = c.stop().unwrap();
            assert_eq!(result.err(), Some(SessionError::Stop(stop)));
            assert!(matches!(
                stop,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    current: 13,
                    requested_delta: 2,
                    ..
                }
            ));
            assert_eq!(
                input.as_ref().unwrap().name.value.text().code_units(),
                original
            );
            assert_eq!(session.active().unwrap().records.len(), 3);
            assert_eq!(session.active().unwrap().payloads.values.len(), 10);
            assert!(session.active().unwrap().effects.form.is_none());
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            release_token_kind(&mut input.map(TokenKind::Tag), &mut c).unwrap();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(stop));
        }
    }

    #[test]
    fn listed_getters_require_actual_active_and_frozen_arena_and_control_owners() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<button>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let node = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &node, None, None).unwrap();
        let mut other = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let foreign = other.root(&mut c).unwrap();
        assert_eq!(
            session.form_association_state(&mut c, &foreign),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session.form_association_owner(&mut c, &foreign).err(),
            Some(SessionError::ForeignNode)
        );
        assert_eq!(
            session.form_association_state(&mut c, &body),
            Err(SessionError::FormAssociationMetadataUnavailable)
        );
        let mut foreign_c = control();
        foreign_c.admit_source(&source).unwrap();
        let before = c.usage();
        assert!(matches!(
            session.form_association_state(&mut foreign_c, &node),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(c.usage(), before);
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().next().unwrap();
        let id = document.node(&body).unwrap().children().next().unwrap();
        let mut foreign_frozen = other.freeze(&mut c).unwrap();
        let foreign_id = foreign_frozen.fragment().unwrap().root().id();
        assert_eq!(
            frozen.form_association_state(&mut c, &foreign_id),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            frozen.form_association_owner(&mut c, &foreign_id).err(),
            Some(SessionError::ForeignNode)
        );
        assert_eq!(
            frozen.form_association_state(&mut c, &id).unwrap().kind,
            FormAssociationKind::Button
        );
        assert_eq!(
            frozen.image_form_state(&mut c, &id),
            Err(SessionError::ImageFormMetadataUnavailable)
        );
        frozen.release(&mut c).unwrap();
        foreign_frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_id_matching_is_exact_untrimmed_utf16_and_empty_id_never_matches_empty_form() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let mut units: Vec<u16> = "<button form='X ".encode_utf16().collect();
        units.extend([0xd800, 0xd83d, 0xde00]);
        units.extend("'><form id='x ".encode_utf16());
        units.extend([0xd800, 0xd83d, 0xde00]);
        units.extend("'><form id='X ".encode_utf16());
        units.extend([0xd800, 0xd83d, 0xde00]);
        units.extend("'><fieldset form=''><form id=''>".encode_utf16());
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(units));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let lower = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &lower, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            None,
            false,
        );
        let exact = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &exact, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&exact),
            false,
        );
        let empty = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &empty, None, None).unwrap();
        let empty_id = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session
            .insert(&mut c, &body, &empty_id, None, None)
            .unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &empty,
            FormAssociationKind::Fieldset,
            None,
            false,
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_detached_id_change_resets_only_same_ordinary_tree() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><button form=f><button form=f><body id=f>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        let in_form = form_creation(&mut session, &mut lexer, &mut c, &form, None);
        session.insert(&mut c, &form, &in_form, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &in_form,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        let outside = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &outside, None, None).unwrap();
        session.remove(&mut c, &body, None).unwrap();
        let before = session.form_association_state(&mut c, &in_form).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("body")
        };
        session
            .add_body_attributes_if_missing(&mut c, &body, &mut Some(tag))
            .unwrap();
        assert_eq!(
            session.form_association_state(&mut c, &in_form).unwrap(),
            before
        );
        assert_listed_owner(
            &session,
            &mut c,
            &in_form,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        assert_listed_owner(
            &session,
            &mut c,
            &outside,
            FormAssociationKind::Button,
            None,
            false,
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    fn listed_move_fixture(
        c: &mut ParseControl,
    ) -> (ParserArenaSession, ParserNode, ParserNode, ParserNode) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<div><button form=f><form id=f><div>");
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, c).unwrap();
        let (mut session, body) = form_session(c);
        let group = form_creation(&mut session, &mut lexer, c, &body, None);
        session.insert(c, &body, &group, None, None).unwrap();
        let button = form_creation(&mut session, &mut lexer, c, &group, None);
        session.insert(c, &group, &button, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, c, &group, None);
        session.insert(c, &group, &form, None, None).unwrap();
        let target = form_creation(&mut session, &mut lexer, c, &body, None);
        session.insert(c, &body, &target, None, None).unwrap();
        lexer.release(c).unwrap();
        (session, group, button, target)
    }

    #[test]
    fn listed_exact_and_one_short_effect_work_retains_cleanup_only_phase_and_first_stop() {
        let mut measured = control();
        let (mut session, group, _, target) = listed_move_fixture(&mut measured);
        let start = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .insert(&mut measured, &target, &group, None, None)
            .unwrap();
        let whole = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - start;
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        let tail = 2 + mem::size_of::<Option<select::EffectFrame>>() as u64 + 1 + 1;
        for short in [false, true] {
            let mut c = control();
            let (mut session, group, button, target) = listed_move_fixture(&mut c);
            let unused = limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                unused - whole + if short { tail + 1 } else { 0 },
            )
            .unwrap();
            let result = session.insert(&mut c, &target, &group, None, None);
            if short {
                let stop = c.stop().unwrap();
                assert_eq!(result, Err(SessionError::Stop(stop)));
                let storage = session.active().unwrap();
                assert!(storage.effects.form.is_none());
                let (list, cursor) = storage
                    .effects
                    .retained_post_connection()
                    .expect("specific unfinished final postconnection clear");
                assert_eq!(cursor, 3);
                assert!(matches!(
                    storage.payloads.values[list],
                    RetainedPayload::Retired
                ));
                assert_eq!(
                    session.form_association_state(&mut c, &button),
                    Err(SessionError::Stop(stop))
                );
                assert_eq!(
                    session.remove(&mut c, &group, None),
                    Err(SessionError::Stop(stop))
                );
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
                let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
                session.release(&mut c).unwrap();
                storage_zero(&c);
                assert_eq!(c.stop(), Some(stop));
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current,
                    work
                );
            } else {
                result.unwrap();
                assert_eq!(
                    session.active().unwrap().records[group.index].parent,
                    Some(target.index)
                );
                assert!(session.active().unwrap().effects.form.is_none());
                // Successful operation exhausts its exact finite budget. Latch
                // the next real attempted credit before terminal disposal.
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
                session.release(&mut c).unwrap();
                storage_zero(&c);
            }
        }
    }

    #[test]
    fn listed_real_cancel_between_removal_and_insertion_retains_completed_global_reset() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let (mut session, group, button, target) = listed_move_fixture(&mut c);
        session.active_mut().unwrap().cancel_after_removal = Some(cancellation);
        assert_eq!(
            session.insert(&mut c, &target, &group, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        let storage = session.active().unwrap();
        assert_eq!(storage.records[group.index].parent, Some(target.index));
        let phase = storage.effects.form.unwrap();
        assert_eq!(phase.stage, FormEffectStage::Adoption);
        assert_eq!(phase.next, Some(group.index));
        let state = storage
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::FormAssociation {
                    node: Some(node),
                    state,
                } if *node == button.index => Some(*state),
                _ => None,
            })
            .unwrap();
        assert_eq!(state.owner, None);
        assert!(!state.parser_inserted);
        assert_eq!(state.evidence, FormAssociationEvidence::Reset);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }

    fn listed_id_slot_fixture(
        c: &mut ParseControl,
    ) -> (
        ParserArenaSession,
        ParserNode,
        ParserNode,
        PreparedAttributeMerge,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<button form=f><form id=f><body id=f title=after>");
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, c).unwrap();
        let (mut session, body) = form_session(c);
        let button = form_creation(&mut session, &mut lexer, c, &body, None);
        session.insert(c, &body, &button, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, c, &body, None);
        session.insert(c, &body, &form, None, None).unwrap();
        assert_listed_owner(
            &session,
            c,
            &button,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, c) else {
            panic!("body")
        };
        let prepared = session
            .active_mut()
            .unwrap()
            .prepare_named_attribute_merge(c, &body, &mut Some(tag), AttributeMergeName::Body)
            .unwrap()
            .unwrap();
        lexer.release(c).unwrap();
        session
            .active_mut()
            .unwrap()
            .edit_attributes(
                c,
                prepared.target,
                prepared.container,
                prepared.stage,
                1,
                true,
            )
            .unwrap();
        // This is the actual fixed append producer, not an invented phase. It
        // retains both the installed slot and incoming redirect until drain.
        let storage = session.active().unwrap();
        assert_eq!(
            storage.effects.form.unwrap().stage,
            FormEffectStage::IdChange
        );
        let RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
            moved, values, ..
        })) = &storage.payloads.values[prepared.stage]
        else {
            panic!("transfer")
        };
        assert_eq!(*moved, 1);
        assert_eq!(values.len(), 1);
        let HtmlNodeKind::Element(element) = &storage.records[body.index].kind else {
            panic!("body")
        };
        assert_eq!(element.attributes().as_slice().len(), 1);
        assert_eq!(
            element.attributes().as_slice()[0]
                .name()
                .local_name()
                .code_units(),
            [105, 100]
        );
        (session, body, button, prepared)
    }

    #[test]
    fn listed_id_append_blocks_observation_before_rebind_and_keeps_cleanup_redirect() {
        let mut c = control();
        let (mut session, body, button, prepared) = listed_id_slot_fixture(&mut c);
        let old_count = session.active().unwrap().records.len();
        assert!(matches!(
            session.form_association_owner(&mut c, &button),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::PendingFormEffects
            )))
        ));
        let stop = c.stop().unwrap();
        assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        let storage = session.active().unwrap();
        assert_eq!(storage.records.len(), old_count);
        assert!(storage.effects.form.is_some());
        let HtmlNodeKind::Element(element) = &storage.records[body.index].kind else {
            panic!("body")
        };
        assert_eq!(element.attributes().as_slice().len(), 1);
        assert!(matches!(
            &storage.payloads.values[prepared.stage],
            RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved: 1, .. }))
        ));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn listed_id_global_drain_exact_and_one_short_keeps_real_slot_owners_and_metadata_prefix() {
        let mut measured = control();
        let (mut session, _, _, _) = listed_id_slot_fixture(&mut measured);
        let start = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .active_mut()
            .unwrap()
            .drain_form_effects(&mut measured)
            .unwrap();
        let whole = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - start;
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        for short in [false, true] {
            let mut c = control();
            let (mut session, body, button, prepared) = listed_id_slot_fixture(&mut c);
            let unused = limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(WorkCategory::StateStep, unused - whole + u64::from(short))
                .unwrap();
            let result = session.active_mut().unwrap().drain_form_effects(&mut c);
            let storage = session.active().unwrap();
            let state = storage
                .payloads
                .values
                .iter()
                .find_map(|entry| match entry {
                    RetainedPayload::FormAssociation {
                        node: Some(node),
                        state,
                    } if *node == button.index => Some(*state),
                    _ => None,
                })
                .unwrap();
            assert_eq!(state.owner, None);
            assert_eq!(state.evidence, FormAssociationEvidence::Reset);
            assert!(!state.parser_inserted);
            assert!(matches!(
                &storage.payloads.values[prepared.stage],
                RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved: 1, .. }))
            ));
            let HtmlNodeKind::Element(element) = &storage.records[body.index].kind else {
                panic!("body")
            };
            assert_eq!(element.attributes().as_slice().len(), 1);
            if short {
                let stop = c.stop().unwrap();
                assert_eq!(result, Err(SessionError::Stop(stop)));
                assert!(session.active().unwrap().effects.form.is_some());
                assert_eq!(
                    session.form_association_owner(&mut c, &button).err(),
                    Some(SessionError::Stop(stop))
                );
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            } else {
                result.unwrap();
                assert!(session.active().unwrap().effects.form.is_none());
                // No credit remains to complete the next slot/rebind. Terminal
                // disposal must reclaim the existing redirect, not drop it.
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
            }
            let stop = c.stop().unwrap();
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let high = c.usage().actual_capacity_high_water;
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(stop));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
            assert_eq!(c.usage().actual_capacity_high_water, high);
        }
    }

    #[test]
    fn listed_id_cancel_after_real_slot_append_retains_owner_and_reclaims_all_partial_storage() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let (mut session, body, button, prepared) = listed_id_slot_fixture(&mut c);
        cancellation.cancel();
        assert_eq!(
            session.active_mut().unwrap().drain_form_effects(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(
            session.form_association_owner(&mut c, &button).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        assert!(session.active().unwrap().effects.form.is_some());
        assert!(matches!(
            &session.active().unwrap().payloads.values[prepared.stage],
            RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved: 1, .. }))
        ));
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[body.index].kind
        else {
            panic!("body")
        };
        assert_eq!(element.attributes().as_slice().len(), 1);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn listed_virtual_removal_excludes_moved_id_from_outside_tree_before_insertion() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let source = crate::HtmlSource::utf8("<button form=f><div><form id=f><div>");
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let group = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &group, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, &mut c, &group, None);
        session.insert(&mut c, &group, &form, None, None).unwrap();
        let target = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &target, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        lexer.release(&mut c).unwrap();
        session.active_mut().unwrap().cancel_after_removal = Some(cancellation);
        assert_eq!(
            session.insert(&mut c, &target, &group, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        let storage = session.active().unwrap();
        assert_eq!(storage.records[group.index].parent, Some(target.index));
        assert_eq!(storage.records[form.index].parent, Some(group.index));
        let state = storage
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::FormAssociation {
                    node: Some(node),
                    state,
                } if *node == button.index => Some(*state),
                _ => None,
            })
            .unwrap();
        // Final links already expose the ID at the new destination. Removal
        // resolution must exclude the moved root from this outside scan.
        assert_eq!(state.owner, None);
        assert_eq!(
            storage.effects.form.unwrap().stage,
            FormEffectStage::Adoption
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn listed_namespaced_id_is_inapplicable_and_imported_control_metadata_is_unavailable() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<button form=f><form id=f>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        // Preexisting immutable native input deliberately exercises namespace
        // identity, not parser-created physical storage or foreign token support.
        let native = HtmlElement {
            name: QualifiedName::new(Namespace::html(), None, "button"),
            attributes: OrderedAttributes::new(vec![Attribute::new(
                QualifiedName::new(
                    Namespace::from_uri("urn:test"),
                    Some(DocumentString::from("p")),
                    "id",
                ),
                "f",
            )])
            .unwrap(),
        };
        let imported = session.allocate_element(&mut c, native).unwrap();
        session
            .insert(&mut c, &body, &imported, None, None)
            .unwrap();
        session.insert(&mut c, &body, &form, None, None).unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            FormAssociationKind::Button,
            Some(&form),
            false,
        );
        assert_eq!(
            session.form_association_state(&mut c, &imported),
            Err(SessionError::FormAssociationMetadataUnavailable)
        );
        assert_eq!(
            session.validate_standalone_document(&mut c),
            Err(SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            )))
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn listed_empty_and_ignored_duplicate_late_id_do_not_replace_existing_form_owner() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<button form=f><form id=f><body id=''><body id=f>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &button, None, None).unwrap();
        let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        for _ in 0..2 {
            let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
                panic!("body")
            };
            session
                .add_body_attributes_if_missing(&mut c, &body, &mut Some(tag))
                .unwrap();
            assert_listed_owner(
                &session,
                &mut c,
                &button,
                FormAssociationKind::Button,
                Some(&form),
                false,
            );
            assert!(session.active().unwrap().effects.form.is_none());
        }
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[body.index].kind
        else {
            panic!("body")
        };
        assert_eq!(element.attributes().as_slice().len(), 1);
        assert!(element.attributes().as_slice()[0].value().is_empty());
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn image_creation_uses_actual_same_tree_and_preserves_nonancestor_parser_owner() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><img form=other><form><img>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        let image = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&form));
        assert_form_owner(&session, &mut c, &image, Some(&form), true);
        session.insert(&mut c, &body, &image, None, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&form), true);
        let detached_form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        let other_image = form_creation(
            &mut session,
            &mut tokenizer,
            &mut c,
            &body,
            Some(&detached_form),
        );
        assert_form_owner(&session, &mut c, &other_image, None, false);
        session
            .insert(&mut c, &form, &other_image, None, None)
            .unwrap();
        assert_form_owner(&session, &mut c, &other_image, Some(&form), false);
        assert_eq!(
            session
                .image_form_state(&mut c, &other_image)
                .unwrap()
                .evidence,
            FormAssociationEvidence::Reset
        );
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn image_remove_then_reinsert_resets_historical_owner_to_actual_nearest_form() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><form><img>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let first = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        let second = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &first, None, None).unwrap();
        session.insert(&mut c, &body, &second, None, None).unwrap();
        let image = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&first));
        session.insert(&mut c, &body, &image, None, None).unwrap();
        session.insert(&mut c, &second, &image, None, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&second), false);
        session.remove(&mut c, &image, None).unwrap();
        assert_form_owner(&session, &mut c, &image, None, false);
        session.insert(&mut c, &first, &image, None, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&first), false);
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn image_subtree_move_keeps_owner_when_form_and_image_share_removed_tree() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><img><div>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        let image = form_creation(&mut session, &mut tokenizer, &mut c, &form, Some(&form));
        session.insert(&mut c, &form, &image, None, None).unwrap();
        let div = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &div, None, None).unwrap();
        session.insert(&mut c, &div, &form, None, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&form), true);
        session.remove(&mut c, &form, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&form), true);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        assert_form_owner(&session, &mut c, &image, Some(&form), true);
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn image_same_parent_reorder_and_adoption_child_transfer_apply_real_resets() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><form><img><img>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let a = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        let b = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &a, None, None).unwrap();
        session.insert(&mut c, &body, &b, None, None).unwrap();
        let one = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&a));
        let two = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&a));
        session.insert(&mut c, &body, &one, None, None).unwrap();
        session.insert(&mut c, &body, &two, None, None).unwrap();
        session.insert(&mut c, &body, &one, None, None).unwrap();
        assert_form_owner(&session, &mut c, &one, None, false);
        session.insert(&mut c, &a, &one, None, None).unwrap();
        session.insert(&mut c, &a, &two, None, None).unwrap();
        session.reparent_children(&mut c, &a, &b, None).unwrap();
        assert_form_owner(&session, &mut c, &one, Some(&b), false);
        assert_form_owner(&session, &mut c, &two, Some(&b), false);
        assert_eq!(
            session.active().unwrap().records[b.index].children,
            [one.index, two.index]
        );
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn image_frozen_metadata_resolves_actual_native_identity_and_unavailable_is_not_null() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><img><img>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
        session.insert(&mut c, &body, &form, None, None).unwrap();
        let image = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&form));
        session.insert(&mut c, &body, &image, None, None).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("img")
        };
        let raw = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
        session.insert(&mut c, &body, &raw, None, None).unwrap();
        assert_eq!(
            session.image_form_state(&mut c, &raw),
            Err(SessionError::ImageFormMetadataUnavailable)
        );
        tokenizer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let (image_id, form_id, raw_id) = {
            let doc = frozen.document().unwrap();
            let html = doc.root().children().next().unwrap();
            let body = doc.node(&html).unwrap().children().next().unwrap();
            let ids: Vec<_> = doc.node(&body).unwrap().children().collect();
            (ids[1].clone(), ids[0].clone(), ids[2].clone())
        };
        assert_eq!(
            frozen.image_form_owner(&mut c, &image_id).unwrap(),
            Some(form_id.clone())
        );
        assert!(
            frozen
                .image_form_state(&mut c, &image_id)
                .unwrap()
                .parser_inserted
        );
        assert_eq!(
            frozen.image_form_state(&mut c, &raw_id),
            Err(SessionError::ImageFormMetadataUnavailable)
        );
        let other = HtmlDocument::builder(
            HtmlConstructionLimits::new(10, 10, 10, 100, 10),
            HtmlDocumentMode::NoQuirks,
        )
        .unwrap()
        .finish();
        let foreign = other.root().id();
        assert_eq!(
            frozen.image_form_state(&mut c, &foreign),
            Err(SessionError::ForeignNode)
        );
        drop((image_id, form_id, raw_id, foreign, other));
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn image_creation_rejects_wrong_arena_alias_and_foreign_lexical_owner_before_take() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<img><img>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let mut other =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let foreign = other.allocate_implicit_html(&mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("img")
        };
        let mut input = Some(tag);
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        assert_eq!(
            session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut input,
                    &foreign,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                )
                .unwrap_err(),
            SessionError::ForeignNode
        );
        assert!(input.is_some());
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
        let alias = input.as_ref().unwrap().name.value.text().clone();
        assert_eq!(
            session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut input,
                    &body,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                )
                .unwrap_err(),
            SessionError::AliasedPayload
        );
        assert!(input.is_some());
        drop(alias);
        release_formatting_tag(&mut input, &mut c).unwrap();
        let mut foreign_control = control();
        foreign_control.admit_source(&source).unwrap();
        let mut foreign_tokenizer =
            LexerFixture::new(&source, State::Data, None, &mut foreign_control).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut foreign_tokenizer, &mut foreign_control) else {
            panic!("img")
        };
        let mut input = Some(tag);
        assert!(matches!(
            session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::parser_session::CreationContext::Ordinary
            ),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert!(input.is_some());
        release_formatting_tag(&mut input, &mut foreign_control).unwrap();
        foreign_tokenizer.release(&mut foreign_control).unwrap();
        storage_zero(&foreign_control);
        tokenizer.release(&mut c).unwrap();
        other.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    fn image_move_fixture(
        c: &mut ParseControl,
    ) -> (ParserArenaSession, ParserNode, ParserNode, ParserNode) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<form><form><img>");
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, c).unwrap();
        let (mut session, body) = form_session(c);
        let old = form_creation(&mut session, &mut tokenizer, c, &body, None);
        let new = form_creation(&mut session, &mut tokenizer, c, &body, None);
        session.insert(c, &body, &old, None, None).unwrap();
        session.insert(c, &body, &new, None, None).unwrap();
        let image = form_creation(&mut session, &mut tokenizer, c, &body, Some(&old));
        session.insert(c, &body, &image, None, None).unwrap();
        tokenizer.release(c).unwrap();
        (session, image, old, new)
    }
    fn retained_image_state(
        session: &ParserArenaSession,
        image: &ParserNode,
    ) -> FormAssociationState {
        session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::FormAssociation { node, state } if *node == Some(image.index) => {
                    Some(*state)
                }
                _ => None,
            })
            .unwrap()
    }
    #[test]
    fn image_effect_work_exact_and_one_short_keep_real_completed_or_unfreezable_owners() {
        let mut measured = control();
        let (mut session, image, _, target) = image_move_fixture(&mut measured);
        let start = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .insert(&mut measured, &target, &image, None, None)
            .unwrap();
        let work = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - start;
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        // Empty receive tail: runner reads2, actual fixed Option receiver
        // copy/layout + write1, empty OwnedVec pop visit1 (no Some move).
        let tail = 2 + mem::size_of::<Option<select::EffectFrame>>() as u64 + 1 + 1;
        for remaining in [work, work - tail - 1] {
            let mut c = control();
            let (mut session, image, _, target) = image_move_fixture(&mut c);
            let before_nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
            let unused = limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(WorkCategory::StateStep, unused - remaining)
                .unwrap();
            let result = session.insert(&mut c, &target, &image, None, None);
            if remaining == work {
                assert_eq!(result, Ok(()));
                assert!(session.active().unwrap().effects.form.is_none());
                let state = retained_image_state(&session, &image);
                assert_eq!(state.owner, Some(target.index));
                assert!(!state.parser_inserted);
                // Exhausted successful work budget: request one actual credit
                // to latch terminal disposal without pretending cleanup is free.
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
            } else {
                let stop = c.stop().unwrap();
                assert!(matches!(
                    stop,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(result, Err(SessionError::Stop(stop)));
                assert_eq!(
                    session.active().unwrap().records[image.index].parent,
                    Some(target.index)
                );
                let state = retained_image_state(&session, &image);
                assert_eq!(state.owner, Some(target.index));
                assert!(!state.parser_inserted);
                let storage = session.active().unwrap();
                assert!(storage.effects.form.is_none());
                let (list, cursor) = storage
                    .effects
                    .retained_post_connection()
                    .expect("specific unfinished final postconnection clear");
                assert_eq!(cursor, 1);
                assert!(matches!(
                    storage.payloads.values[list],
                    RetainedPayload::Retired
                ));
                assert_eq!(
                    session.image_form_state(&mut c, &image),
                    Err(SessionError::Stop(stop))
                );
                assert_eq!(session.root(&mut c).unwrap_err(), SessionError::Stop(stop));
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
                assert_eq!(c.checkpoint(), Err(stop));
            }
            let high = c
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water;
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let stop = c.stop();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                before_nodes
            );
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                used
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .high_water,
                high
            );
        }
    }
    #[test]
    fn image_metadata_admission_failure_preserves_original_tag_and_arena_before_take() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<img>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .auxiliary_entries(13)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        session.insert(&mut c, &root, &html, None, None).unwrap();
        // Produce the actual lexical owner before filling the final retained
        // slots: tokenizer pending events have their own transient admission.
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("real img token")
        };
        // This actual detached allocation fills the last five retained slots.
        // Attaching it would require an unrelated temporary child-order entry;
        // the image's actual intended parent is the attached HTML container.
        let _body = session.allocate_implicit_body(&mut c, None).unwrap();
        // Root/html/body own three actual record tickets, six payload slots and four document facts.
        // No auxiliary entry remains for the image's required metadata slot.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            13
        );
        let name_backing = Arc::as_ptr(tag.name.value.text().backing());
        let original_origins: Vec<_> = tag
            .name
            .origins
            .as_slice()
            .iter()
            .map(|origin| {
                (
                    origin.start,
                    origin.end,
                    origin.source.start(),
                    origin.source.end(),
                )
            })
            .collect();
        let original_topology: Vec<_> = session
            .active()
            .unwrap()
            .records
            .iter()
            .map(|record| (record.parent, record.children.clone()))
            .collect();
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
        let mut input = Some(tag);
        let result = session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &html,
            None,
            crate::construction::parser_session::CreationContext::Ordinary,
        );
        let stop = c.stop().unwrap();
        let retained = input.as_ref().map(|tag| {
            (
                tag.kind,
                Arc::as_ptr(tag.name.value.text().backing()),
                tag.name.value.text().code_units().to_vec(),
                tag.attributes.len(),
                tag.name
                    .origins
                    .as_slice()
                    .iter()
                    .map(|origin| {
                        (
                            origin.start,
                            origin.end,
                            origin.source.start(),
                            origin.source.end(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        });
        let topology: Vec<_> = session
            .active()
            .unwrap()
            .records
            .iter()
            .map(|record| (record.parent, record.children.clone()))
            .collect();
        assert_eq!(
            result.err(),
            Some(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                limit: 13,
                current: 13,
                requested_delta: 2,
            }))
        );
        assert_eq!(topology, original_topology);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedEdges).current,
            edges
        );
        // Reclaim actual surviving owners before the regression assertion. A
        // retained Tag must still be releasable by its original sole controller.
        release_formatting_tag(&mut input, &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(
            retained,
            Some((
                TagKind::Start,
                name_backing,
                vec![u16::from(b'i'), u16::from(b'm'), u16::from(b'g')],
                0,
                original_origins,
            )),
            "required image metadata admission precedes rejectable lexical take"
        );
    }
    #[test]
    fn image_creation_original_rejections_precede_any_metadata_slot_or_capacity() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("</img><img x=one><img x=two>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let payloads = session.active().unwrap().payloads.values.len();
        let capacity = session.active().unwrap().payloads.values.capacity();
        let auxiliary = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("actual end tag")
        };
        let mut end = Some(tag);
        assert_eq!(
            session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut end,
                    &body,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                )
                .unwrap_err(),
            SessionError::WrongToken
        );
        assert!(end.is_some());
        assert_eq!(session.active().unwrap().payloads.values.len(), payloads);
        assert_eq!(
            session.active().unwrap().payloads.values.capacity(),
            capacity
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            auxiliary
        );
        assert!(c.stop().is_none());
        release_formatting_tag(&mut end, &mut c).unwrap();
        let TokenKind::Tag(mut first) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("first actual start tag")
        };
        let TokenKind::Tag(mut second) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("second actual start tag")
        };
        // Independently owned actual parsed fields with the same source/control.
        // This invalid private input has two exact x names, not a forged token
        // duplicate flag or aliased backing that could bypass uniqueness.
        let duplicate = second.attributes.try_pop(&mut c).unwrap().unwrap();
        first.attributes.try_push(&mut c, duplicate).unwrap();
        let mut second = Some(second);
        let mut input = Some(first);
        let alias = input.as_ref().unwrap().name.value.text().clone();
        assert_eq!(
            session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut input,
                    &body,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                )
                .unwrap_err(),
            SessionError::AliasedPayload
        );
        assert!(input.is_some());
        drop(alias);
        assert_eq!(
            session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut input,
                    &body,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                )
                .unwrap_err(),
            SessionError::DuplicateAttribute
        );
        assert_eq!(input.as_ref().unwrap().attributes.len(), 2);
        assert_eq!(session.active().unwrap().payloads.values.len(), payloads);
        assert_eq!(
            session.active().unwrap().payloads.values.capacity(),
            capacity
        );
        assert_eq!(session.active().unwrap().records.len(), 3);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            auxiliary
        );
        assert!(c.stop().is_none());
        release_formatting_tag(&mut input, &mut c).unwrap();
        release_formatting_tag(&mut second, &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn image_creation_binding_stop_retains_initialized_slot_and_actual_node_for_cleanup() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<img>");
        let mut measured = control();
        measured.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut measured).unwrap();
        let (mut session, body) = form_session(&mut measured);
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut measured) else {
            panic!("actual img")
        };
        let start = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session
            .allocate_html_tag_for_creation(
                &mut measured,
                &mut Some(tag),
                &body,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap();
        let complete = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - start;
        lexer.release(&mut measured).unwrap();
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (mut session, body) = form_session(&mut c);
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("actual img")
        };
        let mut input = Some(tag);
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        // The final binding has a real last RecordWrite credit after its visit,
        // layout admission and Option<usize> copy. Deny only that final write.
        c.charge(WorkCategory::StateStep, unused - (complete - 1))
            .unwrap();
        let error = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            )
            .unwrap_err();
        let stop = c.stop().unwrap();
        assert_eq!(error, SessionError::Stop(stop));
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 1,
                ..
            }
        ));
        assert!(input.is_none());
        let storage = session.active().unwrap();
        assert_eq!(storage.records.len(), 4);
        assert_eq!(storage.records[3].parent, None);
        assert!(
            storage
                .payloads
                .values
                .iter()
                .any(|entry| matches!(entry, RetainedPayload::FormAssociation { node: None, .. }))
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            19
        );
        assert_eq!(session.root(&mut c).unwrap_err(), SessionError::Stop(stop));
        assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        assert_eq!(
            session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut input,
                    &body,
                    None,
                    crate::construction::parser_session::CreationContext::Ordinary
                )
                .unwrap_err(),
            SessionError::Stop(stop)
        );
        let high = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            4
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            high
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
    }
    #[test]
    fn image_creation_actual_metadata_slot_has_exact_auxiliary_admission_and_terminal_cleanup() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for cap in [19, 18] {
            let source = crate::HtmlSource::utf8("<img>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut session, body) = form_session(&mut c);
            // Root plus html/body: 3 actual records/tickets, 6 payload slots and 4 document facts.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                13
            );
            let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("img")
            };
            let mut input = Some(tag);
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &body,
                None,
                crate::construction::parser_session::CreationContext::Ordinary,
            );
            assert!(input.is_none());
            if cap == 19 {
                let image = result.unwrap();
                assert_form_owner(&session, &mut c, &image, None, false);
                // Actual name/namespace/attributes + record ticket + metadata.
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    19
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        limit: 18,
                        ..
                    }))
                ));
                // Metadata, name, namespace and attributes are initialized;
                // the next actual record-ticket entry is the rejected slot.
                assert_eq!(session.active().unwrap().records.len(), 3);
                assert_eq!(session.active().unwrap().payloads.values.len(), 15);
                assert!(
                    session
                        .active()
                        .unwrap()
                        .payloads
                        .values
                        .iter()
                        .any(|entry| matches!(
                            entry,
                            RetainedPayload::FormAssociation { node: None, .. }
                        ))
                );
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(c.stop().unwrap()))
                );
            }
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn image_move_rejections_and_cancel_preserve_links_facts_and_first_reason() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let (mut session, image, old, target) = image_move_fixture(&mut c);
        let before = retained_image_state(&session, &image);
        let mut other =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let foreign = other.root(&mut c).unwrap();
        assert_eq!(
            session.insert(&mut c, &foreign, &image, None, None),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(retained_image_state(&session, &image), before);
        assert!(session.active().unwrap().effects.form.is_none());
        assert_eq!(
            session.active().unwrap().records[image.index].parent,
            Some(2)
        );
        cancellation.cancel();
        assert_eq!(
            session.insert(&mut c, &target, &image, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(retained_image_state(&session, &image), before);
        assert!(session.active().unwrap().effects.form.is_none());
        assert_eq!(
            session.image_form_owner(&mut c, &image).unwrap_err(),
            SessionError::Stop(ParseStop::Cancelled)
        );
        assert_eq!(
            session.remove(&mut c, &old, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        other.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn image_real_cancellation_between_removal_and_insertion_preserves_completed_removal() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let (mut session, image, old, target) = image_move_fixture(&mut c);
        session.active_mut().unwrap().cancel_after_removal = Some(cancellation);
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        assert_eq!(
            session.insert(&mut c, &target, &image, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        let storage = session.active().unwrap();
        assert_eq!(storage.records[image.index].parent, Some(target.index));
        assert!(!storage.records[old.index].children.contains(&image.index));
        assert_eq!(storage.records[target.index].children, [image.index]);
        assert_eq!(
            storage.effects.form.unwrap().stage,
            FormEffectStage::Adoption
        );
        let state = retained_image_state(&session, &image);
        assert_eq!(state.owner, None);
        assert!(!state.parser_inserted);
        assert_eq!(state.evidence, FormAssociationEvidence::Reset);
        assert_eq!(
            session.image_form_state(&mut c, &image),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(
            session.insert(&mut c, &old, &image, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
    }

    #[test]
    fn image_removal_clears_parser_flag_before_ancestor_work_can_stop() {
        let mut c = control();
        let (mut session, image, old, target) = image_move_fixture(&mut c);
        for node in [&image, &old, &target] {
            session.validate_node(&mut c, node).unwrap();
        }
        let old_parent = session.active().unwrap().records[image.index].parent;
        session
            .active_mut()
            .unwrap()
            .insert_indices(&mut c, target.index, image.index, None)
            .unwrap();
        let storage = session.active().unwrap();
        let effects = storage.effects.form.unwrap();
        assert_eq!(effects.root, image.index);
        assert_eq!(effects.next, Some(image.index));
        assert!(matches!(effects.stage, FormEffectStage::Removal));
        assert!(matches!(effects.pass, FormEffectPass::Local));
        assert_eq!(effects.old_parent, old_parent);
        assert_eq!(effects.new_parent, Some(target.index));
        assert!(effects.insert);
        // Removal view img parent=None costs1; old form→body→html→Document
        // then None costs4. Runner2 + phase read1/copyF + registry scanL/copyS
        // + roots5 + setter read1/copyS/write1 =F+L+2S+10. The next ancestor
        // read1 fails after reset has cleared the flag, before clearing owner.
        let scan = storage.payloads.values.iter().position(|payload| matches!(payload, RetainedPayload::FormAssociation { node: Some(node), .. } if *node == image.index)).unwrap() as u64 + 1;
        let prefix = mem::size_of::<Option<FormEffects>>() as u64
            + scan
            + 2 * mem::size_of::<FormAssociationState>() as u64
            + 10;
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, unused - prefix).unwrap();
        let error = session
            .active_mut()
            .unwrap()
            .drain_construction_effects(&mut c)
            .unwrap_err();
        let stop = c.stop().unwrap();
        assert_eq!(error, SessionError::Stop(stop));
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 1,
                ..
            }
        ));
        assert_eq!(
            session.active().unwrap().records[image.index].parent,
            Some(target.index)
        );
        let retained = retained_image_state(&session, &image);
        assert!(
            !retained.parser_inserted,
            "reset step1 precedes ordinary ancestor resolution"
        );
        assert_eq!(
            retained.owner,
            Some(old.index),
            "later clear-owner step has not run"
        );
        assert_eq!(retained.evidence, FormAssociationEvidence::Reset);
        assert_eq!(
            session.active().unwrap().effects.form.unwrap().stage,
            FormEffectStage::Removal
        );
        assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn image_self_reference_insertion_runs_removal_before_insertion_with_unchanged_links() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for subtree in [false, true] {
            let source = crate::HtmlSource::utf8(if subtree {
                "<form><div><img>"
            } else {
                "<form><img>"
            });
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let (mut session, body) = form_session(&mut c);
            let form = form_creation(&mut session, &mut tokenizer, &mut c, &body, None);
            session.insert(&mut c, &body, &form, None, None).unwrap();
            let parent = if subtree {
                let div = form_creation(&mut session, &mut tokenizer, &mut c, &body, Some(&form));
                session.insert(&mut c, &body, &div, None, None).unwrap();
                div
            } else {
                body.clone()
            };
            let image = form_creation(&mut session, &mut tokenizer, &mut c, &parent, Some(&form));
            session.insert(&mut c, &parent, &image, None, None).unwrap();
            assert_form_owner(&session, &mut c, &image, Some(&form), true);
            let moved = if subtree { &parent } else { &image };
            let old_order = session.active().unwrap().records[body.index]
                .children
                .clone();
            session
                .insert(&mut c, &body, moved, Some(moved), None)
                .unwrap();
            assert_eq!(
                session.active().unwrap().records[body.index].children,
                old_order
            );
            let result = session.image_form_state(&mut c, &image).unwrap();
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            // Pre-insert reference adjustment preserves final position, but
            // adoption still removes the actual root before inserting it again.
            assert!(!result.parser_inserted);
            assert_eq!(result.owner, None);
            assert_eq!(result.evidence, FormAssociationEvidence::Reset);
        }
    }

    #[test]
    fn table_implied_names_are_real_managed_native_elements_without_lexical_origins() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let nodes = [
            session.allocate_implicit_colgroup(&mut c, None).unwrap(),
            session.allocate_implicit_tbody(&mut c, None).unwrap(),
            session.allocate_implicit_tr(&mut c, None).unwrap(),
        ];
        for node in &nodes {
            session.insert(&mut c, &root, node, None, None).unwrap();
        }
        // Three actual names (8+5+2) and three exact HTML URI strings (28 each).
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            99
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            4
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        let storage = session.active().unwrap();
        assert!(storage.standalone_ineligibility.is_none());
        assert!(storage.payloads.values.iter().all(|entry| !matches!(
            entry,
            RetainedPayload::String {
                origins: Some(_),
                ..
            }
        )));
        let mut result = session.freeze(&mut c).unwrap();
        let fragment = result.fragment().unwrap();
        let actual: Vec<_> = fragment
            .root()
            .children()
            .map(|id| {
                let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
                    panic!("native table element")
                };
                assert!(element.name().prefix().is_none());
                assert_eq!(element.attributes().len(), 0);
                assert_eq!(
                    element.name().namespace().uri().unwrap().code_units(),
                    b"http://www.w3.org/1999/xhtml"
                        .iter()
                        .map(|byte| u16::from(*byte))
                        .collect::<Vec<_>>()
                );
                element.name().local_name().code_units().to_vec()
            })
            .collect();
        assert_eq!(
            actual,
            vec![
                vec![99, 111, 108, 103, 114, 111, 117, 112],
                vec![116, 98, 111, 100, 121],
                vec![116, 114],
            ]
        );
        result.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn table_implied_names_have_exact_managed_unit_caps_and_reclaim_partial_backing() {
        for (kind, units) in [(0, 36), (1, 33), (2, 30)] {
            for cap in [units, units - 1] {
                let mut c = ParseControl::new(
                    limits()
                        .into_builder()
                        .managed_utf16_units(cap)
                        .finish()
                        .unwrap(),
                    HtmlParseCancellation::new(),
                );
                let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
                let result = match kind {
                    0 => session.allocate_implicit_colgroup(&mut c, None),
                    1 => session.allocate_implicit_tbody(&mut c, None),
                    _ => session.allocate_implicit_tr(&mut c, None),
                };
                if cap == units {
                    assert!(result.is_ok());
                    assert_eq!(
                        c.usage()
                            .resource(HtmlParseResource::ManagedUtf16Units)
                            .current,
                        units
                    );
                    assert_eq!(session.active().unwrap().records.len(), 2);
                } else {
                    assert!(
                        matches!(result, Err(SessionError::Stop(ParseStop::LimitExceeded { resource: HtmlParseResource::ManagedUtf16Units, limit, .. })) if limit == cap)
                    );
                    assert_eq!(session.active().unwrap().records.len(), 1);
                    assert_eq!(
                        session.freeze(&mut c).err(),
                        Some(SessionError::Stop(c.stop().unwrap()))
                    );
                }
                let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
                let high = c
                    .usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .high_water;
                let stop = c.stop();
                session.release(&mut c).unwrap();
                storage_zero(&c);
                assert_eq!(c.stop(), stop);
                assert_eq!(
                    c.usage().resource(HtmlParseResource::ManagedNodes).current,
                    nodes
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedCapacityBytes)
                        .high_water,
                    high
                );
            }
        }
    }

    #[test]
    fn table_document_mode_reads_actual_root_and_has_independent_finite_cost() {
        for mode in [
            HtmlDocumentMode::NoQuirks,
            HtmlDocumentMode::LimitedQuirks,
            HtmlDocumentMode::Quirks,
        ] {
            let mut c = control();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Document(mode)).unwrap();
            let visits = c.usage().work(WorkCategory::RecordVisit);
            let copies = c.usage().work(WorkCategory::CopyByte);
            let writes = c.usage().work(WorkCategory::RecordWrite);
            assert_eq!(session.document_mode(&mut c), Ok(mode));
            // Form readiness plus five intrinsic-owner reads + root discriminant, then the mode copy.
            assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 7);
            assert_eq!(
                c.usage().work(WorkCategory::CopyByte) - copies,
                mem::size_of::<HtmlDocumentMode>() as u64
            );
            assert_eq!(c.usage().work(WorkCategory::RecordWrite), writes);
            session
                .set_document_mode(&mut c, HtmlDocumentMode::LimitedQuirks)
                .unwrap();
            assert_eq!(
                session.document_mode(&mut c),
                Ok(HtmlDocumentMode::LimitedQuirks)
            );
            let mut result = session.freeze(&mut c).unwrap();
            assert_eq!(
                result.document().unwrap().mode(),
                HtmlDocumentMode::LimitedQuirks
            );
            result.release(&mut c).unwrap();
            storage_zero(&c);
        }
        let mut c = control();
        let mut fragment = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        assert_eq!(
            fragment.document_mode(&mut c),
            Err(SessionError::NotDocument)
        );
        assert!(c.stop().is_none());
        fragment.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn table_producers_and_mode_reject_foreign_cancel_and_one_short_before_effects() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut foreign = control();
        let wrong = SessionError::Stop(ParseStop::InternalInvariant(
            crate::parser::control::Invariant::ForeignControl,
        ));
        assert_eq!(
            session
                .allocate_implicit_colgroup(&mut foreign, None)
                .unwrap_err(),
            wrong
        );
        assert_eq!(session.document_mode(&mut foreign), Err(wrong));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(
            session.document_mode(&mut c),
            Ok(HtmlDocumentMode::NoQuirks)
        );
        cancellation.cancel();
        assert_eq!(
            session.allocate_implicit_tr(&mut c, None).unwrap_err(),
            SessionError::Stop(ParseStop::Cancelled)
        );
        assert_eq!(
            session.document_mode(&mut c),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(session.active().unwrap().records.len(), 1);
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        // Independent getter envelope: seven actual reads and fixed mode copy.
        let complete = 7 + mem::size_of::<HtmlDocumentMode>() as u64;
        for remaining in [complete, complete - 1] {
            let mut c = control();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::Quirks))
                    .unwrap();
            let unused = limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(WorkCategory::StateStep, unused - remaining)
                .unwrap();
            let result = session.document_mode(&mut c);
            if remaining == complete {
                assert_eq!(result, Ok(HtmlDocumentMode::Quirks));
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
            } else {
                let stop = c.stop().unwrap();
                assert_eq!(result, Err(SessionError::Stop(stop)));
                assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            }
            assert!(matches!(
                session.active().unwrap().records[0].kind,
                HtmlNodeKind::Document(HtmlDocumentMode::Quirks)
            ));
            let stop = c.stop();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }

    pub(super) fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(1_000)
            .buffered_utf16_units(1_000)
            .token_attributes(1_000)
            .auxiliary_entries(1_000)
            .managed_nodes(1_000)
            .managed_edges(1_000)
            .managed_attributes(1_000)
            .managed_utf16_units(100_000)
            .source_map_entries(1_000)
            .source_map_payload_units(1_000)
            .diagnostics(1_000)
            .diagnostic_payload_units(1_000)
            .context_effects(1_000)
            .open_elements(1_000)
            .tree_depth(1_000)
            .managed_capacity_bytes(1_000_000)
            .peak_transient_bytes(2_000_000)
            .atomic_layout_bytes(1_000_000)
            .work_credits(10_000_000)
            .finish()
            .unwrap()
    }
    fn control() -> ParseControl {
        ParseControl::new(limits(), HtmlParseCancellation::new())
    }
    fn element(namespace: Namespace, local: &str) -> HtmlElement {
        HtmlElement {
            name: QualifiedName::new(namespace, None, local),
            attributes: OrderedAttributes::new(Vec::new()).unwrap(),
        }
    }
    fn html(local: &str) -> HtmlElement {
        element(Namespace::html(), local)
    }
    fn frozen(mut session: ParserArenaSession, control: &mut ParseControl) -> FrozenArena {
        match session.freeze(control) {
            Ok(value) => value,
            Err(failure) => panic!("freeze failed: {:?}", failure),
        }
    }
    fn locals(fragment: &HtmlFragment) -> Vec<String> {
        fragment
            .root()
            .children()
            .map(|id| {
                let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
                    panic!("expected element");
                };
                element.name().local_name().to_utf8().unwrap()
            })
            .collect()
    }

    #[test]
    fn implied_head_and_both_root_attribute_paths_keep_order_exact_backing_and_origins() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for implied in [false, true] {
            let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
                "<html id=old><html id=ignored x:id=p a='"
                    .encode_utf16()
                    .chain([0xd800, 0xd83d, 0xde00])
                    .chain("' a=last z=end>".encode_utf16())
                    .collect::<Vec<_>>(),
            ));
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
                panic!("first tag")
            };
            let root = if implied {
                let root = session.allocate_implicit_html(&mut c).unwrap();
                session
                    .add_attributes_if_missing(&mut c, &root, &mut Some(first))
                    .unwrap();
                root
            } else {
                session.allocate_html_tag(&mut c, &mut Some(first)).unwrap()
            };
            let doc = session.root(&mut c).unwrap();
            session.insert(&mut c, &doc, &root, None, None).unwrap();
            let head = session.allocate_implicit_head(&mut c, None).unwrap();
            session.insert(&mut c, &root, &head, None, None).unwrap();
            let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
                panic!("second tag")
            };
            assert!(second.had_duplicate_attributes);
            let expected_origin = second.attributes.as_slice()[2].value.origins.as_slice()[0]
                .source
                .clone();
            let mut second = Some(second);
            session
                .add_attributes_if_missing(&mut c, &root, &mut second)
                .unwrap();
            assert!(second.is_none());
            let storage = session.active().unwrap();
            let HtmlNodeKind::Element(element) = &storage.records[root.index].kind else {
                panic!("root")
            };
            assert_eq!(
                element
                    .attributes()
                    .as_slice()
                    .iter()
                    .map(|a| a.name().local_name().to_utf8().unwrap())
                    .collect::<Vec<_>>(),
                ["id", "x:id", "a", "z"]
            );
            assert_eq!(
                element.attributes().as_slice()[0]
                    .value()
                    .to_utf8()
                    .unwrap(),
                "old"
            );
            assert_eq!(
                element.attributes().as_slice()[2].value().code_units(),
                [0xd800, 0xd83d, 0xde00]
            );
            assert!(
                element.attributes().as_slice().iter().all(|a| a
                    .name()
                    .namespace()
                    .uri()
                    .is_none())
            );
            assert!(storage.payloads.values.iter().any(|v| matches!(v, RetainedPayload::String { origins: Some(origins), location: StringLocation::AttributeValue(_, 2), .. } if storage.observed_origins(origins, &mut c).unwrap()[0].source == expected_origin)));
            assert_eq!(storage.usage.attributes, 4);
            t.release(&mut c).unwrap();
            let mut frozen = session.freeze(&mut c).unwrap();
            let document = frozen.document().unwrap();
            let root_id = document.root().children().next().unwrap();
            let HtmlNodeKind::Element(element) = document.node(&root_id).unwrap().kind() else {
                panic!("root")
            };
            assert_eq!(element.attributes().len(), 4);
            let head_id = document.node(&root_id).unwrap().children().next().unwrap();
            let HtmlNodeKind::Element(head) = document.node(&head_id).unwrap().kind() else {
                panic!("head")
            };
            assert_eq!(head.name().local_name().to_utf8().unwrap(), "head");
            assert!(head.attributes().is_empty());
            frozen.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    fn observed_root_merge(new_attributes: bool) -> crate::parser::control::OwnedSpanObservation {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // A real lexer produces every field and original source association.
        // Sixteen short attributes fit the existing finite fixture capacities.
        let attributes = (0..16).map(|i| format!(" a{i:02}=v")).collect::<String>();
        let input = if new_attributes {
            format!("<html><html{attributes}>")
        } else {
            format!("<html{attributes}><html{attributes}>")
        };
        let source = crate::HtmlSource::utf8(input);
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("actual first tag")
        };
        let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("actual second tag")
        };
        let mut second = Some(second);
        c.clear_owned_span_observation();
        session
            .add_attributes_if_missing(&mut c, &root, &mut second)
            .unwrap();
        let observed = c.owned_span_observation();
        assert!(second.is_none());
        assert_eq!(c.stop(), None);
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[root.index].kind
        else {
            panic!("actual root")
        };
        assert_eq!(element.attributes().len(), 16);
        for (i, attribute) in element.attributes().as_slice().iter().enumerate() {
            assert_eq!(
                attribute.name().local_name().to_utf8().unwrap(),
                format!("a{i:02}")
            );
            assert_eq!(attribute.value().to_utf8().unwrap(), "v");
        }
        assert_eq!(Arc::strong_count(element.attributes().backing()), 2);
        assert_eq!(Arc::weak_count(element.attributes().backing()), 1);
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        observed
    }

    #[test]
    fn wide_real_attribute_merge_respects_declared_poll_span() {
        let observed = observed_root_merge(true);
        assert!(observed.polls > 0);
        assert_eq!(observed.append_slots, 16);
        assert!(
            observed.largest_append_copy_bytes <= 256,
            "actual owned attribute copy exceeds the selected 256-credit poll span: {observed:?}"
        );
        assert!(
            observed.largest_registry_credits_between_polls <= 256,
            "actual owned registry iterations exceed the selected 256-credit poll span: {observed:?}"
        );
    }

    #[test]
    fn duplicate_only_root_merge_respects_declared_poll_span() {
        let observed = observed_root_merge(false);
        assert!(observed.polls > 0);
        assert_eq!(observed.append_slots, 0);
        assert!(
            observed.largest_registry_credits_between_polls <= 256,
            "actual owned registry iterations exceed the selected 256-credit poll span: {observed:?}"
        );
    }

    fn prepare_real_merge<'source>(
        source: &'source crate::HtmlSource,
        c: &mut ParseControl,
    ) -> (
        ParserArenaSession,
        ParserNode,
        PreparedAttributeMerge,
        crate::parser::tokenizer::tests::LexerFixture<'source>,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        c.admit_source(source).unwrap();
        let mut tokenizer = LexerFixture::new(source, State::Data, None, c).unwrap();
        let mut session = ParserArenaSession::new(c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, c) else {
            panic!("actual first tag")
        };
        let root = session.allocate_html_tag(c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, c) else {
            panic!("actual second tag")
        };
        let mut input = Some(second);
        let prepared = session
            .active_mut()
            .unwrap()
            .prepare_attribute_merge(c, &root, &mut input)
            .unwrap()
            .unwrap();
        assert!(input.is_none());
        (session, root, prepared, tokenizer)
    }

    fn attribute_slot_credits() -> u64 {
        // Hand-derived fixed operation: registry remove/restore, one progress
        // write, actual wrapper checks/restoration and one pop/push. There is no
        // whole-vector credit or polling assumption in this expectation.
        (3 + // same-owner identity is uncharged; staging/last-slot2 + id length mismatch1
            2 + 3
            + mem::size_of::<Option<AttributeStorage>>()
            + 2
            + mem::size_of::<OrderedAttributes>()
            + mem::size_of::<usize>()
            + 1
            + 3
            + 9
            + 2 * mem::size_of::<Weak<Vec<Attribute>>>()
            + 2
            + 1
            + 2
            + 2 * mem::size_of::<Attribute>()
            + 3) as u64
    }

    fn assert_partial_merge_storage(
        session: &ParserArenaSession,
        c: &mut ParseControl,
        root: &ParserNode,
        prepared: &PreparedAttributeMerge,
        moved: usize,
    ) {
        let storage = session.active().unwrap();
        let RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
            values,
            moved: actual,
            ..
        })) = &storage.payloads.values[prepared.stage]
        else {
            panic!("actual retained transfer phase")
        };
        assert_eq!(*actual, moved);
        assert_eq!(values.len(), 2 - moved);
        let HtmlNodeKind::Element(element) = &storage.records[root.index].kind else {
            panic!("actual root")
        };
        let expected = [("id", "old"), ("a", "one"), ("b", "two")];
        assert_eq!(element.attributes().len(), 1 + moved);
        for (attribute, (name, value)) in element.attributes().as_slice().iter().zip(expected) {
            assert_eq!(attribute.name().local_name().to_utf8().unwrap(), name);
            assert_eq!(attribute.value().to_utf8().unwrap(), value);
        }
        assert_eq!(Arc::strong_count(element.attributes().backing()), 2);
        assert_eq!(Arc::weak_count(element.attributes().backing()), 1);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            3
        );
        // Checks the real destination/staging lookup for every original ticket,
        // including those already rebound and those still using the redirect.
        storage.payloads.check_aliases(&storage.records, c).unwrap();
    }

    fn dispose_stopped_merge(
        mut session: ParserArenaSession,
        tokenizer: crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
    ) {
        let stop = c.stop().expect("real terminal boundary");
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let capacity_peak = c.usage().actual_capacity_high_water;
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        let attribute_peak = c
            .usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .high_water;
        assert!(matches!(session.freeze(c), Err(SessionError::Stop(actual)) if actual == stop));
        tokenizer.release(c).unwrap();
        session.release(c).unwrap();
        storage_zero(c);
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(c.usage().actual_capacity_high_water, capacity_peak);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .high_water,
            attribute_peak
        );
    }

    #[test]
    fn exact_and_one_short_slot_work_preserve_actual_partial_transfer_and_cleanup() {
        let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
        let mut c = control();
        let (session, _, _, tokenizer) = prepare_real_merge(&source, &mut c);
        let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
        tokenizer.release(&mut c).unwrap();
        let mut session = session;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        for short in [false, true] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .work_credits(setup + attribute_slot_credits() - u64::from(short))
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            let (mut session, root, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let result = session.active_mut().unwrap().edit_attributes(
                &mut c,
                prepared.target,
                prepared.container,
                prepared.stage,
                1,
                true,
            );
            if short {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
            } else {
                result.unwrap();
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current - before,
                    attribute_slot_credits()
                );
                // The next real slot cannot cross the exhausted budget.
                assert!(
                    session
                        .active_mut()
                        .unwrap()
                        .edit_attributes(
                            &mut c,
                            prepared.target,
                            prepared.container,
                            prepared.stage,
                            1,
                            true
                        )
                        .is_err()
                );
            }
            assert_partial_merge_storage(&session, &mut c, &root, &prepared, usize::from(!short));
            dispose_stopped_merge(session, tokenizer, &mut c);
        }
    }

    #[test]
    fn exact_and_one_short_rebind_work_preserve_redirect_and_terminal_cleanup() {
        let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
        let mut c = control();
        let (mut session, _, _, tokenizer) = prepare_real_merge(&source, &mut c);
        let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        let rebind_credits = (1 + 2 * mem::size_of::<StringLocation>() + 1) as u64;
        for short in [false, true] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .work_credits(
                        setup + 2 * attribute_slot_credits() + rebind_credits - u64::from(short),
                    )
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            let (mut session, root, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
            for _ in 0..2 {
                session
                    .active_mut()
                    .unwrap()
                    .edit_attributes(
                        &mut c,
                        prepared.target,
                        prepared.container,
                        prepared.stage,
                        1,
                        true,
                    )
                    .unwrap();
            }
            let index = prepared.stage + 1; // real first registered incoming name
            let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let result = session.active_mut().unwrap().rebind_attribute_location(
                &mut c,
                index,
                prepared.stage,
                prepared.container,
                prepared.start,
            );
            if short {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
            } else {
                result.unwrap();
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current - before,
                    rebind_credits
                );
                assert!(
                    session
                        .active_mut()
                        .unwrap()
                        .rebind_attribute_location(
                            &mut c,
                            index + 1,
                            prepared.stage,
                            prepared.container,
                            prepared.start
                        )
                        .is_err()
                );
            }
            let RetainedPayload::String {
                location,
                origins: Some(origins),
                ..
            } = &session.active().unwrap().payloads.values[index]
            else {
                panic!("actual original name")
            };
            assert!(
                matches!(location, StringLocation::AttributeName(owner, offset)
                if *owner == if short { prepared.stage } else { prepared.container }
                    && *offset == if short { 1 } else { 2 })
            );
            let evidence = session.active().unwrap().evidence.as_ref().unwrap();
            let provenance::GroupBody::Leaf { start, length, .. } =
                evidence.require_group(origins).unwrap().body
            else {
                panic!("original retained leaf");
            };
            assert_eq!(length, 1);
            // Fixed admitted evidence is inspected by the test harness, without
            // attempting a new successful read through the stopped controller.
            assert_eq!(
                evidence
                    .fixed_original_origin(start)
                    .unwrap()
                    .source
                    .start(),
                25
            );
            assert_eq!(
                evidence.fixed_original_origin(start).unwrap().source.end(),
                26
            );
            assert_partial_merge_storage(&session, &mut c, &root, &prepared, 2);
            dispose_stopped_merge(session, tokenizer, &mut c);
        }
    }

    #[test]
    fn cancellation_between_real_slot_and_rebind_operations_retains_exact_owner_state() {
        let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
        for during_rebind in [false, true] {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            let (mut session, root, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
            let moved = if during_rebind { 2 } else { 1 };
            for _ in 0..moved {
                session
                    .active_mut()
                    .unwrap()
                    .edit_attributes(
                        &mut c,
                        prepared.target,
                        prepared.container,
                        prepared.stage,
                        1,
                        true,
                    )
                    .unwrap();
            }
            if during_rebind {
                session
                    .active_mut()
                    .unwrap()
                    .rebind_attribute_location(
                        &mut c,
                        prepared.stage + 1,
                        prepared.stage,
                        prepared.container,
                        prepared.start,
                    )
                    .unwrap();
            }
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            cancellation.cancel();
            let result = if during_rebind {
                session.active_mut().unwrap().rebind_attribute_location(
                    &mut c,
                    prepared.stage + 2,
                    prepared.stage,
                    prepared.container,
                    prepared.start,
                )
            } else {
                session.active_mut().unwrap().edit_attributes(
                    &mut c,
                    prepared.target,
                    prepared.container,
                    prepared.stage,
                    1,
                    true,
                )
            };
            assert_eq!(result, Err(SessionError::Stop(ParseStop::Cancelled)));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
            assert_partial_merge_storage(&session, &mut c, &root, &prepared, moved);
            dispose_stopped_merge(session, tokenizer, &mut c);
        }
    }

    #[test]
    fn merge_rejects_external_alias_and_foreign_origin_before_consuming_input() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<html id=x><html a=y>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let mut incoming = Some(second);
        let alias = match &session.active().unwrap().records[root.index].kind {
            HtmlNodeKind::Element(e) => e.attributes().clone(),
            _ => panic!("root"),
        };
        assert_eq!(
            session.add_attributes_if_missing(&mut c, &root, &mut incoming),
            Err(SessionError::AliasedPayload)
        );
        assert!(incoming.is_some());
        drop(alias);
        let weak = match &session.active().unwrap().records[root.index].kind {
            HtmlNodeKind::Element(e) => Arc::downgrade(e.attributes().backing()),
            _ => panic!("root"),
        };
        assert_eq!(
            session.add_attributes_if_missing(&mut c, &root, &mut incoming),
            Err(SessionError::AliasedPayload)
        );
        assert!(incoming.is_some());
        drop(weak);
        // Correct execution tickets alone cannot qualify an origin from another
        // immutable source. The actual original slot remains owned by caller.
        let foreign_source = crate::HtmlSource::utf8("other");
        let original = mem::replace(
            &mut incoming
                .as_mut()
                .unwrap()
                .attributes
                .get_mut(0, &mut c)
                .unwrap()
                .unwrap()
                .source,
            foreign_source.range(0, 5).unwrap(),
        );
        assert!(matches!(
            session.add_attributes_if_missing(&mut c, &root, &mut incoming),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignSource
            )))
        ));
        assert!(incoming.is_some());
        // Terminal disposal validates real ownership rather than source content.
        let _retained_original = original;
        let mut kind = incoming.take().map(TokenKind::Tag);
        release_token_kind(&mut kind, &mut c).unwrap();
        t.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn imported_attribute_backing_is_explicitly_unavailable_and_retained() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.allocate_element(&mut c, html("html")).unwrap();
        let mut incoming = Some(owned_tag(&mut c, "html", &[("x", &[121])]));
        assert_eq!(
            session.add_attributes_if_missing(&mut c, &root, &mut incoming),
            Err(SessionError::AttributeStorageUnavailable)
        );
        assert!(incoming.is_some());
        let mut kind = incoming.map(TokenKind::Tag);
        release_token_kind(&mut kind, &mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn staged_attribute_cap_failure_preserves_root_and_reclaims_exact_real_slots() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<html id=old><html a=one z=two>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .managed_attributes(2)
                .finish()
                .unwrap(),
            crate::HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(second);
        assert!(matches!(
            session.add_attributes_if_missing(&mut c, &root, &mut input),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedAttributes,
                ..
            }))
        ));
        assert!(input.is_none());
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[root.index].kind
        else {
            panic!("root")
        };
        assert_eq!(element.attributes().len(), 1);
        assert_eq!(
            element.attributes().as_slice()[0]
                .value()
                .to_utf8()
                .unwrap(),
            "old"
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            2
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            0
        );
        let stop = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        t.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .high_water,
            2
        );
    }

    #[test]
    fn merge_foreign_control_and_cancel_leave_the_real_lexical_owner_untaken() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        for cancel in [false, true] {
            let source = crate::HtmlSource::utf8("<html a=x>");
            let cancellation = crate::HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let root = session.allocate_implicit_html(&mut c).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut t, &mut c) else {
                panic!("tag")
            };
            let mut input = Some(tag);
            if cancel {
                cancellation.cancel();
                assert_eq!(
                    session.add_attributes_if_missing(&mut c, &root, &mut input),
                    Err(SessionError::Stop(ParseStop::Cancelled))
                );
            } else {
                let mut foreign = control();
                foreign.admit_source(&source).unwrap();
                assert!(matches!(
                    session.add_attributes_if_missing(&mut foreign, &root, &mut input),
                    Err(SessionError::Stop(ParseStop::InternalInvariant(
                        crate::parser::control::Invariant::ForeignControl
                    )))
                ));
            }
            assert!(input.is_some());
            let HtmlNodeKind::Element(element) =
                &session.active().unwrap().records[root.index].kind
            else {
                panic!("root")
            };
            assert!(element.attributes().is_empty());
            let mut kind = input.map(TokenKind::Tag);
            release_token_kind(&mut kind, &mut c).unwrap();
            t.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn merge_work_boundaries_retain_record_registry_owners_and_cleanup_all_staged_storage() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
        let run = |cap: Option<u64>| {
            let selected = match cap {
                Some(cap) => limits().into_builder().work_credits(cap).finish().unwrap(),
                None => limits(),
            };
            let mut c = ParseControl::new(selected, crate::HtmlParseCancellation::new());
            c.admit_source(&source).unwrap();
            let mut t = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
                panic!("tag")
            };
            let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
            let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
                panic!("tag")
            };
            let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let mut input = Some(second);
            let result = session.add_attributes_if_missing(&mut c, &root, &mut input);
            let completed = c.usage().resource(HtmlParseResource::WorkCredits).current;
            let HtmlNodeKind::Element(element) =
                &session.active().unwrap().records[root.index].kind
            else {
                panic!("root")
            };
            assert_eq!(
                element.attributes().as_slice()[0]
                    .value()
                    .to_utf8()
                    .unwrap(),
                "old"
            );
            assert!((1..=3).contains(&element.attributes().len()));
            assert_eq!(Arc::strong_count(element.attributes().backing()), 2);
            assert_eq!(Arc::weak_count(element.attributes().backing()), 1);
            if cap.is_some() {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
            } else {
                assert!(result.is_ok());
            }
            let stop = c.stop();
            if let Some(tag) = input {
                let mut kind = Some(TokenKind::Tag(tag));
                if release_token_kind(&mut kind, &mut c).is_err() {
                    release_token_kind(&mut kind, &mut c).unwrap();
                }
            }
            t.release(&mut c).unwrap();
            if session.release(&mut c).is_err() {
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            if stop.is_some() {
                assert_eq!(c.stop(), stop);
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current,
                    completed
                );
            }
            (setup, completed)
        };
        let (setup, complete) = run(None);
        // Independently selected stops span borrowed preflight, staged fields,
        // target reserve and final commit/late cleanup; no budget is replenished.
        for cap in [
            setup,
            setup + 1,
            setup + (complete - setup) / 3,
            setup + 2 * (complete - setup) / 3,
            complete - 1,
        ] {
            run(Some(cap));
        }
    }

    fn owned_text(c: &mut ParseControl, units: &[u16]) -> Text {
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(units.to_vec()));
        let mut value = Utf16Buffer::new(c).unwrap();
        value.extend_units(units, c).unwrap();
        let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        origins
            .try_push(
                c,
                TextOrigin {
                    start: 0,
                    end: units.len(),
                    source: source.range(0, units.len()).unwrap(),
                    kind: crate::parser::tokenizer::token::OriginKind::Literal,
                },
            )
            .unwrap();
        Text {
            value: value.freeze(c).unwrap(),
            origins,
        }
    }
    fn owned_tag(c: &mut ParseControl, name: &str, attrs: &[(&str, &[u16])]) -> Tag {
        let source = crate::HtmlSource::utf8("fixture");
        let mut attributes = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        for (name, value) in attrs {
            let name = owned_text(c, &name.encode_utf16().collect::<Vec<_>>());
            let value = owned_text(c, value);
            attributes
                .try_push(
                    c,
                    crate::parser::tokenizer::token::Attribute {
                        name,
                        value,
                        source: source.range(0, 7).unwrap(),
                    },
                )
                .unwrap();
        }
        Tag {
            kind: TagKind::Start,
            name: owned_text(c, &name.encode_utf16().collect::<Vec<_>>()),
            self_closing: false,
            attributes,
            had_duplicate_attributes: false,
        }
    }
    fn native_text(session: &ParserArenaSession, node: &ParserNode) -> Vec<u16> {
        let HtmlNodeKind::Text(value) = &session.active().unwrap().records[node.index].kind else {
            panic!("text node")
        };
        value.code_units().to_vec()
    }
    pub(super) fn storage_zero(c: &ParseControl) {
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::SourceMapEntries,
        ] {
            assert_eq!(c.usage().resource(resource).current, 0, "{resource:?}");
        }
    }
    fn lexical_kind(
        tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> TokenKind {
        loop {
            match tokenizer.next(c, false).unwrap().expect("fixture token") {
                crate::parser::tokenizer::token::Event::Token(event) => {
                    return event.into_token(c).unwrap().kind;
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    /// Actual tokenizer Text with multiple points and per-point origins. Data
    /// currently emits one point per token; an attribute uses the same concrete
    /// Text producer and gives the splitter a real mixed record, not a fake map.
    fn lexical_attribute_text(
        tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> (Text, Text, Text) {
        let TokenKind::Tag(Tag {
            name,
            mut attributes,
            ..
        }) = lexical_kind(tokenizer, c)
        else {
            panic!("tag");
        };
        let attribute = attributes.try_pop(c).unwrap().unwrap();
        attributes.release(c).unwrap();
        (attribute.value, name, attribute.name)
    }
    #[test]
    fn actual_comment_recovered_pi_and_doctype_keep_exact_managed_native_payloads() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<!--a"
                .encode_utf16()
                .chain([0xd800])
                .chain("--><?x a??><!DOCTYPE h PUBLIC 'p' 's'>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let TokenKind::Comment(text) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("comment");
        };
        let comment = session
            .allocate_comment(&mut c, &mut Some(text), None)
            .unwrap();
        session.insert(&mut c, &root, &comment, None, None).unwrap();
        assert!(session.document_accepts_doctype(&mut c).unwrap());
        let TokenKind::ProcessingInstruction { target, data } =
            lexical_kind(&mut tokenizer, &mut c)
        else {
            panic!("PI");
        };
        let pi = session
            .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
            .unwrap();
        session.insert(&mut c, &root, &pi, None, None).unwrap();
        assert!(session.document_accepts_doctype(&mut c).unwrap());
        let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("doctype");
        };
        let doctype = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
        session.insert(&mut c, &root, &doctype, None, None).unwrap();
        assert!(!session.document_accepts_doctype(&mut c).unwrap());
        session
            .set_document_mode(&mut c, HtmlDocumentMode::LimitedQuirks)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            8
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        let mut output = frozen(session, &mut c);
        let doc = output.document().unwrap();
        assert_eq!(doc.mode(), HtmlDocumentMode::LimitedQuirks);
        let children = doc.root().children().collect::<Vec<_>>();
        let HtmlNodeKind::Comment(value) = doc.node(&children[0]).unwrap().kind() else {
            panic!("comment");
        };
        assert_eq!(value.code_units(), [0x61, 0xd800]);
        let HtmlNodeKind::ProcessingInstruction(value) = doc.node(&children[1]).unwrap().kind()
        else {
            panic!("PI");
        };
        assert_eq!(value.target().code_units(), [0x78]);
        assert_eq!(value.data().code_units(), [0x61, 0x3f]);
        assert_eq!(
            HtmlAuthoredProcessingInstruction::try_from(value.clone()),
            Err(HtmlProcessingInstructionError::DataEndsWithQuestionMark)
        );
        let HtmlNodeKind::Doctype(value) = doc.node(&children[2]).unwrap().kind() else {
            panic!("doctype");
        };
        assert_eq!(value.name().code_units(), [0x68]);
        assert_eq!(value.public_id().unwrap().code_units(), [0x70]);
        assert_eq!(value.system_id().unwrap().code_units(), [0x73]);
        output.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn missing_doctype_fields_lower_to_managed_empty_strings_and_detached_nodes_survive_guard() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<!DOCTYPE><!DOCTYPE html>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("doctype");
        };
        assert!(
            value.name.is_none()
                && value.public_id.is_none()
                && value.system_id.is_none()
                && value.force_quirks
        );
        let first = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
        assert!(session.document_accepts_doctype(&mut c).unwrap());
        session.insert(&mut c, &root, &first, None, None).unwrap();
        let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("doctype");
        };
        let detached = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
        assert!(!session.document_accepts_doctype(&mut c).unwrap());
        assert!(session.parent(&mut c, &detached).unwrap().is_none());
        let HtmlNodeKind::Doctype(value) = &session.active().unwrap().records[first.index].kind
        else {
            panic!("doctype");
        };
        assert!(value.name().is_empty());
        assert!(value.public_id().unwrap().is_empty());
        assert!(value.system_id().unwrap().is_empty());
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            3
        );
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn source_implied_html_owns_literal_backing_and_real_empty_attributes_at_exact_unit_cap() {
        // html has four units; http://www.w3.org/1999/xhtml has twenty-eight.
        for cap in [31, 32] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .managed_utf16_units(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let result = session.allocate_implicit_html(&mut c);
            if cap == 31 {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedUtf16Units,
                        ..
                    }))
                ));
                assert_eq!(
                    c.usage().resource(HtmlParseResource::ManagedNodes).current,
                    1
                );
            } else {
                let node = result.unwrap();
                let HtmlNodeKind::Element(value) =
                    &session.active().unwrap().records[node.index].kind
                else {
                    panic!("element");
                };
                assert_eq!(
                    value.name().local_name().code_units(),
                    [0x68, 0x74, 0x6d, 0x6c]
                );
                assert_eq!(value.name().namespace(), &Namespace::html());
                assert!(value.attributes().is_empty());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    32
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current,
                    0
                );
            }
            if session.release(&mut c).is_err() {
                assert!(session.storage.is_some());
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
        }
    }
    #[test]
    fn doctype_partial_work_failures_keep_real_field_owners_reclaimable() {
        let make = |c: &mut ParseControl| {
            Some(Doctype {
                name: Some(owned_text(c, &[0x68])),
                public_id: Some(owned_text(c, &[0xd800])),
                system_id: Some(owned_text(c, &[0x73])),
                force_quirks: false,
            })
        };
        let mut probe = control();
        let mut probe_session = ParserArenaSession::new(&mut probe, RootKind::Fragment).unwrap();
        let mut input = make(&mut probe);
        let before = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        probe_session
            .allocate_doctype(&mut probe, &mut input)
            .unwrap();
        let required = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        probe_session.release(&mut probe).unwrap();
        storage_zero(&probe);
        for remaining in [0, 1, 40, 100, required - 1, required] {
            let mut c = control();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let mut input = make(&mut c);
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - remaining,
            )
            .unwrap();
            let result = session.allocate_doctype(&mut c, &mut input);
            assert_eq!(result.is_ok(), remaining == required);
            let work = c.usage().resource(HtmlParseResource::WorkCredits);
            let stop = c.stop();
            if let Some(value) = input.take() {
                release_token_kind(&mut Some(TokenKind::Doctype(value)), &mut c).unwrap();
            }
            if session.release(&mut c).is_err() {
                // Successful production may use the final available credit.
                // Active disposal then retains its owner for terminal retry.
                assert!(session.storage.is_some());
                session.release(&mut c).unwrap();
            }
            storage_zero(&c);
            if stop.is_some() {
                assert_eq!(c.stop(), stop);
                assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
            }
        }
    }
    #[test]
    fn mixed_whitespace_split_preserves_utf16_and_each_exact_origin_after_newline_preprocessing() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::OriginKind};
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<a v='"
                .encode_utf16()
                .chain([0x20, 9, 13, 10, 0xd83d, 0xde00, 0xd800, 0xfeff])
                .chain("'>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (value, tag_name, attribute_name) = lexical_attribute_text(&mut tokenizer, &mut c);
        tag_name.release(&mut c).unwrap();
        attribute_name.release(&mut c).unwrap();
        let mut input = Some(value);
        let prefix = split_leading_ascii_whitespace(&mut c, &mut input)
            .unwrap()
            .unwrap();
        assert_eq!(prefix.value.text().code_units(), [0x20, 9, 10]);
        assert_eq!(
            input.as_ref().unwrap().value.text().code_units(),
            [0xd83d, 0xde00, 0xd800, 0xfeff]
        );
        let origin = &prefix.origins.as_slice()[2];
        assert_eq!(
            (
                origin.start,
                origin.end,
                origin.source.start(),
                origin.source.end(),
                origin.kind
            ),
            (2, 3, 8, 10, OriginKind::Newline)
        );
        let ranges = input
            .as_ref()
            .unwrap()
            .origins
            .as_slice()
            .iter()
            .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
            .collect::<Vec<_>>();
        assert_eq!(ranges, [(0, 2, 10, 12), (2, 3, 12, 13), (3, 4, 13, 14)]);
        for origin in input.as_ref().unwrap().origins.as_slice() {
            assert_eq!(origin.source.source(), source.identity());
        }
        prefix.release(&mut c).unwrap();
        input.take().unwrap().release(&mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn failed_mixed_split_preserves_original_payload_capacity_and_source_origins() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<a v=' A'>");
        // Tag name + attribute name + two value units + tokenizer's real last
        // start-tag name occupy five units. Prefix fits at six; suffix does not.
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(6)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let (value, tag_name, attribute_name) = lexical_attribute_text(&mut tokenizer, &mut c);
        let mut input = Some(value);
        let capacity = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        assert!(matches!(
            split_leading_ascii_whitespace(&mut c, &mut input),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::BufferedUtf16Units,
                ..
            }))
        ));
        assert_eq!(
            input.as_ref().unwrap().value.text().code_units(),
            [0x20, 0x41]
        );
        assert_eq!(input.as_ref().unwrap().origins.len(), 2);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            capacity
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            5
        );
        let stop = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        release_token_kind(&mut input.map(TokenKind::Text), &mut c).unwrap();
        tag_name.release(&mut c).unwrap();
        attribute_name.release(&mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    }
    #[test]
    fn whitespace_identity_edges_and_unavailable_cut_preserve_the_original_owner() {
        let mut c = control();
        let mut input = Some(owned_text(&mut c, &[0x0b, 0x20, 0xfeff, 0]));
        assert!(
            split_leading_ascii_whitespace(&mut c, &mut input)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            input.as_ref().unwrap().value.text().code_units(),
            [0x0b, 0x20, 0xfeff, 0]
        );
        input.take().unwrap().release(&mut c).unwrap();
        let mut input = Some(owned_text(&mut c, &[9, 10, 12, 13, 32]));
        let prefix = split_leading_ascii_whitespace(&mut c, &mut input)
            .unwrap()
            .unwrap();
        assert!(input.is_none());
        prefix.release(&mut c).unwrap();
        // One aggregate origin contains both sides; no range is fabricated.
        let mut input = Some(owned_text(&mut c, &[32, 65]));
        assert!(matches!(
            split_leading_ascii_whitespace(&mut c, &mut input),
            Err(SessionError::OriginSplitUnavailable)
        ));
        assert_eq!(input.as_ref().unwrap().value.text().code_units(), [32, 65]);
        input.take().unwrap().release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn lexical_disposal_rejects_terminal_foreign_and_alias_owners_and_reclaims_once() {
        let mut c = control();
        let text = owned_text(&mut c, &[0xd800]);
        let alias = text.value.text().clone();
        let mut input = Some(TokenKind::Text(text));
        let mut foreign = control();
        let remaining = limits().work_credits()
            - foreign
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
        foreign.charge(WorkCategory::StateStep, remaining).unwrap();
        foreign.charge(WorkCategory::RecordVisit, 1).unwrap_err();
        assert!(release_token_kind(&mut input, &mut foreign).is_err());
        assert!(input.is_some());
        let remaining =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining).unwrap();
        c.charge(WorkCategory::RecordVisit, 1).unwrap_err();
        assert_eq!(
            release_token_kind(&mut input, &mut c),
            Err(SessionError::AliasedPayload)
        );
        assert!(input.is_some());
        let stop = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        let high_water = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        drop(alias);
        release_token_kind(&mut input, &mut c).unwrap();
        assert!(input.is_none());
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            high_water
        );
        assert_eq!(
            release_token_kind(&mut input, &mut c),
            Err(SessionError::Consumed)
        );
    }
    #[test]
    fn newly_latched_disposal_work_or_cancel_keeps_input_for_terminal_retry() {
        for cancel in [false, true] {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            let mut input = Some(TokenKind::Tag(owned_tag(&mut c, "a", &[("x", &[0xd800])])));
            if cancel {
                cancellation.cancel();
            } else {
                let remaining = limits().work_credits()
                    - c.usage().resource(HtmlParseResource::WorkCredits).current;
                c.charge(WorkCategory::StateStep, remaining).unwrap();
            }
            assert!(release_token_kind(&mut input, &mut c).is_err());
            assert!(input.is_some());
            let stop = c.stop();
            let work = c.usage().resource(HtmlParseResource::WorkCredits);
            release_token_kind(&mut input, &mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
            assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        }
    }
    #[test]
    fn late_lexical_attribute_disposal_stop_consumes_and_reclaims_the_original_payload() {
        let mut probe = control();
        let mut input = Some(TokenKind::Tag(owned_tag(
            &mut probe,
            "a",
            &[("x", &[0xd800])],
        )));
        release_token_kind(&mut input, &mut probe).unwrap();
        let total = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        storage_zero(&probe);
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .work_credits(total - 1)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let mut input = Some(TokenKind::Tag(owned_tag(&mut c, "a", &[("x", &[0xd800])])));
        assert!(matches!(
            release_token_kind(&mut input, &mut c),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 1,
                ..
            }))
        ));
        assert!(input.is_none());
        storage_zero(&c);
        assert!(c.stop().is_some());
    }
    #[test]
    fn active_lexical_validation_checks_actual_owners_and_cannot_resume_after_stop() {
        let mut c = control();
        let mut foreign = control();
        let mut input = Some(TokenKind::Comment(owned_text(&mut foreign, &[0xd800])));
        assert!(validate_token_kind(input.as_ref().unwrap(), &mut c).is_err());
        assert!(input.is_some());
        let stop = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        assert!(validate_token_kind(input.as_ref().unwrap(), &mut c).is_err());
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        release_token_kind(&mut input, &mut foreign).unwrap();
        storage_zero(&foreign);
        storage_zero(&c);
    }
    #[test]
    fn recovered_pi_preflights_both_field_owners_before_consuming_any_input() {
        let mut c = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut input = Some((
            owned_text(&mut c, &[0x78]),
            owned_text(&mut foreign, &[0xd800]),
        ));
        assert!(
            session
                .allocate_processing_instruction(&mut c, &mut input, None)
                .is_err()
        );
        let (target, data) = input.take().expect("both original fields retained");
        assert_eq!(target.value.text().code_units(), [0x78]);
        assert_eq!(data.value.text().code_units(), [0xd800]);
        assert_eq!(session.active().unwrap().records.len(), 1);
        target.release(&mut c).unwrap();
        data.release(&mut foreign).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        storage_zero(&foreign);
    }
    #[test]
    fn document_mode_update_rejects_fragment_and_work_stop_before_the_real_mode_write() {
        let mut c = control();
        let mut fragment = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        assert_eq!(
            fragment.document_accepts_doctype(&mut c),
            Err(SessionError::NotDocument)
        );
        assert_eq!(
            fragment.set_document_mode(&mut c, HtmlDocumentMode::Quirks),
            Err(SessionError::NotDocument)
        );
        assert!(c.stop().is_none());
        fragment.release(&mut c).unwrap();
        let mut document =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, limits().work_credits() - used - 1)
            .unwrap();
        assert!(matches!(
            document.set_document_mode(&mut c, HtmlDocumentMode::Quirks),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        assert!(matches!(
            document.active().unwrap().records[0].kind,
            HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
        ));
        document.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn element_guard_reads_document_children_without_restricting_generic_moves() {
        let mut c = control();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let first = session.allocate_element(&mut c, html("html")).unwrap();
        let second = session.allocate_element(&mut c, html("body")).unwrap();
        let visits = c.usage().work(WorkCategory::RecordVisit);
        assert_eq!(
            session.element_insertion_ignored(&mut c, &first, &root),
            Ok(None)
        );
        // Twelve readiness visits + two checked handles, element kind,
        // ancestry root and target kind.
        assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 17);
        session.insert(&mut c, &root, &first, None, None).unwrap();
        let visits = c.usage().work(WorkCategory::RecordVisit);
        assert_eq!(
            session.element_insertion_ignored(&mut c, &second, &root),
            Ok(Some(ElementInsertionIgnored::DocumentAlreadyHasElement))
        );
        // The additional real child-index and referenced-kind reads cost two.
        assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 19);
        assert_eq!(session.active().unwrap().records[0].children, [first.index]);
        assert!(
            session.active().unwrap().records[second.index]
                .parent
                .is_none()
        );
        // The guard is HTML placement policy; generic native moves are unchanged.
        session.insert(&mut c, &root, &second, None, None).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [first.index, second.index]
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn element_guard_uses_actual_template_and_ordinary_ancestry_and_includes_self() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let template = session.allocate_element(&mut c, html("template")).unwrap();
        let contents = session
            .template_contents(&mut c, &template)
            .unwrap()
            .unwrap();
        let child = session.allocate_element(&mut c, html("div")).unwrap();
        session
            .insert(&mut c, &contents, &child, None, None)
            .unwrap();
        assert!(
            session.active().unwrap().records[contents.index]
                .parent
                .is_none()
        );
        let visits = c.usage().work(WorkCategory::RecordVisit);
        let providers = c.usage().work(WorkCategory::ProviderCall);
        assert_eq!(
            session.element_insertion_ignored(&mut c, &template, &child),
            Ok(Some(
                ElementInsertionIgnored::HostIncludingInclusiveAncestor
            ))
        );
        // Twelve readiness visits + two handles + element kind +
        // child/contents/template ancestors.
        assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 18);
        assert_eq!(c.usage().work(WorkCategory::ProviderCall) - providers, 1);
        assert_eq!(
            session.element_insertion_ignored(&mut c, &template, &template),
            Ok(Some(
                ElementInsertionIgnored::HostIncludingInclusiveAncestor
            ))
        );
        assert_eq!(
            session.element_insertion_ignored(&mut c, &child, &root),
            Ok(Some(ElementInsertionIgnored::ParentAlreadyPresent))
        );
        assert!(
            session.active().unwrap().records[template.index]
                .parent
                .is_none()
        );
        assert_eq!(
            session.active().unwrap().records[contents.index].children,
            [child.index]
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn element_guard_work_stop_precedes_document_child_reads_and_preserves_topology() {
        let mut c = control();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let first = session.allocate_element(&mut c, html("html")).unwrap();
        let second = session.allocate_element(&mut c, html("div")).unwrap();
        session.insert(&mut c, &root, &first, None, None).unwrap();
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes);
        let edges = c.usage().resource(HtmlParseResource::ManagedEdges);
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, limits().work_credits() - used - 18)
            .unwrap();
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        assert!(matches!(
            session.element_insertion_ignored(&mut c, &second, &root),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 2,
                ..
            }))
        ));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            used + 17
        );
        assert_eq!(c.usage().resource(HtmlParseResource::ManagedNodes), nodes);
        assert_eq!(c.usage().resource(HtmlParseResource::ManagedEdges), edges);
        assert_eq!(session.active().unwrap().records[0].children, [first.index]);
        assert!(
            session.active().unwrap().records[second.index]
                .parent
                .is_none()
        );
        let stop = c.stop();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }

    #[test]
    fn element_guard_rejects_foreign_handles_controls_and_cancellation_before_mutation() {
        for cancel in [false, true] {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let root = session.root(&mut c).unwrap();
            let node = session.allocate_element(&mut c, html("div")).unwrap();
            if cancel {
                cancellation.cancel();
                assert_eq!(
                    session.element_insertion_ignored(&mut c, &node, &root),
                    Err(SessionError::Stop(ParseStop::Cancelled))
                );
            } else {
                let mut other = control();
                let mut foreign = ParserArenaSession::new(&mut other, RootKind::Fragment).unwrap();
                let foreign_node = foreign.allocate_element(&mut other, html("div")).unwrap();
                assert_eq!(
                    session.element_insertion_ignored(&mut c, &node, &foreign_node),
                    Err(SessionError::ForeignNode)
                );
                assert!(c.stop().is_none());
                assert!(matches!(
                    session.element_insertion_ignored(&mut other, &node, &root),
                    Err(SessionError::Stop(ParseStop::InternalInvariant(_)))
                ));
                foreign.release(&mut other).unwrap();
                storage_zero(&other);
            }
            assert!(session.active().unwrap().records[0].children.is_empty());
            assert!(
                session.active().unwrap().records[node.index]
                    .parent
                    .is_none()
            );
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn newly_latched_work_or_cancellation_disposes_all_retained_attribute_fields() {
        for cancelled in [false, true] {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            let Tag {
                name, attributes, ..
            } = owned_tag(&mut c, "div", &[("x", &[0xd800])]);
            name.release(&mut c).unwrap();
            let before = c.usage().resource(HtmlParseResource::ManagedCapacityBytes);
            if cancelled {
                cancellation.cancel();
                assert!(c.stop().is_none());
            } else {
                let remaining = limits().work_credits()
                    - c.usage().resource(HtmlParseResource::WorkCredits).current;
                c.charge(WorkCategory::StateStep, remaining).unwrap();
            }
            let work = c.usage().resource(HtmlParseResource::WorkCredits);
            let error = release_token_attributes(attributes, &mut c).unwrap_err();
            if cancelled {
                assert_eq!(error, SessionError::Stop(ParseStop::Cancelled));
            } else {
                assert!(matches!(
                    error,
                    SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
            }
            storage_zero(&c);
            assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .high_water,
                before.high_water
            );
        }
    }

    #[test]
    fn final_empty_attribute_cleanup_work_stop_releases_the_actual_lexical_capacity_ticket() {
        // Select the exact final empty-pop boundary from the successful real
        // producer, then remove its single remaining RecordVisit credit.
        let mut probe = control();
        let mut session = ParserArenaSession::new(&mut probe, RootKind::Fragment).unwrap();
        let mut input = Some(owned_tag(&mut probe, "div", &[("x", &[0x61])]));
        let lexical_bytes = input.as_ref().unwrap().attributes.capacity()
            * mem::size_of::<crate::parser::tokenizer::token::Attribute>();
        session.allocate_html_tag(&mut probe, &mut input).unwrap();
        let completed_work = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        session.release(&mut probe).unwrap();
        storage_zero(&probe);

        let caps = limits()
            .into_builder()
            .work_credits(completed_work - 1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut input = Some(owned_tag(&mut c, "div", &[("x", &[0x61])]));
        assert!(matches!(
            session.allocate_html_tag(&mut c, &mut input),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 1,
                ..
            }))
        ));
        assert!(input.is_none());
        assert_eq!(session.active().unwrap().records.len(), 2);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            2
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            0
        );
        let stop = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes);
        let capacity_high_water = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        assert_eq!(c.usage().resource(HtmlParseResource::ManagedNodes), nodes);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            capacity_high_water
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0,
            "all actual backing must be reclaimed; the lexical attribute Vec alone occupied {lexical_bytes} bytes"
        );
        storage_zero(&c);
    }

    #[test]
    fn active_cleanup_work_stop_retains_native_storage_for_terminal_disposal() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut input = Some(owned_text(&mut c, &[0x61]));
        let inserted = session
            .insert_text(&mut c, &root, None, &mut input, None)
            .unwrap()
            .unwrap();
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, limits().work_credits() - used - 2)
            .unwrap();
        // One registry visit + one actual native slot visit fit. The two Arc
        // metadata provider inspections must be admitted before either executes.
        assert!(matches!(
            session.release(&mut c),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta: 2,
                ..
            }))
        ));
        assert_eq!(native_text(&session, &inserted.node), [0x61]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            1
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        let stop = c.stop();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    }

    #[test]
    fn actual_payload_registry_capacity_and_uninitialized_slot_failure_are_reclaimable() {
        // Independently sum root Record+child-ticket vector, one exact frozen
        // unit+Vec wrapper, and one source-origin record. None includes an Arc
        // header or preexisting source backing.
        let baseline = mem::size_of::<Record>()
            + mem::size_of::<VectorCapacity<usize>>()
            + mem::size_of::<u16>()
            + mem::size_of::<Vec<u16>>()
            + mem::size_of::<TextOrigin>();
        let cap = baseline + mem::size_of::<RetainedPayload>() - 1;
        let caps = limits()
            .into_builder()
            .managed_capacity_bytes(cap as u64)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut input = Some(owned_text(&mut c, &[0x61]));
        assert_eq!(c.usage().actual_capacity_bytes, baseline as u64);
        assert!(matches!(
            session.insert_text(&mut c, &root, None, &mut input, None),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }))
        ));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);

        let caps = limits()
            .into_builder()
            .auxiliary_entries(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut input = Some(owned_text(&mut c, &[0x61]));
        assert!(matches!(
            session.insert_text(&mut c, &root, None, &mut input, None),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                limit: 1,
                current: 1,
                requested_delta: 1
            }))
        ));
        assert!(session.active().unwrap().payloads.values.is_empty());
        assert!(session.active().unwrap().payloads.values.capacity() > 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            1
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn isolated_token_name_units_are_stored_without_utf8_conversion_or_name_restriction() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let name = owned_text(&mut c, &[0x61, 0xd800, 0x3a]);
        let attributes = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
        let mut tag = Some(Tag {
            kind: TagKind::Start,
            name,
            attributes,
            self_closing: false,
            had_duplicate_attributes: false,
        });
        let node = session.allocate_html_tag(&mut c, &mut tag).unwrap();
        let name = session.element_name(&mut c, &node).unwrap().unwrap();
        assert_eq!(name.local_name().code_units(), &[0x61, 0xd800, 0x3a]);
        assert_eq!(name.prefix(), None);
        assert!(name.local_name().to_utf8().is_err());
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn real_tokenizer_case_duplicate_recovery_and_reference_data_reach_managed_native_element() {
        use crate::parser::tokenizer::{
            states::State,
            tests::LexerFixture,
            token::{Event, TokenKind},
        };
        let source = crate::HtmlSource::utf8("<DiV Z=x z=y A:B='&#x1F600;'>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut inserted = None;
        while let Some(event) = tokenizer.next(&mut c, false).unwrap() {
            match event {
                Event::Token(event) => {
                    let token = event.into_token(&mut c).unwrap();
                    match token.kind {
                        TokenKind::Tag(tag) => {
                            assert!(tag.had_duplicate_attributes);
                            let mut input = Some(tag);
                            let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
                            session.insert(&mut c, &root, &node, None, None).unwrap();
                            inserted = Some(node);
                        }
                        TokenKind::Eof => {}
                        _ => panic!("fixture emits only one start tag and EOF"),
                    }
                }
                event => event.release(&mut c).unwrap(),
            }
        }
        tokenizer.release(&mut c).unwrap();
        let node = inserted.unwrap();
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[node.index].kind
        else {
            panic!("element")
        };
        assert_eq!(
            element.name().local_name().code_units(),
            &[0x64, 0x69, 0x76]
        );
        assert_eq!(element.attributes().len(), 2);
        assert_eq!(
            element.attributes().as_slice()[0]
                .name()
                .local_name()
                .code_units(),
            &[0x7a]
        );
        assert_eq!(
            element.attributes().as_slice()[0].value().code_units(),
            &[0x78]
        );
        assert_eq!(
            element.attributes().as_slice()[1]
                .name()
                .local_name()
                .code_units(),
            &[0x61, 0x3a, 0x62]
        );
        assert_eq!(
            element.attributes().as_slice()[1].value().code_units(),
            &[0xd83d, 0xde00]
        );
        let mut frozen = session.freeze(&mut c).unwrap();
        assert_eq!(frozen.fragment().unwrap().root().children().count(), 1);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn managed_attribute_slot_cap_and_container_alias_preserve_actual_cleanup_owner() {
        let caps = limits()
            .into_builder()
            .managed_attributes(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut input = Some(owned_tag(&mut c, "div", &[("a", &[0x61]), ("b", &[0x62])]));
        assert!(matches!(
            session.allocate_html_tag(&mut c, &mut input),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedAttributes,
                limit: 1,
                current: 1,
                requested_delta: 1,
            }))
        ));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            1
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);

        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut input = Some(owned_tag(&mut c, "div", &[("a", &[0x61])]));
        let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
        let HtmlNodeKind::Element(element) = &session.active().unwrap().records[node.index].kind
        else {
            panic!("element")
        };
        let alias = element.attributes().clone();
        assert!(matches!(
            session.release(&mut c),
            Err(SessionError::AliasedPayload)
        ));
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            1
        );
        drop(alias);
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn merged_copy_capacity_failure_retains_old_native_text_and_sticky_cleanup() {
        // Exact managed cap4 admits old1 and incoming1, but the merged2 would
        // raise retained backing to4. One-short3 rejects that final transfer.
        let caps = limits()
            .into_builder()
            .managed_utf16_units(3)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut a = Some(owned_text(&mut c, &[0x61]));
        let a = session
            .insert_text(&mut c, &root, None, &mut a, None)
            .unwrap()
            .unwrap();
        let mut b = Some(owned_text(&mut c, &[0x62]));
        assert!(matches!(
            session.insert_text(&mut c, &root, None, &mut b, None),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                limit: 3,
                current: 2,
                requested_delta: 2,
            }))
        ));
        assert_eq!(native_text(&session, &a.node), [0x61]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            2
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            2
        );
        let stop = c.stop();
        let high = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
            high
        );
    }

    #[test]
    fn topology_hooks_borrow_exact_names_and_distinguish_real_template_creation() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        assert!(session.element_name(&mut c, &root).unwrap().is_none());
        assert!(session.fragment_root(&mut c).unwrap().is_some());
        session.validate_node(&mut c, &root).unwrap();
        let old = session.allocate_element(&mut c, html("template")).unwrap();
        assert!(matches!(
            session.unpatched_template_contents(&mut c, &old),
            Err(SessionError::TemplateInsertionStateUnavailable)
        ));
        let mut tag = Some(owned_tag(&mut c, "template", &[]));
        let new = session.allocate_html_tag(&mut c, &mut tag).unwrap();
        assert!(
            session
                .unpatched_template_contents(&mut c, &new)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            session
                .element_name(&mut c, &new)
                .unwrap()
                .unwrap()
                .local_name()
                .code_units(),
            &[0x74, 0x65, 0x6d, 0x70, 0x6c, 0x61, 0x74, 0x65]
        );
        assert!(!session.same_node(&mut c, &old, &new).unwrap());
        assert!(session.parent(&mut c, &new).unwrap().is_none());
        session.insert(&mut c, &root, &new, None, None).unwrap();
        let parent = session.parent(&mut c, &new).unwrap().unwrap();
        assert!(session.same_node(&mut c, &parent, &root).unwrap());
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn token_backing_transfers_exact_names_order_attributes_and_template_pair_to_native_view() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut tag = Some(owned_tag(
            &mut c,
            "template",
            &[("z", &[0xd800, 0, 0x20]), ("a:b", &[0xd83d, 0xde00])],
        ));
        let original = tag
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let element = session.allocate_html_tag(&mut c, &mut tag).unwrap();
        assert!(tag.is_none());
        session.insert(&mut c, &root, &element, None, None).unwrap();
        let contents = session
            .template_contents(&mut c, &element)
            .unwrap()
            .unwrap();
        assert_ne!(contents.index, element.index);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            2
        );
        let mut frozen = session.freeze(&mut c).unwrap();
        let fragment = frozen.fragment().unwrap();
        let id = fragment.root().children().next().unwrap();
        let HtmlNodeKind::Element(value) = fragment.node(&id).unwrap().kind() else {
            panic!("element")
        };
        assert_eq!(value.name().local_name().code_units().as_ptr(), original);
        assert_eq!(value.name().namespace(), &Namespace::html());
        let attrs = value.attributes().as_slice();
        assert_eq!(attrs[0].name().local_name().code_units(), &[0x7a]);
        assert_eq!(attrs[0].value().code_units(), &[0xd800, 0, 0x20]);
        assert_eq!(
            attrs[1].name().local_name().code_units(),
            &[0x61, 0x3a, 0x62]
        );
        assert_eq!(attrs[1].name().namespace(), &Namespace::none());
        assert_eq!(attrs[1].name().prefix(), None);
        assert_eq!(attrs[1].value().code_units(), &[0xd83d, 0xde00]);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            3
        );
    }
    #[test]
    fn character_insertion_merges_preceding_text_keeps_identity_and_retained_units() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut first = Some(owned_text(&mut c, &[0xd800, 0x20]));
        let first = session
            .insert_text(&mut c, &root, None, &mut first, None)
            .unwrap()
            .unwrap();
        assert_eq!(first.offset, 0);
        let mut next = Some(owned_text(&mut c, &[0xd83d, 0xde00, 0]));
        let next = session
            .insert_text(&mut c, &root, None, &mut next, None)
            .unwrap()
            .unwrap();
        assert_eq!(next.node.index, first.node.index);
        assert_eq!(next.offset, 2);
        assert_eq!(
            native_text(&session, &first.node),
            [0xd800, 0x20, 0xd83d, 0xde00, 0]
        );
        assert_eq!(session.active().unwrap().usage.utf16_units, 5);
        // Actual retained strings: old2 + incoming3 + merged5. Four origin
        // records are the two unique leaf atoms; three groups, two field causes,
        // merge and retirement give nine actual source-map entries.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            10
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            9
        );
        let mut frozen = session.freeze(&mut c).unwrap();
        assert_eq!(frozen.fragment().unwrap().usage().utf16_units(), 5);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn shared_history_reconstructs_every_literal_prefix_without_retired_prefix_backings() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("abcdefgh");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let mut histories = Vec::new();
        let mut node = None;
        for length in 1..=8 {
            let mut input = Some(match lexical_kind(&mut lexer, &mut c) {
                TokenKind::Text(text) => text,
                _ => panic!("literal text"),
            });
            let inserted = session
                .insert_text(&mut c, &root, None, &mut input, None)
                .unwrap()
                .unwrap();
            assert_eq!(inserted.offset, length - 1);
            if let Some(old) = &node {
                assert!(session.same_node(&mut c, old, &inserted.node).unwrap());
            }
            node = Some(inserted.node);
            let storage = session.active().unwrap();
            let slot = storage
                .field_slot(&mut c, StringLocation::Text(node.as_ref().unwrap().index))
                .unwrap();
            let RetainedPayload::String {
                origins: Some(group),
                ..
            } = &storage.payloads.values[slot]
            else {
                panic!("current actual group");
            };
            histories.push(
                storage
                    .evidence
                    .as_ref()
                    .unwrap()
                    .copy_group_reference(&mut c, group)
                    .unwrap(),
            );
        }
        // Exactly eight unique input units + eight current native units. Six
        // retired aggregate slots carry fixed history descriptors, not Vecs.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            16
        );
        assert_eq!(
            session
                .active()
                .unwrap()
                .payloads
                .values
                .iter()
                .filter(|p| matches!(p, RetainedPayload::HistoricalString { .. }))
                .count(),
            6
        );
        let before = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        for (index, group) in histories.iter().enumerate() {
            let storage = session.active().unwrap();
            let evidence = storage.evidence.as_ref().unwrap();
            let reconstructed = evidence
                .materialize_group(&mut c, &storage.payloads, group)
                .unwrap();
            assert_eq!(
                reconstructed.text().code_units(),
                &[97, 98, 99, 100, 101, 102, 103, 104][..index + 1]
            );
            ParseEvidence::release_materialized_group(&mut c, &mut Some(reconstructed)).unwrap();
            let maps = evidence.observed_origins(group, &mut c).unwrap();
            assert_eq!(
                maps.iter()
                    .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                    .collect::<Vec<_>>(),
                (0..index + 1)
                    .map(|i| (i, i + 1, i, i + 1))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current,
                before
            );
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn aggregate_alias_is_refused_before_next_token_transfer_and_reclaims_after_drop() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        for unit in [65, 66] {
            let mut input = Some(owned_text(&mut c, &[unit]));
            session
                .insert_text(&mut c, &root, None, &mut input, None)
                .unwrap();
        }
        let index = session.active().unwrap().records[0].children[0];
        let HtmlNodeKind::Text(value) = &session.active().unwrap().records[index].kind else {
            panic!("text");
        };
        let alias = value.clone();
        let units = c
            .usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current;
        let mut input = Some(owned_text(&mut c, &[67]));
        assert_eq!(
            session
                .insert_text(&mut c, &root, None, &mut input, None)
                .err(),
            Some(SessionError::AliasedPayload)
        );
        assert!(input.is_some());
        assert!(c.stop().is_none());
        assert_eq!(alias.code_units(), [65, 66]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            units
        );
        drop(alias);
        session
            .insert_text(&mut c, &root, None, &mut input, None)
            .unwrap();
        assert!(input.is_none());
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    fn two_leaf_history(
        limits: crate::HtmlParseLimits,
        cancel: HtmlParseCancellation,
    ) -> (
        ParseControl,
        ParserArenaSession,
        OriginGroupRef,
        crate::HtmlSource,
    ) {
        let source = crate::HtmlSource::utf8("AB");
        let mut c = ParseControl::new(limits, cancel);
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = crate::parser::tokenizer::tests::LexerFixture::new(
            &source,
            crate::parser::tokenizer::states::State::Data,
            None,
            &mut c,
        )
        .unwrap();
        let mut node = None;
        for _ in 0..2 {
            let mut text = Some(match lexical_kind(&mut lexer, &mut c) {
                TokenKind::Text(text) => text,
                _ => panic!("literal"),
            });
            node = Some(
                session
                    .insert_text(&mut c, &root, None, &mut text, None)
                    .unwrap()
                    .unwrap()
                    .node,
            );
        }
        lexer.release(&mut c).unwrap();
        let storage = session.active().unwrap();
        let slot = storage
            .field_slot(&mut c, StringLocation::Text(node.unwrap().index))
            .unwrap();
        let RetainedPayload::String {
            origins: Some(group),
            ..
        } = &storage.payloads.values[slot]
        else {
            panic!("group");
        };
        let group = storage
            .evidence
            .as_ref()
            .unwrap()
            .copy_group_reference(&mut c, group)
            .unwrap();
        (c, session, group, source)
    }
    #[test]
    fn history_cursor_cancellation_keeps_canonical_snapshot_and_reclaims_real_frames() {
        let cancel = HtmlParseCancellation::new();
        let (mut c, mut session, group, _) = two_leaf_history(limits(), cancel.clone());
        let evidence = session.active().unwrap().evidence.as_ref().unwrap();
        let before = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        let mut cursor = provenance::GroupOriginCursor::new(evidence, &group, &mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            before + 1
        );
        cancel.cancel();
        assert_eq!(
            cursor.next(evidence, &mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        let snapshot = c.first_stop_usage().unwrap().clone();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        cursor.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            before
        );
        assert_eq!(c.first_stop_usage(), Some(&snapshot));
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        assert!(matches!(
            evidence.materialize_group(&mut c, &session.active().unwrap().payloads, &group),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.first_stop_usage(), Some(&snapshot));
    }
    #[test]
    fn history_cursor_foreign_control_or_evidence_cannot_advance_the_owning_cursor() {
        let (mut c, mut session, group, _) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        let (mut other, mut other_session, _, _) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        let evidence = session.active().unwrap().evidence.as_ref().unwrap();
        let mut cursor = provenance::GroupOriginCursor::new(evidence, &group, &mut c).unwrap();
        assert!(matches!(
            cursor.next(evidence, &mut other),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert!(c.stop().is_none());
        assert!(cursor.release(&mut other).is_err());
        let first = cursor.next(evidence, &mut c).unwrap().unwrap();
        assert_eq!((first.start, first.end), (0, 1));
        cursor.release(&mut c).unwrap();
        other_session.release(&mut other).unwrap();
        storage_zero(&other);
        let mut cursor = provenance::GroupOriginCursor::new(evidence, &group, &mut c).unwrap();
        let foreign = ParseEvidence::prepare(&mut c).unwrap();
        assert!(matches!(
            cursor.next(&foreign, &mut c),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignEvidence
            )))
        ));
        cursor.release(&mut c).unwrap();
        foreign.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn history_cursor_lender_disposal_is_safe_for_owning_control_and_cancellation() {
        for cancelled in [false, true] {
            let cancel = HtmlParseCancellation::new();
            let (mut c, mut session, _, source) = two_leaf_history(limits(), cancel.clone());
            session
                .prepare_native_associations(&mut c, &source)
                .unwrap();
            let baseline_capacity = c.usage().actual_capacity_bytes;
            {
                let mut rows = session.native_associations(&mut c).unwrap();
                let mut cursor = rows.group_origin_cursor(2).unwrap();
                if cancelled {
                    cancel.cancel();
                    assert!(matches!(
                        rows.next_group_origin(&mut cursor),
                        Err(SessionError::Stop(ParseStop::Cancelled))
                    ));
                } else {
                    let first = rows.next_group_origin(&mut cursor).unwrap().unwrap();
                    let second = rows.next_group_origin(&mut cursor).unwrap().unwrap();
                    assert_eq!((first.start, first.end), (0, 1));
                    assert_eq!((second.start, second.end), (1, 2));
                    assert_eq!(first.source, source.range(0, 1).unwrap());
                    assert_eq!(second.source, source.range(1, 2).unwrap());
                    assert!(rows.next_group_origin(&mut cursor).unwrap().is_none());
                }
                let mut cursor = Some(cursor);
                rows.release_group_origin_cursor(&mut cursor).unwrap();
                assert!(cursor.is_none());
                assert_eq!(
                    rows.release_group_origin_cursor(&mut cursor),
                    Err(SessionError::Consumed)
                );
            }
            assert_eq!(c.usage().actual_capacity_bytes, baseline_capacity);
            let first = c.first_stop_usage().cloned();
            if cancelled {
                assert_eq!(c.stop(), Some(ParseStop::Cancelled));
                for category in [
                    WorkCategory::StateStep,
                    WorkCategory::InputUnitRead,
                    WorkCategory::InspectByte,
                    WorkCategory::CopyByte,
                    WorkCategory::RecordVisit,
                    WorkCategory::RecordWrite,
                    WorkCategory::ReserveByte,
                    WorkCategory::ProviderCall,
                ] {
                    assert_eq!(
                        c.usage().work(category),
                        first.as_ref().unwrap().work(category)
                    );
                }
            } else {
                assert!(c.stop().is_none());
                assert!(first.is_none());
            }
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.first_stop_usage(), first.as_ref());
        }
    }

    #[test]
    fn foreign_history_cursor_lender_disposal_keeps_cleanup_owned() {
        let (mut c, mut session, group, source) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        let (mut foreign, mut foreign_session, _, foreign_source) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        foreign_session
            .prepare_native_associations(&mut foreign, &foreign_source)
            .unwrap();
        // Actual A/B ingress creates Leaf0, Leaf1 and their Concat2. Obtain
        // the owned scratch through the real completed lender, not a forged
        // cursor or a private field/ticket mutation.
        let baseline = c.usage().clone();
        let cursor = {
            let mut rows = session.native_associations(&mut c).unwrap();
            let mut cursor = rows.group_origin_cursor(2).unwrap();
            let first = rows.next_group_origin(&mut cursor).unwrap().unwrap();
            assert_eq!((first.start, first.end), (0, 1));
            assert_eq!(first.source, source.range(0, 1).unwrap());
            cursor
        };
        let mut cursor = Some(cursor);
        let owning_usage = c.usage().clone();
        assert!(
            owning_usage
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current
                > baseline
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current
        );
        {
            let mut rows = foreign_session.native_associations(&mut foreign).unwrap();
            assert_eq!(
                rows.release_group_origin_cursor(&mut cursor),
                Err(SessionError::Stop(ParseStop::InternalInvariant(
                    crate::parser::control::Invariant::ForeignControl
                )))
            );
        }
        assert_eq!(c.usage(), &owning_usage);
        assert!(c.stop().is_none());
        assert!(cursor.is_some());
        {
            let mut rows = session.native_associations(&mut c).unwrap();
            // Retry the exact refused cursor through its genuine owning lender.
            rows.release_group_origin_cursor(&mut cursor).unwrap();
            assert!(cursor.is_none());
            assert_eq!(
                rows.release_group_origin_cursor(&mut cursor),
                Err(SessionError::Consumed)
            );
        }
        assert_eq!(
            c.usage().actual_capacity_bytes,
            baseline.actual_capacity_bytes
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            baseline
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current
        );
        let root = session.root(&mut c).unwrap();
        let storage = session.active().unwrap();
        let text = storage
            .node(&mut c, storage.records[root.index].children[0])
            .unwrap();
        assert_eq!(native_text(&session, &text), [65, 66]);
        // The original native Text and historical value remain source-qualified
        // and usable; only the foreign disposal must be refused.
        let storage = session.active().unwrap();
        let output = storage
            .evidence
            .as_ref()
            .unwrap()
            .materialize_group(&mut c, &storage.payloads, &group)
            .unwrap();
        assert_eq!(output.text().code_units(), [65, 66]);
        ParseEvidence::release_materialized_group(&mut c, &mut Some(output)).unwrap();
        let foreign_stop = foreign.first_stop_usage().unwrap().clone();
        foreign_session.release(&mut foreign).unwrap();
        storage_zero(&foreign);
        assert_eq!(foreign.first_stop_usage(), Some(&foreign_stop));
        session.release(&mut c).unwrap();
        assert!(c.stop().is_none());
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            0
        );
        // Foreign refusal and the actual owning retry must not abandon the
        // real traversal allocation or leave logical scratch charged after
        // disposal of the original session.
        storage_zero(&c);
    }

    #[test]
    fn historical_materialization_exact_one_short_releases_output_and_traversal_on_first_stop() {
        let (mut measured, mut session, group, _) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        let base = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let storage = session.active().unwrap();
        let output = storage
            .evidence
            .as_ref()
            .unwrap()
            .materialize_group(&mut measured, &storage.payloads, &group)
            .unwrap();
        assert_eq!(output.text().code_units(), [65, 66]);
        let complete = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        ParseEvidence::release_materialized_group(&mut measured, &mut Some(output)).unwrap();
        session.release(&mut measured).unwrap();
        storage_zero(&measured);
        for short in [false, true] {
            let selected = limits()
                .into_builder()
                .work_credits(complete - u64::from(short))
                .finish()
                .unwrap();
            let (mut c, mut session, group, _) =
                two_leaf_history(selected, HtmlParseCancellation::new());
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                base
            );
            let before = c
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current;
            let storage = session.active().unwrap();
            let result = storage.evidence.as_ref().unwrap().materialize_group(
                &mut c,
                &storage.payloads,
                &group,
            );
            if short {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert!(c.stop().is_some());
            } else {
                let output = result.unwrap();
                assert_eq!(output.text().code_units(), [65, 66]);
                ParseEvidence::release_materialized_group(&mut c, &mut Some(output)).unwrap();
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
            }
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current,
                before
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                0
            );
            let snapshot = c.first_stop_usage().unwrap().clone();
            let work = c.usage().resource(HtmlParseResource::WorkCredits);
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
            assert_eq!(c.first_stop_usage(), Some(&snapshot));
        }
    }
    #[test]
    fn invalidated_pi_map_keeps_exact_retired_decoded_values_for_completion() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let source = crate::HtmlSource::utf8("<?marker a='A' a='B'?>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c)
        else {
            panic!("literal PI");
        };
        let node = session
            .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
            .unwrap();
        let HtmlNodeKind::ProcessingInstruction(pi) =
            &session.active().unwrap().records[node.index].kind
        else {
            panic!("installed PI");
        };
        assert_eq!(
            pi.data().code_units(),
            DocumentString::from("a='A' a='B'").code_units()
        );
        let storage = session.active().unwrap();
        assert!(storage.payloads.values.iter().any(|p| matches!(p, RetainedPayload::PiAttributes { node:Some(owner),values,ready:true } if *owner==node.index && values.len()==0)));
        let evidence = storage.evidence.as_ref().unwrap();
        let mut retired = Vec::new();
        for payload in &storage.payloads.values {
            if let RetainedPayload::String {
                origins: Some(group),
                ..
            } = payload
                && evidence.require_group(group).unwrap().retirement.is_some()
            {
                let reconstructed = evidence
                    .materialize_group(&mut c, &storage.payloads, group)
                    .unwrap();
                let rows = evidence.observed_origins(group, &mut c).unwrap();
                retired.push((
                    reconstructed.text().code_units().to_vec(),
                    rows.iter()
                        .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                        .collect::<Vec<_>>(),
                ));
                ParseEvidence::release_materialized_group(&mut c, &mut Some(reconstructed))
                    .unwrap();
            }
        }
        assert_eq!(
            retired,
            vec![
                (vec![97], vec![(0, 1, 9, 10)]),
                (vec![65], vec![(0, 1, 12, 13)])
            ]
        );
        lexer.release(&mut c).unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn raw_retirement_keeps_transformed_leaf_but_reclaims_equal_copy_aggregate() {
        let (mut c, mut session, original, _) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        for (units, expected_leaf) in [([65, 66], false), ([88, 89], true)] {
            let mut buffer = Utf16Buffer::new(&mut c).unwrap();
            buffer.extend_units(&units, &mut c).unwrap();
            let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
            let storage = session.active_mut().unwrap();
            let evidence = storage.evidence.as_ref().unwrap();
            if !expected_leaf {
                for origin in evidence.observed_origins(&original, &mut c).unwrap() {
                    maps.try_push(&mut c, origin).unwrap();
                }
            }
            let copied = evidence.copy_group_reference(&mut c, &original).unwrap();
            let value = buffer.freeze(&mut c).unwrap();
            let (native, slot) = storage
                .register_field_copy(
                    &mut c,
                    FieldCopy {
                        value,
                        origins: maps,
                        original: Some(copied),
                    },
                )
                .unwrap();
            drop(native);
            let before = c
                .usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current;
            storage.retire_registered_string(&mut c, slot).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                before - if expected_leaf { 0 } else { 2 }
            );
            let group = match &storage.payloads.values[slot] {
                RetainedPayload::String {
                    origins: Some(group),
                    ..
                } if expected_leaf => group,
                RetainedPayload::HistoricalString { origins } if !expected_leaf => origins,
                _ => panic!("actual retired storage"),
            };
            let evidence = storage.evidence.as_ref().unwrap();
            let header = evidence.require_group(group).unwrap();
            assert!(header.retirement.is_some());
            assert_eq!(
                matches!(header.body, provenance::GroupBody::Leaf { .. }),
                expected_leaf
            );
            let observed = evidence
                .materialize_group(&mut c, &storage.payloads, group)
                .unwrap();
            assert_eq!(observed.text().code_units(), units);
            ParseEvidence::release_materialized_group(&mut c, &mut Some(observed)).unwrap();
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn equal_units_with_distinct_atomic_origin_partition_keep_a_new_leaf() {
        let (mut c, mut session, original, source) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        buffer.extend_units(&[65, 66], &mut c).unwrap();
        let mut maps = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
        maps.try_push(
            &mut c,
            TextOrigin {
                start: 0,
                end: 2,
                source: source.range(0, 2).unwrap(),
                kind: crate::parser::tokenizer::token::OriginKind::Literal,
            },
        )
        .unwrap();
        let value = buffer.freeze(&mut c).unwrap();
        let storage = session.active_mut().unwrap();
        let copied = storage
            .evidence
            .as_ref()
            .unwrap()
            .copy_group_reference(&mut c, &original)
            .unwrap();
        let (native, slot) = storage
            .register_field_copy(
                &mut c,
                FieldCopy {
                    value,
                    origins: maps,
                    original: Some(copied),
                },
            )
            .unwrap();
        drop(native);
        let RetainedPayload::String {
            origins: Some(group),
            ..
        } = &storage.payloads.values[slot]
        else {
            panic!("actual leaf");
        };
        let evidence = storage.evidence.as_ref().unwrap();
        assert!(matches!(
            evidence.require_group(group).unwrap().body,
            provenance::GroupBody::Leaf { length: 1, .. }
        ));
        let rows = evidence.observed_origins(group, &mut c).unwrap();
        assert_eq!(
            rows.iter()
                .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                .collect::<Vec<_>>(),
            [(0, 2, 0, 2)]
        );
        assert_eq!(
            evidence
                .observed_origins(&original, &mut c)
                .unwrap()
                .iter()
                .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                .collect::<Vec<_>>(),
            [(0, 1, 0, 1), (1, 2, 1, 2)]
        );
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn materialized_history_alias_and_foreign_refusals_preserve_slot_for_real_cleanup() {
        let (mut c, mut session, group, _) =
            two_leaf_history(limits(), HtmlParseCancellation::new());
        let storage = session.active().unwrap();
        let mut output = Some(
            storage
                .evidence
                .as_ref()
                .unwrap()
                .materialize_group(&mut c, &storage.payloads, &group)
                .unwrap(),
        );
        let alias = output.as_ref().unwrap().text().clone();
        let before = c.usage().clone();
        assert_eq!(
            ParseEvidence::release_materialized_group(&mut c, &mut output),
            Err(SessionError::AliasedPayload)
        );
        assert!(output.is_some());
        assert_eq!(alias.code_units(), [65, 66]);
        assert_eq!(c.usage(), &before);
        assert!(c.stop().is_none());
        let mut foreign = control();
        assert_eq!(
            ParseEvidence::release_materialized_group(&mut foreign, &mut output),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        );
        assert!(output.is_some());
        assert_eq!(c.usage(), &before);
        assert!(c.stop().is_none());
        drop(alias);
        // The original slot can be released successfully after its alias dies.
        ParseEvidence::release_materialized_group(&mut c, &mut output).unwrap();
        assert!(output.is_none());
        assert!(c.stop().is_none());
        let storage = session.active().unwrap();
        output = Some(
            storage
                .evidence
                .as_ref()
                .unwrap()
                .materialize_group(&mut c, &storage.payloads, &group)
                .unwrap(),
        );
        // Actual work exhaustion latches the snapshot before cleanup.
        c.charge(WorkCategory::StateStep, limits().work_credits())
            .unwrap_err();
        let first = c.first_stop_usage().unwrap().clone();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        ParseEvidence::release_materialized_group(&mut c, &mut output).unwrap();
        assert!(output.is_none());
        assert_eq!(c.first_stop_usage(), Some(&first));
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        assert_eq!(
            ParseEvidence::release_materialized_group(&mut c, &mut output),
            Err(SessionError::Consumed)
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.first_stop_usage(), Some(&first));
        storage_zero(&foreign);
    }

    #[test]
    fn completed_projection_refuses_real_pending_leaf_attachment() {
        let source = crate::HtmlSource::utf8("A");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let group = session
            .evidence(&mut c)
            .unwrap()
            .rule_group(&mut c, 1)
            .unwrap();
        assert_eq!(
            session.prepare_native_associations(&mut c, &source).err(),
            Some(SessionError::WrongToken)
        );
        drop(group);
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn append_series_retains_unique_leaves_plus_current_output_at_every_sample() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // N>0 retains N origin atoms, 2N-1 group headers and 2N-1
        // causes; projection adds two native node rows plus N current atoms.
        // Exactly 6N initialized SourceMapEntries are therefore needed. Empty
        // input instead retains one Request cause and projects only its root.
        // Keep existing sample caps; only N256 uses this independently derived
        // logical allowance, including the meaningful final-row one-short cut.
        for (count, one_short) in [
            (0usize, false),
            (1, false),
            (8, false),
            (16, false),
            (32, false),
            (64, false),
            (256, false),
            (256, true),
        ] {
            let source = crate::HtmlSource::utf8("x".repeat(count));
            let caps = if count == 256 {
                limits()
                    .into_builder()
                    .source_map_entries((6 * count - usize::from(one_short)) as u64)
                    .finish()
                    .unwrap()
            } else {
                limits()
            };
            let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
            c.admit_source(&source).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            session.initialize_parse_evidence(&mut c, &source).unwrap();
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let mut last = None;
            for position in 0..count {
                let mut text = Some(match lexical_kind(&mut lexer, &mut c) {
                    TokenKind::Text(text) => text,
                    _ => panic!("literal x"),
                });
                last = Some(
                    session
                        .insert_text(&mut c, &root, None, &mut text, None)
                        .unwrap()
                        .unwrap(),
                );
                assert_eq!(last.as_ref().unwrap().offset, position);
                let allowance = if position == 0 { 1 } else { 2 * (position + 1) };
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    allowance as u64
                );
            }
            let last = last.map(|inserted| inserted.node);
            if let Some(last) = &last {
                assert_eq!(native_text(&session, last), vec![120; count]);
            } else {
                assert!(session.active().unwrap().records[0].children.is_empty());
            }
            // Metadata is one leaf per ingress, one Concat per merge and
            // fixed cause/retirement links. No historical-prefix atom dump.
            let evidence = session.active().unwrap().evidence.as_ref().unwrap();
            assert_eq!(
                evidence.history_counts(),
                if count == 0 {
                    (0, 0, 1)
                } else {
                    (count, 2 * count - 1, 2 * count - 1)
                }
            );
            lexer.release(&mut c).unwrap();
            let retained = c.usage().clone();
            let projection = session.prepare_native_associations(&mut c, &source);
            if one_short {
                assert_eq!(
                    projection,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::SourceMapEntries,
                        limit: 1535,
                        current: 1535,
                        requested_delta: 1,
                    }))
                );
                let first = c.first_stop_usage().unwrap().clone();
                assert_eq!(
                    first.resource(HtmlParseResource::SourceMapEntries).current,
                    1535
                );
                assert_eq!(
                    first.resource(HtmlParseResource::ManagedUtf16Units).current,
                    512
                );
                assert_eq!(
                    session.native_associations(&mut c).err(),
                    Some(SessionError::Stop(c.stop().unwrap()))
                );
                session.release(&mut c).unwrap();
                storage_zero(&c);
                assert_eq!(c.first_stop_usage(), Some(&first));
                continue;
            }
            projection.unwrap();
            let projected = c.usage().clone();
            assert_eq!(
                projected
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                if count == 0 { 2 } else { 6 * count as u64 }
            );
            {
                let mut rows = session.native_associations(&mut c).unwrap();
                // Actual payload registration orders the first Text leaf
                // before that Text's node-cause slot. After a merge, its
                // detached leaf emits no row and the current aggregate follows
                // both node-cause slots. Only current Text emits output rows.
                assert_eq!(rows.len(), if count == 0 { 1 } else { count + 2 });
                assert!(rows.node_for(0, &root).unwrap());
                if let Some(last) = &last {
                    assert!(rows.node_for(if count == 1 { 2 } else { 1 }, last).unwrap());
                }
                for index in 0..count {
                    let output_row = index + if count == 1 { 1 } else { 2 };
                    assert!(rows.text_for(output_row, last.as_ref().unwrap()).unwrap());
                    let (row, _) = rows.row(output_row).unwrap().unwrap();
                    assert_eq!((row.start, row.end), (index, index + 1));
                    assert_eq!(
                        row.source.as_ref(),
                        Some(&source.range(index, index + 1).unwrap())
                    );
                    assert_eq!(
                        row.kind,
                        Some(crate::parser::tokenizer::token::OriginKind::Literal)
                    );
                }
            }
            let categories = [
                WorkCategory::StateStep,
                WorkCategory::InputUnitRead,
                WorkCategory::InspectByte,
                WorkCategory::CopyByte,
                WorkCategory::RecordVisit,
                WorkCategory::RecordWrite,
                WorkCategory::ReserveByte,
                WorkCategory::ProviderCall,
            ];
            eprintln!(
                "append={count} retained_capacity={} retained_peak={} retained_work={:?} projection_work_delta={:?} projected_capacity={} projected_peak={}",
                retained.actual_capacity_bytes,
                retained.actual_capacity_high_water,
                categories.map(|category| retained.work(category)),
                categories.map(|category| projected.work(category) - retained.work(category)),
                projected.actual_capacity_bytes,
                projected.actual_capacity_high_water
            );
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    fn assert_merged_text_retains_qualified_origins(
        source: &crate::HtmlSource,
        expected: Vec<(
            usize,
            usize,
            crate::HtmlSourceRange,
            crate::parser::tokenizer::token::OriginKind,
        )>,
    ) {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        let mut c = control();
        c.admit_source(source).unwrap();
        let mut lexer = LexerFixture::new(source, State::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut original: Option<ParserNode> = None;
        let mut first_origins = None;
        let mut incoming_origins = Vec::new();
        let mut output_length = 0;
        loop {
            match lexical_kind(&mut lexer, &mut c) {
                TokenKind::Text(text) => {
                    let incoming_length = text.value.text().len();
                    incoming_origins.extend(text.origins.as_slice().iter().map(|origin| {
                        (
                            output_length + origin.start,
                            output_length + origin.end,
                            origin.source.clone(),
                            origin.kind,
                        )
                    }));
                    let mut text = Some(text);
                    let inserted = session
                        .insert_text(&mut c, &root, None, &mut text, None)
                        .unwrap()
                        .unwrap();
                    assert!(text.is_none());
                    assert_eq!(inserted.offset, output_length);
                    output_length += incoming_length;
                    if let Some(node) = &original {
                        assert_eq!(inserted.node.index, node.index);
                    } else {
                        let storage = session.active_mut().unwrap();
                        let slot = storage
                            .field_slot(&mut c, StringLocation::Text(inserted.node.index))
                            .unwrap();
                        let RetainedPayload::String { origins, .. } =
                            &storage.payloads.values[slot]
                        else {
                            panic!("registered native Text field");
                        };
                        first_origins = origins.as_ref().map(|origins| {
                            storage
                                .observed_origins(origins, &mut c)
                                .unwrap()
                                .iter()
                                .map(|origin| {
                                    (origin.start, origin.end, origin.source.clone(), origin.kind)
                                })
                                .collect::<Vec<_>>()
                        });
                        original = Some(inserted.node);
                    }
                }
                TokenKind::Eof => break,
                _ => panic!("literal fixture contains only text and EOF"),
            }
        }
        let node = original.unwrap();
        let units = native_text(&session, &node);
        let storage = session.active_mut().unwrap();
        let slot = storage
            .field_slot(&mut c, StringLocation::Text(node.index))
            .unwrap();
        let RetainedPayload::String { origins, .. } = &storage.payloads.values[slot] else {
            panic!("registered native Text field");
        };
        let actual = origins.as_ref().map(|origins| {
            storage
                .observed_origins(origins, &mut c)
                .unwrap()
                .iter()
                .map(|origin| (origin.start, origin.end, origin.source.clone(), origin.kind))
                .collect::<Vec<_>>()
        });
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(
            c.usage().resource(HtmlParseResource::Diagnostics).current,
            0
        );
        assert_eq!(units, [0x41, 0xd83d, 0xde42, 0x0a, 0x2242, 0x0338]);
        assert_eq!(first_origins, Some(vec![expected[0].clone()]));
        assert_eq!(
            incoming_origins, expected,
            "actual tokenizer origins before native transfer"
        );
        assert_eq!(
            actual,
            Some(expected),
            "merged native field must retain rebased source origins"
        );
    }

    #[test]
    fn merged_text_retains_rebased_utf8_scalar_newline_and_entity_origins() {
        use crate::parser::tokenizer::token::OriginKind::{CharacterReference, Literal, Newline};
        let source = crate::HtmlSource::utf8("A🙂\r\n&NotEqualTilde;");
        assert_merged_text_retains_qualified_origins(
            &source,
            vec![
                (0, 1, source.range(0, 1).unwrap(), Literal),
                (1, 3, source.range(1, 5).unwrap(), Literal),
                (3, 4, source.range(5, 7).unwrap(), Newline),
                (4, 5, source.range(7, 22).unwrap(), CharacterReference),
                (5, 6, source.range(7, 22).unwrap(), CharacterReference),
            ],
        );
    }

    #[test]
    fn merged_text_retains_rebased_utf16_scalar_newline_and_entity_origins() {
        use crate::parser::tokenizer::token::OriginKind::{CharacterReference, Literal, Newline};
        let source = crate::HtmlSource::utf16(DocumentString::from("A🙂\r\n&NotEqualTilde;"));
        assert_merged_text_retains_qualified_origins(
            &source,
            vec![
                (0, 1, source.range(0, 1).unwrap(), Literal),
                (1, 3, source.range(1, 3).unwrap(), Literal),
                (3, 4, source.range(3, 5).unwrap(), Newline),
                (4, 5, source.range(5, 20).unwrap(), CharacterReference),
                (5, 6, source.range(5, 20).unwrap(), CharacterReference),
            ],
        );
    }

    #[test]
    fn before_sibling_merges_only_preceding_text_and_document_ignores_text() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let element = session.allocate_element(&mut c, html("div")).unwrap();
        session.insert(&mut c, &root, &element, None, None).unwrap();
        let mut a = Some(owned_text(&mut c, &[0x61]));
        let a = session
            .insert_text(&mut c, &root, Some(&element), &mut a, None)
            .unwrap()
            .unwrap();
        let mut b = Some(owned_text(&mut c, &[0x62]));
        let b = session
            .insert_text(&mut c, &root, Some(&element), &mut b, None)
            .unwrap()
            .unwrap();
        assert_eq!(a.node.index, b.node.index);
        assert_eq!(native_text(&session, &a.node), [0x61, 0x62]);
        let mut after = Some(owned_text(&mut c, &[0x63]));
        let after = session
            .insert_text(&mut c, &root, None, &mut after, None)
            .unwrap()
            .unwrap();
        assert_ne!(a.node.index, after.node.index);
        session.release(&mut c).unwrap();
        storage_zero(&c);
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        assert!(session.fragment_root(&mut c).unwrap().is_none());
        let mut text = Some(owned_text(&mut c, &[0x61]));
        assert!(
            session
                .insert_text(&mut c, &root, None, &mut text, None)
                .unwrap()
                .is_none()
        );
        assert!(text.is_none());
        assert_eq!(session.active().unwrap().records.len(), 1);
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn wrong_case_and_duplicate_token_attributes_do_not_bypass_expanded_name_validation() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut tag = Some(owned_tag(&mut c, "DIV", &[]));
        assert!(matches!(
            session.allocate_html_tag(&mut c, &mut tag),
            Err(SessionError::WrongToken)
        ));
        assert!(tag.is_some());
        let Tag {
            name, attributes, ..
        } = tag.take().unwrap();
        name.release(&mut c).unwrap();
        release_token_attributes(attributes, &mut c).unwrap();
        let mut tag = Some(owned_tag(&mut c, "div", &[("x", &[0x61]), ("x", &[0x62])]));
        assert!(!tag.as_ref().unwrap().had_duplicate_attributes);
        assert!(matches!(
            session.allocate_html_tag(&mut c, &mut tag),
            Err(SessionError::DuplicateAttribute)
        ));
        assert!(tag.is_none());
        assert_eq!(session.active().unwrap().records.len(), 1);
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn external_input_and_native_backing_aliases_reject_without_consuming_owners() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut text = Some(owned_text(&mut c, &[0x61]));
        let alias = text.as_ref().unwrap().value.text().clone();
        assert!(matches!(
            session.insert_text(&mut c, &root, None, &mut text, None),
            Err(SessionError::AliasedPayload)
        ));
        assert!(text.is_some());
        drop(alias);
        let inserted = session
            .insert_text(&mut c, &root, None, &mut text, None)
            .unwrap()
            .unwrap();
        let HtmlNodeKind::Text(value) =
            &session.active().unwrap().records[inserted.node.index].kind
        else {
            panic!("text")
        };
        let alias = value.clone();
        let mut frozen = session.freeze(&mut c).unwrap();
        let capacity = c
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        assert!(matches!(
            frozen.release(&mut c),
            Err(SessionError::AliasedPayload)
        ));
        assert!(frozen.fragment().is_some());
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            capacity
        );
        drop(alias);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn foreign_token_owner_preserves_input_and_original_ledger_for_cleanup() {
        let mut c = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut tag = Some(owned_tag(&mut foreign, "div", &[]));
        assert!(matches!(
            session.allocate_html_tag(&mut c, &mut tag),
            Err(SessionError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::ForeignControl
            )))
        ));
        assert!(tag.is_some());
        let Tag {
            name, attributes, ..
        } = tag.take().unwrap();
        name.release(&mut foreign).unwrap();
        release_token_attributes(attributes, &mut foreign).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        storage_zero(&foreign);
    }
    #[test]
    fn managed_string_cap_stops_before_text_replacement_and_retains_real_failed_payload() {
        let caps = limits()
            .into_builder()
            .managed_utf16_units(1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut first = Some(owned_text(&mut c, &[0x61]));
        let first = session
            .insert_text(&mut c, &root, None, &mut first, None)
            .unwrap()
            .unwrap();
        let mut second = Some(owned_text(&mut c, &[0x62]));
        let error = session.insert_text(&mut c, &root, None, &mut second, None);
        assert!(matches!(
            error,
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                limit: 1,
                current: 1,
                requested_delta: 1
            }))
        ));
        assert_eq!(native_text(&session, &first.node), [0x61]);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            1
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        let stop = c.stop();
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), stop);
        storage_zero(&c);
    }

    #[test]
    fn detached_allocations_and_ordered_moves_keep_permanent_nodes_and_exact_views() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session.allocate_element(&mut control, html("a")).unwrap();
        let b = session.allocate_element(&mut control, html("b")).unwrap();
        let c = session.allocate_element(&mut control, html("c")).unwrap();
        for node in [&a, &b, &c] {
            session
                .insert(&mut control, &root, node, None, None)
                .unwrap();
        }
        session
            .insert(&mut control, &root, &a, Some(&c), None)
            .unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [b.index, a.index, c.index]
        );
        session
            .insert(&mut control, &root, &a, Some(&a), None)
            .unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [b.index, a.index, c.index]
        );
        session
            .insert(&mut control, &root, &c, Some(&b), None)
            .unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [c.index, b.index, a.index]
        );
        session.remove(&mut control, &b, None).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            4
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            2
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .high_water,
            3
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        assert_eq!(locals(fragment), ["c", "a"]);
        assert_eq!(fragment.usage().nodes(), 4);
        assert_eq!(fragment.usage().edges(), 2);
        let detached_id = fragment.arena.id(b.index);
        assert!(fragment.node(&detached_id).unwrap().parent().is_none());
        assert_eq!(fragment.arena.records[b.index].depth, 0);
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            4
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
        assert!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water
                > 0
        );
    }

    #[test]
    fn paired_template_contents_are_separate_and_association_cycles_are_rejected() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let template = session
            .allocate_element(&mut control, html("template"))
            .unwrap();
        let contents = session
            .template_contents(&mut control, &template)
            .unwrap()
            .unwrap();
        let child = session.allocate_element(&mut control, html("x")).unwrap();
        session
            .insert(&mut control, &root, &template, None, None)
            .unwrap();
        session
            .insert(&mut control, &contents, &child, None, None)
            .unwrap();
        assert_eq!(
            session.insert(&mut control, &child, &template, None, None),
            Err(SessionError::Cycle)
        );
        assert_eq!(
            session.insert(&mut control, &template, &child, None, None),
            Err(SessionError::TemplateContentsRequired)
        );
        assert_eq!(
            session.insert(&mut control, &root, &contents, None, None),
            Err(SessionError::AssociationMove)
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        assert_eq!(fragment.usage().nodes(), 4);
        assert_eq!(fragment.usage().edges(), 3);
        assert_eq!(fragment.usage().depth(), 3);
        let template_id = fragment.root().children().next().unwrap();
        let template_node = fragment.node(&template_id).unwrap();
        assert_eq!(template_node.children().len(), 0);
        let contents_id = template_node.template_contents().unwrap();
        assert_eq!(
            fragment.node(&contents_id).unwrap().fragment_owner(),
            Some(&HtmlFragmentOwner::Template(template_id))
        );
        assert!(fragment.node(&contents_id).unwrap().parent().is_none());
        assert_eq!(
            fragment
                .fragment(&contents_id)
                .unwrap()
                .unwrap()
                .root()
                .children()
                .len(),
            1
        );
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn foreign_template_and_namespaced_attributes_keep_preexisting_exact_slots() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let payload = HtmlElement {
            name: QualifiedName::new(
                Namespace::none(),
                Some(DocumentString::from("p")),
                DocumentString::from_utf16(vec![0xD800, 32]),
            ),
            attributes: OrderedAttributes::new(vec![Attribute::new(
                QualifiedName::new(Namespace::none(), None, "v"),
                DocumentString::from_utf16(vec![0xDC00]),
            )])
            .unwrap(),
        };
        let backing = payload.name().local_name().code_units().as_ptr();
        let raw = session.allocate_element(&mut control, payload).unwrap();
        let foreign = session
            .allocate_element(&mut control, element(Namespace::svg(), "template"))
            .unwrap();
        assert!(
            session
                .template_contents(&mut control, &foreign)
                .unwrap()
                .is_none()
        );
        session
            .insert(&mut control, &root, &raw, None, None)
            .unwrap();
        session
            .insert(&mut control, &root, &foreign, None, None)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedAttributes)
                .current,
            1
        );
        // 2 local + 1 prefix + 1 attribute local + 1 attribute value; SVG URI
        // http://www.w3.org/2000/svg (26) plus template (8).
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            39
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        let id = fragment.root().children().next().unwrap();
        let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
            panic!("expected element")
        };
        assert_eq!(element.name().local_name().code_units(), [0xD800, 32]);
        assert_eq!(element.name().local_name().code_units().as_ptr(), backing);
        assert_eq!(
            element.attributes().as_slice()[0].value().code_units(),
            [0xDC00]
        );
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn node_and_depth_cap_failures_leave_topology_unmodified_and_cleanup_releases_capacity() {
        let mut selected = limits();
        selected = selected.into_builder().managed_nodes(2).finish().unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        assert!(matches!(
            session.allocate_element(&mut control, html("template")),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedNodes,
                ..
            }))
        ));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(session.active().unwrap().records[0].children, []);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            1
        );
        assert!(session.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);

        let mut selected = limits();
        selected = selected.into_builder().tree_depth(2).finish().unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let template = session
            .allocate_element(&mut control, html("template"))
            .unwrap();
        let contents = session
            .template_contents(&mut control, &template)
            .unwrap()
            .unwrap();
        let child = session.allocate_element(&mut control, html("x")).unwrap();
        session
            .insert(&mut control, &contents, &child, None, None)
            .unwrap();
        assert!(matches!(
            session.insert(&mut control, &root, &template, None, None),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::TreeDepth,
                ..
            }))
        ));
        assert!(session.active().unwrap().records[0].children.is_empty());
        assert!(
            session.active().unwrap().records[template.index]
                .parent
                .is_none()
        );
        assert_eq!(
            session.active().unwrap().records[contents.index].children,
            [child.index]
        );
        assert!(session.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn foreign_handles_controls_roots_and_wrong_siblings_reject_before_topology_change() {
        let mut owner = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
        let mut other = ParserArenaSession::new(&mut foreign, RootKind::Fragment).unwrap();
        let root = session.root(&mut owner).unwrap();
        let foreign_root = other.root(&mut foreign).unwrap();
        let a = session.allocate_element(&mut owner, html("a")).unwrap();
        let b = session.allocate_element(&mut owner, html("b")).unwrap();
        assert_eq!(
            session.insert(&mut owner, &root, &foreign_root, None, None),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session.insert(&mut owner, &a, &root, None, None),
            Err(SessionError::RootMove)
        );
        assert_eq!(
            session.insert(&mut owner, &root, &a, Some(&b), None),
            Err(SessionError::WrongSibling)
        );
        assert!(session.active().unwrap().records[0].children.is_empty());
        let before = foreign.usage().clone();
        assert!(matches!(
            session.insert(&mut foreign, &root, &a, None, None),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(
            foreign.usage().actual_capacity_bytes,
            before.actual_capacity_bytes
        );
        assert_eq!(
            foreign.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert!(other.release(&mut foreign).is_ok());
        assert!(session.release(&mut owner).is_ok());
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
        assert_eq!(foreign.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn failed_freeze_and_foreign_cleanup_retain_owned_tickets_for_recovery() {
        let mut owner = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
        assert!(matches!(
            session.release(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        let cancellation = HtmlParseCancellation::new();
        let mut stopped = ParseControl::new(limits(), cancellation.clone());
        let mut candidate = ParserArenaSession::new(&mut stopped, RootKind::Fragment).unwrap();
        candidate.allocate_element(&mut stopped, html("x")).unwrap();
        cancellation.cancel();
        assert!(matches!(
            candidate.freeze(&mut stopped),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert!(candidate.release(&mut stopped).is_ok());
        assert_eq!(stopped.usage().actual_capacity_bytes, 0);
        assert_eq!(
            stopped
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            2
        );
        assert!(session.release(&mut owner).is_ok());
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn frozen_document_uses_existing_mode_and_old_arena_survives_later_session() {
        let mut first = control();
        let mut session = ParserArenaSession::new(
            &mut first,
            RootKind::Document(HtmlDocumentMode::LimitedQuirks),
        )
        .unwrap();
        let root = session.root(&mut first).unwrap();
        let node = session.allocate_element(&mut first, html("html")).unwrap();
        session
            .insert(&mut first, &root, &node, None, None)
            .unwrap();
        let mut output = frozen(session, &mut first);
        let old = output.document().unwrap().clone();
        let old_id = old.root().children().next().unwrap();
        assert_eq!(old.mode(), HtmlDocumentMode::LimitedQuirks);
        let mut second = control();
        let mut later = ParserArenaSession::new(&mut second, RootKind::Fragment).unwrap();
        later.allocate_element(&mut second, html("new")).unwrap();
        let mut later = frozen(later, &mut second);
        assert!(later.fragment().unwrap().node(&old_id).is_err());
        assert_eq!(old.root().children().next().unwrap(), old_id);
        assert!(matches!(
            output.release(&mut first),
            Err(SessionError::Stop(_))
        ));
        drop(old);
        assert!(output.release(&mut first).is_ok());
        assert!(later.release(&mut second).is_ok());
        assert_eq!(first.usage().actual_capacity_bytes, 0);
        assert_eq!(second.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn deep_topology_freezes_without_recursive_tree_or_child_depth_cache() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let mut parent = session.root(&mut control).unwrap();
        for _ in 0..48 {
            let child = session.allocate_element(&mut control, html("x")).unwrap();
            session
                .insert(&mut control, &parent, &child, None, None)
                .unwrap();
            parent = child;
        }
        let mut output = frozen(session, &mut control);
        assert_eq!(output.fragment().unwrap().usage().depth(), 48);
        assert_eq!(
            output.fragment().unwrap().arena.records[parent.index].depth,
            48
        );
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn cross_parent_move_changes_both_orders_and_backlinks_without_new_nodes_or_edges() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session.allocate_element(&mut control, html("a")).unwrap();
        let b = session.allocate_element(&mut control, html("b")).unwrap();
        let c = session.allocate_element(&mut control, html("c")).unwrap();
        let d = session.allocate_element(&mut control, html("d")).unwrap();
        session.insert(&mut control, &root, &a, None, None).unwrap();
        session.insert(&mut control, &root, &b, None, None).unwrap();
        session.insert(&mut control, &a, &c, None, None).unwrap();
        session.insert(&mut control, &b, &d, None, None).unwrap();
        session
            .insert(&mut control, &b, &c, Some(&d), None)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            5
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            4
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .high_water,
            4
        );
        let mut output = frozen(session, &mut control);
        let fragment = output.fragment().unwrap();
        let a_id = fragment.arena.id(a.index);
        let b_id = fragment.arena.id(b.index);
        let c_id = fragment.arena.id(c.index);
        let d_id = fragment.arena.id(d.index);
        assert_eq!(fragment.node(&a_id).unwrap().children().len(), 0);
        assert_eq!(
            fragment.node(&b_id).unwrap().children().collect::<Vec<_>>(),
            [c_id.clone(), d_id]
        );
        assert_eq!(fragment.node(&c_id).unwrap().parent(), Some(b_id));
        assert_eq!(fragment.usage().depth(), 2);
        assert!(output.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn edge_preflight_and_zero_physical_capacity_reject_before_actual_record_admission() {
        let mut selected = limits();
        selected = selected.into_builder().managed_edges(0).finish().unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let bytes_before = control.usage().actual_capacity_bytes;
        assert!(matches!(
            session.allocate_element(&mut control, html("template")),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedEdges,
                ..
            }))
        ));
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            1
        );
        assert_eq!(control.usage().actual_capacity_bytes, bytes_before);
        assert!(session.release(&mut control).is_ok());
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        let mut selected = limits();
        selected = selected
            .into_builder()
            .managed_capacity_bytes(0)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        assert!(matches!(
            ParserArenaSession::new(&mut control, RootKind::Fragment),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }))
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            0
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn successful_transfers_consume_each_owner_once_without_failure_time_allocation() {
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let records_pointer = session.active().unwrap().records.as_ptr();
        let mut output = session
            .freeze(&mut control)
            .unwrap_or_else(|error| panic!("freeze failed: {error:?}"));
        assert_eq!(
            output.fragment().unwrap().arena.records.as_ptr(),
            records_pointer
        );
        let before = control.usage().clone();
        assert!(matches!(
            session.root(&mut control),
            Err(SessionError::Consumed)
        ));
        assert!(matches!(
            session.freeze(&mut control),
            Err(SessionError::Consumed)
        ));
        assert_eq!(session.release(&mut control), Err(SessionError::Consumed));
        assert_eq!(control.usage(), &before);
        output.release(&mut control).unwrap();
        assert!(output.fragment().is_none());
        assert!(output.document().is_none());
        let before = control.usage().clone();
        assert_eq!(output.release(&mut control), Err(SessionError::Consumed));
        assert_eq!(control.usage(), &before);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn foreign_freeze_and_frozen_cleanup_preserve_the_borrowed_original_owner() {
        let mut owner = control();
        let mut foreign = control();
        let mut session = ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap();
        let root = session.root(&mut owner).unwrap();
        let node = session.allocate_element(&mut owner, html("x")).unwrap();
        session
            .insert(&mut owner, &root, &node, None, None)
            .unwrap();
        let before = owner.usage().clone();
        assert!(matches!(
            session.freeze(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(session.active().unwrap().records[0].children, [node.index]);
        assert_eq!(owner.usage(), &before);
        let mut output = session
            .freeze(&mut owner)
            .unwrap_or_else(|error| panic!("freeze failed: {error:?}"));
        let before = owner.usage().clone();
        assert!(matches!(
            output.release(&mut foreign),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(locals(output.fragment().unwrap()), ["x"]);
        assert_eq!(owner.usage(), &before);
        output.release(&mut owner).unwrap();
        assert_eq!(owner.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn final_shared_payload_cap_stop_restores_actual_vectors_and_all_tickets() {
        let mut selected = limits();
        // A root owns one actual record slot and one actual child-vector ticket;
        // its empty child vector and zero-sized identity add no known payload.
        let root_capacity = mem::size_of::<Record>() + mem::size_of::<VectorCapacity<usize>>();
        selected = selected
            .into_builder()
            .managed_capacity_bytes(root_capacity as u64)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let records_pointer = session.active().unwrap().records.as_ptr();
        assert!(
            matches!(session.freeze(&mut control), Err(SessionError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            current,
            requested_delta,
            ..
        })) if current == root_capacity as u64 && requested_delta == mem::size_of::<Arena>() as u64)
        );
        assert_eq!(session.active().unwrap().records.as_ptr(), records_pointer);
        assert_eq!(session.active().unwrap().records.len(), 1);
        assert_eq!(control.usage().actual_capacity_bytes, root_capacity as u64);
        session.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            1
        );
    }

    #[test]
    fn empty_end_and_before_sibling_preparations_charge_each_inserted_index_copy() {
        let word = mem::size_of::<usize>() as u64;
        let vector_records =
            (mem::size_of::<Vec<usize>>() + mem::size_of::<VectorCapacity<usize>>()) as u64;
        let mut control = control();
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session
            .allocate_element(&mut control, element(Namespace::none(), "a"))
            .unwrap();
        let before = control.usage().work(WorkCategory::CopyByte);
        let storage = session.active().unwrap();
        let (order, allocation) = storage
            .child_order(&mut control, 0, usize::MAX, Some((a.index, None)))
            .unwrap();
        // Actual Vec/ticket record initialization, capacity field, inserted index.
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - before,
            vector_records + 2 * word
        );
        assert_eq!(order, [a.index]);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            3
        );
        release_order(&mut control, &storage.control, order, allocation).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            2
        );
        session.insert(&mut control, &root, &a, None, None).unwrap();
        let b = session
            .allocate_element(&mut control, element(Namespace::none(), "b"))
            .unwrap();
        let before = control.usage().work(WorkCategory::CopyByte);
        let storage = session.active().unwrap();
        let (order, allocation) = storage
            .child_order(&mut control, 0, usize::MAX, Some((b.index, Some(a.index))))
            .unwrap();
        // The existing child contributes a separate word; insertion contributes
        // another word even though that existing-child loop is already charged.
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - before,
            vector_records + 3 * word
        );
        assert_eq!(order, [b.index, a.index]);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            5
        );
        release_order(&mut control, &storage.control, order, allocation).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            3
        );
        session.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn inserted_index_copy_work_boundary_stops_before_any_scratch_slot_is_initialized() {
        let mut selected = limits();
        let work_limit = selected.work_credits();
        selected = selected
            .into_builder()
            .auxiliary_entries(3)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let a = session
            .allocate_element(&mut control, element(Namespace::none(), "a"))
            .unwrap();
        let word = mem::size_of::<usize>() as u64;
        let records =
            (mem::size_of::<Vec<usize>>() + mem::size_of::<VectorCapacity<usize>>()) as u64;
        // Before the inserted copy: new Vec/ticket has 2 writes + record bytes;
        // parent visit costs 1; reserve has 2 visits, one word ReserveByte,
        // provider call, capacity write and one word CopyByte. Total: records+7+2*word.
        let prefix = records + 7 + 2 * word;
        let before = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        control
            .charge(
                WorkCategory::StateStep,
                work_limit - before - prefix - (word - 1),
            )
            .unwrap();
        let copy_before = control.usage().work(WorkCategory::CopyByte);
        let capacity_before = control.usage().actual_capacity_bytes;
        let storage = session.active().unwrap();
        assert!(
            matches!(storage.child_order(&mut control, 0, usize::MAX, Some((a.index, None))),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                requested_delta,
                ..
            })) if requested_delta == word)
        );
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - copy_before,
            records + word
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            2
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .high_water,
            2
        );
        assert_eq!(control.usage().actual_capacity_bytes, capacity_before);
        assert!(control.usage().actual_capacity_high_water > capacity_before);
        assert!(storage.records[0].children.is_empty());
        let stop = control.stop();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        session.release(&mut control).unwrap();
        assert_eq!(control.stop(), stop);
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn one_scratch_slot_exact_limit_succeeds_and_one_short_preserves_topology() {
        for (auxiliary_limit, succeeds) in [(3, true), (2, false)] {
            let mut selected = limits();
            selected = selected
                .into_builder()
                .auxiliary_entries(auxiliary_limit)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
            let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
            let root = session.root(&mut control).unwrap();
            let child = session
                .allocate_element(&mut control, element(Namespace::none(), "x"))
                .unwrap();
            let result = session.insert(&mut control, &root, &child, None, None);
            if succeeds {
                result.unwrap();
                assert_eq!(session.active().unwrap().records[0].children, [child.index]);
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::ManagedEdges)
                        .current,
                    1
                );
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .high_water,
                    3
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        current: 2,
                        requested_delta: 1,
                        ..
                    }))
                ));
                assert!(session.active().unwrap().records[0].children.is_empty());
                assert!(
                    session.active().unwrap().records[child.index]
                        .parent
                        .is_none()
                );
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::ManagedEdges)
                        .current,
                    0
                );
            }
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                2
            );
            session.release(&mut control).unwrap();
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                0
            );
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                2
            );
            assert_eq!(control.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn simultaneous_cross_parent_orders_require_both_live_scratch_sequences() {
        for (auxiliary_limit, succeeds) in [(9, true), (8, false)] {
            let mut selected = limits();
            selected = selected
                .into_builder()
                .auxiliary_entries(auxiliary_limit)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
            let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
            let root = session.root(&mut control).unwrap();
            let mut nodes = Vec::new();
            for name in ["a", "b", "c", "d", "e"] {
                nodes.push(
                    session
                        .allocate_element(&mut control, element(Namespace::none(), name))
                        .unwrap(),
                );
            }
            let [a, b, c, d, e] = nodes.as_slice() else {
                panic!("expected five fixture handles")
            };
            session.insert(&mut control, &root, a, None, None).unwrap();
            session.insert(&mut control, &root, b, None, None).unwrap();
            session.insert(&mut control, a, c, None, None).unwrap();
            session.insert(&mut control, a, e, None, None).unwrap();
            session.insert(&mut control, b, d, None, None).unwrap();
            let capacity_before = control.usage().actual_capacity_bytes;
            let result = session.insert(&mut control, b, c, Some(d), None);
            if succeeds {
                result.unwrap();
                assert_eq!(
                    session.active().unwrap().records[a.index].children,
                    [e.index]
                );
                assert_eq!(
                    session.active().unwrap().records[b.index].children,
                    [c.index, d.index]
                );
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .high_water,
                    9
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        current: 8,
                        requested_delta: 1,
                        ..
                    }))
                ));
                assert_eq!(
                    session.active().unwrap().records[a.index].children,
                    [c.index, e.index]
                );
                assert_eq!(
                    session.active().unwrap().records[b.index].children,
                    [d.index]
                );
                assert_eq!(control.usage().actual_capacity_bytes, capacity_before);
                assert!(control.usage().actual_capacity_high_water > capacity_before);
            }
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                6
            );
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::ManagedEdges)
                    .current,
                5
            );
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                6
            );
            let stop = control.stop();
            let work = control.usage().resource(HtmlParseResource::WorkCredits);
            session.release(&mut control).unwrap();
            assert_eq!(control.stop(), stop);
            assert_eq!(
                control.usage().resource(HtmlParseResource::WorkCredits),
                work
            );
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                0
            );
            assert_eq!(control.usage().actual_capacity_bytes, 0);
        }
    }

    #[test]
    fn partially_initialized_order_cap_stop_releases_only_its_one_admitted_entry() {
        let mut selected = limits();
        selected = selected
            .into_builder()
            .auxiliary_entries(5)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
        let root = session.root(&mut control).unwrap();
        let a = session
            .allocate_element(&mut control, element(Namespace::none(), "a"))
            .unwrap();
        session.insert(&mut control, &root, &a, None, None).unwrap();
        let b = session
            .allocate_element(&mut control, element(Namespace::none(), "b"))
            .unwrap();
        session.insert(&mut control, &root, &b, None, None).unwrap();
        let c = session
            .allocate_element(&mut control, element(Namespace::none(), "c"))
            .unwrap();
        let capacity_before = control.usage().actual_capacity_bytes;
        assert!(matches!(
            session.active().unwrap().child_order(
                &mut control,
                0,
                usize::MAX,
                Some((c.index, None))
            ),
            Err(SessionError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                current: 5,
                requested_delta: 1,
                ..
            }))
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            4
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .high_water,
            5
        );
        assert_eq!(control.usage().actual_capacity_bytes, capacity_before);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            2
        );
        assert_eq!(
            session.active().unwrap().records[0].children,
            [a.index, b.index]
        );
        assert!(session.active().unwrap().records[c.index].parent.is_none());
        let stop = control.stop();
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        session.release(&mut control).unwrap();
        assert_eq!(control.stop(), stop);
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            4
        );
        assert_eq!(control.usage().actual_capacity_bytes, 0);
    }

    #[test]
    fn template_real_document_pair_node_cap_rejects_before_lexical_take_and_disposes_all_owners() {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // The ordinary root already exists. Creation needs one actual inert
        // Document, the template Element and its hosted Fragment: three more.
        for cap in [3, 4] {
            let source = crate::HtmlSource::utf8("<template>");
            let mut c = ParseControl::new(
                limits().into_builder().managed_nodes(cap).finish().unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
                panic!("actual template")
            };
            let mut input = Some(tag);
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &root,
                None,
                CreationContext::Ordinary,
            );
            if cap == 4 {
                let template = result.unwrap();
                assert!(input.is_none());
                let state = session.template_state(&mut c, &template).unwrap();
                let inert = session
                    .node_document(&mut c, &state.contents)
                    .unwrap()
                    .node(&mut c, &session)
                    .unwrap();
                assert_ne!(inert.index, root.index);
                assert_eq!(session.active().unwrap().records.len(), 4);
                assert!(state.target.is_none() && state.start.is_none() && state.end.is_none());
            } else {
                let Err(SessionError::Stop(first)) = result else {
                    panic!("real node cap")
                };
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedNodes,
                        limit: 3,
                        requested_delta: 3,
                        ..
                    }
                ));
                assert!(input.is_some());
                assert_eq!(session.active().unwrap().records.len(), 1);
                assert!(
                    !session
                        .active()
                        .unwrap()
                        .payloads
                        .values
                        .iter()
                        .any(|entry| matches!(entry, RetainedPayload::Template { .. }))
                );
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
            }
            if let Some(tag) = input {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn template_document_and_facts_slots_have_independent_pretransfer_and_installed_cap_boundaries()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Root record/self-document/cache facts =3 slots. Fresh template
        // adds three native records, five document/template facts, and the
        // three name/namespace/attribute owners: completed total14. The
        // metadata/native preflight alone needs3+8=11 before lexical take.
        for cap in [10, 13, 14] {
            let source = crate::HtmlSource::utf8("<template>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
                panic!("actual template")
            };
            let mut input = Some(tag);
            let result = session.allocate_html_tag_for_creation(
                &mut c,
                &mut input,
                &root,
                None,
                CreationContext::Ordinary,
            );
            if cap == 14 {
                let node = result.unwrap();
                assert!(input.is_none());
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    14
                );
                assert_eq!(session.active().unwrap().records.len(), 4);
                assert!(
                    session
                        .template_state(&mut c, &node)
                        .unwrap()
                        .target
                        .is_none()
                );
            } else {
                let Err(SessionError::Stop(first)) = result else {
                    panic!("actual initialized slot cap")
                };
                assert!(
                    matches!(first, ParseStop::LimitExceeded{resource: HtmlParseResource::AuxiliaryEntries,limit,..} if limit==cap)
                );
                assert_eq!(input.is_some(), cap == 10);
                assert_eq!(
                    session.active().unwrap().records.len(),
                    if cap == 10 { 1 } else { 2 }
                );
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
                assert_eq!(c.stop(), Some(first));
            }
            if let Some(tag) = input {
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn template_pi_map_coexists_with_original_fields_at_exact_managed_units_and_reclaims_stopped_pair()
     {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Original marker target6 and data name='A'8 remain native-owned;
        // the independently decoded name4/value1 add5 managed units.
        for cap in [18, 19] {
            let source = crate::HtmlSource::utf8("<?marker name='A'?>");
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .managed_utf16_units(cap)
                    .finish()
                    .unwrap(),
                HtmlParseCancellation::new(),
            );
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::ProcessingInstruction { target, data } =
                lexical_kind(&mut lexer, &mut c)
            else {
                panic!("real PI")
            };
            let mut input = Some((target, data));
            let result = session.allocate_processing_instruction(&mut c, &mut input, None);
            assert!(input.is_none());
            assert_eq!(session.active().unwrap().records.len(), 2);
            if cap == 19 {
                let node = result.unwrap();
                assert_eq!(
                    session
                        .active()
                        .unwrap()
                        .pi_attribute(&mut c, node.index, "name")
                        .unwrap()
                        .unwrap()
                        .code_units(),
                    [65]
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    19
                );
            } else {
                let Err(SessionError::Stop(first)) = result else {
                    panic!("decoded value coexistence")
                };
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedUtf16Units,
                        current: 18,
                        requested_delta: 1,
                        ..
                    }
                ));
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
                let HtmlNodeKind::ProcessingInstruction(pi) =
                    &session.active().unwrap().records[1].kind
                else {
                    panic!("original PI survives")
                };
                assert_eq!(pi.target().code_units(), [109, 97, 114, 107, 101, 114]);
                assert_eq!(pi.data().code_units(), [110, 97, 109, 101, 61, 39, 65, 39]);
            }
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    #[test]
    fn template_cross_document_cancel_retains_completed_removal_before_any_adoption_binding() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
            limits(),
        );
        let (mut c, mut session, _proof, script) = fresh_script(&request, false, false);
        let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
        let html = session.active().unwrap().node(&mut c, 1).unwrap();
        session.insert(&mut c, &div, &script, None, None).unwrap();
        session.insert(&mut c, &html, &div, None, None).unwrap();
        let old = session
            .node_document(&mut c, &div)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        let new = session
            .node_document(&mut c, &contents)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        assert_ne!(old.index, new.index);
        let metadata = session.script_metadata(&mut c, &script).unwrap();
        session.active_mut().unwrap().cancel_after_removal = Some(request.cancellation().clone());
        assert_eq!(
            session.insert(&mut c, &contents, &div, None, None),
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        let storage = session.active().unwrap();
        assert_eq!(storage.records[div.index].parent, Some(contents.index));
        assert!(!storage.records[html.index].children.contains(&div.index));
        assert_eq!(storage.records[script.index].parent, Some(div.index));
        assert!(matches!(
            storage.effects.form,
            Some(FormEffects {
                stage: FormEffectStage::Adoption,
                ..
            })
        ));
        for node in [div.index, script.index] {
            assert!(storage.payloads.values.iter().any(|p|matches!(p,RetainedPayload::NodeDocument{node:Some(bound),document} if *bound==node && *document==DocumentRef::Owned(old.index))));
        }
        assert!(storage.payloads.values.iter().any(|p|matches!(p,RetainedPayload::Script{node:Some(node),metadata:retained} if *node==script.index && *retained==metadata)));
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(ParseStop::Cancelled))
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
    }
    #[test]
    fn template_pi_every_bounded_allocation_and_decode_prefix_keeps_original_and_map_tickets_owned()
    {
        use crate::parser::tokenizer::{states::State, tests::LexerFixture};
        // Sweep each credit in this finite operation, rather than deriving a
        // success total from it. Independent witnesses require an installed
        // original PI, a partially decoded map, and the complete literal map.
        let mut installed = false;
        let mut partial = false;
        let mut completed = false;
        // The sole evidence identity, origin/group/cause vectors and decoded
        // field groups extend the original prefix interval. Keep every old
        // credit and sweep the extended finite interval without skipping cuts.
        for remaining in 0..=131_072 {
            let source = crate::HtmlSource::utf8("<?marker name='A' x='B'?>");
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
            let TokenKind::ProcessingInstruction { target, data } =
                lexical_kind(&mut lexer, &mut c)
            else {
                panic!("real lexical PI")
            };
            let mut input = Some((target, data));
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - remaining,
            )
            .unwrap();
            let result = session.allocate_processing_instruction(&mut c, &mut input, None);
            let storage = session.active().unwrap();
            if storage.records.len() == 2 {
                installed = true;
                let HtmlNodeKind::ProcessingInstruction(pi) = &storage.records[1].kind else {
                    panic!("installed PI")
                };
                assert_eq!(
                    pi.target().code_units(),
                    DocumentString::from("marker").code_units()
                );
                assert_eq!(
                    pi.data().code_units(),
                    DocumentString::from("name='A' x='B'").code_units()
                );
                partial |= storage.payloads.values.iter().any(|p| matches!(p, RetainedPayload::PiAttributes { values, ready: false, .. } if values.len() != 0));
            }
            match result {
                Ok(node) => {
                    completed = true;
                    let values = storage
                        .payloads
                        .values
                        .iter()
                        .find_map(|p| match p {
                            RetainedPayload::PiAttributes {
                                node: Some(bound),
                                values,
                                ready: true,
                            } if *bound == node.index => Some(values.as_slice()),
                            _ => None,
                        })
                        .unwrap();
                    assert_eq!(values.len(), 2);
                    assert_eq!(values[0].name.code_units(), [110, 97, 109, 101]);
                    assert_eq!(values[0].value.code_units(), [65]);
                    assert_eq!(values[1].name.code_units(), [120]);
                    assert_eq!(values[1].value.code_units(), [66]);
                }
                Err(SessionError::Stop(first)) => {
                    assert!(matches!(
                        first,
                        ParseStop::LimitExceeded {
                            resource: HtmlParseResource::WorkCredits,
                            ..
                        }
                    ));
                    assert_eq!(
                        session.freeze(&mut c).err(),
                        Some(SessionError::Stop(first))
                    );
                    assert_eq!(c.stop(), Some(first));
                }
                other => panic!("unexpected prefix result {other:?}"),
            }
            let first = c.stop();
            if let Some((target, data)) = input {
                target.release(&mut c).unwrap();
                data.release(&mut c).unwrap();
            }
            lexer.release(&mut c).unwrap();
            match session.release(&mut c) {
                Ok(()) => {}
                Err(SessionError::Stop(stop)) => {
                    assert_eq!(c.stop(), Some(stop));
                    session.release(&mut c).unwrap();
                }
                other => panic!("cleanup failed: {other:?}"),
            }
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                0,
                "remaining={remaining}"
            );
            storage_zero(&c);
            if let Some(first) = first {
                assert_eq!(c.stop(), Some(first));
            }
        }
        assert!(
            installed && partial && completed,
            "installed={installed}, partial={partial}, completed={completed}"
        );
    }
}
