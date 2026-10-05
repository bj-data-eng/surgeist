use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError, ParseErrorKind,
    Parser, ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, SourceLocation,
    match_ignore_ascii_case,
};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::url::parse_url;
use super::values::parse_integer_literal;
use super::{
    block_item_diagnostic, is_declaration_recovery_unit, parse_descriptor_boundary,
    top_level_only_at_rule_placement,
};
use crate::error::{
    Error, basic, descriptor_name_error, invalid_descriptor_combination, unsupported_value,
    unsupported_value_at, with_descriptor_context,
};
use crate::syntax::*;

pub(super) fn parse_counter_style_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssCounterStyleName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input.expect_ident_cloned().map_err(basic)?;
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
    Ok(name)
}

pub(super) fn parse_counter_style_rule<'i, 't>(
    source: &'i str,
    name: CssCounterStyleName,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssCounterStyleRule, ParseError<'i, Error>> {
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

    Ok(CssCounterStyleRule::new(
        name,
        descriptors,
        crate::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
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
        let numeric = crate::numeric::NumericInputContext::parsed(self.recovery.source_snapshot());
        let position = crate::CssSourcePosition::from_cssparser(
            declaration_start.position(),
            declaration_start.source_location(),
        );
        let result = (|| {
            Ok(match_ignore_ascii_case! { &name,
                "system" => CssCounterStyleDescriptor::System(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "system", |input| parse_system(input, &numeric))?,
                    position,
                )),
                "negative" => CssCounterStyleDescriptor::Negative(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "negative", |input| parse_negative(input, &numeric))?,
                    position,
                )),
                "symbols" => CssCounterStyleDescriptor::Symbols(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "symbols", |input| parse_symbols(input, &numeric))?,
                    position,
                )),
                "prefix" => CssCounterStyleDescriptor::Prefix(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "prefix", |input| parse_symbol(input, &numeric))?,
                    position,
                )),
                "suffix" => CssCounterStyleDescriptor::Suffix(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "suffix", |input| parse_symbol(input, &numeric))?,
                    position,
                )),
                "range" => CssCounterStyleDescriptor::Range(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "range", |input| parse_range(input, &numeric))?,
                    position,
                )),
                "pad" => CssCounterStyleDescriptor::Pad(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "pad", |input| parse_pad(input, &numeric))?,
                    position,
                )),
                "fallback" => CssCounterStyleDescriptor::Fallback(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "fallback", parse_fallback)?,
                    position,
                )),
                "additive-symbols" => CssCounterStyleDescriptor::AdditiveSymbols(
                    CssDescriptorOccurrence::new(
                        parse_descriptor_boundary(
                            input,
                            "counter-style",
                            "additive-symbols",
                            |input| parse_additive_symbols(input, &numeric),
                        )?,
                        position,
                    ),
                ),
                "speak-as" => CssCounterStyleDescriptor::SpeakAs(CssDescriptorOccurrence::new(
                    parse_descriptor_boundary(input, "counter-style", "speak-as", parse_speak_as)?,
                    position,
                )),
                _ => return Err(descriptor_name_error(
                    declaration_start.source_location(),
                    "counter-style",
                    name.as_ref(),
                )),
            })
        })()
        .map_err(|error| with_descriptor_context(error, "counter-style", name.as_ref()))?;
        self.recovery.retain_component_closures(implicit_closures);
        Ok(result)
    }
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
                Some(parse_integer_literal(input, numeric)?)
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
        parse_integer_literal(input, numeric).map(CssCounterStyleRangeBound::Integer)
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
) -> Result<crate::CssIntegerLiteral, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_integer_literal(input, numeric)?;
    if value.is_negative() {
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
    if let Ok(value) = input.try_parse(|input| parse_url(input, numeric)) {
        return Ok(CssCounterSymbol::Url(value));
    }
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssCounterSymbolIdent::try_new(ident.to_string())
        .map(CssCounterSymbol::Ident)
        .ok_or_else(|| unsupported_value_at(location, None, "invalid custom-ident counter symbol"))
}
