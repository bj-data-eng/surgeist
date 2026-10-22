//! Masking-specific orchestration around shared image and four-side owners.
use super::background::{
    next_starts_background_image, next_starts_border_image_repeat, next_starts_border_image_slice,
    parse_background_repeat, parse_background_size, parse_border_image_outset,
    parse_border_image_repeat, parse_border_image_slice_component, parse_border_image_width,
    parse_image_value,
};
use super::combinators::{Combinator, Tail, attempt, commit, fill, parse_group};
use super::position::parse_physical_position;
use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use crate::syntax::*;

macro_rules! keyword_parser {
    ($function:ident, $ty:ident, $($keyword:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $function<'i, 't>(input: &mut Parser<'i, 't>) -> Result<$ty, ParseError<'i, Error>> {
            let keyword_location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match ident.to_ascii_lowercase().as_str() {
                $($keyword => Ok($ty::$variant),)+
                _ => Err(unexpected_at(keyword_location)),
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
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssBoxEdgeKeyword::from_keyword(&ident)
        .and_then(CssMaskBox::try_new)
        .ok_or_else(|| unexpected_at(keyword_location))
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
    // `<'mask-border-source'> || <'mask-border-slice'> [ /
    // <'mask-border-width'>? [ / <'mask-border-outset'> ]? ]? ||
    // <'mask-border-repeat'> || <'mask-border-mode'>`.
    let (mut source, mut slice, mut width, mut outset, mut repeat, mut mode) =
        (None, None, None, None, None, None);
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => commit(&mut source, input, next_starts_background_image, |input| {
                parse_image_value(input, numeric)
            }),
            1 => {
                if !commit(&mut slice, input, next_starts_border_image_slice, |input| {
                    parse_mask_border_slice(input, numeric)
                })? {
                    return Ok(false);
                }
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    // The first width is optional, including the source's bare 10 /.
                    if !input.is_exhausted() {
                        width = attempt(input, |input| parse_border_image_width(input, numeric))?;
                    }
                    if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                        outset = Some(parse_border_image_outset(input, numeric)?);
                    }
                }
                Ok(true)
            }
            2 => commit(
                &mut repeat,
                input,
                next_starts_border_image_repeat,
                parse_border_image_repeat,
            ),
            3 => fill(&mut mode, input, parse_mask_type),
            _ => unreachable!("four mask-border operands"),
        },
    )?;
    CssMaskBorder::try_new(source, slice, width, outset, repeat, mode)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_mask_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssMaskList, ParseError<'i, Error>> {
    let layers = input.parse_comma_separated(|input| parse_mask_layer(input, numeric))?;
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
    // `<mask-reference> || <position> [ / <bg-size> ]? || <repeat-style> ||
    // <geometry-box> || [ <geometry-box> | no-clip ] || <compositing-operator>
    // || <masking-mode>`.
    let (mut image, mut position, mut size, mut repeat) = (None, None, None, None);
    let (mut origin, mut clip, mut composite, mut mode) = (None, None, None, None);
    parse_group::<7>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            // A URL commits the image operand (Values 4 §2.2), so a request
            // URL modifier failure is reported inside it.
            0 if super::ui::starts_url(input) => commit(
                &mut image,
                input,
                |_| true,
                |input| parse_image_value(input, numeric),
            ),
            0 => fill(&mut image, input, |input| parse_image_value(input, numeric)),
            1 => fill(&mut repeat, input, parse_background_repeat),
            2 => {
                if !fill(&mut position, input, |input| {
                    parse_physical_position(input, numeric)
                })? {
                    return Ok(false);
                }
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    size = Some(parse_background_size(input, numeric)?);
                }
                Ok(true)
            }
            3 => fill(&mut mode, input, parse_mask_mode),
            4 => fill(&mut composite, input, parse_mask_composite),
            5 => fill(&mut origin, input, parse_mask_box),
            6 => fill(&mut clip, input, parse_mask_clip),
            _ => unreachable!("seven mask layer operands"),
        },
    )?;
    let boxes = match (origin, clip) {
        (None, None) => None,
        (None, Some(CssMaskClip::NoClip)) => Some(CssMaskLayerBoxes::NoClip),
        (Some(value), None) | (None, Some(CssMaskClip::Box(value))) => {
            Some(CssMaskLayerBoxes::Box(value))
        }
        (Some(origin), Some(clip)) => Some(CssMaskLayerBoxes::Pair { origin, clip }),
    };
    CssMaskLayer::try_new(image, position, size, repeat, boxes, composite, mode)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
