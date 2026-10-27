use crate::numeric::CssAngleOrZero;
use crate::numeric::{
    CssCalculation, CssHintedNumber, CssLength, CssLengthPercentage, CssNonNegative, CssNumber,
    CssNumeric, CssPercentage,
};
use crate::values::common::CssBoxEdgeKeyword;

/// A checked authored `transform-box` reference-box keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTransformBox {
    edge: CssBoxEdgeKeyword,
}

impl CssTransformBox {
    /// Returns a `transform-box` value, or `None` for `padding-box` and `margin-box`, which the property does not accept.
    #[must_use]
    pub const fn try_new(edge: CssBoxEdgeKeyword) -> Option<Self> {
        match edge {
            CssBoxEdgeKeyword::ContentBox
            | CssBoxEdgeKeyword::BorderBox
            | CssBoxEdgeKeyword::FillBox
            | CssBoxEdgeKeyword::StrokeBox
            | CssBoxEdgeKeyword::ViewBox => Some(Self { edge }),
            CssBoxEdgeKeyword::PaddingBox | CssBoxEdgeKeyword::MarginBox => None,
        }
    }

    /// Returns the underlying box edge keyword, which is never `padding-box` or `margin-box`.
    #[must_use]
    pub const fn edge(&self) -> CssBoxEdgeKeyword {
        self.edge
    }
}

/// The identity of an authored transform function, without its operands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransformFunctionKind {
    /// `matrix()`.
    Matrix,
    /// `matrix3d()`.
    Matrix3d,
    /// `perspective()`.
    Perspective,
    /// `rotate()`.
    Rotate,
    /// `rotate3d()`.
    Rotate3d,
    /// `rotateX()`.
    RotateX,
    /// `rotateY()`.
    RotateY,
    /// `rotateZ()`.
    RotateZ,
    /// `scale()`.
    Scale,
    /// `scale3d()`.
    Scale3d,
    /// `scaleX()`.
    ScaleX,
    /// `scaleY()`.
    ScaleY,
    /// `scaleZ()`.
    ScaleZ,
    /// `skew()`.
    Skew,
    /// `skewX()`.
    SkewX,
    /// `skewY()`.
    SkewY,
    /// `translate()`.
    Translate,
    /// `translate3d()`.
    Translate3d,
    /// `translateX()`.
    TranslateX,
    /// `translateY()`.
    TranslateY,
    /// `translateZ()`.
    TranslateZ,
}

/// One authored operand in a transform scale function.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformScaleComponent {
    /// A `<number>` scale factor.
    Number(CssNumeric<CssNumber>),
    /// A Number result whose dimensional percentage basis remains unresolved.
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
    /// A `<percentage>` scale factor.
    Percentage(CssNumeric<CssPercentage>),
}

/// The six numbers of a `matrix()` function, in authored order `a, b, c, d, e, f`.
#[derive(Clone, Debug)]
pub struct CssTransformMatrix {
    components: [CssNumeric<CssNumber>; 6],
}

impl PartialEq for CssTransformScaleComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left == right
            }
            (Self::Percentage(left), Self::Percentage(right)) => left == right,
            _ => false,
        }
    }
}

impl PartialEq for CssTransformMatrix {
    fn eq(&self, other: &Self) -> bool {
        self.components
            .iter()
            .zip(&other.components)
            .all(|(left, right)| left == right)
    }
}

impl CssTransformMatrix {
    /// Creates a matrix from its six authored numbers.
    pub const fn new(components: [CssNumeric<CssNumber>; 6]) -> Self {
        Self { components }
    }

    /// Returns the six authored numbers in authored order.
    #[must_use]
    pub const fn components(&self) -> &[CssNumeric<CssNumber>; 6] {
        &self.components
    }
}

/// The sixteen numbers of a `matrix3d()` function, in authored column-major order.
#[derive(Clone, Debug)]
pub struct CssTransformMatrix3d {
    components: [CssNumeric<CssNumber>; 16],
}

