//! Real construction records with controlled tokenizer payload transfer.
//!
//! HTML start-tag names and no-namespace attributes retain exact immutable
//! UTF-16 backing, actual vector/shared tickets, and original token text origins.
//! Character insertion merges preceding native text at an explicitly selected
//! location. Retired merge backing remains owned and fully charged until cleanup.
//! Prologue producers retain comments, recovered PI and doctype fields in the
//! same arena, with controlled implied-html backing and exact whitespace splits.
//! Preexisting immutable element inputs retain their earlier physical exclusion.
//! Foreign-name adjustment, foster-parent selection, full patching/template
//! metadata, form metadata, source reports and public parsing remain later
//! producers; this private checkpoint makes no complete parser claim.
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
}
#[derive(Clone, Copy)]
enum AttributeMergeName {
    Html,
    Body,
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

struct SessionStorage {
    control: ControlIdentity,
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
}
/// Actual backing and exact record-qualified ownership, not allocation proxies.
struct PayloadStorage {
    values: Vec<RetainedPayload>,
    allocation: VectorCapacity<RetainedPayload>,
    preexisting_units: usize,
    preexisting_attributes: usize,
}
enum RetainedPayload {
    String {
        value: DocumentString,
        allocation: DocumentStringAllocation,
        origins: Option<OwnedVec<TextOrigin>>,
        location: StringLocation,
    },
    Attributes(Option<AttributeStorage>),
    Script {
        node: usize,
        metadata: ScriptMetadata,
    },
}
/// Source processing model facts retained in the actual arena owner.
/// Document indices are private and resolved only through qualified handles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMetadata {
    parser_document: Option<usize>,
    preparation_document: Option<usize>,
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
enum StringLocation {
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
                RetainedPayload::Script { .. } => {}
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
                if let Some(origins) = origins {
                    origins
                        .release(control)
                        .expect("actual retained origin storage");
                }
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
            Self::AttributeName(container, index) | Self::AttributeValue(container, index) => {
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
                attribute.map(|attribute| match self {
                    Self::AttributeName(..) => attribute.name().local_name(),
                    _ => attribute.value(),
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
        let (records, records_allocation) = control.new_vector()?;
        let (children_allocations, tickets_allocation) = control.new_vector()?;
        let (values, allocation) = control.new_vector()?;
        let (identity, identity_allocation) = control
            .share_owned(Identity)
            .map_err(|(stop, _)| SessionError::Stop(stop))?;
        let mut session = Self {
            control: control.identity(),
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
        if let Err(error) = session.allocate(control, kind, false, 0, 0) {
            session.release(control);
            return Err(error);
        }
        Ok(session)
    }

    fn require(&self, control: &mut ParseControl) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        Ok(())
    }
    fn node(&self, control: &mut ParseControl, index: usize) -> Result<ParserNode, SessionError> {
        self.require(control)?;
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
        let node = self.allocate(
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
        let count = 1 + usize::from(template);
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
        let handle = self.node(control, index)?;
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
        self.usage.nodes += count;
        self.usage.edges += usize::from(template);
        self.usage.attributes += attributes;
        self.usage.utf16_units += units;
        self.usage.depth = self.usage.depth.max(usize::from(template));
        Ok(handle)
    }

    fn prepare_payload_slot(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        self.require(control)?;
        control.reserve_vector(&mut self.payloads.values, &mut self.payloads.allocation, 1)?;
        control.admit_atomic_layout(mem::size_of::<RetainedPayload>())?;
        control.charge(
            WorkCategory::CopyByte,
            mem::size_of::<RetainedPayload>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        control.acquire(HtmlParseResource::AuxiliaryEntries, 1)?;
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
        if let Err(error) = self.prepare_payload_slot(control) {
            value.release(control)?;
            if let Some(origins) = origins {
                origins.release(control)?;
            }
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
            value, allocation, ..
        } = &mut self.payloads.values[index]
        else {
            unreachable!()
        };
        control.move_document_string_to_managed(allocation, value)?;
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
        let Tag {
            name, attributes, ..
        } = input.take().expect("checked start tag");
        self.allocate_html_payload(control, name.value, Some(name.origins), attributes)
    }
    /// Shared real storage transfer for explicit and source-implied elements.
    /// An implied name has no fabricated token or source-origin vector.
    fn allocate_html_payload(
        &mut self,
        control: &mut ParseControl,
        name: FrozenUtf16,
        origins: Option<OwnedVec<TextOrigin>>,
        mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
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
            let node = self.allocate(
                control,
                HtmlNodeKind::Element(HtmlElement {
                    name: QualifiedName::new(Namespace::from_uri(namespace), None, name),
                    attributes,
                }),
                template,
                0,
                0,
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
            let (merged, merged_ticket) = self.register_string(control, merged, None)?;
            let node = self.node(control, index)?;
            control.charge(
                WorkCategory::CopyByte,
                mem::size_of::<DocumentString>() as u64,
            )?;
            control.charge(WorkCategory::ProviderCall, 1)?;
            control.charge(WorkCategory::RecordWrite, 4)?;
            // Fixed commit after all fallible work. The old and incoming backing
            // remain owned and fully charged, detached from the live native slot.
            self.bind_string(
                old_ticket.expect("registered preceding text"),
                StringLocation::Detached,
            );
            self.bind_string(merged_ticket, StringLocation::Text(index));
            self.records[index].kind = HtmlNodeKind::Text(merged);
            self.usage.utf16_units += added;
            Ok(Some(TextInsertion { node, offset }))
        } else {
            let units = value.len();
            control.charge(WorkCategory::RecordWrite, 2)?;
            let node = self.allocate(control, HtmlNodeKind::Text(value), false, 0, 0)?;
            self.bind_string(ticket, StringLocation::Text(node.index));
            self.usage.utf16_units += units;
            self.insert(control, parent, &node, before)?;
            Ok(Some(TextInsertion { node, offset: 0 }))
        }
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
                other => other,
            };
        }
        Ok(())
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
            if record.owner.is_some() {
                return Ok(true);
            }
            let Some(parent) = record.parent else {
                return Ok(false);
            };
            index = parent;
        }
        Err(SessionError::Cycle)
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
    ) -> Result<(), SessionError> {
        let parent = self.index(control, parent)?;
        let node = self.index(control, node)?;
        let before = before.map(|node| self.index(control, node)).transpose()?;
        self.container(control, parent)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node == 0 {
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
                return Ok(());
            }
        }
        let old_parent = self.records[node].parent;
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
        control.charge(WorkCategory::RecordVisit, 1)?;
        if node == 0 {
            return Err(SessionError::RootMove);
        }
        if self.records[node].owner.is_some() {
            return Err(SessionError::AssociationMove);
        }
        let Some(parent) = self.records[node].parent else {
            return Ok(());
        };
        let depth = self.maximum_depth(control, Some((node, None)))?;
        control.observe_tree_depth(depth as u64)?;
        let (order, allocation) = self.child_order(control, parent, node, None)?;
        let admission = (|| {
            control.charge(WorkCategory::RecordWrite, 4)?;
            control.charge(
                WorkCategory::CopyByte,
                (mem::size_of::<Vec<usize>>() * 2) as u64,
            )?;
            control.commit_child_orders(&self.control, order.len() as u64, 0, 1)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = admission {
            release_order(control, &self.control, order, allocation)?;
            return Err(error);
        }
        let (mut order, allocation) = self.swap_order(parent, order, allocation);
        self.records[node].parent = None;
        self.usage.edges -= 1;
        self.usage.depth = depth;
        order.clear();
        release_vector(control, order, allocation)
    }

    fn prepare_freeze(&mut self, control: &mut ParseControl) -> Result<bool, SessionError> {
        self.require(control)?;
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
        Ok(document)
    }

    /// Called only after the owning transition verifies cleanup identity. This
    /// consumes actual storage; exact private tickets prove the release invariant.
    fn release(self, control: &mut ParseControl) {
        let Self {
            control: owner,
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
    /// The sealed execution owner, not a generic Document constructor, proves
    /// the selected ordinary Disabled environment before any payload transfer.
    pub(crate) fn allocate_disabled_parser_script(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
        capability: &crate::parser::tree_builder::StandaloneCapability<'_>,
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
        let node = self.allocate_html_tag(control, input)?;
        let storage = self.active_mut()?;
        storage.prepare_payload_slot(control)?;
        storage.payloads.values.push(RetainedPayload::Script {
            node: node.index,
            metadata: ScriptMetadata {
                parser_document: Some(0),
                preparation_document: None,
                force_async: false,
                already_started: false,
                script_type: None,
                external_file: false,
                ready_to_be_parser_executed: false,
                delaying_load: false,
                result: ScriptResult::Uninitialized,
                steps_when_result_ready: None,
            },
        });
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
            .map(|index| self.active()?.node(control, index))
            .transpose()
    }
    pub(crate) fn script_preparation_document(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.script_metadata(control, node)?
            .preparation_document
            .map(|index| self.active()?.node(control, index))
            .transpose()
    }
    /// Text-mode EOF marks the current script started; it does not prepare it.
    pub(crate) fn mark_script_eof(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::StandaloneCapability<'_>,
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
        capability: &crate::parser::tree_builder::StandaloneCapability<'_>,
    ) -> Result<DisabledScriptPreparation, SessionError> {
        capability.validate_session(control, self)?;
        let storage = self.active_mut()?;
        let index = storage.index(control, node)?;
        let (slot, mut metadata) = script_metadata(&storage.payloads, control, index)?;
        if metadata.already_started {
            return Ok(DisabledScriptPreparation::AlreadyStarted);
        }
        let saved_document = metadata.parser_document;
        metadata.parser_document = None;
        set_script_metadata(storage, control, slot, metadata)?;
        if saved_document.is_some() {
            control.charge(WorkCategory::RecordVisit, 1)?;
            let HtmlNodeKind::Element(element) = &storage.records[index].kind else {
                return Err(SessionError::ScriptMetadataUnavailable);
            };
            if script_attribute(control, element, "async")?.is_none() {
                metadata.force_async = true;
                set_script_metadata(storage, control, slot, metadata)?;
            }
        }
        let mut empty = true;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let children = storage.records[index].children.len();
        for position in 0..children {
            control.charge(WorkCategory::RecordVisit, 3)?;
            let child = storage.records[index].children[position];
            let record = storage
                .records
                .get(child)
                .ok_or(SessionError::ForeignNode)?;
            if let HtmlNodeKind::Text(text) = &record.kind
                && !text.is_empty()
            {
                empty = false;
                break;
            }
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &storage.records[index].kind else {
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
        for _ in 0..storage.records.len() {
            let Some(current) = ancestor else { break };
            control.charge(WorkCategory::RecordVisit, 1)?;
            let record = storage
                .records
                .get(current)
                .ok_or(SessionError::ForeignNode)?;
            if current == 0 && matches!(record.kind, HtmlNodeKind::Document(_)) {
                connected = true;
                ancestor = None;
                break;
            }
            ancestor = record.parent;
        }
        if ancestor.is_some() {
            return Err(SessionError::Cycle);
        }
        if !connected {
            return Ok(DisabledScriptPreparation::Disconnected);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &storage.records[index].kind else {
            unreachable!()
        };
        let Some(kind) = classify_script(control, element)? else {
            return Ok(DisabledScriptPreparation::UnrecognizedType);
        };
        metadata.script_type = Some(kind);
        set_script_metadata(storage, control, slot, metadata)?;
        if saved_document.is_some() {
            metadata.parser_document = saved_document;
            metadata.force_async = false;
            set_script_metadata(storage, control, slot, metadata)?;
        }
        metadata.already_started = true;
        set_script_metadata(storage, control, slot, metadata)?;
        metadata.preparation_document = Some(0);
        set_script_metadata(storage, control, slot, metadata)?;
        Ok(DisabledScriptPreparation::Disabled(kind))
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
        self.allocate_implied_html(control, ImpliedHtmlName::Html)
    }
    pub(crate) fn allocate_implicit_head(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, ImpliedHtmlName::Head)
    }
    pub(crate) fn allocate_implicit_body(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, ImpliedHtmlName::Body)
    }
    pub(crate) fn allocate_implicit_p(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_implied_html(control, ImpliedHtmlName::P)
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
        local: ImpliedHtmlName,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let attributes = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
        let name = literal_string(
            control,
            match local {
                ImpliedHtmlName::Html => b"html",
                ImpliedHtmlName::Head => b"head",
                ImpliedHtmlName::Body => b"body",
                ImpliedHtmlName::P => b"p",
            },
        )?;
        storage.allocate_html_payload(control, name, None, attributes)
    }
    pub(crate) fn allocate_comment(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Text>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        SessionStorage::check_text(control, input.as_mut().ok_or(SessionError::Consumed)?)?;
        let text = take_payload(input, control)?;
        let (value, ticket) = storage.register_text(control, text)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let units = value.len();
        control.charge(WorkCategory::RecordWrite, 2)?;
        let node = storage.allocate(control, HtmlNodeKind::Comment(value), false, 0, 0)?;
        storage.bind_string(ticket, StringLocation::Comment(node.index));
        storage.usage.utf16_units += units;
        Ok(node)
    }
    pub(crate) fn allocate_processing_instruction(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<(Text, Text)>,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let (target, data) = input.as_mut().ok_or(SessionError::Consumed)?;
        SessionStorage::check_text(control, target)?;
        SessionStorage::check_text(control, data)?;
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
        let node = storage.allocate(
            control,
            HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction { target, data }),
            false,
            0,
            0,
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
    ) -> Result<Option<TextInsertion>, SessionError> {
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
    ) -> Result<(), SessionError> {
        self.active_mut()?.insert(control, parent, node, before)
    }
    pub(crate) fn remove(
        &mut self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        self.active_mut()?.remove(control, node)
    }
    pub(crate) fn freeze(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<FrozenArena, SessionError> {
        let document = self.active_mut()?.prepare_freeze(control)?;
        let SessionStorage {
            control: owner,
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
        let view = if document {
            FrozenView::Document(HtmlDocument { arena })
        } else {
            FrozenView::Fragment(HtmlFragment { arena, root: 0 })
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
            }),
        })
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
        } = self
            .storage
            .take()
            .expect("verified frozen owner retains storage");
        let document = matches!(view, FrozenView::Document(_));
        let arena = match view {
            FrozenView::Document(view) => view.arena,
            FrozenView::Fragment(view) => view.arena,
        };
        let arena = match control.release_shared(arena, arena_allocation) {
            Ok(arena) => arena,
            Err(error) => {
                let (stop, arena, arena_allocation) = error.into_parts();
                let view = if document {
                    FrozenView::Document(HtmlDocument { arena })
                } else {
                    FrozenView::Fragment(HtmlFragment { arena, root: 0 })
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
                });
                return Err(stop.into());
            }
        };
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
            && *owner == node
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
/// ASCII-insensitive exact match of a virtual prefix and exact UTF16 range.
/// The language branch never allocates a synthetic `text/` string.
fn script_match(
    control: &mut ParseControl,
    prefix: &[u8],
    value: &DocumentString,
    start: usize,
    end: usize,
    expected: &[u8],
) -> Result<bool, SessionError> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    if prefix.len() + end - start != expected.len() {
        return Ok(false);
    }
    let mut position = 0;
    while position < expected.len() {
        control.charge(
            WorkCategory::InspectByte,
            if position < prefix.len() { 2 } else { 3 },
        )?;
        let actual = if position < prefix.len() {
            u16::from(prefix[position])
        } else {
            value.code_units()[start + position - prefix.len()]
        };
        let folded = if (0x41..=0x5a).contains(&actual) {
            actual + 0x20
        } else {
            actual
        };
        if folded != u16::from(expected[position].to_ascii_lowercase()) {
            return Ok(false);
        }
        position += 1;
    }
    Ok(true)
}
fn javascript_essence(
    control: &mut ParseControl,
    prefix: &[u8],
    value: &DocumentString,
    start: usize,
    end: usize,
) -> Result<bool, SessionError> {
    // MIME39aa535 §4.6's complete essence list; parameters are never removed.
    const ESSENCES: [&[u8]; 16] = [
        b"application/ecmascript",
        b"application/javascript",
        b"application/x-ecmascript",
        b"application/x-javascript",
        b"text/ecmascript",
        b"text/javascript",
        b"text/javascript1.0",
        b"text/javascript1.1",
        b"text/javascript1.2",
        b"text/javascript1.3",
        b"text/javascript1.4",
        b"text/javascript1.5",
        b"text/jscript",
        b"text/livescript",
        b"text/x-ecmascript",
        b"text/x-javascript",
    ];
    let mut position = 0;
    while position < ESSENCES.len() {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if script_match(control, prefix, value, start, end, ESSENCES[position])? {
            return Ok(true);
        }
        position += 1;
    }
    Ok(false)
}
fn classify_script(
    control: &mut ParseControl,
    element: &HtmlElement,
) -> Result<Option<ScriptType>, SessionError> {
    let explicit = script_attribute(control, element, "type")?;
    let language = script_attribute(control, element, "language")?;
    control.charge(WorkCategory::RecordVisit, 2)?;
    if explicit.is_some_and(DocumentString::is_empty)
        || (explicit.is_none() && language.is_none_or(DocumentString::is_empty))
    {
        return Ok(Some(ScriptType::Classic));
    }
    let (prefix, value): (&[u8], _) = if let Some(value) = explicit {
        (b"", value)
    } else {
        (b"text/", language.expect("nonempty language branch"))
    };
    if javascript_essence(control, prefix, value, 0, value.len())? {
        return Ok(Some(ScriptType::Classic));
    }
    if explicit.is_some() {
        let mut start = 0;
        let mut end = value.len();
        while start < end {
            control.charge(WorkCategory::InspectByte, 2)?;
            if !matches!(value.code_units()[start], 0x09 | 0x0a | 0x0c | 0x0d | 0x20) {
                break;
            }
            start += 1;
        }
        while end > start {
            control.charge(WorkCategory::InspectByte, 2)?;
            if !matches!(
                value.code_units()[end - 1],
                0x09 | 0x0a | 0x0c | 0x0d | 0x20
            ) {
                break;
            }
            end -= 1;
        }
        if javascript_essence(control, prefix, value, start, end)? {
            return Ok(Some(ScriptType::Classic));
        }
    }
    for (literal, kind) in [
        (b"module".as_slice(), ScriptType::Module),
        (b"importmap".as_slice(), ScriptType::ImportMap),
        (b"speculationrules".as_slice(), ScriptType::SpeculationRules),
    ] {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if script_match(control, prefix, value, 0, value.len(), literal)? {
            return Ok(Some(kind));
        }
    }
    Ok(None)
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
    use crate::{Attribute, HtmlParseResource};
    use crate::{HtmlParseCancellation, HtmlParseLimits};

    fn body_lexical_value(source: &crate::HtmlSource, c: &mut ParseControl) -> Text {
        use crate::parser::tokenizer::{Tokenizer, states::State};
        c.admit_source(source).unwrap();
        let mut tokenizer = Tokenizer::new(source, State::Data, None, c).unwrap();
        let (value, tag, attribute) = lexical_attribute_text(&mut tokenizer, c);
        tag.release(c).unwrap();
        attribute.release(c).unwrap();
        tokenizer.release(c).unwrap();
        value
    }
    #[test]
    fn recovered_end_br_discards_actual_attributes_and_keeps_name_origin_and_native_empty_attrs() {
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("</BR x=kept x=ignored>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        session.insert(&mut c, &root, &node, None).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("</br x=kept>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
            let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("<body a=x>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let original = input.as_ref().unwrap().name.value.text().backing().as_ptr();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        let body = session.allocate_implicit_body(&mut c).unwrap();
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
        session.allocate_implicit_p(&mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
            let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
            let body = session.allocate_implicit_body(&mut c).unwrap();
            let p = session.allocate_implicit_p(&mut c).unwrap();
            let root = session.root(&mut c).unwrap();
            session.insert(&mut c, &root, &html, None).unwrap();
            session.insert(&mut c, &html, &body, None).unwrap();
            session.insert(&mut c, &body, &p, None).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State, token::OriginKind};
        let source = crate::HtmlSource::utf8("<ImAgE id=x />");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        session.insert(&mut c, &root, &node, None).unwrap();
        assert!(session.active().unwrap().standalone_ineligibility.is_none());
        tokenizer.release(&mut c).unwrap();
        let mut arena = session.freeze(&mut c).unwrap();
        assert_eq!(locals(arena.fragment().unwrap()), ["img"]);
        arena.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn recovered_image_literal_coexistence_cap_preserves_old_backing_before_commit() {
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
            let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("<image id=x>");
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
            let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("\0");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        for implied in [false, true] {
            let source =
                crate::HtmlSource::utf8("<body id=old><body id=ignored x:id=p a=one a=last z=end>");
            let mut c = control();
            c.admit_source(&source).unwrap();
            let mut t = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
                panic!("tag")
            };
            let body = if implied {
                let body = session.allocate_implicit_body(&mut c).unwrap();
                session
                    .add_body_attributes_if_missing(&mut c, &body, &mut Some(first))
                    .unwrap();
                body
            } else {
                session.allocate_html_tag(&mut c, &mut Some(first)).unwrap()
            };
            let root = session.root(&mut c).unwrap();
            session.insert(&mut c, &root, &body, None).unwrap();
            let p = session.allocate_implicit_p(&mut c).unwrap();
            session.insert(&mut c, &body, &p, None).unwrap();
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

    fn fresh_script_owner<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> (
        ParseControl,
        ParserArenaSession,
        crate::parser::tree_builder::StandaloneCapability<'a>,
    ) {
        use crate::parser::tree_builder::StandaloneDocumentDriver;
        let mut driver = Some(StandaloneDocumentDriver::new(request).unwrap());
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
        crate::parser::tree_builder::StandaloneCapability<'a>,
        ParserNode,
    ) {
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let (mut c, mut session, capability) = fresh_script_owner(request);
        let mut tokenizer = Tokenizer::new(request.source(), State::Data, None, &mut c).unwrap();
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
            .allocate_disabled_parser_script(&mut c, &mut incoming, &capability)
            .unwrap();
        assert!(incoming.is_none());
        if connected {
            let html_index = session.active().unwrap().records[0].children[0];
            let html = session.active().unwrap().node(&mut c, html_index).unwrap();
            session.insert(&mut c, &html, &node, None).unwrap();
        }
        if text {
            let TokenKind::Text(value) = lexical_kind(&mut tokenizer, &mut c) else {
                panic!("literal text")
            };
            session
                .insert_text(&mut c, &node, None, &mut Some(value))
                .unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        (c, session, capability, node)
    }

    fn next_fixture_tag(
        tokenizer: &mut crate::parser::tokenizer::Tokenizer<'_>,
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let mut tokenizer = Tokenizer::new(source, State::Data, None, c).unwrap();
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
        assert_eq!(initial.parser_document, Some(0));
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
                Tokenizer::new(request.source(), State::Data, None, &mut c).unwrap();
            for _ in 0..depth {
                let tag = next_fixture_tag(&mut tokenizer, &mut c, &[0x64, 0x69, 0x76]);
                let child = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
                session.insert(&mut c, &parent, &child, None).unwrap();
                parent = child;
            }
            session.insert(&mut c, &parent, &node, None).unwrap();
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
    fn actual_associated_domain_entry_revokes_only_on_commit_and_never_resets() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
            limits(),
        );
        let (mut c, mut session, capability, script) = fresh_script(&request, false, false);
        let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
        session.insert(&mut c, &div, &script, None).unwrap();
        let initial = session.script_metadata(&mut c, &script).unwrap();
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        let mut foreign_control = control();
        assert!(matches!(
            session.insert(&mut foreign_control, &contents, &div, None),
            Err(SessionError::Stop(_))
        ));
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        let mut foreign_arena =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let foreign_root = foreign_arena.root(&mut c).unwrap();
        assert_eq!(
            session.insert(&mut c, &contents, &foreign_root, None),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session.insert(&mut c, &contents, &div, Some(&script)),
            Err(SessionError::WrongSibling)
        );
        assert_eq!(session.active().unwrap().standalone_ineligibility, None);
        assert_eq!(session.active().unwrap().records[div.index].parent, None);
        foreign_arena.release(&mut c).unwrap();
        // The ordinary div carries the actual script descendant into the real
        // template-owned fragment. No script-name guess or callback flag is used.
        session.insert(&mut c, &contents, &div, None).unwrap();
        assert_eq!(
            session.active().unwrap().standalone_ineligibility,
            Some(StandaloneUnsupported::ScriptDocument)
        );
        assert_eq!(
            session.active().unwrap().records[script.index].parent,
            Some(div.index)
        );
        assert_eq!(session.script_metadata(&mut c, &script).unwrap(), initial);
        session.remove(&mut c, &div).unwrap();
        assert_eq!(session.active().unwrap().records[div.index].parent, None);
        session.allocate_element(&mut c, html("span")).unwrap();
        assert_eq!(
            session.active().unwrap().standalone_ineligibility,
            Some(StandaloneUnsupported::ScriptDocument)
        );
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::ScriptDocument);
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
        session.insert(&mut c, &div, &script, None).unwrap();
        request.cancellation().cancel();
        assert_eq!(
            session.insert(&mut c, &contents, &div, None),
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
            6
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
        session.insert(&mut baseline, &div, &script, None).unwrap();
        session
            .insert(&mut baseline, &contents, &div, None)
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
            session.insert(&mut c, &div, &script, None).unwrap();
            let result = session.insert(&mut c, &contents, &div, None);
            let storage = session.active().unwrap();
            assert_eq!(storage.records[script.index].parent, Some(div.index));
            if cap == exact {
                assert_eq!(result, Ok(()));
                assert_eq!(storage.records[div.index].parent, Some(contents.index));
                assert_eq!(storage.records[contents.index].children, [div.index]);
                assert_eq!(
                    storage.standalone_ineligibility,
                    Some(StandaloneUnsupported::ScriptDocument)
                );
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
                6
            );
        }
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
            assert_eq!(retained.parser_document, Some(0));
            if cap == exact {
                assert_eq!(
                    result,
                    Ok(DisabledScriptPreparation::Disabled(ScriptType::Classic))
                );
                assert_eq!(retained.preparation_document, Some(0));
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        // Three node child-storage tickets, two actual attribute containers,
        // and four name/namespace registry entries consume nine slots. Script
        // metadata is the tenth. No attribute-name trie competes with this cap.
        for cap in [9, 10] {
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
                Tokenizer::new(request.source(), State::Data, None, &mut c).unwrap();
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
            let result =
                session.allocate_disabled_parser_script(&mut c, &mut incoming, &capability);
            assert!(incoming.is_none());
            assert_eq!(session.active().unwrap().records.len(), 3);
            let count = session
                .active()
                .unwrap()
                .payloads
                .values
                .iter()
                .filter(|entry| matches!(entry, RetainedPayload::Script { .. }))
                .count();
            if cap == 10 {
                result.unwrap();
                assert_eq!(count, 1);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    10
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        current: 9,
                        requested_delta: 1,
                        ..
                    }))
                ));
                assert_eq!(count, 0);
                let stop = c.stop().unwrap();
                assert!(
                    matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
                );
            }
            let first = c.stop();
            tokenizer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), first);
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                3
            );
        }
    }

