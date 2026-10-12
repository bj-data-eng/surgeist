//! Exact defining-term selection, HTML 877d
//! <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#defining-term>.
//! This does not certify definition adequacy or the surrounding content model.

use super::*;

/// The actual source branch selecting the exact term; ancestor title is unused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlDefiningTermOrigin {
    OwnTitle,
    AbbreviationTitle(HtmlCaptureSubjectId),
    DescendantText,
}

/// Exact UTF-16 term and selecting evidence for one captured HTML dfn.
/// Empty remains a selected value, without trimming or newline normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDefiningTerm {
    value: DocumentString,
    origin: HtmlDefiningTermOrigin,
}
impl HtmlDefiningTerm {
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    pub fn origin(&self) -> &HtmlDefiningTermOrigin {
        &self.origin
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn dfn_defining_term(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlSemanticValue<HtmlDefiningTerm>, HtmlSemanticFailure> {
        if self.known(element)? != Some(KnownHtmlElement::Dfn) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let title = self.attribute(element, "title")?;
        self.charge(1)?;
        if let Some(title) = title {
            return self.dfn_title(title, HtmlDefiningTermOrigin::OwnTitle);
        }

        let node = self.node(subject)?;
        self.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = node.children() else {
            return Ok(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            ));
        };
        let mut single_element = None;
        let mut eligible = true;
        loop {
            self.charge(1)?;
            let Some(child) = children.next() else { break };
            let child_node = self.node(&child)?;
            self.charge(1)?;
            match child_node.kind() {
                // Even an empty Text node defeats the source's no-Text condition.
                HtmlCapturedNodeKind::Text(_) => {
                    self.charge(1)?;
                    eligible = false;
                    break;
                }
                HtmlCapturedNodeKind::Element(_) if single_element.is_some() => {
                    self.charge(1)?;
                    eligible = false;
                    break;
                }
                HtmlCapturedNodeKind::Element(_) => {
                    self.charge(1)?;
                    single_element = Some(child);
                }
                _ => {}
            }
        }
        if eligible && let Some(child) = single_element {
            let child_node = self.node(&child)?;
            self.charge(1)?;
            let HtmlCapturedNodeKind::Element(abbr) = child_node.kind() else {
                unreachable!("the same immutable child was an Element");
            };
            if self.known(abbr)? == Some(KnownHtmlElement::Abbr) {
                let title = self.attribute(abbr, "title")?;
                self.charge(1)?;
                if let Some(title) = title {
                    return self.dfn_title(title, HtmlDefiningTermOrigin::AbbreviationTitle(child));
                }
            }
        }
        let text = self.control_text_content(subject, HtmlControlTextOrigin::DescendantText)?;
        self.charge(1)?;
        Ok(text.map(|value| HtmlDefiningTerm {
            value,
            origin: HtmlDefiningTermOrigin::DescendantText,
        }))
    }

    fn dfn_title(
        &mut self,
        title: &Attribute,
        origin: HtmlDefiningTermOrigin,
    ) -> Result<HtmlSemanticValue<HtmlDefiningTerm>, HtmlSemanticFailure> {
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(Some(title.value()))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlDefiningTerm {
            value: title.value().clone(),
            origin,
        }))
    }
}
