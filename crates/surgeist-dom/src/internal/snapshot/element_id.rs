//! Exact DOM ID lookup over an ordinary tree.

use super::{DescendantSearch, DomError, DomNode, NodeId, ReservedDescendantSearch};
use crate::internal::names::same_code_units;

const ID: &[u16] = &[0x69, 0x64];

#[derive(Clone, Copy)]
pub(crate) struct NonemptyElementId<'a>(&'a [u16]);

impl<'a> NonemptyElementId<'a> {
    pub(crate) fn new(id: &'a [u16]) -> Option<Self> {
        (!id.is_empty()).then_some(Self(id))
    }

    pub(crate) fn matches_with<E: From<DomError>>(
        self,
        node: DomNode<'_>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        let Some(attributes) = node.attributes() else {
            return Ok(false);
        };
        let Some(attribute) = attributes.get_null_namespace_attribute_with(ID, &mut step)? else {
            return Ok(false);
        };
        same_code_units(attribute.value().code_units(), self.0, step)
    }
}

impl DomNode<'_> {
    /// Find the first strict ordinary descendant with the exact nonempty DOM ID.
    ///
    /// `step` admits reached reads and comparisons. `reserve` admits traversal
    /// backing. A live caller qualifies access to the current Source segment.
    ///
    /// # Errors
    ///
    /// Returns the first admission or lookup failure.
    pub fn descendant_element_by_id_with<E: From<DomError>, Backing>(
        self,
        id: &[u16],
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<NodeId>, E> {
        Ok(self
            .element_by_id_with(id, false, step, reserve)?
            .map(DomNode::id))
    }
}

impl<'node> DomNode<'node> {
    pub(super) fn element_by_id_with<E: From<DomError>, Backing>(
        self,
        id: &[u16],
        include_root: bool,
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<Self>, E> {
        let Some(id) = NonemptyElementId::new(id) else {
            return Ok(None);
        };
        self.resolve_element_id_with(
            id,
            include_root,
            step,
            &mut ReservedDescendantSearch(reserve),
        )
    }

    /// Resolve the inclusive root before the walk's first matching descendant.
    pub(super) fn resolve_element_id_with<E: From<DomError>>(
        self,
        id: NonemptyElementId<'_>,
        include_root: bool,
        mut step: impl FnMut() -> Result<(), E>,
        search: &mut impl DescendantSearch<'node, E>,
    ) -> Result<Option<Self>, E> {
        if include_root && id.matches_with(self, &mut step)? {
            return Ok(Some(self));
        }
        search.find(self, step, |node, step| {
            Ok(id.matches_with(node, step)?.then_some(node))
        })
    }
}
