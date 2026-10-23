//! Bounded specified serialization for the shared authored Image graph.

use crate::specified_rule_serialization::SpecifiedProvider as _;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssAngleOrZero, CssColorStopList, CssColorStopListItem, CssFilterImage, CssFilterImageInput,
    CssFilterImageInputRef, CssFilterImageString, CssGradient, CssHorizontalGradientSide,
    CssHorizontalPosition, CssImage, CssImageValue, CssImageValueList, CssLinearGradient,
    CssLinearGradientDirection, CssPosition, CssPositionRef, CssRadialExtent, CssRadialGradient,
    CssRadialShape, CssRadialSize, CssSerializationError, CssSpecifiedLengthPercentage,
    CssVerticalGradientSide, CssVerticalPosition,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssImage {
    /// Appends to an enclosing specified writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        self.value().append_specified(writer)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssImage {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssImage);

impl CssImageValue {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_image_work(Work::Image(self), writer)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssImageValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssImageValue);

enum Work<'a> {
    Image(&'a CssImageValue),
    Filter(&'a CssFilterImage),
    Input(&'a CssFilterImageInput),
    List(&'a crate::CssFilterFunctionList),
    Text(&'static str),
}

fn append_image_work(first: Work<'_>, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    let capacity = || {
        CssSerializationError::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
    };
    let mut work = Vec::new();
    work.try_reserve(1).map_err(|_| capacity())?;
    work.push(first);
    while let Some(item) = work.pop() {
        match item {
            Work::Text(text) => writer.append(text)?,
            Work::List(list) => list.append_to_rule_writer(writer)?,
            Work::Input(input) => match input.view() {
                CssFilterImageInputRef::String(value) => value.append_specified(writer)?,
                CssFilterImageInputRef::Image(value) => {
                    work.try_reserve(1).map_err(|_| capacity())?;
                    work.push(Work::Image(value.value()));
                }
            },
            Work::Filter(value) => {
                writer.node()?;
                work.try_reserve(5).map_err(|_| capacity())?;
                work.push(Work::Text(")"));
                work.push(Work::List(value.filters()));
                work.push(Work::Text(", "));
                work.push(Work::Input(value.input()));
                work.push(Work::Text("filter("));
            }
            Work::Image(image) => match image {
                CssImageValue::None => {
                    writer.node()?;
                    writer.append("none")?;
                }
                CssImageValue::Url(url) => url.append_specified(writer)?,
                CssImageValue::Gradient(gradient) => gradient.append_specified(writer)?,
                CssImageValue::Filter(value) => {
                    work.try_reserve(1).map_err(|_| capacity())?;
                    work.push(Work::Filter(value));
                }
                CssImageValue::LightDark(value) => {
                    writer.node()?;
                    work.try_reserve(5).map_err(|_| capacity())?;
                    work.push(Work::Text(")"));
                    work.push(Work::Image(value.dark()));
                    work.push(Work::Text(", "));
                    work.push(Work::Image(value.light()));
                    work.push(Work::Text("light-dark("));
                }
            },
        }
    }
    Ok(())
}

/// Filter-image providers write through their writer-level `append_specified`.
macro_rules! filter_image_provider {
    ($($ty:ty),+ $(,)?) => {$(
        impl crate::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
                self.append_specified(writer)
            }
        }
        crate::specified_rule_serialization::specified_serialization!($ty);
    )+};
}
filter_image_provider!(CssFilterImage, CssFilterImageInput, CssFilterImageString);
impl CssFilterImage {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_image_work(Work::Filter(self), writer)
    }
}
impl CssFilterImageInput {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_image_work(Work::Input(self), writer)
    }
}
impl CssFilterImageString {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append_string(self.as_str())
    }
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssImageValueList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?;
        writer.comma_separated(self.images(), |writer, image| {
            image.append_specified(writer)
        })?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssImageValueList);

