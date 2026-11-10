use super::combinators::{Combinator, Tail, keyword, parse_group};
use super::syntax_types::Stream;
use super::values::parse_nonnegative_length;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::property_input::StreamExt;
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::{
    LiteralTokens, NumericType, parse_integer_literal, parse_integer_value, parse_numeric_function,
    parse_numeric_type,
};
use crate::numeric::expression::CalculationRoot;
use crate::numeric::{
    CssCalculation, CssCalculationType, CssHintedNumber, CssInteger, CssNumber, CssNumeric,
    CssPercentage, CssPositive,
};
use crate::values::box_model::CssOverflowValue;
use crate::values::box_model::display::*;
use crate::values::box_model::{
    CssAspectRatioValue, CssBoxSizing, CssBreakBetween, CssBreakInside, CssClear, CssContain,
    CssContainComponent, CssContainComponentList, CssContentVisibility, CssFloat,
    CssLayoutPosition, CssOverflow, CssVisibility, CssZIndexValue,
};
use crate::values::common::CssSidePair;
use crate::values::effects::CssOpacityValue;
use crate::values::table::border_spacing::BorderSpacingLength;
use crate::values::table::{CssBorderCollapse, CssCaptionSide, CssEmptyCells, CssTableLayout};
use crate::values::text::CssDirection;
use crate::values::ui::{CssResize, CssScrollbarWidth};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::diagnostic::CssFeatureId] =
    &[crate::diagnostic::CssFeatureId::new(
        "ext.value.grid-lanes-display",
    )];

pub(crate) fn parse_resize<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssResize, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssResize::None),
        "both" => Ok(CssResize::Both),
        "horizontal" => Ok(CssResize::Horizontal),
        "vertical" => Ok(CssResize::Vertical),
        "block" => Ok(CssResize::Block),
        "inline" => Ok(CssResize::Inline),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_contain<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssContain, SyntaxError<'i, Error>> {
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let standalone = match_ignore_ascii_case! { &ident,
            "none" => Some(CssContain::None),
            "strict" => Some(CssContain::Strict),
            "content" => Some(CssContain::Content),
            _ => None,
        };
        if let Some(value) = standalone {
            return Ok(value);
        }
    }
    input.restore(state);

    // `size || layout || style || paint`, retaining authored order.
    const KEYWORDS: [(&str, CssContainComponent); 4] = [
        ("size", CssContainComponent::Size),
        ("layout", CssContainComponent::Layout),
        ("style", CssContainComponent::Style),
        ("paint", CssContainComponent::Paint),
    ];
    let mut components = Vec::new();
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| {
            let mut component = None;
            let matched = keyword(&mut component, input, &KEYWORDS[operand..=operand])?;
            components.extend(component);
            Ok(matched)
        },
    )?;
    CssContainComponentList::try_new(components)
        .map(CssContain::Components)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_border_collapse<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBorderCollapse, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "collapse" => Ok(CssBorderCollapse::Collapse),
        "separate" => Ok(CssBorderCollapse::Separate),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_border_spacing<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssSidePair<BorderSpacingLength>, SyntaxError<'i, Error>> {
    let horizontal = parse_nonnegative_length(input, numeric)?;
    let vertical = if input.only_whitespace_remains() {
        None
    } else {
        Some(parse_nonnegative_length(input, numeric)?)
    };
    Ok(CssSidePair::new(horizontal, vertical))
}

pub(crate) fn parse_caption_side<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssCaptionSide, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "top" => Ok(CssCaptionSide::Top),
        "bottom" => Ok(CssCaptionSide::Bottom),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_empty_cells<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssEmptyCells, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "show" => Ok(CssEmptyCells::Show),
        "hide" => Ok(CssEmptyCells::Hide),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_page_line_minimum<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssNumeric<CssInteger, CssPositive>, SyntaxError<'i, Error>> {
    /// Fragmentation 3 `<integer [1,∞]>`.
    const PAGE_LINE_MINIMUM: NumericType<CssNumeric<CssInteger, CssPositive>> = NumericType {
        root: CalculationRoot::Integer,
        tokens: LiteralTokens::Number,
        literal: |component, _| {
            CssNumeric::<CssInteger, CssPositive>::try_from_component(component)
        },
        calculation: |expression| {
            CssNumeric::<CssInteger, CssPositive>::from_parser_calculation(CssCalculation::<
                CssInteger,
            >::from_expression(
                expression
            ))
        },
    };
    parse_numeric_type(input, numeric, &PAGE_LINE_MINIMUM)
}

