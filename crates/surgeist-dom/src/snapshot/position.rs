//! Pinned DOM position comparison in one canonical snapshot.

use super::{DomNode, DomSnapshot};
use crate::{DomError, NodeId};
use std::cmp::Ordering;

/// The exact source-defined flags returned by a document-position comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDocumentPosition(u16);

impl DomDocumentPosition {
    pub const DISCONNECTED: u16 = 1;
    pub const PRECEDING: u16 = 2;
    pub const FOLLOWING: u16 = 4;
    pub const CONTAINS: u16 = 8;
    pub const CONTAINED_BY: u16 = 16;
    pub const IMPLEMENTATION_SPECIFIC: u16 = 32;

    pub const fn bits(self) -> u16 {
        self.0
    }
}

use DomDocumentPosition as Position;

impl DomNode<'_> {
    /// Compare the other node with this node in the captured ordinary tree.
    /// Canonical lookups and ordinary position-index visits use the supplied
    /// meter. Hosted roots and Attr owners do not become ordinary parents.
    pub fn compare_document_position_with<E: From<DomError>>(
        self,
        other: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<DomDocumentPosition, E> {
        let snapshot = self.snapshot;
        let other = snapshot.node_with(other, &mut step)?;
        if self.id() == other.id() {
            return Ok(Position(0));
        }
        let disconnected = || {
            // The exact live pair has already been qualified to this owner.
            // These coordinates are stable across edits for that pair.
            let precedes =
                (other.id().slot, other.id().generation) < (self.id().slot, self.id().generation);
            Position(
                Position::DISCONNECTED
                    | Position::IMPLEMENTATION_SPECIFIC
                    | if precedes {
                        Position::PRECEDING
                    } else {
                        Position::FOLLOWING
                    },
            )
        };
        let source = PositionOperand::new(other);
        let reference = PositionOperand::new(self);
        let (Some(source_node), Some(reference_node)) = (source.node, reference.node) else {
            return Ok(disconnected());
        };
        if source_node == reference_node {
            let position = match (source.attribute, reference.attribute) {
                (Some(source), Some(reference)) => {
                    let element = snapshot.node_with(source_node, &mut step)?;
                    let attributes = element.attributes().ok_or(DomError::WrongNodeKind)?;
                    let first = attributes
                        .find_with(&mut step, |attribute, _| {
                            let id = attribute.id().node();
                            Ok(id == source || id == reference)
                        })?
                        .ok_or(DomError::NotFound)?;
                    Position::IMPLEMENTATION_SPECIFIC
                        | if first.id().node() == source {
                            Position::PRECEDING
                        } else {
                            Position::FOLLOWING
                        }
                }
                (None, Some(_)) => Position::CONTAINS | Position::PRECEDING,
                (Some(_), None) => Position::CONTAINED_BY | Position::FOLLOWING,
                (None, None) => return Err(DomError::NotFound.into()),
            };
            return Ok(Position(position));
        }
        let (source_root, source_depth) = root_depth(snapshot, source_node, &mut step)?;
        let (reference_root, reference_depth) = root_depth(snapshot, reference_node, &mut step)?;
        if source_root != reference_root {
            return Ok(disconnected());
        }
        let relation = order_with(
            snapshot,
            (source_node, source_depth),
            (reference_node, reference_depth),
            &mut step,
        )?;
        Ok(Position(match relation {
            Relation::Ancestor if source.attribute.is_none() => {
                Position::CONTAINS | Position::PRECEDING
            }
            Relation::Descendant if reference.attribute.is_none() => {
                Position::CONTAINED_BY | Position::FOLLOWING
            }
            Relation::Ancestor | Relation::Preceding => Position::PRECEDING,
            Relation::Descendant | Relation::Following => Position::FOLLOWING,
        }))
    }
}

struct PositionOperand {
    node: Option<NodeId>,
    attribute: Option<NodeId>,
}

impl PositionOperand {
    fn new(node: DomNode<'_>) -> Self {
        match node.as_attr() {
            Some(attribute) => Self {
                node: attribute.owner_element(),
                attribute: Some(node.id()),
            },
            None => Self {
                node: Some(node.id()),
                attribute: None,
            },
        }
    }
}

fn root_depth<E: From<DomError>>(
    snapshot: &DomSnapshot,
    mut node: NodeId,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<(NodeId, usize), E> {
    let mut depth = 0_usize;
    while let Some((parent, _)) = snapshot.state.ordinary_child_position_with(node, step)? {
        node = parent;
        depth = depth
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
    }
    Ok((node, depth))
}

enum Relation {
    Ancestor,
    Descendant,
    Preceding,
    Following,
}

fn order_with<E: From<DomError>>(
    snapshot: &DomSnapshot,
    (mut source, source_depth): (NodeId, usize),
    (mut reference, reference_depth): (NodeId, usize),
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<Relation, E> {
    let mut parent = |node| -> Result<_, E> {
        snapshot
            .state
            .ordinary_child_position_with(node, &mut *step)?
            .ok_or_else(|| DomError::NotFound.into())
    };
    for _ in reference_depth..source_depth {
        source = parent(source)?.0;
        if source == reference {
            return Ok(Relation::Descendant);
        }
    }
    for _ in source_depth..reference_depth {
        reference = parent(reference)?.0;
        if reference == source {
            return Ok(Relation::Ancestor);
        }
    }
    loop {
        let (source_parent, source_index) = parent(source)?;
        let (reference_parent, reference_index) = parent(reference)?;
        if source_parent == reference_parent {
            return match source_index.cmp(&reference_index) {
                Ordering::Less => Ok(Relation::Preceding),
                Ordering::Greater => Ok(Relation::Following),
                Ordering::Equal => Err(DomError::NotFound.into()),
            };
        }
        source = source_parent;
        reference = reference_parent;
    }
}
