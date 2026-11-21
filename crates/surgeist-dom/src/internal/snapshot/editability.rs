//! Caller-admitted editability reads over the exact snapshot's ordinary tree.

use super::{DomError, DomNodeKind, DomSnapshot, NodeId};
use crate::internal::names::in_html_namespace_with;
use surgeist_html::bounded::attributes::contenteditable_processing_with;
use surgeist_html_semantics::{HtmlEditableAttribute, html_foreign_root_inherits_editability_with};

impl DomSnapshot {
    /// Observe whether the given HTML Element is an editing host or editable
    /// in this revision. The read follows ordinary parents and uses a direct
    /// parent Document's actual design mode. It allocates no traversal storage.
    ///
    /// The caller admits reached node, attribute and name observations through
    /// `step`, including its current Source allowance when applicable.
    ///
    /// # Errors
    ///
    /// Returns the first lookup or admission failure, or
    /// [`DomError::WrongNodeKind`] for a subject outside the HTML Element interface.
    pub fn html_is_content_editable_with<E: From<DomError>>(
        &self,
        id: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let attribute_name = b"contenteditable".map(u16::from);
        let mut current = self.node_with(id, &mut step)?;
        loop {
            step()?;
            let element = current.as_element().ok_or(DomError::WrongNodeKind)?;
            let html = in_html_namespace_with(element.name(), &mut step)?;
            if !html && current.id() == id {
                return Err(DomError::WrongNodeKind.into());
            }
            let inherits = if html {
                let attributes = current.attributes().ok_or(DomError::WrongNodeKind)?;
                let raw =
                    attributes.get_null_namespace_attribute_with(&attribute_name, &mut step)?;
                let raw = raw.map(|attribute| attribute.value().code_units());
                match contenteditable_processing_with(raw, |_| step())?.into() {
                    HtmlEditableAttribute::ExplicitHost => return Ok(true),
                    HtmlEditableAttribute::Inherit => true,
                    HtmlEditableAttribute::LocalFalse => false,
                }
            } else {
                if !html_foreign_root_inherits_editability_with(element.name(), &mut step)? {
                    return Ok(false);
                }
                true
            };

            step()?;
            let Some(parent) = current.parent() else {
                return Ok(false);
            };
            let parent = self.node_with(parent, &mut step)?;
            match parent.kind() {
                DomNodeKind::Document(metadata) if html => {
                    step()?;
                    return Ok(metadata.design_mode_enabled());
                }
                DomNodeKind::Element(_) if inherits => current = parent,
                _ => return Ok(false),
            }
        }
    }
}