impl PartialEq for CssTransformMatrix3d {
    fn eq(&self, other: &Self) -> bool {
        self.components
            .iter()
            .zip(&other.components)
            .all(|(left, right)| left == right)
    }
}

impl CssTransformMatrix3d {
    /// Creates a matrix from its sixteen authored numbers.
    pub const fn new(components: [CssNumeric<CssNumber>; 16]) -> Self {
        Self { components }
    }

    /// Returns the sixteen authored numbers in authored order.
    #[must_use]
    pub const fn components(&self) -> &[CssNumeric<CssNumber>; 16] {
        &self.components
    }
}

/// The authored operand of a `perspective()` transform function.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformPerspective {
    /// `none`: no perspective.
    None,
    /// A non-negative `<length>` distance.
    Length(CssNumeric<CssLength, CssNonNegative>),
}

impl PartialEq for CssTransformPerspective {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Length(left), Self::Length(right)) => left == right,
            _ => false,
        }
    }
}

/// The authored operands of a `rotate3d()` function: three axis numbers and an angle.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformRotate3d {
    x: CssNumeric<CssNumber>,
    y: CssNumeric<CssNumber>,
    z: CssNumeric<CssNumber>,
    angle: CssAngleOrZero,
}

impl CssTransformRotate3d {
    /// Creates a value from the x, y and z axis components and the rotation angle.
    pub const fn new(
        x: CssNumeric<CssNumber>,
        y: CssNumeric<CssNumber>,
        z: CssNumeric<CssNumber>,
        angle: CssAngleOrZero,
    ) -> Self {
        Self { x, y, z, angle }
    }

    /// Returns the x component of the rotation axis.
    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssNumber> {
        &self.x
    }

    /// Returns the y component of the rotation axis.
    #[must_use]
    pub const fn y(&self) -> &CssNumeric<CssNumber> {
        &self.y
    }

    /// Returns the z component of the rotation axis.
    #[must_use]
    pub const fn z(&self) -> &CssNumeric<CssNumber> {
        &self.z
    }

    /// Returns the rotation angle.
    #[must_use]
    pub const fn angle(&self) -> &CssAngleOrZero {
        &self.angle
    }
}

/// The authored operands of a `scale()` function.
#[derive(Clone, Debug)]
pub struct CssTransformScale {
    x: CssTransformScaleComponent,
    y: Option<CssTransformScaleComponent>,
}

impl PartialEq for CssTransformScale {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl CssTransformScale {
    /// Creates a value from the x factor and an optional y factor.
    pub const fn new(x: CssTransformScaleComponent, y: Option<CssTransformScaleComponent>) -> Self {
        Self { x, y }
    }
    /// Returns the x factor.
    #[must_use]
    pub const fn x(&self) -> &CssTransformScaleComponent {
        &self.x
    }
    /// Returns the y factor, or `None` when it was omitted.
    #[must_use]
    pub const fn y(&self) -> Option<&CssTransformScaleComponent> {
        self.y.as_ref()
    }
}

/// The authored operands of a `scale3d()` function.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformScale3d {
    x: CssTransformScaleComponent,
    y: CssTransformScaleComponent,
    z: CssTransformScaleComponent,
}

impl CssTransformScale3d {
    /// Creates a value from the x, y and z factors.
    pub const fn new(
        x: CssTransformScaleComponent,
        y: CssTransformScaleComponent,
        z: CssTransformScaleComponent,
    ) -> Self {
        Self { x, y, z }
    }

    /// Returns the x factor.
    #[must_use]
    pub const fn x(&self) -> &CssTransformScaleComponent {
        &self.x
    }

    /// Returns the y factor.
    #[must_use]
    pub const fn y(&self) -> &CssTransformScaleComponent {
        &self.y
    }

    /// Returns the z factor.
    #[must_use]
    pub const fn z(&self) -> &CssTransformScaleComponent {
        &self.z
    }
}

/// The authored operands of a `skew()` function.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformSkew {
    x: CssAngleOrZero,
    y: Option<CssAngleOrZero>,
}

