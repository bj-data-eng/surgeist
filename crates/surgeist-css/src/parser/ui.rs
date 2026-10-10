//! UI interaction, cursor, and outline grammar.
use super::color::parse_color;
use super::url::parse_url;
use super::values::{parse_length, parse_nonnegative_length};
use crate::cursor_values::{
    CssCursor, CssCursorImage, CssCursorImageSource, CssCursorImages, CssCursorUrlSet,
    CssCursorUrlSetDescriptor, CssCursorUrlSetOption, CssCursorUrlSetReference,
};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::syntax::*;
use crate::ui::*;
use crate::validation::unsupported_keyword_reason;
use crate::{CssCaretColor, CssComponentValueRef, CssValueTokenRef};
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

macro_rules! keyword_parser {
    ($fn:ident, $ty:ident, {$($text:literal => $variant:ident),+}) => {
        pub(super) fn $fn<'i, 't>(input: &mut Parser<'i, 't>) -> Result<$ty, ParseError<'i, Error>> {
            let location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident, $($text => Ok($ty::$variant),)+
                _ => Err(unsupported_value_at(location, None, concat!("unsupported ", stringify!($ty), " keyword"))), }
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
    let mut color = None;
    let mut animation = None;
    let mut shape = None;
    let mut autos = 0;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            autos += 1;
            continue;
        }
        if animation.is_none()
            && let Ok(value) = input.try_parse(parse_caret_animation)
        {
            animation = Some(value);
            continue;
        }
        if shape.is_none()
            && let Ok(value) = input.try_parse(parse_caret_shape)
        {
            shape = Some(value);
            continue;
        }
        if color.is_none() {
            match input.try_parse(|input| super::color::parse_color(input, numeric)) {
                Ok(value) => {
                    color = Some(CssCaretColor::Color(Box::new(value)));
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        return Err(unsupported_value_at(
            location,
            None,
            "duplicate or unsupported caret role",
        ));
    }
    // Ambiguous auto fills vacant roles in canonical order after explicit roles.
    for _ in 0..autos {
        if color.is_none() {
            color = Some(CssCaretColor::Auto);
        } else if animation.is_none() {
            animation = Some(CssCaretAnimation::Auto);
        } else if shape.is_none() {
            shape = Some(CssCaretShape::Auto);
        } else {
            return Err(unsupported_value_at(
                input.current_source_location(),
                None,
                "too many caret components",
            ));
        }
    }
    CssCaret::try_new(color, animation, shape).ok_or_else(|| {
        unsupported_value_at(
            input.current_source_location(),
            None,
            "caret requires a component",
        )
    })
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
    let component = numeric
        .collect(input)
        .map_err(|error| numeric_error(numeric, &error, location))?;
    let id = CssNavigationId::try_from_component(component)
        .map_err(|_| unsupported_value_at(location, None, "navigation requires an ID selector"))?;
    let target = if input.is_exhausted() {
        None
    } else {
        input.skip_whitespace();
        let location = input.current_source_location();
        let component = numeric
            .collect(input)
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
                if matches!(numeric.ordinary(), NumericInputContext::Components(..)) {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        "navigation target must not begin with underscore",
                    ));
                }
                CssNavigationTarget::LegacyName(CssLegacyNavigationTargetName::from_parser(
                    component,
                ))
            }
            CssComponentValueRef::Token(CssValueTokenRef::String(_)) => {
                CssNavigationTarget::Name(CssNavigationTargetName::from_parser(component))
            }
            _ => {
                return Err(unsupported_value_at(
                    location,
                    None,
                    "invalid navigation target",
                ));
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
    location: cssparser::SourceLocation,
) -> ParseError<'i, Error> {
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::invalid_component_value(location, component.clone());
    }
    let offset = match error.origin() {
        Some(crate::CssValueOrigin::Parsed(value)) => value.span().start().byte_offset().value(),
        _ => 0,
    };
    unsupported_value_at(
        numeric.error_location(error, location, offset),
        None,
        "invalid navigation component",
    )
}

