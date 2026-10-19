//! HTML877d §13.2.6 integration-point decisions, independent of operand storage.
//!
//! The fragment fake token uses the captured attributes (§13.4 step 19).
//! Over the accepted raw domain, attribute identity is namespace/local name;
//! retained prefix metadata does not change that identity (issue #1175).

use crate::{DocumentString, HtmlAttributeInput, HtmlNameInput};
use std::convert::Infallible;

/// Comparison semantics requested by an HTML integration-point rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIntegrationPointComparison {
    /// Compare exact UTF-16 units with the defining ASCII spelling.
    Exact,
    /// Fold ASCII uppercase units before comparison; do not trim whitespace.
    AsciiCaseInsensitive,
}

/// The remaining decision after classifying an element's expanded name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIntegrationPointRule {
    /// The element is not an HTML integration point.
    Never,
    /// Its expanded name is sufficient; no attributes need to be read.
    Always,
    /// Inspect its no-namespace encoding attribute before deciding.
    EncodingAttribute,
}

/// Adapters own field access and comparison costs. Only the selected branch
/// requests comparisons; the encoding branch alone requires attribute access.
pub fn html_integration_point_rule<E>(
    namespace: Option<&DocumentString>,
    local: &DocumentString,
    mut compare: impl FnMut(
        &DocumentString,
        &'static str,
        HtmlIntegrationPointComparison,
    ) -> Result<bool, E>,
) -> Result<HtmlIntegrationPointRule, E> {
    use HtmlIntegrationPointComparison::Exact;
    use HtmlIntegrationPointRule::{Always, EncodingAttribute, Never};
    let Some(namespace) = namespace else {
        return Ok(Never);
    };
    if compare(namespace, "http://www.w3.org/2000/svg", Exact)? {
        for expected in ["foreignObject", "desc", "title"] {
            if compare(local, expected, Exact)? {
                return Ok(Always);
            }
        }
    } else if compare(namespace, "http://www.w3.org/1998/Math/MathML", Exact)?
        && compare(local, "annotation-xml", Exact)?
    {
        return Ok(EncodingAttribute);
    }
    Ok(Never)
}

/// `Some` identifies the unique encoding attribute, including a rejected MIME
/// value. Adapters can stop there without reading unrelated later attributes.
pub fn html_integration_point_encoding_attribute<E>(
    namespace: Option<&DocumentString>,
    local: &DocumentString,
    value: &DocumentString,
    mut compare: impl FnMut(
        &DocumentString,
        &'static str,
        HtmlIntegrationPointComparison,
    ) -> Result<bool, E>,
) -> Result<Option<bool>, E> {
    use HtmlIntegrationPointComparison::{AsciiCaseInsensitive, Exact};
    if namespace.is_some() || !compare(local, "encoding", Exact)? {
        return Ok(None);
    }
    Ok(Some(
        compare(value, "text/html", AsciiCaseInsensitive)?
            || compare(value, "application/xhtml+xml", AsciiCaseInsensitive)?,
    ))
}

fn borrowed_comparison(
    value: &DocumentString,
    literal: &'static str,
    comparison: HtmlIntegrationPointComparison,
) -> Result<bool, Infallible> {
    Ok(value.len() == literal.len()
        && value
            .code_units()
            .iter()
            .zip(literal.bytes())
            .all(|(&unit, byte)| {
                let unit = match comparison {
                    HtmlIntegrationPointComparison::AsciiCaseInsensitive
                        if (65..=90).contains(&unit) =>
                    {
                        unit + 32
                    }
                    _ => unit,
                };
                unit == u16::from(byte)
            }))
}

/// Classifies an HTML integration point over exact borrowed construction values.
/// Attribute identity uses namespace/local name and ignores retained prefix metadata.
pub fn is_html_integration_point(
    name: &(impl crate::HtmlNameInput + ?Sized),
    attributes: &(impl crate::HtmlAttributesInput + ?Sized),
) -> bool {
    let rule =
        html_integration_point_rule(name.namespace_uri(), name.local_name(), borrowed_comparison)
            .unwrap_or_else(|never| match never {});
    match rule {
        HtmlIntegrationPointRule::Never => false,
        HtmlIntegrationPointRule::Always => true,
        HtmlIntegrationPointRule::EncodingAttribute => {
            crate::semantic_input::attributes_iter(attributes)
                .find_map(|attribute| {
                    let name = attribute.name();
                    html_integration_point_encoding_attribute(
                        name.namespace_uri(),
                        name.local_name(),
                        attribute.value(),
                        borrowed_comparison,
                    )
                    .unwrap_or_else(|never| match never {})
                })
                .unwrap_or(false)
        }
    }
}

/// One grouped comparison requested by MathML text integration-point classification.
pub enum HtmlMathmlTextPointTest<'a> {
    /// Compare the supplied name's namespace against this exact URI.
    Namespace(&'static str),
    /// Compare its local name with this ordered group of exact ASCII names.
    Local(&'a [&'static str]),
}

/// Classifies MathML text integration points without owning operand reads or admission.
/// Namespace refusal/false precedes every local-name observation; local names
/// are supplied as one group so consumers retain their actual group-read costs.
pub fn mathml_text_integration_point<E>(
    mut compare: impl FnMut(HtmlMathmlTextPointTest<'_>) -> Result<bool, E>,
) -> Result<bool, E> {
    Ok(compare(HtmlMathmlTextPointTest::Namespace(
        "http://www.w3.org/1998/Math/MathML",
    ))? && compare(HtmlMathmlTextPointTest::Local(&[
        "mi", "mo", "mn", "ms", "mtext",
    ]))?)
}

/// Classifies MathML text integration points from an exact expanded name.
/// Prefix metadata, scripting and attributes do not influence this local rule.
pub fn is_mathml_text_integration_point(name: &(impl crate::HtmlNameInput + ?Sized)) -> bool {
    // Retain the ordinary capture adapter's exact-unit iterator comparisons;
    // parser adapters independently admit their length/header and pair reads.
    let exact = |value: &DocumentString, literal: &str| {
        value
            .code_units()
            .iter()
            .copied()
            .eq(literal.encode_utf16())
    };
    mathml_text_integration_point(|test| {
        Ok::<_, Infallible>(match test {
            HtmlMathmlTextPointTest::Namespace(expected) => {
                name.namespace_uri().is_some_and(|uri| exact(uri, expected))
            }
            HtmlMathmlTextPointTest::Local(choices) => choices
                .iter()
                .any(|expected| exact(name.local_name(), expected)),
        })
    })
    .unwrap_or_else(|never| match never {})
}
