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

    #[must_use]
    pub const fn edge(&self) -> CssBoxEdgeKeyword {
        self.edge
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransformFunctionKind {
    Matrix,
    Matrix3d,
    Perspective,
    Rotate,
    Rotate3d,
    RotateX,
    RotateY,
    RotateZ,
    Scale,
    Scale3d,
    ScaleX,
    ScaleY,
    ScaleZ,
    Skew,
    SkewX,
    SkewY,
    Translate,
    Translate3d,
    TranslateX,
    TranslateY,
    TranslateZ,
}

/// One authored operand in a transform scale function.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformScaleComponent {
    Number(CssNumeric<CssNumber>),
    /// A Number result whose dimensional percentage basis remains unresolved.
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
    Percentage(CssNumeric<CssPercentage>),
}

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
    pub const fn new(components: [CssNumeric<CssNumber>; 6]) -> Self {
        Self { components }
    }

    #[must_use]
    pub const fn components(&self) -> &[CssNumeric<CssNumber>; 6] {
        &self.components
    }
}

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
    pub const fn new(components: [CssNumeric<CssNumber>; 16]) -> Self {
        Self { components }
    }

    #[must_use]
    pub const fn components(&self) -> &[CssNumeric<CssNumber>; 16] {
        &self.components
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformPerspective {
    None,
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

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformRotate3d {
    x: CssNumeric<CssNumber>,
    y: CssNumeric<CssNumber>,
    z: CssNumeric<CssNumber>,
    angle: CssAngleOrZero,
}

impl CssTransformRotate3d {
    pub const fn new(
        x: CssNumeric<CssNumber>,
        y: CssNumeric<CssNumber>,
        z: CssNumeric<CssNumber>,
        angle: CssAngleOrZero,
    ) -> Self {
        Self { x, y, z, angle }
    }

    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssNumber> {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssNumeric<CssNumber> {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssNumeric<CssNumber> {
        &self.z
    }

    #[must_use]
    pub const fn angle(&self) -> &CssAngleOrZero {
        &self.angle
    }
}

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
    pub const fn new(x: CssTransformScaleComponent, y: Option<CssTransformScaleComponent>) -> Self {
        Self { x, y }
    }
    #[must_use]
    pub const fn x(&self) -> &CssTransformScaleComponent {
        &self.x
    }
    #[must_use]
    pub const fn y(&self) -> Option<&CssTransformScaleComponent> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformScale3d {
    x: CssTransformScaleComponent,
    y: CssTransformScaleComponent,
    z: CssTransformScaleComponent,
}

impl CssTransformScale3d {
    pub const fn new(
        x: CssTransformScaleComponent,
        y: CssTransformScaleComponent,
        z: CssTransformScaleComponent,
    ) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(&self) -> &CssTransformScaleComponent {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssTransformScaleComponent {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssTransformScaleComponent {
        &self.z
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformSkew {
    x: CssAngleOrZero,
    y: Option<CssAngleOrZero>,
}

impl CssTransformSkew {
    pub const fn new(x: CssAngleOrZero, y: Option<CssAngleOrZero>) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssAngleOrZero {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssAngleOrZero> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformTranslate {
    x: CssNumeric<CssLengthPercentage>,
    y: Option<CssNumeric<CssLengthPercentage>>,
}

impl CssTransformTranslate {
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: Option<CssNumeric<CssLengthPercentage>>,
    ) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        self.y.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformTranslate3d {
    x: CssNumeric<CssLengthPercentage>,
    y: CssNumeric<CssLengthPercentage>,
    z: CssNumeric<CssLength>,
}

impl CssTransformTranslate3d {
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: CssNumeric<CssLengthPercentage>,
        z: CssNumeric<CssLength>,
    ) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.y
    }

    #[must_use]
    pub const fn z(&self) -> &CssNumeric<CssLength> {
        &self.z
    }
}

/// An authored transform function with an exact typed payload.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTransformFunction {
    Matrix(CssTransformMatrix),
    Matrix3d(Box<CssTransformMatrix3d>),
    Perspective(CssTransformPerspective),
    Rotate(CssAngleOrZero),
    Rotate3d(CssTransformRotate3d),
    RotateX(CssAngleOrZero),
    RotateY(CssAngleOrZero),
    RotateZ(CssAngleOrZero),
    Scale(CssTransformScale),
    Scale3d(CssTransformScale3d),
    ScaleX(CssTransformScaleComponent),
    ScaleY(CssTransformScaleComponent),
    ScaleZ(CssTransformScaleComponent),
    Skew(CssTransformSkew),
    SkewX(CssAngleOrZero),
    SkewY(CssAngleOrZero),
    Translate(CssTransformTranslate),
    Translate3d(CssTransformTranslate3d),
    TranslateX(CssNumeric<CssLengthPercentage>),
    TranslateY(CssNumeric<CssLengthPercentage>),
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
    #[must_use]
    pub fn try_new(functions: Vec<CssTransformFunction>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

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
    None,
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
    None,
    Values(CssTranslateValues),
}

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
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }
    pub const fn y(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        self.y.as_ref()
    }
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
    None,
    Values(CssScaleValues),
}

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
    None,
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
    Flat,
    Preserve3d,
}
/// Authored visibility of the back face, without executing an accumulated matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackfaceVisibility {
    Visible,
    Hidden,
}
