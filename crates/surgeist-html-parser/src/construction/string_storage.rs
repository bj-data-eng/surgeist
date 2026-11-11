//! Where a managed string lives, and the storage bookkeeping for attribute and text values.

use super::*;
use surgeist_html::{construction::ConstructionNodeKind, values::DocumentString};

/// The identity of one stored string of a record: the field of a node, or of one of its
/// attributes. An attribute field names its element and the attribute's position in the
/// element's list, so it moves with the list when attributes are removed.
///
/// The strings of the non-element fields (text, comment, processing-instruction and doctype) are
/// the record's own, and their origin groups live in [`DetailOrigins`]. The strings of the other
/// fields are held by registry rows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldRef {
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

impl FieldRef {
    /// Whether this is a field of a non-element record, whose string the record itself holds.
    pub(super) fn is_record_string(self) -> bool {
        matches!(
            self,
            Self::Text(_)
                | Self::Comment(_)
                | Self::ProcessingInstructionTarget(_)
                | Self::ProcessingInstructionData(_)
                | Self::DoctypeName(_)
                | Self::DoctypePublic(_)
                | Self::DoctypeSystem(_)
        )
    }

    /// The attribute field this names: its element, its position in the list and which of its
    /// strings it is.
    pub(super) fn attribute(self) -> Option<(usize, usize, AttributeString)> {
        match self {
            Self::AttributeName(node, index) => Some((node, index, AttributeString::Name)),
            Self::AttributeValue(node, index) => Some((node, index, AttributeString::Value)),
            Self::AttributeNamespace(node, index) => {
                Some((node, index, AttributeString::Namespace))
            }
            Self::AttributePrefix(node, index) => Some((node, index, AttributeString::Prefix)),
            _ => None,
        }
    }

    /// The field of the attribute at `index` of `node` that `string` names.
    pub(super) fn of_attribute(node: usize, index: usize, string: AttributeString) -> Self {
        match string {
            AttributeString::Name => Self::AttributeName(node, index),
            AttributeString::Value => Self::AttributeValue(node, index),
            AttributeString::Namespace => Self::AttributeNamespace(node, index),
            AttributeString::Prefix => Self::AttributePrefix(node, index),
        }
    }

