use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser,
};

use super::parse_descriptor_boundary;
use super::property_input::{NativeValueRange, adapt_native};
use super::recovery::RecoveryState;
use crate::error::{
    CssFeatureId, Error, basic, descriptor_name_error, from_parse_error, invalid_component_value,
    invalid_syntax, unexpected_at, with_at_rule_prelude_context, with_descriptor_context,
};
use crate::font_palette_values::CssFontPaletteDescriptorData;
use crate::numeric::NumericInputContext;
use crate::{
    CssAuthoredDeclarationValue, CssComponentValues, CssFontFaceFamily, CssFontPaletteBase,
    CssFontPaletteConstructionError, CssFontPaletteDescriptor, CssFontPaletteDescriptorKind,
    CssFontPaletteDescriptorValue, CssFontPaletteIndex, CssFontPaletteName, CssFontPaletteOverride,
    CssFontPaletteValuesRule, CssRecoveryDiagnostic, CssSerializedValue, CssSourcePosition,
    CssSubstitutionDependentValue,
};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("later.rule.font-palette-values")];
pub(super) static IMPLEMENTED_DESCRIPTORS: &[CssFeatureId] = &[
    CssFeatureId::new("later.descriptor.font-palette-values.font-family"),
    CssFeatureId::new("later.descriptor.font-palette-values.base-palette"),
    CssFeatureId::new("later.descriptor.font-palette-values.override-colors"),
];

pub(super) fn parse_name<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<CssFontPaletteName, ParseError<'i, Error>> {
    recovery.check_specialized_components(source, input, "later.rule.font-palette-values")?;
    (|| {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        input.expect_exhausted().map_err(basic)?;
        CssFontPaletteName::try_new(&ident).map_err(|_| invalid_syntax(location))
    })()
    .map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "font-palette-values",
            "later.rule.font-palette-values",
            "one dashed identifier",
        )
    })
}

pub(super) fn parse_rule<'i>(
    source: &'i str,
    name: CssFontPaletteName,
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssFontPaletteValuesRule, ParseError<'i, Error>> {
    let descriptors = parse_body(source, input, diagnostics, recovery);
    CssFontPaletteValuesRule::try_new(name, descriptors)
        .map(|rule| rule.with_position(position(start)))
        .map_err(|_| invalid_syntax(start.source_location()))
}

pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Vec<CssFontPaletteDescriptor> {
    let mut parser = BodyParser {
        source,
        recovery,
        diagnostics: Vec::new(),
    };
    let descriptors =
        super::descriptor_body::parse(input, &mut parser, "later.rule.font-palette-values");
    diagnostics.extend(parser.diagnostics);
    descriptors
}

fn position(start: &ParserState) -> CssSourcePosition {
    CssSourcePosition::from_cssparser(start.position(), start.source_location())
}

struct BodyParser<'i> {
    source: &'i str,
    recovery: RecoveryState,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}

impl<'i> super::descriptor_body::Receiver<'i, CssFontPaletteDescriptor> for BodyParser<'i> {
    fn recovery_context(&mut self) -> super::descriptor_body::RecoveryContext<'_, 'i> {
        super::descriptor_body::RecoveryContext {
            source: self.source,
            recovery: &self.recovery,
            diagnostics: &mut self.diagnostics,
        }
    }
}

impl<'i> AtRuleParser<'i> for BodyParser<'i> {
    type Prelude = ();
    type AtRule = CssFontPaletteDescriptor;
    type Error = Error;
}

impl<'i> QualifiedRuleParser<'i> for BodyParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssFontPaletteDescriptor;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssFontPaletteDescriptor, Error> for BodyParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    // Syntax 3 (2026 CRD) §5.5.5 reconsumes a failed declaration, and any
    // other non-at-rule unit, as a nested qualified rule with a `;` stop token.
    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> DeclarationParser<'i> for BodyParser<'i> {
    type Declaration = CssFontPaletteDescriptor;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        start: &ParserState,
    ) -> Result<Self::Declaration, ParseError<'i, Error>> {
        let implicit =
            self.recovery
                .check_component_values(self.source, input, "css.descriptor")?;
        let kind = CssFontPaletteDescriptorKind::from_css_name(&name).ok_or_else(|| {
            descriptor_name_error(start.source_location(), "font-palette-values", &name)
        })?;
        let value = parse_descriptor_value_from_parser(input, kind, &self.recovery)
            .map_err(|error| with_descriptor_context(error, "font-palette-values", &name))?;
        self.recovery.retain_component_closures(implicit);
        Ok(CssFontPaletteDescriptor::new(value).with_position(position(start)))
    }
}

