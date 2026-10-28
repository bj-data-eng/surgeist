//! Shapes 1 property composition over shared authored image and shape owners.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use crate::values::shape::{CssShapeBox, CssShapeOutside, CssShapeOutsideShape};
use surgeist_css_parser::Token;

pub(crate) fn parse_shape_outside<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssShapeOutside, SyntaxError<'i, Error>> {
    // `none` is the property branch, never an image-only value.
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssShapeOutside::None);
    }
    let mut reference_box = input.attempt(parse_shape_box).ok();
    if input.only_whitespace_remains() {
        return reference_box
            .map(CssShapeOutside::ShapeBox)
            .ok_or_else(|| unexpected_at(input.site()));
    }
    let state = input.mark();
    let basic_shape = matches!(input.consume_after_whitespace(), Ok(Token::Function(name))
        if super::effects::is_basic_shape_function(name));
    input.restore(state);
    if reference_box.is_none() && !basic_shape {
        // Select the image owner directly: an image's original syntax/resource
        // error must not be discarded by speculative shape backtracking.
        return super::background::parse_image(input, numeric).map(CssShapeOutside::Image);
    }
    let shape = super::effects::parse_clip_path_shape(input, numeric)?;
    if reference_box.is_none() && !input.only_whitespace_remains() {
        reference_box = Some(parse_shape_box(input)?);
    }
    input.require_empty().map_err(basic)?;
    Ok(CssShapeOutside::BasicShape(CssShapeOutsideShape::new(
        shape,
        reference_box,
    )))
}

fn parse_shape_box<'i>(input: &mut Stream<'i>) -> Result<CssShapeBox, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    CssShapeBox::from_keyword(ident).ok_or_else(|| unexpected_at(location))
}