impl CssTransformSkew {
    /// Creates a value from the x angle and an optional y angle.
    pub const fn new(x: CssAngleOrZero, y: Option<CssAngleOrZero>) -> Self {
        Self { x, y }
    }

    /// Returns the x angle.
    #[must_use]
    pub const fn x(&self) -> &CssAngleOrZero {
        &self.x
    }

    /// Returns the y angle, or `None` when it was omitted.
    #[must_use]
    pub const fn y(&self) -> Option<&CssAngleOrZero> {
        self.y.as_ref()
    }
}

/// The authored operands of a `translate()` function.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformTranslate {
    x: CssNumeric<CssLengthPercentage>,
    y: Option<CssNumeric<CssLengthPercentage>>,
}

impl CssTransformTranslate {
    /// Creates a value from the x offset and an optional y offset.
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: Option<CssNumeric<CssLengthPercentage>>,
    ) -> Self {
        Self { x, y }
    }

    /// Returns the x offset.
    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }

    /// Returns the y offset, or `None` when it was omitted.
    #[must_use]
    pub const fn y(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        self.y.as_ref()
    }
}

/// The authored operands of a `translate3d()` function.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformTranslate3d {
    x: CssNumeric<CssLengthPercentage>,
    y: CssNumeric<CssLengthPercentage>,
    z: CssNumeric<CssLength>,
}

impl CssTransformTranslate3d {
    /// Creates a value from the x and y offsets and the z length.
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: CssNumeric<CssLengthPercentage>,
        z: CssNumeric<CssLength>,
    ) -> Self {
        Self { x, y, z }
    }

    /// Returns the x offset.
    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }

    /// Returns the y offset.
    #[must_use]
    pub const fn y(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.y
    }

    /// Returns the z length.
    #[must_use]
    pub const fn z(&self) -> &CssNumeric<CssLength> {
        &self.z
    }
}

/// An authored transform function with an exact typed payload.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformFunction {
    /// A `matrix()` function.
    Matrix(CssTransformMatrix),
    /// A `matrix3d()` function, boxed to keep the enum small.
    Matrix3d(Box<CssTransformMatrix3d>),
    /// A `perspective()` function.
    Perspective(CssTransformPerspective),
    /// A `rotate()` function with its angle.
    Rotate(CssAngleOrZero),
    /// A `rotate3d()` function.
    Rotate3d(CssTransformRotate3d),
    /// A `rotateX()` function with its angle.
    RotateX(CssAngleOrZero),
    /// A `rotateY()` function with its angle.
    RotateY(CssAngleOrZero),
    /// A `rotateZ()` function with its angle.
    RotateZ(CssAngleOrZero),
    /// A `scale()` function.
    Scale(CssTransformScale),
    /// A `scale3d()` function.
    Scale3d(CssTransformScale3d),
    /// A `scaleX()` function with its factor.
    ScaleX(CssTransformScaleComponent),
    /// A `scaleY()` function with its factor.
    ScaleY(CssTransformScaleComponent),
    /// A `scaleZ()` function with its factor.
    ScaleZ(CssTransformScaleComponent),
    /// A `skew()` function.
    Skew(CssTransformSkew),
    /// A `skewX()` function with its angle.
    SkewX(CssAngleOrZero),
    /// A `skewY()` function with its angle.
    SkewY(CssAngleOrZero),
    /// A `translate()` function.
    Translate(CssTransformTranslate),
    /// A `translate3d()` function.
    Translate3d(CssTransformTranslate3d),
    /// A `translateX()` function with its offset.
    TranslateX(CssNumeric<CssLengthPercentage>),
    /// A `translateY()` function with its offset.
    TranslateY(CssNumeric<CssLengthPercentage>),
    /// A `translateZ()` function with its length.
    TranslateZ(CssNumeric<CssLength>),
}