impl CssGradient {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        match self {
            Self::Linear(value) => {
                writer.append("linear-gradient(")?;
                append_linear(value, writer)?;
            }
            Self::RepeatingLinear(value) => {
                writer.append("repeating-linear-gradient(")?;
                append_linear(value, writer)?;
            }
            Self::Radial(value) => {
                writer.append("radial-gradient(")?;
                append_radial(value, writer)?;
            }
            Self::RepeatingRadial(value) => {
                writer.append("repeating-radial-gradient(")?;
                append_radial(value, writer)?;
            }
            Self::Conic(value) => {
                writer.append("conic-gradient(")?;
                append_conic(value, writer)?;
            }
        }
        writer.append(")")
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssGradient {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssGradient);

fn append_linear(value: &CssLinearGradient, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    let mut emitted = false;
    if let Some(direction) = value.direction() {
        emitted = append_direction(direction, writer)?;
    }
    let (default, stops) = writer.capture_value(|writer| append_stops(value.stops(), writer))?;
    append_interpolation(value.interpolation(), default, &mut emitted, writer)?;
    if emitted {
        writer.append(", ")?;
    }
    writer.append(&stops)
}

/// Defaults compose during the real charged stop traversal. A nonlegacy
/// stop suffices to establish Oklab, independently on each side of projection.
#[derive(Clone, Copy)]
struct GradientDefault {
    original: crate::syntax::color::serialization::GradientColorDefault,
    emitted: crate::syntax::color::serialization::GradientColorDefault,
}
impl GradientDefault {
    fn new() -> Self {
        use crate::syntax::color::serialization::GradientColorDefault as D;
        Self {
            original: D::Srgb,
            emitted: D::Srgb,
        }
    }
    fn observe(&mut self, color: &crate::CssColor) {
        use crate::syntax::color::serialization::GradientColorDefault as D;
        let proof = color.gradient_color_defaults();
        if proof.original == D::Oklab {
            self.original = D::Oklab;
        }
        if proof.emitted == D::Oklab {
            self.emitted = D::Oklab;
        }
    }
    fn matches_emitted(self, space: crate::ColorSpace) -> bool {
        use crate::syntax::color::serialization::GradientColorDefault as D;
        match self.emitted {
            D::Oklab => space == crate::ColorSpace::Oklab,
            D::Srgb => space == crate::ColorSpace::Predefined(crate::PredefinedColorSpace::Srgb),
        }
    }
}

fn append_interpolation(
    value: Option<&crate::CssColorInterpolation>,
    default: GradientDefault,
    emitted: &mut bool,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if let Some(value) = value {
        writer.node()?;
        if value
            .predefined()
            .is_some_and(|method| default.matches_emitted(method.space()))
        {
            // Omitted authored methods still consume their ordinary node tariff.
            return Ok(());
        }
        if *emitted {
            writer.append(" ")?;
        }
        writer.append("in ")?;
        let css =
            crate::syntax::color::serialization::serialize_interpolation(value, &writer.context)?;
        writer.append(&css)?;
        *emitted = true;
    } else if default.original != default.emitted {
        // A mandated Color projection can change the implicit default. Emit
        // its original method without inventing an authored input node.
        writer.context.charge_projection(1)?;
        if *emitted {
            writer.append(" ")?;
        }
        writer.append("in ")?;
        writer.append(match default.original {
            crate::syntax::color::serialization::GradientColorDefault::Oklab => "oklab",
            crate::syntax::color::serialization::GradientColorDefault::Srgb => "srgb",
        })?;
        *emitted = true;
    }
    Ok(())
}

fn append_direction(
    direction: &CssLinearGradientDirection,
    writer: &mut SpecifiedRuleWriter,
) -> Result<bool> {
    writer.node()?;
    match direction {
        CssLinearGradientDirection::Angle(angle) => {
            if let CssAngleOrZero::Angle(value) = angle
                && let Some(literal) = value.literal()
            {
                let omitted = literal.is_default_gradient_direction();
                if omitted {
                    writer.without_output(|writer| {
                        angle.append_specified(&mut writer.context, &mut writer.css)
                    })?;
                } else {
                    angle.append_specified(&mut writer.context, &mut writer.css)?;
                }
                Ok(!omitted)
            } else {
                angle.append_specified(&mut writer.context, &mut writer.css)?;
                Ok(true)
            }
        }
        CssLinearGradientDirection::SideOrCorner(side) => {
            let omitted = side.horizontal().is_none()
                && side.vertical() == Some(CssVerticalGradientSide::Bottom);
            if !omitted {
                writer.append("to ")?;
            }
            if let Some(horizontal) = side.horizontal() {
                writer.node()?;
                if !omitted {
                    writer.append(match horizontal {
                        CssHorizontalGradientSide::Left => "left",
                        CssHorizontalGradientSide::Right => "right",
                    })?;
                }
            }
            if let Some(vertical) = side.vertical() {
                writer.node()?;
                if !omitted {
                    if side.horizontal().is_some() {
                        writer.append(" ")?;
                    }
                    writer.append(match vertical {
                        CssVerticalGradientSide::Top => "top",
                        CssVerticalGradientSide::Bottom => "bottom",
                    })?;
                }
            }
            Ok(!omitted)
        }
    }
}

fn append_radial(value: &CssRadialGradient, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    let mut emitted = false;
    if let Some(shape) = value.shape() {
        writer.node()?;
        let omit = matches!(shape, CssRadialShape::Ellipse)
            || (matches!(shape, CssRadialShape::Circle)
                && matches!(value.size(), Some(CssRadialSize::Circle(_))));
        if !omit {
            writer.append(match shape {
                CssRadialShape::Circle => "circle",
                CssRadialShape::Ellipse => "ellipse",
            })?;
            emitted = true;
        }
    }
    if let Some(size) = value.size() {
        writer.node()?;
        let omit = matches!(size, CssRadialSize::Extent(CssRadialExtent::FarthestCorner));
        if !omit {
            if emitted {
                writer.append(" ")?;
            }
            match size {
                CssRadialSize::Extent(extent) => {
                    writer.node()?;
                    writer.append(radial_extent(*extent))?;
                }
                CssRadialSize::Circle(circle) => {
                    circle.append_specified(&mut writer.context, &mut writer.css)?;
                }
                CssRadialSize::Ellipse(ellipse) => {
                    ellipse
                        .horizontal()
                        .append_specified(&mut writer.context, &mut writer.css)?;
                    writer.append(" ")?;
                    ellipse
                        .vertical()
                        .append_specified(&mut writer.context, &mut writer.css)?;
                }
            }
            emitted = true;
        } else {
            writer.node()?;
        }
    }
    if let Some(position) = value.position() {
        if radial_position_is_center(position) {
            // The checked position writer charges one aggregate and each axis,
            // plus one numeric leaf for each authored 50% offset.
            writer.without_output(|writer| position.append_specified(writer))?;
        } else {
            if emitted {
                writer.append(" ")?;
            }
            writer.append("at ")?;
            position.append_specified(writer)?;
            emitted = true;
        }
    }
    let (default, stops) = writer.capture_value(|writer| append_stops(value.stops(), writer))?;
    append_interpolation(value.interpolation(), default, &mut emitted, writer)?;
    if emitted {
        writer.append(", ")?;
    }
    writer.append(&stops)
}

fn radial_extent(extent: CssRadialExtent) -> &'static str {
    match extent {
        CssRadialExtent::ClosestSide => "closest-side",
        CssRadialExtent::FarthestSide => "farthest-side",
        CssRadialExtent::ClosestCorner => "closest-corner",
        CssRadialExtent::FarthestCorner => "farthest-corner",
    }
}

