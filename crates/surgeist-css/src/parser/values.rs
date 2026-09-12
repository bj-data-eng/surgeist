use cssparser::{
    ParseError, Parser, ParserInput, ToCss, Token,
    color::PredefinedColorSpace as ParsedPredefinedColorSpace, match_ignore_ascii_case,
};
use cssparser_color::{Color as ParsedColor, DefaultColorParser, parse_color_with};

use crate::error::{
    CssFeatureId, Error, basic, invalid_color, unsupported_value_at, with_color_context,
};
use crate::syntax::*;
use crate::validation::{LengthUnitStatus, classify_length_unit, parse_global_keyword};

pub(crate) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.syntax-token-stream"),
    CssFeatureId::new("official.value.component-value"),
    CssFeatureId::new("official.value.simple-block"),
    CssFeatureId::new("official.value.function"),
    CssFeatureId::new("official.value.declaration-value"),
    CssFeatureId::new("official.value.any-value"),
    CssFeatureId::new("official.value.an-plus-b"),
    CssFeatureId::new("official.value.unicode-range"),
    CssFeatureId::new("official.value.css-wide-keyword"),
    CssFeatureId::new("official.value.custom-ident"),
    CssFeatureId::new("official.value.ident"),
    CssFeatureId::new("official.value.string"),
    CssFeatureId::new("official.value.url"),
    CssFeatureId::new("official.value.url-modifier"),
    CssFeatureId::new("official.value.integer"),
    CssFeatureId::new("official.value.number"),
    CssFeatureId::new("official.value.dimension"),
    CssFeatureId::new("official.value.percentage"),
    CssFeatureId::new("official.value.length"),
    CssFeatureId::new("official.value.length-percentage"),
    CssFeatureId::new("official.value.angle"),
    CssFeatureId::new("official.value.angle-percentage"),
    CssFeatureId::new("official.value.time"),
    CssFeatureId::new("official.value.time-percentage"),
    CssFeatureId::new("official.value.frequency"),
    CssFeatureId::new("official.value.frequency-percentage"),
    CssFeatureId::new("official.value.resolution"),
    CssFeatureId::new("official.value.calc"),
    CssFeatureId::new("official.value.color"),
    CssFeatureId::new("official.value.alpha"),
    CssFeatureId::new("official.value.hue"),
    CssFeatureId::new("official.value.rgb"),
    CssFeatureId::new("official.value.hex-color"),
    CssFeatureId::new("official.value.named-color"),
    CssFeatureId::new("official.value.system-color"),
    CssFeatureId::new("official.value.deprecated-system-color"),
    CssFeatureId::new("official.value.transparent"),
    CssFeatureId::new("official.value.currentcolor"),
    CssFeatureId::new("official.value.hsl"),
    CssFeatureId::new("official.value.hwb"),
    CssFeatureId::new("official.value.lab"),
    CssFeatureId::new("official.value.lch"),
    CssFeatureId::new("official.value.oklab"),
    CssFeatureId::new("official.value.oklch"),
    CssFeatureId::new("official.value.predefined-color"),
    CssFeatureId::new("ext.value.relative-color"),
    CssFeatureId::new("ext.value.relative-color.rgb"),
    CssFeatureId::new("ext.value.relative-color.hsl"),
    CssFeatureId::new("ext.value.relative-color.hwb"),
    CssFeatureId::new("ext.value.relative-color.lab"),
    CssFeatureId::new("ext.value.relative-color.oklab"),
    CssFeatureId::new("ext.value.relative-color.lch"),
    CssFeatureId::new("ext.value.relative-color.oklch"),
    CssFeatureId::new("ext.value.relative-color.predefined"),
    CssFeatureId::new("ext.value.color-mix"),
];

pub(super) fn parse_box_size_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::BoxSize)
}

pub(super) fn parse_inset_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::Inset)
}

pub(super) fn parse_margin_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::Margin)
}

pub(super) fn parse_padding_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::Padding)
}

pub(super) fn parse_border_width_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::BorderWidth)
}

pub(super) fn parse_radius_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::Radius)
}

pub(super) fn parse_shadow_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::ShadowOffset)
}

pub(super) fn parse_shadow_blur_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::ShadowBlur)
}

pub(super) fn parse_gap_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        Ok(CssLength::Normal)
    } else {
        parse_length_with(input, numeric, LengthGrammar::Gap)
    }
}

#[derive(Clone, Copy)]
pub(super) enum LengthGrammar {
    BoxSize,
    FlowTolerance,
    Inset,
    Margin,
    Padding,
    BorderWidth,
    Radius,
    ShadowOffset,
    ShadowBlur,
    BorderSpacing,
    Clip,
    OutlineOffset,
    Gap,
    FontSize,
    LineHeight,
    TextIndent,
    VerticalAlign,
    LetterSpacing,
    WordSpacing,
    TextDecorationThickness,
    GridTrack,
    BackgroundSize,
    Position,
    NonNegativeLength,
}

impl LengthGrammar {
    const fn allows_percent(self) -> bool {
        matches!(
            self,
            Self::BoxSize
                | Self::FlowTolerance
                | Self::Inset
                | Self::Margin
                | Self::Padding
                | Self::Radius
                | Self::Gap
                | Self::FontSize
                | Self::LineHeight
                | Self::TextIndent
                | Self::VerticalAlign
                | Self::TextDecorationThickness
                | Self::GridTrack
                | Self::BackgroundSize
                | Self::Position
        )
    }

    const fn allows_auto(self) -> bool {
        matches!(self, Self::BoxSize | Self::Inset | Self::Margin)
    }

    const fn allows_intrinsic(self) -> bool {
        matches!(self, Self::BoxSize | Self::Inset)
    }

    const fn allows_normal(self) -> bool {
        matches!(self, Self::Gap | Self::LineHeight)
    }

    const fn allows_line_width_keyword(self) -> bool {
        matches!(self, Self::BorderWidth)
    }

    const fn allows_calc_percent(self) -> bool {
        matches!(
            self,
            Self::BoxSize
                | Self::FlowTolerance
                | Self::Inset
                | Self::Margin
                | Self::Padding
                | Self::Radius
                | Self::Gap
                | Self::FontSize
                | Self::LineHeight
                | Self::TextIndent
                | Self::VerticalAlign
                | Self::TextDecorationThickness
                | Self::GridTrack
                | Self::BackgroundSize
                | Self::Position
        )
    }

    const fn requires_non_negative(self) -> bool {
        matches!(
            self,
            Self::BoxSize
                | Self::Padding
                | Self::BorderWidth
                | Self::Radius
                | Self::ShadowBlur
                | Self::BorderSpacing
                | Self::TextDecorationThickness
                | Self::GridTrack
                | Self::BackgroundSize
                | Self::NonNegativeLength
        )
    }

    const fn context(self) -> &'static str {
        match self {
            Self::BoxSize => "box size",
            Self::FlowTolerance => "flow-tolerance",
            Self::Inset => "inset",
            Self::Margin => "margin",
            Self::Padding => "padding",
            Self::BorderWidth => "border-width",
            Self::Radius => "border-radius",
            Self::ShadowOffset => "box-shadow",
            Self::ShadowBlur => "box-shadow blur",
            Self::BorderSpacing => "border-spacing",
            Self::Clip => "clip",
            Self::OutlineOffset => "outline-offset",
            Self::Gap => "gap",
            Self::FontSize => "font-size",
            Self::LineHeight => "line-height",
            Self::TextIndent => "text-indent",
            Self::VerticalAlign => "vertical-align",
            Self::LetterSpacing => "letter-spacing",
            Self::WordSpacing => "word-spacing",
            Self::TextDecorationThickness => "text-decoration-thickness",
            Self::GridTrack => "grid track",
            Self::BackgroundSize => "background-size",
            Self::Position => "position",
            Self::NonNegativeLength => "non-negative length",
        }
    }
}

pub(super) fn checked_percentage_value<'i>(
    location: cssparser::SourceLocation,
    unit_value: f32,
    non_finite_reason: impl Into<String>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    let value = unit_value * 100.0;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(unsupported_value_at(location, None, non_finite_reason))
    }
}

pub(super) fn parse_length_with<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    grammar: LengthGrammar,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context(input, numeric, grammar, grammar.context())
}