pub(crate) fn parse_break_between<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBreakBetween, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakBetween::Auto),
        "avoid" => Ok(CssBreakBetween::Avoid),
        "avoid-page" => Ok(CssBreakBetween::AvoidPage),
        "page" => Ok(CssBreakBetween::Page),
        "left" => Ok(CssBreakBetween::Left),
        "right" => Ok(CssBreakBetween::Right),
        "recto" => Ok(CssBreakBetween::Recto),
        "verso" => Ok(CssBreakBetween::Verso),
        "avoid-column" => Ok(CssBreakBetween::AvoidColumn),
        "column" => Ok(CssBreakBetween::Column),
        "avoid-region" => Ok(CssBreakBetween::AvoidRegion),
        "region" => Ok(CssBreakBetween::Region),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_break_inside<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBreakInside, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakInside::Auto),
        "avoid" => Ok(CssBreakInside::Avoid),
        "avoid-page" => Ok(CssBreakInside::AvoidPage),
        "avoid-column" => Ok(CssBreakInside::AvoidColumn),
        "avoid-region" => Ok(CssBreakInside::AvoidRegion),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_page_break_between<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBreakBetween, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakBetween::Auto),
        "always" => Ok(CssBreakBetween::Page),
        "avoid" => Ok(CssBreakBetween::Avoid),
        "left" => Ok(CssBreakBetween::Left),
        "right" => Ok(CssBreakBetween::Right),
        "recto" => Ok(CssBreakBetween::Recto),
        "verso" => Ok(CssBreakBetween::Verso),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_page_break_inside<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBreakInside, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakInside::Auto),
        "avoid" => Ok(CssBreakInside::Avoid),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_table_layout<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTableLayout, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssTableLayout::Auto),
        "fixed" => Ok(CssTableLayout::Fixed),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_display<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssDisplayValue, SyntaxError<'i, Error>> {
    enum Component {
        Outside(CssDisplayOutside),
        Inside(CssDisplayInside),
        ListItem,
        Exclusive(CssDisplayValue),
    }
    let mut outside = None;
    let mut inside = None;
    let mut list_item = false;
    let mut count = 0;
    while !input.only_whitespace_remains() {
        if count == 3 {
            return Err(unexpected_at(input.site()));
        }
        count += 1;
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        let component = match_ignore_ascii_case! { &ident,
            "block" => Component::Outside(CssDisplayOutside::Block),
            "inline" => Component::Outside(CssDisplayOutside::Inline),
            "run-in" => Component::Outside(CssDisplayOutside::RunIn),
            "flow" => Component::Inside(CssDisplayInside::Flow),
            "flow-root" => Component::Inside(CssDisplayInside::FlowRoot),
            "table" => Component::Inside(CssDisplayInside::Table),
            "flex" => Component::Inside(CssDisplayInside::Flex),
            "grid" => Component::Inside(CssDisplayInside::Grid),
            "ruby" => Component::Inside(CssDisplayInside::Ruby),
            "list-item" => Component::ListItem,
            "table-row-group" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableRowGroup)),
            "table-header-group" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableHeaderGroup)),
            "table-footer-group" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableFooterGroup)),
            "table-row" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableRow)),
            "table-cell" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableCell)),
            "table-column-group" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableColumnGroup)),
            "table-column" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableColumn)),
            "table-caption" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::TableCaption)),
            "ruby-base" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::RubyBase)),
            "ruby-text" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::RubyText)),
            "ruby-base-container" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::RubyBaseContainer)),
            "ruby-text-container" => Component::Exclusive(CssDisplayValue::Internal(CssDisplayInternal::RubyTextContainer)),
            "contents" => Component::Exclusive(CssDisplayValue::Box(CssDisplayBox::Contents)),
            "none" => Component::Exclusive(CssDisplayValue::Box(CssDisplayBox::None)),
            "inline-block" => Component::Exclusive(CssDisplayValue::Legacy(CssDisplayLegacy::InlineBlock)),
            "inline-table" => Component::Exclusive(CssDisplayValue::Legacy(CssDisplayLegacy::InlineTable)),
            "inline-flex" => Component::Exclusive(CssDisplayValue::Legacy(CssDisplayLegacy::InlineFlex)),
            "inline-grid" => Component::Exclusive(CssDisplayValue::Legacy(CssDisplayLegacy::InlineGrid)),
            "grid-lanes" => Component::Exclusive(CssDisplayValue::GridLanes),
            "inline-grid-lanes" => Component::Exclusive(CssDisplayValue::InlineGridLanes),
            _ => return Err(unexpected_at(location)),
        };
        let duplicate = match component {
            Component::Outside(value) => outside.replace(value).is_some(),
            Component::Inside(value) => inside.replace(value).is_some(),
            Component::ListItem => std::mem::replace(&mut list_item, true),
            Component::Exclusive(value) => {
                if count == 1 && input.only_whitespace_remains() {
                    return Ok(value);
                }
                return Err(unexpected_at(location));
            }
        };
        if duplicate {
            return Err(unexpected_at(location));
        }
    }
    if count == 0 {
        return Err(unexpected_at(input.site()));
    }
    if list_item {
        let inside = match inside {
            None | Some(CssDisplayInside::Flow) => CssDisplayListItemInside::Flow,
            Some(CssDisplayInside::FlowRoot) => CssDisplayListItemInside::FlowRoot,
            Some(_) => {
                return Err(unexpected_at(input.site()));
            }
        };
        return Ok(CssDisplayValue::ListItem {
            outside: outside.unwrap_or(CssDisplayOutside::Block),
            inside,
        });
    }
    let inside = inside.unwrap_or(CssDisplayInside::Flow);
    Ok(CssDisplayValue::OutsideInside {
        outside: outside.unwrap_or(if inside == CssDisplayInside::Ruby {
            CssDisplayOutside::Inline
        } else {
            CssDisplayOutside::Block
        }),
        inside,
    })
}