pub(super) fn parse_cursor<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssCursor, ParseError<'i, Error>> {
    let mut images = Vec::new();
    while let Ok(source) = input.try_parse(|input| parse_cursor_image_source(input, numeric)) {
        let hotspot = if let Ok(x) = input.try_parse(|input| {
            super::values::parse_specified_number(input, numeric, "cursor hotspot")
        }) {
            let y = super::values::parse_specified_number(input, numeric, "cursor hotspot")?;
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
        return Err(unsupported_value(
            input,
            None,
            "cursor requires a URL or URL image set",
        ));
    }
    let options = input.parse_nested_block(|input| {
        let mut options = Vec::new();
        loop {
            let reference = if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
                CssCursorUrlSetReference::Url(url)
            } else {
                let string = input.expect_string_cloned().map_err(basic)?;
                let string = CssContentString::try_new(string.to_string())
                    .ok_or_else(|| unsupported_value(input, None, "invalid cursor URL string"))?;
                CssCursorUrlSetReference::String(string)
            };
            let mut descriptors = Vec::new();
            // Only two descriptor kinds exist. A third token is rejected by the
            // required comma/end boundary; duplicate kinds cross the same checked constructor.
            for _ in 0..2 {
                if let Ok(value) = input.try_parse(|input| {
                    super::values::parse_ordinary_resolution(
                        input,
                        numeric,
                        "cursor image resolution",
                    )
                }) {
                    descriptors.push(CssCursorUrlSetDescriptor::Resolution(value));
                } else if let Ok(value) = input.try_parse(parse_cursor_image_type) {
                    descriptors.push(CssCursorUrlSetDescriptor::Type(value));
                } else {
                    break;
                }
            }
            let option =
                CssCursorUrlSetOption::try_new(reference, descriptors).ok_or_else(|| {
                    unsupported_value(input, None, "duplicate cursor image-set descriptor")
                })?;
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
        return Err(unsupported_value(
            input,
            None,
            "cursor image-set type requires type()",
        ));
    }
    input.parse_nested_block(|input| {
        let value = input.expect_string_cloned().map_err(basic)?;
        let value = CssContentString::try_new(value.to_string())
            .ok_or_else(|| unsupported_value(input, None, "invalid cursor image type string"))?;
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("cursor", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_pointer_events<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPointerEvents, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssPointerEvents::Auto),
        "none" => Ok(CssPointerEvents::None),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("pointer-events", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("user-select", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_outline<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutline, ParseError<'i, Error>> {
    let mut width = None;
    let mut style = None;
    let mut color = None;
    let mut autos = 0;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            autos += 1;
            if autos > 2 {
                return Err(crate::error::unsupported_value_at(
                    location,
                    None,
                    "outline has too many auto components",
                ));
            }
            continue;
        }
        if width.is_none() {
            match input.try_parse(|input| parse_outline_width(input, numeric)) {
                Ok(parsed_width) => {
                    width = Some(parsed_width);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        if style.is_none()
            && let Ok(parsed_style) = input.try_parse(parse_outline_style)
        {
            style = Some(parsed_style);
            continue;
        }
        if color.is_none() {
            match input.try_parse(|input| parse_outline_color(input, numeric)) {
                Ok(parsed_color) => {
                    color = Some(parsed_color);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported outline component",
        ));
    }
    // Resolve auto only after the unordered explicit slots are known. One lone
    // auto (with optional width) sets both semantic slots under selected UI4 §3.1.
    match autos {
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
            return Err(unsupported_value(
                input,
                None,
                "outline auto duplicates an explicit component",
            ));
        }
    }
    if width.is_none() && style.is_none() && color.is_none() {
        None
    } else {
        Some(CssOutline::try_new(width, style, color).expect("nonempty parsed outline"))
    }
    .ok_or_else(|| unsupported_value(input, None, "outline shorthand is empty"))
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
        _ => Err(crate::error::unsupported_value_at(location, None, unsupported_keyword_reason("outline-style", ident.as_ref()))),
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
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "thin" => Ok(CssOutlineWidth::Thin),
            "medium" => Ok(CssOutlineWidth::Medium),
            "thick" => Ok(CssOutlineWidth::Thick),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("outline-width", ident.as_ref()),
            )),
        };
    }
    parse_nonnegative_length(input, numeric, "outline-width").map(CssOutlineWidth::Length)
}

pub(super) fn parse_outline_offset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLength, ParseError<'i, Error>> {
    parse_length(input, numeric, "outline-offset")
}
