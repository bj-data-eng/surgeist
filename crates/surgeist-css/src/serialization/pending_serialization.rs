//! Shared serialization of a substitution-dependent whole component value.

use crate::component::CssComponentValues;
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::{
    CssSpecifiedValueSerializationErrorKind, SpecifiedSerializationContext,
};

/// Pending values retain their complete token stream: no grammar-specific
/// canonicalization is sound until the caller supplies substitution results.
pub(crate) fn append_pending_specified(
    values: &CssComponentValues,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> Result<(), CssSerializationError> {
    let nodes = values.component_count().checked_add(1).ok_or_else(|| {
        CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
    })?;
    context.charge_nodes(nodes)?;
    if context.output_suppressed() {
        return Ok(());
    }
    let serialized = values
        .serialize_tokens(
            crate::component::CssComponentValueLimits::with_max_css_bytes(
                context.remaining_bytes(),
            ),
        )
        .map_err(crate::component::values::specified_component_error)?;
    context.append(output, serialized.as_css())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::CssComponentValueLimits;
    use crate::serialization::CssSerializationLimits;

    #[test]
    fn retained_trailing_reverse_solidus_is_an_unserializable_boundary() {
        // Syntax 3 makes `\` a delimiter only before a newline; without it,
        // `\` at EOF re-tokenizes as an escape. Public construction rejects
        // this stream, so the private harness drops the guarding newline.
        let parsed = crate::component::values::parse_component_values("env(axis) \\\n").unwrap();
        let items = parsed.items();
        assert!(matches!(
            items[items.len() - 1].view(),
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Whitespace(_)
            )
        ));
        let values = CssComponentValues::retain_admitted(
            &items[..items.len() - 1],
            CssComponentValueLimits::default(),
        )
        .unwrap();
        let mut context = SpecifiedSerializationContext::new(CssSerializationLimits::default());
        let mut output = String::new();
        assert_eq!(
            append_pending_specified(&values, &mut context, &mut output)
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::UnserializableBoundary
        );
        assert!(output.is_empty());
    }
}
