//! Context forwarding for the existing structural relations reached by CSSOM
//! inverse composites. This consumer preserves authored branches and field
//! order; the numeric owner alone schedules and admits graph work slots.

use super::*;

pub(crate) trait BoundedStructuralEquality {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult;
}

impl<T: BoundedStructuralEquality + ?Sized> BoundedStructuralEquality for &T {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        T::bounded_eq(*self, *other, context)
    }
}
impl<T: BoundedStructuralEquality + ?Sized> BoundedStructuralEquality for Box<T> {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        self.as_ref().bounded_eq(other.as_ref(), context)
    }
}
impl<T: BoundedStructuralEquality> BoundedStructuralEquality for Option<T> {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Some(a), Some(b)) => a.bounded_eq(b, context),
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }
}
impl<T: BoundedStructuralEquality> BoundedStructuralEquality for [T] {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        if self.len() != other.len() {
            return Ok(false);
        }
        for (a, b) in self.iter().zip(other) {
            if !a.bounded_eq(b, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn token_equal(a: &CssComponentValue, b: &CssComponentValue) -> bool {
    a.token_structural_eq_ignoring_origin(b)
        .expect("checked primitive numeric token")
}
macro_rules! numeric_equality {
    ($($ty:ty),+ $(,)?) => { $(impl BoundedStructuralEquality for $ty {
        fn bounded_eq(&self, other: &Self, context: &mut SpecifiedSerializationContext) -> EqualityResult {
            match (self.literal_component(), other.literal_component()) {
                (Some(a), Some(b)) => Ok(token_equal(a, b)),
                (None, None) => self.calculation().expect("checked math").expression
                    .specified_inverse_eq(&other.calculation().expect("checked math").expression, context),
                _ => Ok(false),
            }
        }
    })+ };
}
numeric_equality!(
    CssSpecifiedNumber,
    CssSpecifiedNonNegativeNumber,
    CssSpecifiedNonNegativePercentage,
    CssSpecifiedNonNegativeFlex,
    CssFontObliqueAngle,
    CssFontWeightNumber
);
macro_rules! length_equality {
    ($($ty:ty),+ $(,)?) => { $(impl BoundedStructuralEquality for $ty {
        fn bounded_eq(&self, other: &Self, context: &mut SpecifiedSerializationContext) -> EqualityResult {
            if self.is_quirky_length() != other.is_quirky_length() { return Ok(false); }
            match (self.literal_component(), other.literal_component()) {
                (Some(a), Some(b)) => Ok(token_equal(a, b)),
                (None, None) => self.calculation().expect("checked math").expression
                    .specified_inverse_eq(&other.calculation().expect("checked math").expression, context),
                _ => Ok(false),
            }
        }
    })+ };
}
length_equality!(
    CssSpecifiedLength,
    CssSpecifiedNonNegativeLength,
    CssSpecifiedLengthPercentage,
    CssSpecifiedNonNegativeLengthPercentage
);

impl BoundedStructuralEquality for CssBoxSize {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::LengthPercentage(a), Self::LengthPercentage(b))
            | (Self::FitContentFunction(a), Self::FitContentFunction(b)) => {
                // The size owner compares retained tokens and expression
                // structure, rather than normalized numeric coefficients.
                match (a.literal_component(), b.literal_component()) {
                    (Some(a), Some(b)) => Ok(token_equal(a, b)),
                    (None, None) => a
                        .calculation()
                        .expect("checked size math")
                        .expression
                        .specified_inverse_eq(
                            &b.calculation().expect("checked size math").expression,
                            context,
                        ),
                    _ => Ok(false),
                }
            }
            (Self::CalcSize(a), Self::CalcSize(b)) => a
                .as_calc_size()
                .specified_inverse_eq(b.as_calc_size(), context),
            (Self::Stretch, Self::Stretch)
            | (Self::Contain, Self::Contain)
            | (Self::MinContent, Self::MinContent)
            | (Self::MaxContent, Self::MaxContent)
            | (Self::FitContent, Self::FitContent) => Ok(true),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssSizeValue {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Auto, Self::Auto) => Ok(true),
            (Self::BoxSize(a), Self::BoxSize(b)) => a.bounded_eq(b, context),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssFlexBasisValue {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (CssFlexBasisRef::Content, CssFlexBasisRef::Content) => Ok(true),
            (CssFlexBasisRef::Size(a), CssFlexBasisRef::Size(b)) => a.bounded_eq(b, context),
            (CssFlexBasisRef::CalcSize(a), CssFlexBasisRef::CalcSize(b)) => {
                a.specified_inverse_eq(b, context)
            }
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssHintedNumberCalculation {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        self.expression
            .specified_inverse_eq(&other.expression, context)
    }
}
impl BoundedStructuralEquality for CssAngleValue {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.literal(), other.literal()) {
            (Some(a), Some(b)) => {
                Ok(a.unit() == b.unit() && token_equal(a.component(), b.component()))
            }
            (None, None) => self
                .calculation()
                .expect("checked angle math")
                .expression
                .specified_inverse_eq(
                    &other.calculation().expect("checked angle math").expression,
                    context,
                ),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssAngleOrZero {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Angle(a), Self::Angle(b)) => a.bounded_eq(b, context),
            (Self::Zero(a), Self::Zero(b)) => Ok(token_equal(a.component(), b.component())),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssPositiveIntegerValue {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Literal(a), Self::Literal(b)) => Ok(token_equal(
                a.integer().component(),
                b.integer().component(),
            )),
            (Self::Calculation(a), Self::Calculation(b)) => {
                a.expression.specified_inverse_eq(&b.expression, context)
            }
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssGridLine {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Indexed(a), Self::Indexed(b)) => {
                if a.name() != b.name() {
                    return Ok(false);
                }
                match (a.value(), b.value()) {
                    (CssIntegerValue::Literal(a), CssIntegerValue::Literal(b)) => {
                        Ok(token_equal(a.component(), b.component()))
                    }
                    (CssIntegerValue::Calculation(a), CssIntegerValue::Calculation(b)) => {
                        a.expression.specified_inverse_eq(&b.expression, context)
                    }
                    _ => Ok(false),
                }
            }
            (Self::Span(a), Self::Span(b)) => {
                Ok(a.integer().bounded_eq(&b.integer(), context)? && a.name() == b.name())
            }
            (Self::Auto, Self::Auto) => Ok(true),
            (Self::Name(a), Self::Name(b)) => Ok(a == b),
            _ => Ok(false),
        }
    }
}

macro_rules! getter_compare {
    (bounded, $a:expr, $b:expr, $context:ident) => {
        BoundedStructuralEquality::bounded_eq(&$a, &$b, $context)?
    };
    (raw, $a:expr, $b:expr, $context:ident) => {
        $a == $b
    };
}
macro_rules! getter_equality {
    ($ty:ty; $($getter:ident: $relation:ident),+ $(,)?) => {
        impl BoundedStructuralEquality for $ty {
            fn bounded_eq(&self, other: &Self, context: &mut SpecifiedSerializationContext) -> EqualityResult {
                Ok(true $( && getter_compare!($relation, self.$getter(), other.$getter(), context) )+)
            }
        }
    };
}
macro_rules! branch_equality {
    ($ty:ty; $($variant:ident),+ $(,)?) => {
        impl BoundedStructuralEquality for $ty {
            fn bounded_eq(&self, other: &Self, context: &mut SpecifiedSerializationContext) -> EqualityResult {
                match (self, other) {
                    $((Self::$variant(a), Self::$variant(b)) => a.bounded_eq(b, context),)+
                    _ => Ok(self == other),
                }
            }
        }
    };
}
// Grid borrows exact primitive tokens and forwards reached math to its owner.
// Finite variants, list lengths and ordered fields stop before later children.
impl BoundedStructuralEquality for CssGridTrackBreadth {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        if self.kind() != other.kind() {
            return Ok(false);
        }
        match self.kind() {
            CssGridTrackBreadthKind::Length => self
                .length_percentage()
                .expect("checked length breadth")
                .bounded_eq(
                    other.length_percentage().expect("matching length breadth"),
                    context,
                ),
            CssGridTrackBreadthKind::Fraction => self
                .flex()
                .expect("checked flex breadth")
                .bounded_eq(other.flex().expect("matching flex breadth"), context),
            CssGridTrackBreadthKind::MinContent
            | CssGridTrackBreadthKind::MaxContent
            | CssGridTrackBreadthKind::Auto => Ok(true),
        }
    }
}
impl BoundedStructuralEquality for CssGridTrackSize {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        if self.kind() != other.kind() {
            return Ok(false);
        }
        match self.kind() {
            CssGridTrackSizeKind::Breadth => self
                .breadth()
                .expect("checked track breadth")
                .bounded_eq(other.breadth().expect("matching track breadth"), context),
            CssGridTrackSizeKind::MinMax => {
                let (min, max) = self.minmax().expect("checked minmax track");
                let (other_min, other_max) = other.minmax().expect("matching minmax track");
                Ok(min.bounded_eq(other_min, context)? && max.bounded_eq(other_max, context)?)
            }
            CssGridTrackSizeKind::FitContent => self
                .fit_content()
                .expect("checked fit-content track")
                .bounded_eq(
                    other.fit_content().expect("matching fit-content track"),
                    context,
                ),
        }
    }
}
getter_equality!(CssGridFixedSize; size: bounded);
branch_equality!(CssGridTrackRepeatComponent; TrackSize);
branch_equality!(CssGridFixedRepeatComponent; FixedSize);
getter_equality!(CssGridTrackRepeatContent; components: bounded);
getter_equality!(CssGridFixedRepeatContent; components: bounded);
getter_equality!(CssGridIntegerTrackRepeat; count: bounded, content: bounded);
getter_equality!(CssGridIntegerFixedRepeat; count: bounded, content: bounded);
getter_equality!(CssGridAutoRepeat; kind: raw, content: bounded);
getter_equality!(CssGridNameRepeat; count: bounded, groups: raw);
branch_equality!(CssGridGeneralTrackComponent; TrackSize, Repeat);
branch_equality!(CssGridAutoTrackComponent; FixedSize, Repeat, AutoRepeat);
branch_equality!(CssGridSubgridComponent; Repeat);
getter_equality!(CssGridGeneralTrackList; components: bounded);
getter_equality!(CssGridAutoTrackList; components: bounded);
getter_equality!(CssGridTrackSizeList; sizes: bounded);
impl BoundedStructuralEquality for CssGridTrackList {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        if self.is_none() || other.is_none() {
            return Ok(self.is_none() == other.is_none());
        }
        if let Some(left) = self.general_list() {
            return match other.general_list() {
                Some(right) => left.bounded_eq(right, context),
                None => Ok(false),
            };
        }
        if let Some(left) = self.auto_list() {
            return match other.auto_list() {
                Some(right) => left.bounded_eq(right, context),
                None => Ok(false),
            };
        }
        match (self.subgrid_components(), other.subgrid_components()) {
            (Some(left), Some(right)) => left.bounded_eq(right, context),
            _ => Ok(false),
        }
    }
}

