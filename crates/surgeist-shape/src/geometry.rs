use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub const ZERO: Self = Self::new_unchecked(0.0, 0.0);

    pub fn try_new(x: f64, y: f64) -> Result<Self> {
        validate_finite(x, "point x")?;
        validate_finite(y, "point y")?;
        Ok(Self::new_unchecked(x, y))
    }

    pub fn try_translate(self, x: f64, y: f64) -> Result<Self> {
        validate_finite(x, "point translate x")?;
        validate_finite(y, "point translate y")?;
        Self::try_new(self.x + x, self.y + y)
    }

    #[must_use]
    pub(crate) const fn new_unchecked(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub(crate) const fn new(x: f64, y: f64) -> Self {
        Self::new_unchecked(x, y)
    }

    #[must_use]
    pub(crate) fn translate(self, x: f64, y: f64) -> Self {
        Self::new_unchecked(self.x + x, self.y + y)
    }

    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::Point {
        kurbo::Point::new(self.x, self.y)
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        validate_finite(self.x, &format!("{name} x"))?;
        validate_finite(self.y, &format!("{name} y"))
    }
}

impl From<Point> for kurbo::Point {
    fn from(point: Point) -> Self {
        point.to_kurbo()
    }
}

impl TryFrom<kurbo::Point> for Point {
    type Error = Error;

    fn try_from(point: kurbo::Point) -> Result<Self> {
        Self::try_new(point.x, point.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    width: f64,
    height: f64,
}

impl Size {
    pub const ZERO: Self = Self::new_unchecked(0.0, 0.0);

    pub fn try_new(width: f64, height: f64) -> Result<Self> {
        validate_non_negative_kind(width, "size width", NumericKind::Size)?;
        validate_non_negative_kind(height, "size height", NumericKind::Size)?;
        Ok(Self::new_unchecked(width, height))
    }

    #[must_use]
    pub(crate) const fn new_unchecked(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub(crate) const fn new(width: f64, height: f64) -> Self {
        Self::new_unchecked(width, height)
    }

    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    #[must_use]
    pub fn min_side(self) -> f64 {
        self.width.min(self.height)
    }

    #[must_use]
    pub fn max_side(self) -> f64 {
        self.width.max(self.height)
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        validate_non_negative_kind(self.width, &format!("{name} width"), NumericKind::Size)?;
        validate_non_negative_kind(self.height, &format!("{name} height"), NumericKind::Size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    origin: Point,
    size: Size,
}

impl Rect {
    pub const ZERO: Self = Self::new_unchecked(0.0, 0.0, 0.0, 0.0);

    pub fn try_new(x: f64, y: f64, width: f64, height: f64) -> Result<Self> {
        let origin = Point::try_new(x, y)?;
        let size = Size::try_new(width, height)?;
        Ok(Self { origin, size })
    }

    pub fn try_from_min_max(min: Point, max: Point) -> Result<Self> {
        Self::try_new(
            min.x,
            min.y,
            (max.x - min.x).max(0.0),
            (max.y - min.y).max(0.0),
        )
    }

    pub fn try_from_origin_size(origin: Point, size: Size) -> Result<Self> {
        origin.validate("rect origin")?;
        size.validate("rect size")?;
        Ok(Self { origin, size })
    }

    pub fn try_inset(self, insets: Insets) -> Result<Self> {
        insets.validate("rect inset")?;
        let width = (self.size.width - insets.left - insets.right).max(0.0);
        let height = (self.size.height - insets.top - insets.bottom).max(0.0);
        Self::try_new(
            self.origin.x + insets.left,
            self.origin.y + insets.top,
            width,
            height,
        )
    }

    pub fn try_outset(self, insets: Insets) -> Result<Self> {
        insets.validate("rect outset")?;
        Self::try_new(
            self.origin.x - insets.left,
            self.origin.y - insets.top,
            self.size.width + insets.left + insets.right,
            self.size.height + insets.top + insets.bottom,
        )
    }

    pub fn try_translate(self, x: f64, y: f64) -> Result<Self> {
        Self::try_from_origin_size(self.origin.try_translate(x, y)?, self.size)
    }

    #[must_use]
    pub(crate) const fn new_unchecked(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point::new_unchecked(x, y),
            size: Size::new_unchecked(width, height),
        }
    }

    #[must_use]
    pub(crate) const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new_unchecked(x, y, width, height)
    }

    #[must_use]
    pub const fn origin(self) -> Point {
        self.origin
    }

    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self::ZERO
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.size.is_empty()
    }

    #[must_use]
    pub fn min(self) -> Point {
        self.origin
    }

    #[must_use]
    pub fn max(self) -> Point {
        Point::new_unchecked(
            self.origin.x + self.size.width,
            self.origin.y + self.size.height,
        )
    }

    #[must_use]
    pub fn center(self) -> Point {
        Point::new_unchecked(
            self.origin.x + self.size.width * 0.5,
            self.origin.y + self.size.height * 0.5,
        )
    }

    #[must_use]
    pub fn width(self) -> f64 {
        self.size.width
    }

    #[must_use]
    pub fn height(self) -> f64 {
        self.size.height
    }

    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        let max = self.max();
        point.x >= self.origin.x && point.x <= max.x && point.y >= self.origin.y && point.y <= max.y
    }

    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        let a = self.max();
        let b = other.max();
        self.origin.x < b.x && a.x > other.origin.x && self.origin.y < b.y && a.y > other.origin.y
    }