pub(super) fn parse_length_with_context<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    grammar: LengthGrammar,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context_mode(input, grammar, context, Some(numeric))
}

pub(super) fn parse_length_with_context_legacy<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context_mode(input, grammar, context, None)
}

fn parse_length_with_context_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
    context: &str,
    numeric: Option<&NumericInputContext<'_>>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    let before_opener = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, .. } if !value.is_finite() => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported non-finite {context} length"),
        )),
        Token::Dimension { value, unit, .. } => match classify_length_unit(unit) {
            LengthUnitStatus::Supported(_) if grammar.requires_non_negative() && *value < 0.0 => {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("unsupported negative {context} length"),
                ))
            }
            LengthUnitStatus::Supported(unit) => Ok(CssLength::dimension(*value, unit)),
            LengthUnitStatus::Unknown => Err(unsupported_value_at(
                location,
                None,
                format!("unknown {context} unit `{unit}`"),
            )),
        },
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                format!("unsupported non-finite {context} percentage"),
            )?;
            if grammar.requires_non_negative() && value < 0.0 {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("unsupported negative {context} percentage"),
                ))
            } else if grammar.allows_percent() {
                Ok(CssLength::percent(value))
            } else {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("unsupported {context} percentage"),
                ))
            }
        }
        Token::Number { value, .. } if *value == 0.0 => Ok(CssLength::Zero),
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "auto" if grammar.allows_auto() => Ok(CssLength::Auto),
            "normal" if grammar.allows_normal() => Ok(CssLength::Normal),
            "thin" if grammar.allows_line_width_keyword() => Ok(CssLength::Thin),
            "medium" if grammar.allows_line_width_keyword() => Ok(CssLength::Medium),
            "thick" if grammar.allows_line_width_keyword() => Ok(CssLength::Thick),
            "min-content" if grammar.allows_intrinsic() => Ok(CssLength::MinContent),
            "max-content" if grammar.allows_intrinsic() => Ok(CssLength::MaxContent),
            "fit-content" if grammar.allows_intrinsic() => Ok(CssLength::FitContent),
            _ => Err(unsupported_value_at(
                location,
                None,
                format!("unsupported {context} `{ident}`"),
            )),
        },
        Token::Function(name) if is_math_function(name) => {
            if let Some(numeric) = numeric {
                let root = if grammar.allows_calc_percent() {
                    CalculationRoot::LengthPercentage
                } else {
                    CalculationRoot::Length
                };
                let expression = parse_numeric_function(input, &before_opener, numeric, root)?;
                Ok(CssLength::Calc(CssCalcLength::Typed(
                    CssLengthPercentageCalculation::from_expression(expression),
                )))
            } else if name.eq_ignore_ascii_case("calc") {
                input
                    .parse_nested_block(|input| {
                        parse_legacy_calc_length_with_grammar(input, grammar)
                    })
                    .map(CssLength::Calc)
            } else {
                Err(calculation_error(location))
            }
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported length function `{name}` for {context}"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

use crate::numeric::NumericInputContext;
pub(super) use crate::numeric::{CalculationRoot, is_math_function};

pub(super) fn parse_numeric_function<'i, 't>(
    input: &mut Parser<'i, 't>,
    before_opener: &cssparser::ParserState,
    numeric: &NumericInputContext<'_>,
    root: CalculationRoot,
) -> Result<CssCalculationExpression, ParseError<'i, Error>> {
    input.reset(before_opener);
    input.skip_whitespace();
    let root_offset = input.position().byte_index();
    let location = input.current_source_location();
    let component = numeric.collect(input).map_err(|error| {
        calculation_error(numeric.error_location(&error, location, root_offset))
    })?;
    let values = crate::CssComponentValues::try_new(vec![component])
        .map_err(|_| calculation_error(location))?;
    numeric
        .admit(values, root)
        .map_err(|error| calculation_error(numeric.error_location(&error, location, root_offset)))
}

fn calculation_error<'i>(location: cssparser::SourceLocation) -> ParseError<'i, Error> {
    unsupported_value_at(location, None, "invalid typed calculation")
}

pub(super) fn parse_legacy_calc_length_with_grammar<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
) -> std::result::Result<CssCalcLength, ParseError<'i, Error>> {
    let first = CssCalcLengthTerm::add(parse_calc_component(input, grammar)?);
    let mut terms = Vec::new();

    while !input.is_exhausted() {
        let location = input.current_source_location();
        let operator = match input.next().map_err(basic)? {
            Token::Delim('+') => CssCalcLengthTerm::add,
            Token::Delim('-') => CssCalcLengthTerm::sub,
            token => {
                return Err(unsupported_value_at(
                    location,
                    None,
                    format!("expected calc operator, got `{}`", token.to_css_string()),
                ));
            }
        };
        let component = parse_calc_component(input, grammar)?;
        terms.push(operator(component));
    }

    Ok(CssCalcLength::sum(first, terms))
}

pub(super) fn parse_calc_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
) -> std::result::Result<CssCalcLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, .. } if !value.is_finite() => Err(unsupported_value_at(
            location,
            None,
            "unsupported non-finite calc length",
        )),
        Token::Dimension { value, unit, .. } => match classify_length_unit(unit) {
            LengthUnitStatus::Supported(_) if grammar.requires_non_negative() && *value < 0.0 => {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported negative calc length",
                ))
            }
            LengthUnitStatus::Supported(unit) => Ok(CssCalcLength::dimension(*value, unit)),
            LengthUnitStatus::Unknown => Err(unsupported_value_at(
                location,
                None,
                format!("unknown calc length unit `{unit}`"),
            )),
        },
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                "unsupported non-finite calc percentage",
            )?;
            if grammar.requires_non_negative() && value < 0.0 {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported negative calc percentage",
                ))
            } else if grammar.allows_calc_percent() {
                Ok(CssCalcLength::percent(value))
            } else {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported calc percentage",
                ))
            }
        }
        Token::Number { value, .. } if *value == 0.0 => Ok(CssCalcLength::px(0.0)),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(|input| parse_legacy_calc_length_with_grammar(input, grammar))
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported calc function `{name}`"),
        )),
        token => Err(unsupported_value_at(
            location,
            None,
            format!("unexpected calc token `{}`", token.to_css_string()),
        )),
    }
}

pub(super) fn parse_number<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    input.expect_number().map_err(basic)
}

pub(super) fn parse_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<i32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => Ok(*value),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            format!("{context} must be an integer"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_positive_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<i32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_integer(input, context)?;
    if value <= 0 {
        Err(unsupported_value_at(
            location,
            None,
            format!("{context} must be a positive integer"),
        ))
    } else {
        Ok(value)
    }
}

pub(super) fn parse_custom_ident_from_str_at<'i>(
    context: &str,
    ident: &str,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssCustomIdent, ParseError<'i, Error>> {
    if ident.is_empty()
        || parse_global_keyword(ident).is_some()
        || ident.eq_ignore_ascii_case("span")
        || ident.eq_ignore_ascii_case("auto")
    {
        Err(unsupported_value_at(
            location,
            None,
            format!("unsupported {context} `{ident}`"),
        ))
    } else {
        Ok(CssCustomIdent::new(ident))
    }
}

pub(super) fn next_is_delim<'i, 't>(input: &mut Parser<'i, 't>, delim: char) -> bool {
    let state = input.state();
    let is_delim = input.try_parse(|input| input.expect_delim(delim)).is_ok();
    input.reset(&state);
    is_delim
}

pub(super) fn next_is_comma<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_comma = input.try_parse(Parser::expect_comma).is_ok();
    input.reset(&state);
    is_comma
}

pub(super) fn next_is_ident<'i, 't>(input: &mut Parser<'i, 't>, expected: &str) -> bool {
    let state = input.state();
    let is_ident = input
        .try_parse(|input| input.expect_ident_matching(expected))
        .is_ok();
    input.reset(&state);
    is_ident
}

