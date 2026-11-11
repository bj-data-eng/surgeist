//! Where a managed string lives, and the storage bookkeeping for attribute and text values.

use super::*;
use surgeist_html::{
    bounded::construction::HtmlStoredString, construction::ConstructionNodeKind,
    values::DocumentString,
};

/// The identity of one stored string of a record: the field of a node, or of one of its
/// attributes. An attribute field names its element and the attribute's position in the
/// element's list, so it moves with the list when attributes are removed.
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
            Self::Text(index) | Self::Comment(index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                let field = if matches!(self, Self::Text(_)) {
                    HtmlStoredString::Text
                } else {
                    HtmlStoredString::Comment
                };
                records
                    .node_at(index)
                    .and_then(|record| record.stored_string(field))
            }
            Self::ProcessingInstructionTarget(index) | Self::ProcessingInstructionData(index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                let field = if matches!(self, Self::ProcessingInstructionTarget(_)) {
                    HtmlStoredString::ProcessingInstructionTarget
                } else {
                    HtmlStoredString::ProcessingInstructionData
                };
                records
                    .node_at(index)
                    .and_then(|record| record.stored_string(field))
            }
            Self::DoctypeName(index) | Self::DoctypePublic(index) | Self::DoctypeSystem(index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                let field = match self {
                    Self::DoctypeName(_) => HtmlStoredString::DoctypeName,
                    Self::DoctypePublic(_) => HtmlStoredString::DoctypePublicId,
                    _ => HtmlStoredString::DoctypeSystemId,
                };
                records
                    .node_at(index)
                    .and_then(|record| record.stored_string(field))
            }
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
