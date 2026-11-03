//! HTML 877d local media presence/content and meta placement; no CSS parser.
use super::*;
pub(crate) mod controlled;
use crate::{
    internal::elements::media::HtmlMediaCursorAction, internal::elements::media::HtmlMediaValue,
    internal::elements::media::HtmlMediaValueCursor,
    internal::elements::media::HtmlMetaMediaPlacement,
};

/// The kind of HTML element whose `media` attribute is interpreted.
///
/// Only these four elements carry a `media` attribute. Applicability checks a foreign element
/// namespace first (`ForeignElementNamespace`), then a different attribute
/// (`DifferentAttribute`); any other element is then `DifferentElement` and has no role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaElementRole {
    /// A `link` element.
    Link,
    /// A `style` element.
    Style,
    /// A `source` element.
    Source,
    /// A `meta` element, whose `media` attribute is only allowed when its
    /// `name` is `theme-color` (HTML877d #the-meta-element).
    Meta,
}
impl HtmlMediaElementRole {
    pub(crate) fn from_known(kind: Option<KnownHtmlElement>) -> Option<Self> {
        match kind {
            Some(KnownHtmlElement::Link) => Some(Self::Link),
            Some(KnownHtmlElement::Style) => Some(Self::Style),
            Some(KnownHtmlElement::Source) => Some(Self::Source),
            Some(KnownHtmlElement::Meta) => Some(Self::Meta),
            _ => None,
        }
    }
}
/// Whether a `meta` element's `media` attribute is allowed by its `name`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaPlacementObservation {
    /// The placement rule was not selected: the attribute rule is unapplied or
    /// the element is not a `meta`.
    Unapplied,
    /// The actual meta name field was not supplied.
    Unchecked,
    /// The placement has a verdict. When `media` is absent the verdict is `NotSpecified` and no
    /// `name` is read; otherwise the `media` attribute was checked against the supplied `name`
    /// field.
    Known(HtmlMetaMediaPlacement),
}
/// Exact original fields and independent HTML observations. CSS is unchecked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMediaAttributeReport {
    raw: Option<Attribute>,
    name: Option<Attribute>,
    applicability: HtmlAttributeApplicability,
    role: Option<HtmlMediaElementRole>,
    value: HtmlMediaValue,
    placement: HtmlMediaPlacementObservation,
}
impl HtmlMediaAttributeReport {
    /// The exact supplied `media` attribute, or `None` when it is absent.
    pub fn raw(&self) -> Option<&Attribute> {
        self.raw.as_ref()
    }
    /// The exact supplied `name` attribute of a `meta` element. `None` when no
    /// name was supplied or the name is established absent.
    pub fn name_attribute(&self) -> Option<&Attribute> {
        self.name.as_ref()
    }
    /// Whether the rule applies to these expanded names and this context. A
    /// foreign namespace, another attribute or element, or a name supplied for
    /// a non-`meta` element makes it not applicable.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.applicability
    }
    /// The element role, or `None` when the element is none of the four that
    /// carry `media`.
    pub fn role(&self) -> Option<HtmlMediaElementRole> {
        self.role
    }
    /// The presence and content classification of the raw value. This is not a
    /// CSS media query parse.
    pub fn value(&self) -> HtmlMediaValue {
        self.value
    }
    /// Whether a `meta` element's `media` attribute is allowed by its `name`.
    pub fn meta_placement(&self) -> HtmlMediaPlacementObservation {
        self.placement
    }
}

impl HtmlValueInterpreter {
    /// Ordinary per-value admission and allocation. CSS grammar/environment and
    /// actual link relationships are unproved. Original raw admission comes first.
    pub fn media(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlMediaAttributeReport, HtmlValueAdmissionError> {
        self.media_report(element, raw, None, false)
    }
    /// Adds the actual no-namespace name field for meta placement. None means
    /// established absence; a wrong expanded attribute is DifferentContext.
    pub fn media_for_meta<A: HtmlAttributeInput>(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&A>,
        name: Option<&A>,
    ) -> Result<HtmlMediaAttributeReport, HtmlValueAdmissionError> {
        self.media_report(element, raw, name, true)
    }
    fn media_report<A: HtmlAttributeInput>(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&A>,
        name: Option<&A>,
        name_supplied: bool,
    ) -> Result<HtmlMediaAttributeReport, HtmlValueAdmissionError> {
        use controlled::{HtmlMediaInterpretation, HtmlMediaInterpretationError};
        let mut owner = Some(HtmlMediaInterpretation::ordinary(element, raw));
        let local = owner.as_mut().expect("retained owner");
        if name_supplied {
            local
                .supply_ordinary_name(name)
                .expect("initial name supply");
        }
        local
            .admit_media(|raw| self.admit_attribute_raw(raw.map(HtmlAttributeInput::value)))
            .map_err(|error| match error {
                controlled::HtmlMediaActionError::Admission(error) => error,
                _ => unreachable!("ordinary media phase"),
            })?;
        if name_supplied {
            local
                .admit_name(|name| self.admit_attribute_raw(name.map(HtmlAttributeInput::value)))
                .map_err(|error| match error {
                    controlled::HtmlMediaActionError::Admission(error) => error,
                    _ => unreachable!("ordinary name phase"),
                })?;
        } else {
            self.admit_attribute_raw(None)?;
        }
        local
            .inspect_namespace(|_| Ok::<_, std::convert::Infallible>(()))
            .expect("ordinary namespace");
        local
            .derive_role(|_| Ok::<_, std::convert::Infallible>(()))
            .expect("ordinary role");
        local.begin_value().expect("ordinary value phase");
        while local.value_action() != Ok(HtmlMediaCursorAction::Complete) {
            local
                .advance_value(|_| Ok::<_, std::convert::Infallible>(()))
                .expect("ordinary value");
        }
        if local.applicability() == HtmlAttributeApplicability::Applicable
            && local.role() == Ok(Some(HtmlMediaElementRole::Meta))
            && raw.is_some()
            && name_supplied
        {
            local
                .begin_theme_color(|_| Ok::<_, std::convert::Infallible>(()))
                .expect("ordinary theme start");
            while local.theme_color_complete() != Ok(true) {
                local
                    .advance_theme_color(|_| Ok::<_, std::convert::Infallible>(()))
                    .expect("ordinary theme");
            }
        }
        local
            .assemble(|| Ok::<_, std::convert::Infallible>(()))
            .expect("ordinary report assembly");
        Ok(
            HtmlMediaInterpretation::try_finish(&mut owner).unwrap_or_else(
                |_: HtmlMediaInterpretationError| unreachable!("assembled media report"),
            ),
        )
    }
}