pub(super) fn parse_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssParsedColor, ParseError<'i, Error>> {
    let start = input.position();
    if next_is_authored_relative_color(input) {
        let current = parse_authored_relative_color(input, numeric)
            .map_err(|error| with_color_context(error, None))?;
        let i01_subset = parse_compatibility_color_text(input.slice_from(start));
        return Ok(CssParsedColor::new(current, i01_subset));
    }
    if next_is_color_mix(input) {
        let current = parse_authored_color_mix(input, numeric)
            .map_err(|error| with_color_context(error, None))?;
        let i01_subset = parse_compatibility_color_text(input.slice_from(start));
        return Ok(CssParsedColor::new(current, i01_subset));
    }
    if let Ok(color) = input.try_parse(parse_compatibility_only_predefined_color) {
        return Ok(CssParsedColor::from_i01(color));
    }
    let start = input.position();
    if next_is_selected_authored_color(input) {
        let current = parse_selected_authored_color(input, numeric)
            .map_err(|error| with_color_context(error, None))?;
        let i01_subset = current
            .has_exact_i01_projection()
            .then(|| parse_compatibility_color_text(input.slice_from(start)))
            .flatten();
        return Ok(CssParsedColor::new(current, i01_subset));
    }
    parse_color_inner(input)
        .map(CssParsedColor::from_i01)
        .map_err(|error| with_color_context(error, None))
}

fn next_is_color_mix<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_color_mix = matches!(
        input.next(),
        Ok(Token::Function(name)) if name.eq_ignore_ascii_case("color-mix")
    );
    input.reset(&state);
    is_color_mix
}

fn next_is_authored_relative_color<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_relative = match input.next() {
        Ok(Token::Function(name)) if relative_color_function_from_name(name).is_some() => input
            .parse_nested_block(|input| {
                input.expect_ident_matching("from").map_err(basic)?;
                while input.next_including_whitespace().is_ok() {}
                Ok(())
            })
            .is_ok(),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    is_relative
}

fn parse_compatibility_only_predefined_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let color = parse_color_inner(input)?;
    if matches!(
        color,
        CssColor::ColorFunction(ref value)
            if value.color_space() == CssPredefinedColorSpace::DisplayP3Linear
    ) {
        Ok(color)
    } else {
        Err(invalid_color(location, None))
    }
}

fn next_is_selected_authored_color<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let selected = match input.next() {
        Ok(Token::Ident(_) | Token::Hash(_) | Token::IDHash(_)) => true,
        Ok(Token::Function(name)) => {
            name.eq_ignore_ascii_case("rgb")
                || name.eq_ignore_ascii_case("rgba")
                || name.eq_ignore_ascii_case("hsl")
                || name.eq_ignore_ascii_case("hsla")
                || name.eq_ignore_ascii_case("hwb")
                || name.eq_ignore_ascii_case("lab")
                || name.eq_ignore_ascii_case("lch")
                || name.eq_ignore_ascii_case("oklab")
                || name.eq_ignore_ascii_case("oklch")
                || name.eq_ignore_ascii_case("color")
        }
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    selected
}

fn parse_compatibility_color_text(source: &str) -> Option<CssColor> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let color = parse_color_inner(&mut parser).ok()?;
    parser.expect_exhausted().ok()?;
    Some(color)
}

fn parse_selected_authored_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("currentcolor") => {
            Ok(CssAuthoredColor::current_color())
        }
        Token::Ident(ident) if ident.eq_ignore_ascii_case("transparent") => {
            Ok(CssAuthoredColor::transparent())
        }
        Token::Ident(ident) => {
            if let Some(system) = authored_system_color(&ident) {
                return Ok(CssAuthoredColor::from_system(system));
            }
            if parse_compatibility_color_text(&ident).is_some() {
                return Ok(CssAuthoredColor::from_named(CssNamedColor::new(
                    ident.to_ascii_lowercase(),
                )));
            }
            Err(invalid_color(location, None))
        }
        Token::Hash(digits) | Token::IDHash(digits)
            if matches!(digits.len(), 3 | 4 | 6 | 8)
                && digits.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            Ok(CssAuthoredColor::hex(CssHexColor::new(digits.as_ref())))
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") =>
        {
            input
                .parse_nested_block(|input| parse_authored_rgb(input, numeric))
                .map(CssAuthoredColor::rgb)
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") =>
        {
            input
                .parse_nested_block(|input| parse_authored_hsl(input, numeric))
                .map(CssAuthoredColor::hsl)
        }
        Token::Function(name) if name.eq_ignore_ascii_case("hwb") => input
            .parse_nested_block(|input| parse_authored_hwb(input, numeric))
            .map(CssAuthoredColor::hwb),
        Token::Function(name) if name.eq_ignore_ascii_case("lab") => input
            .parse_nested_block(|input| parse_authored_lab(input, numeric))
            .map(CssAuthoredColor::lab),
        Token::Function(name) if name.eq_ignore_ascii_case("lch") => input
            .parse_nested_block(|input| parse_authored_lch(input, numeric))
            .map(CssAuthoredColor::lch),
        Token::Function(name) if name.eq_ignore_ascii_case("oklab") => input
            .parse_nested_block(|input| parse_authored_lab(input, numeric))
            .map(CssAuthoredColor::oklab),
        Token::Function(name) if name.eq_ignore_ascii_case("oklch") => input
            .parse_nested_block(|input| parse_authored_lch(input, numeric))
            .map(CssAuthoredColor::oklch),
        Token::Function(name) if name.eq_ignore_ascii_case("color") => input
            .parse_nested_block(|input| parse_authored_predefined_color(input, numeric))
            .map(CssAuthoredColor::predefined),
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            None,
        )),
    }
}

fn parse_authored_rgb<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredRgbColor, ParseError<'i, Error>> {
    let first = parse_authored_color_component(input, numeric, true)?;
    if input.try_parse(Parser::expect_comma).is_ok() {
        if first.is_none() {
            return Err(invalid_color(input.current_source_location(), Some("red")));
        }
        let domain = first.domain();
        let second_location = input.current_source_location();
        let second = parse_authored_color_component(input, numeric, false)?;
        if second.domain() != domain {
            return Err(invalid_color(second_location, Some("component")));
        }
        input.expect_comma().map_err(basic)?;
        let third_location = input.current_source_location();
        let third = parse_authored_color_component(input, numeric, false)?;
        if third.domain() != domain {
            return Err(invalid_color(third_location, Some("component")));
        }
        let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
            Some(parse_authored_alpha(input, numeric, false)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredRgbColor::new(
            CssAuthoredColorSyntax::Legacy,
            [first, second, third],
            alpha,
        ))
    } else {
        let second = parse_authored_color_component(input, numeric, true)?;
        let third = parse_authored_color_component(input, numeric, true)?;
        let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            Some(parse_authored_alpha(input, numeric, true)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredRgbColor::new(
            CssAuthoredColorSyntax::Modern,
            [first, second, third],
            alpha,
        ))
    }
}

fn parse_authored_hsl<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredHslColor, ParseError<'i, Error>> {
    let hue = parse_authored_hue(input, numeric, true)?;
    if input.try_parse(Parser::expect_comma).is_ok() {
        if hue.is_none() {
            return Err(invalid_color(input.current_source_location(), Some("hue")));
        }
        let saturation = parse_authored_percentage_component(input, numeric, false)?;
        input.expect_comma().map_err(basic)?;
        let lightness = parse_authored_percentage_component(input, numeric, false)?;
        let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
            Some(parse_authored_alpha(input, numeric, false)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredHslColor::new(
            CssAuthoredColorSyntax::Legacy,
            hue,
            saturation,
            lightness,
            alpha,
        ))
    } else {
        let saturation = parse_authored_percentage_component(input, numeric, true)?;
        let lightness = parse_authored_percentage_component(input, numeric, true)?;
        let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            Some(parse_authored_alpha(input, numeric, true)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredHslColor::new(
            CssAuthoredColorSyntax::Modern,
            hue,
            saturation,
            lightness,
            alpha,
        ))
    }
}

fn parse_authored_hwb<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredHwbColor, ParseError<'i, Error>> {
    let hue = parse_authored_hue(input, numeric, true)?;
    let whiteness = parse_authored_percentage_component(input, numeric, true)?;
    let blackness = parse_authored_percentage_component(input, numeric, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredHwbColor::new(hue, whiteness, blackness, alpha))
}

