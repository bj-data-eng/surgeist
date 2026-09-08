use cssparser::{ParseError, Parser, ToCss, Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, LengthGrammar, checked_percentage_value, parse_box_size_value,
    parse_length_with, parse_typed_calculation,
};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) fn parse_resize<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssResize, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssResize::None),
        "both" => Ok(CssResize::Both),
        "horizontal" => Ok(CssResize::Horizontal),
        "vertical" => Ok(CssResize::Vertical),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("resize", ident.as_ref()),
        )),
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
            "paint" => CssContainComponent::Paint,
            _ => return Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("contain", ident.as_ref()),
            )),
        };
        if components.contains(&component) {
            return Err(unsupported_value_at(
                location,
                None,
                format!("duplicate contain component `{ident}`"),
            ));
        }
        components.push(component);
    }
    CssContainComponentList::try_new(components)
        .map(CssContain::Components)
        .ok_or_else(|| unsupported_value(input, None, "contain component list is empty"))
}

pub(super) fn parse_border_collapse<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderCollapse, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "collapse" => Ok(CssBorderCollapse::Collapse),
        "separate" => Ok(CssBorderCollapse::Separate),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("border-collapse", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_border_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderSpacing, ParseError<'i, Error>> {
    let horizontal = parse_length_with(input, LengthGrammar::BorderSpacing)?;
    let vertical = if input.is_exhausted() {
        horizontal.clone()
    } else {
        parse_length_with(input, LengthGrammar::BorderSpacing)?
    };
    CssBorderSpacing::try_new(horizontal, vertical).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "border-spacing requires one or two non-negative lengths",
        )
    })
}

pub(super) fn parse_caption_side<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCaptionSide, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "top" => Ok(CssCaptionSide::Top),
        "bottom" => Ok(CssCaptionSide::Bottom),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("caption-side", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_empty_cells<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssEmptyCells, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "show" => Ok(CssEmptyCells::Show),
        "hide" => Ok(CssEmptyCells::Hide),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("empty-cells", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_page_line_minimum<'i, 't>(
    input: &mut Parser<'i, 't>,
    property: &str,
) -> std::result::Result<CssPageLineMinimum, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => CssPageLineMinimum::try_literal(*value).ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                format!("{property} must be a positive integer"),
            )
        }),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            format!("{property} must be an integer"),
        )),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Integer))
            .map(CssIntegerCalculation::from_expression)
            .map(CssPageLineMinimum::from_calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_page_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPageBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssPageBreak::Auto),
        "always" => Ok(CssPageBreak::Always),
        "avoid" => Ok(CssPageBreak::Avoid),
        "left" => Ok(CssPageBreak::Left),
        "right" => Ok(CssPageBreak::Right),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("page-break", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_page_break_inside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPageBreakInside, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssPageBreakInside::Auto),
        "avoid" => Ok(CssPageBreakInside::Avoid),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("page-break-inside", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_table_layout<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTableLayout, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssTableLayout::Auto),
        "fixed" => Ok(CssTableLayout::Fixed),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("table-layout", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_display<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssDisplay, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "block" => Ok(CssDisplay::Block),
        "flex" => Ok(CssDisplay::Flex),
        "grid" => Ok(CssDisplay::Grid),
        "inline-block" => Ok(CssDisplay::InlineBlock),
        "inline-grid" => Ok(CssDisplay::InlineGrid),
        "grid-lanes" => Ok(CssDisplay::GridLanes),
        "inline-grid-lanes" => Ok(CssDisplay::InlineGridLanes),
        "none" => Ok(CssDisplay::None),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("display", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_box_sizing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBoxSizing, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "content-box" => Ok(CssBoxSizing::ContentBox),
        "border-box" => Ok(CssBoxSizing::BorderBox),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("box-sizing", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("position", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssDirection, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "ltr" => Ok(CssDirection::Ltr),
        "rtl" => Ok(CssDirection::Rtl),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("direction", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("overflow", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_overflow_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssValue, ParseError<'i, Error>> {
    let x = parse_overflow(input)?;
    if input.is_exhausted() {
        Ok(CssValue::Overflow(x))
    } else {
        let y = parse_overflow(input)?;
        Ok(CssValue::OverflowAxes(CssOverflowAxes::new(x, y)))
    }
}

pub(super) fn parse_flex_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexDirection, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "row" => Ok(CssFlexDirection::Row),
        "column" => Ok(CssFlexDirection::Column),
        "row-reverse" => Ok(CssFlexDirection::RowReverse),
        "column-reverse" => Ok(CssFlexDirection::ColumnReverse),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("flex-direction", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_flex_flow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexFlow, ParseError<'i, Error>> {
    let mut direction = None;
    let mut wrap = None;

    while !input.is_exhausted() {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        match_ignore_ascii_case! { &ident,
            "row" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::Row,
                location,
                ident.as_ref(),
            )?,
            "column" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::Column,
                location,
                ident.as_ref(),
            )?,
            "row-reverse" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::RowReverse,
                location,
                ident.as_ref(),
            )?,
            "column-reverse" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::ColumnReverse,
                location,
                ident.as_ref(),
            )?,
            "nowrap" => set_flex_flow_wrap(
                &mut wrap,
                CssFlexWrap::NoWrap,
                location,
                ident.as_ref(),
            )?,
            "wrap" => set_flex_flow_wrap(
                &mut wrap,
                CssFlexWrap::Wrap,
                location,
                ident.as_ref(),
            )?,
            "wrap-reverse" => set_flex_flow_wrap(
                &mut wrap,
                CssFlexWrap::WrapReverse,
                location,
                ident.as_ref(),
            )?,
            _ => return Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("flex-flow", ident.as_ref()),
            )),
        }
    }

    if direction.is_none() && wrap.is_none() {
        return Err(unsupported_value(
            input,
            None,
            "flex-flow requires a direction or wrapping component",
        ));
    }

    Ok(CssFlexFlow::new(
        direction.unwrap_or(CssFlexDirection::Row),
        wrap.unwrap_or(CssFlexWrap::NoWrap),
    ))
}

