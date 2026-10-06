//! Explicit authored document parsing mode, independent of namespace bindings.
use crate::*;

/// The document mode that selects contextual authored CSS grammar.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CssParserMode {
    /// Ordinary standards grammar, also required by CSS.supports() method syntax.
    #[default]
    Standards,
    /// Document quirks grammar for the exact property contexts defined by CSS.
    Quirks,
}

/// Immutable authored grammar context. Ordinary free parsing functions use Standards.
///
/// This context does not resolve colors, evaluate queries, bind DOM objects, or
/// supply selector namespaces. Declarations retain it for strict pending reentry.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CssParserContext {
    mode: CssParserMode,
}
impl CssParserContext {
    /// Selects one closed authored document mode.
    #[must_use]
    pub const fn new(mode: CssParserMode) -> Self {
        Self { mode }
    }
    /// Returns the immutable selected mode.
    #[must_use]
    pub const fn mode(self) -> CssParserMode {
        self.mode
    }
    /// Parses a stylesheet with this document mode.
    #[must_use]
    pub fn parse_sheet(self, source: &str) -> CssParseReport<CssSheet> {
        crate::parser::parse_sheet_with_context(source, self)
    }
    /// Parses a declaration list with browser recovery under this mode.
    #[must_use]
    pub fn parse_style_attribute(self, source: &str) -> CssParseReport<CssDeclarationList> {
        crate::parser::parse_style_attribute_with_context(source, self)
    }
    /// Parses one complete declaration occurrence.
    #[must_use]
    pub fn parse_declaration(self, source: &str) -> CssParseReport<Option<CssDeclaration>> {
        crate::parser::parse_declaration_with_context(source, self)
    }
    /// Parses one rule using separately supplied namespace bindings.
    #[must_use]
    pub fn parse_rule(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
    ) -> CssParseReport<Option<CssRule>> {
        crate::parser::parse_rule_with_context(source, namespaces, self)
    }
    /// Parses a real-brace style block using separate namespace bindings.
    #[must_use]
    pub fn parse_style_block(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
    ) -> CssParseReport<Option<CssStyleBlock>> {
        crate::parser::parse_style_block_with_context(source, namespaces, self)
    }
    /// Checks owned components with the selected property-name grammar.
    pub fn parse_property_value(
        self,
        property: CssPropertyNameRef<'_>,
        values: CssComponentValues,
        importance: CssImportance,
    ) -> Result<CssDeclaration, CssPropertyValueParseError> {
        crate::property_value::parse_property_value_with_context(property, values, importance, self)
    }
    /// Checks owned components with an explicit authored grammar.
    pub fn parse_property_value_for_grammar(
        self,
        grammar: CssPropertyGrammar,
        values: CssComponentValues,
        importance: CssImportance,
    ) -> Result<CssDeclaration, CssPropertyValueParseError> {
        crate::property_value::parse_property_value_for_grammar_with_context(
            grammar, values, importance, self,
        )
    }
    /// Parses raw value text with a supplied semantic property name.
    #[must_use]
    pub fn parse_property_value_text(
        self,
        source: &str,
        property: CssPropertyNameRef<'_>,
        importance: CssImportance,
    ) -> CssParseReport<Option<CssDeclaration>> {
        crate::parser::parse_property_value_text_with_context(source, property, importance, self)
    }
    /// Parses raw value text with an explicit authored grammar.
    #[must_use]
    pub fn parse_property_value_text_for_grammar(
        self,
        source: &str,
        grammar: CssPropertyGrammar,
        importance: CssImportance,
    ) -> CssParseReport<Option<CssDeclaration>> {
        crate::parser::parse_property_value_text_for_grammar_with_context(
            source, grammar, importance, self,
        )
    }
    /// Checks @supports-rule condition syntax under this document's mode.
    /// Unsupported declarations remain authored tests with no known typed view.
    pub fn parse_supports_condition(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
    ) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
        self.parse_supports_condition_with_limits(
            source,
            namespaces,
            CssComponentValueLimits::default(),
        )
    }
    /// Checks a rule condition with cumulative component resource limits.
    pub fn parse_supports_condition_with_limits(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
        limits: CssComponentValueLimits,
    ) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
        let values = parse_component_values_with_limits(source, limits)?;
        CssSupportsCondition::try_from_components_in_context(values, namespaces, limits, self)
    }
}

/// Parses the condition-text overload of CSS.supports() with mandatory standards grammar.
/// This returns authored syntax and admission metadata, not an evaluated boolean.
/// A complete condition is tried first, then implicit parentheses on grammar mismatch.
/// Introduced punctuation is programmatic; child origins retain the supplied source.
pub fn parse_css_supports_condition(
    source: &str,
) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
    let values = parse_component_values(source)?;
    let namespaces = CssNamespaceContext::default();
    match CssSupportsCondition::try_from_components(values.clone(), &namespaces) {
        Ok(condition) => Ok(condition),
        Err(
            CssSupportsConstructionError::InvalidConditionGrammar { .. }
            | CssSupportsConstructionError::InvalidDeclarationGrammar { .. },
        ) => {
            let block = CssComponentValue::try_block(CssBlockKind::Parenthesis, values)?;
            CssSupportsCondition::try_from_components(
                CssComponentValues::try_new(vec![block])?,
                &namespaces,
            )
        }
        Err(error) => Err(error),
    }
}

/// Parses the property/value overload of CSS.supports() with mandatory standards grammar.
/// Each supplied argument retains its own source snapshot; no combined source is invented.
/// The property is a literal identifier without CSS whitespace or escape processing.
/// Root importance annotations are rejected, while unsupported ordinary values retain
/// an authored declaration test without a known typed view. This does not evaluate support.
pub fn parse_css_supports_declaration(
    property: &str,
    value: &str,
) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
    let property_values = parse_component_values(property)?;
    if let Some(origin) = property_values.first_implicit_origin() {
        return Err(CssSupportsConstructionError::RecoveredInput {
            origin: origin.clone(),
        });
    }
    if !matches!(property_values.items(), [component]
        if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name == property))
    {
        return Err(CssSupportsConstructionError::InvalidDeclarationGrammar {
            origin: property_values
                .items()
                .first()
                .map_or(CssValueOrigin::Programmatic, |component| {
                    component.origin().clone()
                }),
        });
    }
    let value = parse_component_values(value)?;
    if let Some(origin) = value.first_implicit_origin() {
        return Err(CssSupportsConstructionError::RecoveredInput {
            origin: origin.clone(),
        });
    }
    if let Some(component) = value.items().iter().find(|component| {
        matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Delim('!'))
        )
    }) {
        return Err(CssSupportsConstructionError::InvalidDeclarationGrammar {
            origin: component.origin().clone(),
        });
    }
    let mut items = property_values.items().to_vec();
    items.push(CssComponentValue::try_token(":")?);
    items.extend_from_slice(value.items());
    CssSupportsDeclaration::try_from_components(CssComponentValues::try_new(items)?)
}
