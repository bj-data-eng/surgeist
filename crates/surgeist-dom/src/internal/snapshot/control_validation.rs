//! Validation candidacy from the ordinary tree and native control state.

use std::convert::Infallible;

use super::{DomError, DomNode, DomSnapshot, ascii_units};
use crate::internal::button::DomButton;
use surgeist_html::{
    attributes::HtmlInputType,
    bounded::attributes::{HtmlButtonSubmitterFact, button_type_keyword_with},
    vocabulary::KnownHtmlElement,
};
use surgeist_html_semantics::constraint_validation::{
    DatalistAncestor, ValidationCandidacyReader, ValidationControl, validation_candidacy,
};
use surgeist_html_semantics::fieldset_disabledness::{
    HtmlFieldsetDisablednessReader, HtmlFieldsetParent, disabled_by_fieldset,
};

const TYPE: &[u16] = &ascii_units(b"type");
const COMMAND: &[u16] = &ascii_units(b"command");
const COMMAND_FOR: &[u16] = &ascii_units(b"commandfor");
const DISABLED: &[u16] = &ascii_units(b"disabled");
const READONLY: &[u16] = &ascii_units(b"readonly");

impl DomButton<'_> {
    /// Read the canonical Button type keyword using the current native facts.
    /// Only Auto reaches command presence and the ordinary parent.
    ///
    /// # Errors
    ///
    /// Returns the first admission or native lookup failure.
    pub fn type_keyword_with<E: From<DomError>>(
        self,
        step: impl Fn() -> Result<(), E>,
    ) -> Result<&'static str, E> {
        let raw = attribute(self.node, TYPE, &step)?;
        button_type_keyword_with(
            raw,
            |_| step(),
            |fact| {
                let name = match fact {
                    HtmlButtonSubmitterFact::CommandAttributePresent => COMMAND,
                    HtmlButtonSubmitterFact::CommandforAttributePresent => COMMAND_FOR,
                    HtmlButtonSubmitterFact::ParentIsSelect => {
                        return match parent(self.node, &step)? {
                            Some(parent) => {
                                Ok(known(parent, &step)? == Some(KnownHtmlElement::Select))
                            }
                            None => Ok(false),
                        };
                    }
                };
                Ok(attribute(self.node, name, &step)?.is_some())
            },
        )
    }
}

impl DomNode<'_> {
    /// Read whether an actual HTML Button, Input, Select or TextArea is a
    /// candidate for constraint validation at this snapshot.
    ///
    /// `step` admits reached observations on the caller's current Source
    /// segment. Traversal follows ordinary parent edges.
    ///
    /// # Errors
    ///
    /// Returns the first native or caller refusal. Other node kinds yield
    /// `DomError::WrongNodeKind`.
    pub fn html_will_validate_with<E: From<DomError>>(
        self,
        step: impl Fn() -> Result<(), E>,
    ) -> Result<bool, E> {
        let control = known(self, &step)?
            .and_then(ValidationControl::from_known_html)
            .ok_or(DomError::WrongNodeKind)?;
        match validation_candidacy(
            &mut LiveCandidacy {
                node: self,
                step: &step,
            },
            control,
            None,
        )? {
            Ok(candidate) => Ok(candidate),
            Err(impossible) => match impossible {},
        }
    }
}

struct LiveCandidacy<'a, 'step, Step> {
    node: DomNode<'a>,
    step: &'step Step,
}