fn set_flex_flow_direction<'i>(
    slot: &mut Option<CssFlexDirection>,
    value: CssFlexDirection,
    location: cssparser::SourceLocation,
    ident: &str,
) -> std::result::Result<(), ParseError<'i, Error>> {
    if slot.replace(value).is_some() {
        return Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("flex-flow direction", ident),
        ));
    }
    Ok(())
}

fn set_flex_flow_wrap<'i>(
    slot: &mut Option<CssFlexWrap>,
    value: CssFlexWrap,
    location: cssparser::SourceLocation,
    ident: &str,
) -> std::result::Result<(), ParseError<'i, Error>> {
    if slot.replace(value).is_some() {
        return Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("flex-flow wrap", ident),
        ));
    }
    Ok(())
}

pub(super) fn parse_flex_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexWrap, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "nowrap" => Ok(CssFlexWrap::NoWrap),
        "wrap" => Ok(CssFlexWrap::Wrap),
        "wrap-reverse" => Ok(CssFlexWrap::WrapReverse),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("flex-wrap", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_float<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFloat, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "left" => Ok(CssFloat::Left),
        "right" => Ok(CssFloat::Right),
        "none" => Ok(CssFloat::None),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("float", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("clear", ident.as_ref()),
        )),
    }
}

#[derive(Clone, Copy)]
pub(super) struct AllowedAlignmentKeywords {
    normal: bool,
    distribution: bool,
}

impl AllowedAlignmentKeywords {
    const fn content() -> Self {
        Self {
            normal: true,
            distribution: true,
        }
    }

    const fn item() -> Self {
        Self {
            normal: true,
            distribution: false,
        }
    }
}

pub(super) fn parse_content_alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAlignment, ParseError<'i, Error>> {
    parse_alignment(input, AllowedAlignmentKeywords::content())
}

