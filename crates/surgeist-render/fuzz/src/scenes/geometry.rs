use super::*;

const TRANSFORM: &str = "surgeist-render::concrete-transform";
const CLIP: &str = "surgeist-render::concrete-clip";
const MASK: &str = "surgeist-render::concrete-mask-placement";
const MAX_PATH_ELEMENTS: usize = 128;

#[derive(Clone, Debug)]
pub(super) struct ClipRecipe {
    pub(super) geometry: ClipSource,
    pub(super) transform: [f64; 6],
}
#[derive(Clone, Debug)]
pub(super) enum ClipSource {
    Rect([f64; 4]),
    Path(Vec<[f64; 2]>),
}

fn invalid_rejection<T>(property: &str, result: render::Result<T>) -> Result<CaseOutcome> {
    match result {
        Err(cause) if cause.code() == render::ErrorCode::InvalidInput => rejected(
            &[property],
            "public concrete geometry boundary rejected invalid numeric input",
        ),
        Err(cause) => violated(
            property,
            "concrete-numeric-rejection-kind",
            format!("{cause:?}"),
        ),
        Ok(_) => violated(
            property,
            "concrete-invalid-numeric-accepted",
            "invalid numeric request was accepted",
        ),
    }
}

pub(super) fn coefficients(values: [f64; 6]) -> Result<CaseOutcome> {
    let result = render::Transform::try_new(values);
    if values.iter().any(|value| !value.is_finite()) {
        return invalid_rejection(TRANSFORM, result);
    }
    let transform = match product_observation(result, TRANSFORM, "finite-transform-rejected")? {
        Ok(transform) => transform,
        Err(failure) => return Ok(failure),
    };
    if transform.as_array().map(f64::to_bits) != values.map(f64::to_bits) {
        return violated(
            TRANSFORM,
            "transform-coefficient-bits",
            "finite coefficients changed during checked construction",
        );
    }
    evaluated(&[TRANSFORM])
}

// Integer homogeneous matrices deliberately use a different representation and
// arithmetic from the product's six floating coefficients. The checked lattice
// is small enough that every multiplication, sum, and f64 conversion is exact.
type Matrix = [[i128; 3]; 3];
fn matrix(values: [f64; 6]) -> Matrix {
    [
        [values[0] as i128, values[2] as i128, values[4] as i128],
        [values[1] as i128, values[3] as i128, values[5] as i128],
        [0, 0, 1],
    ]
}
fn multiply(left: Matrix, right: Matrix) -> Matrix {
    let mut result = [[0; 3]; 3];
    for row in 0..3 {
        for column in 0..3 {
            result[row][column] = (0..3).map(|k| left[row][k] * right[k][column]).sum();
        }
    }
    result
}
fn affine(matrix: Matrix) -> [f64; 6] {
    [
        matrix[0][0] as f64,
        matrix[1][0] as f64,
        matrix[0][1] as f64,
        matrix[1][1] as f64,
        matrix[0][2] as f64,
        matrix[1][2] as f64,
    ]
}

pub(super) fn compose(first: [f64; 6], next: [f64; 6], origin: [f64; 2]) -> Result<CaseOutcome> {
    let observed = (|| -> render::Result<_> {
        let first_transform = render::Transform::try_new(first)?;
        let next_transform = render::Transform::try_new(next)?;
        Ok((
            first_transform.then(next_transform)?,
            first_transform.around(render::Point::try_new(origin[0], origin[1])?)?,
        ))
    })();
    let (then, around) =
        match product_observation(observed, TRANSFORM, "transform-lattice-rejected")? {
            Ok(observed) => observed,
            Err(failure) => return Ok(failure),
        };
    let expected_then = affine(multiply(matrix(next), matrix(first)));
    let mut positive = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
    positive[0][2] = origin[0] as i128;
    positive[1][2] = origin[1] as i128;
    let mut negative = positive;
    negative[0][2] *= -1;
    negative[1][2] *= -1;
    let expected_around = affine(multiply(multiply(positive, matrix(first)), negative));
    if then.as_array() != expected_then || around.as_array() != expected_around {
        return violated(
            TRANSFORM,
            "transform-lattice-composition",
            format!(
                "then {:?} != {expected_then:?}; around {:?} != {expected_around:?}",
                then.as_array(),
                around.as_array()
            ),
        );
    }
    evaluated(&[TRANSFORM])
}

