//! Bounded specified text for authored Values 4 URL functions.

use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::values::specified_component_error;
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::common::{CssUrl, CssUrlFunction, CssUrlModifier};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssUrl {
    /// Appends to an enclosing specified-value writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        // An enclosing shorthand can prove this complete URL redundant. Hidden
        // authored input still incurs every provider visit, but needs no escaped
        // output scratch bounded by the already consumed final byte budget.
        let suppressed = writer.context.output_suppressed();
        writer.node()?;
        if !suppressed {
            writer.append(match self.function() {
                CssUrlFunction::Url => "url(",
                CssUrlFunction::Src => "src(",
            })?;
        }

        writer.node()?;
        if !suppressed {
            writer.append_string(self.as_str())?;
        }

        for modifier in self.modifiers() {
            if !suppressed {
                writer.append(" ")?;
            }
            writer.node()?;
            match modifier {
                CssUrlModifier::Ident(ident) => {
                    if !suppressed {
                        writer.append_identifier(ident.as_str())?;
                    }
                }
                CssUrlModifier::Function(function) => {
                    let components = function.argument_components();
                    writer.nodes(components.component_count())?;
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
                        .map_err(specified_component_error)?;
                    let serialized = builder.finish().map_err(specified_component_error)?;
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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssUrl {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssUrl);