impl<'a, E: From<DomError>, Step: Fn() -> Result<(), E>> ValidationCandidacyReader
    for LiveCandidacy<'a, '_, Step>
{
    type Node = DomNode<'a>;
    type Missing = Infallible;
    type Error = E;

    fn input_type(&mut self) -> Result<HtmlInputType, E> {
        self.node
            .as_input()
            .ok_or(DomError::WrongNodeKind)?
            .input_type_with(self.step)
    }

    fn readonly(&mut self) -> Result<bool, E> {
        Ok(attribute(self.node, READONLY, self.step)?.is_some())
    }

    fn button_is_submitter(&mut self) -> Result<Result<bool, Infallible>, E> {
        let button = self.node.as_button().ok_or(DomError::WrongNodeKind)?;
        Ok(Ok(button.type_keyword_with(self.step)? == "submit"))
    }

    fn disabled(&mut self, _custom: bool) -> Result<Result<bool, Infallible>, E> {
        if attribute(self.node, DISABLED, self.step)?.is_some() {
            return Ok(Ok(true));
        }
        disabled_by_fieldset(
            &mut FieldsetReader {
                snapshot: self.node.snapshot,
                step: self.step,
            },
            self.node,
        )
    }

    fn ancestry_start(&mut self) -> Result<Self::Node, E> {
        Ok(self.node)
    }

    fn datalist_step(
        &mut self,
        node: Self::Node,
    ) -> Result<DatalistAncestor<Self::Node, Infallible>, E> {
        let Some(parent) = parent(node, self.step)? else {
            return Ok(DatalistAncestor::Root);
        };
        if known(parent, self.step)? == Some(KnownHtmlElement::Datalist) {
            return Ok(DatalistAncestor::Datalist);
        }
        Ok(DatalistAncestor::Parent(parent))
    }
}

fn attribute<'a, E: From<DomError>>(
    node: DomNode<'a>,
    name: &[u16],
    step: &impl Fn() -> Result<(), E>,
) -> Result<Option<&'a [u16]>, E> {
    let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
    Ok(attributes
        .get_null_namespace_attribute_with(name, step)?
        .map(|attribute| attribute.value().code_units()))
}

fn known<E>(
    node: DomNode<'_>,
    step: &impl Fn() -> Result<(), E>,
) -> Result<Option<KnownHtmlElement>, E> {
    step()?;
    Ok(node
        .as_element()
        .and_then(|element| KnownHtmlElement::from_name(element.name())))
}

fn parent<'a, E: From<DomError>>(
    node: DomNode<'a>,
    step: &impl Fn() -> Result<(), E>,
) -> Result<Option<DomNode<'a>>, E> {
    step()?;
    node.parent()
        .map(|parent| node.snapshot.node_with(parent, step))
        .transpose()
}

struct FieldsetReader<'a, 'step, Step> {
    snapshot: &'a DomSnapshot,
    step: &'step Step,
}

impl<'a, E: From<DomError>, Step: Fn() -> Result<(), E>> HtmlFieldsetDisablednessReader
    for FieldsetReader<'a, '_, Step>
{
    type Node = DomNode<'a>;
    type Missing = Infallible;
    type Error = E;

    fn ordinary_parent(
        &mut self,
        node: Self::Node,
    ) -> Result<HtmlFieldsetParent<Self::Node, Infallible>, E> {
        Ok(parent(node, self.step)?.map_or(HtmlFieldsetParent::Root, HtmlFieldsetParent::Parent))
    }

    fn disabled_fieldset(&mut self, node: Self::Node) -> Result<bool, E> {
        Ok(known(node, self.step)? == Some(KnownHtmlElement::Fieldset)
            && attribute(node, DISABLED, self.step)?.is_some())
    }

    fn html_legend(&mut self, node: Self::Node) -> Result<bool, E> {
        Ok(known(node, self.step)? == Some(KnownHtmlElement::Legend))
    }

    fn first_legend_branch(
        &mut self,
        fieldset: Self::Node,
        branch: Self::Node,
    ) -> Result<Result<bool, Infallible>, E> {
        (self.step)()?;
        for child in fieldset.children() {
            let child = self.snapshot.node_with(*child, self.step)?;
            if self.html_legend(child)? {
                return Ok(Ok(child.id() == branch.id()));
            }
        }
        Ok(Ok(false))
    }
}
