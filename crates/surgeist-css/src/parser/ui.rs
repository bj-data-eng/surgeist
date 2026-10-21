//! UI interaction, cursor, and outline grammar.
use super::color::parse_color;
use super::combinators::{Combinator, Tail, fill, keyword, parse_group};
use super::property_input::Parser;
use super::url::parse_url;
use super::values::{next_is_ident, parse_length};
use crate::cursor_values::{
    CssCursor, CssCursorImage, CssCursorImageSource, CssCursorImages, CssCursorUrlSet,
    CssCursorUrlSetDescriptor, CssCursorUrlSetOption, CssCursorUrlSetReference,
};
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use crate::numeric::NumericInputContext;
use crate::syntax::*;
use crate::ui::*;
use crate::{CssCaretColor, CssComponentValueRef, CssValueTokenRef};
use cssparser::{Token, match_ignore_ascii_case};

macro_rules! keyword_parser {
    ($fn:ident, $ty:ident, {$($text:literal => $variant:ident),+}) => {
        pub(super) fn $fn<'i, 't>(input: &mut Parser<'i, 't>) -> Result<$ty, ParseError<'i, Error>> {
            let location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident, $($text => Ok($ty::$variant),)+
                _ => Err(unexpected_at(location)), }
        }
    };
}
keyword_parser!(parse_caret_animation, CssCaretAnimation, { "auto" => Auto, "manual" => Manual });
keyword_parser!(parse_caret_shape, CssCaretShape, { "auto" => Auto, "bar" => Bar, "block" => Block, "underscore" => Underscore });
keyword_parser!(parse_interactivity, CssInteractivity, { "auto" => Auto, "inert" => Inert });
keyword_parser!(parse_appearance, CssAppearance, { "none" => None, "auto" => Auto, "base" => Base, "base-select" => BaseSelect,
    "searchfield" => Searchfield, "textarea" => Textarea, "checkbox" => Checkbox, "radio" => Radio,
    "menulist" => Menulist, "listbox" => Listbox, "meter" => Meter, "progress-bar" => ProgressBar,
    "button" => Button, "textfield" => Textfield, "menulist-button" => MenulistButton });
