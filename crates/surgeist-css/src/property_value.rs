//! Checked property-value construction and diagnostics mapped to input components.

use std::fmt;

use crate::{
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValues, CssDeclaration,
    CssImportance, CssPropertyNameRef, CssSerializedOrigin, CssSerializedValue, Error, ErrorKind,
};

/// A component invariant or property grammar rejected by checked value construction.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssPropertyValueErrorKind {
    /// Component serialization could not preserve the supplied syntax or resource bounds.
    Component(CssComponentValueErrorKind),
    /// The declaration-value boundary or selected property grammar rejected the input.
    Grammar(ErrorKind),
}

/// A property-value construction failure located in the original component input.
///
/// Unlike a stylesheet [`Error`], this failure can originate in programmatic or mixed-source
/// tokens. It never presents generated serialization coordinates as authored source positions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPropertyValueParseError {
    detail: Box<PropertyValueParseErrorDetail>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PropertyValueParseErrorDetail {
    kind: CssPropertyValueErrorKind,
    origin: CssSerializedOrigin,
}

impl CssPropertyValueParseError {
    /// Returns the structured component or grammar failure.
    #[must_use]
    pub fn kind(&self) -> &CssPropertyValueErrorKind {
        &self.detail.kind
    }

    /// Returns the responsible original token, generated boundary, or EOF origin.
    #[must_use]
    pub fn origin(&self) -> &CssSerializedOrigin {
        &self.detail.origin
    }

    fn from_component(error: CssComponentValueError) -> Self {
        Self {
            detail: Box::new(PropertyValueParseErrorDetail {
                kind: CssPropertyValueErrorKind::Component(error.kind()),
                origin: CssSerializedOrigin::Token(error.origin().clone()),
            }),
        }
    }

    fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        let origin = serialized
            .origin_at(error.position().byte_offset().value())
            .expect("the property parser reports a cursor within its serialized input or at EOF")
            .clone();
        Self {
            detail: Box::new(PropertyValueParseErrorDetail {
                kind: CssPropertyValueErrorKind::Grammar(error.kind().clone()),
                origin,
            }),
        }
    }
}

impl fmt::Display for CssPropertyValueParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS property value {:?}", self.kind())
    }
}

impl std::error::Error for CssPropertyValueParseError {}

/// Checks one property's owned components and constructs an authored declaration occurrence.
///
/// The property identity selects the same grammar as stylesheet declarations. Whole-value
/// CSS-wide keywords and valid substitution-dependent values remain symbolic. Custom names
/// preserve case and allow empty values. Top-level annotations and declaration delimiters are
/// rejected: importance comes only from the separate argument.
///
/// Supplied token origins remain intact. The returned declaration has no parsed property-name
/// position or single parsed declaration-value span; inspect its components for their origins.
/// Serialization preserves token boundaries, checks byte arithmetic, and obeys the immutable
/// component tree's previously validated structural and serialization constraints. This function
/// does not substitute variables, expand shorthands, resolve values, or apply cascade.
pub fn parse_property_value(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_property_value_body(property, &value)?;
    Ok(CssDeclaration::new_constructed(body, importance, value))
}

// Declaration construction and strict expansion reentry share both the checked
// grammar and its original-component error mapping. Reentry retains the original
// occurrence and therefore must not manufacture a replacement declaration.
pub(crate) fn checked_property_value_body(
    property: CssPropertyNameRef<'_>,
    value: &CssComponentValues,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    crate::parser::parse_property_value_body(property, serialized.as_css())
        .map_err(|error| CssPropertyValueParseError::from_grammar(error, &serialized))
}

/// Checks components using an explicit canonical or legacy authored grammar.
///
/// This shares token-boundary preservation, limits, and original-origin mapping
/// with [`parse_property_value`]. The constructed occurrence has programmatic
/// declaration provenance; supplied component origins remain intact.
pub fn parse_property_value_for_grammar(
    grammar: crate::CssPropertyGrammar,
    value: CssComponentValues,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_grammar_value_body(grammar, &value)?;
    Ok(CssDeclaration::new_constructed(body, importance, value))
}
pub(crate) fn checked_grammar_value_body(
    grammar: crate::CssPropertyGrammar,
    value: &CssComponentValues,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    crate::parser::parse_property_value_body_for_grammar(grammar, serialized.as_css())
        .map_err(|error| CssPropertyValueParseError::from_grammar(error, &serialized))
}
