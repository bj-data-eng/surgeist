//! Shared serialization of a substitution-dependent whole component value.

use crate::{
    CssComponentValueErrorKind, CssComponentValues, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind,
    specified_serialization::SpecifiedSerializationContext,
};

/// Pending values retain their complete token stream: no grammar-specific
/// canonicalization is sound until the caller supplies substitution results.
pub(crate) fn append_pending_specified(
    values: &CssComponentValues,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> Result<(), CssSpecifiedValueSerializationError> {
    let nodes = values.component_count().checked_add(1).ok_or_else(|| {
        CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    context.charge_input(nodes)?;
    context.charge_projection(nodes)?;
    let serialized = values
        .serialize_with_limit(context.remaining_bytes())
        .map_err(|error| {
            let kind = match error.kind() {
                CssComponentValueErrorKind::ByteLimit => {
                    CssSpecifiedValueSerializationErrorKind::ByteLimit
                }
                _ => CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            };
            CssSpecifiedValueSerializationError::new(kind)
        })?;
    context.append(output, serialized.as_css())
}