pub(super) fn parse_align_items<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAlignItems, ParseError<'i, Error>> {
    let alignment = parse_alignment(input, AllowedAlignmentKeywords::item())?;
    Ok(match alignment {
        CssAlignment::Normal => CssAlignItems::Normal,
        CssAlignment::Start => CssAlignItems::Start,
        CssAlignment::End => CssAlignItems::End,
        CssAlignment::SafeEnd => CssAlignItems::SafeEnd,
        CssAlignment::FlexStart => CssAlignItems::FlexStart,
        CssAlignment::FlexEnd => CssAlignItems::FlexEnd,
        CssAlignment::SafeFlexEnd => CssAlignItems::SafeFlexEnd,
        CssAlignment::Center => CssAlignItems::Center,
        CssAlignment::SafeCenter => CssAlignItems::SafeCenter,
        CssAlignment::Baseline => CssAlignItems::Baseline,
        CssAlignment::FirstBaseline => CssAlignItems::FirstBaseline,
        CssAlignment::LastBaseline => CssAlignItems::LastBaseline,
        CssAlignment::Stretch => CssAlignItems::Stretch,
        CssAlignment::SpaceBetween | CssAlignment::SpaceAround | CssAlignment::SpaceEvenly => {
            unreachable!("item alignment parser disables distribution keywords")
        }
    })
}

pub(super) fn parse_alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: AllowedAlignmentKeywords,
) -> std::result::Result<CssAlignment, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    let first = first.to_ascii_lowercase();
    let safe = first == "safe";
    let has_overflow_prefix = safe || first == "unsafe";
    let keyword = if has_overflow_prefix {
        input
            .expect_ident_cloned()
            .map_err(basic)?
            .to_ascii_lowercase()
    } else {
        first.clone()
    };
    let original = if has_overflow_prefix {
        format!("{first} {keyword}")
    } else {
        keyword.clone()
    };

    if first == "unsafe" {
        return Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("alignment", original),
        ));
    }

    match keyword.as_str() {
        "normal" if options.normal && !has_overflow_prefix => Ok(CssAlignment::Normal),
        "start" if !has_overflow_prefix => Ok(CssAlignment::Start),
        "end" if safe => Ok(CssAlignment::SafeEnd),
        "end" => Ok(CssAlignment::End),
        "flex-start" if !has_overflow_prefix => Ok(CssAlignment::FlexStart),
        "flex-end" if safe => Ok(CssAlignment::SafeFlexEnd),
        "flex-end" => Ok(CssAlignment::FlexEnd),
        "center" if safe => Ok(CssAlignment::SafeCenter),
        "center" => Ok(CssAlignment::Center),
        "baseline" if !has_overflow_prefix => Ok(CssAlignment::Baseline),
        "first" => {
            let baseline = input.expect_ident_cloned().map_err(basic)?;
            if has_overflow_prefix {
                Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("alignment", format!("{first} first {baseline}")),
                ))
            } else if baseline.eq_ignore_ascii_case("baseline") {
                Ok(CssAlignment::FirstBaseline)
            } else {
                Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("alignment", format!("first {baseline}")),
                ))
            }
        }
        "last" => {
            let baseline = input.expect_ident_cloned().map_err(basic)?;
            if has_overflow_prefix {
                Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("alignment", format!("{first} last {baseline}")),
                ))
            } else if baseline.eq_ignore_ascii_case("baseline") {
                Ok(CssAlignment::LastBaseline)
            } else {
                Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("alignment", format!("last {baseline}")),
                ))
            }
        }
        "stretch" if !has_overflow_prefix => Ok(CssAlignment::Stretch),
        "space-between" if options.distribution && !has_overflow_prefix => {
            Ok(CssAlignment::SpaceBetween)
        }
        "space-around" if options.distribution && !has_overflow_prefix => {
            Ok(CssAlignment::SpaceAround)
        }
        "space-evenly" if options.distribution && !has_overflow_prefix => {
            Ok(CssAlignment::SpaceEvenly)
        }
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("alignment", original),
        )),
    }
}

pub(super) fn parse_place_alignment<'i, 't, T: Copy>(
    input: &mut Parser<'i, 't>,
    mut parse_component: impl FnMut(
        &mut Parser<'i, 't>,
    ) -> std::result::Result<T, ParseError<'i, Error>>,
    make: impl Fn(T, T) -> CssPlaceAlignment,
) -> std::result::Result<CssPlaceAlignment, ParseError<'i, Error>> {
    let first = parse_component(input)?;
    let second = if input.is_exhausted() {
        first
    } else {
        parse_component(input)?
    };
    if !input.is_exhausted() {
        return Err(unsupported_value(
            input,
            None,
            "place alignment shorthand has too many values",
        ));
    }
    Ok(make(first, second))
}