impl PartialEq for CssTransformFunction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TranslateX(left), Self::TranslateX(right))
            | (Self::TranslateY(left), Self::TranslateY(right)) => left == right,
            (Self::TranslateZ(left), Self::TranslateZ(right)) => left == right,
            (Self::Matrix(left), Self::Matrix(right)) => left == right,
            (Self::Matrix3d(left), Self::Matrix3d(right)) => left == right,
            (Self::Perspective(left), Self::Perspective(right)) => left == right,
            (Self::Rotate(left), Self::Rotate(right))
            | (Self::RotateX(left), Self::RotateX(right))
            | (Self::RotateY(left), Self::RotateY(right))
            | (Self::RotateZ(left), Self::RotateZ(right)) => left == right,
            (Self::Rotate3d(left), Self::Rotate3d(right)) => left == right,
            (Self::Scale(left), Self::Scale(right)) => left == right,
            (Self::Scale3d(left), Self::Scale3d(right)) => left == right,
            (Self::ScaleX(left), Self::ScaleX(right))
            | (Self::ScaleY(left), Self::ScaleY(right)) => left == right,
            (Self::ScaleZ(left), Self::ScaleZ(right)) => left == right,
            (Self::Skew(left), Self::Skew(right)) => left == right,
            (Self::SkewX(left), Self::SkewX(right)) | (Self::SkewY(left), Self::SkewY(right)) => {
                left == right
            }
            (Self::Translate(left), Self::Translate(right)) => left == right,
            (Self::Translate3d(left), Self::Translate3d(right)) => left == right,
            _ => false,
        }
    }
}

impl CssTransformFunction {
    /// Returns the identity of this function, discarding its operands.
    #[must_use]
    pub const fn kind(&self) -> CssTransformFunctionKind {
        match self {
            Self::Matrix(_) => CssTransformFunctionKind::Matrix,
            Self::Matrix3d(_) => CssTransformFunctionKind::Matrix3d,
            Self::Perspective(_) => CssTransformFunctionKind::Perspective,
            Self::Rotate(_) => CssTransformFunctionKind::Rotate,
            Self::Rotate3d(_) => CssTransformFunctionKind::Rotate3d,
            Self::RotateX(_) => CssTransformFunctionKind::RotateX,
            Self::RotateY(_) => CssTransformFunctionKind::RotateY,
            Self::RotateZ(_) => CssTransformFunctionKind::RotateZ,
            Self::Scale(_) => CssTransformFunctionKind::Scale,
            Self::Scale3d(_) => CssTransformFunctionKind::Scale3d,
            Self::ScaleX(_) => CssTransformFunctionKind::ScaleX,
            Self::ScaleY(_) => CssTransformFunctionKind::ScaleY,
            Self::ScaleZ(_) => CssTransformFunctionKind::ScaleZ,
            Self::Skew(_) => CssTransformFunctionKind::Skew,
            Self::SkewX(_) => CssTransformFunctionKind::SkewX,
            Self::SkewY(_) => CssTransformFunctionKind::SkewY,
            Self::Translate(_) => CssTransformFunctionKind::Translate,
            Self::Translate3d(_) => CssTransformFunctionKind::Translate3d,
            Self::TranslateX(_) => CssTransformFunctionKind::TranslateX,
            Self::TranslateY(_) => CssTransformFunctionKind::TranslateY,
            Self::TranslateZ(_) => CssTransformFunctionKind::TranslateZ,
        }
    }
}

/// A non-empty ordered list of authored transform functions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformFunctionList {
    functions: Vec<CssTransformFunction>,
}

impl CssTransformFunctionList {
    /// Returns the list, or `None` when `functions` is empty.
    #[must_use]
    pub fn try_new(functions: Vec<CssTransformFunction>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

    /// Returns the authored functions in order.
    #[must_use]
    pub fn functions(&self) -> &[CssTransformFunction] {
        &self.functions
    }
}

/// The authored value of the `transform` property.
///
/// # Specified serialization
///
/// Emits canonical specified syntax from retained typed operands. Percentages and calculations
/// retain their owning specified policy without resolving reference boxes or matrices. Numeric
/// output retains six-place formatting. General rotate3d axes share the Rotate owner's
/// approximation and representability policy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTransform {
    /// `none`: no transform.
    None,
    /// One or more transform functions.
    Functions(CssTransformFunctionList),
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained typed operands. Percentages and calculations
/// retain their owning specified policy without resolving reference boxes or matrices. Numeric
/// output retains six-place formatting. General rotate3d axes share the Rotate owner's
/// approximation and representability policy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssTranslate {
    /// `none`: no translation.
    None,
    /// One to three translation components.
    Values(CssTranslateValues),
}