    #[test]
    fn script_allocation_alias_rejection_preserves_the_actual_lexical_owner_and_arena() {
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script src=''>"),
            limits(),
        );
        let (mut c, mut session, capability, _) = fresh_script(&request, true, false);
        let mut tokenizer = Tokenizer::new(request.source(), State::Data, None, &mut c).unwrap();
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
            session.allocate_disabled_parser_script(&mut c, &mut input, &capability),
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
    fn an_actual_template_association_cannot_be_guessed_as_the_ordinary_script_document() {
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><template><script src=''>"),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, false, false);
        let initial = session.script_metadata(&mut c, &node).unwrap();
        let mut tokenizer = Tokenizer::new(request.source(), State::Data, None, &mut c).unwrap();
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
        session.insert(&mut c, &contents, &node, None).unwrap();
        tokenizer.release(&mut c).unwrap();
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::ScriptDocument);
        assert_eq!(
            session.prepare_disabled_script(&mut c, &node, &capability),
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

    fn limits() -> HtmlParseLimits {
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
            let mut t = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
            session.insert(&mut c, &doc, &root, None).unwrap();
            let head = session.allocate_implicit_head(&mut c).unwrap();
            session.insert(&mut c, &root, &head, None).unwrap();
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
            assert!(storage.payloads.values.iter().any(|v| matches!(v, RetainedPayload::String { origins: Some(origins), location: StringLocation::AttributeValue(_, 2), .. } if origins.as_slice()[0].source == expected_origin)));
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        crate::parser::tokenizer::Tokenizer<'source>,
    ) {
        use crate::parser::tokenizer::{Tokenizer, states::State};
        c.admit_source(source).unwrap();
        let mut tokenizer = Tokenizer::new(source, State::Data, None, c).unwrap();
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
        (2 + 3
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
        tokenizer: crate::parser::tokenizer::Tokenizer<'_>,
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
            assert_eq!(origins.as_slice()[0].source.start(), 25);
            assert_eq!(origins.as_slice()[0].source.end(), 26);
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("<html id=x><html a=y>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut t = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
        let mut t = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        for cancel in [false, true] {
            let source = crate::HtmlSource::utf8("<html a=x>");
            let cancellation = crate::HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(), cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut t = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
        let run = |cap: Option<u64>| {
            let selected = match cap {
                Some(cap) => limits().into_builder().work_credits(cap).finish().unwrap(),
                None => limits(),
            };
            let mut c = ParseControl::new(selected, crate::HtmlParseCancellation::new());
            c.admit_source(&source).unwrap();
            let mut t = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
    fn storage_zero(c: &ParseControl) {
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
        tokenizer: &mut crate::parser::tokenizer::Tokenizer<'_>,
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
        tokenizer: &mut crate::parser::tokenizer::Tokenizer<'_>,
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<!--a"
                .encode_utf16()
                .chain([0xd800])
                .chain("--><?x a??><!DOCTYPE h PUBLIC 'p' 's'>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let TokenKind::Comment(text) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("comment");
        };
        let comment = session.allocate_comment(&mut c, &mut Some(text)).unwrap();
        session.insert(&mut c, &root, &comment, None).unwrap();
        assert!(session.document_accepts_doctype(&mut c).unwrap());
        let TokenKind::ProcessingInstruction { target, data } =
            lexical_kind(&mut tokenizer, &mut c)
        else {
            panic!("PI");
        };
        let pi = session
            .allocate_processing_instruction(&mut c, &mut Some((target, data)))
            .unwrap();
        session.insert(&mut c, &root, &pi, None).unwrap();
        assert!(session.document_accepts_doctype(&mut c).unwrap());
        let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("doctype");
        };
        let doctype = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
        session.insert(&mut c, &root, &doctype, None).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
        let source = crate::HtmlSource::utf8("<!DOCTYPE><!DOCTYPE html>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        session.insert(&mut c, &root, &first, None).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State, token::OriginKind};
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<a v='"
                .encode_utf16()
                .chain([0x20, 9, 13, 10, 0xd83d, 0xde00, 0xd800, 0xfeff])
                .chain("'>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
        use crate::parser::tokenizer::{Tokenizer, states::State};
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
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
                .allocate_processing_instruction(&mut c, &mut input)
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
        // Two checked handle reads, element kind, ancestry root, target kind.
        assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 5);
        session.insert(&mut c, &root, &first, None).unwrap();
        let visits = c.usage().work(WorkCategory::RecordVisit);
        assert_eq!(
            session.element_insertion_ignored(&mut c, &second, &root),
            Ok(Some(ElementInsertionIgnored::DocumentAlreadyHasElement))
        );
        // The additional real child-index and referenced-kind reads cost two.
        assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 7);
        assert_eq!(session.active().unwrap().records[0].children, [first.index]);
        assert!(
            session.active().unwrap().records[second.index]
                .parent
                .is_none()
        );
        // The guard is HTML placement policy; generic native moves are unchanged.
        session.insert(&mut c, &root, &second, None).unwrap();
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
        session.insert(&mut c, &contents, &child, None).unwrap();
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
        // Two handles + element kind + child/contents/template ancestors.
        assert_eq!(c.usage().work(WorkCategory::RecordVisit) - visits, 6);
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
        session.insert(&mut c, &root, &first, None).unwrap();
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes);
        let edges = c.usage().resource(HtmlParseResource::ManagedEdges);
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, limits().work_credits() - used - 6)
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
            used + 5
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
            .insert_text(&mut c, &root, None, &mut input)
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
            session.insert_text(&mut c, &root, None, &mut input),
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
            session.insert_text(&mut c, &root, None, &mut input),
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
            Tokenizer,
            states::State,
            token::{Event, TokenKind},
        };
        let source = crate::HtmlSource::utf8("<DiV Z=x z=y A:B='&#x1F600;'>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
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
                            session.insert(&mut c, &root, &node, None).unwrap();
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
            .insert_text(&mut c, &root, None, &mut a)
            .unwrap()
            .unwrap();
        let mut b = Some(owned_text(&mut c, &[0x62]));
        assert!(matches!(
            session.insert_text(&mut c, &root, None, &mut b),
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
        session.insert(&mut c, &root, &new, None).unwrap();
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
        session.insert(&mut c, &root, &element, None).unwrap();
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
            .insert_text(&mut c, &root, None, &mut first)
            .unwrap()
            .unwrap();
        assert_eq!(first.offset, 0);
        let mut next = Some(owned_text(&mut c, &[0xd83d, 0xde00, 0]));
        let next = session
            .insert_text(&mut c, &root, None, &mut next)
            .unwrap()
            .unwrap();
        assert_eq!(next.node.index, first.node.index);
        assert_eq!(next.offset, 2);
        assert_eq!(
            native_text(&session, &first.node),
            [0xd800, 0x20, 0xd83d, 0xde00, 0]
        );
        assert_eq!(session.active().unwrap().usage.utf16_units, 5);
        // Actual retained strings: old2 + incoming3 + merged5; origins remain
        // the exact two original token payloads, not invented merged records.
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
            2
        );
        let mut frozen = session.freeze(&mut c).unwrap();
        assert_eq!(frozen.fragment().unwrap().usage().utf16_units(), 5);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn before_sibling_merges_only_preceding_text_and_document_ignores_text() {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let element = session.allocate_element(&mut c, html("div")).unwrap();
        session.insert(&mut c, &root, &element, None).unwrap();
        let mut a = Some(owned_text(&mut c, &[0x61]));
        let a = session
            .insert_text(&mut c, &root, Some(&element), &mut a)
            .unwrap()
            .unwrap();
        let mut b = Some(owned_text(&mut c, &[0x62]));
        let b = session
            .insert_text(&mut c, &root, Some(&element), &mut b)
            .unwrap()
            .unwrap();
        assert_eq!(a.node.index, b.node.index);
        assert_eq!(native_text(&session, &a.node), [0x61, 0x62]);
        let mut after = Some(owned_text(&mut c, &[0x63]));
        let after = session
            .insert_text(&mut c, &root, None, &mut after)
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
                .insert_text(&mut c, &root, None, &mut text)
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
            session.insert_text(&mut c, &root, None, &mut text),
            Err(SessionError::AliasedPayload)
        ));
        assert!(text.is_some());
        drop(alias);
        let inserted = session
            .insert_text(&mut c, &root, None, &mut text)
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
            .insert_text(&mut c, &root, None, &mut first)
            .unwrap()
            .unwrap();
        let mut second = Some(owned_text(&mut c, &[0x62]));
        let error = session.insert_text(&mut c, &root, None, &mut second);
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
            session.insert(&mut control, &root, node, None).unwrap();
        }
        session.insert(&mut control, &root, &a, Some(&c)).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [b.index, a.index, c.index]
        );
        session.insert(&mut control, &root, &a, Some(&a)).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [b.index, a.index, c.index]
        );
        session.insert(&mut control, &root, &c, Some(&b)).unwrap();
        assert_eq!(
            session.active().unwrap().records[0].children,
            [c.index, b.index, a.index]
        );
        session.remove(&mut control, &b).unwrap();
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
            .insert(&mut control, &root, &template, None)
            .unwrap();
        session
            .insert(&mut control, &contents, &child, None)
            .unwrap();
        assert_eq!(
            session.insert(&mut control, &child, &template, None),
            Err(SessionError::Cycle)
        );
        assert_eq!(
            session.insert(&mut control, &template, &child, None),
            Err(SessionError::TemplateContentsRequired)
        );
        assert_eq!(
            session.insert(&mut control, &root, &contents, None),
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
        session.insert(&mut control, &root, &raw, None).unwrap();
        session.insert(&mut control, &root, &foreign, None).unwrap();
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
            .insert(&mut control, &contents, &child, None)
            .unwrap();
        assert!(matches!(
            session.insert(&mut control, &root, &template, None),
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
            session.insert(&mut owner, &root, &foreign_root, None),
            Err(SessionError::ForeignNode)
        );
        assert_eq!(
            session.insert(&mut owner, &a, &root, None),
            Err(SessionError::RootMove)
        );
        assert_eq!(
            session.insert(&mut owner, &root, &a, Some(&b)),
            Err(SessionError::WrongSibling)
        );
        assert!(session.active().unwrap().records[0].children.is_empty());
        let before = foreign.usage().clone();
        assert!(matches!(
            session.insert(&mut foreign, &root, &a, None),
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
        session.insert(&mut first, &root, &node, None).unwrap();
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
            session.insert(&mut control, &parent, &child, None).unwrap();
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
        session.insert(&mut control, &root, &a, None).unwrap();
        session.insert(&mut control, &root, &b, None).unwrap();
        session.insert(&mut control, &a, &c, None).unwrap();
        session.insert(&mut control, &b, &d, None).unwrap();
        session.insert(&mut control, &b, &c, Some(&d)).unwrap();
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
        session.insert(&mut owner, &root, &node, None).unwrap();
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
        session.insert(&mut control, &root, &a, None).unwrap();
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
            let result = session.insert(&mut control, &root, &child, None);
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
            session.insert(&mut control, &root, a, None).unwrap();
            session.insert(&mut control, &root, b, None).unwrap();
            session.insert(&mut control, a, c, None).unwrap();
            session.insert(&mut control, a, e, None).unwrap();
            session.insert(&mut control, b, d, None).unwrap();
            let capacity_before = control.usage().actual_capacity_bytes;
            let result = session.insert(&mut control, b, c, Some(d));
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
        session.insert(&mut control, &root, &a, None).unwrap();
        let b = session
            .allocate_element(&mut control, element(Namespace::none(), "b"))
            .unwrap();
        session.insert(&mut control, &root, &b, None).unwrap();
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
}
