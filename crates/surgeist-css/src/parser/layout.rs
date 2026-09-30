use super::values::parse_nonnegative_length;
use cssparser::{ParseError, Parser, ToCss, Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, parse_integer_literal, parse_integer_value, parse_numeric_function,
};
use crate::CssOverflowValue;
use crate::display::*;
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderSpacing, ParseError<'i, Error>> {
    let horizontal = parse_nonnegative_length(input, numeric, "border-spacing")?;
    let vertical = if input.is_exhausted() {
        horizontal.clone()
    } else {
        parse_nonnegative_length(input, numeric, "border-spacing")?
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
    numeric: &crate::numeric::NumericInputContext<'_>,
    property: &str,
) -> std::result::Result<CssPageLineMinimum, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&numeric_start);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, format!("invalid {property} integer"))
            })?;
            CssPageLineMinimum::try_from_component(component).map_err(|_| {
                unsupported_value_at(
                    location,
                    None,
                    format!("{property} must be a positive integer"),
                )
            })
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .and_then(|value| {
                    CssPageLineMinimum::try_from_calculation(value).map_err(|_| {
                        unsupported_value_at(
                            location,
                            None,
                            format!("invalid {property} integer calculation"),
                        )
                    })
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("break-before/after", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("break-inside", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(input, None,
            unsupported_keyword_reason("page-break-before/after", ident.as_ref()))),
    }
}

pub(super) fn parse_page_break_inside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBreakInside, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssBreakInside::Auto),
        "avoid" => Ok(CssBreakInside::Avoid),
        _ => Err(unsupported_value(input, None,
            unsupported_keyword_reason("page-break-inside", ident.as_ref()))),
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
            return Err(unsupported_value(
                input,
                None,
                "display has too many components",
            ));
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
            _ => return Err(unsupported_value_at(location, None, unsupported_keyword_reason("display", ident.as_ref()))),
        };
        let duplicate = match component {
            Component::Outside(value) => outside.replace(value).is_some(),
            Component::Inside(value) => inside.replace(value).is_some(),
            Component::ListItem => std::mem::replace(&mut list_item, true),
            Component::Exclusive(value) => {
                if count == 1 && input.is_exhausted() {
                    return Ok(value);
                }
                return Err(unsupported_value_at(
                    location,
                    None,
                    "exclusive display keyword cannot be combined",
                ));
            }
        };
        if duplicate {
            return Err(unsupported_value_at(
                location,
                None,
                "duplicate display component category",
            ));
        }
    }
    if count == 0 {
        return Err(unsupported_value(input, None, "display value is empty"));
    }
    if list_item {
        let inside = match inside {
            None | Some(CssDisplayInside::Flow) => CssDisplayListItemInside::Flow,
            Some(CssDisplayInside::FlowRoot) => CssDisplayListItemInside::FlowRoot,
            Some(_) => {
                return Err(unsupported_value(
                    input,
                    None,
                    "list-item permits only flow or flow-root",
                ));
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
        "auto" | "overlay" => Ok(CssOverflow::Auto),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("overflow", ident.as_ref()),
        )),
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
        "inline-start" => Ok(CssClear::InlineStart),
        "inline-end" => Ok(CssClear::InlineEnd),
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
}

pub(super) fn parse_content_alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<crate::CssAlignmentValue, ParseError<'i, Error>> {
    parse_alignment(input, AllowedAlignmentKeywords::content())
}