branch_equality!(CssFontSize; LengthPercentage);
branch_equality!(CssFontWeight; Absolute);
branch_equality!(CssAbsoluteFontWeight; Number);
branch_equality!(CssColumnCount; Count);
branch_equality!(CssLineHeight; Number, HintedNumberCalculation, LengthPercentage);
branch_equality!(CssTextDecorationThickness; Length);
branch_equality!(CssOutlineWidth; Length);
impl BoundedStructuralEquality for CssFontStyle {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Oblique { angle: a }, Self::Oblique { angle: b }) => a.bounded_eq(b, context),
            _ => Ok(self == other),
        }
    }
}
impl BoundedStructuralEquality for CssFlowTolerance {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.as_ref(), other.as_ref()) {
            (
                CssFlowToleranceRef::LengthPercentage(a),
                CssFlowToleranceRef::LengthPercentage(b),
            ) => a.bounded_eq(b, context),
            _ => Ok(self == other),
        }
    }
}
getter_equality!(CssCubicBezierX; value: bounded);
// The owning numeric_fields_eq compares y1/y2 before the x-coordinate carriers.
getter_equality!(CssCubicBezier; y1: bounded, y2: bounded, x1: bounded, x2: bounded);
getter_equality!(CssSteps; count: bounded, position: raw);
branch_equality!(CssEasing; CubicBezier, Steps);
getter_equality!(CssEasingList; values: bounded);

