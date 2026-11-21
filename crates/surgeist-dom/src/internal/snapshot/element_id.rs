//! Exact DOM ID lookup over an ordinary tree.

use super::{Descendant, DomError, DomNode, NodeId};
use crate::internal::names::same_code_units;

const ID: &[u16] = &[0x69, 0x64];

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
        mut step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<Self>, E> {
        if id.is_empty() {
            return Ok(None);
        }
        if include_root && self.matches_element_id(id, &mut step)? {
            return Ok(Some(self));
        }
        self.find_descendant_with(step, reserve, |node, step| {
            if node.matches_element_id(id, step)? {
                return Ok(Descendant::Found(node));
            }
            Ok(Descendant::Descend)
        })
    }

    fn matches_element_id<E: From<DomError>>(
        self,
        id: &[u16],
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        let Some(attributes) = self.attributes() else {
            return Ok(false);
        };
        let Some(attribute) = attributes.get_null_namespace_attribute_with(ID, &mut *step)? else {
            return Ok(false);
        };
        same_code_units(attribute.value().code_units(), id, step)
    }
}