pub(crate) fn parse_box_sizing<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBoxSizing, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "content-box" => Ok(CssBoxSizing::ContentBox),
        "border-box" => Ok(CssBoxSizing::BorderBox),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_position<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssLayoutPosition, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "static" => Ok(CssLayoutPosition::Static),
        "relative" => Ok(CssLayoutPosition::Relative),
        "absolute" => Ok(CssLayoutPosition::Absolute),
        "fixed" => Ok(CssLayoutPosition::Fixed),
        "sticky" => Ok(CssLayoutPosition::Sticky),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_direction<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssDirection, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "ltr" => Ok(CssDirection::Ltr),
        "rtl" => Ok(CssDirection::Rtl),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_overflow<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssOverflow, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssOverflow::Visible),
        "clip" => Ok(CssOverflow::Clip),
        "hidden" => Ok(CssOverflow::Hidden),
        "scroll" => Ok(CssOverflow::Scroll),
        "auto" | "overlay" => Ok(CssOverflow::Auto),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_overflow_value<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssOverflowValue, SyntaxError<'i, Error>> {
    let x = parse_overflow(input)?;
    let y = if input.only_whitespace_remains() {
        None
    } else {
        Some(parse_overflow(input)?)
    };
    Ok(CssOverflowValue::new(x, y))
}

pub(crate) fn parse_float<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFloat, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "left" => Ok(CssFloat::Left),
        "right" => Ok(CssFloat::Right),
        "none" => Ok(CssFloat::None),
        "inline-start" => Ok(CssFloat::InlineStart),
        "inline-end" => Ok(CssFloat::InlineEnd),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_clear<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssClear, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "left" => Ok(CssClear::Left),
        "right" => Ok(CssClear::Right),
        "both" => Ok(CssClear::Both),
        "none" => Ok(CssClear::None),
        "inline-start" => Ok(CssClear::InlineStart),
        "inline-end" => Ok(CssClear::InlineEnd),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_visibility<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssVisibility, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssVisibility::Visible),
        "hidden" => Ok(CssVisibility::Hidden),
        "collapse" => Ok(CssVisibility::Collapse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_content_visibility<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssContentVisibility, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssContentVisibility::Visible),
        "hidden" => Ok(CssContentVisibility::Hidden),
        "auto" => Ok(CssContentVisibility::Auto),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_opacity<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssOpacityValue, SyntaxError<'i, Error>> {
    let numeric_start = input.mark();
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Number(_) | Token::Percentage(_) => {
            input.restore(numeric_start);
            input.discard_whitespace();
            let offset = input.site();
            let component = input.collect_numeric_component(numeric).map_err(|error| {
                unexpected_at(numeric.property_error_location(&error, location, offset))
            })?;
            crate::values::effects::opacity_scalar::admit_opacity_scalar(component)
                .map_err(|_| unexpected_at(location))
        }
        Token::Function(name) if crate::numeric::expression::is_math_function(name) => {
            let calculation = parse_numeric_function(
                input,
                &numeric_start,
                numeric,
                CalculationRoot::NumberPercentage,
            )?;
            Ok(match calculation.result_type() {
                CssCalculationType::Number
                    if calculation.as_ref().numeric_type().percent_hint().is_some() =>
                {
                    CssOpacityValue::HintedNumberCalculation(
                        CssCalculation::<CssHintedNumber>::from_expression(calculation),
                    )
                }
                CssCalculationType::Number => {
                    CssOpacityValue::NumberCalculation(
                        CssCalculation::<CssNumber>::from_expression(calculation),
                    )
                }
                CssCalculationType::Percentage => CssOpacityValue::PercentageCalculation(
                    CssCalculation::<CssPercentage>::from_expression(calculation),
                ),
                _ => unreachable!("checked opacity Number or Percentage"),
            })
        }
        token => Err(location.unexpected::<Error>(token)),
    }
}

