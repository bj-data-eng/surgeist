use cssparser::{BasicParseErrorKind, CowRcStr, ParseError, ParseErrorKind, Parser, ParserState};
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, QualifiedRule, RuleGrammar};
use surgeist_css_parser::{Token, match_ignore_ascii_case};

use super::background::parse_image;
use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, grammar_error, location_at, offset_of,
};
use super::declaration_unit::DeclarationText;
use super::descriptor_stream::{
    Boundary, DescriptorGrammar, Value, consume_remaining, drive_block_items, expect_exhausted,
    fail, from_component, native, parse_descriptor_checked, parse_descriptor_text, parse_scope,
    validate_root,
};
use super::property_input::StreamExt;
use super::recovery::RecoveryState;
use super::syntax_types::{Attached, Stream, SyntaxError};
use super::top_level_only_at_rule_placement;
use super::values::parse_integer_value;
use crate::diagnostic::error::{
    descriptor_name_error, unexpected_at, unexpected_syntax_token, with_descriptor_context,
};
use crate::diagnostic::{CssError, CssRecoveryAction};
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssInteger, CssNumeric, CssNumericLiteral};
use crate::rule::counter_style::CssCounterStyleDescriptorKind;
use crate::rule::counter_style::descriptor_values::CounterStyleValueData;
use crate::rule::counter_style::syntax::CssCounterStyleDescriptor;
use crate::rule::counter_style::{
    CssCounterAdditiveSymbols, CssCounterAdditiveTuple, CssCounterStyleDescriptors,
    CssCounterStyleFixedSystem, CssCounterStyleName, CssCounterStyleNegative, CssCounterStylePad,
    CssCounterStyleRange, CssCounterStyleRangeBound, CssCounterStyleRangeInterval,
    CssCounterStyleRanges, CssCounterStyleRule, CssCounterStyleSpeakAs, CssCounterStyleSystem,
    CssCounterSymbol, CssCounterSymbolIdent, CssCounterSymbols,
};
use crate::values::content::CssContentString;

pub(crate) struct CounterStylePrelude {
    name: CssCounterStyleName,
    origin: crate::source::CssParsedOrigin,
}

pub(crate) fn parse_counter_style_name<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_snapshot: &crate::source::CssSourceSnapshot,
) -> Result<CounterStylePrelude, ParseError<'i, CssError>> {
    parse_scope(input, source_snapshot, |input, numeric| {
        input.discard_whitespace();
        let token_start = input.site();
        let location = input.site();
        let name = input.consume_ident()?;
        let origin = crate::source::CssParsedOrigin::from_range(
            source_snapshot,
            offset_of(token_start)..offset_of(input.site()),
        )
        .expect("counter name token belongs to the original source");
        let name = CssCounterStyleName::try_new(name.to_string())
            .ok_or_else(|| fail(numeric, location, unexpected_at))?;
        expect_exhausted(input)?;
        // Counter Styles 3 §3 excludes these names only from rule definitions.
        // They remain valid references in extends, fallback and speak-as.
        if matches!(
            name.as_str(),
            "decimal" | "disc" | "square" | "circle" | "disclosure-open" | "disclosure-closed"
        ) {
            return Err(fail(numeric, location, unexpected_at));
        }
        Ok(CounterStylePrelude { name, origin })
    })
}

pub(crate) fn parse_counter_style_rule<'i, 't>(
    source: &'i str,
    prelude: CounterStylePrelude,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssCounterStyleRule, ParseError<'i, CssError>> {
    let descriptors = parse_body(source, input, diagnostics, recovery)?;
    Ok(CssCounterStyleRule::from_parsed(
        prelude.name,
        descriptors,
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
        prelude.origin,
    ))
}

/// The descriptors of the block the native parser `input` is inside.
pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssCounterStyleDescriptors, ParseError<'i, CssError>> {
    let snapshot = recovery.source_snapshot().clone();
    let (occurrences, mut recovered) = parse_scope(input, &snapshot, |contents, _| {
        Ok(drive_body(contents, source, &recovery))
    })?;
    diagnostics.append(&mut recovered);
    Ok(CssCounterStyleDescriptors::from_occurrences(occurrences))
}

fn drive_body<'i>(
    contents: &mut Stream<'i>,
    source: &str,
    recovery: &RecoveryState,
) -> (
    Vec<CssCounterStyleDescriptor>,
    Vec<crate::diagnostic::CssRecoveryDiagnostic>,
) {
    let mut grammar = CounterStyleDescriptorParser {
        source,
        recovery: recovery.clone(),
    };
    let mut readmit = CounterStyleDescriptorParser {
        source,
        recovery: recovery.clone(),
    };
    drive_block_items(
        contents,
        &mut grammar,
        &mut readmit,
        source,
        recovery,
        CssRecoveryAction::DropDescriptor,
    )
}

