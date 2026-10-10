//! Explicit authored document mode and glyph definition, independent of namespace bindings.
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

/// Immutable authored grammar context. Ordinary free parsing functions use Standards
/// and the finite Writing Modes glyph alias. Named SVG selection is independent of mode.
///
/// This context does not resolve colors, evaluate queries, bind DOM objects, or
/// supply selector namespaces. Declarations retain it for strict pending reentry.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CssParserContext {
    mode: CssParserMode,
    glyph: GlyphDefinition,
}
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
enum GlyphDefinition {
    #[default]
    WritingModes,
    Svg,
}
impl CssParserContext {
    /// Selects one closed authored document mode.
    #[must_use]
    pub const fn new(mode: CssParserMode) -> Self {
        Self {
            mode,
            glyph: GlyphDefinition::WritingModes,
        }
    }
    /// Selects the independent frozen SVG authored definition in property-name lookup.
    /// Explicit finite grammar handles retain their finite meaning. This is idempotent.
    #[must_use]
    pub const fn with_svg_glyph_orientation_vertical(mut self) -> Self {
        self.glyph = GlyphDefinition::Svg;
        self
    }
    pub(crate) fn selects_svg_glyph(self, name: &str) -> bool {
        self.glyph == GlyphDefinition::Svg
            && name.eq_ignore_ascii_case("glyph-orientation-vertical")
    }
    const fn standards(mut self) -> Self {
        self.mode = CssParserMode::Standards;
        self
    }
    /// Parses a fixed SVG presentation attribute value with Normal importance.
    /// This retains document mode but uses attribute admission, independently of lookup selection.
    #[must_use]
    pub fn parse_svg_glyph_orientation_vertical_attribute_value(
        self,
        source: &str,
    ) -> CssParseReport<Option<CssDeclaration>> {
        crate::parser::parse_svg_glyph_attribute_value(source, self)
    }
    /// Checks a fixed SVG presentation attribute's components; markup binding is downstream.
    pub fn parse_svg_glyph_orientation_vertical_attribute_components(
        self,
        values: CssComponentValues,
    ) -> Result<CssDeclaration, CssPropertyValueParseError> {
        let body = crate::property_value::checked_svg_glyph_value_body(
            crate::svg_glyph::SvgGlyphAdmission::attribute(self),
            &values,
            self,
        )?;
        Ok(CssDeclaration::new_constructed(
            self,
            body,
            CssImportance::Normal,
            values,
        ))
    }
    /// Parses CSS.supports() condition text with Standards admission and this definition choice.
    pub fn parse_css_supports_condition(
        self,
        source: &str,
    ) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
        css_supports_condition(source, self.standards())
    }
    /// Parses the literal property/value overload with Standards admission and this definition choice.
    pub fn parse_css_supports_declaration(
        self,
        property: &str,
        value: &str,
    ) -> Result<CssSupportsDeclaration, CssSupportsConstructionError> {
        css_supports_declaration(property, value, self.standards())
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
    /// Parses raw declaration-list text with CSS Syntax unit recovery under this mode.
    #[must_use]
    pub fn parse_declaration_list_text(self, source: &str) -> CssParseReport<CssDeclarationList> {
        crate::parser::parse_declaration_list_text_with_context(source, self)
    }
    /// Parses raw current block contents with this declaration grammar context.
    #[must_use]
    pub fn parse_declaration_block_contents(
        self,
        source: &str,
    ) -> CssParseReport<CssDeclarationList> {
        self.parse_declaration_block_contents_with_limits(
            source,
            CssComponentValueLimits::default(),
        )
    }
    /// Parses raw block contents with cumulative original-input limits.
    #[must_use]
    pub fn parse_declaration_block_contents_with_limits(
        self,
        source: &str,
        limits: CssComponentValueLimits,
    ) -> CssParseReport<CssDeclarationList> {
        crate::parser::parse_declaration_block_contents_with_context(source, limits, self)
    }
    /// Parses raw Page declaration contents with this document-mode context.
    /// Resource failure is `None`; recovered empty contents are `Some(empty)`.
    #[must_use]
    pub fn parse_page_declaration_block_contents(
        self,
        source: &str,
    ) -> CssParseReport<Option<crate::CssPageDeclarationBlock>> {
        self.parse_page_declaration_block_contents_with_limits(
            source,
            CssComponentValueLimits::default(),
        )
    }
    /// Parses raw Page contents with cumulative original-input limits.
    #[must_use]
    pub fn parse_page_declaration_block_contents_with_limits(
        self,
        source: &str,
        limits: CssComponentValueLimits,
    ) -> CssParseReport<Option<crate::CssPageDeclarationBlock>> {
        crate::parser::parse_page_declaration_block_contents_with_context(source, limits, self)
    }
    /// Parses one complete detached margin child in an actual Page destination.
    #[must_use]
    pub fn parse_page_margin_rule(
        self,
        source: &str,
    ) -> CssParseReport<Option<crate::CssMarginRule>> {
        self.parse_page_margin_rule_with_limits(source, CssComponentValueLimits::default())
    }
    /// Uses cumulative original-input limits and this document-mode context.
    #[must_use]
    pub fn parse_page_margin_rule_with_limits(
        self,
        source: &str,
        limits: CssComponentValueLimits,
    ) -> CssParseReport<Option<crate::CssMarginRule>> {
        crate::parser::parse_page_margin_rule_with_context(source, limits, self)
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
    /// Parses a genuine keyframes body with this context on its declarations.
    ///
    /// Invalid children recover independently; no animation name is invented.
    /// Source boundaries and clean validation match [`parse_keyframes_block`].
    #[must_use]
    pub fn parse_keyframes_block(
        self,
        source: &str,
    ) -> CssParseReport<Option<CssBlockFragment<Vec<CssKeyframeBlock>>>> {
        crate::parser::parse_keyframes_block_with_context(source, self)
    }
    /// Parses genuine keyframe declarations with this document grammar context.
    ///
    /// Local declaration recovery and fatal structural errors match
    /// [`parse_keyframe_declaration_block`]. Retained occurrences preserve `self`.
    #[must_use]
    pub fn parse_keyframe_declaration_block(
        self,
        source: &str,
    ) -> CssParseReport<Option<CssBlockFragment<CssKeyframeDeclarationList>>> {
        crate::parser::parse_keyframe_declaration_block_with_context(source, self)
    }
    /// Parses a genuine Page body with this context and the selected Page domain.
    ///
    /// Descriptor/property admission and ordered margin children follow
    /// [`parse_page_block`]; document mode does not bypass domain checks.
    #[must_use]
    pub fn parse_page_block(
        self,
        source: &str,
    ) -> CssParseReport<Option<CssBlockFragment<crate::CssPageBody>>> {
        crate::parser::parse_page_block_with_context(source, self)
    }
    /// Parses one genuine margin declaration block using this property context.
    #[must_use]
    pub fn parse_margin_block(
        self,
        source: &str,
    ) -> CssParseReport<Option<CssBlockFragment<CssMarginDeclarationBlock>>> {
        crate::parser::parse_margin_block_with_context(source, self)
    }
    /// Parses an ordinary inner rule block with this declaration grammar context.
    ///
    /// Namespace bindings remain separate; boundaries and recovery match
    /// [`parse_group_block`]. No enclosing style ancestry is supplied.
    #[must_use]
    pub fn parse_group_block(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
    ) -> CssParseReport<Option<CssBlockFragment<CssRuleList>>> {
        crate::parser::parse_group_block_with_context(source, namespaces, self)
    }
    /// Parses a scope body with this context and explicit actual style ancestry.
    ///
    /// Direct declarations and children retain `self`; source and placement
    /// rules match [`parse_scope_block`]. No scope prelude is invented.
    #[must_use]
    pub fn parse_scope_block(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
        ancestry: CssStyleAncestor,
    ) -> CssParseReport<Option<CssBlockFragment<CssScopedRuleList>>> {
        crate::parser::parse_scope_block_with_context(source, namespaces, ancestry, self)
    }
    /// Parses an ordinary group inside a scope with this declaration context.
    ///
    /// Explicit ancestry, namespace binding and Page placement follow
    /// [`parse_scoped_group_block`]. Actual declarations retain `self`.
    #[must_use]
    pub fn parse_scoped_group_block(
        self,
        source: &str,
        namespaces: &CssNamespaceContext,
        ancestry: CssStyleAncestor,
    ) -> CssParseReport<Option<CssBlockFragment<CssScopedRuleList>>> {
        crate::parser::parse_scoped_group_block_with_context(source, namespaces, ancestry, self)
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
    /// Checks detached Page components with this document mode and definition choice.
    /// Page admission and strict pending reentry follow [`parse_page_property_value`].
    pub fn parse_page_property_value(
        self,
        property: CssPropertyNameRef<'_>,
        values: CssComponentValues,
        importance: CssImportance,
    ) -> Result<CssDeclaration, CssPropertyValueParseError> {
        crate::property_value::parse_page_property_value_with_context(
            property, values, importance, self,
        )
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
    css_supports_condition(source, CssParserContext::default())
}
fn css_supports_condition(
    source: &str,
    context: CssParserContext,
) -> Result<CssSupportsCondition, CssSupportsConstructionError> {
    let values = parse_component_values(source)?;
    let namespaces = CssNamespaceContext::default();
    match CssSupportsCondition::try_from_components_in_context(
        values.clone(),
        &namespaces,
        CssComponentValueLimits::default(),
        context,
    ) {
        Ok(condition) => Ok(condition),
        Err(
            CssSupportsConstructionError::InvalidConditionGrammar { .. }
            | CssSupportsConstructionError::InvalidDeclarationGrammar { .. },
        ) => {
            let block = CssComponentValue::try_block(CssBlockKind::Parenthesis, values)?;
            CssSupportsCondition::try_from_components_in_context(
                CssComponentValues::try_new(vec![block])?,
                &namespaces,
                CssComponentValueLimits::default(),
                context,
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
    css_supports_declaration(property, value, CssParserContext::default())
}
fn css_supports_declaration(
    property: &str,
    value: &str,
    context: CssParserContext,
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
    CssSupportsDeclaration::try_from_components_in_context(
        CssComponentValues::try_new(items)?,
        context,
    )
}
