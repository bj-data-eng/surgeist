use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{
    Recovered, consume_failed_rule_block, parse_descriptor_boundary, structural_rule_diagnostic,
};
use crate::descriptor_values::{FontFeatureDisplayData, FontFeatureValueData};
use crate::error::{
    Error, basic, descriptor_name_error, invalid_syntax, with_at_rule_prelude_context,
    with_descriptor_context,
};
use crate::numeric::NumericInputContext;
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
    let recovered = parse_outer_body(source, input, recovery);
    diagnostics.extend(recovered.diagnostics);
    CssFontFeatureValuesRule::try_new(families, recovered.syntax)
        .map(|rule| rule.with_position(position(start)))
        .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))
}

pub(super) fn parse_outer_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: RecoveryState,
) -> Recovered<Vec<CssFontFeatureValuesItem>> {
    let mut parser = BodyParser {
        source,
        recovery,
        kind: None,
        diagnostics: Vec::new(),
    };
    let members = parse_body(input, &mut parser);
    let items = members
        .into_iter()
        .map(|member| match member {
            Member::Item(item) => item,
            Member::Definition(_) => unreachable!("outer body returns only mixed rule items"),
        })
        .collect();
    Recovered {
        syntax: items,
        diagnostics: parser.diagnostics,
    }
}

pub(super) fn parse_definition_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    kind: CssFontFeatureValueKind,
    recovery: RecoveryState,
) -> Recovered<Vec<CssFontFeatureValueDefinition>> {
    let mut parser = BodyParser {
        source,
        recovery,
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
    Recovered {
        syntax: definitions,
        diagnostics: parser.diagnostics,
    }
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
        let recovered = parse_definition_body(self.source, input, kind, self.recovery.clone());
        let block = CssFontFeatureValueBlock::try_new(kind, recovered.syntax)
            .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))?
            .with_position(position(start));
        self.diagnostics.extend(recovered.diagnostics);
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
        let value_start = input.state();
        input.reset(start);
        input.expect_ident().map_err(basic)?;
        let name_origin = CssParsedOrigin::from_range(
            self.recovery.source_snapshot(),
            start.position().byte_index()..input.position().byte_index(),
        )
        .expect("actual declaration name");
        input.reset(&value_start);
        let result = parse_descriptor_boundary(input, owner, &name, |input| {
            if let Some(kind) = self.kind {
                let friendly = CssFontFeatureValueName::try_new(name.to_string())
                    .map_err(|error| invalid_syntax(start.source_location(), error.to_string()))?;
                let (data, components, origin) = super::collect_declaration_value(
                    input,
                    self.recovery.source_snapshot(),
                    |input| {
                        parse_feature_value_with_name(
                            input,
                            kind,
                            self.recovery.source_snapshot(),
                            Some(start.source_location()),
                        )
                    },
                )?;
                let value = CssFontFeatureValue::from_parsed(kind, data, components, origin);
                Ok(Member::Definition(
                    CssFontFeatureValueDefinition::new(friendly, value)
                        .with_parsed_name(name_origin.clone())
                        .with_position(position(start)),
                ))
            } else {
                if !name.eq_ignore_ascii_case("font-display") {
                    return Err(descriptor_name_error(start.source_location(), owner, &name));
                }
                let (data, components, origin) = super::collect_declaration_value(
                    input,
                    self.recovery.source_snapshot(),
                    |input| parse_display_value(input, self.recovery.source_snapshot()),
                )?;
                let value = CssFontFeatureDisplayValue::from_parsed(data, components, origin);
                Ok(Member::Item(CssFontFeatureValuesItem::FontDisplay(
                    CssFontFeatureDisplayOccurrence::new(value)
                        .with_parsed_name(name_origin.clone())
                        .with_position(position(start)),
                )))
            }
        })
        .map_err(|error| with_descriptor_context(error, owner, &name))?;
        self.recovery.retain_component_closures(implicit);
        Ok(result)
    }
}

// The rule owner keeps its actual friendly-name error position; value fragments
// select the responsible component origin. Both use this exact token grammar.
pub(super) fn indexes_from_components<'i>(
    values: &CssComponentValues,
    error_for_component: impl Fn(&CssComponentValue, &str) -> ParseError<'i, Error>,
) -> Result<Vec<CssFontFeatureValueIndex>, ParseError<'i, Error>> {
    let mut indexes = Vec::new();
    for value in values.items() {
        match value.view() {
            CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_)) => {}
            CssComponentValueRef::Token(CssValueTokenRef::Number(number))
                if number.kind() == CssNumericTokenKind::Integer =>
            {
                let index = CssFontFeatureValueIndex::try_from_decimal(number.representation())
                    .map_err(|error| error_for_component(value, &error.to_string()))?;
                indexes.push(match value.origin() {
                    CssValueOrigin::Parsed(origin) => index.with_origin(origin.clone()),
                    _ => index,
                });
            }
            _ => {
                return Err(error_for_component(
                    value,
                    "expected nonnegative integer tokens",
                ));
            }
        }
    }
    Ok(indexes)
}