fn parse_authored_lab<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredLabColor, ParseError<'i, Error>> {
    let lightness = parse_authored_color_component(input, numeric, true)?;
    let a = parse_authored_color_component(input, numeric, true)?;
    let b = parse_authored_color_component(input, numeric, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredLabColor::new(lightness, a, b, alpha))
}

fn parse_authored_lch<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredLchColor, ParseError<'i, Error>> {
    let lightness = parse_authored_color_component(input, numeric, true)?;
    let chroma = parse_authored_color_component(input, numeric, true)?;
    let hue = parse_authored_hue(input, numeric, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredLchColor::new(lightness, chroma, hue, alpha))
}

fn parse_authored_predefined_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredPredefinedColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color_space = match_ignore_ascii_case! { &ident,
        "srgb" => CssPredefinedColorSpace::Srgb,
        "srgb-linear" => CssPredefinedColorSpace::SrgbLinear,
        "display-p3" => CssPredefinedColorSpace::DisplayP3,
        "a98-rgb" => CssPredefinedColorSpace::A98Rgb,
        "prophoto-rgb" => CssPredefinedColorSpace::ProphotoRgb,
        "rec2020" => CssPredefinedColorSpace::Rec2020,
        "xyz" | "xyz-d65" => CssPredefinedColorSpace::XyzD65,
        "xyz-d50" => CssPredefinedColorSpace::XyzD50,
        _ => {
            return Err(with_color_context(
                location.new_unexpected_token_error::<Error>(Token::Ident(ident)),
                Some("color space"),
            ));
        }
    };
    let channels = [
        parse_authored_color_component(input, numeric, true)?,
        parse_authored_color_component(input, numeric, true)?,
        parse_authored_color_component(input, numeric, true)?,
    ];
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredPredefinedColor::new(
        color_space,
        channels,
        alpha,
    ))
}

fn parse_authored_color_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    let before_opener = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(ident) if allow_none && ident.eq_ignore_ascii_case("none") => {
            Ok(CssAuthoredColorComponent::None)
        }
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssAuthoredColorComponent::Number)
            .ok_or_else(|| invalid_color(location, Some("component"))),
        Token::Percentage { unit_value, .. } => CssFiniteNumber::try_new(unit_value * 100.0)
            .map(CssAuthoredColorComponent::Percentage)
            .ok_or_else(|| invalid_color(location, Some("component"))),
        Token::Function(name) if is_math_function(&name) => {
            parse_authored_number_or_percentage_calculation(
                input,
                numeric,
                &before_opener,
                location,
            )
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            Some("component"),
        )),
    }
}

fn parse_authored_percentage_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_authored_color_component(input, numeric, allow_none)?;
    if matches!(
        value,
        CssAuthoredColorComponent::None
            | CssAuthoredColorComponent::Percentage(_)
            | CssAuthoredColorComponent::PercentageCalculation(_)
    ) {
        Ok(value)
    } else {
        Err(invalid_color(location, Some("percentage")))
    }
}

fn parse_authored_alpha<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    parse_authored_color_component(input, numeric, allow_none)
}

fn parse_authored_number_or_percentage_calculation<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    before_opener: &cssparser::ParserState,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    if let Ok(expression) = input.try_parse(|input| {
        parse_numeric_function(input, before_opener, numeric, CalculationRoot::Number)
    }) {
        return Ok(CssAuthoredColorComponent::NumberCalculation(
            CssNumberCalculation::from_expression(expression),
        ));
    }
    parse_numeric_function(input, before_opener, numeric, CalculationRoot::Percentage)
        .map(CssPercentageCalculation::from_expression)
        .map(CssAuthoredColorComponent::PercentageCalculation)
        .map_err(|_| invalid_color(location, Some("component")))
}

fn parse_authored_hue<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredHue, ParseError<'i, Error>> {
    let before_opener = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(ident) if allow_none && ident.eq_ignore_ascii_case("none") => {
            Ok(CssAuthoredHue::None)
        }
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssAuthoredHue::Number)
            .ok_or_else(|| invalid_color(location, Some("hue"))),
        token @ Token::Dimension { .. } => {
            let Token::Dimension {
                value, ref unit, ..
            } = token
            else {
                unreachable!("matched dimension token")
            };
            let unit = match unit.to_ascii_lowercase().as_str() {
                "deg" => CssAngleUnit::Degrees,
                "grad" => CssAngleUnit::Gradians,
                "rad" => CssAngleUnit::Radians,
                "turn" => CssAngleUnit::Turns,
                _ => {
                    return Err(with_color_context(
                        location.new_unexpected_token_error::<Error>(token),
                        Some("hue"),
                    ));
                }
            };
            CssAngleLiteral::try_new(value, unit)
                .map(CssAuthoredHue::Angle)
                .ok_or_else(|| invalid_color(location, Some("hue")))
        }
        Token::Function(name) if is_math_function(&name) => {
            if let Ok(expression) = input.try_parse(|input| {
                parse_numeric_function(input, &before_opener, numeric, CalculationRoot::Number)
            }) {
                return Ok(CssAuthoredHue::NumberCalculation(
                    CssNumberCalculation::from_expression(expression),
                ));
            }
            parse_numeric_function(input, &before_opener, numeric, CalculationRoot::Angle)
                .map(CssAngleCalculation::from_expression)
                .map(CssAuthoredHue::AngleCalculation)
                .map_err(|_| invalid_color(location, Some("hue")))
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            Some("hue"),
        )),
    }
}

fn authored_system_color(ident: &str) -> Option<CssAuthoredSystemColor> {
    let value = match_ignore_ascii_case! { ident,
        "canvas" => CssAuthoredSystemColor::Canvas,
        "canvastext" => CssAuthoredSystemColor::CanvasText,
        "linktext" => CssAuthoredSystemColor::LinkText,
        "visitedtext" => CssAuthoredSystemColor::VisitedText,
        "activetext" => CssAuthoredSystemColor::ActiveText,
        "buttonface" => CssAuthoredSystemColor::ButtonFace,
        "buttontext" => CssAuthoredSystemColor::ButtonText,
        "buttonborder" => CssAuthoredSystemColor::ButtonBorder,
        "field" => CssAuthoredSystemColor::Field,
        "fieldtext" => CssAuthoredSystemColor::FieldText,
        "highlight" => CssAuthoredSystemColor::Highlight,
        "highlighttext" => CssAuthoredSystemColor::HighlightText,
        "mark" => CssAuthoredSystemColor::Mark,
        "marktext" => CssAuthoredSystemColor::MarkText,
        "graytext" => CssAuthoredSystemColor::GrayText,
        "selecteditem" => CssAuthoredSystemColor::SelectedItem,
        "selecteditemtext" => CssAuthoredSystemColor::SelectedItemText,
        "accentcolor" => CssAuthoredSystemColor::AccentColor,
        "accentcolortext" => CssAuthoredSystemColor::AccentColorText,
        "activeborder" => CssAuthoredSystemColor::ActiveBorder,
        "activecaption" => CssAuthoredSystemColor::ActiveCaption,
        "appworkspace" => CssAuthoredSystemColor::AppWorkspace,
        "background" => CssAuthoredSystemColor::Background,
        "buttonhighlight" => CssAuthoredSystemColor::ButtonHighlight,
        "buttonshadow" => CssAuthoredSystemColor::ButtonShadow,
        "captiontext" => CssAuthoredSystemColor::CaptionText,
        "inactiveborder" => CssAuthoredSystemColor::InactiveBorder,
        "inactivecaption" => CssAuthoredSystemColor::InactiveCaption,
        "inactivecaptiontext" => CssAuthoredSystemColor::InactiveCaptionText,
        "infobackground" => CssAuthoredSystemColor::InfoBackground,
        "infotext" => CssAuthoredSystemColor::InfoText,
        "menu" => CssAuthoredSystemColor::Menu,
        "menutext" => CssAuthoredSystemColor::MenuText,
        "scrollbar" => CssAuthoredSystemColor::Scrollbar,
        "threeddarkshadow" => CssAuthoredSystemColor::ThreeDDarkShadow,
        "threedface" => CssAuthoredSystemColor::ThreeDFace,
        "threedhighlight" => CssAuthoredSystemColor::ThreeDHighlight,
        "threedlightshadow" => CssAuthoredSystemColor::ThreeDLightShadow,
        "threedshadow" => CssAuthoredSystemColor::ThreeDShadow,
        "window" => CssAuthoredSystemColor::Window,
        "windowframe" => CssAuthoredSystemColor::WindowFrame,
        "windowtext" => CssAuthoredSystemColor::WindowText,
        _ => return None,
    };
    Some(value)
}

