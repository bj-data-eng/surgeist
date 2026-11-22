//! Intrinsic validation candidacy over qualified control and ancestry facts.

use surgeist_html::{
    attributes::HtmlInputType, bounded::attributes::input_readonly_applies,
    vocabulary::KnownHtmlElement,
};

/// A submittable control that can be a candidate for constraint validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationControl {
    Button,
    Input,
    Select,
    Textarea,
    FormAssociatedCustom,
}

impl ValidationControl {
    /// Select the built-in family whose candidate result depends on live facts.
    pub fn from_known_html(element: KnownHtmlElement) -> Option<Self> {
        match element {
            KnownHtmlElement::Button => Some(Self::Button),
            KnownHtmlElement::Input => Some(Self::Input),
            KnownHtmlElement::Select => Some(Self::Select),
            KnownHtmlElement::Textarea => Some(Self::Textarea),
            _ => None,
        }
    }
}

/// One reached ordinary-ancestor observation.
#[derive(Debug)]
pub enum DatalistAncestor<Node, Missing> {
    /// The current position has no ordinary parent.
    Root,
    /// Continue from this qualified parent.
    Parent(Node),
    /// A reached ordinary parent is an HTML Datalist.
    Datalist,
    /// The provider lacks a required fact.
    Missing(Missing),
}

/// Admitted facts for one control. Complete live sources can use
/// `Infallible` for missing facts. A refusal propagates immediately.
pub trait ValidationCandidacyReader {
    /// A position in the provider's ordinary topology.
    type Node;
    /// An unavailable fact in a partial representation.
    type Missing;
    /// The provider's first refusal or lookup failure.
    type Error;

    /// Read the qualified Input type state.
    fn input_type(&mut self) -> Result<HtmlInputType, Self::Error>;
    /// Observe the exact readonly attribute's presence.
    fn readonly(&mut self) -> Result<bool, Self::Error>;
    /// Classify the Button's current submitter role.
    fn button_is_submitter(&mut self) -> Result<Result<bool, Self::Missing>, Self::Error>;
    /// Observe effective disabledness, including the Fieldset rule.
    fn disabled(&mut self, custom: bool) -> Result<Result<bool, Self::Missing>, Self::Error>;
    /// Qualify the control's starting position.
    fn ancestry_start(&mut self) -> Result<Self::Node, Self::Error>;
    /// Advance one ordinary ancestor and classify Datalist membership.
    fn datalist_step(
        &mut self,
        node: Self::Node,
    ) -> Result<DatalistAncestor<Self::Node, Self::Missing>, Self::Error>;
}

/// Evaluate the reached candidacy facts. A bar proves false despite an
/// earlier missing fact. Otherwise preserve the first missing fact.
///
/// An unresolved custom classification is supplied as `missing` with the
/// potential form-associated custom control kind.
///
/// # Errors
///
/// Returns the first provider failure unchanged.
pub fn validation_candidacy<R: ValidationCandidacyReader>(
    reader: &mut R,
    control: ValidationControl,
    mut missing: Option<R::Missing>,
) -> Result<Result<bool, R::Missing>, R::Error> {
    match control {
        ValidationControl::Input => {
            let kind = reader.input_type()?;
            if matches!(
                kind,
                HtmlInputType::Hidden | HtmlInputType::Reset | HtmlInputType::Button
            ) || (input_readonly_applies(kind) && reader.readonly()?)
            {
                return Ok(Ok(false));
            }
        }
        ValidationControl::Button => match reader.button_is_submitter()? {
            Ok(false) => return Ok(Ok(false)),
            Ok(true) => {}
            Err(fact) => {
                missing.get_or_insert(fact);
            }
        },
        ValidationControl::Textarea | ValidationControl::FormAssociatedCustom
            if reader.readonly()? =>
        {
            return Ok(Ok(false));
        }
        _ => {}
    }
    match reader.disabled(control == ValidationControl::FormAssociatedCustom)? {
        Ok(true) => return Ok(Ok(false)),
        Ok(false) => {}
        Err(fact) => {
            missing.get_or_insert(fact);
        }
    }
    let mut current = reader.ancestry_start()?;
    loop {
        match reader.datalist_step(current)? {
            DatalistAncestor::Root => return Ok(missing.map_or(Ok(true), Err)),
            DatalistAncestor::Parent(parent) => current = parent,
            DatalistAncestor::Datalist => return Ok(Ok(false)),
            DatalistAncestor::Missing(fact) => return Ok(Err(missing.unwrap_or(fact))),
        }
    }
}