pub(super) fn parse_accent_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssAccentColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssAccentColor::Auto);
    }
    super::color::parse_color(input, numeric).map(|color| CssAccentColor::Color(Box::new(color)))
}
pub(super) fn parse_caret<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssCaret, ParseError<'i, Error>> {
    // `<'caret-color'> || <'caret-animation'> || <'caret-shape'>`. Each member
    // admits `auto`, so up to three `auto` operands are collected as ambiguous.
    let (mut color, mut animation, mut shape) = (None, None, None);
    let mut autos = [None; 3];
    let matched = parse_group::<6>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0..=2 => keyword(&mut autos[operand], input, &[("auto", ())]),
            _ if next_is_ident(input, "auto") => Ok(false),
            3 => fill(&mut animation, input, parse_caret_animation),
            4 => fill(&mut shape, input, parse_caret_shape),
            5 => fill(&mut color, input, |input| {
                parse_color(input, numeric).map(|value| CssCaretColor::Color(Box::new(value)))
            }),
            _ => unreachable!("six caret operands"),
        },
    )?;
    // Ambiguous auto fills vacant roles in canonical order after explicit roles.
    for _ in matched[..3].iter().filter(|matched| **matched) {
        if color.is_none() {
            color = Some(CssCaretColor::Auto);
        } else if animation.is_none() {
            animation = Some(CssCaretAnimation::Auto);
        } else if shape.is_none() {
            shape = Some(CssCaretShape::Auto);
        } else {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssCaret::try_new(color, animation, shape)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_interest_delay_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInterestDelayValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssInterestDelayValue::Normal);
    }
    super::timing::parse_delay(input, numeric).map(CssInterestDelayValue::Time)
}
pub(super) fn parse_interest_delay<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInterestDelay, ParseError<'i, Error>> {
    let start = parse_interest_delay_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_interest_delay_value(input, numeric)?)
    };
    Ok(CssInterestDelay::from_parser(start, end))
}
pub(super) fn parse_navigation<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNavigation, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssNavigation::Auto);
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| numeric_error(numeric, &error, location))?;
    let id = CssNavigationId::try_from_component(component).map_err(|_| unexpected_at(location))?;
    let target = if input.is_exhausted() {
        None
    } else {
        input.skip_whitespace();
        let location = input.current_source_location();
        let component = input
            .collect_numeric_component(numeric)
            .map_err(|error| numeric_error(numeric, &error, location))?;
        Some(match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(value))
                if value.eq_ignore_ascii_case("current") =>
            {
                CssNavigationTarget::Current
            }
            CssComponentValueRef::Token(CssValueTokenRef::Ident(value))
                if value.eq_ignore_ascii_case("root") =>
            {
                CssNavigationTarget::Root
            }
            CssComponentValueRef::Token(CssValueTokenRef::String(value))
                if value.starts_with('_') =>
            {
                if !numeric.is_source_input() {
                    return Err(unexpected_at(location));
                }
                CssNavigationTarget::LegacyName(CssLegacyNavigationTargetName::from_parser(
                    component,
                ))
            }
            CssComponentValueRef::Token(CssValueTokenRef::String(_)) => {
                CssNavigationTarget::Name(CssNavigationTargetName::from_parser(component))
            }
            _ => {
                return Err(unexpected_at(location));
            }
        })
    };
    Ok(CssNavigation::Id(Box::new(
        CssNavigationReference::from_parser(id, target),
    )))
}
fn numeric_error<'i>(
    numeric: &NumericInputContext<'_>,
    error: &crate::CssNumericConstructionError,
    location: super::property_input::Site<'i>,
) -> ParseError<'i, Error> {
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::property::invalid_component_value(location, component.clone());
    }
    unexpected_at(numeric.property_error_location(error, location, location))
}

pub(super) fn parse_cursor<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssCursor, ParseError<'i, Error>> {
    let mut images = Vec::new();
    while let Ok(source) = input.try_parse(|input| parse_cursor_image_source(input, numeric)) {
        let hotspot = if let Ok(x) =
            input.try_parse(|input| super::values::parse_specified_number(input, numeric))
        {
            let y = super::values::parse_specified_number(input, numeric)?;
            Some([x, y])
        } else {
            None
        };
        images.push(CssCursorImage::new(source, hotspot));
        input.expect_comma().map_err(basic)?;
    }
    let fallback = parse_cursor_keyword(input)?;
    if images.is_empty() {
        Ok(CssCursor::Keyword(fallback))
    } else {
        let images = CssCursorImages::try_new(images, fallback)
            .expect("at least one cursor image was parsed");
        Ok(CssCursor::Images(images))
    }
}

fn parse_cursor_image_source<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCursorImageSource, ParseError<'i, Error>> {
    if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
        return Ok(CssCursorImageSource::Url(url));
    }
    let name = input.expect_function().map_err(basic)?.clone();
    if !name.eq_ignore_ascii_case("image-set") && !name.eq_ignore_ascii_case("-webkit-image-set") {
        return Err(unexpected_at(input.current_source_location()));
    }
    let options = input.parse_nested_block(|input| {
        let mut options = Vec::new();
        loop {
            let reference = if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
                CssCursorUrlSetReference::Url(url)
            } else {
                let string = input.expect_string_cloned().map_err(basic)?;
                let string = CssContentString::try_new(string.to_string())
                    .ok_or_else(|| unexpected_at(input.current_source_location()))?;
                CssCursorUrlSetReference::String(string)
            };
            let mut descriptors = Vec::new();
            // Only two descriptor kinds exist. A third token is rejected by the
            // required comma/end boundary; duplicate kinds cross the same checked constructor.
            for _ in 0..2 {
                if let Ok(value) = input
                    .try_parse(|input| super::values::parse_ordinary_resolution(input, numeric))
                {
                    descriptors.push(CssCursorUrlSetDescriptor::Resolution(value));
                } else if let Ok(value) = input.try_parse(parse_cursor_image_type) {
                    descriptors.push(CssCursorUrlSetDescriptor::Type(value));
                } else {
                    break;
                }
            }
            let option = CssCursorUrlSetOption::try_new(reference, descriptors)
                .ok_or_else(|| unexpected_at(input.current_source_location()))?;
            options.push(option);
            if input.is_exhausted() {
                break;
            }
            input.expect_comma().map_err(basic)?;
        }
        Ok(options)
    })?;
    let set =
        CssCursorUrlSet::try_new(options).expect("at least one cursor image-set option was parsed");
    Ok(CssCursorImageSource::UrlSet(set))
}

