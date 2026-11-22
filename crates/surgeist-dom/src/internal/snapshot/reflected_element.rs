//! Live association over the recorded ordinary and shadow-including edges.

use super::{DomError, DomNode, NodeId, ascii_units};
use crate::internal::reflected::ReflectedTarget;

const COMMAND_FOR: &[u16] = &ascii_units(b"commandfor");
const POPOVER_TARGET: &[u16] = &ascii_units(b"popovertarget");

impl DomNode<'_> {
    /// Resolve the command-for Element for an actual HTML Button.
    ///
    /// `step` admits reached lookups and comparisons. `reserve` admits
    /// traversal backing. Returns the first native or caller refusal.
    pub fn html_command_for_with<E: From<DomError>, Backing>(
        self,
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<NodeId>, E> {
        self.associated_element_with(ReflectedTarget::CommandFor, step, reserve)
    }

    /// Resolve the reflected Element for an actual HTML Button or Input.
    ///
    /// An explicit weak target uses ordinary target ancestry and
    /// shadow-including invoker ancestry. The content attribute instead
    /// selects the first exact ID in the invoker's ordinary tree.
    ///
    /// `step` admits reached lookups and comparisons. `reserve` admits
    /// traversal backing. Returns the first native or caller refusal.
    pub fn html_popover_target_with<E: From<DomError>, Backing>(
        self,
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<NodeId>, E> {
        self.associated_element_with(ReflectedTarget::PopoverTarget, step, reserve)
    }

    fn associated_element_with<E: From<DomError>, Backing>(
        self,
        attribute: ReflectedTarget,
        mut step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<NodeId>, E> {
        step()?;
        let name = match attribute {
            ReflectedTarget::CommandFor => COMMAND_FOR,
            ReflectedTarget::PopoverTarget => POPOVER_TARGET,
        };
        let attributes = self.attributes().ok_or(DomError::WrongNodeKind)?;
        let content = attributes.get_null_namespace_attribute_with(name, &mut step)?;
        step()?;
        let explicit = self
            .node
            .reflected_targets()
            .and_then(|targets| targets.get(attribute))
            .ok_or(DomError::WrongNodeKind)?;
        if let Some(target) = explicit {
            let target = self.snapshot.node_with(target, &mut step)?;
            return self.scopes_explicit_target(target, &mut step);
        }
        let Some(content) = content.filter(|value| !value.value().code_units().is_empty()) else {
            return Ok(None);
        };
        let root = self.root_with(false, &mut step)?;
        Ok(root
            .element_by_id_with(content.value().code_units(), true, step, reserve)?
            .map(DomNode::id))
    }

    fn scopes_explicit_target<E: From<DomError>>(
        self,
        target: Self,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        let root = target.root_with(false, &mut *step)?;
        if root.id() == target.id() {
            return Ok(None);
        }
        // A shared ordinary ancestor of the target is sufficient exactly
        // when the target's ordinary root is also an ancestor of the invoker.
        let mut current = self;
        while let Some(parent) = current.shadow_including_parent() {
            current = self.snapshot.node_with(parent, &mut *step)?;
            if current.id() == root.id() {
                return Ok(Some(target.id()));
            }
        }
        Ok(None)
    }
}
