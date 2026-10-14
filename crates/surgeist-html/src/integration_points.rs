//! HTML877d §13.2.6 integration-point decisions, independent of operand storage.
//!
//! The fragment fake token uses the captured attributes (§13.4 step 19).
//! Over the accepted raw domain, attribute identity is namespace/local name;
//! retained prefix metadata does not change that identity (issue #1175).

use crate::{DocumentString, OrderedAttributes, QualifiedName};
use std::convert::Infallible;

#[derive(Clone, Copy)]
pub(crate) enum Comparison {
    Exact,
    AsciiCaseInsensitive,
}

pub(crate) enum IntegrationPointRule {
    Never,
    Always,
    EncodingAttribute,
}

/// Adapters own field access and comparison costs. Only the selected branch
/// requests comparisons; the encoding branch alone requires attribute access.
pub(crate) fn element_rule<E>(
    namespace: Option<&DocumentString>,
    local: &DocumentString,
    mut compare: impl FnMut(&DocumentString, &'static str, Comparison) -> Result<bool, E>,
) -> Result<IntegrationPointRule, E> {
    let Some(namespace) = namespace else {
        return Ok(IntegrationPointRule::Never);
    };
    if compare(namespace, "http://www.w3.org/2000/svg", Comparison::Exact)? {
        for expected in ["foreignObject", "desc", "title"] {
            if compare(local, expected, Comparison::Exact)? {
                return Ok(IntegrationPointRule::Always);
            }
        }
    } else if compare(
        namespace,
        "http://www.w3.org/1998/Math/MathML",
        Comparison::Exact,
    )? && compare(local, "annotation-xml", Comparison::Exact)?
    {
        return Ok(IntegrationPointRule::EncodingAttribute);
    }
    Ok(IntegrationPointRule::Never)
}

/// `Some` identifies the unique encoding attribute, including a rejected MIME
/// value. Adapters can stop there without reading unrelated later attributes.
pub(crate) fn encoding_attribute<E>(
    namespace: Option<&DocumentString>,
    local: &DocumentString,
    value: &DocumentString,
    mut compare: impl FnMut(&DocumentString, &'static str, Comparison) -> Result<bool, E>,
) -> Result<Option<bool>, E> {
    if namespace.is_some() || !compare(local, "encoding", Comparison::Exact)? {
        return Ok(None);
    }
    Ok(Some(
        compare(value, "text/html", Comparison::AsciiCaseInsensitive)?
            || compare(
                value,
                "application/xhtml+xml",
                Comparison::AsciiCaseInsensitive,
            )?,
    ))
}

fn borrowed_comparison(
    value: &DocumentString,
    literal: &'static str,
    comparison: Comparison,
) -> Result<bool, Infallible> {
    Ok(value.len() == literal.len()
        && value
            .code_units()
            .iter()
            .zip(literal.bytes())
            .all(|(&unit, byte)| {
                let unit = match comparison {
                    Comparison::AsciiCaseInsensitive if (65..=90).contains(&unit) => unit + 32,
                    _ => unit,
                };
                unit == u16::from(byte)
            }))
}

pub(crate) fn is_html_integration_point(
    name: &QualifiedName,
    attributes: &OrderedAttributes,
) -> bool {
    let rule = element_rule(
        name.namespace().uri(),
        name.local_name(),
        borrowed_comparison,
    )
    .unwrap_or_else(|never| match never {});
    match rule {
        IntegrationPointRule::Never => false,
        IntegrationPointRule::Always => true,
        IntegrationPointRule::EncodingAttribute => attributes
            .iter()
            .find_map(|attribute| {
                let name = attribute.name();
                encoding_attribute(
                    name.namespace().uri(),
                    name.local_name(),
                    attribute.value(),
                    borrowed_comparison,
                )
                .unwrap_or_else(|never| match never {})
            })
            .unwrap_or(false),
    }
}
