//! Static tabIndex over a canonical node and its reached ordinary relationships.

use super::{DomError, DomNode};
use crate::internal::names::in_html_namespace_with;
use surgeist_html::{values::DocumentString, vocabulary::KnownHtmlElement};
use surgeist_html_semantics::{
    HtmlTabIndexDefault, HtmlTabIndexRecoveryError, recover_tab_index_with,
};

impl DomNode<'_> {
    /// Observe this Element's static tabIndex in the current snapshot view.
    ///
    /// The caller admits the reached names, attribute, integer recovery and
    /// ordinary relations through step. Only an HTML Summary's default can
    /// reach a parent or sibling. This method computes no focus behavior.
    ///
    /// # Errors
    ///
    /// Returns the first lookup or admission failure. A non-Element or an
    /// Element outside the HTML, SVG and MathML namespaces returns
    /// [WrongNodeKind](DomError::WrongNodeKind).
    pub fn tab_index_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<i32, E> {
        step()?;
        let element = self.as_element().ok_or(DomError::WrongNodeKind)?;
        let name = element.name();
        step()?;
        let html = in_html_namespace_with(name, &mut step)?;
        if !html
            && !exact_name_with(
                name.namespace_uri(),
                b"http://www.w3.org/2000/svg",
                &mut step,
            )?
            && !exact_name_with(
                name.namespace_uri(),
                b"http://www.w3.org/1998/Math/MathML",
                &mut step,
            )?
        {
            return Err(DomError::WrongNodeKind.into());
        }

        let attributes = self.attributes().ok_or(DomError::WrongNodeKind)?;
        let key = b"tabindex".map(u16::from);
        if let Some(attribute) = attributes.get_null_namespace_attribute_with(&key, &mut step)? {
            match recover_tab_index_with(attribute.value(), |work| admit_work(work, &mut step))? {
                Ok(value) => return Ok(value),
                Err(HtmlTabIndexRecoveryError::RepresentationExhausted) => {
                    return Err(DomError::RepresentationExhausted.into());
                }
                Err(
                    HtmlTabIndexRecoveryError::InvalidSyntax
                    | HtmlTabIndexRecoveryError::OutOfRange,
                ) => {}
            }
        }

        let local_name = name.local_name().code_units();
        let default = if html {
            HtmlTabIndexDefault::for_html_with(local_name, |work| admit_work(work, &mut step))?
        } else {
            HtmlTabIndexDefault::for_svg_or_mathml_with(local_name, |work| {
                admit_work(work, &mut step)
            })?
        };
        match default {
            HtmlTabIndexDefault::Zero => Ok(0),
            HtmlTabIndexDefault::NegativeOne => Ok(-1),
            HtmlTabIndexDefault::FirstSummary => {
                Ok(if self.is_first_details_summary_with(&mut step)? {
                    0
                } else {
                    -1
                })
            }
        }
    }

    fn is_first_details_summary_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        step()?;
        let Some(parent) = self.parent() else {
            return Ok(false);
        };
        let parent = self.snapshot.node_with(parent, &mut step)?;
        if !html_name_with(parent, KnownHtmlElement::Details, &mut step)? {
            return Ok(false);
        }
        step()?;
        for child in parent.children() {
            step()?;
            let node = self.snapshot.node_with(*child, &mut step)?;
            if html_name_with(node, KnownHtmlElement::Summary, &mut step)? {
                return Ok(node.id() == self.id());
            }
        }
        Ok(false)
    }
}

fn admit_work<E>(work: usize, mut step: impl FnMut() -> Result<(), E>) -> Result<(), E> {
    for _ in 0..work {
        step()?;
    }
    Ok(())
}

fn html_name_with<E>(
    node: DomNode<'_>,
    expected: KnownHtmlElement,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    step()?;
    let Some(element) = node.as_element() else {
        return Ok(false);
    };
    let name = element.name();
    step()?;
    if !in_html_namespace_with(name, &mut step)? {
        return Ok(false);
    }
    exact_name_with(
        Some(name.local_name()),
        expected.local_name().as_bytes(),
        step,
    )
}

fn exact_name_with<E>(
    actual: Option<&DocumentString>,
    expected: &[u8],
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    step()?;
    let Some(actual) = actual.filter(|actual| actual.len() == expected.len()) else {
        return Ok(false);
    };
    for (unit, byte) in actual.code_units().iter().zip(expected) {
        step()?;
        if *unit != u16::from(*byte) {
            return Ok(false);
        }
    }
    Ok(true)
}