fn radial_position_is_center(position: &CssPosition) -> bool {
    let CssPositionRef::Cartesian(position) = position.view() else {
        return false;
    };
    let horizontal = match position.horizontal() {
        CssHorizontalPosition::Center => true,
        CssHorizontalPosition::Offset(offset) => literal_percentage_is(offset, "5", 1),
        _ => false,
    };
    let vertical = match position.vertical() {
        CssVerticalPosition::Center => true,
        CssVerticalPosition::Offset(offset) => literal_percentage_is(offset, "5", 1),
        _ => false,
    };
    horizontal && vertical
}

fn append_stops(
    stops: &CssColorStopList,
    writer: &mut SpecifiedRuleWriter,
) -> Result<GradientDefault> {
    let mut default = GradientDefault::new();
    writer.node()?;
    let last = stops.items().len() - 1;
    for (index, item) in stops.items().iter().enumerate() {
        if index != 0 {
            writer.append(", ")?;
        }
        writer.node()?;
        match item {
            CssColorStopListItem::Stop(stop) => {
                stop.color()
                    .append_specified(&mut writer.context, &mut writer.css)?;
                default.observe(stop.color());
                for position in stop.positions() {
                    let omit = stop.positions().len() == 1
                        && ((index == 0 && is_direct_zero(position))
                            || (last > 0 && index == last && is_direct_hundred_percent(position)));
                    if omit {
                        writer.node()?;
                    } else {
                        writer.append(" ")?;
                        append_line_position(position, writer)?;
                    }
                }
            }
            CssColorStopListItem::Hint(position) => append_line_position(position, writer)?,
        }
    }
    Ok(default)
}

fn append_conic(value: &crate::CssConicGradient, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    let mut emitted = false;
    if let Some(from) = value.from() {
        let omitted = match from {
            CssAngleOrZero::Zero(_) => true,
            CssAngleOrZero::Angle(angle) => angle.literal().is_some_and(|literal| {
                crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation()).len
                    == 0
            }),
        };
        if omitted {
            writer.without_output(|writer| {
                from.append_specified(&mut writer.context, &mut writer.css)
            })?;
        } else {
            writer.append("from ")?;
            from.append_specified(&mut writer.context, &mut writer.css)?;
            emitted = true;
        }
    }
    if let Some(position) = value.position() {
        if radial_position_is_center(position) {
            writer.without_output(|writer| position.append_specified(writer))?;
        } else {
            if emitted {
                writer.append(" ")?;
            }
            writer.append("at ")?;
            position.append_specified(writer)?;
            emitted = true;
        }
    }
    let (default, stops) =
        writer.capture_value(|writer| append_angular_stops(value.stops(), writer))?;
    append_interpolation(value.interpolation(), default, &mut emitted, writer)?;
    if emitted {
        writer.append(", ")?;
    }
    writer.append(&stops)
}