fn parse_cursor_image_type<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssContentString, ParseError<'i, Error>> {
    let name = input.expect_function().map_err(basic)?.clone();
    if !name.eq_ignore_ascii_case("type") {
        return Err(unexpected_at(input.current_source_location()));
    }
    input.parse_nested_block(|input| {
        let value = input.expect_string_cloned().map_err(basic)?;
        let value = CssContentString::try_new(value.to_string())
            .ok_or_else(|| unexpected_at(input.current_source_location()))?;
        input.expect_exhausted().map_err(basic)?;
        Ok(value)
    })
}

pub(super) fn parse_cursor_keyword<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCursorKeyword, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssCursorKeyword::Auto),
        "default" => Ok(CssCursorKeyword::Default),
        "none" => Ok(CssCursorKeyword::None),
        "context-menu" => Ok(CssCursorKeyword::ContextMenu),
        "help" => Ok(CssCursorKeyword::Help),
        "pointer" => Ok(CssCursorKeyword::Pointer),
        "progress" => Ok(CssCursorKeyword::Progress),
        "wait" => Ok(CssCursorKeyword::Wait),
        "cell" => Ok(CssCursorKeyword::Cell),
        "crosshair" => Ok(CssCursorKeyword::Crosshair),
        "text" => Ok(CssCursorKeyword::Text),
        "vertical-text" => Ok(CssCursorKeyword::VerticalText),
        "alias" => Ok(CssCursorKeyword::Alias),
        "copy" => Ok(CssCursorKeyword::Copy),
        "move" => Ok(CssCursorKeyword::Move),
        "no-drop" => Ok(CssCursorKeyword::NoDrop),
        "not-allowed" => Ok(CssCursorKeyword::NotAllowed),
        "grab" => Ok(CssCursorKeyword::Grab),
        "grabbing" => Ok(CssCursorKeyword::Grabbing),
        "all-scroll" => Ok(CssCursorKeyword::AllScroll),
        "col-resize" => Ok(CssCursorKeyword::ColResize),
        "row-resize" => Ok(CssCursorKeyword::RowResize),
        "n-resize" => Ok(CssCursorKeyword::NResize),
        "e-resize" => Ok(CssCursorKeyword::EResize),
        "s-resize" => Ok(CssCursorKeyword::SResize),
        "w-resize" => Ok(CssCursorKeyword::WResize),
        "ne-resize" => Ok(CssCursorKeyword::NeResize),
        "nw-resize" => Ok(CssCursorKeyword::NwResize),
        "se-resize" => Ok(CssCursorKeyword::SeResize),
        "sw-resize" => Ok(CssCursorKeyword::SwResize),
        "ew-resize" => Ok(CssCursorKeyword::EwResize),
        "ns-resize" => Ok(CssCursorKeyword::NsResize),
        "nesw-resize" => Ok(CssCursorKeyword::NeswResize),
        "nwse-resize" => Ok(CssCursorKeyword::NwseResize),
        "zoom-in" => Ok(CssCursorKeyword::ZoomIn),
        "zoom-out" => Ok(CssCursorKeyword::ZoomOut),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_pointer_events<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPointerEvents, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssPointerEvents::Auto),
        "none" => Ok(CssPointerEvents::None),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_user_select<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUserSelect, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssUserSelect::Auto),
        "text" => Ok(CssUserSelect::Text),
        "none" => Ok(CssUserSelect::None),
        "all" => Ok(CssUserSelect::All),
        "contain" => Ok(CssUserSelect::Contain),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_outline<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutline, ParseError<'i, Error>> {
    // `<'outline-width'> || <'outline-style'> || <'outline-color'>`. Style
    // and color both admit `auto`, so up to two `auto` operands are collected
    // as ambiguous and assigned after the explicit roles are known.
    let (mut width, mut style, mut color) = (None, None, None);
    let mut autos = [None; 2];
    let [first_auto, second_auto, ..] = parse_group::<5>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 | 1 => keyword(&mut autos[operand], input, &[("auto", ())]),
            2 => fill(&mut width, input, |input| {
                parse_outline_width(input, numeric)
            }),
            _ if next_is_ident(input, "auto") => Ok(false),
            3 => fill(&mut style, input, parse_outline_style),
            4 => fill(&mut color, input, |input| {
                parse_outline_color(input, numeric)
            }),
            _ => unreachable!("five outline operands"),
        },
    )?;
    // Resolve auto only after the unordered explicit slots are known. One lone
    // auto (with optional width) sets both semantic slots under selected UI4 §3.1.
    match usize::from(first_auto) + usize::from(second_auto) {
        0 => {}
        1 if style.is_none() || color.is_none() => {
            if style.is_none() {
                style = Some(CssOutlineStyle::Auto);
            }
            if color.is_none() {
                color = Some(CssOutlineColor::Auto);
            }
        }
        2 if style.is_none() && color.is_none() => {
            style = Some(CssOutlineStyle::Auto);
            color = Some(CssOutlineColor::Auto);
        }
        _ => {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    if width.is_none() && style.is_none() && color.is_none() {
        None
    } else {
        Some(CssOutline::try_new(width, style, color).expect("nonempty parsed outline"))
    }
    .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_outline_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOutlineStyle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssOutlineStyle::Auto),
        "none" => Ok(CssOutlineStyle::None),
        "dotted" => Ok(CssOutlineStyle::Dotted),
        "dashed" => Ok(CssOutlineStyle::Dashed),
        "solid" => Ok(CssOutlineStyle::Solid),
        "double" => Ok(CssOutlineStyle::Double),
        "groove" => Ok(CssOutlineStyle::Groove),
        "ridge" => Ok(CssOutlineStyle::Ridge),
        "inset" => Ok(CssOutlineStyle::Inset),
        "outset" => Ok(CssOutlineStyle::Outset),
        _ => Err(crate::error::property::unexpected_at(location)),
    }
}

pub(super) fn parse_outline_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutlineColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssOutlineColor::Auto);
    }
    let start = input.state();
    let stripes =
        matches!(input.next(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case("stripes"));
    input.reset(&start);
    if stripes {
        return super::image_1d::parse_image_1d(input, numeric)
            .map(|image| CssOutlineColor::Image1D(Box::new(image)));
    }
    parse_color(input, numeric).map(|color| CssOutlineColor::Color(Box::new(color)))
}

pub(super) fn parse_outline_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutlineWidth, ParseError<'i, Error>> {
    // UI 4 `outline-width: <line-width>` reuses the one `<line-width>` grammar.
    super::border_width::parse_exact_line_width(input, numeric).map(|width| match width {
        CssLineWidth::Thin => CssOutlineWidth::Thin,
        CssLineWidth::Medium => CssOutlineWidth::Medium,
        CssLineWidth::Thick => CssOutlineWidth::Thick,
        CssLineWidth::Length(length) => CssOutlineWidth::Length(length),
    })
}

pub(super) fn parse_outline_offset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLength, ParseError<'i, Error>> {
    parse_length(input, numeric)
}
