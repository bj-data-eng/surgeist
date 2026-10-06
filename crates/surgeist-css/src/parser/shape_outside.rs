//! Shapes 1 property composition over shared authored image and shape owners.

use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::{CssShapeBox, CssShapeOutside, CssShapeOutsideShape};
use cssparser::{ParseError, Parser, Token};

pub(super) fn parse_shape_outside<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssShapeOutside, ParseError<'i, Error>> {
    // `none` is the property branch, never an image-only value.
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssShapeOutside::None);
    }
    let mut reference_box = input.try_parse(parse_shape_box).ok();
    if input.is_exhausted() {
        return reference_box
            .map(CssShapeOutside::ShapeBox)
            .ok_or_else(|| unsupported_value(input, None, "missing shape-outside shape or box"));
    }
    let state = input.state();
    let basic_shape = matches!(input.next(), Ok(Token::Function(name))
        if super::effects::is_basic_shape_function(name));
    input.reset(&state);
    if reference_box.is_none() && !basic_shape {
        // Select the image owner directly: an image's original syntax/resource
        // error must not be discarded by speculative shape backtracking.
        return super::background::parse_image(input, numeric).map(CssShapeOutside::Image);
    }
    let shape = super::effects::parse_clip_path_shape(input, numeric)?;
    if reference_box.is_none() && !input.is_exhausted() {
        reference_box = Some(parse_shape_box(input)?);
    }
    input.expect_exhausted().map_err(basic)?;
    Ok(CssShapeOutside::BasicShape(CssShapeOutsideShape::new(
        shape,
        reference_box,
    )))
}

fn parse_shape_box<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssShapeBox, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssShapeBox::from_keyword(ident.as_ref()).ok_or_else(|| {
        unsupported_value_at(location, None, format!("unsupported shape box `{ident}`"))
    })
}