struct CounterStyleDescriptorParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for CounterStyleDescriptorParser<'_> {
    type Item = CssCounterStyleDescriptor;
    type Error = NativeError<'i>;

    // The prelude of an at-rule is read to its end whatever it holds, and the
    // block is then rejected: a recognized top-level rule is misplaced here, and
    // any other is unknown.
    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let name = rule.name();
        let location = location_at(snapshot, offset_of(rule.prelude().start_site()));
        let fail = |error| grammar_error(rule.start(), error);
        if rule.block().is_none() {
            return Err(fail(unexpected_syntax_token(
                location_at(snapshot, offset_of(rule.end())),
                Token::Semicolon,
            )));
        }
        if name.eq_ignore_ascii_case("counter-style") || name.eq_ignore_ascii_case("page") {
            return Err(fail(top_level_only_at_rule_placement(location, name)));
        }
        Err(fail(ParseError {
            kind: ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(CowRcStr::from(name))),
            location,
        }))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for CounterStyleDescriptorParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl<'i> DescriptorGrammar<'i, CssCounterStyleDescriptor> for CounterStyleDescriptorParser<'_> {
    fn readmit(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssCounterStyleDescriptor, NativeError<'i>> {
        self.admit(name, text)
    }
}

impl CounterStyleDescriptorParser<'_> {
    /// Admits the descriptor named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssCounterStyleDescriptor, NativeError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let implicit_closures = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.descriptor",
        )?;
        // The semantic name may be decoded or case-folded; its source is not.
        let name_origin = crate::source::CssParsedOrigin::from_range(snapshot, text.name_range())
            .expect("counter descriptor name belongs to the original source");
        let result = (|| {
            let kind = match_ignore_ascii_case! { name,
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
                    location_at(snapshot, text.name_start()),
                    "counter-style",
                    name,
                )),
            };
            parse_occurrence(snapshot, text, &name_origin, kind)
        })()
        .map_err(|error| {
            if crate::diagnostic::error::is_resource_parse_error(&error) {
                error
            } else {
                with_descriptor_context(error, "counter-style", name)
            }
        })?;
        self.recovery.retain_component_closures(implicit_closures);
        Ok(result)
    }
}

fn parse_occurrence(
    source_snapshot: &crate::source::CssSourceSnapshot,
    text: &DeclarationText,
    name_origin: &crate::source::CssParsedOrigin,
    descriptor: CssCounterStyleDescriptorKind,
) -> Result<CssCounterStyleDescriptor, ParseError<'static, CssError>> {
    parse_descriptor_text(
        source_snapshot,
        text.value_start,
        text.end,
        "counter-style",
        descriptor.as_str(),
        Boundary::ValidatedValue,
        |input, components, numeric, range| {
            let value = parse_value_data(input, descriptor, &components, numeric)?;
            let value_origin = crate::source::CssParsedOrigin::from_range(source_snapshot, range)
                .expect("parser value boundaries belong to the original source");
            let value = crate::rule::counter_style::CssCounterStyleDescriptorValue::from_parsed(
                descriptor,
                value,
                components.clone(),
                value_origin.clone(),
            );
            Ok(value.into_occurrence(name_origin.clone(), value_origin, components))
        },
    )
}

pub(super) fn parse_descriptor_value<'i>(
    input: &mut Stream<'i>,
    kind: CssCounterStyleDescriptorKind,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CounterStyleValueData> {
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
            CounterStyleValueData::Fallback(parse_fallback(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::AdditiveSymbols => {
            CounterStyleValueData::AdditiveSymbols(parse_additive_symbols(input, numeric)?)
        }
        CssCounterStyleDescriptorKind::SpeakAs => {
            CounterStyleValueData::SpeakAs(parse_speak_as(input, numeric)?)
        }
    })
}

fn parse_system<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleSystem> {
    let ident = input.consume_ident()?;
    let system = match_ignore_ascii_case! { ident,
        "cyclic" => CssCounterStyleSystem::Cyclic,
        "numeric" => CssCounterStyleSystem::Numeric,
        "alphabetic" => CssCounterStyleSystem::Alphabetic,
        "symbolic" => CssCounterStyleSystem::Symbolic,
        "additive" => CssCounterStyleSystem::Additive,
        "fixed" => {
            let first_symbol_value = if input.only_whitespace_remains() {
                None
            } else {
                Some(from_component(numeric, input, |input| parse_integer_value(input, numeric))?)
            };
            CssCounterStyleSystem::Fixed(CssCounterStyleFixedSystem::new(first_symbol_value))
        },
        "extends" => {
            let location = input.site();
            let name = input.consume_ident()?;
            let name = CssCounterStyleName::try_new(name.to_string()).ok_or_else(|| {
                fail(numeric, location, unexpected_at)
            })?;
            CssCounterStyleSystem::Extends(name)
        },
        _ => return Err(fail(numeric, input.site(), unexpected_at)),
    };
    expect_exhausted(input)?;
    Ok(system)
}

