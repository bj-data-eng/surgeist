//! Bounded specified text for authored Values 4 URL functions.

use crate::component_values::CssCanonicalBuilder;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssComponentValueError, CssComponentValueErrorKind, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssUrl,
    CssUrlFunction, CssUrlModifier,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssUrl {
    /// Serializes the authored URL function without resolving its target or modifiers.
    ///
    /// The decoded target uses CSS string serialization, which replaces U+0000
    /// with U+FFFD. The stored authored value remains unchanged.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one complete URL function under cumulative node and byte limits.
    ///
    /// One URL aggregate, its string target, each modifier, and every retained
    /// argument component (including nested components and trivia) each charge
    /// one input node and one projection node.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    /// Appends to an enclosing specified-value writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        // An enclosing shorthand can prove this complete URL redundant. Hidden
        // authored input still incurs every provider visit, but needs no escaped
        // output scratch bounded by the already consumed final byte budget.
        let suppressed = writer.context.output_suppressed();
        charge_nodes(writer, 1)?;
        if !suppressed {
            writer.append(match self.function() {
                CssUrlFunction::Url => "url(",
                CssUrlFunction::Src => "src(",
            })?;
        }

        charge_nodes(writer, 1)?;
        if !suppressed {
            writer.append_string(self.as_str())?;
        }

        for modifier in self.modifiers() {
            if !suppressed {
                writer.append(" ")?;
            }
            charge_nodes(writer, 1)?;
            match modifier {
                CssUrlModifier::Ident(ident) => {
                    if !suppressed {
                        writer.append_identifier(ident.as_str())?;
                    }
                }
                CssUrlModifier::Function(function) => {
                    let components = function.argument_components();
                    charge_nodes(writer, components.component_count())?;
                    if suppressed {
                        continue;
                    }
                    writer.append_identifier(function.name())?;
                    writer.append("(")?;
                    // The opening parenthesis separates the first component from
                    // the function name. A successful builder finish guarantees
                    // that appending the closing parenthesis cannot consume an
                    // unfinished token, comment, or escape.
                    let mut builder = CssCanonicalBuilder::new(writer.context.remaining_bytes());
                    builder
                        .push_components(components.items())
                        .map_err(component_error)?;
                    let serialized = builder.finish().map_err(component_error)?;
                    writer.append(serialized.as_css())?;
                    writer.append(")")?;
                }
            }
        }
        if !suppressed {
            writer.append(")")?;
        }
        Ok(())
    }
}

fn charge_nodes(writer: &mut SpecifiedRuleWriter, count: usize) -> Result<()> {
    writer.context.charge_input(count)?;
    writer.context.charge_projection(count)
}

fn component_error(error: CssComponentValueError) -> CssSpecifiedValueSerializationError {
    use CssComponentValueErrorKind as Component;
    use CssSpecifiedValueSerializationErrorKind as Specified;

    let kind = match error.kind() {
        Component::ByteLimit => Specified::ByteLimit,
        Component::CapacityOverflow => Specified::CapacityOverflow,
        Component::UnserializableBoundary => Specified::UnserializableBoundary,
        // Argument components are checked at construction and retain their
        // lexical form. Any later non-resource failure is an unrepresentable
        // boundary, not a resource exhaustion.
        _ => Specified::UnserializableBoundary,
    };
    CssSpecifiedValueSerializationError::new(kind)
}