fn parse_color_inner<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    if let Ok(color) = input.try_parse(parse_relative_color) {
        return Ok(color);
    }
    if let Ok(color) = input.try_parse(parse_color_mix) {
        return Ok(color);
    }
    if let Ok(color) = input.try_parse(parse_absolute_color_with_cssparser_color) {
        return Ok(color);
    }
    if let Ok(color) = input.try_parse(parse_system_color) {
        return Ok(color);
    }
    Err(invalid_color(input.current_source_location(), None))
}

fn parse_authored_relative_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let Token::Function(name) = token else {
        return Err(location.new_unexpected_token_error(token));
    };
    let Some(function) = relative_color_function_from_name(&name) else {
        return Err(location.new_unexpected_token_error(Token::Function(name)));
    };
    input
        .parse_nested_block(|input| {
            parse_authored_relative_color_arguments(input, numeric, function)
        })
        .map(CssAuthoredColor::relative)
}

fn parse_authored_relative_color_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    function: RelativeColorFunction,
) -> std::result::Result<CssAuthoredRelativeColor, ParseError<'i, Error>> {
    input.expect_ident_matching("from").map_err(basic)?;
    let (source, _) = parse_color(input, numeric)?.into_parts();
    let (function, environment, domains) = relative_color_signature(input, function)?;
    let channels = [
        parse_typed_relative_color_expression(input, numeric, environment, domains[0])?,
        parse_typed_relative_color_expression(input, numeric, environment, domains[1])?,
        parse_typed_relative_color_expression(input, numeric, environment, domains[2])?,
    ];
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_typed_relative_color_expression(
            input,
            numeric,
            environment,
            CssRelativeColorResultDomain::Alpha,
        )?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredRelativeColor::new(
        function,
        environment,
        source,
        channels,
        alpha,
    ))
}

fn relative_color_signature<'i, 't>(
    input: &mut Parser<'i, 't>,
    function: RelativeColorFunction,
) -> std::result::Result<
    (
        CssRelativeColorFunction,
        CssRelativeColorEnvironment,
        [CssRelativeColorResultDomain; 3],
    ),
    ParseError<'i, Error>,
> {
    use CssRelativeColorResultDomain::{Hue, NumberPercentage};
    let signature = match function {
        RelativeColorFunction::Rgb => (
            CssRelativeColorFunction::Rgb,
            CssRelativeColorEnvironment::Rgb,
            [NumberPercentage; 3],
        ),
        RelativeColorFunction::Hsl => (
            CssRelativeColorFunction::Hsl,
            CssRelativeColorEnvironment::Hsl,
            [Hue, NumberPercentage, NumberPercentage],
        ),
        RelativeColorFunction::Hwb => (
            CssRelativeColorFunction::Hwb,
            CssRelativeColorEnvironment::Hwb,
            [Hue, NumberPercentage, NumberPercentage],
        ),
        RelativeColorFunction::Lab => (
            CssRelativeColorFunction::Lab,
            CssRelativeColorEnvironment::Lab,
            [NumberPercentage; 3],
        ),
        RelativeColorFunction::Lch => (
            CssRelativeColorFunction::Lch,
            CssRelativeColorEnvironment::Lch,
            [NumberPercentage, NumberPercentage, Hue],
        ),
        RelativeColorFunction::Oklab => (
            CssRelativeColorFunction::Oklab,
            CssRelativeColorEnvironment::Oklab,
            [NumberPercentage; 3],
        ),
        RelativeColorFunction::Oklch => (
            CssRelativeColorFunction::Oklch,
            CssRelativeColorEnvironment::Oklch,
            [NumberPercentage, NumberPercentage, Hue],
        ),
        RelativeColorFunction::Color => {
            let space = parse_relative_predefined_color_space(input)?;
            let environment = match space {
                CssPredefinedColorSpace::XyzD50 | CssPredefinedColorSpace::XyzD65 => {
                    CssRelativeColorEnvironment::Xyz(space)
                }
                _ => CssRelativeColorEnvironment::PredefinedRgb(space),
            };
            (
                CssRelativeColorFunction::Color(space),
                environment,
                [NumberPercentage; 3],
            )
        }
    };
    Ok(signature)
}

fn parse_typed_relative_color_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
) -> std::result::Result<CssTypedRelativeColorExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.position();
    let before_opener = input.state();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let value = match token {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("none") => {
            CssRelativeColorExpressionValue::None
        }
        Token::Ident(ident) => {
            let Some(channel) = relative_color_channel(environment, &ident) else {
                return Err(with_color_context(
                    location.new_unexpected_token_error::<Error>(Token::Ident(ident)),
                    Some("relative channel"),
                ));
            };
            CssRelativeColorExpressionValue::Channel(channel)
        }
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssRelativeColorExpressionValue::Number)
            .ok_or_else(|| invalid_color(location, Some("relative channel")))?,
        Token::Percentage { unit_value, .. } => CssFiniteNumber::try_new(unit_value * 100.0)
            .map(CssRelativeColorExpressionValue::Percentage)
            .ok_or_else(|| invalid_color(location, Some("relative channel")))?,
        Token::Dimension { value, unit, .. }
            if matches!(result_domain, CssRelativeColorResultDomain::Hue) =>
        {
            parse_relative_angle(value, &unit)
                .map(CssRelativeColorExpressionValue::Angle)
                .ok_or_else(|| invalid_color(location, Some("relative hue")))?
        }
        Token::Function(name) if is_math_function(&name) => {
            let expression = parse_numeric_function(
                input,
                &before_opener,
                numeric,
                CalculationRoot::Relative(environment, result_domain),
            )?;
            let authored = CssAuthoredDeclarationValue::new(input.slice_from(start).trim_end());
            CssRelativeColorExpressionValue::Calculation(
                CssRelativeColorCalculation::from_expression(authored, expression),
            )
        }
        token => {
            return Err(with_color_context(
                location.new_unexpected_token_error::<Error>(token),
                Some("relative channel"),
            ));
        }
    };
    if !relative_direct_value_is_valid(result_domain, &value) {
        return Err(invalid_color(location, Some("relative channel")));
    }
    Ok(CssTypedRelativeColorExpression::new(
        environment,
        result_domain,
        value,
    ))
}

fn relative_direct_value_is_valid(
    domain: CssRelativeColorResultDomain,
    value: &CssRelativeColorExpressionValue,
) -> bool {
    match value {
        CssRelativeColorExpressionValue::None
        | CssRelativeColorExpressionValue::Channel(_)
        | CssRelativeColorExpressionValue::Calculation(_) => true,
        CssRelativeColorExpressionValue::Number(_) => true,
        CssRelativeColorExpressionValue::Percentage(_) => {
            !matches!(domain, CssRelativeColorResultDomain::Hue)
        }
        CssRelativeColorExpressionValue::Angle(_) => {
            matches!(domain, CssRelativeColorResultDomain::Hue)
        }
    }
}

pub(crate) fn numeric_relative_channel(
    environment: CssRelativeColorEnvironment,
    name: &str,
) -> Option<(CssRelativeColorChannel, CssCalculationType)> {
    let channel = relative_color_channel(environment, name)?;
    Some((channel, relative_channel_type(environment, channel)))
}

