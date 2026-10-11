//! Intrinsic area shape/coords author constraints and image-map coordinate
//! descriptors, from HTML #attr-area-shape, #attr-area-coords and
//! #image-map-processing-model. Coordinates are displayed-image CSS pixels;
//! image dimensions, association, layering and hit testing remain external.

use super::*;
use crate::numeric::{ScientificDecimalKeyError, strict_scientific_decimal_key_units};
use crate::{
    HtmlFloatingPointList, HtmlFloatingPointNumber, HtmlInteger, HtmlNonNegativeInteger,
    HtmlValueAuthorState, HtmlValueRecoveryState, HtmlValueReport,
};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

/// Conforming native shape keywords; recovery aliases are not native variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaShape {
    Circle,
    WholeImage,
    Polygon,
    Rectangle,
}
impl HtmlAreaShape {
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::WholeImage => "default",
            Self::Polygon => "poly",
            Self::Rectangle => "rect",
        }
    }
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Shape.qualified_name(),
            self.canonical_keyword(),
        )
    }
}
const SHAPE: EnumRule<HtmlAreaShape> = EnumRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::Shape,
        rows: &[142],
        source: source!("attr-area-shape"),
        context: Context::Elements(&[KnownHtmlElement::Area]),
        vocabulary_source: None,
    },
    keywords: &[
        Keyword {
            spelling: "circle",
            canonical: "circle",
            state: HtmlAreaShape::Circle,
            conforming: true,
        },
        Keyword {
            spelling: "circ",
            canonical: "circle",
            state: HtmlAreaShape::Circle,
            conforming: false,
        },
        Keyword {
            spelling: "default",
            canonical: "default",
            state: HtmlAreaShape::WholeImage,
            conforming: true,
        },
        Keyword {
            spelling: "poly",
            canonical: "poly",
            state: HtmlAreaShape::Polygon,
            conforming: true,
        },
        Keyword {
            spelling: "polygon",
            canonical: "poly",
            state: HtmlAreaShape::Polygon,
            conforming: false,
        },
        Keyword {
            spelling: "rect",
            canonical: "rect",
            state: HtmlAreaShape::Rectangle,
            conforming: true,
        },
        Keyword {
            spelling: "rectangle",
            canonical: "rect",
            state: HtmlAreaShape::Rectangle,
            conforming: false,
        },
    ],
    missing: Some(HtmlAreaShape::Rectangle),
    invalid: Some(HtmlAreaShape::Rectangle),
    empty: None,
    exact_keywords: false,
};

