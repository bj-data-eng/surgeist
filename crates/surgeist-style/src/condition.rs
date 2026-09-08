use std::collections::BTreeMap;

use crate::{AuthoredTokens, CustomPropertyName, Error, ErrorCode, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct MediaQueryList {
    queries: Vec<MediaQuery>,
}

impl MediaQueryList {
    pub fn try_new(queries: impl IntoIterator<Item = MediaQuery>) -> Result<Self> {
        let queries = queries.into_iter().collect::<Vec<_>>();
        if queries.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "media query list cannot be empty",
            ));
        }
        Ok(Self { queries })
    }

    #[must_use]
    pub fn queries(&self) -> &[MediaQuery] {
        &self.queries
    }

    #[must_use]
    pub fn matches(&self, environment: &MediaEnvironment) -> bool {
        self.queries.iter().any(|query| query.matches(environment))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaQuery {
    Condition(MediaCondition),
    Typed(TypedMediaQuery),
}

impl MediaQuery {
    #[must_use]
    pub fn matches(&self, environment: &MediaEnvironment) -> bool {
        match self {
            Self::Condition(condition) => condition.matches(environment),
            Self::Typed(query) => query.matches(environment),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedMediaQuery {
    modifier: Option<MediaQueryModifier>,
    media_type: MediaType,
    condition: Option<MediaCondition>,
}

impl TypedMediaQuery {
    #[must_use]
    pub const fn new(
        modifier: Option<MediaQueryModifier>,
        media_type: MediaType,
        condition: Option<MediaCondition>,
    ) -> Self {
        Self {
            modifier,
            media_type,
            condition,
        }
    }

    #[must_use]
    pub const fn modifier(&self) -> Option<MediaQueryModifier> {
        self.modifier
    }

    #[must_use]
    pub const fn media_type(&self) -> MediaType {
        self.media_type
    }

    #[must_use]
    pub const fn condition(&self) -> Option<&MediaCondition> {
        self.condition.as_ref()
    }

    #[must_use]
    pub fn matches(&self, environment: &MediaEnvironment) -> bool {
        let matches = self.media_type.matches(environment.media_type)
            && self
                .condition
                .as_ref()
                .is_none_or(|condition| condition.matches(environment));
        match self.modifier {
            Some(MediaQueryModifier::Not) => !matches,
            Some(MediaQueryModifier::Only) | None => matches,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MediaQueryModifier {
    Not,
    Only,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MediaType {
    #[default]
    All,
    Screen,
    Print,
}

impl MediaType {
    #[must_use]
    pub const fn matches(self, environment: Self) -> bool {
        matches!(self, Self::All)
            || matches!(
                (self, environment),
                (Self::Screen, Self::Screen) | (Self::Print, Self::Print)
            )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaCondition {
    Feature(MediaFeatureQuery),
    Not(Box<MediaCondition>),
    And(MediaConditionList),
    Or(MediaConditionList),
}

impl MediaCondition {
    #[must_use]
    pub fn matches(&self, environment: &MediaEnvironment) -> bool {
        match self {
            Self::Feature(query) => query.matches(environment),
            Self::Not(condition) => !condition.matches(environment),
            Self::And(conditions) => conditions
                .conditions()
                .iter()
                .all(|condition| condition.matches(environment)),
            Self::Or(conditions) => conditions
                .conditions()
                .iter()
                .any(|condition| condition.matches(environment)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaConditionList {
    conditions: Vec<MediaCondition>,
}

impl MediaConditionList {
    pub fn try_new(conditions: impl IntoIterator<Item = MediaCondition>) -> Result<Self> {
        let conditions = conditions.into_iter().collect::<Vec<_>>();
        if conditions.len() < 2 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "media condition list requires at least two conditions",
            ));
        }
        Ok(Self { conditions })
    }

    #[must_use]
    pub fn conditions(&self) -> &[MediaCondition] {
        &self.conditions
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QueryComparison {
    LessThan,
    LessThanOrEqual,
    Equal,
    GreaterThanOrEqual,
    GreaterThan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RangeFeature<T> {
    comparison: Option<QueryComparison>,
    value: T,
}

impl<T> RangeFeature<T> {
    #[must_use]
    pub const fn new(comparison: Option<QueryComparison>, value: T) -> Self {
        Self { comparison, value }
    }

    #[must_use]
    pub const fn comparison(&self) -> Option<QueryComparison> {
        self.comparison
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QueryLength {
    value: f32,
    unit: QueryLengthUnit,
}

impl QueryLength {
    pub fn try_new(value: f32, unit: QueryLengthUnit) -> Result<Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "query length must be finite and non-negative",
            ));
        }
        Ok(Self { value, unit })
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }

    #[must_use]
    pub const fn unit(self) -> QueryLengthUnit {
        self.unit
    }

    #[must_use]
    pub fn to_css_px(self, basis: &QueryLengthBasis) -> Option<f32> {
        basis.resolve(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QueryLengthUnit {
    Px,
    Em,
    Rem,
    Ex,
    Rex,
    Cap,
    Rcap,
    Ch,
    Rch,
    Ic,
    Ric,
    Lh,
    Rlh,
    Vw,
    Vh,
    Vi,
    Vb,
    Vmin,
    Vmax,
    Svw,
    Svh,
    Svi,
    Svb,
    Svmin,
    Svmax,
    Lvw,
    Lvh,
    Lvi,
    Lvb,
    Lvmin,
    Lvmax,
    Dvw,
    Dvh,
    Dvi,
    Dvb,
    Dvmin,
    Dvmax,
    Cqw,
    Cqh,
    Cqi,
    Cqb,
    Cqmin,
    Cqmax,
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QueryLengthBasis {
    em: Option<QueryLength>,
    rem: Option<QueryLength>,
    ex: Option<QueryLength>,
    rex: Option<QueryLength>,
    cap: Option<QueryLength>,
    rcap: Option<QueryLength>,
    ch: Option<QueryLength>,
    rch: Option<QueryLength>,
    ic: Option<QueryLength>,
    ric: Option<QueryLength>,
    lh: Option<QueryLength>,
    rlh: Option<QueryLength>,
    viewport_width: Option<QueryLength>,
    viewport_height: Option<QueryLength>,
    small_viewport_width: Option<QueryLength>,
    small_viewport_height: Option<QueryLength>,
    large_viewport_width: Option<QueryLength>,
    large_viewport_height: Option<QueryLength>,
    dynamic_viewport_width: Option<QueryLength>,
    dynamic_viewport_height: Option<QueryLength>,
    container_width: Option<QueryLength>,
    container_height: Option<QueryLength>,
    container_inline_size: Option<QueryLength>,
    container_block_size: Option<QueryLength>,
}

impl QueryLengthBasis {
    const MAX_RESOLVE_DEPTH: u8 = 16;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn root_font_size(mut self, value: QueryLength) -> Self {
        self.rem = Some(value);
        self
    }

    #[must_use]
    pub fn font_size(mut self, value: QueryLength) -> Self {
        self.em = Some(value);
        self
    }

    #[must_use]
    pub fn ex_size(mut self, value: QueryLength) -> Self {
        self.ex = Some(value);
        self
    }
    #[must_use]
    pub fn cap_size(mut self, value: QueryLength) -> Self {
        self.cap = Some(value);
        self
    }
    #[must_use]
    pub fn ch_size(mut self, value: QueryLength) -> Self {
        self.ch = Some(value);
        self
    }
    #[must_use]
    pub fn ic_size(mut self, value: QueryLength) -> Self {
        self.ic = Some(value);
        self
    }
    #[must_use]
    pub fn line_height(mut self, value: QueryLength) -> Self {
        self.lh = Some(value);
        self
    }
    #[must_use]
    pub fn root_ex_size(mut self, value: QueryLength) -> Self {
        self.rex = Some(value);
        self
    }
    #[must_use]
    pub fn root_cap_size(mut self, value: QueryLength) -> Self {
        self.rcap = Some(value);
        self
    }
    #[must_use]
    pub fn root_ch_size(mut self, value: QueryLength) -> Self {
        self.rch = Some(value);
        self
    }
    #[must_use]
    pub fn root_ic_size(mut self, value: QueryLength) -> Self {
        self.ric = Some(value);
        self
    }
    #[must_use]
    pub fn root_line_height(mut self, value: QueryLength) -> Self {
        self.rlh = Some(value);
        self
    }
    #[must_use]
    pub fn viewport_width(mut self, value: QueryLength) -> Self {
        self.viewport_width = Some(value);
        self
    }
    #[must_use]
    pub fn viewport_height(mut self, value: QueryLength) -> Self {
        self.viewport_height = Some(value);
        self
    }
    #[must_use]
    pub fn small_viewport_width(mut self, value: QueryLength) -> Self {
        self.small_viewport_width = Some(value);
        self
    }
    #[must_use]
    pub fn small_viewport_height(mut self, value: QueryLength) -> Self {
        self.small_viewport_height = Some(value);
        self
    }
    #[must_use]
    pub fn large_viewport_width(mut self, value: QueryLength) -> Self {
        self.large_viewport_width = Some(value);
        self
    }
    #[must_use]
    pub fn large_viewport_height(mut self, value: QueryLength) -> Self {
        self.large_viewport_height = Some(value);
        self
    }
    #[must_use]
    pub fn dynamic_viewport_width(mut self, value: QueryLength) -> Self {
        self.dynamic_viewport_width = Some(value);
        self
    }
    #[must_use]
    pub fn dynamic_viewport_height(mut self, value: QueryLength) -> Self {
        self.dynamic_viewport_height = Some(value);
        self
    }
    #[must_use]
    pub fn container_width(mut self, value: QueryLength) -> Self {
        self.container_width = Some(value);
        self
    }
    #[must_use]
    pub fn container_height(mut self, value: QueryLength) -> Self {
        self.container_height = Some(value);
        self
    }
    #[must_use]
    pub fn container_inline_size(mut self, value: QueryLength) -> Self {
        self.container_inline_size = Some(value);
        self
    }
    #[must_use]
    pub fn container_block_size(mut self, value: QueryLength) -> Self {
        self.container_block_size = Some(value);
        self
    }

    #[must_use]
    pub const fn font_size_basis(&self) -> Option<QueryLength> {
        self.em
    }
    #[must_use]
    pub const fn root_font_size_basis(&self) -> Option<QueryLength> {
        self.rem
    }
    #[must_use]
    pub const fn ex_size_basis(&self) -> Option<QueryLength> {
        self.ex
    }
    #[must_use]
    pub const fn cap_size_basis(&self) -> Option<QueryLength> {
        self.cap
    }
    #[must_use]
    pub const fn ch_size_basis(&self) -> Option<QueryLength> {
        self.ch
    }
    #[must_use]
    pub const fn ic_size_basis(&self) -> Option<QueryLength> {
        self.ic
    }
    #[must_use]
    pub const fn line_height_basis(&self) -> Option<QueryLength> {
        self.lh
    }
    #[must_use]
    pub const fn root_ex_size_basis(&self) -> Option<QueryLength> {
        self.rex
    }
    #[must_use]
    pub const fn root_cap_size_basis(&self) -> Option<QueryLength> {
        self.rcap
    }
    #[must_use]
    pub const fn root_ch_size_basis(&self) -> Option<QueryLength> {
        self.rch
    }
    #[must_use]
    pub const fn root_ic_size_basis(&self) -> Option<QueryLength> {
        self.ric
    }
    #[must_use]
    pub const fn root_line_height_basis(&self) -> Option<QueryLength> {
        self.rlh
    }
    #[must_use]
    pub const fn viewport_width_basis(&self) -> Option<QueryLength> {
        self.viewport_width
    }
    #[must_use]
    pub const fn viewport_height_basis(&self) -> Option<QueryLength> {
        self.viewport_height
    }
    #[must_use]
    pub const fn small_viewport_width_basis(&self) -> Option<QueryLength> {
        self.small_viewport_width
    }
    #[must_use]
    pub const fn small_viewport_height_basis(&self) -> Option<QueryLength> {
        self.small_viewport_height
    }
    #[must_use]
    pub const fn large_viewport_width_basis(&self) -> Option<QueryLength> {
        self.large_viewport_width
    }
    #[must_use]
    pub const fn large_viewport_height_basis(&self) -> Option<QueryLength> {
        self.large_viewport_height
    }
    #[must_use]
    pub const fn dynamic_viewport_width_basis(&self) -> Option<QueryLength> {
        self.dynamic_viewport_width
    }
    #[must_use]
    pub const fn dynamic_viewport_height_basis(&self) -> Option<QueryLength> {
        self.dynamic_viewport_height
    }
    #[must_use]
    pub const fn container_width_basis(&self) -> Option<QueryLength> {
        self.container_width
    }
    #[must_use]
    pub const fn container_height_basis(&self) -> Option<QueryLength> {
        self.container_height
    }
    #[must_use]
    pub const fn container_inline_size_basis(&self) -> Option<QueryLength> {
        self.container_inline_size
    }
    #[must_use]
    pub const fn container_block_size_basis(&self) -> Option<QueryLength> {
        self.container_block_size
    }

    #[must_use]
    pub fn resolve(&self, length: QueryLength) -> Option<f32> {
        self.resolve_with_depth(length, 0)
    }

    fn resolve_basis(&self, basis: Option<QueryLength>, depth: u8) -> Option<f32> {
        self.resolve_with_depth(basis?, depth + 1)
    }

    fn percentage_basis(
        &self,
        basis: Option<QueryLength>,
        length: QueryLength,
        depth: u8,
    ) -> Option<f32> {
        Some(length.value * self.resolve_basis(basis, depth)? / 100.0)
    }

    fn min_basis(
        &self,
        first: Option<QueryLength>,
        second: Option<QueryLength>,
        length: QueryLength,
        depth: u8,
    ) -> Option<f32> {
        Some(
            length.value
                * self
                    .resolve_basis(first, depth)?
                    .min(self.resolve_basis(second, depth)?)
                / 100.0,
        )
    }

    fn max_basis(
        &self,
        first: Option<QueryLength>,
        second: Option<QueryLength>,
        length: QueryLength,
        depth: u8,
    ) -> Option<f32> {
        Some(
            length.value
                * self
                    .resolve_basis(first, depth)?
                    .max(self.resolve_basis(second, depth)?)
                / 100.0,
        )
    }

    fn resolve_with_depth(&self, length: QueryLength, depth: u8) -> Option<f32> {
        if depth >= Self::MAX_RESOLVE_DEPTH {
            return None;
        }
        match length.unit {
            QueryLengthUnit::Px => Some(length.value),
            QueryLengthUnit::Cm => Some(length.value * 96.0 / 2.54),
            QueryLengthUnit::Mm => Some(length.value * 96.0 / 25.4),
            QueryLengthUnit::Q => Some(length.value * 96.0 / 101.6),
            QueryLengthUnit::In => Some(length.value * 96.0),
            QueryLengthUnit::Pc => Some(length.value * 16.0),
            QueryLengthUnit::Pt => Some(length.value * 96.0 / 72.0),
            QueryLengthUnit::Em => Some(length.value * self.resolve_basis(self.em, depth)?),
            QueryLengthUnit::Rem => Some(length.value * self.resolve_basis(self.rem, depth)?),
            QueryLengthUnit::Ex => Some(length.value * self.resolve_basis(self.ex, depth)?),
            QueryLengthUnit::Rex => Some(length.value * self.resolve_basis(self.rex, depth)?),
            QueryLengthUnit::Cap => Some(length.value * self.resolve_basis(self.cap, depth)?),
            QueryLengthUnit::Rcap => Some(length.value * self.resolve_basis(self.rcap, depth)?),
            QueryLengthUnit::Ch => Some(length.value * self.resolve_basis(self.ch, depth)?),
            QueryLengthUnit::Rch => Some(length.value * self.resolve_basis(self.rch, depth)?),
            QueryLengthUnit::Ic => Some(length.value * self.resolve_basis(self.ic, depth)?),
            QueryLengthUnit::Ric => Some(length.value * self.resolve_basis(self.ric, depth)?),
            QueryLengthUnit::Lh => Some(length.value * self.resolve_basis(self.lh, depth)?),
            QueryLengthUnit::Rlh => Some(length.value * self.resolve_basis(self.rlh, depth)?),
            QueryLengthUnit::Vw | QueryLengthUnit::Vi => {
                self.percentage_basis(self.viewport_width, length, depth)
            }
            QueryLengthUnit::Vh | QueryLengthUnit::Vb => {
                self.percentage_basis(self.viewport_height, length, depth)
            }
            QueryLengthUnit::Vmin => {
                self.min_basis(self.viewport_width, self.viewport_height, length, depth)
            }
            QueryLengthUnit::Vmax => {
                self.max_basis(self.viewport_width, self.viewport_height, length, depth)
            }
            QueryLengthUnit::Svw | QueryLengthUnit::Svi => {
                self.percentage_basis(self.small_viewport_width, length, depth)
            }
            QueryLengthUnit::Svh | QueryLengthUnit::Svb => {
                self.percentage_basis(self.small_viewport_height, length, depth)
            }
            QueryLengthUnit::Svmin => self.min_basis(
                self.small_viewport_width,
                self.small_viewport_height,
                length,
                depth,
            ),
            QueryLengthUnit::Svmax => self.max_basis(
                self.small_viewport_width,
                self.small_viewport_height,
                length,
                depth,
            ),
            QueryLengthUnit::Lvw | QueryLengthUnit::Lvi => {
                self.percentage_basis(self.large_viewport_width, length, depth)
            }
            QueryLengthUnit::Lvh | QueryLengthUnit::Lvb => {
                self.percentage_basis(self.large_viewport_height, length, depth)
            }
            QueryLengthUnit::Lvmin => self.min_basis(
                self.large_viewport_width,
                self.large_viewport_height,
                length,
                depth,
            ),
            QueryLengthUnit::Lvmax => self.max_basis(
                self.large_viewport_width,
                self.large_viewport_height,
                length,
                depth,
            ),
            QueryLengthUnit::Dvw | QueryLengthUnit::Dvi => {
                self.percentage_basis(self.dynamic_viewport_width, length, depth)
            }
            QueryLengthUnit::Dvh | QueryLengthUnit::Dvb => {
                self.percentage_basis(self.dynamic_viewport_height, length, depth)
            }
            QueryLengthUnit::Dvmin => self.min_basis(
                self.dynamic_viewport_width,
                self.dynamic_viewport_height,
                length,
                depth,
            ),
            QueryLengthUnit::Dvmax => self.max_basis(
                self.dynamic_viewport_width,
                self.dynamic_viewport_height,
                length,
                depth,
            ),
            QueryLengthUnit::Cqw => self.percentage_basis(self.container_width, length, depth),
            QueryLengthUnit::Cqh => self.percentage_basis(self.container_height, length, depth),
            QueryLengthUnit::Cqi => {
                self.percentage_basis(self.container_inline_size, length, depth)
            }
            QueryLengthUnit::Cqb => self.percentage_basis(self.container_block_size, length, depth),
            QueryLengthUnit::Cqmin => self.min_basis(
                self.container_inline_size,
                self.container_block_size,
                length,
                depth,
            ),
            QueryLengthUnit::Cqmax => self.max_basis(
                self.container_inline_size,
                self.container_block_size,
                length,
                depth,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution {
    value: f32,
    unit: ResolutionUnit,
}

impl Resolution {
    pub fn try_new(value: f32, unit: ResolutionUnit) -> Result<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "resolution must be finite and positive",
            ));
        }
        Ok(Self { value, unit })
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }

    #[must_use]
    pub const fn unit(self) -> ResolutionUnit {
        self.unit
    }

    #[must_use]
    pub const fn to_dppx(self) -> f32 {
        match self.unit {
            ResolutionUnit::Dpi => self.value / 96.0,
            ResolutionUnit::Dpcm => self.value * 2.54 / 96.0,
            ResolutionUnit::Dppx => self.value,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResolutionUnit {
    Dpi,
    Dpcm,
    Dppx,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ratio {
    numerator: f32,
    denominator: f32,
}

impl Ratio {
    pub fn try_new(numerator: f32, denominator: f32) -> Result<Self> {
        if !numerator.is_finite()
            || numerator < 0.0
            || !denominator.is_finite()
            || denominator <= 0.0
        {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "ratio must have a finite non-negative numerator and finite positive denominator",
            ));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    #[must_use]
    pub const fn numerator(self) -> f32 {
        self.numerator
    }

    #[must_use]
    pub const fn denominator(self) -> f32 {
        self.denominator
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.numerator / self.denominator
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NonNegativeInteger {
    value: u32,
}

impl NonNegativeInteger {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorSchemePreference {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReducedMotionPreference {
    Reduce,
    NoPreference,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReducedTransparencyPreference {
    Reduce,
    NoPreference,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContrastPreference {
    NoPreference,
    More,
    Less,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ForcedColorsMode {
    None,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HoverCapability {
    None,
    Hover,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PointerCapability {
    None,
    Coarse,
    Fine,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DisplayMode {
    Fullscreen,
    Standalone,
    MinimalUi,
    Browser,
    PictureInPicture,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MediaFeatureQuery {
    Width(RangeFeature<QueryLength>),
    Height(RangeFeature<QueryLength>),
    Resolution(RangeFeature<Resolution>),
    Color(RangeFeature<NonNegativeInteger>),
    Monochrome(RangeFeature<NonNegativeInteger>),
    Orientation(Orientation),
    PrefersColorScheme(ColorSchemePreference),
    PrefersReducedMotion(ReducedMotionPreference),
    PrefersReducedTransparency(ReducedTransparencyPreference),
    PrefersContrast(ContrastPreference),
    ForcedColors(ForcedColorsMode),
    Hover(HoverCapability),
    AnyHover(HoverCapability),
    Pointer(PointerCapability),
    AnyPointer(PointerCapability),
    DisplayMode(DisplayMode),
}

impl MediaFeatureQuery {
    #[must_use]
    pub fn matches(&self, environment: &MediaEnvironment) -> bool {
        match self {
            Self::Width(query) => {
                matches_length_range(query, environment.width, environment.length_basis())
            }
            Self::Height(query) => {
                matches_length_range(query, environment.height, environment.length_basis())
            }
            Self::Resolution(query) => matches_float_range(
                query,
                environment.resolution.map(Resolution::to_dppx),
                query.value().to_dppx(),
            ),
            Self::Color(query) => matches_float_range(
                query,
                environment.color.map(|value| value.value() as f32),
                query.value().value() as f32,
            ),
            Self::Monochrome(query) => matches_float_range(
                query,
                environment.monochrome.map(|value| value.value() as f32),
                query.value().value() as f32,
            ),
            Self::Orientation(query) => environment
                .orientation()
                .is_some_and(|orientation| orientation == *query),
            Self::PrefersColorScheme(query) => environment
                .prefers_color_scheme
                .is_some_and(|value| value == *query),
            Self::PrefersReducedMotion(query) => environment
                .prefers_reduced_motion
                .is_some_and(|value| value == *query),
            Self::PrefersReducedTransparency(query) => environment
                .prefers_reduced_transparency
                .is_some_and(|value| value == *query),
            Self::PrefersContrast(query) => environment
                .prefers_contrast
                .is_some_and(|value| value == *query),
            Self::ForcedColors(query) => environment
                .forced_colors
                .is_some_and(|value| value == *query),
            Self::Hover(query) => environment.hover.is_some_and(|value| value == *query),
            Self::AnyHover(query) => environment.any_hover.is_some_and(|value| value == *query),
            Self::Pointer(query) => environment.pointer.is_some_and(|value| value == *query),
            Self::AnyPointer(query) => environment.any_pointer.is_some_and(|value| value == *query),
            Self::DisplayMode(query) => environment
                .display_mode
                .is_some_and(|value| value == *query),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaEnvironment {
    media_type: MediaType,
    width: Option<QueryLength>,
    height: Option<QueryLength>,
    length_basis: QueryLengthBasis,
    resolution: Option<Resolution>,
    color: Option<NonNegativeInteger>,
    monochrome: Option<NonNegativeInteger>,
    orientation: Option<Orientation>,
    prefers_color_scheme: Option<ColorSchemePreference>,
    prefers_reduced_motion: Option<ReducedMotionPreference>,
    prefers_reduced_transparency: Option<ReducedTransparencyPreference>,
    prefers_contrast: Option<ContrastPreference>,
    forced_colors: Option<ForcedColorsMode>,
    hover: Option<HoverCapability>,
    any_hover: Option<HoverCapability>,
    pointer: Option<PointerCapability>,
    any_pointer: Option<PointerCapability>,
    display_mode: Option<DisplayMode>,
}

impl MediaEnvironment {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn media_type(mut self, media_type: MediaType) -> Self {
        self.media_type = media_type;
        self
    }

    #[must_use]
    pub const fn width(mut self, width: QueryLength) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub const fn height(mut self, height: QueryLength) -> Self {
        self.height = Some(height);
        self
    }

    #[must_use]
    pub fn with_length_basis(mut self, length_basis: QueryLengthBasis) -> Self {
        self.length_basis = length_basis;
        self
    }

    #[must_use]
    pub const fn resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    #[must_use]
    pub const fn color(mut self, color: NonNegativeInteger) -> Self {
        self.color = Some(color);
        self
    }

    #[must_use]
    pub const fn monochrome(mut self, monochrome: NonNegativeInteger) -> Self {
        self.monochrome = Some(monochrome);
        self
    }

    #[must_use]
    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    #[must_use]
    pub const fn prefers_color_scheme(mut self, value: ColorSchemePreference) -> Self {
        self.prefers_color_scheme = Some(value);
        self
    }

    #[must_use]
    pub const fn prefers_reduced_motion(mut self, value: ReducedMotionPreference) -> Self {
        self.prefers_reduced_motion = Some(value);
        self
    }

    #[must_use]
    pub const fn prefers_reduced_transparency(
        mut self,
        value: ReducedTransparencyPreference,
    ) -> Self {
        self.prefers_reduced_transparency = Some(value);
        self
    }

    #[must_use]
    pub const fn prefers_contrast(mut self, value: ContrastPreference) -> Self {
        self.prefers_contrast = Some(value);
        self
    }

    #[must_use]
    pub const fn forced_colors(mut self, value: ForcedColorsMode) -> Self {
        self.forced_colors = Some(value);
        self
    }

    #[must_use]
    pub const fn hover(mut self, value: HoverCapability) -> Self {
        self.hover = Some(value);
        self
    }

    #[must_use]
    pub const fn any_hover(mut self, value: HoverCapability) -> Self {
        self.any_hover = Some(value);
        self
    }

    #[must_use]
    pub const fn pointer(mut self, value: PointerCapability) -> Self {
        self.pointer = Some(value);
        self
    }

    #[must_use]
    pub const fn any_pointer(mut self, value: PointerCapability) -> Self {
        self.any_pointer = Some(value);
        self
    }

    #[must_use]
    pub const fn display_mode(mut self, value: DisplayMode) -> Self {
        self.display_mode = Some(value);
        self
    }

    #[must_use]
    pub const fn media_type_fact(&self) -> MediaType {
        self.media_type
    }
    #[must_use]
    pub const fn width_fact(&self) -> Option<QueryLength> {
        self.width
    }
    #[must_use]
    pub const fn height_fact(&self) -> Option<QueryLength> {
        self.height
    }
    #[must_use]
    pub const fn length_basis(&self) -> &QueryLengthBasis {
        &self.length_basis
    }
    #[must_use]
    pub const fn resolution_fact(&self) -> Option<Resolution> {
        self.resolution
    }
    #[must_use]
    pub const fn color_fact(&self) -> Option<NonNegativeInteger> {
        self.color
    }
    #[must_use]
    pub const fn monochrome_fact(&self) -> Option<NonNegativeInteger> {
        self.monochrome
    }
    #[must_use]
    pub const fn orientation_fact(&self) -> Option<Orientation> {
        self.orientation
    }
    #[must_use]
    pub const fn prefers_color_scheme_fact(&self) -> Option<ColorSchemePreference> {
        self.prefers_color_scheme
    }
    #[must_use]
    pub const fn prefers_reduced_motion_fact(&self) -> Option<ReducedMotionPreference> {
        self.prefers_reduced_motion
    }
    #[must_use]
    pub const fn prefers_reduced_transparency_fact(&self) -> Option<ReducedTransparencyPreference> {
        self.prefers_reduced_transparency
    }
    #[must_use]
    pub const fn prefers_contrast_fact(&self) -> Option<ContrastPreference> {
        self.prefers_contrast
    }
    #[must_use]
    pub const fn forced_colors_fact(&self) -> Option<ForcedColorsMode> {
        self.forced_colors
    }
    #[must_use]
    pub const fn hover_fact(&self) -> Option<HoverCapability> {
        self.hover
    }
    #[must_use]
    pub const fn any_hover_fact(&self) -> Option<HoverCapability> {
        self.any_hover
    }
    #[must_use]
    pub const fn pointer_fact(&self) -> Option<PointerCapability> {
        self.pointer
    }
    #[must_use]
    pub const fn any_pointer_fact(&self) -> Option<PointerCapability> {
        self.any_pointer
    }
    #[must_use]
    pub const fn display_mode_fact(&self) -> Option<DisplayMode> {
        self.display_mode
    }

    #[must_use]
    pub fn orientation(&self) -> Option<Orientation> {
        if let Some(orientation) = self.orientation {
            return Some(orientation);
        }
        let width = self.width?.to_css_px(&self.length_basis)?;
        let height = self.height?.to_css_px(&self.length_basis)?;
        Some(if width >= height {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        })
    }
}

fn matches_length_range(
    query: &RangeFeature<QueryLength>,
    fact: Option<QueryLength>,
    basis: &QueryLengthBasis,
) -> bool {
    let Some(actual) = fact.and_then(|value| value.to_css_px(basis)) else {
        return false;
    };
    let Some(expected) = query.value().to_css_px(basis) else {
        return false;
    };
    compare_values(query.comparison(), actual, expected)
}

fn matches_float_range<T>(query: &RangeFeature<T>, actual: Option<f32>, expected: f32) -> bool {
    actual.is_some_and(|actual| compare_values(query.comparison(), actual, expected))
}

fn compare_values(comparison: Option<QueryComparison>, actual: f32, expected: f32) -> bool {
    match comparison.unwrap_or(QueryComparison::Equal) {
        QueryComparison::LessThan => actual < expected,
        QueryComparison::LessThanOrEqual => actual <= expected,
        QueryComparison::Equal => actual == expected,
        QueryComparison::GreaterThanOrEqual => actual >= expected,
        QueryComparison::GreaterThan => actual > expected,
    }
}

fn validate_condition_ident(value: &str, label: &'static str) -> Result<()> {
    if value.is_empty() || value.contains('\0') {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{label} cannot be empty or contain U+0000"),
        ));
    }
    if matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    ) {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{label} cannot be a CSS-wide keyword"),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ContainerName {
    value: String,
}

impl ContainerName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_condition_ident(&value, "container name")?;
        if matches!(
            value.to_ascii_lowercase().as_str(),
            "none" | "and" | "or" | "not" | "style"
        ) {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "container name is reserved",
            ));
        }
        Ok(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContainerFacts {
    name: Option<ContainerName>,
    width: Option<QueryLength>,
    height: Option<QueryLength>,
    inline_size: Option<QueryLength>,
    block_size: Option<QueryLength>,
    length_basis: QueryLengthBasis,
    aspect_ratio: Option<Ratio>,
    orientation: Option<Orientation>,
    custom_properties: BTreeMap<CustomPropertyName, AuthoredTokens>,
}

impl ContainerFacts {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn name(mut self, name: ContainerName) -> Self {
        self.name = Some(name);
        self
    }

    #[must_use]
    pub const fn width(mut self, width: QueryLength) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub const fn height(mut self, height: QueryLength) -> Self {
        self.height = Some(height);
        self
    }

    #[must_use]
    pub const fn inline_size(mut self, inline_size: QueryLength) -> Self {
        self.inline_size = Some(inline_size);
        self
    }

    #[must_use]
    pub const fn block_size(mut self, block_size: QueryLength) -> Self {
        self.block_size = Some(block_size);
        self
    }

    #[must_use]
    pub fn with_length_basis(mut self, length_basis: QueryLengthBasis) -> Self {
        self.length_basis = length_basis;
        self
    }

    #[must_use]
    pub const fn aspect_ratio(mut self, aspect_ratio: Ratio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    #[must_use]
    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    #[must_use]
    pub fn custom_property(mut self, name: CustomPropertyName, value: AuthoredTokens) -> Self {
        self.custom_properties.insert(name, value);
        self
    }

    #[must_use]
    pub const fn name_fact(&self) -> Option<&ContainerName> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn width_fact(&self) -> Option<QueryLength> {
        self.width
    }

    #[must_use]
    pub const fn height_fact(&self) -> Option<QueryLength> {
        self.height
    }

    #[must_use]
    pub const fn inline_size_fact(&self) -> Option<QueryLength> {
        self.inline_size
    }

    #[must_use]
    pub const fn block_size_fact(&self) -> Option<QueryLength> {
        self.block_size
    }

    #[must_use]
    pub const fn length_basis(&self) -> &QueryLengthBasis {
        &self.length_basis
    }

    #[must_use]
    pub const fn aspect_ratio_fact(&self) -> Option<Ratio> {
        self.aspect_ratio
    }

    #[must_use]
    pub const fn orientation_fact(&self) -> Option<Orientation> {
        self.orientation
    }

    #[must_use]
    pub fn orientation(&self) -> Option<Orientation> {
        if let Some(orientation) = self.orientation {
            return Some(orientation);
        }
        let width = self.width?.to_css_px(&self.length_basis)?;
        let height = self.height?.to_css_px(&self.length_basis)?;
        Some(if width >= height {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        })
    }

    #[must_use]
    pub fn custom_property_fact(&self, name: &CustomPropertyName) -> Option<&AuthoredTokens> {
        self.custom_properties.get(name)
    }

    #[must_use]
    pub const fn custom_properties(&self) -> &BTreeMap<CustomPropertyName, AuthoredTokens> {
        &self.custom_properties
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerCondition {
    Feature(ContainerFeatureQuery),
    Style(ContainerStyleQuery),
    Not(Box<ContainerCondition>),
    And(ContainerConditionList),
    Or(ContainerConditionList),
}

impl ContainerCondition {
    #[must_use]
    pub fn matches(&self, facts: &ContainerFacts) -> bool {
        match self {
            Self::Feature(query) => query.matches(facts),
            Self::Style(query) => query.matches(facts),
            Self::Not(condition) => !condition.matches(facts),
            Self::And(conditions) => conditions
                .conditions()
                .iter()
                .all(|condition| condition.matches(facts)),
            Self::Or(conditions) => conditions
                .conditions()
                .iter()
                .any(|condition| condition.matches(facts)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerConditionList {
    conditions: Vec<ContainerCondition>,
}

impl ContainerConditionList {
    pub fn try_new(conditions: impl IntoIterator<Item = ContainerCondition>) -> Result<Self> {
        let conditions = conditions.into_iter().collect::<Vec<_>>();
        if conditions.len() < 2 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "container condition list requires at least two conditions",
            ));
        }
        Ok(Self { conditions })
    }

    #[must_use]
    pub fn conditions(&self) -> &[ContainerCondition] {
        &self.conditions
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerFeatureQuery {
    Width(RangeFeature<QueryLength>),
    Height(RangeFeature<QueryLength>),
    InlineSize(RangeFeature<QueryLength>),
    BlockSize(RangeFeature<QueryLength>),
    AspectRatio(RangeFeature<Ratio>),
    Orientation(Orientation),
}

impl ContainerFeatureQuery {
    #[must_use]
    pub fn matches(&self, facts: &ContainerFacts) -> bool {
        match self {
            Self::Width(query) => matches_length_range(query, facts.width, facts.length_basis()),
            Self::Height(query) => matches_length_range(query, facts.height, facts.length_basis()),
            Self::InlineSize(query) => {
                matches_length_range(query, facts.inline_size, facts.length_basis())
            }
            Self::BlockSize(query) => {
                matches_length_range(query, facts.block_size, facts.length_basis())
            }
            Self::AspectRatio(query) => matches_float_range(
                query,
                facts.aspect_ratio.map(Ratio::value),
                query.value().value(),
            ),
            Self::Orientation(query) => facts
                .orientation()
                .is_some_and(|orientation| orientation == *query),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerStyleQuery {
    CustomPropertyPresence(CustomPropertyName),
    CustomPropertyValue {
        name: CustomPropertyName,
        value: AuthoredTokens,
    },
}

impl ContainerStyleQuery {
    #[must_use]
    pub fn matches(&self, facts: &ContainerFacts) -> bool {
        match self {
            Self::CustomPropertyPresence(name) => facts.custom_properties.contains_key(name),
            Self::CustomPropertyValue { name, value } => facts
                .custom_property_fact(name)
                .is_some_and(|fact| fact == value),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionFacts {
    media: MediaEnvironment,
    container: Option<ContainerFacts>,
}

impl ConditionFacts {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn media(mut self, media: MediaEnvironment) -> Self {
        self.media = media;
        self
    }

    #[must_use]
    pub fn container(mut self, container: ContainerFacts) -> Self {
        self.container = Some(container);
        self
    }

    #[must_use]
    pub const fn media_environment(&self) -> &MediaEnvironment {
        &self.media
    }

    #[must_use]
    pub const fn container_facts(&self) -> Option<&ContainerFacts> {
        self.container.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Media(MediaQueryList),
    Container(ContainerCondition),
}

impl Condition {
    #[must_use]
    pub fn media(query: MediaQueryList) -> Self {
        Self::Media(query)
    }

    #[must_use]
    pub fn container(condition: ContainerCondition) -> Self {
        Self::Container(condition)
    }

    #[must_use]
    pub fn matches(&self, facts: &ConditionFacts) -> bool {
        match self {
            Self::Media(query) => query.matches(facts.media_environment()),
            Self::Container(condition) => facts
                .container_facts()
                .is_some_and(|container| condition.matches(container)),
        }
    }

    #[must_use]
    pub const fn is_media(&self) -> bool {
        matches!(self, Self::Media(_))
    }

    #[must_use]
    pub const fn is_container(&self) -> bool {
        matches!(self, Self::Container(_))
    }

    #[must_use]
    pub fn matches_all(conditions: &[Self], facts: &ConditionFacts) -> bool {
        conditions.iter().all(|condition| condition.matches(facts))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AuthoredTokens, CustomPropertyName};

    use super::*;

    #[test]
    fn media_query_lists_match_environment_facts() {
        let query = MediaQueryList::try_new([MediaQuery::Condition(MediaCondition::Feature(
            MediaFeatureQuery::Width(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(640.0, QueryLengthUnit::Px).unwrap(),
            )),
        ))])
        .unwrap();

        let matching = MediaEnvironment::new()
            .media_type(MediaType::Screen)
            .width(QueryLength::try_new(800.0, QueryLengthUnit::Px).unwrap())
            .height(QueryLength::try_new(600.0, QueryLengthUnit::Px).unwrap());
        let too_small = MediaEnvironment::new()
            .media_type(MediaType::Screen)
            .width(QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap());

        assert!(query.matches(&matching));
        assert!(!query.matches(&too_small));
    }

    #[test]
    fn media_query_typed_queries_support_modifiers_types_and_conditions() {
        let screen = MediaQuery::Typed(TypedMediaQuery::new(
            None,
            MediaType::Screen,
            Some(MediaCondition::Feature(MediaFeatureQuery::Orientation(
                Orientation::Landscape,
            ))),
        ));
        let not_print = MediaQuery::Typed(TypedMediaQuery::new(
            Some(MediaQueryModifier::Not),
            MediaType::Print,
            None,
        ));
        let only_screen = MediaQuery::Typed(TypedMediaQuery::new(
            Some(MediaQueryModifier::Only),
            MediaType::Screen,
            None,
        ));
        let list = MediaQueryList::try_new([screen, not_print, only_screen]).unwrap();
        let environment = MediaEnvironment::new()
            .media_type(MediaType::Screen)
            .width(QueryLength::try_new(800.0, QueryLengthUnit::Px).unwrap())
            .height(QueryLength::try_new(400.0, QueryLengthUnit::Px).unwrap());

        assert!(list.matches(&environment));
    }

    #[test]
    fn media_query_condition_lists_require_two_conditions() {
        assert!(
            MediaConditionList::try_new([MediaCondition::Feature(MediaFeatureQuery::Hover(
                HoverCapability::Hover
            ),)])
            .is_err()
        );

        let conditions = MediaConditionList::try_new([
            MediaCondition::Feature(MediaFeatureQuery::Hover(HoverCapability::Hover)),
            MediaCondition::Not(Box::new(MediaCondition::Feature(
                MediaFeatureQuery::ForcedColors(ForcedColorsMode::Active),
            ))),
        ])
        .unwrap();

        let environment = MediaEnvironment::new()
            .hover(HoverCapability::Hover)
            .forced_colors(ForcedColorsMode::None);
        assert!(MediaCondition::And(conditions).matches(&environment));
    }

    #[test]
    fn media_query_lengths_preserve_css_units_and_require_basis_for_resolution() {
        let query = QueryLength::try_new(40.0, QueryLengthUnit::Rem).unwrap();
        let without_basis = MediaEnvironment::new()
            .width(QueryLength::try_new(800.0, QueryLengthUnit::Px).unwrap());
        let with_basis = without_basis.clone().with_length_basis(
            QueryLengthBasis::new()
                .root_font_size(QueryLength::try_new(20.0, QueryLengthUnit::Px).unwrap()),
        );

        assert_eq!(query.unit(), QueryLengthUnit::Rem);
        assert_eq!(query.to_css_px(with_basis.length_basis()), Some(800.0));
        assert_eq!(query.to_css_px(without_basis.length_basis()), None);
    }

    #[test]
    fn condition_facts_match_media_and_container_conditions() {
        let query = MediaQueryList::try_new([MediaQuery::Condition(MediaCondition::Feature(
            MediaFeatureQuery::Hover(HoverCapability::Hover),
        ))])
        .unwrap();
        let container =
            ContainerCondition::Feature(ContainerFeatureQuery::Width(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap(),
            )));
        let facts = ConditionFacts::new()
            .media(MediaEnvironment::new().hover(HoverCapability::Hover))
            .container(
                ContainerFacts::new()
                    .width(QueryLength::try_new(400.0, QueryLengthUnit::Px).unwrap()),
            );

        assert!(Condition::media(query).matches(&facts));
        assert!(Condition::container(container).matches(&facts));
    }

    #[test]
    fn container_conditions_do_not_match_without_container_facts() {
        let condition = Condition::container(ContainerCondition::Feature(
            ContainerFeatureQuery::Width(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap(),
            )),
        ));

        assert!(!condition.matches(&ConditionFacts::new()));
    }

    #[test]
    fn container_conditions_match_named_container_facts() {
        let name = ContainerName::try_new("sidebar").unwrap();
        let condition =
            ContainerCondition::Feature(ContainerFeatureQuery::InlineSize(RangeFeature::new(
                Some(QueryComparison::GreaterThanOrEqual),
                QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap(),
            )));
        let facts = ContainerFacts::new()
            .name(name.clone())
            .inline_size(QueryLength::try_new(400.0, QueryLengthUnit::Px).unwrap())
            .block_size(QueryLength::try_new(600.0, QueryLengthUnit::Px).unwrap());

        assert!(condition.matches(&facts));
        assert_eq!(facts.name_fact(), Some(&name));
    }

    #[test]
    fn container_conditions_require_two_children_for_and_or() {
        assert!(
            ContainerConditionList::try_new([ContainerCondition::Feature(
                ContainerFeatureQuery::Orientation(Orientation::Portrait),
            )])
            .is_err()
        );
    }

    mod container_conditions {
        use super::*;

        #[test]
        fn container_style_queries_match_custom_property_facts() {
            let name = CustomPropertyName::try_new("--theme").unwrap();
            let value = AuthoredTokens::new("dark");
            let facts = ContainerFacts::new().custom_property(name.clone(), value.clone());

            assert!(
                ContainerCondition::Style(ContainerStyleQuery::CustomPropertyPresence(
                    name.clone(),
                ))
                .matches(&facts)
            );
            assert!(
                ContainerCondition::Style(ContainerStyleQuery::CustomPropertyValue { name, value })
                    .matches(&facts)
            );
        }
    }
}