fn relative_color_channel(
    environment: CssRelativeColorEnvironment,
    ident: &str,
) -> Option<CssRelativeColorChannel> {
    use CssRelativeColorChannel::{A, Alpha, B, C, G, H, L, R, S, W, X, Y, Z};
    let channel = match environment {
        CssRelativeColorEnvironment::Rgb | CssRelativeColorEnvironment::PredefinedRgb(_) => {
            match_ignore_ascii_case! { ident,
                "r" => R,
                "g" => G,
                "b" => B,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Hsl => match_ignore_ascii_case! { ident,
            "h" => H,
            "s" => S,
            "l" => L,
            "alpha" => Alpha,
            _ => return None,
        },
        CssRelativeColorEnvironment::Hwb => match_ignore_ascii_case! { ident,
            "h" => H,
            "w" => W,
            "b" => B,
            "alpha" => Alpha,
            _ => return None,
        },
        CssRelativeColorEnvironment::Lab | CssRelativeColorEnvironment::Oklab => {
            match_ignore_ascii_case! { ident,
                "l" => L,
                "a" => A,
                "b" => B,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Lch | CssRelativeColorEnvironment::Oklch => {
            match_ignore_ascii_case! { ident,
                "l" => L,
                "c" => C,
                "h" => H,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Xyz(_) => match_ignore_ascii_case! { ident,
            "x" => X,
            "y" => Y,
            "z" => Z,
            "alpha" => Alpha,
            _ => return None,
        },
    };
    Some(channel)
}

fn relative_channel_type(
    environment: CssRelativeColorEnvironment,
    channel: CssRelativeColorChannel,
) -> CssCalculationType {
    use CssRelativeColorChannel::{A, Alpha, B, C, G, H, L, R, S, W, X, Y, Z};
    match (environment, channel) {
        (CssRelativeColorEnvironment::Hsl, H)
        | (CssRelativeColorEnvironment::Hwb, H)
        | (CssRelativeColorEnvironment::Lch, H)
        | (CssRelativeColorEnvironment::Oklch, H) => CssCalculationType::Angle,
        (CssRelativeColorEnvironment::Hsl, S | L)
        | (CssRelativeColorEnvironment::Hwb, W | B)
        | (CssRelativeColorEnvironment::Lab | CssRelativeColorEnvironment::Oklab, L)
        | (CssRelativeColorEnvironment::Lch | CssRelativeColorEnvironment::Oklch, L) => {
            CssCalculationType::Percentage
        }
        (_, R | G | B | A | C | X | Y | Z | Alpha | H | S | L | W) => CssCalculationType::Number,
    }
}

fn parse_relative_angle(value: f32, unit: &str) -> Option<CssAngleLiteral> {
    let unit = match unit.to_ascii_lowercase().as_str() {
        "deg" => CssAngleUnit::Degrees,
        "grad" => CssAngleUnit::Gradians,
        "rad" => CssAngleUnit::Radians,
        "turn" => CssAngleUnit::Turns,
        _ => return None,
    };
    CssAngleLiteral::try_new(value, unit)
}

fn parse_relative_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) => {
            let Some(function) = relative_color_function_from_name(&name) else {
                input.reset(&state);
                return Err(location.new_unexpected_token_error::<Error>(Token::Function(name)));
            };
            input
                .parse_nested_block(|input| parse_relative_color_arguments(input, function))
                .map(CssColor::Relative)
        }
        token => {
            input.reset(&state);
            Err(location.new_unexpected_token_error::<Error>(token))
        }
    }
}

#[derive(Clone, Copy)]
enum RelativeColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color,
}

fn relative_color_function_from_name(name: &str) -> Option<RelativeColorFunction> {
    let function = match_ignore_ascii_case! { name,
        "rgb" | "rgba" => RelativeColorFunction::Rgb,
        "hsl" | "hsla" => RelativeColorFunction::Hsl,
        "hwb" => RelativeColorFunction::Hwb,
        "lab" => RelativeColorFunction::Lab,
        "lch" => RelativeColorFunction::Lch,
        "oklab" => RelativeColorFunction::Oklab,
        "oklch" => RelativeColorFunction::Oklch,
        "color" => RelativeColorFunction::Color,
        _ => return None,
    };
    Some(function)
}

fn parse_relative_color_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    function: RelativeColorFunction,
) -> std::result::Result<CssRelativeColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    input.expect_ident_matching("from").map_err(basic)?;
    let source = parse_color_inner(input)?;
    let function = match function {
        RelativeColorFunction::Rgb => CssRelativeColorFunction::Rgb,
        RelativeColorFunction::Hsl => CssRelativeColorFunction::Hsl,
        RelativeColorFunction::Hwb => CssRelativeColorFunction::Hwb,
        RelativeColorFunction::Lab => CssRelativeColorFunction::Lab,
        RelativeColorFunction::Lch => CssRelativeColorFunction::Lch,
        RelativeColorFunction::Oklab => CssRelativeColorFunction::Oklab,
        RelativeColorFunction::Oklch => CssRelativeColorFunction::Oklch,
        RelativeColorFunction::Color => {
            CssRelativeColorFunction::Color(parse_relative_predefined_color_space(input)?)
        }
    };

    let mut components = Vec::with_capacity(function.component_count());
    for _ in 0..function.component_count() {
        components.push(parse_color_component_expression(input)?);
    }

    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_color_component_expression(input)?)
    } else {
        None
    };

    input.expect_exhausted().map_err(basic)?;
    CssRelativeColor::try_new(function, source, components, alpha).ok_or_else(|| {
        unsupported_value_at(location, None, "unsupported relative color component count")
    })
}

fn parse_relative_predefined_color_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPredefinedColorSpace, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color_space = match_ignore_ascii_case! { &ident,
        "srgb" => CssPredefinedColorSpace::Srgb,
        "srgb-linear" => CssPredefinedColorSpace::SrgbLinear,
        "display-p3" => CssPredefinedColorSpace::DisplayP3,
        "display-p3-linear" => CssPredefinedColorSpace::DisplayP3Linear,
        "a98-rgb" => CssPredefinedColorSpace::A98Rgb,
        "prophoto-rgb" => CssPredefinedColorSpace::ProphotoRgb,
        "rec2020" => CssPredefinedColorSpace::Rec2020,
        "xyz" => CssPredefinedColorSpace::XyzD65,
        "xyz-d50" => CssPredefinedColorSpace::XyzD50,
        "xyz-d65" => CssPredefinedColorSpace::XyzD65,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported relative color space `{ident}`"),
        )),
    };
    Ok(color_space)
}

fn parse_color_component_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorComponentExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.position();
    consume_color_component_expression(input)?;
    let authored = CssAuthoredDeclarationValue::new(input.slice_from(start).trim_end());
    Ok(CssColorComponentExpression::new(authored, Vec::new()))
}

fn consume_color_component_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    if token.is_parse_error() {
        return Err(input.new_unexpected_token_error(token));
    }
    match token {
        Token::Ident(_)
        | Token::Number { .. }
        | Token::Percentage { .. }
        | Token::Dimension { .. } => Ok(()),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(parse_color_component_calc_expression)
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported relative color component function `{name}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_color_component_calc_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    parse_color_component_calc_operand(input)?;
    while !input.is_exhausted() {
        parse_color_component_calc_operator(input)?;
        parse_color_component_calc_operand(input)?;
    }
    Ok(())
}

fn parse_color_component_calc_operator<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Delim('+') | Token::Delim('-') | Token::Delim('*') | Token::Delim('/') => Ok(()),
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "expected relative color calc operator, got `{}`",
                token.to_css_string()
            ),
        )),
    }
}

fn parse_color_component_calc_operand<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    if token.is_parse_error() {
        return Err(input.new_unexpected_token_error(token));
    }
    match token {
        Token::Ident(_)
        | Token::Number { .. }
        | Token::Percentage { .. }
        | Token::Dimension { .. } => Ok(()),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(parse_color_component_calc_expression)
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported relative color calc function `{name}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_color_mix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) if name.eq_ignore_ascii_case("color-mix") => input
            .parse_nested_block(parse_color_mix_arguments)
            .map(CssColor::ColorMix),
        token => {
            input.reset(&state);
            Err(location.new_unexpected_token_error::<Error>(token))
        }
    }
}

fn parse_authored_color_mix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) if name.eq_ignore_ascii_case("color-mix") => input
            .parse_nested_block(|input| parse_authored_color_mix_arguments(input, numeric))
            .map(CssAuthoredColor::color_mix),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_authored_color_mix_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredColorMix, ParseError<'i, Error>> {
    input.expect_ident_matching("in").map_err(basic)?;
    let interpolation = parse_authored_color_mix_interpolation_method(input)?;
    input.expect_comma().map_err(basic)?;
    let left = parse_authored_color_mix_component(input, numeric)?;
    input.expect_comma().map_err(basic)?;
    let right = parse_authored_color_mix_component(input, numeric)?;
    input.expect_exhausted().map_err(basic)?;

    CssAuthoredColorMix::try_new(interpolation, left, right).ok_or_else(|| {
        invalid_color(
            input.current_source_location(),
            Some("color-mix interpolation"),
        )
    })
}

