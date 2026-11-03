//! Intrinsic area shape/coords author constraints and image-map coordinate
//! descriptors, from HTML #attr-area-shape, #attr-area-coords and
//! #image-map-processing-model. Coordinates are displayed-image CSS pixels;
//! image dimensions, association, layering and hit testing remain external.

use super::*;
use crate::internal::microsyntax::numeric::{
    ScientificDecimalKeyError, strict_scientific_decimal_key_units,
};
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
    /// The `circle` keyword: a circle given by its centre and radius. The alias `circ` is
    /// recognised but nonconforming.
    Circle,
    /// The `default` keyword: the whole image, which takes no `coords`.
    WholeImage,
    /// The `poly` keyword: a polygon given by its vertices. The alias `polygon` is recognised but
    /// nonconforming.
    Polygon,
    /// The `rect` keyword: a rectangle given by two opposite corners, and also what a missing or
    /// invalid value selects. The alias `rectangle` is recognised but nonconforming.
    Rectangle,
}
impl HtmlAreaShape {
    /// Returns the conforming keyword for the shape: `circle`, `default`, `poly` or `rect`.
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::WholeImage => "default",
            Self::Polygon => "poly",
            Self::Rectangle => "rect",
        }
    }
    /// Builds the no-namespace `shape` attribute with the canonical keyword.
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
    /// Pairs two exact integers. Any integer is admitted, because a point's validity depends on the
    /// shape that uses it.
    pub fn new(x: HtmlInteger, y: HtmlInteger) -> Self {
        Self { x, y }
    }
    /// Returns the exact horizontal coordinate.
    pub fn x(&self) -> &HtmlInteger {
        &self.x
    }
    /// Returns the exact vertical coordinate.
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
    /// Pairs a centre with a radius. The radius type already excludes negative values, and zero is
    /// allowed.
    pub fn new(center: HtmlAreaIntegerPoint, radius: HtmlNonNegativeInteger) -> Self {
        Self { center, radius }
    }
    /// Returns the exact centre.
    pub fn center(&self) -> &HtmlAreaIntegerPoint {
        &self.center
    }
    /// Returns the exact radius, which is never negative.
    pub fn radius(&self) -> &HtmlNonNegativeInteger {
        &self.radius
    }
}
/// An authored rectangle whose left edge lies strictly left of its right edge and whose top edge
/// lies strictly above its bottom edge, in exact integers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAreaRectangle {
    top_left: HtmlAreaIntegerPoint,
    bottom_right: HtmlAreaIntegerPoint,
}
impl HtmlAreaRectangle {
    /// Pairs two corners. Fails with `NonincreasingX` unless the first x is less than the second
    /// and with `NonincreasingY` unless the first y is less than the second; x is checked first.
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
    /// Returns the corner with the smaller x and y.
    pub fn top_left(&self) -> &HtmlAreaIntegerPoint {
        &self.top_left
    }
    /// Returns the corner with the larger x and y.
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
    /// Takes the vertices in order. Fails with `TooFewVertices` unless there are at least three.
    pub fn new(vertices: Vec<HtmlAreaIntegerPoint>) -> Result<Self, HtmlAreaNativeError> {
        if vertices.len() < 3 {
            return Err(HtmlAreaNativeError::TooFewVertices);
        }
        Ok(Self { vertices })
    }
    /// Returns the vertices in their authored order; there are at least three.
    pub fn vertices(&self) -> &[HtmlAreaIntegerPoint] {
        &self.vertices
    }
}
/// Why native area geometry cannot be constructed. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAreaNativeError {
    /// A rectangle's first x is not strictly less than its second x.
    NonincreasingX,
    /// A rectangle's first y is not strictly less than its second y.
    NonincreasingY,
    /// A polygon has fewer than three vertices.
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
    /// A circle area, authored as `shape=circle` with three coordinates.
    Circle(HtmlAreaCircle),
    /// A rectangle area, authored as `shape=rect` with four coordinates.
    Rectangle(HtmlAreaRectangle),
    /// A polygon area, authored as `shape=poly` with an even number of at least six coordinates.
    Polygon(HtmlAreaPolygon),
    /// The whole image, authored as `shape=default` with no `coords`.
    WholeImage,
}
impl HtmlAuthoredArea {
    /// Returns the shape keyword state that this geometry is authored with.
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
#[non_exhaustive]
pub enum HtmlAreaCoordinatesAuthorError {
    /// The shape is circle, rect or poly and `coords` is absent.
    MissingRequired,
    /// The shape is `default`, which takes no coordinates, and `coords` is present.
    ForbiddenPresence,
    /// `coords` is not a valid list of comma-separated integers.
    InvalidListSyntax,
    /// `coords` has the wrong item count: three for a circle, four for a rectangle, an even number
    /// of at least six for a polygon.
    Cardinality,
    /// The item at the zero-based `index` is a valid number but not an integer.
    NonintegerItem {
        /// Zero-based index of the first item that is not an integer.
        index: usize,
    },
    /// A circle's radius is less than zero. The check uses the exact decimal value, without
    /// rounding.
    NegativeRadius,
    /// A rectangle's first x is not strictly less than its second x.
    NonincreasingX,
    /// A rectangle's first y is not strictly less than its second y.
    NonincreasingY,
}
impl fmt::Display for HtmlAreaCoordinatesAuthorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequired => f.write_str("area shape requires a coords attribute"),
            Self::ForbiddenPresence => f.write_str("default area shape forbids a coords attribute"),
            Self::InvalidListSyntax => {
                f.write_str("area coords is not a valid list of comma-separated integers")
            }
            Self::Cardinality => f.write_str("area coords has the wrong number of values"),
            Self::NonintegerItem { index } => {
                write!(f, "area coords item at index {index} is not an integer")
            }
            Self::NegativeRadius => f.write_str("area circle radius is negative"),
            Self::NonincreasingX => f.write_str("area rectangle x coordinates must increase"),
            Self::NonincreasingY => f.write_str("area rectangle y coordinates must increase"),
        }
    }
}
impl Error for HtmlAreaCoordinatesAuthorError {}
/// Shape-specific author judgment of `coords`. It is independent of the common list grammar
/// evidence in the report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaCoordinatesAuthorState {
    /// The rule does not apply to these names, or no `coords` list was examined.
    Unapplied,
    /// `coords` satisfies the requirements of its shape.
    Conforming,
    /// `coords` violates a requirement of its shape; the payload is the first violation found.
    Nonconforming(HtmlAreaCoordinatesAuthorError),
    /// The grammar was valid but checked exact-key representation exhausted.
    RepresentationUnavailable,
}
/// Why a recognised shape processes to an empty area, which covers no part of the image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAreaEmptyReason {
    /// The coordinate list is shorter than the shape needs.
    TooFewCoordinates {
        /// Number of coordinates the shape needs: three, four or six.
        required: usize,
        /// Number of coordinates the recovered list supplies.
        actual: usize,
    },
    /// A circle's radius is zero or negative, so the circle is empty.
    NonpositiveRadius,
}
/// A checked finite processing coordinate in displayed-image CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlAreaProcessedPoint {
    x: HtmlFloatingPointNumber,
    y: HtmlFloatingPointNumber,
}
impl HtmlAreaProcessedPoint {
    /// Returns the finite horizontal coordinate in displayed-image CSS pixels.
    pub fn x(&self) -> HtmlFloatingPointNumber {
        self.x
    }
    /// Returns the finite vertical coordinate in displayed-image CSS pixels.
    pub fn y(&self) -> HtmlFloatingPointNumber {
        self.y
    }
}
/// A processed circle with a positive radius, in displayed-image CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlAreaProcessedCircle {
    center: HtmlAreaProcessedPoint,
    radius: HtmlFloatingPointNumber,
}
impl HtmlAreaProcessedCircle {
    /// Returns the centre.
    pub fn center(&self) -> HtmlAreaProcessedPoint {
        self.center
    }
    /// Returns the radius, which is greater than zero.
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
    /// Returns the corner with the smaller x and y after any swap of out-of-order values.
    pub fn top_left(&self) -> HtmlAreaProcessedPoint {
        self.top_left
    }
    /// Returns the corner with the larger x and y after any swap of out-of-order values.
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
    /// Returns the borrowed coordinates as an alternating x, y sequence of even length, dropping a
    /// trailing unpaired value.
    pub fn coordinates(&self) -> &'a [HtmlFloatingPointNumber] {
        self.coordinates
    }
    /// Returns the vertices in coordinate order, one per x, y pair.
    pub fn vertices(&self) -> impl ExactSizeIterator<Item = HtmlAreaProcessedPoint> + 'a {
        self.coordinates
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| HtmlAreaProcessedPoint {
                x: pair[0],
                y: pair[1],
            })
    }
}
/// The area an image map processes `shape` and `coords` to. It borrows the report's coordinates and
/// performs no hit test.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlAreaProcessedShape<'a> {
    /// The rule does not apply to the supplied names; the payload says why.
    Unapplied(HtmlAttributeApplicability),
    /// The shape has too few coordinates or a nonpositive radius, so it covers nothing.
    Empty(HtmlAreaEmptyReason),
    /// Symbolic complete image rectangle; dimensions are not guessed.
    WholeImage,
    /// A circle with a positive radius.
    Circle(HtmlAreaProcessedCircle),
    /// A rectangle. Equal edges are allowed here, unlike in authored rectangles.
    Rectangle(HtmlAreaProcessedRectangle),
    /// A polygon over the paired coordinates.
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
    /// Returns the `shape` attribute report. Its intrinsic state is `Rectangle` for a missing or
    /// invalid value.
    pub fn shape(&self) -> &HtmlAttributeReport<HtmlAreaShape> {
        &self.shape
    }
    /// Exact coords metadata, presence and applicability. The envelope's
    /// keyword recognition/author fields are Unapplied: list grammar belongs
    /// to common(), and shape-specific requirements belong to author().
    pub fn coords(&self) -> &HtmlAttributeReport<()> {
        &self.coords
    }
    /// Returns the common floating-point list report for `coords`, or `None` when the rule does not
    /// apply to the supplied names.
    pub fn common(&self) -> Option<&HtmlValueReport<HtmlFloatingPointList, HtmlFloatingPointList>> {
        self.common.as_ref()
    }
    /// Library diagnostic precedence: presence, list grammar, cardinality,
    /// exact integrality, then radius/rectangle ordering. This is not a new
    /// normative processing algorithm or complete element conformance verdict.
    pub fn author(&self) -> HtmlAreaCoordinatesAuthorState {
        self.author
    }
    /// Returns the URL of the image map processing model clause that defines how the coordinates
    /// become an area.
    pub fn processing_source_clause(&self) -> &'static str {
        source!("image-map-processing-model")
    }
    /// Returns the area that the shape and coordinates process to. Processing reads the recovered
    /// floating-point list, so a `coords` value that recovery cannot read yields `Empty` with too
    /// few coordinates.
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
        element: &(impl crate::HtmlNameInput + ?Sized),
        shape: Option<&impl crate::HtmlAttributeInput>,
        coords: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAreaCoordinatesReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(shape.map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(coords.map(HtmlAttributeInput::value))?;
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
