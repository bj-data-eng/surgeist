//! Authored shape command parsing over shared exact scalar and position owners.
use super::position::parse_full_position_bounded;
use super::syntax_types::Stream;
use super::values::{next_is_ident, parse_angle_value, parse_length_percentage};
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::numeric::expression::NumericInputContext;
use crate::parser::syntax_types::SyntaxError;
use crate::values::position::{CssHorizontalPositionKeyword, CssVerticalPositionKeyword};
use crate::values::shape::{
    CssFillRule, CssShapeAbsoluteControlPoint, CssShapeArc, CssShapeArcRadii, CssShapeArcSize,
    CssShapeArcSweep, CssShapeCommand, CssShapeCommandList, CssShapeControlAnchor,
    CssShapeCoordinatePair, CssShapeCurve, CssShapeEndpoint, CssShapeFunction,
    CssShapeHorizontalLine, CssShapeRelativeControlPoint, CssShapeSmooth, CssShapeVerticalLine,
};

type Result<'i, T> = std::result::Result<T, SyntaxError<'i, Error>>;
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

pub(super) fn parse_shape<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeFunction> {
    let fill = if input.attempt(|p| p.consume_keyword("nonzero")).is_ok() {
        Some(CssFillRule::Nonzero)
    } else if input.attempt(|p| p.consume_keyword("evenodd")).is_ok() {
        Some(CssFillRule::Evenodd)
    } else {
        None
    };
    input.consume_keyword("from").map_err(basic)?;
    let start = parse_full_position_bounded(input, numeric, COMMANDS)?;
    // The individually adopted pinned-source editorial disposition requires this comma.
    input.consume_comma().map_err(basic)?;
    let mut commands = vec![parse_command(input, numeric)?];
    while !input.only_whitespace_remains() {
        input.consume_comma().map_err(basic)?;
        commands.push(parse_command(input, numeric)?);
    }
    Ok(CssShapeFunction::new(
        fill,
        start,
        CssShapeCommandList::try_new(commands).expect("first command is required"),
    ))
}
fn pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeCoordinatePair> {
    Ok(CssShapeCoordinatePair::new(
        parse_length_percentage(input, numeric)?,
        parse_length_percentage(input, numeric)?,
    ))
}
fn affinity<'i>(input: &mut Stream<'i>) -> Result<'i, bool> {
    let location = input.site();
    let word = input.consume_ident().map_err(basic)?;
    if word.eq_ignore_ascii_case("to") {
        Ok(true)
    } else if word.eq_ignore_ascii_case("by") {
        Ok(false)
    } else {
        Err(crate::diagnostic::error::property::unexpected_at(location))
    }
}
fn endpoint<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeEndpoint> {
    if affinity(input)? {
        parse_full_position_bounded(input, numeric, ENDPOINT).map(CssShapeEndpoint::To)
    } else {
        pair(input, numeric).map(CssShapeEndpoint::By)
    }
}
fn anchor<'i>(input: &mut Stream<'i>) -> Result<'i, CssShapeControlAnchor> {
    let location = input.site();
    let word = input.consume_ident().map_err(basic)?;
    match word.to_ascii_lowercase().as_str() {
        "start" => Ok(CssShapeControlAnchor::Start),
        "end" => Ok(CssShapeControlAnchor::End),
        "origin" => Ok(CssShapeControlAnchor::Origin),
        _ => Err(crate::diagnostic::error::property::unexpected_at(location)),
    }
}
fn relative_control<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeRelativeControlPoint> {
    let offset = pair(input, numeric)?;
    let anchor = if next_is_ident(input, "from") {
        input.consume_keyword("from").map_err(basic)?;
        Some(anchor(input)?)
    } else {
        None
    };
    Ok(CssShapeRelativeControlPoint::new(offset, anchor))
}
fn absolute_control<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeAbsoluteControlPoint> {
    // Explicit anchors admit exactly a coordinate pair, never arbitrary position families.
    if let Ok(offset) = input.attempt(|p| {
        let offset = pair(p, numeric)?;
        p.consume_keyword("from").map_err(basic)?;
        Ok::<_, SyntaxError<'i, Error>>(offset)
    }) {
        return Ok(CssShapeAbsoluteControlPoint::from_coordinates(
            offset,
            Some(anchor(input)?),
        ));
    }
    parse_full_position_bounded(input, numeric, CONTROL)
        .map(CssShapeAbsoluteControlPoint::from_position)
}
fn parse_command<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeCommand> {
    let location = input.site();
    let command = input.consume_ident().map_err(basic)?.to_ascii_lowercase();
    match command.as_str() {
        "close" => Ok(CssShapeCommand::Close),
        "move" => endpoint(input, numeric).map(CssShapeCommand::Move),
        "line" => endpoint(input, numeric).map(CssShapeCommand::Line),
        "hline" => {
            let to = affinity(input)?;
            if to {
                let keyword = input.attempt(|p| {
                    let word = p.consume_ident().map_err(basic)?;
                    match word.to_ascii_lowercase().as_str() {
                        "left" => Ok(CssHorizontalPositionKeyword::Left),
                        "center" => Ok(CssHorizontalPositionKeyword::Center),
                        "right" => Ok(CssHorizontalPositionKeyword::Right),
                        "x-start" => Ok(CssHorizontalPositionKeyword::XStart),
                        "x-end" => Ok(CssHorizontalPositionKeyword::XEnd),
                        _ => Err(unexpected_at(p.site())),
                    }
                });
                if let Ok(keyword) = keyword {
                    return Ok(CssShapeCommand::HorizontalLine(
                        CssShapeHorizontalLine::ToKeyword(keyword),
                    ));
                }
            }
            let value = parse_length_percentage(input, numeric)?;
            Ok(CssShapeCommand::HorizontalLine(if to {
                CssShapeHorizontalLine::ToOffset(value)
            } else {
                CssShapeHorizontalLine::By(value)
            }))
        }
        "vline" => {
            let to = affinity(input)?;
            if to {
                let keyword = input.attempt(|p| {
                    let word = p.consume_ident().map_err(basic)?;
                    match word.to_ascii_lowercase().as_str() {
                        "top" => Ok(CssVerticalPositionKeyword::Top),
                        "center" => Ok(CssVerticalPositionKeyword::Center),
                        "bottom" => Ok(CssVerticalPositionKeyword::Bottom),
                        "y-start" => Ok(CssVerticalPositionKeyword::YStart),
                        "y-end" => Ok(CssVerticalPositionKeyword::YEnd),
                        _ => Err(unexpected_at(p.site())),
                    }
                });
                if let Ok(keyword) = keyword {
                    return Ok(CssShapeCommand::VerticalLine(
                        CssShapeVerticalLine::ToKeyword(keyword),
                    ));
                }
            }
            let value = parse_length_percentage(input, numeric)?;
            Ok(CssShapeCommand::VerticalLine(if to {
                CssShapeVerticalLine::ToOffset(value)
            } else {
                CssShapeVerticalLine::By(value)
            }))
        }
        "curve" => {
            let end = endpoint(input, numeric)?;
            input.consume_keyword("with").map_err(basic)?;
            Ok(CssShapeCommand::Curve(match end {
                CssShapeEndpoint::To(end) => {
                    let first = absolute_control(input, numeric)?;
                    let second = if input.attempt(|p| p.consume_delim('/')).is_ok() {
                        Some(absolute_control(input, numeric)?)
                    } else {
                        None
                    };
                    CssShapeCurve::to(end, first, second)
                }
                CssShapeEndpoint::By(end) => {
                    let first = relative_control(input, numeric)?;
                    let second = if input.attempt(|p| p.consume_delim('/')).is_ok() {
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
            let with = input.attempt(|p| p.consume_keyword("with")).is_ok();
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
        _ => Err(crate::diagnostic::error::property::unexpected_at(location)),
    }
}
fn parse_arc<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<'i, CssShapeArc> {
    let end = endpoint(input, numeric)?;
    let (mut radii, mut sweep, mut size, mut rotation) = (None, None, None, None);
    while !input.only_whitespace_remains() && !super::values::next_is_comma(input) {
        let location = input.site();
        let option = input.consume_ident().map_err(basic)?.to_ascii_lowercase();
        match option.as_str() {
            "of" if radii.is_none() => {
                let horizontal = parse_length_percentage(input, numeric)?;
                radii = Some(
                    if let Ok(vertical) = input.attempt(|p| parse_length_percentage(p, numeric)) {
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
            "rotate" if rotation.is_none() => rotation = Some(parse_angle_value(input, numeric)?),
            _ => {
                return Err(crate::diagnostic::error::property::unexpected_at(location));
            }
        }
    }
    let radii = radii.ok_or_else(|| unexpected_at(input.site()))?;
    Ok(CssShapeArc::new(end, radii, sweep, size, rotation))
}