    #[must_use]
    pub(crate) fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let a = self.max();
        let b = other.max();
        Self::new_unchecked(
            self.origin.x.min(other.origin.x),
            self.origin.y.min(other.origin.y),
            (a.x.max(b.x) - self.origin.x.min(other.origin.x)).max(0.0),
            (a.y.max(b.y) - self.origin.y.min(other.origin.y)).max(0.0),
        )
    }

    #[must_use]
    pub(crate) fn intersection(self, other: Self) -> Self {
        let a = self.max();
        let b = other.max();
        let min_x = self.origin.x.max(other.origin.x);
        let min_y = self.origin.y.max(other.origin.y);
        Self::new_unchecked(
            min_x,
            min_y,
            (a.x.min(b.x) - min_x).max(0.0),
            (a.y.min(b.y) - min_y).max(0.0),
        )
    }

    #[must_use]
    pub(crate) fn inset_unchecked(self, insets: Insets) -> Self {
        let width = (self.size.width - insets.left - insets.right).max(0.0);
        let height = (self.size.height - insets.top - insets.bottom).max(0.0);
        Self::new_unchecked(
            self.origin.x + insets.left,
            self.origin.y + insets.top,
            width,
            height,
        )
    }

    #[must_use]
    pub(crate) fn inset(self, insets: Insets) -> Self {
        self.inset_unchecked(insets)
    }

    #[must_use]
    pub(crate) fn outset_unchecked(self, insets: Insets) -> Self {
        Self::new_unchecked(
            self.origin.x - insets.left,
            self.origin.y - insets.top,
            self.size.width + insets.left + insets.right,
            self.size.height + insets.top + insets.bottom,
        )
    }

    #[must_use]
    pub(crate) fn outset(self, insets: Insets) -> Self {
        self.outset_unchecked(insets)
    }

    pub fn transformed_bounds(self, transform: Transform) -> Result<Self> {
        transform.try_apply_rect(self)
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::Rect {
        let max = self.max();
        kurbo::Rect::new(self.origin.x, self.origin.y, max.x, max.y)
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        self.origin.validate(name)?;
        self.size.validate(name)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Rect> for kurbo::Rect {
    fn from(rect: Rect) -> Self {
        rect.to_kurbo()
    }
}

impl TryFrom<kurbo::Rect> for Rect {
    type Error = Error;

    fn try_from(rect: kurbo::Rect) -> Result<Self> {
        let min = Point::try_new(rect.x0, rect.y0)?;
        let max = Point::try_new(rect.x1, rect.y1)?;
        Self::try_from_min_max(min, max)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

impl Insets {
    pub fn try_new(top: f64, right: f64, bottom: f64, left: f64) -> Result<Self> {
        validate_finite(top, "inset top")?;
        validate_finite(right, "inset right")?;
        validate_finite(bottom, "inset bottom")?;
        validate_finite(left, "inset left")?;
        Ok(Self::new_unchecked(top, right, bottom, left))
    }

    pub fn try_all(value: f64) -> Result<Self> {
        Self::try_new(value, value, value, value)
    }

    #[must_use]
    pub(crate) const fn new_unchecked(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub(crate) const fn all(value: f64) -> Self {
        Self::new_unchecked(value, value, value, value)
    }

    #[must_use]
    pub const fn top(self) -> f64 {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> f64 {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> f64 {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> f64 {
        self.left
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new_unchecked(0.0, 0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn horizontal(self) -> f64 {
        self.left + self.right
    }

    #[must_use]
    pub fn vertical(self) -> f64 {
        self.top + self.bottom
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }

    pub(crate) fn validate(self, name: &str) -> Result<()> {
        validate_finite(self.top, &format!("{name} top"))?;
        validate_finite(self.right, &format!("{name} right"))?;
        validate_finite(self.bottom, &format!("{name} bottom"))?;
        validate_finite(self.left, &format!("{name} left"))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Radii {
    top_left: f64,
    top_right: f64,
    bottom_right: f64,
    bottom_left: f64,
}

impl Radii {
    pub fn try_new(
        top_left: f64,
        top_right: f64,
        bottom_right: f64,
        bottom_left: f64,
    ) -> Result<Self> {
        validate_non_negative_kind(top_left, "top-left radius", NumericKind::Radius)?;
        validate_non_negative_kind(top_right, "top-right radius", NumericKind::Radius)?;
        validate_non_negative_kind(bottom_right, "bottom-right radius", NumericKind::Radius)?;
        validate_non_negative_kind(bottom_left, "bottom-left radius", NumericKind::Radius)?;
        Ok(Self::new_unchecked(
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        ))
    }

    pub fn try_all(radius: f64) -> Result<Self> {
        Self::try_new(radius, radius, radius, radius)
    }

    pub fn try_inset(self, amount: f64) -> Result<Self> {
        validate_finite(amount, "radius inset amount")?;
        Self::try_new(
            (self.top_left - amount).max(0.0),
            (self.top_right - amount).max(0.0),
            (self.bottom_right - amount).max(0.0),
            (self.bottom_left - amount).max(0.0),
        )
    }

    pub fn try_outset(self, amount: f64) -> Result<Self> {
        validate_finite(amount, "radius outset amount")?;
        Self::try_new(
            (self.top_left + amount).max(0.0),
            (self.top_right + amount).max(0.0),
            (self.bottom_right + amount).max(0.0),
            (self.bottom_left + amount).max(0.0),
        )
    }

    #[must_use]
    pub(crate) const fn new_unchecked(
        top_left: f64,
        top_right: f64,
        bottom_right: f64,
        bottom_left: f64,
    ) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    #[must_use]
    pub(crate) const fn new(
        top_left: f64,
        top_right: f64,
        bottom_right: f64,
        bottom_left: f64,
    ) -> Self {
        Self::new_unchecked(top_left, top_right, bottom_right, bottom_left)
    }

    #[must_use]
    pub(crate) fn outset(self, amount: f64) -> Self {
        Self::new_unchecked(
            (self.top_left + amount).max(0.0),
            (self.top_right + amount).max(0.0),
            (self.bottom_right + amount).max(0.0),
            (self.bottom_left + amount).max(0.0),
        )
    }

    #[must_use]
    pub const fn top_left(self) -> f64 {
        self.top_left
    }

    #[must_use]
    pub const fn top_right(self) -> f64 {
        self.top_right
    }

    #[must_use]
    pub const fn bottom_right(self) -> f64 {
        self.bottom_right
    }

    #[must_use]
    pub const fn bottom_left(self) -> f64 {
        self.bottom_left
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new_unchecked(0.0, 0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }

    #[must_use]
    pub fn is_uniform(self) -> bool {
        self.top_left == self.top_right
            && self.top_left == self.bottom_right
            && self.top_left == self.bottom_left
    }

    pub fn normalized_for(self, rect: Rect) -> Result<Self> {
        self.validate()?;
        rect.validate("radius rect")?;
        let top = self.top_left + self.top_right;
        let bottom = self.bottom_left + self.bottom_right;
        let left = self.top_left + self.bottom_left;
        let right = self.top_right + self.bottom_right;
        let mut factor: f64 = 1.0;
        if top > 0.0 {
            factor = factor.min(rect.width() / top);
        }
        if bottom > 0.0 {
            factor = factor.min(rect.width() / bottom);
        }
        if left > 0.0 {
            factor = factor.min(rect.height() / left);
        }
        if right > 0.0 {
            factor = factor.min(rect.height() / right);
        }
        factor = factor.clamp(0.0, 1.0);
        Self::try_new(
            self.top_left * factor,
            self.top_right * factor,
            self.bottom_right * factor,
            self.bottom_left * factor,
        )
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::RoundedRectRadii {
        kurbo::RoundedRectRadii::new(
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        )
    }

    fn validate(self) -> Result<()> {
        validate_non_negative_kind(self.top_left, "top-left radius", NumericKind::Radius)?;
        validate_non_negative_kind(self.top_right, "top-right radius", NumericKind::Radius)?;
        validate_non_negative_kind(
            self.bottom_right,
            "bottom-right radius",
            NumericKind::Radius,
        )?;
        validate_non_negative_kind(self.bottom_left, "bottom-left radius", NumericKind::Radius)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    matrix: [f64; 6],
}

impl Transform {
    pub const IDENTITY: Self = Self::new_unchecked([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn try_new(matrix: [f64; 6]) -> Result<Self> {
        for (index, value) in matrix.into_iter().enumerate() {
            validate_finite(value, &format!("transform matrix {index}"))?;
        }
        Ok(Self::new_unchecked(matrix))
    }

    pub fn try_translate(x: f64, y: f64) -> Result<Self> {
        validate_finite(x, "transform translate x")?;
        validate_finite(y, "transform translate y")?;
        Ok(Self::new_unchecked([1.0, 0.0, 0.0, 1.0, x, y]))
    }

    pub fn try_scale(x: f64, y: f64) -> Result<Self> {
        validate_finite(x, "transform scale x")?;
        validate_finite(y, "transform scale y")?;
        Ok(Self::new_unchecked([x, 0.0, 0.0, y, 0.0, 0.0]))
    }

    pub fn try_rotate(radians: f64) -> Result<Self> {
        validate_finite(radians, "transform rotation")?;
        let (sin, cos) = radians.sin_cos();
        Self::try_new([cos, sin, -sin, cos, 0.0, 0.0])
    }

    pub fn try_then(self, next: Self) -> Result<Self> {
        let [a, b, c, d, e, f] = self.matrix;
        let [g, h, i, j, k, l] = next.matrix;
        Self::try_new([
            a * g + c * h,
            b * g + d * h,
            a * i + c * j,
            b * i + d * j,
            a * k + c * l + e,
            b * k + d * l + f,
        ])
    }

    pub fn try_apply_point(self, point: Point) -> Result<Point> {
        let [a, b, c, d, e, f] = self.matrix;
        Point::try_new(a * point.x + c * point.y + e, b * point.x + d * point.y + f)
    }

    pub fn try_apply_rect(self, rect: Rect) -> Result<Rect> {
        let min = rect.min();
        let max = rect.max();
        let points = [
            self.try_apply_point(min)?,
            self.try_apply_point(Point::new_unchecked(max.x, min.y))?,
            self.try_apply_point(max)?,
            self.try_apply_point(Point::new_unchecked(min.x, max.y))?,
        ];
        bounds_of_points(&points).ok_or_else(|| Error::new(ErrorCode::EmptyPath, "empty bounds"))
    }

    #[must_use]
    pub(crate) const fn new_unchecked(matrix: [f64; 6]) -> Self {
        Self { matrix }
    }

    #[must_use]
    pub const fn matrix(self) -> [f64; 6] {
        self.matrix
    }

    #[must_use]
    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let [a, b, c, d, e, f] = self.matrix;
        let det = a * d - b * c;
        if det.abs() <= f64::EPSILON {
            return None;
        }
        let inv = 1.0 / det;
        Self::try_new([
            d * inv,
            -b * inv,
            -c * inv,
            a * inv,
            (c * f - d * e) * inv,
            (b * e - a * f) * inv,
        ])
        .ok()
    }

    #[must_use]
    pub fn to_kurbo(self) -> kurbo::Affine {
        kurbo::Affine::new(self.matrix)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn bounds_of_points(points: &[Point]) -> Option<Rect> {
    let first = points.first()?;
    let mut min_x = first.x;
    let mut min_y = first.y;
    let mut max_x = first.x;
    let mut max_y = first.y;
    for point in &points[1..] {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Rect::try_from_min_max(
        Point::new_unchecked(min_x, min_y),
        Point::new_unchecked(max_x, max_y),
    )
    .ok()
}
