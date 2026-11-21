//! Caller-admitted translation mode over the exact snapshot's ordinary tree.

use super::{DomError, DomSnapshot, NodeId};
use crate::internal::names::in_html_namespace_with;
use surgeist_html::bounded::attributes::translate_processing_with;
use surgeist_html_semantics::HtmlTranslationMode;

impl DomSnapshot {
    /// Observe an Element's HTML translation mode in this revision.
    ///
    /// A decisive HTML attribute supplies the mode. Other Elements follow
    /// their ordinary parent Element. The default at the end of the chain is
    /// enabled. The caller admits each reached observation through `step`,
    /// including its current Source allowance when applicable.
    ///
    /// # Errors
    ///
    /// Returns the first lookup or admission error, or
    /// [`DomError::WrongNodeKind`] for a non-Element subject.
    pub fn html_translation_mode_with<E: From<DomError>>(
        &self,
        id: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlTranslationMode, E> {
        let attribute_name = b"translate".map(u16::from);
        let mut current = self.node_with(id, &mut step)?;
        loop {
            step()?;
            let element = current.as_element().ok_or(DomError::WrongNodeKind)?;
            if in_html_namespace_with(element.name(), &mut step)? {
                let attributes = current.attributes().ok_or(DomError::WrongNodeKind)?;
                let raw =
                    attributes.get_null_namespace_attribute_with(&attribute_name, &mut step)?;
                let raw = raw.map(|attribute| attribute.value().code_units());
                let state = translate_processing_with(raw, |_| step())?;
                if let Some(mode) = HtmlTranslationMode::from_attribute(state) {
                    return Ok(mode);
                }
            }
            step()?;
            let Some(parent) = current.ordinary_parent_element(&mut step)? else {
                return Ok(HtmlTranslationMode::Enabled);
            };
            current = parent;
        }
    }
}
