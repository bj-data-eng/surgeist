//! Masking-specific orchestration around shared image and four-side owners.
use super::background::{
    next_starts_background_image, next_starts_border_image_repeat, next_starts_border_image_slice,
    parse_background_repeat_prefix, parse_background_size_prefix, parse_border_image_outset_prefix,
    parse_border_image_repeat_prefix, parse_border_image_slice_component,
    parse_border_image_width_prefix, parse_image_value,
};
use super::position::parse_physical_position_prefix;
use super::values::next_is_comma;
use crate::error::{Error, basic, unexpected_at};
use crate::syntax::*;
use cssparser::{ParseError, Parser};

macro_rules! keyword_parser {
    ($function:ident, $ty:ident, $($keyword:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $function<'i, 't>(input: &mut Parser<'i, 't>) -> Result<$ty, ParseError<'i, Error>> {
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match ident.to_ascii_lowercase().as_str() {
                $($keyword => Ok($ty::$variant),)+
                _ => Err(unexpected_at(input.current_source_location())),
            }
        }
    };
}
keyword_parser!(parse_mask_mode, CssMaskMode, "alpha" => Alpha, "luminance" => Luminance, "match-source" => MatchSource);
keyword_parser!(parse_mask_composite, CssMaskComposite, "add" => Add, "subtract" => Subtract, "intersect" => Intersect, "exclude" => Exclude);
keyword_parser!(parse_mask_type, CssMaskType, "alpha" => Alpha, "luminance" => Luminance);
keyword_parser!(parse_clip_rule, CssClipRule, "nonzero" => Nonzero, "evenodd" => Evenodd);

pub(super) fn parse_mask_box<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssMaskBox, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssBoxEdgeKeyword::from_keyword(&ident)
        .and_then(CssMaskBox::try_new)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_mask_clip<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssMaskClip, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("no-clip"))
        .is_ok()
    {
        return Ok(CssMaskClip::NoClip);
    }
    parse_mask_box(input).map(CssMaskClip::Box)
}

macro_rules! list_parser {
    ($function:ident, $list:ident, $parse:ident) => {
        pub(super) fn $function<'i, 't>(
            input: &mut Parser<'i, 't>,
        ) -> Result<$list, ParseError<'i, Error>> {
            let values = input.parse_comma_separated($parse)?;
            $list::try_new(values).ok_or_else(|| unexpected_at(input.current_source_location()))
        }
    };
}
list_parser!(parse_mask_mode_list, CssMaskModeList, parse_mask_mode);
list_parser!(parse_mask_origin_list, CssMaskBoxList, parse_mask_box);
list_parser!(parse_mask_clip_list, CssMaskClipList, parse_mask_clip);
list_parser!(
    parse_mask_composite_list,
    CssMaskCompositeList,
    parse_mask_composite
);

/// Masking's fill is trailing, unlike Backgrounds' independent && fill grammar.
pub(super) fn parse_mask_border_slice<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssBorderImageSlice, ParseError<'i, Error>> {
    let mut values = vec![parse_border_image_slice_component(input, numeric)?];
    while values.len() < 4 {
        match input.try_parse(|input| parse_border_image_slice_component(input, numeric)) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }
    let fill = input
        .try_parse(|input| input.expect_ident_matching("fill"))
        .is_ok();
    Ok(CssBorderImageSlice::try_new(values, fill).expect("one to four checked components"))
}

pub(super) fn parse_mask_border<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssMaskBorder, ParseError<'i, Error>> {
    let (mut source, mut slice, mut width, mut outset, mut repeat, mut mode) =
        (None, None, None, None, None, None);
    while !input.is_exhausted() {
        if source.is_none() && next_starts_background_image(input) {
            source = Some(parse_image_value(input, numeric)?);
            continue;
        }
        if slice.is_none() && next_starts_border_image_slice(input) {
            slice = Some(parse_mask_border_slice(input, numeric)?);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                // The first width is optional, including the source's bare 10 /.
                if !input.is_exhausted()
                    && let Ok(value) =
                        input.try_parse(|input| parse_border_image_width_prefix(input, numeric))
                {
                    width = Some(value);
                }
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    outset = Some(parse_border_image_outset_prefix(input, numeric)?);
                }
            }
            continue;
        }
        if repeat.is_none() && next_starts_border_image_repeat(input) {
            repeat = Some(parse_border_image_repeat_prefix(input)?);
            continue;
        }
        if mode.is_none()
            && let Ok(value) = input.try_parse(parse_mask_type)
        {
            mode = Some(value);
            continue;
        }
        return Err(unexpected_at(input.current_source_location()));
    }
    CssMaskBorder::try_new(source, slice, width, outset, repeat, mode)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_mask_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssMaskList, ParseError<'i, Error>> {
    let mut layers = Vec::new();
    loop {
        layers.push(parse_mask_layer(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    if layers.is_empty() {
        Err(unexpected_at(input.current_source_location()))
    } else {
        Ok(CssMaskList::new(layers))
    }
}

pub(super) fn parse_mask_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssMaskLayer, ParseError<'i, Error>> {
    let mut image = None;
    let mut position = None;
    let mut size = None;
    let mut repeat = None;
    let mut boxes = Vec::new();
    let mut no_clip = false;
    let mut composite = None;
    let mut mode = None;

    while !input.is_exhausted() && !next_is_comma(input) {
        if image.is_none() {
            match input.try_parse(|input| parse_image_value(input, numeric)) {
                Ok(value) => {
                    image = Some(value);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        if repeat.is_none()
            && let Ok(parsed_repeat) = input.try_parse(parse_background_repeat_prefix)
        {
            repeat = Some(parsed_repeat);
            continue;
        }
        if position.is_none() {
            match input.try_parse(|input| parse_physical_position_prefix(input, numeric)) {
                Ok(parsed_position) => {
                    position = Some(parsed_position);
                    if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                        size = Some(parse_background_size_prefix(input, numeric)?);
                    }
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        if mode.is_none()
            && let Ok(value) = input.try_parse(parse_mask_mode)
        {
            mode = Some(value);
            continue;
        }
        if composite.is_none()
            && let Ok(value) = input.try_parse(parse_mask_composite)
        {
            composite = Some(value);
            continue;
        }
        if let Ok(value) = input.try_parse(parse_mask_clip) {
            match value {
                CssMaskClip::Box(value) => boxes.push(value),
                CssMaskClip::NoClip if !no_clip => no_clip = true,
                CssMaskClip::NoClip => {
                    return Err(unexpected_at(input.current_source_location()));
                }
            }
            if boxes.len() > 2 || no_clip && boxes.len() > 1 {
                return Err(unexpected_at(input.current_source_location()));
            }
            continue;
        }
        return Err(unexpected_at(input.current_source_location()));
    }
    let boxes = match (boxes.as_slice(), no_clip) {
        ([], false) => None,
        ([], true) => Some(CssMaskLayerBoxes::NoClip),
        ([value], false) => Some(CssMaskLayerBoxes::Box(*value)),
        ([origin], true) => Some(CssMaskLayerBoxes::Pair {
            origin: *origin,
            clip: CssMaskClip::NoClip,
        }),
        ([origin, clip], false) => Some(CssMaskLayerBoxes::Pair {
            origin: *origin,
            clip: CssMaskClip::Box(*clip),
        }),
        _ => {
            return Err(unexpected_at(input.current_source_location()));
        }
    };
    CssMaskLayer::try_new(image, position, size, repeat, boxes, composite, mode)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
