//! Cumulative specified serialization of authored clipping shapes and reference boxes.

use crate::{
    CssBasicShape, CssCircleRadius, CssCircleShape, CssClipPath, CssClipPathShape,
    CssEllipseRadius, CssEllipseShape, CssFillRule, CssInsetShape, CssPathShape, CssPolygonShape,
    CssPosition, CssRadialExtent, CssRectShape, CssRectShapeEdge,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssXywhShape,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_shape_methods {
    () => {
        /// Serializes authored components without resolving geometry or inserting defaults.
        pub fn serialize_specified(&self) -> Result<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Uses one cumulative input, projection and UTF-8 byte budget for all children.
        /// Failure returns no partial CSS and leaves authored values and origins unchanged.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> Result<String> {
            let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
            self.append_to_rule_writer(&mut writer)?;
            Ok(writer.css)
        }

        pub(crate) fn append_to_rule_writer(
            &self,
            writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
        ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
            self.append_specified(writer)?;
            Ok(())
        }
    };
}

fn charge_node(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

impl CssClipPath {
    specified_shape_methods!();
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                charge_node(writer)?;
                writer.append("none")
            }
            Self::GeometryBox(value) => {
                charge_node(writer)?;
                writer.append(value.as_css_str())
            }
            Self::Url(value) => value.append_specified(writer),
            Self::BasicShape(value) => value.append_specified(writer),
        }
    }
}

impl CssClipPathShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
        self.shape().append_specified(writer)?;
        if let Some(reference_box) = self.reference_box() {
            writer.append(" ")?;
            charge_node(writer)?;
            writer.append(reference_box.as_css_str())?;
        }
        Ok(())
    }
}

impl CssBasicShape {
    specified_shape_methods!();
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

impl CssInsetShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
        writer.append("inset(")?;
        charge_node(writer)?; // Authored offsets list, independently of arity.
        for (index, offset) in self.offsets().values().iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            offset.append_specified(&mut writer.context, &mut writer.css)?;
        }
        if let Some(round) = self.round() {
            writer.append(" round ")?;
            round.append_specified(writer)?;
        }
        writer.append(")")
    }
}

impl CssCircleShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
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

impl CssEllipseShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
        writer.append("ellipse(")?;
        if let Some(radii) = self.radii() {
            charge_node(writer)?;
            append_ellipse_radius(radii.horizontal(), writer)?;
            writer.append(" ")?;
            append_ellipse_radius(radii.vertical(), writer)?;
        }
        append_position(self.position(), self.radii().is_some(), writer)?;
        writer.append(")")
    }
}

fn append_ellipse_radius(value: &CssEllipseRadius, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    match value {
        CssEllipseRadius::Extent(value) => append_extent(*value, writer),
        CssEllipseRadius::LengthPercentage(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
    }
}

fn append_extent(value: CssRadialExtent, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    charge_node(writer)?;
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
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
        writer.append("polygon(")?;
        if let Some(fill) = self.fill_rule() {
            charge_node(writer)?;
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
        charge_node(writer)?; // Point list.
        for (index, point) in self.points().points().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            charge_node(writer)?;
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

impl CssPathShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
        writer.append("path(")?;
        if let Some(fill) = self.fill_rule() {
            charge_node(writer)?;
            writer.append(match fill {
                CssFillRule::Nonzero => "nonzero",
                CssFillRule::Evenodd => "evenodd",
            })?;
            writer.append(", ")?;
        }
        charge_node(writer)?;
        writer.append_string(self.data().as_str())?;
        writer.append(")")
    }
}

impl CssRectShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
        writer.append("rect(")?;
        for (index, edge) in [self.top(), self.right(), self.bottom(), self.left()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                writer.append(" ")?;
            }
            match edge {
                CssRectShapeEdge::Auto => {
                    charge_node(writer)?;
                    writer.append("auto")?;
                }
                CssRectShapeEdge::LengthPercentage(value) => {
                    value.append_specified(&mut writer.context, &mut writer.css)?
                }
            }
        }
        append_rectangle_round(self.round(), writer)?;
        writer.append(")")
    }
}

impl CssXywhShape {
    specified_shape_methods!();
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_node(writer)?;
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

fn append_rectangle_round(
    round: Option<&crate::CssBorderRadiusShorthand>,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if let Some(round) = round {
        writer.append(" round ")?;
        round.append_specified(writer)?;
    }
    Ok(())
}
