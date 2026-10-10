use super::values::parse_nonnegative_length;
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, parse_integer_literal, parse_integer_value, parse_numeric_function,
};
use crate::CssOverflowValue;
use crate::display::*;
use crate::error::{Error, basic, unexpected_at};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::CssFeatureId] =
    &[crate::CssFeatureId::new("ext.value.grid-lanes-display")];

pub(super) fn parse_resize<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssResize, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssResize::None),
        "both" => Ok(CssResize::Both),
        "horizontal" => Ok(CssResize::Horizontal),
        "vertical" => Ok(CssResize::Vertical),
        "block" => Ok(CssResize::Block),
        "inline" => Ok(CssResize::Inline),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_contain<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContain, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
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
    input.reset(&state);

    let mut components = Vec::new();
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        let component = match_ignore_ascii_case! { &ident,
            "size" => CssContainComponent::Size,
            "layout" => CssContainComponent::Layout,
            "style" => CssContainComponent::Style,
            "paint" => CssContainComponent::Paint,
            _ => return Err(unexpected_at(location)),
        };
        if components.contains(&component) {
            return Err(unexpected_at(location));
        }
        components.push(component);
    }
    CssContainComponentList::try_new(components)
        .map(CssContain::Components)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_border_collapse<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderCollapse, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "collapse" => Ok(CssBorderCollapse::Collapse),
        "separate" => Ok(CssBorderCollapse::Separate),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_border_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderSpacing, ParseError<'i, Error>> {
    let horizontal = parse_nonnegative_length(input, numeric)?;
    let vertical = if input.is_exhausted() {
        horizontal.clone()
    } else {
        parse_nonnegative_length(input, numeric)?
    };
    Ok(CssBorderSpacing::new(horizontal, vertical))
}

