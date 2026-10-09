use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError, ParseErrorKind,
    Parser, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, SourceLocation,
    match_ignore_ascii_case,
};

use super::background::parse_image;
use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::values::parse_integer_value;
use super::{
    block_item_diagnostic, collect_declaration_value, is_declaration_recovery_unit,
    parse_descriptor_boundary, top_level_only_at_rule_placement,
};
use crate::CssCounterStyleDescriptorKind;
use crate::descriptor_values::CounterStyleValueData;
use crate::error::{
    Error, basic, descriptor_name_error, invalid_descriptor_combination, unsupported_value,
    unsupported_value_at, with_descriptor_context,
};
use crate::syntax::*;

pub(super) struct CounterStylePrelude {
    name: CssCounterStyleName,
    origin: crate::CssParsedOrigin,
}

pub(super) fn parse_counter_style_name<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_snapshot: &crate::CssSourceSnapshot,
) -> Result<CounterStylePrelude, ParseError<'i, Error>> {
    input.skip_whitespace();
    let token_start = input.position();
    let location = input.current_source_location();
    let name = input.expect_ident_cloned().map_err(basic)?;
    let origin = crate::CssParsedOrigin::from_range(
        source_snapshot,
        token_start.byte_index()..input.position().byte_index(),
    )
    .expect("counter name token belongs to the original source");
    let name = CssCounterStyleName::try_new(name.to_string()).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            "counter-style names exclude CSS-wide keywords, `default` and `none`",
        )
    })?;
    input.expect_exhausted().map_err(basic)?;
    // Counter Styles 3 §3 excludes these names only from rule definitions.
    // They remain valid references in extends, fallback and speak-as.
    if matches!(
        name.as_str(),
        "decimal" | "disc" | "square" | "circle" | "disclosure-open" | "disclosure-closed"
    ) {
        return Err(unsupported_value_at(
            location,
            None,
            "counter-style definitions cannot use protected predefined names",
        ));
    }
    Ok(CounterStylePrelude { name, origin })
}

pub(super) fn parse_counter_style_rule<'i, 't>(
    source: &'i str,
    prelude: CounterStylePrelude,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssCounterStyleRule, ParseError<'i, Error>> {
    let descriptors = parse_body(source, input, diagnostics, recovery)?;
    Ok(CssCounterStyleRule::from_parsed(
        prelude.name,
        descriptors,
        crate::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
        prelude.origin,
    ))
}

pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssCounterStyleDescriptors, ParseError<'i, Error>> {
    let mut occurrences = Vec::new();
    let mut descriptor_parser = CounterStyleDescriptorParser { source, recovery };
    let mut items = RuleBodyParser::new(input, &mut descriptor_parser);

    loop {
        let progress = RecoveryProgress::record(items.input);
        let Some(item) = items.next() else {
            break;
        };
        let retained = item.is_ok();
        let progress_outcome = progress.finish(items.input, retained);
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(descriptor) => occurrences.push(descriptor),
            Err((error, failed_unit)) => {
                let action = if is_declaration_recovery_unit(failed_unit) {
                    crate::CssRecoveryAction::DropDescriptor
                } else {
                    crate::CssRecoveryAction::DropAtRule
                };
                if let Some(diagnostic) =
                    block_item_diagnostic(source, error, failed_unit, unit_end, action)
                {
                    diagnostics.push(diagnostic);
                }
            }
        }
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }

    let descriptors =
        CssCounterStyleDescriptors::from_occurrences(occurrences).map_err(|issue| {
            invalid_descriptor_combination(
                input,
                issue.position(),
                "counter-style",
                issue.responsible(),
                issue.conflicting(),
            )
        })?;

    Ok(descriptors)
}

struct CounterStyleDescriptorParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

enum CounterStyleBodyAtRulePrelude<'i> {
    TopLevelOnly(CowRcStr<'i>, SourceLocation),
    Other(CowRcStr<'i>, SourceLocation),
}

impl<'i> AtRuleParser<'i> for CounterStyleDescriptorParser<'i> {
    type Prelude = CounterStyleBodyAtRulePrelude<'i>;
    type AtRule = CssCounterStyleDescriptor;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let location = input.current_source_location();
        while input.next_including_whitespace_and_comments().is_ok() {}
        if name.eq_ignore_ascii_case("counter-style") || name.eq_ignore_ascii_case("page") {
            return Ok(CounterStyleBodyAtRulePrelude::TopLevelOnly(name, location));
        }
        Ok(CounterStyleBodyAtRulePrelude::Other(name, location))
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        _input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        match prelude {
            CounterStyleBodyAtRulePrelude::TopLevelOnly(name, location) => {
                Err(top_level_only_at_rule_placement(location, name.as_ref()))
            }
            CounterStyleBodyAtRulePrelude::Other(name, location) => Err(ParseError {
                kind: ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(name)),
                location,
            }),
        }
    }
}

