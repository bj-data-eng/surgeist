use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use super::values::{
    parse_border_width_component, parse_color, parse_radius_component, parse_shadow_blur_length,
    parse_shadow_length,
};
use crate::error::{CssFeatureId, Error, basic, unsupported_value};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.box-edge-keywords")];

pub(super) fn parse_box_decoration_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBoxDecorationBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "slice" => Ok(CssBoxDecorationBreak::Slice),
        "clone" => Ok(CssBoxDecorationBreak::Clone),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("box-decoration-break", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_edges<'i, 't>(
    input: &mut Parser<'i, 't>,
    mut parse_component: impl FnMut(
        &mut Parser<'i, 't>,
    ) -> std::result::Result<CssLength, ParseError<'i, Error>>,
) -> std::result::Result<CssEdges, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() {
        values.push(parse_component(input)?);
        if values.len() == 4 && !input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "edge shorthand has too many values",
            ));
        }
    }
    Ok(match values.as_slice() {
        [all] => CssEdges::all(all.clone()),
        [vertical, horizontal] => CssEdges::new(
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ),
        [top, horizontal, bottom] => CssEdges::new(
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ),
        [top, right, bottom, left] => {
            CssEdges::new(top.clone(), right.clone(), bottom.clone(), left.clone())
        }
        [] => {
            return Err(unsupported_value(
                input,
                None,
                "edge shorthand is missing a value",
            ));
        }
        _ => unreachable!("edge shorthand parser caps values at four"),
    })
}

pub(super) fn parse_border_styles<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderStyles, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() {
        values.push(parse_border_style(input)?);
        if values.len() == 4 && !input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "border-style shorthand has too many values",
            ));
        }
    }
    Ok(match values.as_slice() {
        [all] => CssBorderStyles::all(*all),
        [vertical, horizontal] => {
            CssBorderStyles::new(*vertical, *horizontal, *vertical, *horizontal)
        }
        [top, horizontal, bottom] => CssBorderStyles::new(*top, *horizontal, *bottom, *horizontal),
        [top, right, bottom, left] => CssBorderStyles::new(*top, *right, *bottom, *left),
        [] => {
            return Err(unsupported_value(
                input,
                None,
                "border-style shorthand is missing a value",
            ));
        }
        _ => unreachable!("border-style shorthand parser caps values at four"),
    })
}

pub(super) fn parse_border_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssBorderStyle::None),
        "hidden" => Ok(CssBorderStyle::Hidden),
        "dotted" => Ok(CssBorderStyle::Dotted),
        "dashed" => Ok(CssBorderStyle::Dashed),
        "solid" => Ok(CssBorderStyle::Solid),
        "double" => Ok(CssBorderStyle::Double),
        "groove" => Ok(CssBorderStyle::Groove),
        "ridge" => Ok(CssBorderStyle::Ridge),
        "inset" => Ok(CssBorderStyle::Inset),
        "outset" => Ok(CssBorderStyle::Outset),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("border-style", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_border<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorder, ParseError<'i, Error>> {
    let mut width = None;
    let mut style = None;
    let mut color = None;

    while !input.is_exhausted() {
        if let Ok(parsed_width) = input.try_parse(parse_border_width_component) {
            if width.replace(parsed_width).is_some() {
                return Err(unsupported_value(input, None, "duplicate border width"));
            }
            continue;
        }
        if let Ok(parsed_style) = input.try_parse(parse_border_style) {
            if style.replace(parsed_style).is_some() {
                return Err(unsupported_value(input, None, "duplicate border style"));
            }
            continue;
        }
        if let Ok(parsed_color) = input.try_parse(parse_color) {
            if color.replace(parsed_color).is_some() {
                return Err(unsupported_value(input, None, "duplicate border color"));
            }
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported border component",
        ));
    }

    if width.is_none() && style.is_none() && color.is_none() {
        Err(unsupported_value(
            input,
            None,
            "border shorthand is missing a component",
        ))
    } else {
        Ok(CssBorder::new_current(width, style, color))
    }
}

pub(super) fn parse_corner_radius<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCornerRadius, ParseError<'i, Error>> {
    let horizontal = parse_radius_component(input)?;
    let vertical = if input.is_exhausted() {
        horizontal.clone()
    } else {
        parse_radius_component(input)?
    };
    Ok(CssCornerRadius::new(horizontal, vertical))
}

pub(super) fn parse_border_radius<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderRadii, ParseError<'i, Error>> {
    let horizontal = parse_radius_component_list(input)?;
    if horizontal.is_empty() {
        return Err(unsupported_value(
            input,
            None,
            "border-radius shorthand is missing a value",
        ));
    }

    let vertical = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        let vertical = parse_radius_component_list(input)?;
        if vertical.is_empty() {
            return Err(unsupported_value(
                input,
                None,
                "border-radius slash is missing vertical radii",
            ));
        }
        vertical
    } else {
        horizontal.clone()
    };

    let (h_top_left, h_top_right, h_bottom_right, h_bottom_left) =
        expand_radius_components(horizontal);
    let (v_top_left, v_top_right, v_bottom_right, v_bottom_left) =
        expand_radius_components(vertical);

    Ok(CssBorderRadii::new(
        CssCornerRadius::new(h_top_left, v_top_left),
        CssCornerRadius::new(h_top_right, v_top_right),
        CssCornerRadius::new(h_bottom_right, v_bottom_right),
        CssCornerRadius::new(h_bottom_left, v_bottom_left),
    ))
}

