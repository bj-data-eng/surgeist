//! Masking-specific orchestration around shared image and four-side owners.
use super::background::{
    next_starts_background_image, next_starts_border_image_repeat, next_starts_border_image_slice,
    parse_border_image_outset_prefix, parse_border_image_repeat_prefix,
    parse_border_image_slice_component, parse_border_image_width_prefix, parse_image_value,
};
use crate::error::{Error, basic, unsupported_value};
use crate::syntax::*;
use cssparser::{ParseError, Parser};

macro_rules! keyword_parser {
    ($function:ident, $ty:ident, $($keyword:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $function<'i, 't>(input: &mut Parser<'i, 't>) -> Result<$ty, ParseError<'i, Error>> {
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match ident.to_ascii_lowercase().as_str() {
                $($keyword => Ok($ty::$variant),)+
                _ => Err(unsupported_value(input, None, concat!("invalid ", stringify!($ty), " keyword"))),
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
        .ok_or_else(|| unsupported_value(input, None, "invalid mask geometry box"))
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
            $list::try_new(values).ok_or_else(|| unsupported_value(input, None, "empty mask list"))
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
        return Err(unsupported_value(
            input,
            None,
            "unsupported or duplicate mask-border component",
        ));
    }
    CssMaskBorder::try_new(source, slice, width, outset, repeat, mode)
        .ok_or_else(|| unsupported_value(input, None, "empty mask-border shorthand"))
}
