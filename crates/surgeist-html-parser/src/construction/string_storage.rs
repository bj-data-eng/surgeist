//! Where a managed string lives, and the storage bookkeeping for attribute and text values.

use super::*;
use surgeist_html::{
    bounded::construction::HtmlStoredString,
    construction::ConstructionNodeKind,
    values::{Attribute, DocumentString, OrderedAttributes},
};

pub(super) enum AttributeStorage {
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

impl StringLocation {
    pub(super) fn value<'a>(
        &self,
        records: ConstructionView<'a>,
        payloads: &'a [RetainedPayload],
        registry_index: usize,
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
            Self::AttributeName(container, index)
            | Self::AttributeValue(container, index)
            | Self::AttributeNamespace(container, index)
            | Self::AttributePrefix(container, index) => {
                cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 2)?;
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
                            cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
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

pub(crate) struct TextInsertion {
    pub(crate) _node: ParserNode,
    pub(crate) _offset: usize,
}

pub(super) struct ReleasedStorage {
    pub(super) construction: ConstructionOwner,
    pub(super) payloads: PayloadStorage,
}

/// The position `index` counts from the end of `original_len` values.
pub(super) fn mirrored_position(original_len: usize, index: usize) -> Option<usize> {
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
