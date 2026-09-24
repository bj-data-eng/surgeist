use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{consume_failed_rule_block, parse_descriptor_boundary, structural_rule_diagnostic};
use crate::error::{
    CssFeatureId, Error, basic, descriptor_name_error, from_parse_error, invalid_component_value,
    invalid_syntax, unsupported_value_at, with_at_rule_prelude_context, with_descriptor_context,
};
use crate::font_palette_values::CssFontPaletteDescriptorData;
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::{
    CssAuthoredDeclarationValue, CssComponentValueRef, CssComponentValues, CssFontFaceFamily,
    CssFontPaletteBase, CssFontPaletteConstructionError, CssFontPaletteDescriptor,
    CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue, CssFontPaletteIndex,
    CssFontPaletteName, CssFontPaletteOverride, CssFontPaletteValuesRule, CssIntegerCalculation,
    CssIntegerValue, CssRecoveryAction, CssRecoveryDiagnostic, CssSerializedValue,
    CssSourcePosition, CssSubstitutionDependentValue, CssValueTokenRef,
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
        CssFontPaletteName::try_new(&ident)
            .map_err(|error| invalid_syntax(location, error.to_string()))
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
    let mut parser = BodyParser {
        source,
        recovery,
        diagnostics: Vec::new(),
    };
    let mut descriptors = Vec::new();
    let mut previous_end = input.position().byte_index();
    let mut items = RuleBodyParser::new(input, &mut parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        let Some(item) = items.next() else {
            break;
        };
        let failed_error = item.as_ref().err().and_then(|_| {
            consume_failed_rule_block(
                items.parser.source,
                items.input,
                true,
                &items.parser.recovery,
                "later.rule.font-palette-values",
            )
            .1
        });
        let outcome = progress.finish(items.input, item.is_ok());
        let end = items.input.position().byte_index();
        match item {
            Ok(descriptor) => descriptors.push(descriptor),
            Err((error, unit)) => {
                let action = if unit.trim_start().starts_with('@') {
                    CssRecoveryAction::DropAtRule
                } else {
                    CssRecoveryAction::DropDescriptor
                };
                if let Some(diagnostic) = structural_rule_diagnostic(
                    items.parser.source,
                    failed_error.unwrap_or(error),
                    unit,
                    previous_end,
                    end,
                    action,
                ) {
                    items.parser.diagnostics.push(diagnostic);
                }
            }
        }
        previous_end = end;
        if outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    diagnostics.extend(parser.diagnostics);
    CssFontPaletteValuesRule::try_new(name, descriptors)
        .map(|rule| rule.with_position(position(start)))
        .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))
}

fn position(start: &ParserState) -> CssSourcePosition {
    CssSourcePosition::from_cssparser(start.position(), start.source_location())
}

struct BodyParser<'i> {
    source: &'i str,
    recovery: RecoveryState,
    diagnostics: Vec<CssRecoveryDiagnostic>,
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

    fn parse_qualified(&self) -> bool {
        false
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
    for (index, component) in components.items().iter().enumerate() {
        let invalid_root = match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Semicolon) => true,
            CssComponentValueRef::Block(block) => block.kind() == crate::CssBlockKind::CurlyBracket,
            _ => false,
        };
        if invalid_root {
            let offset = serialized
                .component_offset_for_path(&[index])
                .expect("serialized root component has an offset");
            let mut prefix_input = cssparser::ParserInput::new(&serialized.as_css()[..offset]);
            let mut prefix = Parser::new(&mut prefix_input);
            while prefix.next_including_whitespace_and_comments().is_ok() {}
            return Err(from_parse_error(
                serialized.as_css(),
                invalid_syntax(
                    prefix.current_source_location(),
                    "root descriptor delimiter",
                ),
            ));
        }
    }
    let mut parser_input = cssparser::ParserInput::new(serialized.as_css());
    let mut input = Parser::new(&mut parser_input);
    let numeric = NumericInputContext::components(components, serialized);
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
    .map_err(|error| from_parse_error(serialized.as_css(), error))
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
        consume_remaining_components(input)?;
        return Ok(CssFontPaletteDescriptorData::Pending(
            CssSubstitutionDependentValue::new(CssAuthoredDeclarationValue::new(
                input.slice_from(start),
            )),
        ));
    }
    Ok(match kind {
        CssFontPaletteDescriptorKind::FontFamily => {
            CssFontPaletteDescriptorData::FontFamily(parse_families(input)?)
        }
        CssFontPaletteDescriptorKind::BasePalette => {
            CssFontPaletteDescriptorData::BasePalette(parse_base(input, numeric)?)
        }
        CssFontPaletteDescriptorKind::OverrideColors => {
            CssFontPaletteDescriptorData::OverrideColors(parse_overrides(input, numeric)?)
        }
    })
}

fn consume_remaining_components<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<(), ParseError<'i, Error>> {
    loop {
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, cssparser::BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(basic(error)),
        };
        if matches!(
            token,
            Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock
        ) {
            input.parse_nested_block(consume_remaining_components)?;
        }
    }
    Ok(())
}

fn parse_families<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<CssFontFaceFamily>, ParseError<'i, Error>> {
    input.parse_comma_separated(|input| {
        let family = super::typography::parse_non_generic_font_family_name(input)?;
        input.expect_exhausted().map_err(basic)?;
        CssFontFaceFamily::try_new(family.as_str()).ok_or_else(|| {
            invalid_syntax(
                input.current_source_location(),
                "invalid decoded font family",
            )
        })
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
        let (color, _) = super::values::parse_color(input, numeric)?.into_parts();
        let pair = CssFontPaletteOverride::try_new(index, color).map_err(|error| match error {
            CssFontPaletteConstructionError::ContextualColor(_) => {
                unsupported_value_at(location, None, "override color must be absolute")
            }
            _ => invalid_syntax(location, error.to_string()),
        })?;
        input.expect_exhausted().map_err(basic)?;
        Ok(pair)
    })
}

fn parse_index<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFontPaletteIndex, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    let value = match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&state);
            super::layout::parse_current_integer_literal(input, numeric)?
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression = super::values::parse_numeric_function(
                input,
                &state,
                numeric,
                CalculationRoot::Integer,
            )?;
            CssIntegerValue::Calculation(CssIntegerCalculation::from_expression(expression))
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    CssFontPaletteIndex::try_new(value)
        .map_err(|error| unsupported_value_at(location, None, error.to_string()))
}
