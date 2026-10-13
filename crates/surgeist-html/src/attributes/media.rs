//! HTML 877d local media presence/content and meta placement; no CSS parser.
use super::*;
use crate::{HtmlMediaCursorAction, HtmlMediaValue, HtmlMediaValueCursor, HtmlMetaMediaPlacement};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaElementRole {
    Link,
    Style,
    Source,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaPlacementObservation {
    Unapplied,
    /// The actual meta name field was not supplied.
    Unchecked,
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
    pub fn raw(&self) -> Option<&Attribute> {
        self.raw.as_ref()
    }
    pub fn name_attribute(&self) -> Option<&Attribute> {
        self.name.as_ref()
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.applicability
    }
    pub fn role(&self) -> Option<HtmlMediaElementRole> {
        self.role
    }
    pub fn value(&self) -> HtmlMediaValue {
        self.value
    }
    pub fn meta_placement(&self) -> HtmlMediaPlacementObservation {
        self.placement
    }
    pub(crate) fn observed(
        raw: Option<&Attribute>,
        name: Option<&Attribute>,
        role: HtmlMediaElementRole,
        value: HtmlMediaValue,
        placement: HtmlMediaPlacementObservation,
    ) -> Self {
        Self {
            raw: raw.cloned(),
            name: name.cloned(),
            applicability: HtmlAttributeApplicability::Applicable,
            role: Some(role),
            value,
            placement,
        }
    }
}

pub(crate) fn media_field(attribute: &Attribute, local: &[u16]) -> bool {
    attribute.name().namespace() == &Namespace::none()
        && attribute.name().local_name().code_units() == local
}

impl HtmlValueInterpreter {
    /// Ordinary per-value admission and allocation. CSS grammar/environment and
    /// actual link relationships are unproved. Original raw admission comes first.
    pub fn media(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlMediaAttributeReport, HtmlValueAdmissionError> {
        self.media_report(element, raw, None, false)
    }
    /// Adds the actual no-namespace name field for meta placement. None means
    /// established absence; a wrong expanded attribute is DifferentContext.
    pub fn media_for_meta(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        name: Option<&Attribute>,
    ) -> Result<HtmlMediaAttributeReport, HtmlValueAdmissionError> {
        self.media_report(element, raw, name, true)
    }
    fn media_report(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        name: Option<&Attribute>,
        name_supplied: bool,
    ) -> Result<HtmlMediaAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        self.admit_attribute_raw(name.map(Attribute::value))?;
        let role = HtmlMediaElementRole::from_known(KnownHtmlElement::from_name(element));
        let applicability = if element.namespace() != &Namespace::html() {
            HtmlAttributeApplicability::ForeignElementNamespace
        } else if raw.is_some_and(|a| !media_field(a, &[109, 101, 100, 105, 97])) {
            HtmlAttributeApplicability::DifferentAttribute
        } else if role.is_none() {
            HtmlAttributeApplicability::DifferentElement
        } else if name_supplied
            && (role != Some(HtmlMediaElementRole::Meta)
                || name.is_some_and(|a| !media_field(a, &[110, 97, 109, 101])))
        {
            HtmlAttributeApplicability::DifferentContext
        } else {
            HtmlAttributeApplicability::Applicable
        };
        let mut cursor = HtmlMediaValueCursor::new(raw.map(|a| a.value().code_units()));
        while cursor.next_action() != HtmlMediaCursorAction::Complete {
            cursor.advance();
        }
        let placement = if applicability != HtmlAttributeApplicability::Applicable
            || role != Some(HtmlMediaElementRole::Meta)
        {
            HtmlMediaPlacementObservation::Unapplied
        } else if raw.is_none() {
            HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::NotSpecified)
        } else if !name_supplied {
            HtmlMediaPlacementObservation::Unchecked
        } else {
            let mut name_cursor =
                crate::media::ThemeColorNameCursor::new(name.map(|a| a.value().code_units()));
            while !name_cursor.is_complete() {
                name_cursor.advance();
            }
            HtmlMediaPlacementObservation::Known(if name_cursor.result() {
                HtmlMetaMediaPlacement::AllowedForThemeColor
            } else {
                HtmlMetaMediaPlacement::Forbidden
            })
        };
        Ok(HtmlMediaAttributeReport {
            raw: raw.cloned(),
            name: name.cloned(),
            applicability,
            role,
            value: cursor.result().expect("finished local cursor"),
            placement,
        })
    }
}