pub(super) fn parse_descriptor_value_from_parser<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontPaletteDescriptorKind,
    recovery: &RecoveryState,
) -> Result<CssFontPaletteDescriptorValue, ParseError<'i, Error>> {
    super::descriptor_values::validate_root(input, "font-palette-values", kind.css_name())?;
    let numeric = NumericInputContext::parsed(recovery.source_snapshot());
    parse_descriptor_boundary(input, "font-palette-values", kind.css_name(), |input| {
        let beginning = input.state();
        let components = CssComponentValues::collect_from_parser(input, recovery.source_snapshot())
            .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
        input.reset(&beginning);
        let data = parse_value_data(input, kind, &components, &numeric)?;
        input.expect_exhausted().map_err(basic)?;
        Ok(CssFontPaletteDescriptorValue::from_parts(
            kind, components, data,
        ))
    })
}

/// The component-native constructor uses the same grammar with strict numeric admission.
pub(crate) fn construct_descriptor_value(
    kind: CssFontPaletteDescriptorKind,
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<CssFontPaletteDescriptorData, Error> {
    let working_source = crate::tokenization::prepare(serialized.as_css());
    let mut parser_input = cssparser::ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let numeric = NumericInputContext::components(components, serialized);
    let result = (|| {
        super::descriptor_values::validate_root(
            &mut input,
            "font-palette-values",
            kind.css_name(),
        )?;
        parse_descriptor_boundary(
            &mut input,
            "font-palette-values",
            kind.css_name(),
            |input| {
                let data = parse_value_data(input, kind, components, &numeric)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(data)
            },
        )
    })();
    result.map_err(|error| {
        from_parse_error(
            serialized.as_css(),
            crate::error::with_descriptor_value_context(
                error,
                "font-palette-values",
                kind.css_name(),
            ),
        )
    })
}

fn parse_value_data<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontPaletteDescriptorKind,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteDescriptorData, ParseError<'i, Error>> {
    let qualifying =
        super::variables::descriptor_substitution_qualifies(components.items(), numeric)
            .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    if qualifying {
        let start = input.position();
        super::descriptor_values::consume_remaining_components(input)?;
        return Ok(CssFontPaletteDescriptorData::Pending(
            CssSubstitutionDependentValue::new(CssAuthoredDeclarationValue::new(
                input.slice_from(start),
            )),
        ));
    }
    Ok(match kind {
        CssFontPaletteDescriptorKind::FontFamily => {
            CssFontPaletteDescriptorData::FontFamily(parse_families(input, numeric)?)
        }
        CssFontPaletteDescriptorKind::BasePalette => {
            CssFontPaletteDescriptorData::BasePalette(parse_base(input, numeric)?)
        }
        CssFontPaletteDescriptorKind::OverrideColors => {
            CssFontPaletteDescriptorData::OverrideColors(parse_overrides(input, numeric)?)
        }
    })
}

fn parse_families<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<Vec<CssFontFaceFamily>, ParseError<'i, Error>> {
    input.parse_comma_separated(|input| {
        let family = adapt_native(input, numeric, NativeValueRange::Remaining, |input, _| {
            super::typography::parse_non_generic_font_family_name(input)
        })?;
        input.expect_exhausted().map_err(basic)?;
        CssFontFaceFamily::try_new(family.as_str())
            .ok_or_else(|| invalid_syntax(input.current_source_location()))
    })
}

fn parse_base<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteBase, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if ident.eq_ignore_ascii_case("light") {
            return Ok(CssFontPaletteBase::Light);
        }
        if ident.eq_ignore_ascii_case("dark") {
            return Ok(CssFontPaletteBase::Dark);
        }
        input.reset(&state);
    }
    parse_index(input, numeric).map(CssFontPaletteBase::Index)
}

fn parse_overrides<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<Vec<CssFontPaletteOverride>, ParseError<'i, Error>> {
    input.parse_comma_separated(|input| {
        let index = parse_index(input, numeric)?;
        let location = input.current_source_location();
        let color = adapt_native(
            input,
            numeric,
            NativeValueRange::Component,
            super::color::parse_color,
        )?;
        let pair = CssFontPaletteOverride::try_new(index, color).map_err(|error| match error {
            CssFontPaletteConstructionError::ContextualColor(_) => unexpected_at(location),
            _ => invalid_syntax(location),
        })?;
        input.expect_exhausted().map_err(basic)?;
        Ok(pair)
    })
}

fn parse_index<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteIndex, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = adapt_native(
        input,
        numeric,
        NativeValueRange::Component,
        super::values::parse_integer_value,
    )?;
    CssFontPaletteIndex::try_new(value).map_err(|_| unexpected_at(location))
}