/// Exact authored coordinates, independent of binary64 processing precision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaIntegerPoint {
    x: HtmlInteger,
    y: HtmlInteger,
}
impl HtmlAreaIntegerPoint {
    pub fn new(x: HtmlInteger, y: HtmlInteger) -> Self {
        Self { x, y }
    }
    pub fn x(&self) -> &HtmlInteger {
        &self.x
    }
    pub fn y(&self) -> &HtmlInteger {
        &self.y
    }
}
/// An authored circle permits a zero radius; processing then yields Empty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaCircle {
    center: HtmlAreaIntegerPoint,
    radius: HtmlNonNegativeInteger,
}
impl HtmlAreaCircle {
    pub fn new(center: HtmlAreaIntegerPoint, radius: HtmlNonNegativeInteger) -> Self {
        Self { center, radius }
    }
    pub fn center(&self) -> &HtmlAreaIntegerPoint {
        &self.center
    }
    pub fn radius(&self) -> &HtmlNonNegativeInteger {
        &self.radius
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaRectangle {
    top_left: HtmlAreaIntegerPoint,
    bottom_right: HtmlAreaIntegerPoint,
}
impl HtmlAreaRectangle {
    pub fn new(
        top_left: HtmlAreaIntegerPoint,
        bottom_right: HtmlAreaIntegerPoint,
    ) -> Result<Self, HtmlAreaNativeError> {
        if top_left.x.compare(&bottom_right.x) != Ordering::Less {
            return Err(HtmlAreaNativeError::NonincreasingX);
        }
        if top_left.y.compare(&bottom_right.y) != Ordering::Less {
            return Err(HtmlAreaNativeError::NonincreasingY);
        }
        Ok(Self {
            top_left,
            bottom_right,
        })
    }
    pub fn top_left(&self) -> &HtmlAreaIntegerPoint {
        &self.top_left
    }
    pub fn bottom_right(&self) -> &HtmlAreaIntegerPoint {
        &self.bottom_right
    }
}
/// Ordered vertices; the source imposes no distinctness or convexity constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaPolygon {
    vertices: Vec<HtmlAreaIntegerPoint>,
}
impl HtmlAreaPolygon {
    pub fn new(vertices: Vec<HtmlAreaIntegerPoint>) -> Result<Self, HtmlAreaNativeError> {
        if vertices.len() < 3 {
            return Err(HtmlAreaNativeError::TooFewVertices);
        }
        Ok(Self { vertices })
    }
    pub fn vertices(&self) -> &[HtmlAreaIntegerPoint] {
        &self.vertices
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaNativeError {
    NonincreasingX,
    NonincreasingY,
    TooFewVertices,
}
impl fmt::Display for HtmlAreaNativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonincreasingX => "area rectangle x coordinates must increase",
            Self::NonincreasingY => "area rectangle y coordinates must increase",
            Self::TooFewVertices => "area polygon requires at least three vertices",
        })
    }
}
impl Error for HtmlAreaNativeError {}
/// Intrinsically conforming native geometry. WholeImage omits coords.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAuthoredArea {
    Circle(HtmlAreaCircle),
    Rectangle(HtmlAreaRectangle),
    Polygon(HtmlAreaPolygon),
    WholeImage,
}
impl HtmlAuthoredArea {
    pub fn shape(&self) -> HtmlAreaShape {
        match self {
            Self::Circle(_) => HtmlAreaShape::Circle,
            Self::Rectangle(_) => HtmlAreaShape::Rectangle,
            Self::Polygon(_) => HtmlAreaShape::Polygon,
            Self::WholeImage => HtmlAreaShape::WholeImage,
        }
    }
    /// Canonical exact integer spelling; no floating projection or reparse.
    pub fn into_attributes(self) -> (Attribute, Option<Attribute>) {
        let shape = self.shape().into_attribute();
        let mut units = Vec::new();
        let mut append = |value: DocumentString| {
            if !units.is_empty() {
                units.push(u16::from(b','));
            }
            units.extend_from_slice(value.code_units());
        };
        match self {
            Self::Circle(circle) => {
                append(circle.center.x.as_document_string());
                append(circle.center.y.as_document_string());
                append(circle.radius.as_document_string());
            }
            Self::Rectangle(rect) => {
                append(rect.top_left.x.as_document_string());
                append(rect.top_left.y.as_document_string());
                append(rect.bottom_right.x.as_document_string());
                append(rect.bottom_right.y.as_document_string());
            }
            Self::Polygon(poly) => {
                for point in poly.vertices {
                    append(point.x.as_document_string());
                    append(point.y.as_document_string());
                }
            }
            Self::WholeImage => return (shape, None),
        }
        (
            shape,
            Some(Attribute::new(
                KnownHtmlAttribute::Coords.qualified_name(),
                DocumentString::from_utf16(units),
            )),
        )
    }
}

