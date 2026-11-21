//! HTML-aware Option text over the ordinary published DOM tree.

use super::{Descendant, DomError, DomNode};
use surgeist_html_semantics::HtmlOptionText;

impl DomNode<'_> {
    pub(crate) fn visit_option_text_with<E: From<DomError>, Backing>(
        self,
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
        mut emit: impl FnMut(&[u16]) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut text = HtmlOptionText::default();
        self.find_descendant_with(step, reserve, |node, step| {
            if let Some(element) = node.as_element()
                && HtmlOptionText::prunes_descendants_with(element.name(), &mut *step)?
            {
                return Ok(Descendant::Prune);
            }
            if let Some(data) = node.kind().text_data() {
                for &unit in data.code_units() {
                    step()?;
                    text.push(unit, &mut emit)?;
                }
            }
            Ok(Descendant::<()>::Descend)
        })?;
        Ok(())
    }
}