branch_equality!(CssHorizontalPosition; Offset, LeftOffset, RightOffset, XStartOffset, XEndOffset);
branch_equality!(CssVerticalPosition; Offset, TopOffset, BottomOffset, YStartOffset, YEndOffset);
branch_equality!(CssBlockPosition; StartOffset, EndOffset);
branch_equality!(CssInlinePosition; StartOffset, EndOffset);
branch_equality!(CssRelativeAxisPosition; StartOffset, EndOffset);
getter_equality!(CssCartesianPosition; horizontal: bounded, vertical: bounded);
getter_equality!(CssPhysicalPosition; horizontal: bounded, vertical: bounded);
getter_equality!(CssBackgroundPosition; horizontal: bounded, vertical: bounded);
getter_equality!(CssNamedFlowPosition; block: bounded, inline: bounded);
getter_equality!(CssRelativeFlowPosition; block: bounded, inline: bounded);
getter_equality!(CssPhysicalPositionList; positions: bounded);
getter_equality!(CssBackgroundPositionList; positions: bounded);
impl BoundedStructuralEquality for CssPosition {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (CssPositionRef::Cartesian(a), CssPositionRef::Cartesian(b)) => {
                a.bounded_eq(b, context)
            }
            (CssPositionRef::NamedFlow(a), CssPositionRef::NamedFlow(b)) => {
                a.bounded_eq(b, context)
            }
            (CssPositionRef::RelativeFlow(a), CssPositionRef::RelativeFlow(b)) => {
                a.bounded_eq(b, context)
            }
            _ => Ok(false),
        }
    }
}

