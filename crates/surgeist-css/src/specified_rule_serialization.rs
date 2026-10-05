//! Fail-closed canonical specified rule and stylesheet composition.

use std::fmt;

use crate::{
    CssNamespaceRule, CssRule, CssSheet, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits,
    specified_serialization::SpecifiedSerializationContext,
};

/// Why generic rule or sheet serialization could not return complete CSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpecifiedRuleSerializationErrorKind {
    UnsupportedRule,
    UnsupportedEncoding,
    /// A node, byte or capacity resource was exhausted.
    Resource(CssSpecifiedValueSerializationErrorKind),
    /// A value or component boundary cannot be faithfully emitted.
    Value(CssSpecifiedValueSerializationErrorKind),
}

/// An unsupported rule is never omitted or silently serialized as raw syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedRuleSerializationError {
    kind: CssSpecifiedRuleSerializationErrorKind,
    rule_index: Option<usize>,
    source: Option<CssSpecifiedValueSerializationError>,
}

impl CssSpecifiedRuleSerializationError {
    #[must_use]
    pub const fn kind(&self) -> CssSpecifiedRuleSerializationErrorKind {
        self.kind
    }

    /// Zero-based stylesheet rule index when a rule caused the failure.
    #[must_use]
    pub const fn rule_index(&self) -> Option<usize> {
        self.rule_index
    }

    fn unsupported_rule(rule_index: Option<usize>) -> Self {
        Self {
            kind: CssSpecifiedRuleSerializationErrorKind::UnsupportedRule,
            rule_index,
            source: None,
        }
    }

    fn unsupported_encoding() -> Self {
        Self {
            kind: CssSpecifiedRuleSerializationErrorKind::UnsupportedEncoding,
            rule_index: None,
            source: None,
        }
    }

    fn value_error(error: CssSpecifiedValueSerializationError, rule_index: Option<usize>) -> Self {
        let kind = match error.kind() {
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            | CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            | CssSpecifiedValueSerializationErrorKind::ByteLimit
            | CssSpecifiedValueSerializationErrorKind::CapacityOverflow => {
                CssSpecifiedRuleSerializationErrorKind::Resource(error.kind())
            }
            CssSpecifiedValueSerializationErrorKind::UnserializableBoundary
            | CssSpecifiedValueSerializationErrorKind::UnrepresentableValue => {
                CssSpecifiedRuleSerializationErrorKind::Value(error.kind())
            }
        };
        Self {
            kind,
            rule_index,
            source: Some(error),
        }
    }
}

impl fmt::Display for CssSpecifiedRuleSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CssSpecifiedRuleSerializationErrorKind::UnsupportedRule => {
                formatter.write_str("canonical serialization is not available for this CSS rule")
            }
            CssSpecifiedRuleSerializationErrorKind::UnsupportedEncoding => formatter
                .write_str("canonical serialization is not available for encoding metadata"),
            CssSpecifiedRuleSerializationErrorKind::Resource(_) => {
                formatter.write_str("canonical CSS rule serialization exceeded a resource limit")
            }
            CssSpecifiedRuleSerializationErrorKind::Value(_) => {
                formatter.write_str("canonical CSS rule serialization cannot represent a value")
            }
        }
    }
}

impl std::error::Error for CssSpecifiedRuleSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

pub(crate) struct SpecifiedRuleWriter {
    pub(crate) context: SpecifiedSerializationContext,
    pub(crate) css: String,
}

impl SpecifiedRuleWriter {
    pub(crate) fn new(limits: CssSpecifiedValueSerializationLimits) -> Self {
        Self {
            context: SpecifiedSerializationContext::new(limits),
            css: String::new(),
        }
    }

    pub(crate) fn append(&mut self, text: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.append(&mut self.css, text)
    }

    /// Emits a namespace declaration without resolving or normalizing its literal name.
    fn namespace(
        &mut self,
        rule: &CssNamespaceRule,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append("@namespace ")?;
        if let Some(prefix) = rule.prefix() {
            self.context.charge_input(1)?;
            self.context.charge_projection(1)?;
            self.append_identifier(prefix.as_str())?;
            self.append(" ")?;
        }
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append("url(")?;
        self.append_string(rule.name().as_str())?;
        self.append(");")
    }

    /// Visits a proved simple initial through its provider without producing bytes.
    /// Restores the enclosing emission mode even when a nested visit fails.
    pub(crate) fn without_output<T>(
        &mut self,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSpecifiedValueSerializationError>,
    ) -> Result<T, CssSpecifiedValueSerializationError> {
        let previous = self.context.replace_output_suppression(true);
        let result = visit(self);
        self.context.replace_output_suppression(previous);
        result
    }

    pub(crate) fn append_identifier(
        &mut self,
        value: &str,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.append_escaped(value, EscapedKind::Identifier)
    }

    pub(crate) fn append_string(
        &mut self,
        value: &str,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.append_escaped(value, EscapedKind::String)
    }

    fn append_escaped(
        &mut self,
        value: &str,
        kind: EscapedKind,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        if self.context.output_suppressed() {
            return Ok(());
        }
        let mut scratch = String::new();
        let mut bounded = BoundedEscaped {
            context: &self.context,
            css: &mut scratch,
            error: None,
        };
        let result = match kind {
            EscapedKind::Identifier => cssparser::serialize_identifier(value, &mut bounded),
            EscapedKind::String => cssparser::serialize_string(value, &mut bounded),
        };
        if result.is_err() {
            return Err(bounded.error.unwrap_or_else(|| {
                CssSpecifiedValueSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            }));
        }
        self.append(&scratch)
    }
}

enum EscapedKind {
    Identifier,
    String,
}