impl<'i> QualifiedRuleParser<'i> for CounterStyleDescriptorParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssCounterStyleDescriptor;
    type Error = Error;
}

impl<'i> RuleBodyItemParser<'i, CssCounterStyleDescriptor, Error>
    for CounterStyleDescriptorParser<'i>
{
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> DeclarationParser<'i> for CounterStyleDescriptorParser<'i> {
    type Declaration = CssCounterStyleDescriptor;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.descriptor")?;
        // Revisit the original descriptor-name token, just as declarations do.
        // The semantic name may be decoded/case-folded; its source is not.
        let value_start = input.state();
        input.reset(declaration_start);
        input.expect_ident().map_err(basic)?;
        let name_origin = crate::CssParsedOrigin::from_range(
            self.recovery.source_snapshot(),
            declaration_start.position().byte_index()..input.position().byte_index(),
        )
        .expect("counter descriptor name belongs to the original source");
        input.reset(&value_start);
        let result = (|| {
            let kind = match_ignore_ascii_case! { &name,
                "system" => CssCounterStyleDescriptorKind::System,
                "negative" => CssCounterStyleDescriptorKind::Negative,
                "symbols" => CssCounterStyleDescriptorKind::Symbols,
                "prefix" => CssCounterStyleDescriptorKind::Prefix,
                "suffix" => CssCounterStyleDescriptorKind::Suffix,
                "range" => CssCounterStyleDescriptorKind::Range,
                "pad" => CssCounterStyleDescriptorKind::Pad,
                "fallback" => CssCounterStyleDescriptorKind::Fallback,
                "additive-symbols" => CssCounterStyleDescriptorKind::AdditiveSymbols,
                "speak-as" => CssCounterStyleDescriptorKind::SpeakAs,
                _ => return Err(descriptor_name_error(
                    declaration_start.source_location(),
                    "counter-style",
                    name.as_ref(),
                )),
            };
            parse_occurrence(input, self.recovery.source_snapshot(), &name_origin, kind)
        })()
        .map_err(|error| {
            if crate::error::is_resource_parse_error(&error) {
                error
            } else {
                with_descriptor_context(error, "counter-style", name.as_ref())
            }
        })?;
        self.recovery.retain_component_closures(implicit_closures);
        Ok(result)
    }
}

fn parse_occurrence<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_snapshot: &crate::CssSourceSnapshot,
    name_origin: &crate::CssParsedOrigin,
    descriptor: CssCounterStyleDescriptorKind,
) -> Result<CssCounterStyleDescriptor, ParseError<'i, Error>> {
    parse_descriptor_boundary(input, "counter-style", descriptor.css_name(), |input| {
        let (value, components, value_origin) =
            collect_declaration_value(input, source_snapshot, |input| {
                parse_authored_value(input, descriptor, source_snapshot)
            })?;
        let value = crate::CssCounterStyleDescriptorValue::from_parsed(
            descriptor,
            value,
            components.clone(),
            value_origin.clone(),
        );
        Ok(value.into_occurrence(name_origin.clone(), value_origin, components))
    })
}

