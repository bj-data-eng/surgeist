//! HTML877d §13.2.6 integration-point decisions, independent of operand storage.
//!
//! The fragment fake token uses the captured attributes (§13.4 step 19).
//! Over the accepted raw domain, attribute identity is namespace/local name;
//! retained prefix metadata does not change that identity (issue #1175).

use crate::{
    internal::values::DocumentString,
    internal::values::semantic_input::HtmlAttributeInput,
    internal::values::semantic_input::HtmlNameInput,
    vocabulary::{KnownMathmlElement, MATHML_TEXT_INTEGRATION_POINTS, SVG_HTML_INTEGRATION_POINTS},
};
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
/// requests comparisons; the encoding branch alone requires attribute access. The comparator
/// receives the exact UTF-16 code units of the name it compares.
pub fn html_integration_point_rule<E>(
    namespace: Option<&DocumentString>,
    local: &DocumentString,
    mut compare: impl FnMut(&[u16], &'static str, HtmlIntegrationPointComparison) -> Result<bool, E>,
) -> Result<HtmlIntegrationPointRule, E> {
    use HtmlIntegrationPointComparison::Exact;
    use HtmlIntegrationPointRule::{Always, EncodingAttribute, Never};
    let Some(namespace) = namespace else {
        return Ok(Never);
    };
    if compare(namespace.code_units(), "http://www.w3.org/2000/svg", Exact)? {
        for known in SVG_HTML_INTEGRATION_POINTS {
            if compare(local.code_units(), known.local_name(), Exact)? {
                return Ok(Always);
            }
        }
    } else if compare(
        namespace.code_units(),
        "http://www.w3.org/1998/Math/MathML",
        Exact,
    )? && compare(
        local.code_units(),
        KnownMathmlElement::AnnotationXml.local_name(),
        Exact,
    )? {
        return Ok(EncodingAttribute);
    }
    Ok(Never)
}

/// `Some` identifies the unique encoding attribute, including a rejected MIME
/// value. Adapters can stop there without reading unrelated later attributes.
/// `value` is the attribute's exact UTF-16 code units; the comparator receives the units of
/// the name or of the value it compares.
pub fn html_integration_point_encoding_attribute<E>(
    namespace: Option<&DocumentString>,
    local: &DocumentString,
    value: &[u16],
    mut compare: impl FnMut(&[u16], &'static str, HtmlIntegrationPointComparison) -> Result<bool, E>,
) -> Result<Option<bool>, E> {
    use HtmlIntegrationPointComparison::{AsciiCaseInsensitive, Exact};
    if namespace.is_some() || !compare(local.code_units(), "encoding", Exact)? {
        return Ok(None);
    }
    Ok(Some(
        compare(value, "text/html", AsciiCaseInsensitive)?
            || compare(value, "application/xhtml+xml", AsciiCaseInsensitive)?,
    ))
}

fn borrowed_comparison(
    value: &[u16],
    literal: &'static str,
    comparison: HtmlIntegrationPointComparison,
) -> Result<bool, Infallible> {
    Ok(value.len() == literal.len()
        && value
            .iter()
            .zip(literal.bytes())
            .all(|(&unit, byte)| match comparison {
                HtmlIntegrationPointComparison::AsciiCaseInsensitive => {
                    crate::internal::microsyntax::tokens::ascii_case_insensitive_unit(unit, byte)
                }
                HtmlIntegrationPointComparison::Exact => unit == u16::from(byte),
            }))
}

/// Classifies an HTML integration point over exact borrowed construction values.
/// Attribute identity uses namespace/local name and ignores retained prefix metadata.
pub fn is_html_integration_point(
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
) -> bool {
    let rule =
        html_integration_point_rule(name.namespace_uri(), name.local_name(), borrowed_comparison)
            .unwrap_or_else(|never| match never {});
    match rule {
        HtmlIntegrationPointRule::Never => false,
        HtmlIntegrationPointRule::Always => true,
        HtmlIntegrationPointRule::EncodingAttribute => {
            crate::internal::values::semantic_input::attributes_iter(attributes)
                .find_map(|attribute| {
                    let name = attribute.name();
                    html_integration_point_encoding_attribute(
                        name.namespace_uri(),
                        name.local_name(),
                        attribute.value().code_units(),
                        borrowed_comparison,
                    )
                    .unwrap_or_else(|never| match never {})
                })
                .unwrap_or(false)
        }
    }
}

/// One grouped comparison requested by MathML text integration-point classification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    ))? && compare(HtmlMathmlTextPointTest::Local(&MATHML_TEXT_POINT_NAMES))?)
}

const MATHML_TEXT_POINT_NAMES: [&str; MATHML_TEXT_INTEGRATION_POINTS.len()] =
    local_names(&MATHML_TEXT_INTEGRATION_POINTS);

/// The canonical spellings of a group of MathML elements.
const fn local_names<const N: usize>(group: &[KnownMathmlElement; N]) -> [&'static str; N] {
    let mut names = [""; N];
    let mut position = 0;
    while position < N {
        names[position] = group[position].local_name();
        position += 1;
    }
    names
}

/// Classifies MathML text integration points from an exact expanded name.
/// Prefix metadata, scripting and attributes do not influence this local rule.
pub fn is_mathml_text_integration_point(
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> bool {
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