pub(super) fn parse_radius_component_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Vec<CssLength>, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() {
        let state = input.state();
        if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            input.reset(&state);
            break;
        }

        values.push(parse_radius_component(input)?);
        if values.len() == 4 && !input.is_exhausted() {
            let state = input.state();
            let slash_is_next = input.try_parse(|input| input.expect_delim('/')).is_ok();
            input.reset(&state);
            if !slash_is_next {
                return Err(unsupported_value(
                    input,
                    None,
                    "border-radius shorthand has too many values",
                ));
            }
        }
    }
    Ok(values)
}

pub(super) fn expand_radius_components(
    values: Vec<CssLength>,
) -> (CssLength, CssLength, CssLength, CssLength) {
    match values.as_slice() {
        [all] => (all.clone(), all.clone(), all.clone(), all.clone()),
        [vertical, horizontal] => (
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ),
        [top_left, top_right_bottom_left, bottom_right] => (
            top_left.clone(),
            top_right_bottom_left.clone(),
            bottom_right.clone(),
            top_right_bottom_left.clone(),
        ),
        [top_left, top_right, bottom_right, bottom_left] => (
            top_left.clone(),
            top_right.clone(),
            bottom_right.clone(),
            bottom_left.clone(),
        ),
        [] => unreachable!("caller validates non-empty border-radius components"),
        _ => unreachable!("border-radius component parser caps values at four"),
    }
}

pub(super) fn parse_box_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBoxShadow, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned)
        && ident.eq_ignore_ascii_case("none")
        && input.is_exhausted()
    {
        return Ok(CssBoxShadow::None);
    }
    input.reset(&state);

    let mut shadows = Vec::new();
    loop {
        shadows.push(parse_shadow(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "box-shadow list has an empty item",
            ));
        }
    }

    Ok(CssBoxShadow::Shadows(
        CssBoxShadowList::new(shadows).expect("box-shadow parser records at least one shadow"),
    ))
}

pub(super) fn parse_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssShadow, ParseError<'i, Error>> {
    let mut inset = false;
    let mut color = None;
    let mut lengths = Vec::new();

    while !input.is_exhausted() {
        let state = input.state();
        if input.try_parse(Parser::expect_comma).is_ok() {
            input.reset(&state);
            break;
        }

        if input
            .try_parse(|input| input.expect_ident_matching("inset"))
            .is_ok()
        {
            if inset {
                return Err(unsupported_value(input, None, "duplicate box-shadow inset"));
            }
            inset = true;
            continue;
        }

        if let Ok(parsed_color) = input.try_parse(parse_color) {
            if color.replace(parsed_color).is_some() {
                return Err(unsupported_value(input, None, "duplicate box-shadow color"));
            }
            continue;
        }

        let parsed_length = match lengths.len() {
            0 | 1 | 3 => input.try_parse(parse_shadow_length).ok(),
            2 => input.try_parse(parse_shadow_blur_length).ok(),
            _ => None,
        };
        if let Some(parsed_length) = parsed_length {
            lengths.push(parsed_length);
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported box-shadow component",
        ));
    }

    match lengths.as_slice() {
        [offset_x, offset_y] => Ok(CssShadow::new_current(
            inset,
            offset_x.clone(),
            offset_y.clone(),
            None,
            None,
            color,
        )),
        [offset_x, offset_y, blur] => Ok(CssShadow::new_current(
            inset,
            offset_x.clone(),
            offset_y.clone(),
            Some(blur.clone()),
            None,
            color,
        )),
        [offset_x, offset_y, blur, spread] => Ok(CssShadow::new_current(
            inset,
            offset_x.clone(),
            offset_y.clone(),
            Some(blur.clone()),
            Some(spread.clone()),
            color,
        )),
        _ => Err(unsupported_value(
            input,
            None,
            "box-shadow requires at least two offsets",
        )),
    }
}

pub(super) fn parse_drop_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssDropShadow, ParseError<'i, Error>> {
    let mut color = None;
    let mut lengths = Vec::new();

    while !input.is_exhausted() {
        if let Ok(parsed_color) = input.try_parse(parse_color) {
            if color.replace(parsed_color).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate drop-shadow color",
                ));
            }
            continue;
        }

        let parsed_length = match lengths.len() {
            0 | 1 => input.try_parse(parse_shadow_length).ok(),
            2 => input.try_parse(parse_shadow_blur_length).ok(),
            _ => None,
        };
        if let Some(parsed_length) = parsed_length {
            lengths.push(parsed_length);
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported drop-shadow component",
        ));
    }

    match lengths.as_slice() {
        [offset_x, offset_y] => {
            CssDropShadow::try_new_current(offset_x.clone(), offset_y.clone(), None, color)
        }
        [offset_x, offset_y, blur] => CssDropShadow::try_new_current(
            offset_x.clone(),
            offset_y.clone(),
            Some(blur.clone()),
            color,
        ),
        _ => None,
    }
    .ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "drop-shadow requires two offsets and an optional non-negative blur",
        )
    })
}
