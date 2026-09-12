use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{consume_failed_rule_block, parse_descriptor_boundary, structural_rule_diagnostic};
use crate::error::{
    Error, basic, descriptor_name_error, invalid_syntax, with_at_rule_prelude_context,
    with_descriptor_context,
};
use crate::*;
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser,
};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("later.rule.font-feature-values")];

pub(super) fn parse_families<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<Vec<CssFontFaceFamily>, ParseError<'i, Error>> {
    recovery.check_specialized_components(source, input, "later.rule.font-feature-values")?;
    input
        .parse_comma_separated(|input| {
            let name = super::typography::parse_non_generic_font_family_name(input)?;
            input.expect_exhausted().map_err(basic)?;
            CssFontFaceFamily::try_new(name.as_str()).ok_or_else(|| {
                invalid_syntax(input.current_source_location(), "invalid font family")
            })
        })
        .map_err(|error| {
            with_at_rule_prelude_context(
                error,
                "font-feature-values",
                "later.rule.font-feature-values",
                "a nonempty list of non-generic font family names",
            )
        })
}
pub(super) fn parse_rule<'i>(
    source: &'i str,
    families: Vec<CssFontFaceFamily>,
    input: &mut Parser<'i, '_>,
    start: &ParserState,
    diagnostics: &mut Vec<CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssFontFeatureValuesRule, ParseError<'i, Error>> {
    let mut parser = BodyParser {
        source,
        recovery,
        kind: None,
        diagnostics: Vec::new(),
    };
    let members = parse_body(input, &mut parser);
    diagnostics.extend(parser.diagnostics);
    let items = members
        .into_iter()
        .map(|member| match member {
            Member::Item(item) => item,
            Member::Definition(_) => unreachable!("outer body returns only mixed rule items"),
        })
        .collect();
    CssFontFeatureValuesRule::try_new(families, items)
        .map(|rule| rule.with_position(position(start)))
        .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))
}
fn position(start: &ParserState) -> CssSourcePosition {
    CssSourcePosition::from_cssparser(start.position(), start.source_location())
}

enum Member {
    Item(CssFontFeatureValuesItem),
    Definition(CssFontFeatureValueDefinition),
}
struct BodyParser<'i> {
    source: &'i str,
    recovery: RecoveryState,
    kind: Option<CssFontFeatureValueKind>,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
fn parse_body<'i>(input: &mut Parser<'i, '_>, parser: &mut BodyParser<'i>) -> Vec<Member> {
    let mut result = Vec::new();
    let mut previous_end = input.position().byte_index();
    let mut items = RuleBodyParser::new(input, parser);
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
                "later.rule.font-feature-values",
            )
            .1
        });
        let outcome = progress.finish(items.input, item.is_ok());
        let end = items.input.position().byte_index();
        match item {
            Ok(member) => result.push(member),
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
    result
}
impl<'i> AtRuleParser<'i> for BodyParser<'i> {
    type Prelude = CssFontFeatureValueKind;
    type AtRule = Member;
    type Error = Error;
    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Error>> {
        self.recovery.check_specialized_components(
            self.source,
            input,
            "later.rule.font-feature-values",
        )?;
        let kind = self
            .kind
            .is_none()
            .then(|| CssFontFeatureValueKind::from_css_name(&name))
            .flatten()
            .ok_or_else(|| input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name)))?;
        input.expect_exhausted().map_err(basic)?;
        Ok(kind)
    }
    fn parse_block<'t>(
        &mut self,
        kind: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Error>> {
        let mut depth =
            self.recovery
                .enter_rule_block(self.source, input, "later.rule.font-feature-values")?;
        let mut parser = BodyParser {
            source: self.source,
            recovery: self.recovery.clone(),
            kind: Some(kind),
            diagnostics: Vec::new(),
        };
        let members = parse_body(input, &mut parser);
        let definitions = members
            .into_iter()
            .map(|member| match member {
                Member::Definition(value) => value,
                Member::Item(_) => unreachable!("subsidiary bodies return only definitions"),
            })
            .collect();
        let block = CssFontFeatureValueBlock::try_new(kind, definitions)
            .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))?
            .with_position(position(start));
        self.diagnostics.extend(parser.diagnostics);
        depth.retain();
        Ok(Member::Item(CssFontFeatureValuesItem::Block(block)))
    }
}
impl<'i> QualifiedRuleParser<'i> for BodyParser<'i> {
    type Prelude = ();
    type QualifiedRule = Member;
    type Error = Error;
}
impl<'i> RuleBodyItemParser<'i, Member, Error> for BodyParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}
impl<'i> DeclarationParser<'i> for BodyParser<'i> {
    type Declaration = Member;
    type Error = Error;
    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        start: &ParserState,
    ) -> Result<Member, ParseError<'i, Error>> {
        let implicit =
            self.recovery
                .check_component_values(self.source, input, "css.descriptor")?;
        let owner = self
            .kind
            .map_or("font-feature-values", CssFontFeatureValueKind::css_name);
        let result = parse_descriptor_boundary(input, owner, &name, |input| {
            if let Some(kind) = self.kind {
                let friendly = CssFontFeatureValueName::try_new(name.to_string())
                    .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))?;
                let values =
                    CssComponentValues::collect_from_parser(input, self.recovery.source_snapshot())
                        .map_err(|error| {
                            crate::error::invalid_component_value(
                                input.current_source_location(),
                                error,
                            )
                        })?;
                let mut indexes = Vec::new();
                for value in values.items() {
                    match value.view() {
                        CssComponentValueRef::Comment(_)
                        | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_)) => {}
                        CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                            if number.kind() == CssNumericTokenKind::Integer =>
                        {
                            let index =
                                CssFontFeatureValueIndex::try_from_decimal(number.representation())
                                    .map_err(|error| {
                                        invalid_syntax(start.source_location(), error.to_string())
                                    })?;
                            let CssValueOrigin::Parsed(origin) = value.origin() else {
                                unreachable!("collected number has original source provenance")
                            };
                            indexes.push(index.with_origin(origin.clone()));
                        }
                        _ => {
                            return Err(invalid_syntax(
                                start.source_location(),
                                "expected nonnegative integer tokens",
                            ));
                        }
                    }
                }
                let definition = CssFontFeatureValueDefinition::try_new(friendly, indexes)
                    .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))?;
                kind.validate(&definition)
                    .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))?;
                Ok(Member::Definition(
                    definition.with_position(position(start)),
                ))
            } else {
                if !name.eq_ignore_ascii_case("font-display") {
                    return Err(descriptor_name_error(start.source_location(), owner, &name));
                }
                let value = super::font_face::parse_font_display(input)?;
                Ok(Member::Item(CssFontFeatureValuesItem::FontDisplay(
                    CssFontFeatureDisplayOccurrence::new(value).with_position(position(start)),
                )))
            }
        })
        .map_err(|error| with_descriptor_context(error, owner, &name))?;
        self.recovery.retain_component_closures(implicit);
        Ok(result)
    }
}
