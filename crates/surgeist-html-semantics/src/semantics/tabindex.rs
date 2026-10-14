//! Static tabIndex getter, HTML 877d6146487f #dom-tabindex.
//! This does not compute focusability, navigation order or focus effects.
use super::*;
use surgeist_html::bounded::{integer_long_work_bound, parse_integer_long};

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
            let bound = integer_long_work_bound(attribute.value().len())
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
            self.charge(bound)?;
            let parsed = parse_integer_long(attribute.value());
            self.poll()?;
            match parsed {
                Ok(Some(value)) => {
                    self.charge(1)?;
                    return Ok(HtmlSemanticValue::Known(HtmlTabIndexReadback {
                        value,
                        attribute: Some(attribute.clone()),
                        origin: HtmlTabIndexOrigin::Recovered,
                    }));
                }
                Ok(None) => HtmlTabIndexOrigin::OutOfRangeDefault,
                Err(_) => HtmlTabIndexOrigin::ParseFailureDefault,
            }
        } else {
            HtmlTabIndexOrigin::MissingDefault
        };
        let value = if html {
            match self.known(element)? {
                Some(
                    KnownHtmlElement::A
                    | KnownHtmlElement::Area
                    | KnownHtmlElement::Button
                    | KnownHtmlElement::Iframe
                    | KnownHtmlElement::Input
                    | KnownHtmlElement::Object
                    | KnownHtmlElement::Select
                    | KnownHtmlElement::Textarea,
                ) => 0,
                Some(KnownHtmlElement::Summary) => {
                    match self.summary_for_parent_details(subject)? {
                        HtmlSemanticValue::Known(true) => 0,
                        HtmlSemanticValue::Known(false) => -1,
                        other => return Ok(other.map(|_| unreachable!("known predicate handled"))),
                    }
                }
                _ => {
                    // The obsolete source spelling is outside the modern vocabulary.
                    // Exact slice comparison visits at most five units; no temporary string.
                    self.charge(6)?;
                    if element.name().local_name().code_units() == [102, 114, 97, 109, 101] {
                        0
                    } else {
                        -1
                    }
                }
            }
        } else {
            // The two already qualified foreign mixin interfaces default only a.
            self.charge(2)?;
            if element.name().local_name().code_units() == [97] {
                0
            } else {
                -1
            }
        };
        self.charge(1)?;
        Ok(HtmlSemanticValue::Known(HtmlTabIndexReadback {
            value,
            attribute: attribute.cloned(),
            origin,
        }))
    }
}