fn append_angular_stops(
    stops: &crate::CssAngularColorStopList,
    writer: &mut SpecifiedRuleWriter,
) -> Result<GradientDefault> {
    let mut default = GradientDefault::new();
    writer.node()?;
    let last = stops.items().len() - 1;
    for (index, item) in stops.items().iter().enumerate() {
        if index != 0 {
            writer.append(", ")?;
        }
        writer.node()?;
        match item {
            crate::CssAngularColorStopListItem::Stop(stop) => {
                stop.color()
                    .append_specified(&mut writer.context, &mut writer.css)?;
                default.observe(stop.color());
                for position in stop.positions() {
                    let omit = stop.positions().len() == 1
                        && ((index == 0 && angular_endpoint_is(position, false))
                            || (last > 0 && index == last && angular_endpoint_is(position, true)));
                    if omit {
                        writer.without_output(|writer| position.append_specified(writer))?;
                    } else {
                        writer.append(" ")?;
                        position.append_specified(writer)?;
                    }
                }
            }
            crate::CssAngularColorStopListItem::Hint(hint) => hint.append_specified(writer)?,
        }
    }
    Ok(default)
}

fn angular_endpoint_is(value: &crate::CssAngularColorStopPosition, end: bool) -> bool {
    use crate::{CssAngleUnit as U, CssAngularColorStopPosition as P};
    match value {
        P::Zero(_) => !end,
        P::Angle(value) => value.literal().is_some_and(|literal| {
            let decimal =
                crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation());
            if !end {
                return decimal.is_zero();
            }
            let (digits, exponent) = match literal.unit() {
                U::Degrees => ("36", 1),
                U::Gradians => ("4", 2),
                U::Turns => ("1", 0),
                U::Radians => return false,
            };
            decimal.is_positive(digits, exponent)
        }),
        P::Percentage(value) => value.literal_component().is_some_and(|component| {
            let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number)) =
                component.view()
            else {
                return false;
            };
            let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
            if !end {
                return decimal.is_zero();
            }
            decimal.is_positive("1", 2)
        }),
        P::Calculation(_) => false,
    }
}