getter_equality!(CssImageValueList; images: bounded);
getter_equality!(CssImage; value: bounded);
impl BoundedStructuralEquality for CssImageValue {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::Gradient(a), Self::Gradient(b)) => a.bounded_eq(b, context),
            (Self::LightDark(a), Self::LightDark(b)) => a.specified_inverse_eq(b, context),
            (Self::Filter(a), Self::Filter(b)) => a.specified_inverse_eq(b, context),
            _ => Ok(self == other),
        }
    }
}
impl BoundedStructuralEquality for CssFilterImageInput {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (CssFilterImageInputRef::Image(a), CssFilterImageInputRef::Image(b)) => {
                a.bounded_eq(b, context)
            }
            // String Eq retains its exact existing authored token/provenance policy.
            (CssFilterImageInputRef::String(a), CssFilterImageInputRef::String(b)) => Ok(a == b),
            _ => Ok(false),
        }
    }
}
branch_equality!(CssGradient; Linear, Radial, RepeatingLinear, RepeatingRadial);
branch_equality!(CssLinearGradientDirection; Angle);
impl BoundedStructuralEquality for CssColor {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        self.specified_inverse_eq(other, context)
    }
}
getter_equality!(CssGradientColorStop; position: bounded, color: bounded);
branch_equality!(CssColorStopListItem; Stop, Hint);
getter_equality!(CssColorStopList; items: bounded);
getter_equality!(CssLinearGradient; direction: bounded, stops: bounded);
getter_equality!(CssRadialEllipseSize; horizontal: bounded, vertical: bounded);
branch_equality!(CssRadialSize; Circle, Ellipse);
getter_equality!(CssRadialGradient; shape: raw, size: bounded, position: bounded, stops: bounded);
branch_equality!(CssFilterAmount; Number, HintedNumberCalculation, Percentage);
impl BoundedStructuralEquality for CssFilterBlur {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        Ok(self.length().bounded_eq(other.length(), context)?
            && self.authored_length().is_none() == other.authored_length().is_none())
    }
}
impl BoundedStructuralEquality for CssFilterHueRotate {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        Ok(self.angle().bounded_eq(other.angle(), context)?
            && self.authored_angle().is_none() == other.authored_angle().is_none())
    }
}
getter_equality!(CssDropShadow; offset_x: bounded, offset_y: bounded, standard_deviation: bounded, color: bounded);
branch_equality!(CssFilterFunction; Blur, Brightness, Contrast, DropShadow, Grayscale, HueRotate, Invert, Opacity, Saturate, Sepia);
getter_equality!(CssFilterFunctionList; functions: bounded);

branch_equality!(CssListStyleTypeValue; CounterStyle);
branch_equality!(CssCounterStyleValue; Symbols);
getter_equality!(CssSymbolsStyleValue; system: raw, symbols: bounded);
branch_equality!(CssCounterSymbolValue; Image);

