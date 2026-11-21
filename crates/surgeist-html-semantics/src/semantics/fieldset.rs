//! Preserve ordered captured observations while sharing the Fieldset rule.

use super::*;
use crate::fieldset_disabledness::{
    HtmlFieldsetDisablednessReader, HtmlFieldsetParent, disabled_by_fieldset,
};

pub(super) fn disabled_above<'a>(
    session: &mut HtmlSemanticSession<'a>,
    node: HtmlCapturedNode<'a>,
) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
    Ok(match disabled_by_fieldset(&mut Reader(session), node)? {
        Ok(disabled) => HtmlSemanticValue::Known(disabled),
        Err(missing) => HtmlSemanticValue::Missing(missing),
    })
}

struct Reader<'session, 'capture>(&'session mut HtmlSemanticSession<'capture>);

impl<'a> HtmlFieldsetDisablednessReader for Reader<'_, 'a> {
    type Node = HtmlCapturedNode<'a>;
    type Missing = HtmlSemanticMissing;
    type Error = HtmlSemanticFailure;

    fn ordinary_parent(
        &mut self,
        node: Self::Node,
    ) -> Result<HtmlFieldsetParent<Self::Node, Self::Missing>, Self::Error> {
        Ok(match self.0.parent(node)? {
            Ok(Some(id)) => HtmlFieldsetParent::Parent(self.0.node(&id)?),
            Ok(None) => HtmlFieldsetParent::Root,
            Err(missing) => HtmlFieldsetParent::Missing(missing),
        })
    }

    fn disabled_fieldset(&mut self, node: Self::Node) -> Result<bool, Self::Error> {
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(false);
        };
        Ok(self.0.known(element)? == Some(KnownHtmlElement::Fieldset)
            && self.0.disabled_attribute(element, false)?)
    }

    fn html_legend(&mut self, node: Self::Node) -> Result<bool, Self::Error> {
        let branch = self.0.node(&node.id())?;
        Ok(match branch.kind() {
            HtmlCapturedNodeKind::Element(element) => {
                self.0.known(element)? == Some(KnownHtmlElement::Legend)
            }
            _ => false,
        })
    }

    fn first_legend_branch(
        &mut self,
        fieldset: Self::Node,
        branch: Self::Node,
    ) -> Result<Result<bool, Self::Missing>, Self::Error> {
        self.0.charge(1)?;
        let HtmlCapturedChildren::Complete(mut children) = fieldset.children() else {
            return Ok(Err(HtmlSemanticMissing::OrderedChildren(fieldset.id())));
        };
        loop {
            self.0.charge(1)?;
            let Some(child) = children.next() else {
                return Ok(Ok(false));
            };
            let node = self.0.node(&child)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.0.known(element)? == Some(KnownHtmlElement::Legend)
            {
                return Ok(Ok(child == branch.id()));
            }
        }
    }
}
