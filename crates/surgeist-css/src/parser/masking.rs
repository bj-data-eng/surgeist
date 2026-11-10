//! Masking-specific orchestration around shared image and four-side owners.
use super::background::{
    next_starts_background_image, next_starts_border_image_repeat, next_starts_border_image_slice,
    parse_background_repeat, parse_background_size, parse_border_image_outset,
    parse_border_image_repeat, parse_border_image_slice_component, parse_border_image_width,
    parse_image_value,
};
use super::combinators::{Combinator, Tail, attempt, commit, fill, parse_group};
use super::position::parse_position_prefix;
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use crate::values::border::CssBorderImageSlice;
use crate::values::effects::{
    CssClipRule, CssMaskBorder, CssMaskBoxList, CssMaskClip, CssMaskClipList, CssMaskComposite,
    CssMaskCompositeList, CssMaskLayer, CssMaskLayerBoxes, CssMaskList, CssMaskMode,
    CssMaskModeList, CssMaskType,
};
use crate::values::transform::CssCoordBox;

macro_rules! keyword_parser {
    ($function:ident, $ty:ident, $($keyword:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $function<'i>(input: &mut Stream<'i>) -> Result<$ty, SyntaxError<'i, Error>> {
            let keyword_location = input.site();
            let ident = input.consume_ident().map_err(basic)?;
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

pub(super) fn parse_mask_box<'i>(
    input: &mut Stream<'i>,
) -> Result<CssCoordBox, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    CssCoordBox::from_keyword(ident).ok_or_else(|| unexpected_at(keyword_location))
}
pub(super) fn parse_mask_clip<'i>(
    input: &mut Stream<'i>,
) -> Result<CssMaskClip, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("no-clip"))
        .is_ok()
    {
        return Ok(CssMaskClip::NoClip);
    }
    parse_mask_box(input).map(CssMaskClip::Box)
}

macro_rules! list_parser {
    ($function:ident, $list:ident, $parse:ident) => {
        pub(super) fn $function<'i>(
            input: &mut Stream<'i>,
        ) -> Result<$list, SyntaxError<'i, Error>> {
            let values = input.comma_segments($parse)?;
            $list::try_new(values).ok_or_else(|| unexpected_at(input.site()))
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
pub(crate) fn parse_mask_border_slice<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssBorderImageSlice, SyntaxError<'i, Error>> {
    let mut values = vec![parse_border_image_slice_component(input, numeric)?];
    while values.len() < 4 {
        match input.attempt(|input| parse_border_image_slice_component(input, numeric)) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }
    let fill = input.attempt(|input| input.consume_keyword("fill")).is_ok();
    Ok(CssBorderImageSlice::try_new(values, fill).expect("one to four checked components"))
}

pub(crate) fn parse_mask_border<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssMaskBorder, SyntaxError<'i, Error>> {
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
                if input.attempt(|input| input.consume_delim('/')).is_ok() {
                    // The first width is optional, including the source's bare 10 /.
                    if !input.only_whitespace_remains() {
                        width = attempt(input, |input| parse_border_image_width(input, numeric))?;
                    }
                    if input.attempt(|input| input.consume_delim('/')).is_ok() {
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
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_mask_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssMaskList, SyntaxError<'i, Error>> {
    let layers = input.comma_segments(|input| parse_mask_layer(input, numeric))?;
    if layers.is_empty() {
        Err(unexpected_at(input.site()))
    } else {
        Ok(CssMaskList::new(layers))
    }
}

pub(super) fn parse_mask_layer<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssMaskLayer, SyntaxError<'i, Error>> {
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
                    parse_position_prefix(input, numeric)
                })? {
                    return Ok(false);
                }
                if input.attempt(|input| input.consume_delim('/')).is_ok() {
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
        .ok_or_else(|| unexpected_at(input.site()))
}
