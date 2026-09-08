use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token, match_ignore_ascii_case,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::values::parse_custom_ident_from_str_at;
use super::{
    DeclarationMode, block_item_diagnostic, consume_failed_rule_block,
    is_declaration_recovery_unit, parse_declaration_core, structural_recovery_production,
    structural_rule_diagnostic,
};
use crate::error::{
    CssFeatureId, Error, basic, invalid_at_rule_placement, unsupported_value, unsupported_value_at,
};
use crate::syntax::*;

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.rule.keyframes")];

pub(super) fn parse_keyframes_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssKeyframesName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(name) = input.try_parse(Parser::expect_ident_cloned) {
        if name.eq_ignore_ascii_case("none") {
            return Err(unsupported_value_at(
                location,
                None,
                "`none` is reserved and cannot be a keyframes name",
            ));
        }
        return parse_custom_ident_from_str_at("keyframes name", name.as_ref(), location)
            .map(CssKeyframesName::Ident);
    }

    let value = input.expect_string_cloned().map_err(basic)?;
    CssKeyframesString::try_new(value.to_string())
        .map(CssKeyframesName::String)
        .ok_or_else(|| unsupported_value(input, None, "keyframes string name is empty"))
}

pub(super) fn parse_keyframes_rule<'i, 't>(
    source: &'i str,
    name: CssKeyframesName,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> std::result::Result<CssKeyframesRule, ParseError<'i, Error>> {
    let mut parser = KeyframeBlockParser {
        source,
        diagnostics: Vec::new(),
        recovery,
    };
    let mut blocks = Vec::new();
    let mut previous_end = input.position().byte_index();
    {
        let mut items = RuleBodyParser::new(input, &mut parser);
        loop {
            let progress = RecoveryProgress::record(items.input);
            let Some(item) = items.next() else {
                break;
            };
            let failed_block_error = item.as_ref().err().and_then(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    items.input,
                    true,
                    &items.parser.recovery,
                    structural_recovery_production(failed_unit),
                )
                .1
            });
            let retained = item.is_ok();
            let progress_outcome = progress.finish(items.input, retained);
            let unit_end = items.input.position().byte_index();
            match item {
                Ok(block) => blocks.push(block),
                Err((error, failed_unit)) => {
                    let error = failed_block_error.unwrap_or(error);
                    if let Some(diagnostic) = structural_rule_diagnostic(
                        source,
                        error,
                        failed_unit,
                        previous_end,
                        unit_end,
                        crate::CssRecoveryAction::DropKeyframeBlock,
                    ) {
                        items.parser.diagnostics.push(diagnostic);
                    }
                }
            }
            previous_end = unit_end;
            if progress_outcome == RecoveryLoopOutcome::Terminated {
                break;
            }
        }
    }
    diagnostics.append(&mut parser.diagnostics);

    Ok(CssKeyframesRule::new(
        name,
        blocks,
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
}

struct KeyframeBlockParser<'s> {
    source: &'s str,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
}

impl<'i> AtRuleParser<'i> for KeyframeBlockParser<'i> {
    type Prelude = ();
    type AtRule = CssKeyframeBlock;
    type Error = Error;
}

impl<'i> QualifiedRuleParser<'i> for KeyframeBlockParser<'i> {
    type Prelude = CssKeyframeSelectorList;
    type QualifiedRule = CssKeyframeBlock;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        parse_keyframe_selector_list(input)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut depth =
            self.recovery
                .enter_rule_block(self.source, input, "baseline.keyframes.block")?;
        let mut declarations = Vec::new();
        let mut declaration_parser = KeyframeDeclarationParser {
            source: self.source,
            recovery: self.recovery.clone(),
        };
        let mut items = RuleBodyParser::new(input, &mut declaration_parser);
        loop {
            let progress = RecoveryProgress::record(items.input);
            let Some(item) = items.next() else {
                break;
            };
            let position = items.input.position().byte_index();
            let failed_at_block = item.is_err()
                && position > 0
                && self.source.as_bytes().get(position - 1) == Some(&b'{');
            let retained = item.is_ok();
            let progress_outcome = progress.finish(items.input, retained);
            let unit_end = items.input.position().byte_index();
            match item {
                Ok(declaration) => declarations.push(declaration),
                Err((error, failed_unit))
                    if is_declaration_recovery_unit(failed_unit) && !failed_at_block =>
                {
                    if let Some(diagnostic) = block_item_diagnostic(
                        self.source,
                        error,
                        failed_unit,
                        unit_end,
                        crate::CssRecoveryAction::DropDeclaration,
                    ) {
                        self.diagnostics.push(diagnostic);
                    }
                }
                Err((error, _)) => return Err(error),
            }
            if progress_outcome == RecoveryLoopOutcome::Terminated {
                break;
            }
        }

        let result = CssKeyframeBlock::new(
            selectors,
            CssKeyframeDeclarationList::new(declarations),
            crate::source::CssSourcePosition::from_cssparser(
                start.position(),
                start.source_location(),
            ),
        );
        depth.retain();
        Ok(result)
    }
}

impl<'i> DeclarationParser<'i> for KeyframeBlockParser<'i> {
    type Declaration = CssKeyframeBlock;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssKeyframeBlock, Error> for KeyframeBlockParser<'i> {
    fn parse_declarations(&self) -> bool {
        false
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

fn parse_keyframe_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssKeyframeSelectorList, ParseError<'i, Error>> {
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_keyframe_selector(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "keyframe selector list has an empty item",
            ));
        }
    }

    CssKeyframeSelectorList::try_new(selectors)
        .ok_or_else(|| unsupported_value(input, None, "invalid keyframe selector list"))
}

fn parse_keyframe_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssKeyframeSelector, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "from" => Ok(CssKeyframeSelector::From),
            "to" => Ok(CssKeyframeSelector::To),
            _ => Err(unsupported_value_at(
                location,
                None,
                format!("unsupported keyframe selector `{ident}`"),
            )),
        },
        Token::Percentage { unit_value, .. } => {
            let value = *unit_value * 100.0;
            CssKeyframePercent::try_new(value)
                .map(CssKeyframeSelector::Percent)
                .ok_or_else(|| {
                    unsupported_value_at(
                        location,
                        None,
                        "keyframe selector must be 0% through 100%",
                    )
                })
        }
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            "keyframe selector percentages must include `%`",
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

struct KeyframeDeclarationParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

impl<'i> AtRuleParser<'i> for KeyframeDeclarationParser<'i> {
    type Prelude = ();
    type AtRule = CssKeyframeDeclaration;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        Err(invalid_at_rule_placement(
            input.current_source_location(),
            name.as_ref(),
            "a keyframe declaration list",
        ))
    }
}

impl<'i> QualifiedRuleParser<'i> for KeyframeDeclarationParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssKeyframeDeclaration;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssKeyframeDeclaration, Error> for KeyframeDeclarationParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        false
    }
}

impl<'i> DeclarationParser<'i> for KeyframeDeclarationParser<'i> {
    type Declaration = CssKeyframeDeclaration;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.declaration")?;
        let parsed =
            parse_declaration_core(DeclarationMode::Keyframe, name, input, declaration_start)?;
        self.recovery.retain_component_closures(implicit_closures);
        Ok(CssKeyframeDeclaration::new(parsed.body, parsed.position))
    }
}
