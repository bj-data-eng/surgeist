//! Cumulative specified emission of authored shape commands.
use crate::numeric::{CssLengthPercentage, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::position::{CssHorizontalPositionKeyword, CssVerticalPositionKeyword};
use crate::values::shape::{
    CssFillRule, CssShapeAbsoluteControlPoint, CssShapeAbsoluteControlPointRef, CssShapeArcRadii,
    CssShapeArcSize, CssShapeArcSweep, CssShapeCommand, CssShapeControlAnchor,
    CssShapeCoordinatePair, CssShapeCurveRef, CssShapeEndpoint, CssShapeFunction,
    CssShapeHorizontalLine, CssShapeRelativeControlPoint, CssShapeSmoothRef, CssShapeVerticalLine,
};
type Result<T> = std::result::Result<T, CssSerializationError>;
fn scalar(v: &CssNumeric<CssLengthPercentage>, w: &mut SpecifiedRuleWriter) -> Result<()> {
    v.append_specified(&mut w.context, &mut w.css)
}
fn pair(v: &CssShapeCoordinatePair, w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.node()?;
    scalar(v.x(), w)?;
    w.append(" ")?;
    scalar(v.y(), w)
}
fn endpoint(v: &CssShapeEndpoint, w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.node()?;
    match v {
        CssShapeEndpoint::To(p) => {
            w.append("to ")?;
            p.append_specified(w)
        }
        CssShapeEndpoint::By(p) => {
            w.append("by ")?;
            pair(p, w)
        }
    }
}
fn anchor(v: CssShapeControlAnchor, w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.node()?;
    w.append(" from ")?;
    w.append(match v {
        CssShapeControlAnchor::Start => "start",
        CssShapeControlAnchor::End => "end",
        CssShapeControlAnchor::Origin => "origin",
    })
}
fn absolute(v: &CssShapeAbsoluteControlPoint, w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.node()?;
    match v.view() {
        CssShapeAbsoluteControlPointRef::Position(p) => p.append_specified(w),
        CssShapeAbsoluteControlPointRef::Coordinates { offset, anchor: a } => {
            pair(offset, w)?;
            anchor(a, w)
        }
    }
}
fn relative(v: &CssShapeRelativeControlPoint, w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.node()?;
    pair(v.offset(), w)?;
    if let Some(a) = v.anchor() {
        anchor(a, w)?;
    }
    Ok(())
}
impl CssShapeFunction {
    pub(crate) fn append_specified(&self, w: &mut SpecifiedRuleWriter) -> Result<()> {
        w.node()?;
        w.append("shape(")?;
        if let Some(fill) = self.fill_rule() {
            w.node()?;
            w.append(match fill {
                CssFillRule::Nonzero => "nonzero ",
                CssFillRule::Evenodd => "evenodd ",
            })?;
        }
        w.node()?;
        w.append("from ")?;
        self.start().append_specified(w)?;
        w.append(", ")?;
        w.node()?;
        for (i, c) in self.commands().commands().iter().enumerate() {
            if i != 0 {
                w.append(", ")?;
            }
            command(c, w)?;
        }
        w.append(")")
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssShapeFunction {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssShapeFunction);
fn command(c: &CssShapeCommand, w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.node()?;
    match c {
        CssShapeCommand::Close => w.append("close"),
        CssShapeCommand::Move(p) => {
            w.append("move ")?;
            endpoint(p, w)
        }
        CssShapeCommand::Line(p) => {
            w.append("line ")?;
            endpoint(p, w)
        }
        CssShapeCommand::HorizontalLine(v) => {
            w.append("hline ")?;
            w.node()?;
            match v {
                CssShapeHorizontalLine::By(v) => {
                    w.append("by ")?;
                    scalar(v, w)
                }
                CssShapeHorizontalLine::ToOffset(v) => {
                    w.append("to ")?;
                    scalar(v, w)
                }
                CssShapeHorizontalLine::ToKeyword(v) => {
                    w.append("to ")?;
                    w.append(match v {
                        CssHorizontalPositionKeyword::Left => "left",
                        CssHorizontalPositionKeyword::Center => "center",
                        CssHorizontalPositionKeyword::Right => "right",
                        CssHorizontalPositionKeyword::XStart => "x-start",
                        CssHorizontalPositionKeyword::XEnd => "x-end",
                    })
                }
            }
        }
        CssShapeCommand::VerticalLine(v) => {
            w.append("vline ")?;
            w.node()?;
            match v {
                CssShapeVerticalLine::By(v) => {
                    w.append("by ")?;
                    scalar(v, w)
                }
                CssShapeVerticalLine::ToOffset(v) => {
                    w.append("to ")?;
                    scalar(v, w)
                }
                CssShapeVerticalLine::ToKeyword(v) => {
                    w.append("to ")?;
                    w.append(match v {
                        CssVerticalPositionKeyword::Top => "top",
                        CssVerticalPositionKeyword::Center => "center",
                        CssVerticalPositionKeyword::Bottom => "bottom",
                        CssVerticalPositionKeyword::YStart => "y-start",
                        CssVerticalPositionKeyword::YEnd => "y-end",
                    })
                }
            }
        }
        CssShapeCommand::Curve(v) => {
            w.append("curve ")?;
            w.node()?;
            match v.view() {
                CssShapeCurveRef::To { end, first, second } => {
                    w.append("to ")?;
                    end.append_specified(w)?;
                    w.node()?;
                    w.append(" with ")?;
                    absolute(first, w)?;
                    if let Some(second) = second {
                        w.append(" / ")?;
                        absolute(second, w)?;
                    }
                }
                CssShapeCurveRef::By { end, first, second } => {
                    w.append("by ")?;
                    pair(end, w)?;
                    w.node()?;
                    w.append(" with ")?;
                    relative(first, w)?;
                    if let Some(second) = second {
                        w.append(" / ")?;
                        relative(second, w)?;
                    }
                }
            }
            Ok(())
        }
        CssShapeCommand::Smooth(v) => {
            w.append("smooth ")?;
            w.node()?;
            match v.view() {
                CssShapeSmoothRef::To { end, control } => {
                    w.append("to ")?;
                    end.append_specified(w)?;
                    if let Some(control) = control {
                        w.node()?;
                        w.append(" with ")?;
                        absolute(control, w)?;
                    }
                }
                CssShapeSmoothRef::By { end, control } => {
                    w.append("by ")?;
                    pair(end, w)?;
                    if let Some(control) = control {
                        w.node()?;
                        w.append(" with ")?;
                        relative(control, w)?;
                    }
                }
            }
            Ok(())
        }
        CssShapeCommand::Arc(v) => {
            w.append("arc ")?;
            endpoint(v.endpoint(), w)?;
            w.node()?;
            w.append(" of ")?;
            match v.radii() {
                CssShapeArcRadii::One(v) => scalar(v, w)?,
                CssShapeArcRadii::Two {
                    horizontal,
                    vertical,
                } => {
                    scalar(horizontal, w)?;
                    w.append(" ")?;
                    scalar(vertical, w)?;
                }
            };
            if let Some(sweep) = v.sweep() {
                w.node()?;
                w.append(match sweep {
                    CssShapeArcSweep::Cw => " cw",
                    CssShapeArcSweep::Ccw => " ccw",
                })?;
            }
            if let Some(size) = v.size() {
                w.node()?;
                w.append(match size {
                    CssShapeArcSize::Large => " large",
                    CssShapeArcSize::Small => " small",
                })?;
            }
            if let Some(rotation) = v.rotation() {
                w.node()?;
                w.append(" rotate ")?;
                rotation.append_specified(&mut w.context, &mut w.css)?;
            }
            Ok(())
        }
    }
}