/// The authored components of the individual `translate` property.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTranslateValues {
    x: CssNumeric<CssLengthPercentage>,
    y: Option<CssNumeric<CssLengthPercentage>>,
    z: Option<CssNumeric<CssLength>>,
}

impl CssTranslateValues {
    /// Retains one to three consecutive axes; an authored Z requires an authored Y.
    pub fn try_new(
        x: CssNumeric<CssLengthPercentage>,
        y: Option<CssNumeric<CssLengthPercentage>>,
        z: Option<CssNumeric<CssLength>>,
    ) -> Option<Self> {
        if y.is_none() && z.is_some() {
            return None;
        }
        Some(Self { x, y, z })
    }
    /// Returns the x offset.
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }
    /// Returns the y offset, or `None` when it was omitted.
    pub const fn y(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        self.y.as_ref()
    }
    /// Returns the z length, or `None` when it was omitted.
    pub const fn z(&self) -> Option<&CssNumeric<CssLength>> {
        self.z.as_ref()
    }
}

/// # Specified serialization
///
/// Emits canonical specified syntax from retained typed operands. Percentages and calculations
/// retain their owning specified policy without resolving reference boxes or matrices. Numeric
/// output retains six-place formatting. General rotate3d axes share the Rotate owner's
/// approximation and representability policy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssScale {
    /// `none`: no scaling.
    None,
    /// One to three scale factors.
    Values(CssScaleValues),
}

/// The authored factors of the individual `scale` property, one to three in axis order.
#[derive(Clone, Debug)]
pub struct CssScaleValues {
    values: Vec<CssTransformScaleComponent>,
}
impl PartialEq for CssScaleValues {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl CssScaleValues {
    /// Retains one to three checked number/percentage factors in authored axis order.
    /// Symbolic number, percentage and hinted-number calculations remain unresolved.
    #[must_use]
    pub fn try_new(values: Vec<CssTransformScaleComponent>) -> Option<Self> {
        (1..=3).contains(&values.len()).then_some(Self { values })
    }
    /// Returns the authored factors in axis order.
    #[must_use]
    pub fn values(&self) -> &[CssTransformScaleComponent] {
        &self.values
    }
}

/// Authored perspective; zero is valid and the rendering floor is not applied here.
///
/// # Specified serialization
///
/// Emits canonical specified syntax from retained typed operands. Percentages and calculations
/// retain their owning specified policy without resolving reference boxes or matrices. Numeric
/// output retains six-place formatting. General rotate3d axes share the Rotate owner's
/// approximation and representability policy.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssPerspective {
    /// `none`: no perspective.
    None,
    /// A non-negative `<length>` distance.
    Length(CssNumeric<CssLength, CssNonNegative>),
}
impl PartialEq for CssPerspective {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Length(a), Self::Length(b)) => a == b,
            _ => false,
        }
    }
}
/// Authored transform-style, distinct from contextual used-value flattening.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransformStyle {
    /// `flat`: the initial value; a transformed element does not create a 3D rendering context
    /// and creates a flattened representation of its content.
    Flat,
    /// `preserve-3d`: overrides the flattening, letting descendants of the transformed element
    /// share the same 3D rendering context.
    Preserve3d,
}
/// Authored visibility of the back face, without executing an accumulated matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackfaceVisibility {
    /// `visible`: the element remains visible when its reverse side is towards the viewer.
    Visible,
    /// `hidden`: the element is invisible when its reverse side is towards the viewer.
    Hidden,
}