fn parse_authored_color_mix_interpolation_method<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolationMethod, ParseError<'i, Error>> {
    let space = parse_color_mix_interpolation_space(input)?;
    let hue_location = input.current_source_location();
    let hue = input.try_parse(parse_color_mix_hue_interpolation).ok();
    let interpolation = CssColorInterpolationMethod::new(space, hue);
    if hue.is_some() && !space.is_polar() {
        Err(invalid_color(hue_location, Some("hue interpolation")))
    } else {
        Ok(interpolation)
    }
}

fn parse_authored_color_mix_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssAuthoredColorMixComponent, ParseError<'i, Error>> {
    let (color, _) = parse_color(input, numeric)?.into_parts();
    let percentage = if next_is_percentage(input) {
        Some(parse_authored_color_mix_percentage(input)?)
    } else {
        None
    };
    Ok(CssAuthoredColorMixComponent::new(color, percentage))
}

fn next_is_percentage<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_percentage = matches!(input.next(), Ok(Token::Percentage { .. }));
    input.reset(&state);
    is_percentage
}

fn parse_authored_color_mix_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColorMixPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let percentage = input.expect_percentage().map_err(basic)? * 100.0;
    CssAuthoredColorMixPercentage::try_new(percentage)
        .ok_or_else(|| invalid_color(location, Some("color-mix component percentage")))
}

fn parse_color_mix_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorMix, ParseError<'i, Error>> {
    input.expect_ident_matching("in").map_err(basic)?;
    let interpolation = parse_color_mix_interpolation_method(input)?;
    input.expect_comma().map_err(basic)?;
    let left = parse_color_mix_component(input)?;
    input.expect_comma().map_err(basic)?;
    let right = parse_color_mix_component(input)?;

    Ok(CssColorMix::new(interpolation, left, right))
}

fn parse_color_mix_interpolation_method<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolationMethod, ParseError<'i, Error>> {
    let space = parse_color_mix_interpolation_space(input)?;
    let hue = input.try_parse(parse_color_mix_hue_interpolation).ok();
    Ok(CssColorInterpolationMethod::new(space, hue))
}

fn parse_color_mix_interpolation_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolationSpace, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let space = match_ignore_ascii_case! { &ident,
        "srgb" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
        "srgb-linear" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::SrgbLinear),
        "display-p3" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::DisplayP3),
        "display-p3-linear" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::DisplayP3Linear),
        "a98-rgb" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::A98Rgb),
        "prophoto-rgb" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::ProphotoRgb),
        "rec2020" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Rec2020),
        "xyz" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD65),
        "xyz-d50" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD50),
        "xyz-d65" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD65),
        "hsl" => CssColorInterpolationSpace::Hsl,
        "hwb" => CssColorInterpolationSpace::Hwb,
        "lab" => CssColorInterpolationSpace::Lab,
        "lch" => CssColorInterpolationSpace::Lch,
        "oklab" => CssColorInterpolationSpace::Oklab,
        "oklch" => CssColorInterpolationSpace::Oklch,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported color-mix interpolation space `{ident}`"),
        )),
    };
    Ok(space)
}

fn parse_color_mix_hue_interpolation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssHueInterpolationMethod, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let hue = match_ignore_ascii_case! { &ident,
        "shorter" => CssHueInterpolationMethod::Shorter,
        "longer" => CssHueInterpolationMethod::Longer,
        "increasing" => CssHueInterpolationMethod::Increasing,
        "decreasing" => CssHueInterpolationMethod::Decreasing,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported color-mix hue interpolation method `{ident}`"),
        )),
    };
    input.expect_ident_matching("hue").map_err(basic)?;
    Ok(hue)
}

fn parse_color_mix_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorMixComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let color = parse_color_inner(input)?;
    let percentage = input.try_parse(parse_color_mix_percentage).ok();
    CssColorMixComponent::try_new(color, percentage).ok_or_else(|| {
        unsupported_value_at(location, None, "unsupported color-mix component percentage")
    })
}

fn parse_color_mix_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    input
        .expect_percentage()
        .map(|percentage| percentage * 100.0)
        .map_err(basic)
}

fn parse_absolute_color_with_cssparser_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match parse_color_with(&DefaultColorParser, input) {
        Ok(parsed) => map_parsed_color(parsed, location),
        Err(_) => Err(invalid_color(location, None)),
    }
}

fn parse_system_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color = match_ignore_ascii_case! { &ident,
        "canvas" => CssSystemColor::Canvas,
        "canvastext" => CssSystemColor::CanvasText,
        "linktext" => CssSystemColor::LinkText,
        "visitedtext" => CssSystemColor::VisitedText,
        "activetext" => CssSystemColor::ActiveText,
        "buttonface" => CssSystemColor::ButtonFace,
        "buttontext" => CssSystemColor::ButtonText,
        "buttonborder" => CssSystemColor::ButtonBorder,
        "field" => CssSystemColor::Field,
        "fieldtext" => CssSystemColor::FieldText,
        "highlight" => CssSystemColor::Highlight,
        "highlighttext" => CssSystemColor::HighlightText,
        "mark" => CssSystemColor::Mark,
        "marktext" => CssSystemColor::MarkText,
        "graytext" => CssSystemColor::GrayText,
        "selecteditem" => CssSystemColor::SelectedItem,
        "selecteditemtext" => CssSystemColor::SelectedItemText,
        "accentcolor" => CssSystemColor::AccentColor,
        "accentcolortext" => CssSystemColor::AccentColorText,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported system color `{ident}`"),
        )),
    };
    Ok(CssColor::System(color))
}

