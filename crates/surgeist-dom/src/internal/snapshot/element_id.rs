//! Exact DOM ID lookup over an ordinary tree.

use super::{Descendant, DomError, DomNode, NodeId};
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
        self.resolve_element_id_with(id, include_root, step, |step| {
            self.find_descendant_with(step, reserve, |node, step| {
                if id.matches_with(node, step)? {
                    return Ok(Descendant::Found(node));
                }
                Ok(Descendant::Descend)
            })
        })
    }

    /// Resolve the inclusive root before the walk's first matching descendant.
    /// The walk chooses the traversal backing and uses the same ID matcher.
    pub(crate) fn resolve_element_id_with<E, Step>(
        self,
        id: NonemptyElementId<'_>,
        include_root: bool,
        mut step: Step,
        first_in_descendants: impl FnOnce(Step) -> Result<Option<Self>, E>,
    ) -> Result<Option<Self>, E>
    where
        E: From<DomError>,
        Step: FnMut() -> Result<(), E>,
    {
        if include_root && id.matches_with(self, &mut step)? {
            return Ok(Some(self));
        }
        first_in_descendants(step)
    }
}