impl crate::CssAngularColorStopPosition {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Angle(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Percentage(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Zero(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::Calculation(value) => crate::numeric::project_specified_into(
                &value.expression,
                &mut writer.context,
                &mut writer.css,
            )
            .map(|_| ()),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for crate::CssAngularColorStopPosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(crate::CssAngularColorStopPosition);

fn is_direct_zero(value: &CssSpecifiedLengthPercentage) -> bool {
    value
        .literal_component()
        .is_some_and(|component| match component.view() {
            crate::CssComponentValueRef::Token(
                crate::CssValueTokenRef::Number(number)
                | crate::CssValueTokenRef::Percentage(number)
                | crate::CssValueTokenRef::Dimension { number, .. },
            ) => crate::exact_decimal::LexicalDecimal::new(number.representation()).is_zero(),
            _ => false,
        })
}

fn literal_percentage_is(
    value: &CssSpecifiedLengthPercentage,
    digits: &str,
    exponent: i128,
) -> bool {
    value.literal_component().is_some_and(|component| {
        let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number)) =
            component.view()
        else {
            return false;
        };
        crate::exact_decimal::LexicalDecimal::new(number.representation())
            .is_positive(digits, exponent)
    })
}
fn is_direct_hundred_percent(value: &CssSpecifiedLengthPercentage) -> bool {
    literal_percentage_is(value, "1", 2)
}
fn append_line_position(
    value: &CssSpecifiedLengthPercentage,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    value.append_specified(&mut writer.context, &mut writer.css)
}

#[cfg(test)]
mod filter_image_writer_tests {
    use super::*;
    use crate::{CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};
    use crate::{CssSerializationLimits as L, CssSpecifiedValueSerializationErrorKind as K};

    fn value(text: &str) -> CssFilterImage {
        let source = crate::parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
            crate::parse_component_values(text).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let CssKnownPropertyValueRef::BackgroundImage(images) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("background images")
        };
        let [CssImageValue::Filter(value)] = images.images().images() else {
            panic!("one filter image")
        };
        (**value).clone()
    }

    #[test]
    fn enclosing_prefix_and_later_sibling_share_nodes_and_utf8_bytes() {
        let first = value("filter('é',blur())");
        let second = value("filter(url(b),hue-rotate())");
        let first_before = first.clone();
        let second_before = second.clone();
        let expected = "images: filter(\"é\", blur()), filter(url(\"b\"), hue-rotate());";
        // Existing String filter4 + URL filter5, with no additional writer carrier tariff.
        for (limits, kind) in [
            (L::new(8, 9, expected.len()), K::InputNodeLimit),
            (L::new(9, 8, expected.len()), K::ProjectionNodeLimit),
            (L::new(9, 9, expected.len() - 1), K::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("images: ").unwrap();
            first.append_specified(&mut writer).unwrap();
            writer.append(", ").unwrap();
            let result = second
                .append_specified(&mut writer)
                .and_then(|()| writer.append(";"));
            assert_eq!(result.unwrap_err().kind(), kind);
            assert!(writer.css.starts_with("images: filter(\"é\", blur()), "));
            assert_eq!(first, first_before);
            assert_eq!(second, second_before);
        }
        for _ in 0..2 {
            let mut writer = SpecifiedRuleWriter::new(L::new(9, 9, expected.len()));
            writer.append("images: ").unwrap();
            first.append_specified(&mut writer).unwrap();
            writer.append(", ").unwrap();
            second.append_specified(&mut writer).unwrap();
            writer.append(";").unwrap();
            assert_eq!(writer.css, expected);
        }
    }

    #[test]
    fn gradient_suppression_inside_filter_restores_enclosing_suppression_and_visits_defaults() {
        let filter = value("filter(linear-gradient(180deg,red 0%,blue 100%),blur())");
        let original = filter.clone();
        let mut writer = SpecifiedRuleWriter::new(L::new(13, 13, 4));
        writer
            .without_output(|writer| {
                filter.append_specified(writer)?;
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        // Suppression does not refund the complete 13-node graph visit.
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            K::InputNodeLimit
        );
        writer.append("kept").unwrap();
        assert_eq!(writer.css, "kept");
        assert_eq!(filter, original);
    }

    #[test]
    fn suppressed_child_projection_error_restores_mode_and_does_not_refund_visited_input() {
        let filter = value("filter('a',blur(calc(1px + 2em)))");
        let before = filter.clone();
        // Filter1 + String1 + list1 + blur/math5 input, 9 projection.
        let mut writer = SpecifiedRuleWriter::new(L::new(8, 8, 4));
        let error = writer
            .without_output(|writer| filter.append_specified(writer))
            .unwrap_err();
        assert_eq!(error.kind(), K::ProjectionNodeLimit);
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            K::InputNodeLimit
        );
        writer.append("kept").unwrap();
        assert_eq!(writer.css, "kept");
        assert_eq!(filter, before);
        assert_eq!(
            filter.serialize_specified(L::new(8, 9, 64)).unwrap(),
            "filter(\"a\", blur(calc(2em + 1px)))"
        );
    }

    #[test]
    fn suppression_failure_restores_outer_mode_before_final_emission() {
        let filter = value("filter(linear-gradient(180deg,red 0%,blue 100%),blur())");
        let mut writer = SpecifiedRuleWriter::new(L::new(12, 13, 4));
        writer
            .without_output(|writer| {
                let error = writer
                    .without_output(|writer| filter.append_specified(writer))
                    .unwrap_err();
                assert_eq!(error.kind(), K::InputNodeLimit);
                assert!(writer.context.output_suppressed());
                writer.append("discarded")
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        writer.append("kept").unwrap();
        assert_eq!(writer.css, "kept");
    }

    #[test]
    fn projected_child_byte_failure_cannot_reset_the_enclosing_prefix_budget() {
        let filter = value("filter('a',blur(calc(1px + 2em)))");
        let expected = "x:filter(\"a\", blur(calc(2em + 1px)))";
        let before = filter.clone();
        let mut writer = SpecifiedRuleWriter::new(L::new(8, 9, expected.len() - 1));
        writer.append("x:").unwrap();
        assert_eq!(
            filter.append_specified(&mut writer).unwrap_err().kind(),
            K::ByteLimit
        );
        assert!(writer.css.starts_with("x:"));
        let mut retry = SpecifiedRuleWriter::new(L::new(8, 9, expected.len()));
        retry.append("x:").unwrap();
        filter.append_specified(&mut retry).unwrap();
        assert_eq!(retry.css, expected);
        assert_eq!(filter, before);
    }
}
