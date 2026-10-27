//! Cumulative specified serialization of authored clipping shapes and reference boxes.

use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::effects::{CssClipPath, CssClipPathShape};
use crate::values::image::CssRadialExtent;
use crate::values::position::CssPosition;
use crate::values::shape::{
    CssBasicShape, CssCircleRadius, CssCircleShape, CssEllipseRadius, CssEllipseShape, CssFillRule,
    CssInsetShape, CssPathShape, CssPolygonShape, CssRectShape, CssXywhShape,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! specified_shape_methods {
    ($ty:ty) => {
        impl crate::serialization::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
                self.append_specified(writer)?;
                Ok(())
            }
        }
        crate::serialization::specified_rule_serialization::specified_serialization!($ty);
    };
}

impl CssClipPath {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                writer.node()?;
                writer.append("none")
            }
            Self::GeometryBox(value) => {
                writer.node()?;
                writer.append(value.as_str())
            }
            Self::Url(value) => value.append_specified(writer),
            Self::BasicShape(value) => value.append_specified(writer),
        }
    }
}
specified_shape_methods!(CssClipPath);

impl CssClipPathShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.shape().append_specified(writer)?;
        if let Some(reference_box) = self.reference_box() {
            writer.append(" ")?;
            writer.node()?;
            writer.append(reference_box.as_str())?;
        }
        Ok(())
    }
}
specified_shape_methods!(CssClipPathShape);

impl CssBasicShape {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Shape(value) => value.append_specified(writer),
            Self::Inset(value) => value.append_specified(writer),
            Self::Circle(value) => value.append_specified(writer),
            Self::Ellipse(value) => value.append_specified(writer),
            Self::Polygon(value) => value.append_specified(writer),
            Self::Path(value) => value.append_specified(writer),
            Self::Rect(value) => value.append_specified(writer),
            Self::Xywh(value) => value.append_specified(writer),
        }
    }
}
specified_shape_methods!(CssBasicShape);

impl CssInsetShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("inset(")?;
        writer.node()?; // Authored offsets list, independently of arity.
        writer.space_separated(self.offsets().values(), |writer, offset| {
            offset.append_specified(&mut writer.context, &mut writer.css)
        })?;
        if let Some(round) = self.round() {
            writer.append(" round ")?;
            round.append_specified(writer)?;
        }
        writer.append(")")
    }
}
specified_shape_methods!(CssInsetShape);

impl CssCircleShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("circle(")?;
        let explicit_radius = !matches!(self.radius(), CssCircleRadius::Default);
        match self.radius() {
            CssCircleRadius::Default => {}
            CssCircleRadius::Extent(value) => append_extent(*value, writer)?,
            CssCircleRadius::LengthPercentage(value) => {
                value.append_specified(&mut writer.context, &mut writer.css)?
            }
        }
        append_position(self.position(), explicit_radius, writer)?;
        writer.append(")")
    }
}
specified_shape_methods!(CssCircleShape);

impl CssEllipseShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("ellipse(")?;
        if let Some(radii) = self.radii() {
            writer.node()?;
            append_ellipse_radius(radii.horizontal(), writer)?;
            writer.append(" ")?;
            append_ellipse_radius(radii.vertical(), writer)?;
        }
        append_position(self.position(), self.radii().is_some(), writer)?;
        writer.append(")")
    }
}
specified_shape_methods!(CssEllipseShape);

fn append_ellipse_radius(value: &CssEllipseRadius, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    match value {
        CssEllipseRadius::Extent(value) => append_extent(*value, writer),
        CssEllipseRadius::LengthPercentage(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
    }
}

fn append_extent(value: CssRadialExtent, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.node()?;
    writer.append(match value {
        CssRadialExtent::ClosestSide => "closest-side",
        CssRadialExtent::FarthestSide => "farthest-side",
        CssRadialExtent::ClosestCorner => "closest-corner",
        CssRadialExtent::FarthestCorner => "farthest-corner",
    })
}

fn append_position(
    position: Option<&CssPosition>,
    has_radius: bool,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if let Some(position) = position {
        if has_radius {
            writer.append(" ")?;
        }
        writer.append("at ")?;
        position.append_specified(writer)?;
    }
    Ok(())
}

impl CssPolygonShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("polygon(")?;
        if let Some(fill) = self.fill_rule() {
            writer.node()?;
            writer.append(match fill {
                CssFillRule::Nonzero => "nonzero",
                CssFillRule::Evenodd => "evenodd",
            })?;
        }
        if let Some(round) = self.round() {
            if self.fill_rule().is_some() {
                writer.append(" ")?;
            }
            writer.append("round ")?;
            round.append_specified(&mut writer.context, &mut writer.css)?;
        }
        if self.fill_rule().is_some() || self.round().is_some() {
            writer.append(", ")?;
        }
        writer.node()?; // Point list.
        for (index, point) in self.points().points().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            writer.node()?;
            point
                .x()
                .append_specified(&mut writer.context, &mut writer.css)?;
            writer.append(" ")?;
            point
                .y()
                .append_specified(&mut writer.context, &mut writer.css)?;
        }
        writer.append(")")
    }
}
specified_shape_methods!(CssPolygonShape);

impl CssPathShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("path(")?;
        if let Some(fill) = self.fill_rule() {
            writer.node()?;
            writer.append(match fill {
                CssFillRule::Nonzero => "nonzero",
                CssFillRule::Evenodd => "evenodd",
            })?;
            writer.append(", ")?;
        }
        writer.node()?;
        writer.append_string(self.data().as_str())?;
        writer.append(")")
    }
}
specified_shape_methods!(CssPathShape);

impl CssRectShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("rect(")?;
        for (index, edge) in [self.top(), self.right(), self.bottom(), self.left()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                writer.append(" ")?;
            }
            edge.append_to_rule_writer(writer)?;
        }
        append_rectangle_round(self.round(), writer)?;
        writer.append(")")
    }
}
specified_shape_methods!(CssRectShape);

impl CssXywhShape {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("xywh(")?;
        self.x()
            .append_specified(&mut writer.context, &mut writer.css)?;
        writer.append(" ")?;
        self.y()
            .append_specified(&mut writer.context, &mut writer.css)?;
        writer.append(" ")?;
        self.width()
            .append_specified(&mut writer.context, &mut writer.css)?;
        writer.append(" ")?;
        self.height()
            .append_specified(&mut writer.context, &mut writer.css)?;
        append_rectangle_round(self.round(), writer)?;
        writer.append(")")
    }
}
specified_shape_methods!(CssXywhShape);

fn append_rectangle_round(
    round: Option<&crate::values::border::CssBorderRadiusShorthand>,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if let Some(round) = round {
        writer.append(" round ")?;
        round.append_specified(writer)?;
    }
    Ok(())
}