pub(super) fn parse_caption_side<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCaptionSide, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "top" => Ok(CssCaptionSide::Top),
        "bottom" => Ok(CssCaptionSide::Bottom),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_empty_cells<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssEmptyCells, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "show" => Ok(CssEmptyCells::Show),
        "hide" => Ok(CssEmptyCells::Hide),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_page_line_minimum<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPageLineMinimum, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&numeric_start);
            let component = numeric
                .collect(input)
                .map_err(|_| unexpected_at(location))?;
            CssPageLineMinimum::try_from_component(component).map_err(|_| unexpected_at(location))
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .and_then(|value| {
                    CssPageLineMinimum::try_from_calculation(value)
                        .map_err(|_| unexpected_at(location))
                })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_break_between<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBreakBetween, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
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
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_break_inside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBreakInside, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakInside::Auto),
        "avoid" => Ok(CssBreakInside::Avoid),
        "avoid-page" => Ok(CssBreakInside::AvoidPage),
        "avoid-column" => Ok(CssBreakInside::AvoidColumn),
        "avoid-region" => Ok(CssBreakInside::AvoidRegion),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_page_break_between<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBreakBetween, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakBetween::Auto),
        "always" => Ok(CssBreakBetween::Page),
        "avoid" => Ok(CssBreakBetween::Avoid),
        "left" => Ok(CssBreakBetween::Left),
        "right" => Ok(CssBreakBetween::Right),
        "recto" => Ok(CssBreakBetween::Recto),
        "verso" => Ok(CssBreakBetween::Verso),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_page_break_inside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBreakInside, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakInside::Auto),
        "avoid" => Ok(CssBreakInside::Avoid),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_table_layout<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTableLayout, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssTableLayout::Auto),
        "fixed" => Ok(CssTableLayout::Fixed),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_display<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssDisplayValue, ParseError<'i, Error>> {
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
    while !input.is_exhausted() {
        if count == 3 {
            return Err(unexpected_at(input.current_source_location()));
        }
        count += 1;
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
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
                if count == 1 && input.is_exhausted() {
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
        return Err(unexpected_at(input.current_source_location()));
    }
    if list_item {
        let inside = match inside {
            None | Some(CssDisplayInside::Flow) => CssDisplayListItemInside::Flow,
            Some(CssDisplayInside::FlowRoot) => CssDisplayListItemInside::FlowRoot,
            Some(_) => {
                return Err(unexpected_at(input.current_source_location()));
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

pub(super) fn parse_box_sizing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBoxSizing, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "content-box" => Ok(CssBoxSizing::ContentBox),
        "border-box" => Ok(CssBoxSizing::BorderBox),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLayoutPosition, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "static" => Ok(CssLayoutPosition::Static),
        "relative" => Ok(CssLayoutPosition::Relative),
        "absolute" => Ok(CssLayoutPosition::Absolute),
        "fixed" => Ok(CssLayoutPosition::Fixed),
        "sticky" => Ok(CssLayoutPosition::Sticky),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssDirection, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "ltr" => Ok(CssDirection::Ltr),
        "rtl" => Ok(CssDirection::Rtl),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_overflow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOverflow, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssOverflow::Visible),
        "clip" => Ok(CssOverflow::Clip),
        "hidden" => Ok(CssOverflow::Hidden),
        "scroll" => Ok(CssOverflow::Scroll),
        "auto" | "overlay" => Ok(CssOverflow::Auto),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_overflow_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOverflowValue, ParseError<'i, Error>> {
    let x = parse_overflow(input)?;
    let y = if input.is_exhausted() {
        None
    } else {
        Some(parse_overflow(input)?)
    };
    Ok(CssOverflowValue::new(x, y))
}

pub(super) fn parse_float<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFloat, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "left" => Ok(CssFloat::Left),
        "right" => Ok(CssFloat::Right),
        "none" => Ok(CssFloat::None),
        "inline-start" => Ok(CssFloat::InlineStart),
        "inline-end" => Ok(CssFloat::InlineEnd),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_clear<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssClear, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "left" => Ok(CssClear::Left),
        "right" => Ok(CssClear::Right),
        "both" => Ok(CssClear::Both),
        "none" => Ok(CssClear::None),
        "inline-start" => Ok(CssClear::InlineStart),
        "inline-end" => Ok(CssClear::InlineEnd),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_visibility<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssVisibility, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssVisibility::Visible),
        "hidden" => Ok(CssVisibility::Hidden),
        "collapse" => Ok(CssVisibility::Collapse),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_content_visibility<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContentVisibility, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssContentVisibility::Visible),
        "hidden" => Ok(CssContentVisibility::Hidden),
        "auto" => Ok(CssContentVisibility::Auto),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_opacity<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOpacityValue, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Percentage { .. } => {
            input.reset(&numeric_start);
            input.skip_whitespace();
            let offset = input.position().byte_index();
            let component = numeric
                .collect(input)
                .map_err(|error| unexpected_at(numeric.error_location(&error, location, offset)))?;
            crate::opacity_scalar::admit_opacity_scalar(component)
                .map_err(|_| unexpected_at(location))
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
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
                        CssHintedNumberCalculation::from_expression(calculation),
                    )
                }
                CssCalculationType::Number => CssOpacityValue::NumberCalculation(
                    CssNumberCalculation::from_expression(calculation),
                ),
                CssCalculationType::Percentage => CssOpacityValue::PercentageCalculation(
                    CssPercentageCalculation::from_expression(calculation),
                ),
                _ => unreachable!("checked opacity Number or Percentage"),
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_aspect_ratio<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssAspectRatioValue, ParseError<'i, Error>> {
    let auto_first = input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok();
    if auto_first && input.is_exhausted() {
        return Ok(CssAspectRatioValue::Auto);
    }
    let numerator = parse_ratio_operand(input, numeric)?;
    let denominator = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_ratio_operand(input, numeric)?)
    } else {
        None
    };
    let ratio = crate::CssSpecifiedRatio::new(numerator, denominator);
    if auto_first
        || input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
    {
        Ok(CssAspectRatioValue::AutoRatio(ratio))
    } else {
        Ok(CssAspectRatioValue::Ratio(ratio))
    }
}

fn parse_ratio_operand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<crate::CssRatioOperand, ParseError<'i, Error>> {
    let start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&start);
            let component = numeric
                .collect(input)
                .map_err(|_| unexpected_at(location))?;
            crate::CssRatioOperand::try_from_component(component)
                .map_err(|_| unexpected_at(location))
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &start, numeric, CalculationRoot::Number)
                .map(CssNumberCalculation::from_expression)
                .and_then(|calculation| {
                    crate::CssRatioOperand::try_from_calculation(calculation)
                        .map_err(|_| unexpected_at(location))
                })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_scrollbar_width<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssScrollbarWidth, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "auto" => Ok(CssScrollbarWidth::Auto),
            "thin" => Ok(CssScrollbarWidth::Thin),
            "none" => Ok(CssScrollbarWidth::None),
            _ => Err(unexpected_at(location)),
        },
        _ => Err(unexpected_at(location)),
    }
}

pub(super) fn parse_order<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssIntegerValue, ParseError<'i, Error>> {
    parse_integer_value(input, numeric)
}

pub(super) fn parse_z_index<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssZIndexValue, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssZIndexValue::Auto),
        Token::Ident(_) => Err(unexpected_at(location)),
        Token::Number { .. } => {
            input.reset(&numeric_start);
            parse_integer_literal(input, numeric)
                .map(CssIntegerValue::Literal)
                .map(CssZIndexValue::Integer)
        }
        Token::Dimension { .. } => Err(unexpected_at(location)),
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .map(CssIntegerValue::Calculation)
                .map(CssZIndexValue::Integer)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}
