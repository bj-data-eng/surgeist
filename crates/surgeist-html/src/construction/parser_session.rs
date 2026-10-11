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
//! metadata, form/script metadata, source reports and public parsing remain later
//! producers; this private checkpoint makes no complete parser claim.

use super::*;
use crate::parser::control::{
    ControlIdentity, DocumentStringAllocation, OwnedVec, ParseControl, ParseStop, SharedAllocation,
    VectorCapacity, WorkCategory,
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

pub(crate) struct ParserArenaSession {
    storage: Option<SessionStorage>,
}

struct SessionStorage {
    control: ControlIdentity,
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
}
enum AttributeStorage {
    Building {
        values: Vec<Attribute>,
        allocation: VectorCapacity<Attribute>,
        original_len: usize,
        reversed_pairs: usize,
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
                RetainedPayload::Attributes(Some(AttributeStorage::Building { .. })) => {}
                RetainedPayload::Attributes(None) => return Err(SessionError::Consumed),
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
                    AttributeStorage::Building { values, .. } => values.len(),
                    AttributeStorage::Frozen { values, .. } => values.len(),
                };
                match storage {
                    AttributeStorage::Building {
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
    pub(crate) fn allocate_implicit_html(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<ParserNode, SessionError> {
        let storage = self.active_mut()?;
        storage.require(control)?;
        let attributes = OwnedVec::new(HtmlParseResource::TokenAttributes)?;
        let name = literal_string(control, b"html")?;
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
