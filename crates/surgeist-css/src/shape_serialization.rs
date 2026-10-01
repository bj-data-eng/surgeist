//! Cumulative specified emission of authored shape commands.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::*;
type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;
fn charge(w: &mut SpecifiedRuleWriter) -> Result<()> {
    w.context.charge_input(1)?;
    w.context.charge_projection(1)
}
fn scalar(v: &CssSpecifiedLengthPercentage, w: &mut SpecifiedRuleWriter) -> Result<()> {
    v.append_specified(&mut w.context, &mut w.css)
}
fn pair(v: &CssShapeCoordinatePair, w: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(w)?;
    scalar(v.x(), w)?;
    w.append(" ")?;
    scalar(v.y(), w)
}
fn endpoint(v: &CssShapeEndpoint, w: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(w)?;
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
    charge(w)?;
    w.append(" from ")?;
    w.append(match v {
        CssShapeControlAnchor::Start => "start",
        CssShapeControlAnchor::End => "end",
        CssShapeControlAnchor::Origin => "origin",
    })
}
fn absolute(v: &CssShapeAbsoluteControlPoint, w: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(w)?;
    match v.view() {
        CssShapeAbsoluteControlPointRef::Position(p) => p.append_specified(w),
        CssShapeAbsoluteControlPointRef::Coordinates { offset, anchor: a } => {
            pair(offset, w)?;
            anchor(a, w)
        }
    }
}
fn relative(v: &CssShapeRelativeControlPoint, w: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(w)?;
    pair(v.offset(), w)?;
    if let Some(a) = v.anchor() {
        anchor(a, w)?;
    }
    Ok(())
}
impl CssShapeFunction {
    /// Serializes authored components without resolving geometry or inserting defaults.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Uses one cumulative input, projection and byte budget for the complete function.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut w = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut w)?;
        Ok(w.css)
    }
    pub(crate) fn append_specified(&self, w: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(w)?;
        w.append("shape(")?;
        if let Some(fill) = self.fill_rule() {
            charge(w)?;
            w.append(match fill {
                CssFillRule::Nonzero => "nonzero ",
                CssFillRule::Evenodd => "evenodd ",
            })?;
        }
        charge(w)?;
        w.append("from ")?;
        self.start().append_specified(w)?;
        w.append(", ")?;
        charge(w)?;
        for (i, c) in self.commands().commands().iter().enumerate() {
            if i != 0 {
                w.append(", ")?;
            }
            command(c, w)?;
        }
        w.append(")")
    }
}
fn command(c: &CssShapeCommand, w: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(w)?;
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
            charge(w)?;
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
            charge(w)?;
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
            charge(w)?;
            match v.view() {
                CssShapeCurveRef::To { end, first, second } => {
                    w.append("to ")?;
                    end.append_specified(w)?;
                    charge(w)?;
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
                    charge(w)?;
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
            charge(w)?;
            match v.view() {
                CssShapeSmoothRef::To { end, control } => {
                    w.append("to ")?;
                    end.append_specified(w)?;
                    if let Some(control) = control {
                        charge(w)?;
                        w.append(" with ")?;
                        absolute(control, w)?;
                    }
                }
                CssShapeSmoothRef::By { end, control } => {
                    w.append("by ")?;
                    pair(end, w)?;
                    if let Some(control) = control {
                        charge(w)?;
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
            charge(w)?;
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
                charge(w)?;
                w.append(match sweep {
                    CssShapeArcSweep::Cw => " cw",
                    CssShapeArcSweep::Ccw => " ccw",
                })?;
            }
            if let Some(size) = v.size() {
                charge(w)?;
                w.append(match size {
                    CssShapeArcSize::Large => " large",
                    CssShapeArcSize::Small => " small",
                })?;
            }
            if let Some(rotation) = v.rotation() {
                charge(w)?;
                w.append(" rotate ")?;
                rotation.append_specified(&mut w.context, &mut w.css)?;
            }
            Ok(())
        }
    }
}