pub(super) fn parse_alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
    options: AllowedAlignmentKeywords,
) -> std::result::Result<crate::CssAlignmentValue, ParseError<'i, Error>> {
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
        "normal" if options.normal && !has_overflow_prefix => {
            Ok(crate::CssAlignmentValue::Normal { overflow: None })
        }
        "start" if !has_overflow_prefix => Ok(crate::CssAlignmentValue::Position {
            overflow: None,
            position: crate::CssAlignmentPosition::Start,
        }),
        "end" if safe => Ok(crate::CssAlignmentValue::Position {
            overflow: Some(crate::CssOverflowPosition::Safe),
            position: crate::CssAlignmentPosition::End,
        }),
        "end" => Ok(crate::CssAlignmentValue::Position {
            overflow: None,
            position: crate::CssAlignmentPosition::End,
        }),
        "flex-start" if !has_overflow_prefix => Ok(crate::CssAlignmentValue::Position {
            overflow: None,
            position: crate::CssAlignmentPosition::FlexStart,
        }),
        "flex-end" if safe => Ok(crate::CssAlignmentValue::Position {
            overflow: Some(crate::CssOverflowPosition::Safe),
            position: crate::CssAlignmentPosition::FlexEnd,
        }),
        "flex-end" => Ok(crate::CssAlignmentValue::Position {
            overflow: None,
            position: crate::CssAlignmentPosition::FlexEnd,
        }),
        "center" if safe => Ok(crate::CssAlignmentValue::Position {
            overflow: Some(crate::CssOverflowPosition::Safe),
            position: crate::CssAlignmentPosition::Center,
        }),
        "center" => Ok(crate::CssAlignmentValue::Position {
            overflow: None,
            position: crate::CssAlignmentPosition::Center,
        }),
        "baseline" if !has_overflow_prefix => Ok(crate::CssAlignmentValue::Baseline(
            crate::CssBaselinePosition::Baseline,
        )),
        "first" => {
            let baseline = input.expect_ident_cloned().map_err(basic)?;
            if has_overflow_prefix {
                Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("alignment", format!("{first} first {baseline}")),
                ))
            } else if baseline.eq_ignore_ascii_case("baseline") {
                Ok(crate::CssAlignmentValue::Baseline(
                    crate::CssBaselinePosition::First,
                ))
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
                Ok(crate::CssAlignmentValue::Baseline(
                    crate::CssBaselinePosition::Last,
                ))
            } else {
                Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("alignment", format!("last {baseline}")),
                ))
            }
        }
        "stretch" if !has_overflow_prefix => Ok(crate::CssAlignmentValue::Stretch),
        "space-between" if options.distribution && !has_overflow_prefix => {
            Ok(crate::CssAlignmentValue::SpaceBetween)
        }
        "space-around" if options.distribution && !has_overflow_prefix => {
            Ok(crate::CssAlignmentValue::SpaceAround)
        }
        "space-evenly" if options.distribution && !has_overflow_prefix => {
            Ok(crate::CssAlignmentValue::SpaceEvenly)
        }
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("alignment", original),
        )),
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOpacityValue, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Percentage { .. } => {
            input.reset(&numeric_start);
            input.skip_whitespace();
            let offset = input.position().byte_index();
            let component = numeric.collect(input).map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, offset),
                    None,
                    "invalid opacity scalar component",
                )
            })?;
            crate::opacity_scalar::admit_opacity_scalar(component)
                .map_err(|_| unsupported_value_at(location, None, "invalid opacity scalar token"))
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            if let Ok(calculation) = input.try_parse(|input| {
                parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Number)
            }) {
                return Ok(CssOpacityValue::NumberCalculation(
                    CssNumberCalculation::from_expression(calculation),
                ));
            }
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Percentage)
                .map(CssPercentageCalculation::from_expression)
                .map(CssOpacityValue::PercentageCalculation)
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
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid ratio number component")
            })?;
            crate::CssRatioOperand::try_from_component(component).map_err(|_| {
                unsupported_value_at(location, None, "ratio number must be nonnegative")
            })
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &start, numeric, CalculationRoot::Number)
                .map(CssNumberCalculation::from_expression)
                .and_then(|calculation| {
                    crate::CssRatioOperand::try_from_calculation(calculation).map_err(|_| {
                        unsupported_value_at(location, None, "invalid ratio number math")
                    })
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
        Token::Ident(ident) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported z-index `{ident}`"),
        )),
        Token::Number { .. } => {
            input.reset(&numeric_start);
            parse_integer_literal(input, numeric).map(CssZIndexValue::Integer)
        }
        Token::Dimension { unit, .. } => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported z-index length unit `{unit}`"),
        )),
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .map(CssIntegerValue::Calculation)
                .map(CssZIndexValue::Integer)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}