pub(super) fn overflow() -> Result<CaseOutcome> {
    let setup = (|| -> render::Result<_> {
        Ok((
            render::Transform::scale(f64::MAX, 1.)?,
            render::Transform::scale(2., 1.)?,
            render::Point::try_new(f64::MAX, 0.)?,
        ))
    })();
    let (large, two, origin) =
        match product_observation(setup, TRANSFORM, "transform-overflow-setup-rejected")? {
            Ok(setup) => setup,
            Err(failure) => return Ok(failure),
        };
    let result = large.then(two);
    if !matches!(&result, Err(cause) if cause.code() == render::ErrorCode::InvalidInput) {
        return invalid_rejection(TRANSFORM, result);
    }
    invalid_rejection(TRANSFORM, two.around(origin))
}

fn rect_valid([x, y, width, height]: [f64; 4]) -> bool {
    [x, y, width, height].iter().all(|value| value.is_finite())
        && width >= 0.
        && height >= 0.
        && (x + width).is_finite()
        && (y + height).is_finite()
}
fn rect([x, y, width, height]: [f64; 4]) -> render::Result<render::Rect> {
    render::Rect::try_new(x, y, width, height)
}

impl ClipRecipe {
    pub(super) fn validate(&self, budget: &mut Budget) -> Result<()> {
        if let ClipSource::Path(points) = &self.geometry {
            if !(3..MAX_PATH_ELEMENTS).contains(&points.len()) {
                return Err(error(
                    "clip polygon point ceiling or fewer than three points",
                ));
            }
            credit(budget, (points.len() + 1) * 128, points.len() + 1, 1)?;
        }
        Ok(())
    }
    pub(super) fn read(reader: &mut Reader<'_>, budget: &mut Budget) -> Result<Self> {
        let geometry = match reader.u8()? {
            0 => ClipSource::Rect(read_floats(reader)?),
            1 => {
                let count = reader.count(MAX_PATH_ELEMENTS - 1)?;
                if count < 3 {
                    return Err(error("clip polygon requires three points"));
                }
                credit(budget, (count + 1) * 128, count + 1, 1)?;
                let mut points = Vec::new();
                points
                    .try_reserve_exact(count)
                    .map_err(|_| error("clip point allocation"))?;
                for _ in 0..count {
                    points.push(read_floats(reader)?);
                }
                ClipSource::Path(points)
            }
            _ => return Err(error("unknown clip geometry tag")),
        };
        Ok(Self {
            geometry,
            transform: read_floats(reader)?,
        })
    }
    pub(super) fn write(&self, writer: &mut Writer) -> Result<()> {
        match &self.geometry {
            ClipSource::Rect(values) => {
                writer.u8(0)?;
                write_floats(writer, values)?;
            }
            ClipSource::Path(points) => {
                writer.u8(1)?;
                writer.count(points.len(), MAX_PATH_ELEMENTS - 1)?;
                for point in points {
                    write_floats(writer, point)?;
                }
            }
        }
        write_floats(writer, &self.transform)
    }
    fn bounds(&self) -> Option<[f64; 4]> {
        match &self.geometry {
            ClipSource::Rect(values) => rect_valid(*values).then_some(*values),
            ClipSource::Path(points) => {
                if points.iter().flatten().any(|value| !value.is_finite()) {
                    return None;
                }
                let min_x = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
                let max_x = points
                    .iter()
                    .map(|p| p[0])
                    .fold(f64::NEG_INFINITY, f64::max);
                let min_y = points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
                let max_y = points
                    .iter()
                    .map(|p| p[1])
                    .fold(f64::NEG_INFINITY, f64::max);
                let values = [min_x, min_y, max_x - min_x, max_y - min_y];
                rect_valid(values).then_some(values)
            }
        }
    }
    fn expected_acceptance(&self) -> bool {
        let Some([x, y, width, height]) = self.bounds() else {
            return false;
        };
        if self.transform.iter().any(|value| !value.is_finite()) {
            return false;
        }
        let [a, b, c, d, e, f] = self.transform;
        let corners = [
            [x, y],
            [x + width, y],
            [x, y + height],
            [x + width, y + height],
        ];
        let transformed = corners.map(|[x, y]| [a * x + c * y + e, b * x + d * y + f]);
        if transformed.iter().flatten().any(|value| !value.is_finite()) {
            return false;
        }
        let min_x = transformed
            .iter()
            .map(|p| p[0])
            .fold(f64::INFINITY, f64::min);
        let max_x = transformed
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = transformed
            .iter()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min);
        let max_y = transformed
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max);
        (max_x - min_x).is_finite() && (max_y - min_y).is_finite()
    }
    fn input(&self) -> render::Result<render::ClipInput> {
        let input = match &self.geometry {
            ClipSource::Rect(values) => {
                render::ClipInput::try_shape(render::Shape::rect(rect(*values)?))?
            }
            ClipSource::Path(points) => {
                let mut path = render::Path::new();
                for (index, [x, y]) in points.iter().copied().enumerate() {
                    let point = render::Point::try_new(x, y)?;
                    if index == 0 {
                        path.move_to(point);
                    } else {
                        path.line_to(point);
                    }
                }
                path.close();
                render::ClipInput::try_filled_path(render::FilledPath::try_new(
                    path,
                    render::FillRule::EvenOdd,
                )?)?
            }
        };
        Ok(
            input.with_coordinate_space(render::CoordinateSpaceTag::surface(
                render::Transform::try_new(self.transform)?,
            )?),
        )
    }
    pub(super) fn evaluate(&self) -> Result<CaseOutcome> {
        let mut context = render::RenderAccountingContext::default();
        let result = self
            .input()
            .and_then(|input| input.normalize(render::Capabilities::CURRENT, &mut context));
        if !self.expected_acceptance() {
            return invalid_rejection(CLIP, result);
        }
        let normalized = match product_observation(result, CLIP, "concrete-clip-rejected")? {
            Ok(normalized) => normalized,
            Err(failure) => return Ok(failure),
        };
        let geometry_equal = match (&self.geometry, normalized.geometry().kind()) {
            (ClipSource::Rect(expected), render::ClipGeometryKind::Rect(actual)) => {
                [actual.x(), actual.y(), actual.width(), actual.height()].map(f64::to_bits)
                    == expected.map(f64::to_bits)
            }
            (ClipSource::Path(expected), render::ClipGeometryKind::Path(actual)) => {
                let elements = actual.path().elements();
                actual.fill_rule() == render::FillRule::EvenOdd
                    && elements.len() == expected.len() + 1
                    && expected
                        .iter()
                        .enumerate()
                        .all(|(index, point)| match elements[index] {
                            render::PathElement::MoveTo(actual) if index == 0 => {
                                [actual.x(), actual.y()].map(f64::to_bits)
                                    == point.map(f64::to_bits)
                            }
                            render::PathElement::LineTo(actual) if index > 0 => {
                                [actual.x(), actual.y()].map(f64::to_bits)
                                    == point.map(f64::to_bits)
                            }
                            _ => false,
                        })
                    && matches!(elements.last(), Some(render::PathElement::Close))
            }
            _ => false,
        };
        let tag_equal = normalized.coordinate_space().is_some_and(|tag| {
            tag.kind() == render::CoordinateSpaceKind::Surface
                && tag.transform().as_array().map(f64::to_bits) == self.transform.map(f64::to_bits)
        });
        if !geometry_equal || !tag_equal {
            return violated(
                CLIP,
                "clip-authored-geometry-and-space",
                "normalization changed concrete source geometry or coordinate-space tag",
            );
        }
        let Some(report) = context.last_attempt() else {
            return violated(
                CLIP,
                "clip-accounting-report-missing",
                "public concrete clip normalization published no accounting report",
            );
        };
        let usage = report.usage();
        if let ClipSource::Path(points) = &self.geometry
            && usage.work(render::RenderWork::PathElementVisits) != (3 * (points.len() + 1)) as u64
        {
            return violated(
                CLIP,
                "clip-path-accounting",
                "validation, copy, and bounds inspection must visit each polygon element",
            );
        }
        evaluated(&[CLIP])
    }
}

pub(super) fn mask(image: &ImageRecipe, bounds: [f64; 4]) -> Result<CaseOutcome> {
    let original = match product_observation(image.materialize(), MASK, "concrete-mask-rejected")? {
        Ok(image) => image,
        Err(failure) => return Ok(failure),
    };
    let result = rect(bounds)
        .and_then(|bounds| render::ResolvedLayerAlphaMask::try_new(original.clone(), bounds));
    if !rect_valid(bounds) || bounds[2] <= 0. || bounds[3] <= 0. {
        return invalid_rejection(MASK, result);
    }
    let mask = match product_observation(result, MASK, "concrete-mask-rejected")? {
        Ok(mask) => mask,
        Err(failure) => return Ok(failure),
    };
    let actual = mask.bounds();
    if [actual.x(), actual.y(), actual.width(), actual.height()].map(f64::to_bits)
        != bounds.map(f64::to_bits)
        || mask.image() != &original
        || mask.image().id() != original.id()
    {
        return violated(
            MASK,
            "concrete-mask-resource-and-bounds",
            "concrete positive layer-local bounds or retained image changed",
        );
    }
    evaluated(&[MASK])
}