pub(super) fn parse_descriptor_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    kind: CssCounterStyleDescriptorKind,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CounterStyleValueData, ParseError<'i, Error>> {
    Ok(match kind {
        CssCounterStyleDescriptorKind::System => {
            CounterStyleValueData::System(parse_system(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Negative => {
            CounterStyleValueData::Negative(parse_negative(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Symbols => {
            CounterStyleValueData::Symbols(parse_symbols(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Prefix => {
            CounterStyleValueData::Prefix(parse_symbol(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Suffix => {
            CounterStyleValueData::Suffix(parse_symbol(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Range => {
            CounterStyleValueData::Range(parse_range(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Pad => {
            CounterStyleValueData::Pad(parse_pad(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::Fallback => {
            CounterStyleValueData::Fallback(parse_fallback(input)?)
        }
        CssCounterStyleDescriptorKind::AdditiveSymbols => {
            CounterStyleValueData::AdditiveSymbols(parse_additive_symbols(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::SpeakAs => {
            CounterStyleValueData::SpeakAs(parse_speak_as(input)?)
        }
    })
}

fn parse_system<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterStyleSystem, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let system = match_ignore_ascii_case! { &ident,
        "cyclic" => CssCounterStyleSystem::Cyclic,
        "numeric" => CssCounterStyleSystem::Numeric,
        "alphabetic" => CssCounterStyleSystem::Alphabetic,
        "symbolic" => CssCounterStyleSystem::Symbolic,
        "additive" => CssCounterStyleSystem::Additive,
        "fixed" => {
            let first_symbol_value = if input.is_exhausted() {
                None
            } else {
                Some(parse_integer_value(input, numeric)?)
            };
            CssCounterStyleSystem::Fixed(CssCounterStyleFixedSystem::new(first_symbol_value))
        },
        "extends" => {
            let location = input.current_source_location();
            let name = input.expect_ident_cloned().map_err(basic)?;
            let name = CssCounterStyleName::try_new(name.to_string()).ok_or_else(|| {
                unsupported_value_at(location, None, "invalid extended counter-style name")
            })?;
            CssCounterStyleSystem::Extends(name)
        },
        _ => return Err(unsupported_value(input, None, "unsupported counter-style system")),
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(system)
}

fn parse_symbols<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterSymbols, ParseError<'i, Error>> {
    let mut symbols = Vec::new();
    while !input.is_exhausted() {
        symbols.push(parse_symbol_component(input, numeric)?);
    }
    if symbols.is_empty() {
        Err(unsupported_value(input, None, "symbols must not be empty"))
    } else {
        Ok(CssCounterSymbols::new(symbols))
    }
}

fn parse_negative<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterStyleNegative, ParseError<'i, Error>> {
    let prefix = parse_symbol_component(input, numeric)?;
    let suffix = if input.is_exhausted() {
        None
    } else {
        Some(parse_symbol_component(input, numeric)?)
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssCounterStyleNegative::new(prefix, suffix))
}

fn parse_range<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterStyleRange, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        input.expect_exhausted().map_err(basic)?;
        return Ok(CssCounterStyleRange::Auto);
    }

    let mut ranges = Vec::new();
    loop {
        let lower_location = input.current_source_location();
        let lower = parse_range_bound(input, numeric)?;
        let upper = parse_range_bound(input, numeric)?;
        let interval = CssCounterStyleRangeInterval::try_new(lower, upper).ok_or_else(|| {
            unsupported_value_at(
                lower_location,
                None,
                "counter-style range lower bound exceeds its upper bound",
            )
        })?;
        ranges.push(interval);
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
    }
    Ok(CssCounterStyleRange::Ranges(
        CssCounterStyleRanges::try_new(ranges).expect("parsed nonempty counter ranges"),
    ))
}

fn parse_range_bound<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterStyleRangeBound, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("infinite"))
        .is_ok()
    {
        Ok(CssCounterStyleRangeBound::Infinite)
    } else {
        parse_integer_value(input, numeric).map(CssCounterStyleRangeBound::Integer)
    }
}

fn parse_pad<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterStylePad, ParseError<'i, Error>> {
    let (minimum_length, symbol) = if let Ok(minimum_length) =
        input.try_parse(|input| parse_nonnegative_integer(input, numeric))
    {
        (minimum_length, parse_symbol_component(input, numeric)?)
    } else {
        let symbol = parse_symbol_component(input, numeric)?;
        (parse_nonnegative_integer(input, numeric)?, symbol)
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(
        CssCounterStylePad::try_new(minimum_length, symbol)
            .expect("checked nonnegative pad length"),
    )
}

fn parse_fallback<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssCounterStyleName, ParseError<'i, Error>> {
    let name = parse_counter_style_name_component(input)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(name)
}

fn parse_additive_symbols<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterAdditiveSymbols, ParseError<'i, Error>> {
    let mut tuples = Vec::new();
    loop {
        let (tuple, weight_location) = parse_additive_tuple(input, numeric)?;
        tuples.push(tuple);
        if !CssCounterAdditiveSymbols::weights_strictly_descend(
            &tuples[tuples.len().saturating_sub(2)..],
        ) {
            return Err(unsupported_value_at(
                weight_location,
                None,
                "additive-symbol weights must be strictly descending",
            ));
        }
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
    }
    Ok(CssCounterAdditiveSymbols::try_new(tuples).expect("checked descending additive list"))
}

fn parse_additive_tuple<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<(CssCounterAdditiveTuple, cssparser::SourceLocation), ParseError<'i, Error>> {
    let initial_location = input.current_source_location();
    let (weight, symbol, weight_location) =
        if let Ok(weight) = input.try_parse(|input| parse_nonnegative_integer(input, numeric)) {
            (
                weight,
                parse_symbol_component(input, numeric)?,
                initial_location,
            )
        } else {
            let symbol = parse_symbol_component(input, numeric)?;
            let weight_location = input.current_source_location();
            (
                parse_nonnegative_integer(input, numeric)?,
                symbol,
                weight_location,
            )
        };
    Ok((
        CssCounterAdditiveTuple::try_new(weight, symbol)
            .expect("checked nonnegative additive weight"),
        weight_location,
    ))
}

fn parse_speak_as<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssCounterStyleSpeakAs, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let value = match_ignore_ascii_case! { &ident,
        "auto" => CssCounterStyleSpeakAs::Auto,
        "bullets" => CssCounterStyleSpeakAs::Bullets,
        "numbers" => CssCounterStyleSpeakAs::Numbers,
        "words" => CssCounterStyleSpeakAs::Words,
        "spell-out" => CssCounterStyleSpeakAs::SpellOut,
        _ => CssCounterStyleName::try_new(ident.to_string())
            .map(CssCounterStyleSpeakAs::CounterStyle)
            .ok_or_else(|| unsupported_value_at(location, None, "invalid spoken counter-style name"))?,
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(value)
}

fn parse_nonnegative_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssIntegerValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_integer_value(input, numeric)?;
    if matches!(&value, CssIntegerValue::Literal(literal) if literal.is_negative()) {
        Err(unsupported_value_at(
            location,
            None,
            "counter-style integer must be nonnegative",
        ))
    } else {
        Ok(value)
    }
}

fn parse_counter_style_name_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssCounterStyleName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input.expect_ident_cloned().map_err(basic)?;
    CssCounterStyleName::try_new(name.to_string())
        .ok_or_else(|| unsupported_value_at(location, None, "invalid counter-style name"))
}

fn parse_symbol<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterSymbol, ParseError<'i, Error>> {
    let symbol = parse_symbol_component(input, numeric)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(symbol)
}

fn parse_symbol_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCounterSymbol, ParseError<'i, Error>> {
    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return CssContentString::try_new(value.to_string())
            .map(CssCounterSymbol::String)
            .ok_or_else(|| unsupported_value(input, None, "counter symbol contains null"));
    }
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return CssCounterSymbolIdent::try_new(ident.to_string())
            .map(CssCounterSymbol::Ident)
            .ok_or_else(|| {
                unsupported_value_at(location, None, "invalid custom-ident counter symbol")
            });
    }
    // An image is selected only after the string/custom-ident alternatives.
    // In particular, `none` is a symbol identifier, not property-level no image.
    // Do not speculate and discard an image provider's typed resource failure.
    parse_image(input, numeric).map(CssCounterSymbol::Image)
}

pub(super) fn parse_authored_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssCounterStyleDescriptorKind,
    snapshot: &crate::CssSourceSnapshot,
) -> Result<CounterStyleValueData, ParseError<'i, Error>> {
    super::descriptor_values::validate_root(input, "counter-style", kind.css_name())?;
    let start = input.state();
    let components =
        crate::CssComponentValues::collect_from_parser(input, snapshot).map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        })?;
    input.reset(&start);
    let numeric = crate::numeric::NumericInputContext::parsed(snapshot);
    parse_value_data(input, kind, &components, &numeric)
}
fn parse_value_data<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssCounterStyleDescriptorKind,
    components: &crate::CssComponentValues,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CounterStyleValueData, ParseError<'i, Error>> {
    if super::variables::descriptor_environment_qualifies(components.items(), numeric).map_err(
        |error| crate::error::invalid_component_value(input.current_source_location(), error),
    )? {
        let start = input.position();
        super::descriptor_values::consume_remaining_components(input)?;
        return Ok(CounterStyleValueData::Pending(
            crate::CssSubstitutionDependentValue::new(crate::CssAuthoredDeclarationValue::new(
                input.slice_from(start),
            )),
        ));
    }
    input.skip_whitespace();
    let value = parse_descriptor_value(input, kind, numeric)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(value)
}
pub(crate) fn construct_descriptor_value(
    kind: CssCounterStyleDescriptorKind,
    components: &crate::CssComponentValues,
    serialized: &crate::CssSerializedValue,
) -> Result<CounterStyleValueData, Error> {
    let source = crate::tokenization::prepare(serialized.as_css());
    let mut parser_input = cssparser::ParserInput::new(&source);
    let mut input = Parser::new(&mut parser_input);
    let numeric = crate::numeric::NumericInputContext::components(components, serialized);
    let result = (|| {
        super::descriptor_values::validate_root(&mut input, "counter-style", kind.css_name())?;
        parse_descriptor_boundary(&mut input, "counter-style", kind.css_name(), |input| {
            parse_value_data(input, kind, components, &numeric)
        })
    })();
    result.map_err(|error| crate::error::from_parse_error(serialized.as_css(), error))
}