fn parse_symbols<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterSymbols> {
    let mut symbols = Vec::new();
    while !input.only_whitespace_remains() {
        symbols.push(parse_symbol_component(input, numeric)?);
    }
    if symbols.is_empty() {
        Err(fail(numeric, input.site(), unexpected_at))
    } else {
        Ok(CssCounterSymbols::new(symbols))
    }
}

fn parse_negative<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleNegative> {
    let prefix = parse_symbol_component(input, numeric)?;
    let suffix = if input.only_whitespace_remains() {
        None
    } else {
        Some(parse_symbol_component(input, numeric)?)
    };
    expect_exhausted(input)?;
    Ok(CssCounterStyleNegative::new(prefix, suffix))
}

fn parse_range<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleRange> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        expect_exhausted(input)?;
        return Ok(CssCounterStyleRange::Auto);
    }

    let mut ranges = Vec::new();
    loop {
        let lower_location = input.site();
        let lower = parse_range_bound(input, numeric)?;
        let upper = parse_range_bound(input, numeric)?;
        let interval = CssCounterStyleRangeInterval::try_new(lower, upper)
            .ok_or_else(|| fail(numeric, lower_location, unexpected_at))?;
        ranges.push(interval);
        if input.only_whitespace_remains() {
            break;
        }
        input.consume_comma()?;
    }
    Ok(CssCounterStyleRange::Ranges(
        CssCounterStyleRanges::try_new(ranges).expect("parsed nonempty counter ranges"),
    ))
}

fn parse_range_bound<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleRangeBound> {
    if input
        .attempt(|input| input.consume_keyword("infinite"))
        .is_ok()
    {
        Ok(CssCounterStyleRangeBound::Infinite)
    } else {
        from_component(numeric, input, |input| parse_integer_value(input, numeric))
            .map(CssCounterStyleRangeBound::Integer)
    }
}

fn parse_pad<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStylePad> {
    let (minimum_length, symbol) = if let Ok(minimum_length) =
        input.attempt(|input| parse_nonnegative_integer(input, numeric))
    {
        (minimum_length, parse_symbol_component(input, numeric)?)
    } else {
        let symbol = parse_symbol_component(input, numeric)?;
        (parse_nonnegative_integer(input, numeric)?, symbol)
    };
    expect_exhausted(input)?;
    Ok(
        CssCounterStylePad::try_new(minimum_length, symbol)
            .expect("checked nonnegative pad length"),
    )
}

fn parse_fallback<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleName> {
    let name = parse_counter_style_name_component(input, numeric)?;
    expect_exhausted(input)?;
    Ok(name)
}

fn parse_additive_symbols<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterAdditiveSymbols> {
    let mut tuples = Vec::new();
    loop {
        let (tuple, weight_location) = parse_additive_tuple(input, numeric)?;
        tuples.push(tuple);
        if !CssCounterAdditiveSymbols::weights_strictly_descend(
            &tuples[tuples.len().saturating_sub(2)..],
        ) {
            return Err(fail(numeric, weight_location, unexpected_at));
        }
        if input.only_whitespace_remains() {
            break;
        }
        input.consume_comma()?;
    }
    Ok(CssCounterAdditiveSymbols::try_new(tuples).expect("checked descending additive list"))
}

