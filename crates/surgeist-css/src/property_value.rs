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

    pub(crate) fn from_grammar(error: Error, serialized: &CssSerializedValue) -> Self {
        if let ErrorKind::InvalidComponentValue(detail) = error.kind()
            && crate::error::is_component_resource_error(detail)
        {
            return Self::from_component((**detail).clone());
        }
        let origin = error.component_input_origin(serialized);
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

fn is_time_property(property: crate::CssKnownProperty) -> bool {
    matches!(
        property,
        crate::CssKnownProperty::TransitionDuration
            | crate::CssKnownProperty::TransitionDelay
            | crate::CssKnownProperty::AnimationDuration
            | crate::CssKnownProperty::AnimationDelay
            | crate::CssKnownProperty::Transition
            | crate::CssKnownProperty::Animation
    )
}

fn recovered_component_error(value: &CssComponentValues) -> Option<CssPropertyValueParseError> {
    let origin = value.first_implicit_origin()?;
    Some(CssPropertyValueParseError {
        detail: Box::new(PropertyValueParseErrorDetail {
            kind: CssPropertyValueErrorKind::Grammar(crate::error::implicit_eof("").kind().clone()),
            origin: CssSerializedOrigin::End(Some(origin.clone())),
        }),
    })
}

/// Checks one property's owned components and constructs an authored declaration occurrence.
///
/// The property identity selects the same grammar as stylesheet declarations. Whole-value
/// CSS-wide keywords and valid substitution-dependent values remain symbolic. Custom names
/// preserve case and allow empty values. Known grammars reject implicit closures in the original
/// components, including CSS-wide and substitution-dependent values. Top-level annotations
/// and declaration delimiters are rejected: importance comes only from the separate argument.
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
    parse_property_value_with_context(
        property,
        value,
        importance,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_property_value_with_context(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_property_value_body(property, &value, parser_context)?;
    Ok(CssDeclaration::new_constructed(
        parser_context,
        body,
        importance,
        value,
    ))
}

/// Checks detached components as one authored Page declaration.
///
/// Admits represented Page/margin-applicable shared properties and custom values.
/// CSS-wide, calculation, length-percentage and substitution grammar remains
/// symbolic, including font-relative operands. Page physical margins exclude the
/// crate's logical-margin extension and nonzero unitless values. Custom tokens are
/// retained without margin restrictions or variable lookup.
///
/// The occurrence retains Page semantic context independently of document mode,
/// so strict pending reentry cannot silently use ordinary element grammar.
/// Component origins and supplied importance are preserved; no name/source span,
/// page selector, cascade environment or enclosing rule is fabricated.
///
/// ```
/// use surgeist_css::{
///     CssExpansion, CssImportance, CssKnownProperty, CssPropertyNameRef,
///     expand_declaration, parse_component_values, parse_page_property_value,
/// };
/// let margin = parse_page_property_value(
///     CssPropertyNameRef::Known(CssKnownProperty::Margin),
///     parse_component_values("var(--m, 1px)").unwrap(),
///     CssImportance::Normal,
/// ).unwrap();
/// let CssExpansion::Pending(pending) = expand_declaration(&margin).unwrap() else {
///     panic!("pending authored Page margin");
/// };
/// assert!(pending.reenter(parse_component_values("1em").unwrap()).is_ok());
/// assert!(pending.reenter(parse_component_values("calc(1Q + 2%)").unwrap()).is_ok());
/// ```
pub fn parse_page_property_value(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    parse_page_property_value_with_context(
        property,
        value,
        importance,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_page_property_value_with_context(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_page_property_value_body(property, &value, parser_context)?;
    Ok(CssDeclaration::new_constructed_in_context(
        parser_context,
        crate::syntax::DeclarationContext::Page,
        body,
        importance,
        value,
    ))
}

pub(crate) fn checked_page_property_value_body(
    property: CssPropertyNameRef<'_>,
    value: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    let body = checked_property_value_body(property, value, parser_context)?;
    let admitted = match property {
        CssPropertyNameRef::Known(property) => crate::parser::is_page_margin_property(property),
        CssPropertyNameRef::Custom(_) => true,
        CssPropertyNameRef::SvgGlyphOrientationVertical => false,
    };
    let violation = crate::parser::page_declaration_violation(&body, value);
    if !admitted || violation.is_some() {
        let component = violation
            .as_ref()
            .and_then(|violation| violation.component())
            .or_else(|| crate::parser::first_page_component(value));
        let name = match property {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            CssPropertyNameRef::SvgGlyphOrientationVertical => "glyph-orientation-vertical",
        };
        let error = component.map_or_else(
            || {
                crate::error::unexpected_end_at(
                    cssparser::SourceLocation { line: 0, column: 1 },
                    "an admitted Page declaration",
                )
            },
            |component| {
                // Only the typed kind is retained below. The public diagnostic
                // origin comes directly from this component, including when it
                // has no parsed source position.
                let position = match component.origin() {
                    crate::CssValueOrigin::Parsed(origin) => origin.span().start(),
                    _ => crate::CssSourcePosition::from_byte_offset_in("", 0),
                };
                crate::error::unexpected_component_value_at(
                    component,
                    "an admitted Page declaration",
                    position,
                )
            },
        );
        let error = crate::error::with_property_context(error, name);
        let cssparser::ParseErrorKind::Custom(error) = error.kind else {
            unreachable!("Page context errors use the typed property grammar error")
        };
        return Err(CssPropertyValueParseError {
            detail: Box::new(PropertyValueParseErrorDetail {
                kind: CssPropertyValueErrorKind::Grammar(error.kind().clone()),
                origin: component.map_or(CssSerializedOrigin::End(None), |component| {
                    CssSerializedOrigin::Token(component.origin().clone())
                }),
            }),
        });
    }
    Ok(body)
}

// Declaration construction and strict expansion reentry share both the checked
// grammar and its original-component error mapping. Reentry retains the original
// occurrence and therefore must not manufacture a replacement declaration.

pub(crate) fn checked_property_value_body(
    property: CssPropertyNameRef<'_>,
    value: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    if let CssPropertyNameRef::Known(known) = property
        && is_time_property(known)
        && let Some(error) = recovered_component_error(value)
    {
        return Err(error);
    }
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    if let CssPropertyNameRef::Known(known) = property
        && !is_time_property(known)
        && let Some(error) = recovered_component_error(value)
    {
        return Err(error);
    }
    if matches!(property, CssPropertyNameRef::SvgGlyphOrientationVertical)
        && let Some(error) = recovered_component_error(value)
    {
        return Err(error);
    }
    crate::parser::parse_property_value_body(
        property,
        serialized.as_css(),
        &crate::numeric::NumericInputContext::components(value, &serialized),
        parser_context,
    )
    .map_err(|error| CssPropertyValueParseError::from_grammar(error, &serialized))
}

pub(crate) fn checked_svg_glyph_value_body(
    admission: crate::svg_glyph::SvgGlyphAdmission,
    value: &CssComponentValues,
    context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    if let Some(error) = recovered_component_error(value) {
        return Err(error);
    }
    crate::parser::parse_svg_glyph_value_body(
        admission,
        serialized.as_css(),
        &crate::numeric::NumericInputContext::components(value, &serialized),
        context,
    )
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
    parse_property_value_for_grammar_with_context(
        grammar,
        value,
        importance,
        crate::CssParserContext::default(),
    )
}

pub(crate) fn parse_property_value_for_grammar_with_context(
    grammar: crate::CssPropertyGrammar,
    value: CssComponentValues,
    importance: CssImportance,
    parser_context: crate::CssParserContext,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let body = checked_grammar_value_body(grammar, &value, parser_context)?;
    Ok(CssDeclaration::new_constructed(
        parser_context,
        body,
        importance,
        value,
    ))
}

pub(crate) fn checked_grammar_value_body(
    grammar: crate::CssPropertyGrammar,
    value: &CssComponentValues,
    parser_context: crate::CssParserContext,
) -> Result<crate::CssDeclarationBody, CssPropertyValueParseError> {
    if is_time_property(grammar.target_property())
        && let Some(error) = recovered_component_error(value)
    {
        return Err(error);
    }
    let serialized = value
        .serialize()
        .map_err(CssPropertyValueParseError::from_component)?;
    if !is_time_property(grammar.target_property())
        && let Some(error) = recovered_component_error(value)
    {
        return Err(error);
    }
    crate::parser::parse_property_value_body_for_grammar(
        grammar,
        serialized.as_css(),
        &crate::numeric::NumericInputContext::components(value, &serialized),
        parser_context,
    )
    .map_err(|error| CssPropertyValueParseError::from_grammar(error, &serialized))
}

/// Constructs one checked margin-context property value with genuine supplied component origins.
/// Page-only descriptors are parsed by `parse_page_descriptor_value`; ordinary property grammar
/// and applicability share the Page producer boundary without fabricated rule/name coordinates.
pub fn parse_margin_property_value(
    property: CssPropertyNameRef<'_>,
    value: CssComponentValues,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    parse_page_property_value(property, value, importance)
}
