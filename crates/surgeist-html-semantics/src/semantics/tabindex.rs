//! Static tabIndex getter, HTML 877d6146487f #dom-tabindex.
//! This does not compute focusability, navigation order or focus effects.
use super::*;
use surgeist_html::bounded::numeric::{integer_long_work_bound, parse_integer_long};
use surgeist_html::{
    values::{Attribute, DocumentString},
    vocabulary::KnownHtmlElement,
};

const LONGEST_HTML_NAME: usize = {
    let mut names = KnownHtmlElement::ALL;
    let mut longest = 0;
    while let Some((name, remaining)) = names.split_first() {
        let len = name.local_name().len();
        if len > longest {
            longest = len;
        }
        names = remaining;
    }
    longest
};

/// Why the getter returned its value. Defaults depend on source element role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTabIndexOrigin {
    Recovered,
    MissingDefault,
    ParseFailureDefault,
    OutOfRangeDefault,
}
/// Capture-qualified static getter value with the exact selected raw attribute.
/// Processing recovery is independent of authored syntax conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTabIndexReadback {
    value: i32,
    attribute: Option<Attribute>,
    origin: HtmlTabIndexOrigin,
}
impl HtmlTabIndexReadback {
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn attribute(&self) -> Option<&Attribute> {
        self.attribute.as_ref()
    }
    pub fn origin(&self) -> HtmlTabIndexOrigin {
        self.origin
    }
}
/// The reason a present tabindex attribute cannot supply a signed long.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTabIndexRecoveryError {
    InvalidSyntax,
    OutOfRange,
    RepresentationExhausted,
}

/// Recover the static tabindex integer after admitting the HTML parser's work.
///
/// The outer error is the original caller refusal. The inner error identifies
/// why a present attribute could not supply an in-range value. An unrepresentable
/// work bound prevents parsing.
pub fn recover_tab_index_with<E>(
    raw: &DocumentString,
    admit: impl FnOnce(usize) -> Result<(), E>,
) -> Result<Result<i32, HtmlTabIndexRecoveryError>, E> {
    let Some(work) = integer_long_work_bound(raw.len()) else {
        return Ok(Err(HtmlTabIndexRecoveryError::RepresentationExhausted));
    };
    admit(work)?;
    Ok(match parse_integer_long(raw) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(HtmlTabIndexRecoveryError::OutOfRange),
        Err(_) => Err(HtmlTabIndexRecoveryError::InvalidSyntax),
    })
}

/// The static HTML tabindex default after classification of the element's name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTabIndexDefault {
    Zero,
    NegativeOne,
    FirstSummary,
}

impl HtmlTabIndexDefault {
    /// Select the fallback for an exact HTML local name. Admit the bounded
    /// vocabulary lookup and any reached legacy name comparison.
    pub fn for_html_with<E>(
        local_name: &[u16],
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, E> {
        // The vocabulary hash reads a candidate name at most twice,
        // stopping at its header for lengths beyond the longest spelling.
        let work = if local_name.len() <= LONGEST_HTML_NAME {
            1 + 2 * local_name.len()
        } else {
            1
        };
        admit(work)?;
        Self::for_known_html_with(KnownHtmlElement::from_units(local_name), local_name, admit)
    }

    fn for_known_html_with<E>(
        known: Option<KnownHtmlElement>,
        local_name: &[u16],
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<Self, E> {
        Ok(match known {
            Some(
                KnownHtmlElement::A
                | KnownHtmlElement::Area
                | KnownHtmlElement::Button
                | KnownHtmlElement::Iframe
                | KnownHtmlElement::Input
                | KnownHtmlElement::Object
                | KnownHtmlElement::Select
                | KnownHtmlElement::Textarea,
            ) => Self::Zero,
            Some(KnownHtmlElement::Summary) => Self::FirstSummary,
            _ => tab_index_exact_name_with(local_name, &b"frame".map(u16::from), admit)?,
        })
    }

    /// Select the fallback for an exact SVG or MathML element name, admitting
    /// the comparison with the canonical anchor spelling.
    pub fn for_svg_or_mathml_with<E>(
        local_name: &[u16],
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<Self, E> {
        tab_index_exact_name_with(local_name, &[u16::from(b'a')], admit)
    }
}

fn tab_index_exact_name_with<E>(
    local_name: &[u16],
    expected: &[u16],
    admit: impl FnOnce(usize) -> Result<(), E>,
) -> Result<HtmlTabIndexDefault, E> {
    admit(expected.len() + 1)?;
    Ok(if local_name == expected {
        HtmlTabIndexDefault::Zero
    } else {
        HtmlTabIndexDefault::NegativeOne
    })
}

impl HtmlSemanticSession<'_> {
    pub(super) fn tab_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTabIndexReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let html = self.html_namespace(element)?;
        if !html
            && !self.namespace_is(element, "http://www.w3.org/2000/svg")?
            && !self.namespace_is(element, "http://www.w3.org/1998/Math/MathML")?
        {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let attribute = self.attribute(element, "tabindex")?;
        self.interpreter
            .admit_attribute_raw(attribute.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let origin = if let Some(attribute) = attribute {
            let parsed = recover_tab_index_with(attribute.value(), |work| self.charge(work))?;
            if parsed == Err(HtmlTabIndexRecoveryError::RepresentationExhausted) {
                return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
            }
            self.poll()?;
            match parsed {
                Ok(value) => {
                    self.charge(1)?;
                    return Ok(HtmlSemanticValue::Known(HtmlTabIndexReadback {
                        value,
                        attribute: Some(attribute.clone()),
                        origin: HtmlTabIndexOrigin::Recovered,
                    }));
                }
                Err(HtmlTabIndexRecoveryError::OutOfRange) => HtmlTabIndexOrigin::OutOfRangeDefault,
                Err(HtmlTabIndexRecoveryError::InvalidSyntax) => {
                    HtmlTabIndexOrigin::ParseFailureDefault
                }
                Err(HtmlTabIndexRecoveryError::RepresentationExhausted) => {
                    return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
                }
            }
        } else {
            HtmlTabIndexOrigin::MissingDefault
        };
        let local = element.name().local_name().code_units();
        let default = if html {
            let known = self.known(element)?;
            HtmlTabIndexDefault::for_known_html_with(known, local, |work| self.charge(work))?
        } else {
            HtmlTabIndexDefault::for_svg_or_mathml_with(local, |work| self.charge(work))?
        };
        let value = match default {
            HtmlTabIndexDefault::Zero => 0,
            HtmlTabIndexDefault::NegativeOne => -1,
            HtmlTabIndexDefault::FirstSummary => match self.summary_for_parent_details(subject)? {
                HtmlSemanticValue::Known(true) => 0,
                HtmlSemanticValue::Known(false) => -1,
                HtmlSemanticValue::Missing(missing) => {
                    return Ok(HtmlSemanticValue::Missing(missing));
                }
                HtmlSemanticValue::Inapplicable => {
                    return Ok(HtmlSemanticValue::Inapplicable);
                }
                HtmlSemanticValue::Unsupported(reason) => {
                    return Ok(HtmlSemanticValue::Unsupported(reason));
                }
            },
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlTabIndexReadback {
            value,
            attribute: attribute.cloned(),
            origin,
        }))
    }
}
