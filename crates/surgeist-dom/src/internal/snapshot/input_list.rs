//! HTML Input suggestions-source observation in an admitted snapshot.

use super::{DomError, DomNode, NodeId, ascii_units};
use crate::internal::{input::DomInput, names::in_html_namespace_with};
use surgeist_html::vocabulary::KnownHtmlElement;
use surgeist_html_semantics::input_suggestions::input_suggestions_source;

const LIST: &[u16] = &ascii_units(b"list");

impl DomInput<'_> {
    /// Read this Input's current suggestions source in its ordinary tree.
    ///
    /// The literal `list` attribute selects the first exact ID, including
    /// the root. The result exists only if that element is an HTML Datalist.
    /// A live caller qualifies its access to the current published segment.
    ///
    /// `step` admits reached reads and comparisons. `reserve` admits reached
    /// traversal allocations while their predecessors remain retained.
    ///
    /// # Errors
    ///
    /// Returns the first reached admission or native lookup failure.
    pub fn suggestions_source_with<E: From<DomError>, Backing>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<Option<NodeId>, E> {
        let attributes = self.node.attributes().ok_or(DomError::WrongNodeKind)?;
        let list = attributes
            .get_null_namespace_attribute_with(LIST, &mut step)?
            .map(|attribute| attribute.value());
        Ok(input_suggestions_source(
            &mut step,
            list,
            |step| self.input_type_with(step),
            |step, list| {
                self.node.root_with(false, &mut *step)?.element_by_id_with(
                    list.code_units(),
                    true,
                    step,
                    reserve,
                )
            },
            |step, candidate| {
                step()?;
                let element = candidate.as_element().ok_or(DomError::WrongNodeKind)?;
                let name = element.name();
                if !in_html_namespace_with(name, &mut *step)? {
                    return Ok(None);
                }
                step()?;
                Ok(KnownHtmlElement::from_units(name.local_name().code_units()))
            },
        )?
        .map(DomNode::id))
    }
}