pub(crate) fn parse_aspect_ratio<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAspectRatioValue, SyntaxError<'i, Error>> {
    let auto_first = input.attempt(|input| input.consume_keyword("auto")).is_ok();
    if auto_first && input.only_whitespace_remains() {
        return Ok(CssAspectRatioValue::Auto);
    }
    let numerator = parse_ratio_operand(input, numeric)?;
    let denominator = if input.attempt(|input| input.consume_delim('/')).is_ok() {
        Some(parse_ratio_operand(input, numeric)?)
    } else {
        None
    };
    let ratio = crate::numeric::CssSpecifiedRatio::new(numerator, denominator);
    if auto_first || input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        Ok(CssAspectRatioValue::AutoRatio(ratio))
    } else {
        Ok(CssAspectRatioValue::Ratio(ratio))
    }
}

fn parse_ratio_operand<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<
    crate::numeric::CssNumeric<crate::numeric::CssNumber, crate::numeric::CssNonNegative>,
    SyntaxError<'i, Error>,
> {
    /// Values 4 `<ratio>` operand: `<number [0,∞]>`.
    const RATIO_OPERAND: NumericType<
        crate::numeric::CssNumeric<crate::numeric::CssNumber, crate::numeric::CssNonNegative>,
    > = NumericType {
        root: CalculationRoot::Number,
        tokens: LiteralTokens::Number,
        literal: |component, _| {
            crate::numeric::CssNumeric::<crate::numeric::CssNumber, crate::numeric::CssNonNegative>::try_from_component(
                    component,
                )
        },
        calculation: |expression| {
            crate::numeric::CssNumeric::<crate::numeric::CssNumber, crate::numeric::CssNonNegative>::from_parser_calculation(CssCalculation::<CssNumber>::from_expression(
                expression,
            ))
        },
    };
    parse_numeric_type(input, numeric, &RATIO_OPERAND)
}

pub(crate) fn parse_scrollbar_width<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssScrollbarWidth, SyntaxError<'i, Error>> {
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "auto" => Ok(CssScrollbarWidth::Auto),
            "thin" => Ok(CssScrollbarWidth::Thin),
            "none" => Ok(CssScrollbarWidth::None),
            _ => Err(unexpected_at(location)),
        },
        _ => Err(unexpected_at(location)),
    }
}

pub(crate) fn parse_order<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssNumeric<CssInteger>, SyntaxError<'i, Error>> {
    parse_integer_value(input, numeric)
}

pub(crate) fn parse_z_index<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssZIndexValue, SyntaxError<'i, Error>> {
    let numeric_start = input.mark();
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssZIndexValue::Auto),
        Token::Ident(_) => Err(unexpected_at(location)),
        Token::Number(_) => {
            input.restore(numeric_start);
            parse_integer_literal(input, numeric)
                .map(CssNumeric::from_literal)
                .map(CssZIndexValue::Integer)
        }
        Token::Dimension { .. } => Err(unexpected_at(location)),
        Token::Function(name) if crate::numeric::expression::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)?;
            CssNumeric::from_parser_calculation(CssCalculation::from_expression(expression))
                .map(CssZIndexValue::Integer)
                .map_err(|_| unexpected_at(location))
        }
        token => Err(location.unexpected::<Error>(token)),
    }
}