    /// The units of the string of a non-element field in the record it belongs to. `None` for a
    /// field of another kind, a record of another kind, or an identifier the doctype lacks.
    pub(super) fn units<'a>(self, records: ConstructionView<'a>) -> Option<&'a [u16]> {
        let node = match self {
            Self::Text(node)
            | Self::Comment(node)
            | Self::ProcessingInstructionTarget(node)
            | Self::ProcessingInstructionData(node)
            | Self::DoctypeName(node)
            | Self::DoctypePublic(node)
            | Self::DoctypeSystem(node) => node,
            _ => return None,
        };
        match (self, records.node_at(node)?.kind()) {
            (Self::Text(_), ConstructionNodeKind::Text(value))
            | (Self::Comment(_), ConstructionNodeKind::Comment(value)) => Some(value.code_units()),
            (
                Self::ProcessingInstructionTarget(_),
                ConstructionNodeKind::ProcessingInstruction(value),
            ) => Some(value.target().code_units()),
            (
                Self::ProcessingInstructionData(_),
                ConstructionNodeKind::ProcessingInstruction(value),
            ) => Some(value.data().code_units()),
            (Self::DoctypeName(_), ConstructionNodeKind::Doctype(value)) => {
                Some(value.name().code_units())
            }
            (Self::DoctypePublic(_), ConstructionNodeKind::Doctype(value)) => {
                value.public_id().map(|id| id.code_units())
            }
            (Self::DoctypeSystem(_), ConstructionNodeKind::Doctype(value)) => {
                value.system_id().map(|id| id.code_units())
            }
            _ => None,
        }
    }

    pub(super) fn value<'a>(
        &self,
        records: ConstructionView<'a>,
        payloads: &'a [RetainedPayload],
        control: &mut ParseControl,
    ) -> Result<Option<&'a DocumentString>, SessionError> {
        let value = match *self {
            Self::PiName(map, index) | Self::PiValue(map, index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 2)?;
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
            // A non-element field has no registry row: its string is the record's.
            Self::Text(_)
            | Self::Comment(_)
            | Self::ProcessingInstructionTarget(_)
            | Self::ProcessingInstructionData(_)
            | Self::DoctypeName(_)
            | Self::DoctypePublic(_)
            | Self::DoctypeSystem(_) => return Err(SessionError::WrongToken),
            Self::ElementName(index) | Self::Namespace(index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                match records.node_at(index).map(|record| record.kind()) {
                    Some(ConstructionNodeKind::Element(value)) => match self {
                        Self::ElementName(..) => Some(value.name().local_name()),
                        _ => value.name().namespace().uri(),
                    },
                    _ => None,
                }
            }
            Self::AttributeName(node, index)
            | Self::AttributeValue(node, index)
            | Self::AttributeNamespace(node, index)
            | Self::AttributePrefix(node, index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 2)?;
                let attribute = match records.node_at(node).map(|record| record.kind()) {
                    Some(ConstructionNodeKind::Element(element)) => element.attributes().get(index),
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

/// Which string of a non-element record a field names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum RecordStringKind {
    Text,
    Comment,
    ProcessingInstructionTarget,
    ProcessingInstructionData,
    DoctypeName,
    DoctypePublic,
    DoctypeSystem,
}

/// A string field of a non-element record, in eight bytes: a leaf names its string by it, so
/// that the many leaves of a history stay small. Record positions are 32-bit (the storage
/// refuses a record past `u32::MAX`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RecordString {
    node: u32,
    kind: RecordStringKind,
}
impl RecordString {
    /// The record string `field` names; `None` for a field that a registry row holds or whose
    /// record position does not fit 32 bits.
    pub(super) fn of(field: FieldRef) -> Option<Self> {
        let (node, kind) = match field {
            FieldRef::Text(node) => (node, RecordStringKind::Text),
            FieldRef::Comment(node) => (node, RecordStringKind::Comment),
            FieldRef::ProcessingInstructionTarget(node) => {
                (node, RecordStringKind::ProcessingInstructionTarget)
            }
            FieldRef::ProcessingInstructionData(node) => {
                (node, RecordStringKind::ProcessingInstructionData)
            }
            FieldRef::DoctypeName(node) => (node, RecordStringKind::DoctypeName),
            FieldRef::DoctypePublic(node) => (node, RecordStringKind::DoctypePublic),
            FieldRef::DoctypeSystem(node) => (node, RecordStringKind::DoctypeSystem),
            _ => return None,
        };
        Some(Self {
            node: u32::try_from(node).ok()?,
            kind,
        })
    }
    /// The field this names.
    pub(super) fn field(self) -> FieldRef {
        let node = self.node as usize;
        match self.kind {
            RecordStringKind::Text => FieldRef::Text(node),
            RecordStringKind::Comment => FieldRef::Comment(node),
            RecordStringKind::ProcessingInstructionTarget => {
                FieldRef::ProcessingInstructionTarget(node)
            }
            RecordStringKind::ProcessingInstructionData => {
                FieldRef::ProcessingInstructionData(node)
            }
            RecordStringKind::DoctypeName => FieldRef::DoctypeName(node),
            RecordStringKind::DoctypePublic => FieldRef::DoctypePublic(node),
            RecordStringKind::DoctypeSystem => FieldRef::DoctypeSystem(node),
        }
    }
}

/// Which string of an attribute a field names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AttributeString {
    Name,
    Value,
    Namespace,
    Prefix,
}

pub(crate) struct TextInsertion {
    pub(crate) _node: ParserNode,
    pub(crate) _offset: usize,
}

pub(super) struct ReleasedStorage {
    pub(super) construction: ConstructionOwner,
    pub(super) payloads: PayloadStorage,
}