fn parse_additive_tuple<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, (CssCounterAdditiveTuple, super::syntax_types::Site<'i>)> {
    let initial_location = input.site();
    let (weight, symbol, weight_location) =
        if let Ok(weight) = input.attempt(|input| parse_nonnegative_integer(input, numeric)) {
            (
                weight,
                parse_symbol_component(input, numeric)?,
                initial_location,
            )
        } else {
            let symbol = parse_symbol_component(input, numeric)?;
            let weight_location = input.site();
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

fn parse_speak_as<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleSpeakAs> {
    let location = input.site();
    let ident = input.consume_ident()?;
    let value = match_ignore_ascii_case! { ident,
        "auto" => CssCounterStyleSpeakAs::Auto,
        "bullets" => CssCounterStyleSpeakAs::Bullets,
        "numbers" => CssCounterStyleSpeakAs::Numbers,
        "words" => CssCounterStyleSpeakAs::Words,
        "spell-out" => CssCounterStyleSpeakAs::SpellOut,
        _ => CssCounterStyleName::try_new(ident.to_string())
            .map(CssCounterStyleSpeakAs::CounterStyle)
            .ok_or_else(|| fail(numeric, location, unexpected_at))?,
    };
    expect_exhausted(input)?;
    Ok(value)
}

fn parse_nonnegative_integer<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssNumeric<CssInteger>> {
    let location = input.site();
    let value = from_component(numeric, input, |input| parse_integer_value(input, numeric))?;
    if value.literal().is_some_and(CssNumericLiteral::is_negative) {
        Err(fail(numeric, location, unexpected_at))
    } else {
        Ok(value)
    }
}

fn parse_counter_style_name_component<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterStyleName> {
    let location = input.site();
    let name = input.consume_ident()?;
    CssCounterStyleName::try_new(name.to_string())
        .ok_or_else(|| fail(numeric, location, unexpected_at))
}

fn parse_symbol<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterSymbol> {
    let symbol = parse_symbol_component(input, numeric)?;
    expect_exhausted(input)?;
    Ok(symbol)
}

fn parse_symbol_component<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssCounterSymbol> {
    if let Ok(value) = input.attempt(Stream::consume_string) {
        return CssContentString::try_new(value.to_string())
            .map(CssCounterSymbol::String)
            .ok_or_else(|| fail(numeric, input.site(), unexpected_at));
    }
    let location = input.site();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        return CssCounterSymbolIdent::try_new(ident.to_string())
            .map(CssCounterSymbol::Ident)
            .ok_or_else(|| fail(numeric, location, unexpected_at));
    }
    // An image is selected only after the string/custom-ident alternatives.
    // In particular, `none` is a symbol identifier, not property-level no image.
    // Do not speculate and discard an image provider's typed resource failure.
    from_component(numeric, input, |input| parse_image(input, numeric)).map(CssCounterSymbol::Image)
}

/// The value of a counter-style descriptor from the scope of the native
/// parser `input`.
pub(super) fn parse_authored_value<'i>(
    input: &mut Parser<'i, '_>,
    kind: CssCounterStyleDescriptorKind,
    snapshot: &crate::source::CssSourceSnapshot,
) -> Result<CounterStyleValueData, ParseError<'i, CssError>> {
    let start = input.position().byte_index();
    let end = super::property_input::native_scope_end(input);
    let value = parse_descriptor_text(
        snapshot,
        start,
        end,
        "counter-style",
        kind.as_str(),
        Boundary::ValidatedText,
        |input, components, numeric, _| parse_value_data(input, kind, &components, numeric),
    )?;
    super::property_input::advance_native_to(input, end);
    Ok(value)
}

fn parse_value_data<'i>(
    input: &mut Stream<'i>,
    kind: CssCounterStyleDescriptorKind,
    components: &crate::component::CssComponentValues,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CounterStyleValueData> {
    if super::variables::descriptor_environment_qualifies(components.items(), numeric).map_err(
        |error| {
            fail(numeric, input.site(), |at| {
                crate::diagnostic::error::invalid_component_value(at, error)
            })
        },
    )? {
        let start = input.site();
        consume_remaining(input);
        return Ok(CounterStyleValueData::Pending(
            crate::declaration::CssSubstitutionDependentValue::new(
                crate::declaration::CssAuthoredDeclarationValue::new(
                    input.authored_text_since(start),
                ),
            ),
        ));
    }
    input.discard_whitespace();
    let value = parse_descriptor_value(input, kind, numeric)?;
    expect_exhausted(input)?;
    Ok(value)
}

pub(crate) fn construct_descriptor_value(
    kind: CssCounterStyleDescriptorKind,
    components: &crate::component::CssComponentValues,
    serialized: &crate::component::CssSerializedValue,
) -> Result<CounterStyleValueData, CssError> {
    let numeric = NumericInputContext::components(components, serialized);
    let result = (|| {
        let document = crate::syntax_consumption::checked_document(
            components.items(),
            crate::syntax_consumption::SyntaxInputLimits::default(),
            0,
        )
        .map_err(|error| {
            crate::diagnostic::error::invalid_component_value(
                cssparser::SourceLocation { line: 0, column: 1 },
                error,
            )
        })?;
        let mut input = document.root();
        validate_root(&mut input, &numeric, "counter-style", kind.as_str())
            .map_err(|error| native(&numeric, &error))?;
        parse_descriptor_checked(
            &mut input,
            &numeric,
            "counter-style",
            kind.as_str(),
            |input| parse_value_data(input, kind, components, &numeric),
        )
    })();
    result.map_err(|error| {
        crate::diagnostic::error::from_parse_error(
            serialized.as_css(),
            crate::diagnostic::error::with_descriptor_value_context(
                error,
                "counter-style",
                kind.as_str(),
            ),
        )
    })
}