/// Shape-specific author diagnostics, independent of common grammar evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaCoordinatesAuthorError {
    MissingRequired,
    ForbiddenPresence,
    InvalidListSyntax,
    Cardinality,
    NonintegerItem { index: usize },
    NegativeRadius,
    NonincreasingX,
    NonincreasingY,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaCoordinatesAuthorState {
    Unapplied,
    Conforming,
    Nonconforming(HtmlAreaCoordinatesAuthorError),
    /// The grammar was valid but checked exact-key representation exhausted.
    RepresentationUnavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaEmptyReason {
    TooFewCoordinates { required: usize, actual: usize },
    NonpositiveRadius,
}
/// A checked finite processing coordinate in displayed-image CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlAreaProcessedPoint {
    x: HtmlFloatingPointNumber,
    y: HtmlFloatingPointNumber,
}
impl HtmlAreaProcessedPoint {
    pub fn x(&self) -> HtmlFloatingPointNumber {
        self.x
    }
    pub fn y(&self) -> HtmlFloatingPointNumber {
        self.y
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlAreaProcessedCircle {
    center: HtmlAreaProcessedPoint,
    radius: HtmlFloatingPointNumber,
}
impl HtmlAreaProcessedCircle {
    pub fn center(&self) -> HtmlAreaProcessedPoint {
        self.center
    }
    pub fn radius(&self) -> HtmlFloatingPointNumber {
        self.radius
    }
}
/// Processing permits equal edges, unlike authored rectangle constraints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlAreaProcessedRectangle {
    top_left: HtmlAreaProcessedPoint,
    bottom_right: HtmlAreaProcessedPoint,
}
impl HtmlAreaProcessedRectangle {
    pub fn top_left(&self) -> HtmlAreaProcessedPoint {
        self.top_left
    }
    pub fn bottom_right(&self) -> HtmlAreaProcessedPoint {
        self.bottom_right
    }
}
/// Even prefix of the common recovery list, without duplicating its allocation.
/// Polygon interiors use the source's even-odd rule; no hit test is performed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlAreaProcessedPolygon<'a> {
    coordinates: &'a [HtmlFloatingPointNumber],
}
impl<'a> HtmlAreaProcessedPolygon<'a> {
    pub fn coordinates(&self) -> &'a [HtmlFloatingPointNumber] {
        self.coordinates
    }
    pub fn vertices(&self) -> impl ExactSizeIterator<Item = HtmlAreaProcessedPoint> + 'a {
        self.coordinates
            .chunks_exact(2)
            .map(|pair| HtmlAreaProcessedPoint {
                x: pair[0],
                y: pair[1],
            })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlAreaProcessedShape<'a> {
    Unapplied(HtmlAttributeApplicability),
    Empty(HtmlAreaEmptyReason),
    /// Symbolic complete image rectangle; dimensions are not guessed.
    WholeImage,
    Circle(HtmlAreaProcessedCircle),
    Rectangle(HtmlAreaProcessedRectangle),
    Polygon(HtmlAreaProcessedPolygon<'a>),
}
/// Exact supplied shape/coords records and independently qualified phases.
///
/// Raw values each obey the interpreter's per-value UTF-16 admission cap.
/// Native values, common reports and exact author keys use ordinary allocation;
/// this operation promises no controlled physical ledger or cancellation bound.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlAreaCoordinatesReport {
    shape: HtmlAttributeReport<HtmlAreaShape>,
    coords: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlFloatingPointList, HtmlFloatingPointList>>,
    author: HtmlAreaCoordinatesAuthorState,
}
impl HtmlAreaCoordinatesReport {
    pub fn shape(&self) -> &HtmlAttributeReport<HtmlAreaShape> {
        &self.shape
    }
    /// Exact coords metadata, presence and applicability. The envelope's
    /// keyword recognition/author fields are Unapplied: list grammar belongs
    /// to common(), and shape-specific requirements belong to author().
    pub fn coords(&self) -> &HtmlAttributeReport<()> {
        &self.coords
    }
    pub fn common(&self) -> Option<&HtmlValueReport<HtmlFloatingPointList, HtmlFloatingPointList>> {
        self.common.as_ref()
    }
    /// Library diagnostic precedence: presence, list grammar, cardinality,
    /// exact integrality, then radius/rectangle ordering. This is not a new
    /// normative processing algorithm or complete element conformance verdict.
    pub fn author(&self) -> HtmlAreaCoordinatesAuthorState {
        self.author
    }
    pub fn processing_source_clause(&self) -> &'static str {
        source!("image-map-processing-model")
    }
    pub fn processed_shape(&self) -> HtmlAreaProcessedShape<'_> {
        use HtmlAreaProcessedShape as Output;
        let Some(shape) = self.shape.intrinsic().copied() else {
            return Output::Unapplied(self.shape.applicability());
        };
        if shape == HtmlAreaShape::WholeImage {
            return Output::WholeImage;
        }
        let Some(common) = &self.common else {
            return Output::Unapplied(self.coords.applicability());
        };
        let values = match common.recovery() {
            HtmlValueRecoveryState::Value(list) => list.values(),
            _ => &[],
        };
        let required = match shape {
            HtmlAreaShape::Circle => 3,
            HtmlAreaShape::Polygon => 6,
            HtmlAreaShape::Rectangle => 4,
            HtmlAreaShape::WholeImage => 0,
        };
        if values.len() < required {
            return Output::Empty(HtmlAreaEmptyReason::TooFewCoordinates {
                required,
                actual: values.len(),
            });
        }
        match shape {
            HtmlAreaShape::Circle => {
                if values[2].value() <= 0.0 {
                    return Output::Empty(HtmlAreaEmptyReason::NonpositiveRadius);
                }
                Output::Circle(HtmlAreaProcessedCircle {
                    center: HtmlAreaProcessedPoint {
                        x: values[0],
                        y: values[1],
                    },
                    radius: values[2],
                })
            }
            HtmlAreaShape::Rectangle => {
                let (mut x1, mut x2) = (values[0], values[2]);
                let (mut y1, mut y2) = (values[1], values[3]);
                if x1.value() > x2.value() {
                    std::mem::swap(&mut x1, &mut x2);
                }
                if y1.value() > y2.value() {
                    std::mem::swap(&mut y1, &mut y2);
                }
                Output::Rectangle(HtmlAreaProcessedRectangle {
                    top_left: HtmlAreaProcessedPoint { x: x1, y: y1 },
                    bottom_right: HtmlAreaProcessedPoint { x: x2, y: y2 },
                })
            }
            HtmlAreaShape::Polygon => Output::Polygon(HtmlAreaProcessedPolygon {
                coordinates: &values[..values.len() - values.len() % 2],
            }),
            HtmlAreaShape::WholeImage => Output::WholeImage,
        }
    }
}
impl HtmlValueInterpreter {
    /// Both supplied raw values are admitted before expanded-name applicability.
    /// Wrong roles never run a grammar or become a missing-attribute default.
    pub fn area_coordinates(
        &self,
        element: &QualifiedName,
        shape: Option<&Attribute>,
        coords: Option<&Attribute>,
    ) -> Result<HtmlAreaCoordinatesReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(shape.map(Attribute::value))?;
        self.admit_attribute_raw(coords.map(Attribute::value))?;
        let shape = self.enumerated(&SHAPE, element, shape)?;
        let coords = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Coords,
                rows: &[33],
                source: source!("attr-area-coords"),
                context: Context::Elements(&[KnownHtmlElement::Area]),
                vocabulary_source: None,
            },
            element,
            coords,
        );
        let common = if coords.applicability() == HtmlAttributeApplicability::Applicable {
            Some(self.floating_point_list(coords.raw())?)
        } else {
            None
        };
        let author = area_author(&shape, &coords, common.as_ref());
        Ok(HtmlAreaCoordinatesReport {
            shape,
            coords,
            common,
            author,
        })
    }
}
fn area_author(
    shape: &HtmlAttributeReport<HtmlAreaShape>,
    coords: &HtmlAttributeReport<()>,
    common: Option<&HtmlValueReport<HtmlFloatingPointList, HtmlFloatingPointList>>,
) -> HtmlAreaCoordinatesAuthorState {
    use HtmlAreaCoordinatesAuthorError as Error;
    use HtmlAreaCoordinatesAuthorState as State;
    let Some(shape) = shape.intrinsic() else {
        return State::Unapplied;
    };
    let Some(common) = common else {
        return State::Unapplied;
    };
    let fail = |error| State::Nonconforming(error);
    if *shape == HtmlAreaShape::WholeImage {
        return if coords.raw().is_some() {
            fail(Error::ForbiddenPresence)
        } else {
            State::Conforming
        };
    }
    let Some(raw) = coords.raw() else {
        return fail(Error::MissingRequired);
    };
    if matches!(common.author(), HtmlValueAuthorState::Invalid(_)) {
        return fail(Error::InvalidListSyntax);
    }
    let items = if raw.is_empty() {
        0
    } else {
        raw.code_units()
            .split(|unit| *unit == u16::from(b','))
            .count()
    };
    let cardinal = match shape {
        HtmlAreaShape::Circle => items == 3,
        HtmlAreaShape::Rectangle => items == 4,
        HtmlAreaShape::Polygon => items >= 6 && items % 2 == 0,
        HtmlAreaShape::WholeImage => true,
    };
    if !cardinal {
        return fail(Error::Cardinality);
    }
    // The common report proved complete list grammar, including overflow items.
    // This pass adds mathematical constraints without rounding author numbers.
    let mut keys = Vec::with_capacity(items.min(4));
    for (index, token) in raw
        .code_units()
        .split(|unit| *unit == u16::from(b','))
        .enumerate()
    {
        let key = match strict_scientific_decimal_key_units(token) {
            Ok(key) => key,
            Err(ScientificDecimalKeyError::RepresentationExhausted) => {
                return State::RepresentationUnavailable;
            }
            Err(ScientificDecimalKeyError::Syntax(_)) => return fail(Error::InvalidListSyntax),
        };
        if !key.is_integer() {
            return fail(Error::NonintegerItem { index });
        }
        if *shape != HtmlAreaShape::Polygon {
            keys.push(key);
        }
    }
    match shape {
        HtmlAreaShape::Circle if !keys[2].is_zero() && !keys[2].is_positive() => {
            fail(Error::NegativeRadius)
        }
        HtmlAreaShape::Rectangle => {
            let compare = |a: usize, b: usize| keys[a].compare(&keys[b]);
            match compare(0, 2) {
                Ok(Ordering::Less) => {}
                Ok(_) => return fail(Error::NonincreasingX),
                Err(_) => return State::RepresentationUnavailable,
            }
            match compare(1, 3) {
                Ok(Ordering::Less) => State::Conforming,
                Ok(_) => fail(Error::NonincreasingY),
                Err(_) => State::RepresentationUnavailable,
            }
        }
        _ => State::Conforming,
    }
}