branch_equality!(CssOffsetPosition; Position);
branch_equality!(CssOffsetAnchor; Position);
getter_equality!(CssOffsetRotate; angle: bounded, modifier: raw);
getter_equality!(CssRay; angle: bounded, size: raw, contain: raw, position: bounded);
branch_equality!(CssOffsetPathKind; Ray, BasicShape);
getter_equality!(CssOffsetPathValue; path: bounded, coord_box: raw);
impl BoundedStructuralEquality for CssOffsetPath {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (CssOffsetPathRef::Path(a), CssOffsetPathRef::Path(b)) => a.bounded_eq(b, context),
            _ => Ok(self == other),
        }
    }
}
branch_equality!(CssCircleRadius; LengthPercentage);
branch_equality!(CssEllipseRadius; LengthPercentage);
getter_equality!(CssEllipseRadii; horizontal: bounded, vertical: bounded);
getter_equality!(CssCircleShape; radius: bounded, position: bounded);
getter_equality!(CssEllipseShape; radii: bounded, position: bounded);
getter_equality!(CssInsetShapeOffsets; values: bounded);
getter_equality!(CssInsetShape; offsets: bounded, round: bounded);
getter_equality!(CssPolygonPoint; x: bounded, y: bounded);
getter_equality!(CssPolygonPointList; points: bounded);
getter_equality!(CssPolygonShape; round: bounded, fill_rule: raw, points: bounded);
branch_equality!(CssRectShapeEdge; LengthPercentage);
getter_equality!(CssRectShape; top: bounded, right: bounded, bottom: bounded, left: bounded, round: bounded);
getter_equality!(CssXywhShape; x: bounded, y: bounded, width: bounded, height: bounded, round: bounded);
getter_equality!(CssBorderRadiusShorthand; horizontal_values: bounded, authored_vertical_values: bounded);
branch_equality!(CssBasicShape; Shape, Inset, Circle, Ellipse, Polygon, Rect, Xywh);
getter_equality!(CssShapeFunction; fill_rule: raw, start: bounded, commands: bounded);
getter_equality!(CssShapeCommandList; commands: bounded);
getter_equality!(CssShapeCoordinatePair; x: bounded, y: bounded);
branch_equality!(CssShapeEndpoint; To, By);
branch_equality!(CssShapeCommand; Move, Line, HorizontalLine, VerticalLine, Curve, Smooth, Arc);
branch_equality!(CssShapeHorizontalLine; ToOffset, By);
branch_equality!(CssShapeVerticalLine; ToOffset, By);
getter_equality!(CssShapeRelativeControlPoint; offset: bounded, anchor: raw);
impl BoundedStructuralEquality for CssShapeAbsoluteControlPoint {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (
                CssShapeAbsoluteControlPointRef::Position(a),
                CssShapeAbsoluteControlPointRef::Position(b),
            ) => a.bounded_eq(b, context),
            (
                CssShapeAbsoluteControlPointRef::Coordinates {
                    offset: a,
                    anchor: aa,
                },
                CssShapeAbsoluteControlPointRef::Coordinates {
                    offset: b,
                    anchor: ba,
                },
            ) => Ok(a.bounded_eq(b, context)? && aa == ba),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssShapeCurve {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (
                CssShapeCurveRef::To {
                    end: a,
                    first: af,
                    second: as_,
                },
                CssShapeCurveRef::To {
                    end: b,
                    first: bf,
                    second: bs,
                },
            ) => Ok(a.bounded_eq(b, context)?
                && af.bounded_eq(bf, context)?
                && as_.bounded_eq(&bs, context)?),
            (
                CssShapeCurveRef::By {
                    end: a,
                    first: af,
                    second: as_,
                },
                CssShapeCurveRef::By {
                    end: b,
                    first: bf,
                    second: bs,
                },
            ) => Ok(a.bounded_eq(b, context)?
                && af.bounded_eq(bf, context)?
                && as_.bounded_eq(&bs, context)?),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssShapeSmooth {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self.view(), other.view()) {
            (
                CssShapeSmoothRef::To {
                    end: a,
                    control: ac,
                },
                CssShapeSmoothRef::To {
                    end: b,
                    control: bc,
                },
            ) => Ok(a.bounded_eq(b, context)? && ac.bounded_eq(&bc, context)?),
            (
                CssShapeSmoothRef::By {
                    end: a,
                    control: ac,
                },
                CssShapeSmoothRef::By {
                    end: b,
                    control: bc,
                },
            ) => Ok(a.bounded_eq(b, context)? && ac.bounded_eq(&bc, context)?),
            _ => Ok(false),
        }
    }
}
impl BoundedStructuralEquality for CssShapeArcRadii {
    fn bounded_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> EqualityResult {
        match (self, other) {
            (Self::One(a), Self::One(b)) => a.bounded_eq(b, context),
            (
                Self::Two {
                    horizontal: a,
                    vertical: av,
                },
                Self::Two {
                    horizontal: b,
                    vertical: bv,
                },
            ) => Ok(a.bounded_eq(b, context)? && av.bounded_eq(bv, context)?),
            _ => Ok(false),
        }
    }
}
getter_equality!(CssShapeArc; endpoint: bounded, radii: bounded, sweep: raw, size: raw, rotation: bounded);
