//! Authored shape command parsing over shared exact scalar and position owners.
use super::position::parse_full_position_bounded;
use super::values::{
    AngleParserContext, next_is_ident, parse_angle_value, parse_length_percentage,
};
use crate::error::{Error, basic, unsupported_value};
use crate::numeric::NumericInputContext;
use crate::syntax::*;
use cssparser::{ParseError, Parser};

type Result<'i, T> = std::result::Result<T, ParseError<'i, Error>>;
const COMMANDS: &[&str] = &[
    "move", "line", "hline", "vline", "curve", "smooth", "arc", "close",
];
const ENDPOINT: &[&str] = &[
    "with", "of", "cw", "ccw", "large", "small", "rotate", "move", "line", "hline", "vline",
    "curve", "smooth", "arc", "close",
];
const CONTROL: &[&str] = &[
    "from", "move", "line", "hline", "vline", "curve", "smooth", "arc", "close",
];

pub(super) fn parse_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeFunction> {
    let fill = if input
        .try_parse(|p| p.expect_ident_matching("nonzero"))
        .is_ok()
    {
        Some(CssFillRule::Nonzero)
    } else if input
        .try_parse(|p| p.expect_ident_matching("evenodd"))
        .is_ok()
    {
        Some(CssFillRule::Evenodd)
    } else {
        None
    };
    input.expect_ident_matching("from").map_err(basic)?;
    let start = parse_full_position_bounded(input, numeric, COMMANDS)?;
    // The individually adopted pinned-source editorial disposition requires this comma.
    input.expect_comma().map_err(basic)?;
    let mut commands = vec![parse_command(input, numeric)?];
    while !input.is_exhausted() {
        input.expect_comma().map_err(basic)?;
        commands.push(parse_command(input, numeric)?);
    }
    Ok(CssShapeFunction::new(
        fill,
        start,
        CssShapeCommandList::try_new(commands).expect("first command is required"),
    ))
}
fn pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeCoordinatePair> {
    Ok(CssShapeCoordinatePair::new(
        parse_length_percentage(input, numeric, "shape coordinate")?,
        parse_length_percentage(input, numeric, "shape coordinate")?,
    ))
}
fn affinity<'i, 't>(input: &mut Parser<'i, 't>) -> Result<'i, bool> {
    let location = input.current_source_location();
    let word = input.expect_ident_cloned().map_err(basic)?;
    if word.eq_ignore_ascii_case("to") {
        Ok(true)
    } else if word.eq_ignore_ascii_case("by") {
        Ok(false)
    } else {
        Err(crate::error::unsupported_value_at(
            location,
            None,
            "expected shape to or by",
        ))
    }
}
fn endpoint<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeEndpoint> {
    if affinity(input)? {
        parse_full_position_bounded(input, numeric, ENDPOINT).map(CssShapeEndpoint::To)
    } else {
        pair(input, numeric).map(CssShapeEndpoint::By)
    }
}
fn anchor<'i, 't>(input: &mut Parser<'i, 't>) -> Result<'i, CssShapeControlAnchor> {
    let location = input.current_source_location();
    let word = input.expect_ident_cloned().map_err(basic)?;
    match word.to_ascii_lowercase().as_str() {
        "start" => Ok(CssShapeControlAnchor::Start),
        "end" => Ok(CssShapeControlAnchor::End),
        "origin" => Ok(CssShapeControlAnchor::Origin),
        _ => Err(crate::error::unsupported_value_at(
            location,
            None,
            "invalid shape control anchor",
        )),
    }
}
fn relative_control<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeRelativeControlPoint> {
    let offset = pair(input, numeric)?;
    let anchor = if next_is_ident(input, "from") {
        input.expect_ident_matching("from").map_err(basic)?;
        Some(anchor(input)?)
    } else {
        None
    };
    Ok(CssShapeRelativeControlPoint::new(offset, anchor))
}
fn absolute_control<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeAbsoluteControlPoint> {
    // Explicit anchors admit exactly a coordinate pair, never arbitrary position families.
    if let Ok(offset) = input.try_parse(|p| {
        let offset = pair(p, numeric)?;
        p.expect_ident_matching("from").map_err(basic)?;
        Ok::<_, ParseError<'i, Error>>(offset)
    }) {
        return Ok(CssShapeAbsoluteControlPoint::from_coordinates(
            offset,
            Some(anchor(input)?),
        ));
    }
    parse_full_position_bounded(input, numeric, CONTROL)
        .map(CssShapeAbsoluteControlPoint::from_position)
}
fn parse_command<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeCommand> {
    let location = input.current_source_location();
    let command = input
        .expect_ident_cloned()
        .map_err(basic)?
        .to_ascii_lowercase();
    match command.as_str() {
        "close" => Ok(CssShapeCommand::Close),
        "move" => endpoint(input, numeric).map(CssShapeCommand::Move),
        "line" => endpoint(input, numeric).map(CssShapeCommand::Line),
        "hline" => {
            let to = affinity(input)?;
            if to {
                let keyword = input.try_parse(|p| {
                    let word = p.expect_ident_cloned().map_err(basic)?;
                    match word.to_ascii_lowercase().as_str() {
                        "left" => Ok(CssHorizontalPositionKeyword::Left),
                        "center" => Ok(CssHorizontalPositionKeyword::Center),
                        "right" => Ok(CssHorizontalPositionKeyword::Right),
                        "x-start" => Ok(CssHorizontalPositionKeyword::XStart),
                        "x-end" => Ok(CssHorizontalPositionKeyword::XEnd),
                        _ => Err(unsupported_value(
                            p,
                            None,
                            "invalid horizontal line keyword",
                        )),
                    }
                });
                if let Ok(keyword) = keyword {
                    return Ok(CssShapeCommand::HorizontalLine(
                        CssShapeHorizontalLine::ToKeyword(keyword),
                    ));
                }
            }
            let value = parse_length_percentage(input, numeric, "horizontal line")?;
            Ok(CssShapeCommand::HorizontalLine(if to {
                CssShapeHorizontalLine::ToOffset(value)
            } else {
                CssShapeHorizontalLine::By(value)
            }))
        }
        "vline" => {
            let to = affinity(input)?;
            if to {
                let keyword = input.try_parse(|p| {
                    let word = p.expect_ident_cloned().map_err(basic)?;
                    match word.to_ascii_lowercase().as_str() {
                        "top" => Ok(CssVerticalPositionKeyword::Top),
                        "center" => Ok(CssVerticalPositionKeyword::Center),
                        "bottom" => Ok(CssVerticalPositionKeyword::Bottom),
                        "y-start" => Ok(CssVerticalPositionKeyword::YStart),
                        "y-end" => Ok(CssVerticalPositionKeyword::YEnd),
                        _ => Err(unsupported_value(p, None, "invalid vertical line keyword")),
                    }
                });
                if let Ok(keyword) = keyword {
                    return Ok(CssShapeCommand::VerticalLine(
                        CssShapeVerticalLine::ToKeyword(keyword),
                    ));
                }
            }
            let value = parse_length_percentage(input, numeric, "vertical line")?;
            Ok(CssShapeCommand::VerticalLine(if to {
                CssShapeVerticalLine::ToOffset(value)
            } else {
                CssShapeVerticalLine::By(value)
            }))
        }
        "curve" => {
            let end = endpoint(input, numeric)?;
            input.expect_ident_matching("with").map_err(basic)?;
            Ok(CssShapeCommand::Curve(match end {
                CssShapeEndpoint::To(end) => {
                    let first = absolute_control(input, numeric)?;
                    let second = if input.try_parse(|p| p.expect_delim('/')).is_ok() {
                        Some(absolute_control(input, numeric)?)
                    } else {
                        None
                    };
                    CssShapeCurve::to(end, first, second)
                }
                CssShapeEndpoint::By(end) => {
                    let first = relative_control(input, numeric)?;
                    let second = if input.try_parse(|p| p.expect_delim('/')).is_ok() {
                        Some(relative_control(input, numeric)?)
                    } else {
                        None
                    };
                    CssShapeCurve::by(end, first, second)
                }
            }))
        }
        "smooth" => {
            let end = endpoint(input, numeric)?;
            let with = input.try_parse(|p| p.expect_ident_matching("with")).is_ok();
            Ok(CssShapeCommand::Smooth(match end {
                CssShapeEndpoint::To(end) => CssShapeSmooth::to(
                    end,
                    if with {
                        Some(absolute_control(input, numeric)?)
                    } else {
                        None
                    },
                ),
                CssShapeEndpoint::By(end) => CssShapeSmooth::by(
                    end,
                    if with {
                        Some(relative_control(input, numeric)?)
                    } else {
                        None
                    },
                ),
            }))
        }
        "arc" => parse_arc(input, numeric).map(CssShapeCommand::Arc),
        _ => Err(crate::error::unsupported_value_at(
            location,
            None,
            "invalid shape command",
        )),
    }
}
fn parse_arc<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeArc> {
    let end = endpoint(input, numeric)?;
    let (mut radii, mut sweep, mut size, mut rotation) = (None, None, None, None);
    while !input.is_exhausted() && !super::values::next_is_comma(input) {
        let location = input.current_source_location();
        let option = input
            .expect_ident_cloned()
            .map_err(basic)?
            .to_ascii_lowercase();
        match option.as_str() {
            "of" if radii.is_none() => {
                let horizontal = parse_length_percentage(input, numeric, "arc radius")?;
                radii = Some(
                    if let Ok(vertical) =
                        input.try_parse(|p| parse_length_percentage(p, numeric, "arc radius"))
                    {
                        CssShapeArcRadii::Two {
                            horizontal,
                            vertical,
                        }
                    } else {
                        CssShapeArcRadii::One(horizontal)
                    },
                );
            }
            "cw" | "ccw" if sweep.is_none() => {
                sweep = Some(if option == "cw" {
                    CssShapeArcSweep::Cw
                } else {
                    CssShapeArcSweep::Ccw
                })
            }
            "large" | "small" if size.is_none() => {
                size = Some(if option == "large" {
                    CssShapeArcSize::Large
                } else {
                    CssShapeArcSize::Small
                })
            }
            "rotate" if rotation.is_none() => {
                rotation = Some(parse_angle_value(
                    input,
                    numeric,
                    AngleParserContext::ShapeRotation,
                )?)
            }
            _ => {
                return Err(crate::error::unsupported_value_at(
                    location,
                    None,
                    "invalid or duplicate arc option",
                ));
            }
        }
    }
    let radii = radii.ok_or_else(|| unsupported_value(input, None, "arc requires of and radii"))?;
    Ok(CssShapeArc::new(end, radii, sweep, size, rotation))
}
