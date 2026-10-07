use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{consume_failed_rule_block, parse_descriptor_boundary, structural_rule_diagnostic};
use crate::color_profile::CssColorProfileDescriptorData;
use crate::error::{
    CssFeatureId, Error, basic, descriptor_name_error, from_parse_error, invalid_component_value,
    invalid_syntax, with_at_rule_prelude_context, with_descriptor_context,
};
use crate::numeric::NumericInputContext;
use crate::{
    CssAuthoredDeclarationValue, CssColorProfileComponentName, CssColorProfileDescriptor,
    CssColorProfileDescriptorKind, CssColorProfileDescriptorValue, CssColorProfileName,
    CssColorProfileRenderingIntent, CssColorProfileRule, CssColorProfileRuleName,
    CssComponentValues, CssRecoveryAction, CssRecoveryDiagnostic, CssSerializedValue,
    CssSourcePosition, CssSubstitutionDependentValue,
};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("interop.rule.color-profile")];
pub(super) static IMPLEMENTED_DESCRIPTORS: &[CssFeatureId] = &[
    CssFeatureId::new("interop.descriptor.color-profile.src"),
    CssFeatureId::new("interop.descriptor.color-profile.rendering-intent"),
    CssFeatureId::new("interop.descriptor.color-profile.components"),
];

pub(super) fn parse_name<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<CssColorProfileRuleName, ParseError<'i, Error>> {
    recovery.check_specialized_components(source, input, "interop.rule.color-profile")?;
    (|| {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        input.expect_exhausted().map_err(basic)?;
        if ident.eq_ignore_ascii_case("device-cmyk") {
            Ok(CssColorProfileRuleName::DeviceCmyk)
        } else {
            CssColorProfileName::try_new(ident.as_ref())
                .map(CssColorProfileRuleName::Custom)
                .ok_or_else(|| {
                    invalid_syntax(location, "expected a dashed identifier or device-cmyk")
                })
        }
    })()
    .map_err(|error| {
        with_at_rule_prelude_context(
            error,
            "color-profile",
            "interop.rule.color-profile",
            "one dashed identifier or device-cmyk",
        )
    })
}

pub(super) fn parse_rule<'i>(
    source: &'i str,
    name: CssColorProfileRuleName,
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssColorProfileRule, ParseError<'i, Error>> {
    let descriptors = parse_body(source, input, diagnostics, recovery);
    Ok(CssColorProfileRule::new(name, descriptors).with_position(position(start)))
}

pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Vec<CssColorProfileDescriptor> {
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
                "interop.rule.color-profile",
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

impl<'i> AtRuleParser<'i> for BodyParser<'i> {
    type Prelude = ();
    type AtRule = CssColorProfileDescriptor;
    type Error = Error;
}

impl<'i> QualifiedRuleParser<'i> for BodyParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssColorProfileDescriptor;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssColorProfileDescriptor, Error> for BodyParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        false
    }
}

impl<'i> DeclarationParser<'i> for BodyParser<'i> {
    type Declaration = CssColorProfileDescriptor;
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
        let kind = CssColorProfileDescriptorKind::from_css_name(&name).ok_or_else(|| {
            descriptor_name_error(start.source_location(), "color-profile", &name)
        })?;
        let value = parse_descriptor_value_from_parser(input, kind, &self.recovery)
            .map_err(|error| with_descriptor_context(error, "color-profile", &name))?;
        self.recovery.retain_component_closures(implicit);
        Ok(CssColorProfileDescriptor::new(value).with_position(position(start)))
    }
}

pub(super) fn parse_descriptor_value_from_parser<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssColorProfileDescriptorKind,
    recovery: &RecoveryState,
) -> Result<CssColorProfileDescriptorValue, ParseError<'i, Error>> {
    validate_descriptor_root(input, kind)?;
    let numeric = NumericInputContext::parsed(recovery.source_snapshot());
    parse_descriptor_boundary(input, "color-profile", kind.css_name(), |input| {
        let beginning = input.state();
        let components = CssComponentValues::collect_from_parser(input, recovery.source_snapshot())
            .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
        input.reset(&beginning);
        let data = parse_value_data(input, kind, &components, &numeric)?;
        input.expect_exhausted().map_err(basic)?;
        Ok(CssColorProfileDescriptorValue::from_parts(
            kind, components, data,
        ))
    })
}

/// The component-native constructor uses the same grammar with strict numeric admission.
pub(crate) fn construct_descriptor_value(
    kind: CssColorProfileDescriptorKind,
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<CssColorProfileDescriptorData, Error> {
    let working_source = crate::tokenization::prepare(serialized.as_css());
    let mut parser_input = cssparser::ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    validate_descriptor_root(&mut input, kind)
        .map_err(|error| from_parse_error(serialized.as_css(), error))?;
    let numeric = NumericInputContext::components(components, serialized);
    parse_descriptor_boundary(&mut input, "color-profile", kind.css_name(), |input| {
        let data = parse_value_data(input, kind, components, &numeric)?;
        input.expect_exhausted().map_err(basic)?;
        Ok(data)
    })
    .map_err(|error| from_parse_error(serialized.as_css(), error))
}

// All descriptor front doors apply the same root grammar before substitution
// classification. Nested blocks remain data, including var()/env() fallbacks.
fn validate_descriptor_root<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssColorProfileDescriptorKind,
) -> Result<(), ParseError<'i, Error>> {
    let start = input.state();
    loop {
        let token_start = input.position();
        let location = input.current_source_location();
        let token = match input.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(error) if matches!(error.kind, cssparser::BasicParseErrorKind::EndOfInput) => break,
            Err(error) => return Err(basic(error)),
        };
        if matches!(
            token,
            Token::Semicolon
                | Token::CloseCurlyBracket
                | Token::CloseParenthesis
                | Token::CloseSquareBracket
        ) {
            return Err(crate::error::invalid_descriptor_token_at(
                location,
                "color-profile",
                kind.css_name(),
                &token,
                input.slice_from(token_start),
            ));
        }
        super::fragments::finish_nested_component(input, &token)?;
    }
    input.reset(&start);
    Ok(())
}

fn parse_value_data<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssColorProfileDescriptorKind,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColorProfileDescriptorData, ParseError<'i, Error>> {
    let qualifying =
        super::variables::descriptor_environment_qualifies(components.items(), numeric)
            .map_err(|error| invalid_component_value(input.current_source_location(), error))?;
    if qualifying {
        let start = input.position();
        consume_remaining_components(input)?;
        return Ok(CssColorProfileDescriptorData::Pending(
            CssSubstitutionDependentValue::new(CssAuthoredDeclarationValue::new(
                input.slice_from(start),
            )),
        ));
    }
    Ok(match kind {
        CssColorProfileDescriptorKind::Src => {
            CssColorProfileDescriptorData::Src(super::url::parse_url(input, numeric)?)
        }
        CssColorProfileDescriptorKind::RenderingIntent => {
            let location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            CssColorProfileDescriptorData::RenderingIntent(
                CssColorProfileRenderingIntent::from_css_name(&ident)
                    .ok_or_else(|| invalid_syntax(location, "expected a rendering intent"))?,
            )
        }
        CssColorProfileDescriptorKind::Components => {
            CssColorProfileDescriptorData::Components(input.parse_comma_separated(|input| {
                let location = input.current_source_location();
                let ident = input.expect_ident_cloned().map_err(basic)?;
                input.expect_exhausted().map_err(basic)?;
                CssColorProfileComponentName::try_new(ident.as_ref()).ok_or_else(|| {
                    invalid_syntax(location, "expected a component identifier other than none")
                })
            })?)
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