pub(super) fn parse_visibility<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssVisibility, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "visible" => Ok(CssVisibility::Visible),
        "hidden" => Ok(CssVisibility::Hidden),
        "collapse" => Ok(CssVisibility::Collapse),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("visibility", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("content-visibility", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_opacity<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOpacityValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => {
            let value = CssFiniteNumber::try_new(*value).ok_or_else(|| {
                unsupported_value_at(location, None, "opacity number must be finite")
            })?;
            Ok(CssOpacity::try_new(value.value())
                .map(CssOpacityValue::Literal)
                .unwrap_or(CssOpacityValue::Number(value)))
        }
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                "opacity percentage must be finite",
            )?;
            CssFiniteNumber::try_new(value)
                .map(CssOpacityValue::Percentage)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "opacity percentage must be finite")
                })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            if let Ok(calculation) = input.try_parse(|input| {
                input.parse_nested_block(|input| {
                    parse_typed_calculation(input, CalculationRoot::Number)
                })
            }) {
                return Ok(CssOpacityValue::Calculation(
                    CssNumberCalculation::from_expression(calculation),
                ));
            }
            input
                .parse_nested_block(|input| {
                    parse_typed_calculation(input, CalculationRoot::Percentage)
                })
                .map(CssPercentageCalculation::from_expression)
                .map(CssOpacityValue::PercentageCalculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_flex_factor<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<CssNonNegativeNumberValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => CssNonNegativeNumber::try_new(*value)
            .map(CssNonNegativeNumberValue::Literal)
            .ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    format!("{context} must be a finite non-negative number"),
                )
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
            .map(CssNumberCalculation::from_expression)
            .map(CssNonNegativeNumberValue::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_aspect_ratio<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAspectRatioValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => CssAspectRatio::try_new(*value)
            .map(CssAspectRatioValue::Literal)
            .ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    "aspect-ratio must be a finite positive number",
                )
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
            .map(CssNumberCalculation::from_expression)
            .map(CssAspectRatioValue::Calculation),
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
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("scrollbar-width", ident.as_ref()),
            )),
        },
        token => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported scrollbar-width `{}`", token.to_css_string()),
        )),
    }
}

pub(super) fn parse_order<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssIntegerValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => Ok(CssIntegerValue::Literal(*value)),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            "order must be an integer",
        )),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Integer))
            .map(CssIntegerCalculation::from_expression)
            .map(CssIntegerValue::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_flex<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexValue, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "none" => Ok(CssFlexValue::None),
            "auto" => Ok(CssFlexValue::Auto),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("flex", ident.as_ref()),
            )),
        };
    }

    let grow = parse_flex_factor(input, "flex-grow")?;
    let mut shrink = None;
    let mut basis = None;
    if !input.is_exhausted() {
        if let Ok(parsed_shrink) = input.try_parse(|input| parse_flex_factor(input, "flex-shrink"))
        {
            shrink = Some(parsed_shrink);
            if !input.is_exhausted() {
                basis = Some(parse_box_size_value(input)?);
            }
        } else {
            basis = Some(parse_box_size_value(input)?);
        }
    }
    Ok(CssFlexValue::Components(CssFlexComponents::new(
        grow, shrink, basis,
    )))
}

pub(super) fn parse_z_index<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssZIndexValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssZIndexValue::Auto),
        Token::Ident(ident) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported z-index `{ident}`"),
        )),
        Token::Number {
            int_value: Some(value),
            ..
        } => Ok(CssZIndexValue::Integer(CssIntegerValue::Literal(*value))),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            "unsupported z-index non-integer number",
        )),
        Token::Dimension { unit, .. } => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported z-index length unit `{unit}`"),
        )),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Integer))
            .map(CssIntegerCalculation::from_expression)
            .map(CssIntegerValue::Calculation)
            .map(CssZIndexValue::Integer),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}