struct BoundedEscaped<'a> {
    context: &'a SpecifiedSerializationContext,
    css: &'a mut String,
    error: Option<CssSpecifiedValueSerializationError>,
}

impl fmt::Write for BoundedEscaped<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.context
            .append_temporary(self.css, text)
            .map_err(|error| {
                self.error = Some(error);
                fmt::Error
            })
    }
}

impl CssNamespaceRule {
    /// Serializes the declaration using CSSOM's escaped prefix and quoted URL form.
    /// The stored literal name and optional source position remain unchanged.
    /// CSSOM string serialization replaces programmatic U+0000 with U+FFFD.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under cumulative node and generated-byte limits.
    /// The rule, its optional prefix, and its literal name each charge one input
    /// and one projection node. Enclosing sheets share these same budgets.
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.namespace(self)?;
        Ok(writer.css)
    }
}

impl CssRule {
    /// Serializes only rule kinds with a complete canonical specified writer.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedRuleSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        append_rule(&mut writer, self, None)?;
        Ok(writer.css)
    }
}

impl CssSheet {
    /// Serializes a complete supported stylesheet atomically in source order.
    /// Unsupported rules or legacy encoding metadata fail closed.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedRuleSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedRuleSerializationError> {
        if self.encoding().is_some() {
            return Err(CssSpecifiedRuleSerializationError::unsupported_encoding());
        }
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .context
            .charge_input(1)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, None))?;
        writer
            .context
            .charge_projection(1)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, None))?;
        for (index, rule) in self.rules().iter().enumerate() {
            if index != 0 {
                writer.append("\n").map_err(|error| {
                    CssSpecifiedRuleSerializationError::value_error(error, Some(index))
                })?;
            }
            append_rule(&mut writer, rule, Some(index))?;
        }
        Ok(writer.css)
    }
}

fn append_rule(
    writer: &mut SpecifiedRuleWriter,
    rule: &CssRule,
    index: Option<usize>,
) -> Result<(), CssSpecifiedRuleSerializationError> {
    match rule {
        CssRule::CounterStyle(rule) => rule
            .append_to_rule_writer(writer)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::FontFace(rule) => writer
            .font_face(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::FontFeatureValues(rule) => writer
            .font_features(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::Namespace(rule) => writer
            .namespace(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::ColorProfile(rule) => writer
            .color_profile(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::FontPaletteValues(rule) => writer
            .palette(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::SupportsCondition(rule) => writer
            .named_supports_rule(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index)),
        CssRule::NestedDeclarations(rule) => {
            // The logical rule and its authored declaration list are distinct
            // aggregates; the enum carrier does not add another rule charge.
            writer
                .context
                .charge_input(1)
                .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index))?;
            writer
                .context
                .charge_projection(1)
                .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index))?;
            writer
                .append_authored_declaration_list(rule.declarations())
                .map_err(|error| CssSpecifiedRuleSerializationError::value_error(error, index))
        }
        _ => Err(CssSpecifiedRuleSerializationError::unsupported_rule(index)),
    }
}

#[cfg(test)]
mod suppression_tests {
    use super::*;

    #[test]
    fn nested_suppression_restores_enclosing_mode_on_success_and_failure() {
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(1, 1, 1));
        writer
            .without_output(|writer| {
                writer.append("discarded outer text")?;
                writer.without_output(|writer| writer.append("discarded nested text"))?;
                assert!(writer.context.output_suppressed());
                let error = writer
                    .without_output(|writer| writer.context.charge_input(2))
                    .unwrap_err();
                assert_eq!(
                    error.kind(),
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                );
                assert!(writer.context.output_suppressed());
                writer.append("still discarded")
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }

    #[test]
    fn failing_suppressed_literal_preserves_nodes_and_resumes_final_emission() {
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(1, 0, 1));
        let zero = crate::CssSpecifiedLengthPercentage::zero();
        let error = writer
            .without_output(|writer| zero.append_specified(&mut writer.context, &mut writer.css))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }
}

#[cfg(test)]
mod value_error_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn value_failure_mapping_preserves_category_source_and_rule_index() {
        use CssSpecifiedValueSerializationErrorKind as Kind;
        for (kind, resource) in [
            (Kind::InputNodeLimit, true),
            (Kind::ProjectionNodeLimit, true),
            (Kind::ByteLimit, true),
            (Kind::CapacityOverflow, true),
            (Kind::UnserializableBoundary, false),
            (Kind::UnrepresentableValue, false),
        ] {
            for index in [None, Some(7)] {
                let source = CssSpecifiedValueSerializationError::new(kind);
                let error = CssSpecifiedRuleSerializationError::value_error(source.clone(), index);
                assert_eq!(
                    error.kind(),
                    if resource {
                        CssSpecifiedRuleSerializationErrorKind::Resource(kind)
                    } else {
                        CssSpecifiedRuleSerializationErrorKind::Value(kind)
                    }
                );
                assert_eq!(error.rule_index(), index);
                assert_eq!(
                    error
                        .source()
                        .unwrap()
                        .downcast_ref::<CssSpecifiedValueSerializationError>(),
                    Some(&source)
                );
                assert_eq!(
                    error.to_string(),
                    if resource {
                        "canonical CSS rule serialization exceeded a resource limit"
                    } else {
                        "canonical CSS rule serialization cannot represent a value"
                    }
                );
            }
        }
    }

    #[test]
    fn unrepresentable_failure_explains_a_value_limit_without_a_resource_claim() {
        let error = CssSpecifiedValueSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
        );
        assert_eq!(
            error.to_string(),
            "specified value cannot be represented by the selected serializer"
        );
        assert!(error.source().is_none());
    }
}