fn qualifies(
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
    location: cssparser::SourceLocation,
) -> Result<bool, ParseError<'static, Error>> {
    super::variables::descriptor_environment_qualifies(components.items(), numeric)
        .map_err(|error| crate::error::invalid_component_value(location, error))
}
fn pending<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<CssSubstitutionDependentValue, ParseError<'i, Error>> {
    let start = input.position();
    super::descriptor_values::consume_remaining_components(input)?;
    Ok(CssSubstitutionDependentValue::new(
        CssAuthoredDeclarationValue::new(input.slice_from(start)),
    ))
}
pub(super) fn parse_display_value<'i>(
    input: &mut Parser<'i, '_>,
    snapshot: &CssSourceSnapshot,
) -> Result<FontFeatureDisplayData, ParseError<'i, Error>> {
    super::descriptor_values::validate_root(input, "font-feature-values", "font-display")?;
    let start = input.state();
    let components = CssComponentValues::collect_from_parser(input, snapshot).map_err(|error| {
        crate::error::invalid_component_value(input.current_source_location(), error)
    })?;
    input.reset(&start);
    display_data(input, &components, &NumericInputContext::parsed(snapshot))
}
fn display_data<'i>(
    input: &mut Parser<'i, '_>,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Result<FontFeatureDisplayData, ParseError<'i, Error>> {
    if qualifies(components, numeric, input.current_source_location())? {
        return Ok(FontFeatureDisplayData::Pending(pending(input)?));
    }
    let value = super::font_face::parse_font_display(input)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(FontFeatureDisplayData::Ordinary(value))
}
pub(crate) fn construct_display_value(
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<FontFeatureDisplayData, Error> {
    let source = crate::tokenization::prepare(serialized.as_css());
    let mut parser_input = cssparser::ParserInput::new(&source);
    let mut input = Parser::new(&mut parser_input);
    let numeric = NumericInputContext::components(components, serialized);
    let result = (|| {
        super::descriptor_values::validate_root(&mut input, "font-feature-values", "font-display")?;
        parse_descriptor_boundary(&mut input, "font-feature-values", "font-display", |input| {
            display_data(input, components, &numeric)
        })
    })();
    result.map_err(|error| crate::error::from_parse_error(serialized.as_css(), error))
}

pub(super) fn parse_feature_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontFeatureValueKind,
    snapshot: &CssSourceSnapshot,
) -> Result<FontFeatureValueData, ParseError<'i, Error>> {
    parse_feature_value_with_name(input, kind, snapshot, None)
}
fn parse_feature_value_with_name<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontFeatureValueKind,
    snapshot: &CssSourceSnapshot,
    name_location: Option<cssparser::SourceLocation>,
) -> Result<FontFeatureValueData, ParseError<'i, Error>> {
    let start = input.state();
    let components = CssComponentValues::collect_from_parser(input, snapshot).map_err(|error| {
        crate::error::invalid_component_value(input.current_source_location(), error)
    })?;
    input.reset(&start);
    feature_data(
        input,
        kind,
        &components,
        &NumericInputContext::parsed(snapshot),
        name_location,
        crate::error::unexpected_component_value,
    )
}
fn feature_data<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssFontFeatureValueKind,
    components: &CssComponentValues,
    numeric: &NumericInputContext<'_>,
    name_location: Option<cssparser::SourceLocation>,
    error: impl Fn(&CssComponentValue, &'static str) -> ParseError<'i, Error>,
) -> Result<FontFeatureValueData, ParseError<'i, Error>> {
    // Annotation/boundary tokens are disallowed before whole-value qualification.
    for component in components.items() {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Semicolon | CssValueTokenRef::Delim('!'))
        ) {
            return Err(error(component, "nonnegative integer tokens"));
        }
    }
    if qualifies(components, numeric, input.current_source_location())? {
        return Ok(FontFeatureValueData::Pending(pending(input)?));
    }
    let indexes = indexes_from_components(components, |component, reason| {
        if let Some(location) = name_location {
            invalid_syntax(location, reason)
        } else {
            error(component, "nonnegative integer tokens")
        }
    })?;
    super::descriptor_values::consume_remaining_components(input)?;
    kind.validate_index_count(indexes.len()).map_err(|issue| {
        if let Some(location) = name_location {
            return invalid_syntax(location, issue.to_string());
        }
        let surplus = components
            .items()
            .iter()
            .filter(|component| {
                !matches!(
                    component.view(),
                    CssComponentValueRef::Comment(_)
                        | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                )
            })
            .enumerate()
            .find(|(index, _)| kind.validate_index_count(index + 1).is_err());
        match surplus {
            Some((_, component)) => error(component, "the selected feature-value index count"),
            None => crate::error::unexpected_end_at(
                input.current_source_location(),
                "the selected feature-value index count",
            ),
        }
    })?;
    Ok(FontFeatureValueData::Indexes(indexes))
}
pub(crate) fn construct_feature_value(
    kind: CssFontFeatureValueKind,
    components: &CssComponentValues,
    serialized: &CssSerializedValue,
) -> Result<FontFeatureValueData, Error> {
    let source = crate::tokenization::prepare(serialized.as_css());
    let mut parser_input = cssparser::ParserInput::new(&source);
    let mut input = Parser::new(&mut parser_input);
    let numeric = NumericInputContext::components(components, serialized);
    feature_data(
        &mut input,
        kind,
        components,
        &numeric,
        None,
        |component, expectation| {
            let index = components
                .items()
                .iter()
                .position(|item| std::ptr::eq(item, component))
                .expect("root grammar component");
            let offset = serialized
                .component_offset_for_path(&[index])
                .expect("serialized root component");
            crate::error::unexpected_component_value_at(
                component,
                expectation,
                CssSourcePosition::from_byte_offset_in(serialized.as_css(), offset),
            )
        },
    )
    .map_err(|error| crate::error::from_parse_error(serialized.as_css(), error))
}