fn map_parsed_color<'i>(
    parsed: ParsedColor,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let color = match parsed {
        ParsedColor::CurrentColor => CssColor::CurrentColor,
        ParsedColor::Rgba(color) => CssColor::Rgba(
            CssRgbaColor::try_new(color.red, color.green, color.blue, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Hsl(color) => CssColor::Hsl(
            CssHslColor::try_new(color.hue, color.saturation, color.lightness, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Hwb(color) => CssColor::Hwb(
            CssHwbColor::try_new(color.hue, color.whiteness, color.blackness, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Lab(color) => CssColor::Lab(
            CssLabColor::try_new(color.lightness, color.a, color.b, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Lch(color) => CssColor::Lch(
            CssLchColor::try_new(color.lightness, color.chroma, color.hue, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Oklab(color) => CssColor::Oklab(
            CssLabColor::try_new(color.lightness, color.a, color.b, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Oklch(color) => CssColor::Oklch(
            CssLchColor::try_new(color.lightness, color.chroma, color.hue, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::ColorFunction(color) => CssColor::ColorFunction(
            CssColorFunction::try_new(
                map_predefined_color_space(color.color_space),
                [color.c1, color.c2, color.c3],
                color.alpha,
            )
            .ok_or_else(|| invalid_color_component(location))?,
        ),
    };
    Ok(color)
}

fn invalid_color_component<'i>(location: cssparser::SourceLocation) -> ParseError<'i, Error> {
    invalid_color(location, Some("component"))
}

fn map_predefined_color_space(color_space: ParsedPredefinedColorSpace) -> CssPredefinedColorSpace {
    match color_space {
        ParsedPredefinedColorSpace::Srgb => CssPredefinedColorSpace::Srgb,
        ParsedPredefinedColorSpace::SrgbLinear => CssPredefinedColorSpace::SrgbLinear,
        ParsedPredefinedColorSpace::DisplayP3 => CssPredefinedColorSpace::DisplayP3,
        ParsedPredefinedColorSpace::DisplayP3Linear => CssPredefinedColorSpace::DisplayP3Linear,
        ParsedPredefinedColorSpace::A98Rgb => CssPredefinedColorSpace::A98Rgb,
        ParsedPredefinedColorSpace::ProphotoRgb => CssPredefinedColorSpace::ProphotoRgb,
        ParsedPredefinedColorSpace::Rec2020 => CssPredefinedColorSpace::Rec2020,
        ParsedPredefinedColorSpace::XyzD50 => CssPredefinedColorSpace::XyzD50,
        ParsedPredefinedColorSpace::XyzD65 => CssPredefinedColorSpace::XyzD65,
    }
}

#[cfg(test)]
mod typed_calculation_tests {
    use cssparser::{Parser, ParserInput};

    use super::*;
    use crate::error::{CssErrorCode, from_parse_error};

    fn parse(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::Error> {
        parse_complete(&format!("calc({source})"), root)
    }

    fn parse_complete(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::Error> {
        let snapshot = crate::CssSourceSnapshot::new(source);
        let numeric = NumericInputContext::parsed(&snapshot);
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        let before = parser.state();
        parser
            .parse_entirely(|input| parse_numeric_function(input, &before, &numeric, root))
            .map_err(|error| from_parse_error(source, error))
    }

    fn body(expression: &CssCalculationExpression) -> CssCalculationExpressionRef<'_> {
        let CssCalculationExpressionRef::NestedCalc(root) = expression.as_ref() else {
            panic!("expected calc root")
        };
        root.operand()
    }

    #[test]
    fn typed_root_parser_preserves_all_compound_node_kinds_and_precedence() {
        let number = parse("1 + 6 / 2", CalculationRoot::Number).unwrap();
        assert_eq!(number.result_type(), CssCalculationType::Number);
        let CssCalculationExpressionRef::Sum(sum) = body(&number) else {
            panic!("expected number sum");
        };
        assert_eq!(sum.len(), 2);
        assert_eq!(sum.term(0).unwrap().operator(), None);
        assert_eq!(
            sum.term(1).unwrap().operator(),
            Some(CssCalculationSumOperator::Add)
        );
        let CssCalculationExpressionRef::Product(quotient) = sum.term(1).unwrap().expression()
        else {
            panic!("division must bind inside the sum");
        };
        assert_eq!(quotient.len(), 2);
        assert_eq!(quotient.factor(0).unwrap().operator(), None);
        assert_eq!(
            quotient.factor(1).unwrap().operator(),
            Some(CssCalculationProductOperator::Divide)
        );

        let integer = parse("1 + 2 * 3", CalculationRoot::Integer).unwrap();
        assert_eq!(integer.result_type(), CssCalculationType::Number);
        let CssCalculationExpressionRef::Sum(integer_sum) = body(&integer) else {
            panic!("expected integer sum");
        };
        let CssCalculationExpressionRef::Product(integer_product) =
            integer_sum.term(1).unwrap().expression()
        else {
            panic!("expected integer product");
        };
        assert_eq!(
            integer_product.factor(1).unwrap().operator(),
            Some(CssCalculationProductOperator::Multiply)
        );
        assert!(integer_product.factor(integer_product.len()).is_none());

        let percentage = parse("10% - 20%", CalculationRoot::Percentage).unwrap();
        assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
        let CssCalculationExpressionRef::Sum(percentage_sum) = body(&percentage) else {
            panic!("expected percentage sum");
        };
        assert_eq!(
            percentage_sum.term(1).unwrap().operator(),
            Some(CssCalculationSumOperator::Subtract)
        );
        assert!(percentage_sum.term(percentage_sum.len()).is_none());

        let length = parse("1px + (2em * 3)", CalculationRoot::Length).unwrap();
        let CssCalculationExpressionRef::Sum(sum) = body(&length) else {
            panic!("expected length sum");
        };
        let CssCalculationExpressionRef::Group(group) = sum.term(1).unwrap().expression() else {
            panic!("expected retained authored group");
        };
        assert!(matches!(
            group.operand(),
            CssCalculationExpressionRef::Product(_)
        ));

        let angle = parse("1deg + calc(2turn)", CalculationRoot::Angle).unwrap();
        assert_eq!(angle.result_type(), CssCalculationType::Angle);
        let CssCalculationExpressionRef::Sum(sum) = body(&angle) else {
            panic!("expected angle sum");
        };
        let CssCalculationExpressionRef::NestedCalc(nested) = sum.term(1).unwrap().expression()
        else {
            panic!("expected retained nested calc");
        };
        assert!(matches!(
            nested.operand(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(value))
                if value.representation() == "2" && value.unit() == Some("turn")
        ));

        let time = parse("1s + (-2ms)", CalculationRoot::Time).unwrap();
        assert_eq!(time.result_type(), CssCalculationType::Time);
        let CssCalculationExpressionRef::Sum(sum) = body(&time) else {
            panic!("expected time sum")
        };
        let CssCalculationExpressionRef::Group(group) = sum.term(1).unwrap().expression() else {
            panic!("expected group")
        };
        assert!(
            matches!(group.operand(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Time(v)) if v.representation() == "-2" && v.unit() == Some("ms"))
        );
        assert!(parse("1s + -(2ms)", CalculationRoot::Time).is_err());

        let frequency = parse("1khz / 2", CalculationRoot::Frequency).unwrap();
        assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
        assert!(matches!(
            body(&frequency),
            CssCalculationExpressionRef::Product(_)
        ));
    }

    #[test]
    fn typed_root_parser_promotes_only_the_selected_percentage_context() {
        let mixed = parse("1px + 2%", CalculationRoot::LengthPercentage).unwrap();
        assert_eq!(mixed.result_type(), CssCalculationType::LengthPercentage);
        for (root, unit) in [
            (CalculationRoot::Length, "px"),
            (CalculationRoot::Angle, "deg"),
            (CalculationRoot::Time, "s"),
            (CalculationRoot::Frequency, "hz"),
        ] {
            assert!(parse(&format!("1{unit} + 2%"), root).is_err());
        }
    }

    #[test]
    fn typed_root_parser_rejects_invalid_types_and_grammar_but_defers_arithmetic() {
        for (source, root) in [
            ("1px + 2deg", CalculationRoot::Length),
            ("1px * 2em", CalculationRoot::Length),
            ("1px / 2em", CalculationRoot::Length),
            ("1px +", CalculationRoot::Length),
            ("1px red", CalculationRoot::Length),
            ("1px", CalculationRoot::Number),
        ] {
            assert!(parse(source, root).is_err(), "{source}");
        }
        for source in ["1px / 0", "1px / -0", "1px / calc(1 - 1)", "1e999px"] {
            assert_eq!(
                parse(source, CalculationRoot::Length)
                    .unwrap()
                    .result_type(),
                CssCalculationType::Length
            );
        }
        for source in ["3.4e38 * 2", "1e999", "2147483648"] {
            assert_eq!(
                parse(source, CalculationRoot::Integer)
                    .unwrap()
                    .result_type(),
                CssCalculationType::Number
            );
        }
        for (source, root) in [
            ("1e999%", CalculationRoot::Percentage),
            ("1e999deg", CalculationRoot::Angle),
            ("1e999s", CalculationRoot::Time),
            ("1e999hz", CalculationRoot::Frequency),
        ] {
            assert!(parse(source, root).is_ok(), "{source}");
        }
    }

    #[test]
    fn typed_root_parser_accepts_256_nested_calculations_and_rejects_257() {
        for depth in [255_usize, 256] {
            let source = format!("{}1px{}", "calc(".repeat(depth), ")".repeat(depth));
            let result_type = std::thread::scope(|scope| {
                std::thread::Builder::new()
                    .stack_size(16 * 1024 * 1024)
                    .spawn_scoped(scope, || {
                        parse_complete(&source, CalculationRoot::Length)
                            .map(|expression| expression.result_type())
                    })
                    .unwrap()
                    .join()
                    .unwrap()
            })
            .unwrap();
            assert_eq!(result_type, CssCalculationType::Length);
        }

        let depth = 257_usize;
        let source = format!("{}1px{}", "calc(".repeat(depth), ")".repeat(depth));
        let error = std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || parse_complete(&source, CalculationRoot::Length))
                .unwrap()
                .join()
                .unwrap()
        })
        .expect_err("depth 257 must fail");
        assert_eq!(error.code(), CssErrorCode::UnexpectedEnd);
    }
}
