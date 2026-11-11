use super::{DefaultScalar, LayoutScalar, scalar::canonical_zero};
use core::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatioOf<S: LayoutScalar = DefaultScalar>(S);

pub type AspectRatio = AspectRatioOf<DefaultScalar>;

impl<S: LayoutScalar> AspectRatioOf<S> {
    #[must_use]
    pub fn new(value: S) -> Option<Self> {
        (value.is_finite() && value > S::ZERO).then_some(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridLine(isize);

impl GridLine {
    #[must_use]
    pub const fn new(value: isize) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    #[must_use]
    pub const fn get(self) -> isize {
        self.0
    }
}

impl TryFrom<isize> for GridLine {
    type Error = ();

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridSpan(NonZeroUsize);

impl GridSpan {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

impl TryFrom<usize> for GridSpan {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

/// Physical space supplied by the containing boundary, independent of sizing operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AvailableSpaceOf<S: LayoutScalar = DefaultScalar> {
    Definite(NonNegativeFiniteOf<S>),
    Indefinite,
}

pub type AvailableSpace = AvailableSpaceOf<DefaultScalar>;

impl<S: LayoutScalar> AvailableSpaceOf<S> {
    #[must_use]
    pub const fn definite_size(self) -> Option<NonNegativeFiniteOf<S>> {
        match self {
            Self::Definite(value) => Some(value),
            Self::Indefinite => None,
        }
    }

    #[must_use]
    pub(crate) fn roughly_eq(self, other: Self) -> bool {
        match (self, other) {
            (Self::Definite(a), Self::Definite(b)) => (a.get() - b.get()).abs() < S::EPSILON,
            (Self::Indefinite, Self::Indefinite) => true,
            _ => false,
        }
    }
}

/// The requested sizing operation, not a containing dimension or contribution role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SizingOperation {
    Normal,
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisSizingRequestOf<S: LayoutScalar = DefaultScalar> {
    space: AvailableSpaceOf<S>,
    operation: SizingOperation,
}

pub type AxisSizingRequest = AxisSizingRequestOf<DefaultScalar>;

impl<S: LayoutScalar> AxisSizingRequestOf<S> {
    #[must_use]
    pub const fn new(space: AvailableSpaceOf<S>, operation: SizingOperation) -> Self {
        Self { space, operation }
    }

    #[must_use]
    pub const fn space(self) -> AvailableSpaceOf<S> {
        self.space
    }

    #[must_use]
    pub const fn operation(self) -> SizingOperation {
        self.operation
    }

    #[must_use]
    pub const fn with_space(self, space: AvailableSpaceOf<S>) -> Self {
        Self { space, ..self }
    }

    #[must_use]
    pub const fn with_operation(self, operation: SizingOperation) -> Self {
        Self { operation, ..self }
    }
}

/// A qualified finite candidate, proven absence of a finite bound, or missing authority.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrthogonalFallbackLimitOf<S: LayoutScalar = DefaultScalar> {
    Unknown,
    Unbounded,
    Finite(NonNegativeFiniteOf<S>),
}

pub type OrthogonalFallbackLimit = OrthogonalFallbackLimitOf<DefaultScalar>;

impl<S: LayoutScalar> OrthogonalFallbackLimitOf<S> {
    /// Qualifies the containing block's inner maximum, floored by its inner minimum.
    /// Inputs are definite bounds; absence means a proven non-finite bound.
    #[must_use]
    pub fn from_containing_bounds(
        minimum: Option<NonNegativeFiniteOf<S>>,
        maximum: Option<NonNegativeFiniteOf<S>>,
    ) -> Self {
        let Some(mut value) = maximum else {
            return Self::Unbounded;
        };
        if let Some(minimum) = minimum
            && minimum.get() > value.get()
        {
            value = minimum;
        }
        Self::Finite(value)
    }

    /// Qualifies ONE actual scrollport, without retaining any outer scrollport.
    /// A finite used Auto size is not a definite inner size. Producers must use
    /// their actual definite/derived phase authority before constructing these inputs.
    #[must_use]
    pub fn from_scrollport(
        definite_inner_size: Option<NonNegativeFiniteOf<S>>,
        minimum: Option<NonNegativeFiniteOf<S>>,
        maximum: Option<NonNegativeFiniteOf<S>>,
    ) -> Self {
        let Some(mut value) = definite_inner_size.or(maximum) else {
            return Self::Unbounded;
        };
        if let Some(maximum) = maximum
            && value.get() > maximum.get()
        {
            value = maximum;
        }
        if let Some(minimum) = minimum
            && value.get() < minimum.get()
        {
            value = minimum;
        }
        Self::Finite(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NearestScrollportOf<S: LayoutScalar = DefaultScalar> {
    Unknown,
    Absent,
    Present(super::Size<OrthogonalFallbackLimitOf<S>>),
}

pub type NearestScrollport = NearestScrollportOf<DefaultScalar>;

/// An actual newly entered scroll container replaces the WHOLE nearest frame,
/// including Unknown and Unbounded axes. A non-scroll container preserves it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollportFallbackChangeOf<S: LayoutScalar = DefaultScalar> {
    Preserve,
    Enter(super::Size<OrthogonalFallbackLimitOf<S>>),
}

pub type ScrollportFallbackChange = ScrollportFallbackChangeOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrthogonalFallbackErrorOf<S: LayoutScalar = DefaultScalar> {
    MissingInitialContainingBlock { axis: super::PhysicalAxis },
    MissingContainingBounds { axis: super::PhysicalAxis },
    MissingNearestScrollport { axis: super::PhysicalAxis },
    MissingScrollportLimit { axis: super::PhysicalAxis },
    InvalidContentSpace { axis: super::PhysicalAxis, value: S },
}

pub type OrthogonalFallbackError = OrthogonalFallbackErrorOf<DefaultScalar>;

impl<S: LayoutScalar> core::fmt::Display for OrthogonalFallbackErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingInitialContainingBlock { .. } => {
                f.write_str("orthogonal fallback needs initial containing-block authority")
            }
            Self::MissingContainingBounds { .. } => {
                f.write_str("orthogonal fallback needs containing-block bound authority")
            }
            Self::MissingNearestScrollport { .. } => {
                f.write_str("orthogonal fallback needs nearest-scrollport authority")
            }
            Self::MissingScrollportLimit { .. } => {
                f.write_str("orthogonal fallback needs the nearest scrollport's axis limit")
            }
            Self::InvalidContentSpace { .. } => {
                f.write_str("orthogonal fallback content-space arithmetic is invalid")
            }
        }
    }
}

impl<S: LayoutScalar> std::error::Error for OrthogonalFallbackErrorOf<S> {}

/// Original ICB authority, current nearest scrollport, and local containing limits.
/// No collapsed ancestor minimum is retained or propagated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrthogonalFallbackOf<S: LayoutScalar = DefaultScalar> {
    initial_containing_block: super::Size<Option<NonNegativeFiniteOf<S>>>,
    nearest_scrollport: NearestScrollportOf<S>,
    containing_limits: super::Size<OrthogonalFallbackLimitOf<S>>,
}

pub type OrthogonalFallback = OrthogonalFallbackOf<DefaultScalar>;

impl<S: LayoutScalar> OrthogonalFallbackOf<S> {
    #[must_use]
    pub const fn new(
        initial_containing_block: super::Size<Option<NonNegativeFiniteOf<S>>>,
        nearest_scrollport: NearestScrollportOf<S>,
        containing_limits: super::Size<OrthogonalFallbackLimitOf<S>>,
    ) -> Self {
        Self {
            initial_containing_block,
            nearest_scrollport,
            containing_limits,
        }
    }

    #[must_use]
    pub const fn initial_containing_block(self) -> super::Size<Option<NonNegativeFiniteOf<S>>> {
        self.initial_containing_block
    }

    #[must_use]
    pub const fn nearest_scrollport(self) -> NearestScrollportOf<S> {
        self.nearest_scrollport
    }

    #[must_use]
    pub const fn containing_limits(self) -> super::Size<OrthogonalFallbackLimitOf<S>> {
        self.containing_limits
    }

    #[must_use]
    pub const fn for_child(
        self,
        containing_limits: super::Size<OrthogonalFallbackLimitOf<S>>,
        scrollport: ScrollportFallbackChangeOf<S>,
    ) -> Self {
        let nearest_scrollport = match scrollport {
            ScrollportFallbackChangeOf::Preserve => self.nearest_scrollport,
            ScrollportFallbackChangeOf::Enter(limits) => NearestScrollportOf::Present(limits),
        };
        Self {
            containing_limits,
            nearest_scrollport,
            ..self
        }
    }

    pub fn resolve(
        self,
        axis: super::PhysicalAxis,
    ) -> Result<NonNegativeFiniteOf<S>, OrthogonalFallbackErrorOf<S>> {
        let mut smallest = orthogonal_axis(self.initial_containing_block, axis);
        let mut missing = smallest
            .is_none()
            .then_some(OrthogonalFallbackErrorOf::MissingInitialContainingBlock { axis });
        let mut consider = |limit, reason| match limit {
            OrthogonalFallbackLimitOf::Unknown => {
                if missing.is_none() {
                    missing = Some(reason);
                }
            }
            OrthogonalFallbackLimitOf::Unbounded => {}
            OrthogonalFallbackLimitOf::Finite(value) => {
                if smallest.is_none_or(|old| value.get() < old.get()) {
                    smallest = Some(value);
                }
            }
        };
        consider(
            orthogonal_axis(self.containing_limits, axis),
            OrthogonalFallbackErrorOf::MissingContainingBounds { axis },
        );
        match self.nearest_scrollport {
            NearestScrollportOf::Unknown => consider(
                OrthogonalFallbackLimitOf::Unknown,
                OrthogonalFallbackErrorOf::MissingNearestScrollport { axis },
            ),
            NearestScrollportOf::Absent => {}
            NearestScrollportOf::Present(limits) => consider(
                orthogonal_axis(limits, axis),
                OrthogonalFallbackErrorOf::MissingScrollportLimit { axis },
            ),
        }
        // Every candidate is nonnegative. A zero determines the minimum
        // even when another source has not supplied its authority.
        if smallest.is_some_and(|value| value.get() == S::ZERO) {
            return Ok(NonNegativeFiniteOf::ZERO);
        }
        if let Some(error) = missing {
            return Err(error);
        }
        Ok(smallest.expect("known ICB authority supplies a finite candidate"))
    }
}

fn orthogonal_axis<T>(size: super::Size<T>, axis: super::PhysicalAxis) -> T {
    match axis {
        super::PhysicalAxis::Horizontal => size.width,
        super::PhysicalAxis::Vertical => size.height,
    }
}

/// Final opaque content-space projection. It cannot be requalified for descendants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContentSizingRequestOf<S: LayoutScalar = DefaultScalar> {
    axes: super::Size<AxisSizingRequestOf<S>>,
    orthogonal_fallback: super::Size<Result<NonNegativeFiniteOf<S>, OrthogonalFallbackErrorOf<S>>>,
}

pub type ContentSizingRequest = ContentSizingRequestOf<DefaultScalar>;

impl<S: LayoutScalar> ContentSizingRequestOf<S> {
    pub(crate) const fn new(
        axes: super::Size<AxisSizingRequestOf<S>>,
        orthogonal_fallback: super::Size<
            Result<NonNegativeFiniteOf<S>, OrthogonalFallbackErrorOf<S>>,
        >,
    ) -> Self {
        Self {
            axes,
            orthogonal_fallback,
        }
    }

    #[must_use]
    pub const fn axes(self) -> super::Size<AxisSizingRequestOf<S>> {
        self.axes
    }

    #[must_use]
    pub const fn orthogonal_fallback(
        self,
    ) -> super::Size<Result<NonNegativeFiniteOf<S>, OrthogonalFallbackErrorOf<S>>> {
        self.orthogonal_fallback
    }
}

/// Physical axis requests and their independently authenticated recursive fallback context.
/// Same-box queries may change axes while retaining this context. Child producers
/// must replace local limits and enter any actual new scroll container through
/// OrthogonalFallbackOf::for_child; neither percentages nor used Auto sizes imply
/// a definite fallback fact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SizingRequestOf<S: LayoutScalar = DefaultScalar> {
    axes: super::Size<AxisSizingRequestOf<S>>,
    orthogonal_fallback: OrthogonalFallbackOf<S>,
}

pub type SizingRequest = SizingRequestOf<DefaultScalar>;

impl<S: LayoutScalar> SizingRequestOf<S> {
    #[must_use]
    pub const fn new(
        axes: super::Size<AxisSizingRequestOf<S>>,
        orthogonal_fallback: OrthogonalFallbackOf<S>,
    ) -> Self {
        Self {
            axes,
            orthogonal_fallback,
        }
    }

    #[must_use]
    pub const fn axes(self) -> super::Size<AxisSizingRequestOf<S>> {
        self.axes
    }

    #[must_use]
    pub const fn orthogonal_fallback(self) -> OrthogonalFallbackOf<S> {
        self.orthogonal_fallback
    }

    #[must_use]
    pub const fn with_axes(self, axes: super::Size<AxisSizingRequestOf<S>>) -> Self {
        Self { axes, ..self }
    }

    #[must_use]
    pub const fn with_orthogonal_fallback(
        self,
        orthogonal_fallback: OrthogonalFallbackOf<S>,
    ) -> Self {
        Self {
            orthogonal_fallback,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FiniteScalarErrorOf<S: LayoutScalar = DefaultScalar> {
    NonFinite { value: S },
}

impl<S: LayoutScalar> core::fmt::Display for FiniteScalarErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFinite { .. } => f.write_str("scalar must be finite"),
        }
    }
}

impl<S: LayoutScalar> std::error::Error for FiniteScalarErrorOf<S> {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NonNegativeFiniteScalarErrorOf<S: LayoutScalar = DefaultScalar> {
    NonFinite { value: S },
    Negative { value: S },
}

impl<S: LayoutScalar> core::fmt::Display for NonNegativeFiniteScalarErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFinite { .. } => f.write_str("scalar must be finite"),
            Self::Negative { .. } => f.write_str("scalar must be non-negative"),
        }
    }
}

impl<S: LayoutScalar> std::error::Error for NonNegativeFiniteScalarErrorOf<S> {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonNegativeFiniteOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

impl<S: LayoutScalar> NonNegativeFiniteOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        if !value.is_finite() {
            return Err(NonNegativeFiniteScalarErrorOf::NonFinite { value });
        }

        if value < S::ZERO {
            return Err(NonNegativeFiniteScalarErrorOf::Negative { value });
        }

        Ok(Self {
            value: canonical_zero(value),
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthPercentageErrorOf<S: LayoutScalar = DefaultScalar> {
    InvalidAbsolutePx(FiniteScalarErrorOf<S>),
    InvalidPercentFraction(FiniteScalarErrorOf<S>),
}

impl<S: LayoutScalar> core::fmt::Display for LengthPercentageErrorOf<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidAbsolutePx(_) => f.write_str("absolute length coefficient must be finite"),
            Self::InvalidPercentFraction(_) => f.write_str("percentage coefficient must be finite"),
        }
    }
}

impl<S: LayoutScalar> std::error::Error for LengthPercentageErrorOf<S> {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LengthPercentageOf<S: LayoutScalar = DefaultScalar> {
    absolute_px: S,
    percent_fraction: S,
}

impl<S: LayoutScalar> LengthPercentageOf<S> {
    pub const ZERO: Self = Self {
        absolute_px: S::ZERO,
        percent_fraction: S::ZERO,
    };

    pub fn px(value: S) -> Result<Self, FiniteScalarErrorOf<S>> {
        let absolute_px = finite_scalar(value)?;
        Ok(Self {
            absolute_px,
            percent_fraction: S::ZERO,
        })
    }

    pub fn from_percent_fraction(value: S) -> Result<Self, FiniteScalarErrorOf<S>> {
        let percent_fraction = finite_scalar(value)?;
        Ok(Self {
            absolute_px: S::ZERO,
            percent_fraction,
        })
    }

    pub fn from_coefficients(
        absolute_px: S,
        percent_fraction: S,
    ) -> Result<Self, LengthPercentageErrorOf<S>> {
        Ok(Self {
            absolute_px: finite_scalar(absolute_px)
                .map_err(LengthPercentageErrorOf::InvalidAbsolutePx)?,
            percent_fraction: finite_scalar(percent_fraction)
                .map_err(LengthPercentageErrorOf::InvalidPercentFraction)?,
        })
    }

    #[must_use]
    pub const fn absolute_px(self) -> S {
        self.absolute_px
    }

    #[must_use]
    pub const fn percent_fraction(self) -> S {
        self.percent_fraction
    }

    #[must_use]
    pub fn depends_on_basis(self) -> bool {
        self.percent_fraction != S::ZERO
    }

    #[must_use]
    pub fn resolve_against(self, basis: PercentageBasisOf<S>) -> NumericResolutionOf<S> {
        if !self.depends_on_basis() {
            return NumericResolutionOf::Resolved(self.absolute_px);
        }

        let PercentageBasisOf::Definite(basis) = basis else {
            return NumericResolutionOf::MissingBasis { value: self };
        };

        let resolved = self.absolute_px + self.percent_fraction * basis.get();
        if resolved.is_finite() {
            NumericResolutionOf::Resolved(canonical_zero(resolved))
        } else {
            NumericResolutionOf::InvalidNumeric {
                value: self,
                basis: PercentageBasisOf::Definite(basis),
                resolved,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PercentageBasisOf<S: LayoutScalar = DefaultScalar> {
    Missing,
    Definite(NonNegativeFiniteOf<S>),
}

impl<S: LayoutScalar> PercentageBasisOf<S> {
    pub const MISSING: Self = Self::Missing;

    pub fn definite(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        Ok(Self::Definite(NonNegativeFiniteOf::new(value)?))
    }

    #[must_use]
    pub const fn definite_value(self) -> Option<NonNegativeFiniteOf<S>> {
        match self {
            Self::Missing => None,
            Self::Definite(value) => Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericResolutionOf<S: LayoutScalar = DefaultScalar> {
    Resolved(S),
    MissingBasis {
        value: LengthPercentageOf<S>,
    },
    InvalidNumeric {
        value: LengthPercentageOf<S>,
        basis: PercentageBasisOf<S>,
        resolved: S,
    },
}

fn finite_scalar<S: LayoutScalar>(value: S) -> Result<S, FiniteScalarErrorOf<S>> {
    if value.is_finite() {
        Ok(canonical_zero(value))
    } else {
        Err(FiniteScalarErrorOf::NonFinite { value })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthResolutionStatus<S: LayoutScalar = DefaultScalar> {
    Resolved,
    MissingBasis,
    InvalidNumeric { value: S },
    NonNumeric,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnresolvedLengthReason {
    Basis,
    InvalidNumeric,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedLengthAutoOf<S: LayoutScalar = DefaultScalar> {
    Auto,
    Resolved(S),
    Unresolved(UnresolvedLengthReason),
}

pub type ResolvedLengthAuto = ResolvedLengthAutoOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LengthResolutionOf<S: LayoutScalar = DefaultScalar> {
    pub value: Option<S>,
    pub depends_on_basis: bool,
    status: LengthResolutionStatus<S>,
}

pub type LengthResolution = LengthResolutionOf<DefaultScalar>;

impl<S: LayoutScalar> LengthResolutionOf<S> {
    #[must_use]
    pub const fn definite(value: S, depends_on_basis: bool) -> Self {
        Self {
            value: Some(value),
            depends_on_basis,
            status: LengthResolutionStatus::Resolved,
        }
    }

    #[must_use]
    pub const fn unresolved(depends_on_basis: bool) -> Self {
        Self {
            value: None,
            depends_on_basis,
            status: LengthResolutionStatus::MissingBasis,
        }
    }

    #[must_use]
    pub const fn invalid_numeric(value: S, depends_on_basis: bool) -> Self {
        Self {
            value: None,
            depends_on_basis,
            status: LengthResolutionStatus::InvalidNumeric { value },
        }
    }

    #[must_use]
    pub const fn non_numeric() -> Self {
        Self {
            value: None,
            depends_on_basis: false,
            status: LengthResolutionStatus::NonNumeric,
        }
    }

    #[must_use]
    pub const fn status(self) -> LengthResolutionStatus<S> {
        self.status
    }

    #[must_use]
    pub const fn unresolved_reason(self) -> Option<UnresolvedLengthReason> {
        match self.status {
            LengthResolutionStatus::Resolved | LengthResolutionStatus::NonNumeric => None,
            LengthResolutionStatus::MissingBasis => Some(UnresolvedLengthReason::Basis),
            LengthResolutionStatus::InvalidNumeric { .. } => {
                Some(UnresolvedLengthReason::InvalidNumeric)
            }
        }
    }
}

fn optional_basis<S: LayoutScalar>(
    basis: Option<S>,
) -> Result<PercentageBasisOf<S>, NonNegativeFiniteScalarErrorOf<S>> {
    match basis {
        Some(basis) => PercentageBasisOf::definite(basis),
        None => Ok(PercentageBasisOf::Missing),
    }
}

fn resolution_optional<S: LayoutScalar>(resolution: NumericResolutionOf<S>) -> Option<S> {
    match resolution {
        NumericResolutionOf::Resolved(value) => Some(value),
        NumericResolutionOf::MissingBasis { .. } | NumericResolutionOf::InvalidNumeric { .. } => {
            None
        }
    }
}

fn length_resolution_against<S: LayoutScalar>(
    length_percentage: LengthPercentageOf<S>,
    basis: PercentageBasisOf<S>,
) -> LengthResolutionOf<S> {
    match length_percentage.resolve_against(basis) {
        NumericResolutionOf::Resolved(value) => {
            LengthResolutionOf::definite(value, length_percentage.depends_on_basis())
        }
        NumericResolutionOf::MissingBasis { .. } => LengthResolutionOf::unresolved(true),
        NumericResolutionOf::InvalidNumeric {
            value, resolved, ..
        } => LengthResolutionOf::invalid_numeric(resolved, value.depends_on_basis()),
    }
}

fn invalid_basis_value<S: LayoutScalar>(error: NonNegativeFiniteScalarErrorOf<S>) -> S {
    match error {
        NonNegativeFiniteScalarErrorOf::NonFinite { value }
        | NonNegativeFiniteScalarErrorOf::Negative { value } => value,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthOf<S: LayoutScalar = DefaultScalar> {
    Normal,
    Value(LengthPercentageOf<S>),
}

pub type Length = LengthOf<DefaultScalar>;

impl<S: LayoutScalar> LengthOf<S> {
    pub const NORMAL: Self = Self::Normal;
    pub const ZERO: Self = Self::Value(LengthPercentageOf::ZERO);

    #[must_use]
    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::Value(LengthPercentageOf::px(value).expect("trusted crate length px literal"))
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::Value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted crate length percent literal"),
        )
    }

    #[must_use]
    pub const fn value(value: LengthPercentageOf<S>) -> Self {
        Self::Value(value)
    }

    #[must_use]
    pub fn depends_on_basis(self) -> bool {
        match self {
            Self::Value(value) => value.depends_on_basis(),
            Self::Normal => false,
        }
    }

    #[must_use]
    pub fn percent_fraction(self) -> S {
        match self {
            Self::Value(value) => value.percent_fraction(),
            Self::Normal => S::ZERO,
        }
    }

    #[must_use]
    pub fn resolve_against(self, basis: PercentageBasisOf<S>) -> LengthResolutionOf<S> {
        self.resolve_with_status_against(basis)
    }

    #[must_use]
    pub fn resolve(self, basis: S) -> LengthResolutionOf<S> {
        match PercentageBasisOf::definite(basis) {
            Ok(basis) => self.resolve_against(basis),
            Err(error) => LengthResolutionOf::invalid_numeric(
                invalid_basis_value(error),
                self.depends_on_basis(),
            ),
        }
    }

    #[must_use]
    pub fn resolve_optional_against(self, basis: PercentageBasisOf<S>) -> Option<S> {
        match self {
            Self::Normal => Some(S::ZERO),
            Self::Value(value) => resolution_optional(value.resolve_against(basis)),
        }
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<S>) -> Option<S> {
        match optional_basis(basis) {
            Ok(basis) => self.resolve_optional_against(basis),
            Err(_) => None,
        }
    }

    #[must_use]
    pub fn resolve_with_status_against(self, basis: PercentageBasisOf<S>) -> LengthResolutionOf<S> {
        match self {
            Self::Normal => LengthResolutionOf::definite(S::ZERO, false),
            Self::Value(value) => length_resolution_against(value, basis),
        }
    }

    #[must_use]
    pub fn resolve_with_status(self, basis: Option<S>) -> LengthResolutionOf<S> {
        match optional_basis(basis) {
            Ok(basis) => self.resolve_with_status_against(basis),
            Err(error) => LengthResolutionOf::invalid_numeric(
                invalid_basis_value(error),
                self.depends_on_basis(),
            ),
        }
    }
}

impl<S: LayoutScalar> Default for LengthOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthAutoOf<S: LayoutScalar = DefaultScalar> {
    Value(LengthPercentageOf<S>),
    Auto,
}

pub type LengthAuto = LengthAutoOf<DefaultScalar>;

impl<S: LayoutScalar> LengthAutoOf<S> {
    pub const ZERO: Self = Self::Value(LengthPercentageOf::ZERO);
    pub const AUTO: Self = Self::Auto;

    #[must_use]
    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::Value(LengthPercentageOf::px(value).expect("trusted crate length-auto px literal"))
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::Value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted crate length-auto percent literal"),
        )
    }

    #[must_use]
    pub const fn value(value: LengthPercentageOf<S>) -> Self {
        Self::Value(value)
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    #[must_use]
    pub fn depends_on_basis(self) -> bool {
        match self {
            Self::Value(value) => value.depends_on_basis(),
            Self::Auto => false,
        }
    }

    #[must_use]
    pub fn resolve_against(self, basis: PercentageBasisOf<S>) -> LengthResolutionOf<S> {
        self.resolve_with_status_against(basis)
    }

    #[must_use]
    pub fn resolve(self, basis: S) -> LengthResolutionOf<S> {
        match PercentageBasisOf::definite(basis) {
            Ok(basis) => self.resolve_against(basis),
            Err(error) => LengthResolutionOf::invalid_numeric(
                invalid_basis_value(error),
                self.depends_on_basis(),
            ),
        }
    }

    #[must_use]
    pub fn resolve_optional_against(self, basis: PercentageBasisOf<S>) -> Option<S> {
        match self {
            Self::Value(value) => resolution_optional(value.resolve_against(basis)),
            Self::Auto => None,
        }
    }

    #[must_use]
    pub fn resolve_optional(self, basis: Option<S>) -> Option<S> {
        match optional_basis(basis) {
            Ok(basis) => self.resolve_optional_against(basis),
            Err(_) => None,
        }
    }

    #[must_use]
    pub fn resolve_with_status_against(self, basis: PercentageBasisOf<S>) -> LengthResolutionOf<S> {
        match self {
            Self::Value(value) => length_resolution_against(value, basis),
            Self::Auto => LengthResolutionOf::non_numeric(),
        }
    }

    #[must_use]
    pub fn resolve_with_status(self, basis: Option<S>) -> LengthResolutionOf<S> {
        match optional_basis(basis) {
            Ok(basis) => self.resolve_with_status_against(basis),
            Err(error) => LengthResolutionOf::invalid_numeric(
                invalid_basis_value(error),
                self.depends_on_basis(),
            ),
        }
    }

    #[must_use]
    pub fn resolve_auto_with_status_against(
        self,
        basis: PercentageBasisOf<S>,
    ) -> ResolvedLengthAutoOf<S> {
        if self.is_auto() {
            return ResolvedLengthAutoOf::Auto;
        }

        let resolution = self.resolve_with_status_against(basis);
        if let Some(value) = resolution.value {
            return ResolvedLengthAutoOf::Resolved(value);
        }
        if let Some(reason) = resolution.unresolved_reason() {
            return ResolvedLengthAutoOf::Unresolved(reason);
        }

        panic!("non-auto length resolution produced non-numeric status")
    }

    #[must_use]
    pub fn resolve_auto_with_status(self, basis: Option<S>) -> ResolvedLengthAutoOf<S> {
        match optional_basis(basis) {
            Ok(basis) => self.resolve_auto_with_status_against(basis),
            Err(_) => ResolvedLengthAutoOf::Unresolved(UnresolvedLengthReason::InvalidNumeric),
        }
    }

    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl<S: LayoutScalar> Default for LengthAutoOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<S: LayoutScalar> From<LengthOf<S>> for LengthAutoOf<S> {
    fn from(value: LengthOf<S>) -> Self {
        match value {
            LengthOf::Normal => Self::ZERO,
            LengthOf::Value(value) => Self::Value(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackFlexFactorOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type TrackFlexFactor = TrackFlexFactorOf<DefaultScalar>;

impl<S: LayoutScalar> TrackFlexFactorOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn try_new(value: S) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        if !value.is_finite() {
            return Err(NonNegativeFiniteScalarErrorOf::NonFinite { value });
        }
        if value < S::ZERO {
            return Err(NonNegativeFiniteScalarErrorOf::Negative { value });
        }
        Ok(Self {
            value: canonical_zero(value),
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }
}

impl<S: LayoutScalar> Default for TrackFlexFactorOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

/// A minimum track breadth.
///
/// ```compile_fail
/// use surgeist_layout::{MinTrackSizing, TrackFlexFactor};
/// let factor = TrackFlexFactor::try_new(1.0).unwrap();
/// let _ = MinTrackSizing::Flex(factor);
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{Length, MinTrackSizing};
/// let _: MinTrackSizing = Length::Normal.into();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum MinTrackSizingOf<S: LayoutScalar = DefaultScalar> {
    Calculation(crate::SizingCalculationOf<S>),
    Auto,
    MinContent,
    MaxContent,
}

pub type MinTrackSizing = MinTrackSizingOf<DefaultScalar>;

impl<S: LayoutScalar> MinTrackSizingOf<S> {
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    #[must_use]
    pub(crate) fn px(value: S) -> Self {
        Self::Calculation(crate::SizingCalculationOf::value(
            LengthPercentageOf::px(value).expect("trusted crate track px literal"),
        ))
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::Calculation(crate::SizingCalculationOf::value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted crate track percent literal"),
        ))
    }

    #[must_use]
    pub const fn is_intrinsic(&self) -> bool {
        matches!(self, Self::Auto | Self::MinContent | Self::MaxContent)
    }

    #[must_use]
    pub fn depends_on_basis(&self) -> bool {
        match self {
            Self::Calculation(calculation) => calculation.depends_on_basis(),
            Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub(crate) fn definite(&self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Calculation(calculation) => track_calculation_definite(calculation, basis),
            Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }
}

impl<S: LayoutScalar> From<LengthPercentageOf<S>> for MinTrackSizingOf<S> {
    fn from(value: LengthPercentageOf<S>) -> Self {
        Self::Calculation(crate::SizingCalculationOf::value(value))
    }
}

/// A maximum track breadth.
///
/// ```compile_fail
/// use surgeist_layout::MaxTrackSizing;
/// let _ = MaxTrackSizing::fr(1.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum MaxTrackSizingOf<S: LayoutScalar = DefaultScalar> {
    Calculation(crate::SizingCalculationOf<S>),
    Flex(TrackFlexFactorOf<S>),
    Auto,
    MinContent,
    MaxContent,
    FitContent(crate::SizingCalculationOf<S>),
}

pub type MaxTrackSizing = MaxTrackSizingOf<DefaultScalar>;

impl<S: LayoutScalar> MaxTrackSizingOf<S> {
    pub const AUTO: Self = Self::Auto;
    pub const MIN_CONTENT: Self = Self::MinContent;
    pub const MAX_CONTENT: Self = Self::MaxContent;

    #[must_use]
    pub(crate) fn px(value: S) -> Self {
        Self::Calculation(crate::SizingCalculationOf::value(
            LengthPercentageOf::px(value).expect("trusted crate track px literal"),
        ))
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::Calculation(crate::SizingCalculationOf::value(
            LengthPercentageOf::from_percent_fraction(value)
                .expect("trusted crate track percent literal"),
        ))
    }

    #[must_use]
    pub const fn flex(value: TrackFlexFactorOf<S>) -> Self {
        Self::Flex(value)
    }

    #[must_use]
    pub const fn fit_content(limit: crate::SizingCalculationOf<S>) -> Self {
        Self::FitContent(limit)
    }

    #[must_use]
    pub const fn is_flexible(&self) -> bool {
        matches!(self, Self::Flex(_))
    }

    #[must_use]
    pub const fn is_intrinsic(&self) -> bool {
        matches!(
            self,
            Self::Auto | Self::MinContent | Self::MaxContent | Self::FitContent(_)
        )
    }

    #[must_use]
    pub fn depends_on_basis(&self) -> bool {
        match self {
            Self::Calculation(calculation) | Self::FitContent(calculation) => {
                calculation.depends_on_basis()
            }
            Self::Flex(_) | Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }

    #[must_use]
    pub(crate) fn definite(&self, basis: Option<S>) -> Option<S> {
        match self {
            Self::Calculation(calculation) => track_calculation_definite(calculation, basis),
            Self::Flex(_)
            | Self::Auto
            | Self::MinContent
            | Self::MaxContent
            | Self::FitContent(_) => None,
        }
    }
}

impl<S: LayoutScalar> From<LengthPercentageOf<S>> for MaxTrackSizingOf<S> {
    fn from(value: LengthPercentageOf<S>) -> Self {
        Self::Calculation(crate::SizingCalculationOf::value(value))
    }
}

/// A complete track sizing pair.
///
/// ```compile_fail
/// use surgeist_layout::TrackSizing;
/// let _ = TrackSizing::fr(1.0);
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{PreferredSize, TrackSizing};
/// let _: TrackSizing = PreferredSize::AUTO.into();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TrackSizingOf<S: LayoutScalar = DefaultScalar> {
    pub min: MinTrackSizingOf<S>,
    pub max: MaxTrackSizingOf<S>,
}

pub type TrackSizing = TrackSizingOf<DefaultScalar>;

impl<S: LayoutScalar> TrackSizingOf<S> {
    pub const AUTO: Self = Self {
        min: MinTrackSizingOf::AUTO,
        max: MaxTrackSizingOf::AUTO,
    };
    pub const MIN_CONTENT: Self = Self {
        min: MinTrackSizingOf::MIN_CONTENT,
        max: MaxTrackSizingOf::MIN_CONTENT,
    };
    pub const MAX_CONTENT: Self = Self {
        min: MinTrackSizingOf::MAX_CONTENT,
        max: MaxTrackSizingOf::MAX_CONTENT,
    };

    #[must_use]
    pub const fn new(min: MinTrackSizingOf<S>, max: MaxTrackSizingOf<S>) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub(crate) fn px(value: S) -> Self {
        Self::new(MinTrackSizingOf::px(value), MaxTrackSizingOf::px(value))
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::new(
            MinTrackSizingOf::percent(value),
            MaxTrackSizingOf::percent(value),
        )
    }

    #[must_use]
    pub fn calculation(calculation: crate::SizingCalculationOf<S>) -> Self {
        Self::new(
            MinTrackSizingOf::Calculation(calculation.clone()),
            MaxTrackSizingOf::Calculation(calculation),
        )
    }

    #[must_use]
    pub const fn flex(value: TrackFlexFactorOf<S>) -> Self {
        Self::new(MinTrackSizingOf::AUTO, MaxTrackSizingOf::flex(value))
    }

    #[must_use]
    pub const fn fit_content(limit: crate::SizingCalculationOf<S>) -> Self {
        Self::new(MinTrackSizingOf::AUTO, MaxTrackSizingOf::fit_content(limit))
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizingOf<S>, max: MaxTrackSizingOf<S>) -> Self {
        Self::new(min, max)
    }

    pub(crate) fn is_fixed_without_basis(&self) -> bool {
        self.min
            .definite(None)
            .zip(self.max.definite(None))
            .is_some_and(|(minimum, maximum)| minimum == maximum && minimum.is_finite())
    }

    #[must_use]
    pub fn depends_on_basis(&self) -> bool {
        self.min.depends_on_basis() || self.max.depends_on_basis()
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent_fraction(&self) -> S {
        fn affine_fraction<S: LayoutScalar>(calculation: &crate::SizingCalculationOf<S>) -> S {
            calculation
                .affine_value()
                .map_or(S::ZERO, |value| value.percent_fraction())
        }

        let min = match &self.min {
            MinTrackSizingOf::Calculation(calculation) => affine_fraction(calculation),
            MinTrackSizingOf::Auto
            | MinTrackSizingOf::MinContent
            | MinTrackSizingOf::MaxContent => S::ZERO,
        };
        let max = match &self.max {
            MaxTrackSizingOf::Calculation(calculation)
            | MaxTrackSizingOf::FitContent(calculation) => affine_fraction(calculation),
            MaxTrackSizingOf::Flex(_)
            | MaxTrackSizingOf::Auto
            | MaxTrackSizingOf::MinContent
            | MaxTrackSizingOf::MaxContent => S::ZERO,
        };
        min.max(max)
    }
}

fn track_calculation_definite<S: LayoutScalar>(
    calculation: &crate::SizingCalculationOf<S>,
    basis: Option<S>,
) -> Option<S> {
    let basis = match basis {
        Some(value) => PercentageBasisOf::definite(value).ok()?,
        None => PercentageBasisOf::MISSING,
    };
    let resolution = calculation.resolve_against(basis);
    (resolution.status() == LengthResolutionStatus::Resolved)
        .then_some(resolution.value?.max(S::ZERO))
}

impl<S: LayoutScalar> Default for TrackSizingOf<S> {
    fn default() -> Self {
        Self::AUTO
    }
}

impl<S: LayoutScalar> From<LengthPercentageOf<S>> for TrackSizingOf<S> {
    fn from(value: LengthPercentageOf<S>) -> Self {
        Self::calculation(crate::SizingCalculationOf::value(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackRepeat {
    Count(TrackRepeatCount),
    AutoFill,
    AutoFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackRepeatCount(NonZeroUsize);

impl TrackRepeatCount {
    #[must_use]
    pub const fn new(value: usize) -> Option<Self> {
        match NonZeroUsize::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackComponentListOf<S: LayoutScalar = DefaultScalar>(Vec<TrackComponentOf<S>>);

pub type TrackComponentList = TrackComponentListOf<DefaultScalar>;

impl<S: LayoutScalar> TrackComponentListOf<S> {
    #[must_use]
    pub fn as_slice(&self) -> &[TrackComponentOf<S>] {
        &self.0
    }

    fn into_vec(self) -> Vec<TrackComponentOf<S>> {
        self.0
    }
}

impl<S: LayoutScalar> TryFrom<Vec<TrackComponentOf<S>>> for TrackComponentListOf<S> {
    type Error = TrackRepetitionError;

    fn try_from(value: Vec<TrackComponentOf<S>>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(TrackRepetitionError::EmptyComponents)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackRepetitionError {
    ZeroCount,
    EmptyComponents,
}

impl core::fmt::Display for TrackRepetitionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroCount => f.write_str("track repeat count must be greater than zero"),
            Self::EmptyComponents => f.write_str("track repeat components must not be empty"),
        }
    }
}

impl std::error::Error for TrackRepetitionError {}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackRepetitionOf<S: LayoutScalar = DefaultScalar> {
    repeat: TrackRepeat,
    components: TrackComponentListOf<S>,
}

pub type TrackRepetition = TrackRepetitionOf<DefaultScalar>;

impl<S: LayoutScalar> TrackRepetitionOf<S> {
    pub fn count(
        count: usize,
        tracks: Vec<TrackSizingOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        Self::count_components(count, track_sizing_components_from_tracks(tracks))
    }

    pub fn auto_fill(tracks: Vec<TrackSizingOf<S>>) -> Result<Self, TrackRepetitionError> {
        Self::auto_fill_components(track_sizing_components_from_tracks(tracks))
    }

    pub fn auto_fit(tracks: Vec<TrackSizingOf<S>>) -> Result<Self, TrackRepetitionError> {
        Self::auto_fit_components(track_sizing_components_from_tracks(tracks))
    }

    pub fn count_components(
        count: usize,
        components: Vec<TrackComponentOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        let count = TrackRepeatCount::new(count).ok_or(TrackRepetitionError::ZeroCount)?;
        let components = TrackComponentListOf::try_from(components)?;
        Ok(Self::from_validated(TrackRepeat::Count(count), components))
    }

    pub fn auto_fill_components(
        components: Vec<TrackComponentOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        let components = TrackComponentListOf::try_from(components)?;
        Ok(Self::from_validated(TrackRepeat::AutoFill, components))
    }

    pub fn auto_fit_components(
        components: Vec<TrackComponentOf<S>>,
    ) -> Result<Self, TrackRepetitionError> {
        let components = TrackComponentListOf::try_from(components)?;
        Ok(Self::from_validated(TrackRepeat::AutoFit, components))
    }

    #[must_use]
    pub const fn from_validated(repeat: TrackRepeat, components: TrackComponentListOf<S>) -> Self {
        Self { components, repeat }
    }

    #[must_use]
    pub fn sizing_tracks(&self) -> Vec<TrackSizingOf<S>> {
        track_sizing_components_of(self.components.as_slice())
    }

    #[must_use]
    pub const fn repeat(&self) -> TrackRepeat {
        self.repeat
    }

    #[must_use]
    pub fn components(&self) -> &[TrackComponentOf<S>] {
        self.components.as_slice()
    }

    #[must_use]
    pub fn into_components(self) -> Vec<TrackComponentOf<S>> {
        self.components.into_vec()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TrackComponentOf<S: LayoutScalar = DefaultScalar> {
    LineNames(Vec<String>),
    Track(TrackSizingOf<S>),
    Repeat(TrackRepetitionOf<S>),
    Subgrid(SubgridTrack),
}

pub type TrackComponent = TrackComponentOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrack {
    pub name_components: Vec<SubgridLineNameComponent>,
}

impl SubgridTrack {
    #[must_use]
    pub fn new(line_names: Vec<Vec<String>>) -> Self {
        Self {
            name_components: line_names
                .into_iter()
                .map(SubgridLineNameComponent::LineNames)
                .collect(),
        }
    }

    #[must_use]
    pub fn line_names(&self) -> Vec<Vec<String>> {
        let mut line_names = Vec::new();
        for component in &self.name_components {
            match component {
                SubgridLineNameComponent::LineNames(names) => line_names.push(names.clone()),
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::Count(count),
                    line_name_sets,
                } => {
                    for _ in 0..*count {
                        line_names.extend(line_name_sets.iter().cloned());
                    }
                }
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::AutoFill,
                    line_name_sets,
                } => line_names.extend(line_name_sets.iter().cloned()),
            }
        }
        line_names
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubgridLineNameComponent {
    LineNames(Vec<String>),
    Repeat {
        count: SubgridLineNameRepeatCount,
        line_name_sets: Vec<Vec<String>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgridLineNameRepeatCount {
    Count(usize),
    AutoFill,
}

impl<S: LayoutScalar> TrackComponentOf<S> {
    pub const AUTO: Self = Self::Track(TrackSizingOf::AUTO);
    pub const MIN_CONTENT: Self = Self::Track(TrackSizingOf::MIN_CONTENT);
    pub const MAX_CONTENT: Self = Self::Track(TrackSizingOf::MAX_CONTENT);

    #[must_use]
    #[cfg(test)]
    pub(crate) fn px(value: S) -> Self {
        Self::Track(TrackSizingOf::px(value))
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn percent(value: S) -> Self {
        Self::Track(TrackSizingOf::percent(value))
    }

    #[must_use]
    pub const fn flex(value: TrackFlexFactorOf<S>) -> Self {
        Self::Track(TrackSizingOf::flex(value))
    }

    #[must_use]
    pub const fn fit_content(limit: crate::SizingCalculationOf<S>) -> Self {
        Self::Track(TrackSizingOf::fit_content(limit))
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizingOf<S>, max: MaxTrackSizingOf<S>) -> Self {
        Self::Track(TrackSizingOf::minmax(min, max))
    }

    #[must_use]
    pub fn line_names(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::LineNames(names.into_iter().map(Into::into).collect())
    }
}

impl<S: LayoutScalar> From<TrackSizingOf<S>> for TrackComponentOf<S> {
    fn from(value: TrackSizingOf<S>) -> Self {
        Self::Track(value)
    }
}

impl<S: LayoutScalar> From<LengthPercentageOf<S>> for TrackComponentOf<S> {
    fn from(value: LengthPercentageOf<S>) -> Self {
        Self::Track(value.into())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GridTemplateAreas {
    pub rows: Vec<GridTemplateAreaRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridTemplateAreaRow {
    pub cells: Vec<Option<String>>,
}

#[must_use]
pub fn track_sizing_components(components: &[TrackComponent]) -> Vec<TrackSizing> {
    track_sizing_components_of(components)
}

#[must_use]
pub fn track_sizing_components_of<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Vec<TrackSizingOf<S>> {
    let mut tracks = Vec::new();
    for component in components {
        match component {
            TrackComponentOf::Track(track) => tracks.push(track.clone()),
            TrackComponentOf::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                if repeated_tracks.is_empty() {
                    continue;
                }
                match repetition.repeat() {
                    TrackRepeat::Count(count) => {
                        for _ in 0..count.get() {
                            tracks.extend(repeated_tracks.iter().cloned());
                        }
                    }
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                        tracks.extend(repeated_tracks);
                    }
                }
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    tracks
}

fn track_sizing_components_from_tracks<S: LayoutScalar>(
    tracks: Vec<TrackSizingOf<S>>,
) -> Vec<TrackComponentOf<S>> {
    tracks.into_iter().map(TrackComponentOf::Track).collect()
}

#[cfg(test)]
mod value_tests {
    use super::{
        LayoutScalar, LengthAutoOf, LengthOf, LengthPercentageOf, LengthResolutionStatus,
        MaxTrackSizingOf, MinTrackSizingOf, NumericResolutionOf, PercentageBasisOf,
        TrackFlexFactorOf, TrackSizingOf, canonical_zero,
    };
    use crate::SizingCalculationOf;

    fn assert_fri06_mr02_signed_zero_scalar_and_value_boundaries<S: LayoutScalar>() {
        for value in [S::ZERO, -S::ZERO] {
            let canonical = canonical_zero(value);
            assert_eq!(canonical, S::ZERO);
            assert!(!canonical.to_f64().is_sign_negative());
        }

        for value in [S::from_f64(13.5), S::from_f64(-7.25)] {
            assert_eq!(canonical_zero(value), value);
        }
        assert_eq!(canonical_zero(S::INFINITY), S::INFINITY);
        assert_eq!(canonical_zero(-S::INFINITY), -S::INFINITY);
        assert!(canonical_zero(S::NAN).to_f64().is_nan());

        let value = LengthPercentageOf::from_coefficients(-S::ZERO, S::from_f64(-0.25))
            .expect("finite coefficients remain accepted");
        assert_eq!(value.absolute_px(), S::ZERO);
        assert!(!value.absolute_px().to_f64().is_sign_negative());
        assert_eq!(value.percent_fraction(), S::from_f64(-0.25));

        for non_finite in [S::INFINITY, -S::INFINITY, S::NAN] {
            assert!(LengthPercentageOf::px(non_finite).is_err());
            assert!(LengthPercentageOf::from_percent_fraction(non_finite).is_err());
        }
    }

    #[test]
    fn fri06_mr02_signed_zero_scalar_primitive_and_value_boundaries_are_preserved() {
        assert_fri06_mr02_signed_zero_scalar_and_value_boundaries::<f32>();
        assert_fri06_mr02_signed_zero_scalar_and_value_boundaries::<f64>();
    }

    #[test]
    fn track_sizing_flex_factor_validates_both_scalar_lanes() {
        let f32_factor = TrackFlexFactorOf::<f32>::try_new(1.25).expect("valid f32 factor");
        let f64_factor = TrackFlexFactorOf::<f64>::try_new(2.5).expect("valid f64 factor");
        assert_eq!(f32_factor.get(), 1.25);
        assert_eq!(f64_factor.get(), 2.5);
        assert_eq!(TrackFlexFactorOf::<f32>::default(), TrackFlexFactorOf::ZERO);
        assert_eq!(TrackFlexFactorOf::<f64>::default(), TrackFlexFactorOf::ZERO);

        assert!(matches!(
            TrackFlexFactorOf::<f32>::try_new(-1.0),
            Err(super::NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 })
        ));
        assert!(matches!(
            TrackFlexFactorOf::<f64>::try_new(-1.0),
            Err(super::NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 })
        ));
        assert!(matches!(
            TrackFlexFactorOf::<f32>::try_new(f32::INFINITY),
            Err(super::NonNegativeFiniteScalarErrorOf::NonFinite { value }) if value.is_infinite()
        ));
        assert!(matches!(
            TrackFlexFactorOf::<f32>::try_new(f32::NAN),
            Err(super::NonNegativeFiniteScalarErrorOf::NonFinite { value }) if value.is_nan()
        ));
        assert!(matches!(
            TrackFlexFactorOf::<f64>::try_new(f64::NEG_INFINITY),
            Err(super::NonNegativeFiniteScalarErrorOf::NonFinite { value }) if value.is_infinite()
        ));
        assert!(matches!(
            TrackFlexFactorOf::<f64>::try_new(f64::NAN),
            Err(super::NonNegativeFiniteScalarErrorOf::NonFinite { value }) if value.is_nan()
        ));

        assert_eq!(
            TrackFlexFactorOf::<f32>::try_new(-0.0)
                .expect("signed zero is valid")
                .get()
                .to_bits(),
            0.0f32.to_bits()
        );
        assert_eq!(
            TrackFlexFactorOf::<f64>::try_new(-0.0)
                .expect("signed zero is valid")
                .get()
                .to_bits(),
            0.0f64.to_bits()
        );
    }

    #[test]
    fn track_sizing_construction_matrix_uses_calculations_and_validated_flex() {
        let calculation = SizingCalculationOf::<f64>::value(
            LengthPercentageOf::from_coefficients(12.0, 0.25).expect("finite affine value"),
        );
        let factor = TrackFlexFactorOf::<f64>::try_new(1.5).expect("valid factor");

        let fixed = TrackSizingOf::calculation(calculation.clone());
        assert_eq!(
            fixed,
            TrackSizingOf::new(
                MinTrackSizingOf::Calculation(calculation.clone()),
                MaxTrackSizingOf::Calculation(calculation.clone()),
            )
        );
        assert_eq!(
            TrackSizingOf::flex(factor),
            TrackSizingOf::minmax(MinTrackSizingOf::Auto, MaxTrackSizingOf::Flex(factor))
        );
        assert_eq!(
            TrackSizingOf::fit_content(calculation.clone()),
            TrackSizingOf::minmax(
                MinTrackSizingOf::Auto,
                MaxTrackSizingOf::FitContent(calculation.clone()),
            )
        );

        assert_eq!(TrackSizingOf::<f64>::default(), TrackSizingOf::AUTO);
        let f32_calculation = SizingCalculationOf::<f32>::value(
            LengthPercentageOf::from_coefficients(8.0, 0.5).expect("finite affine value"),
        );
        assert_eq!(
            TrackSizingOf::calculation(f32_calculation.clone()),
            TrackSizingOf::new(
                MinTrackSizingOf::Calculation(f32_calculation.clone()),
                MaxTrackSizingOf::Calculation(f32_calculation),
            )
        );
        assert_eq!(
            TrackSizingOf::<f64>::MIN_CONTENT,
            TrackSizingOf::minmax(MinTrackSizingOf::MinContent, MaxTrackSizingOf::MinContent,)
        );
        assert_eq!(
            TrackSizingOf::<f64>::MAX_CONTENT,
            TrackSizingOf::minmax(MinTrackSizingOf::MaxContent, MaxTrackSizingOf::MaxContent,)
        );
        assert_eq!(
            TrackSizingOf::<f64>::calculation(
                SizingCalculationOf::value(LengthPercentageOf::ZERO,)
            ),
            TrackSizingOf::calculation(SizingCalculationOf::value(LengthPercentageOf::ZERO))
        );
    }

    #[test]
    fn value_length_percentage_constructs_f32_px_percent_and_mixed_values() {
        let px = LengthPercentageOf::<f32>::px(12.5).expect("finite px");
        assert_eq!(px.absolute_px(), 12.5);
        assert_eq!(px.percent_fraction(), 0.0);
        assert!(!px.depends_on_basis());

        let percent =
            LengthPercentageOf::<f32>::from_percent_fraction(0.25).expect("finite percent");
        assert_eq!(percent.absolute_px(), 0.0);
        assert_eq!(percent.percent_fraction(), 0.25);
        assert!(percent.depends_on_basis());

        let mixed = LengthPercentageOf::<f32>::from_coefficients(10.0, 0.5).expect("finite mixed");
        let basis = PercentageBasisOf::<f32>::definite(80.0).expect("valid basis");
        assert_eq!(
            mixed.resolve_against(basis),
            NumericResolutionOf::Resolved(50.0)
        );
    }

    #[test]
    fn value_length_percentage_constructs_f64_negative_percent_and_resolves() {
        let value =
            LengthPercentageOf::<f64>::from_coefficients(30.0, -0.25).expect("finite mixed value");
        let basis = PercentageBasisOf::<f64>::definite(40.0).expect("valid basis");

        assert_eq!(value.absolute_px(), 30.0);
        assert_eq!(value.percent_fraction(), -0.25);
        assert!(value.depends_on_basis());
        assert_eq!(
            value.resolve_against(basis),
            NumericResolutionOf::Resolved(20.0)
        );
    }

    #[test]
    fn value_length_percentage_canonicalizes_signed_zero_coefficients() {
        let f32_value =
            LengthPercentageOf::<f32>::from_coefficients(-0.0, -0.0).expect("finite zeros");
        assert_eq!(f32_value.absolute_px().to_bits(), 0.0f32.to_bits());
        assert_eq!(f32_value.percent_fraction().to_bits(), 0.0f32.to_bits());
        assert_eq!(
            f32_value.resolve_against(PercentageBasisOf::Missing),
            NumericResolutionOf::Resolved(0.0)
        );

        let f64_value =
            LengthPercentageOf::<f64>::from_coefficients(-0.0, -0.0).expect("finite zeros");
        assert_eq!(f64_value.absolute_px().to_bits(), 0.0f64.to_bits());
        assert_eq!(f64_value.percent_fraction().to_bits(), 0.0f64.to_bits());
        assert_eq!(
            f64_value.resolve_against(PercentageBasisOf::Missing),
            NumericResolutionOf::Resolved(0.0)
        );
    }

    #[test]
    fn value_length_percentage_rejects_non_finite_coefficients() {
        assert!(LengthPercentageOf::<f32>::px(f32::INFINITY).is_err());
        assert!(LengthPercentageOf::<f32>::from_percent_fraction(f32::NAN).is_err());
        assert!(LengthPercentageOf::<f64>::from_coefficients(f64::NAN, 0.0).is_err());
        assert!(LengthPercentageOf::<f64>::from_coefficients(0.0, f64::INFINITY).is_err());
    }

    #[test]
    fn value_public_length_construction_uses_validated_affine_values() {
        assert!(LengthPercentageOf::<f32>::px(f32::NAN).is_err());
        assert!(LengthPercentageOf::<f32>::from_percent_fraction(f32::INFINITY).is_err());
        assert!(LengthPercentageOf::<f32>::from_coefficients(0.0, f32::NAN).is_err());

        let validated =
            LengthPercentageOf::<f32>::from_coefficients(8.0, 0.25).expect("finite coefficients");
        let length = LengthOf::<f32>::value(validated);

        assert_eq!(length.resolve_optional(Some(80.0)), Some(28.0));
    }

    #[test]
    fn value_public_length_resolution_preserves_affine_failure_status() {
        let basis_dependent = LengthOf::<f32>::value(
            LengthPercentageOf::<f32>::from_coefficients(7.0, 0.5).expect("finite coefficients"),
        );
        let invalid_numeric = LengthOf::<f32>::value(
            LengthPercentageOf::<f32>::from_coefficients(f32::MAX, 1.0)
                .expect("finite coefficients"),
        );

        let missing_basis = basis_dependent.resolve_against(PercentageBasisOf::Missing);
        let overflow = invalid_numeric.resolve(f32::MAX);

        assert_eq!(missing_basis.value, None);
        assert_eq!(missing_basis.status(), LengthResolutionStatus::MissingBasis);
        assert_eq!(overflow.value, None);
        assert_eq!(
            overflow.status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f32::INFINITY,
            }
        );
    }

    #[test]
    fn value_invalid_affine_numeric_result_finite_overflow_is_unresolved_not_nan() {
        let length = LengthOf::<f32>::value(
            LengthPercentageOf::<f32>::from_coefficients(f32::MAX, 1.0)
                .expect("finite coefficients"),
        );

        let resolution = length.resolve_with_status(Some(f32::MAX));

        assert_eq!(resolution.value, None);
        assert_eq!(
            resolution.status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f32::INFINITY,
            }
        );
    }

    #[test]
    fn value_invalid_affine_numeric_result_never_resolves_to_non_finite_value() {
        let length = LengthOf::<f32>::value(
            LengthPercentageOf::<f32>::from_coefficients(f32::MAX, f32::MAX)
                .expect("finite coefficients"),
        );

        assert_eq!(length.resolve_optional(Some(f32::MAX)), None);
        assert_ne!(
            length.resolve_with_status(Some(f32::MAX)).status(),
            LengthResolutionStatus::Resolved
        );
    }

    #[test]
    fn value_percentage_basis_rejects_negative_and_non_finite_values() {
        assert!(PercentageBasisOf::<f32>::definite(-1.0).is_err());
        assert!(PercentageBasisOf::<f32>::definite(f32::INFINITY).is_err());
        assert!(PercentageBasisOf::<f64>::definite(f64::NAN).is_err());
        assert!(PercentageBasisOf::<f64>::definite(-0.0).is_ok());
    }

    #[test]
    fn value_raw_length_resolution_rejects_invalid_bases() {
        let length = LengthOf::<f32>::percent(0.25);

        assert_eq!(
            length.resolve(-1.0).status(),
            LengthResolutionStatus::InvalidNumeric { value: -1.0 }
        );
        assert_eq!(length.resolve_optional(Some(-1.0)), None);
        assert_eq!(
            length.resolve_with_status(Some(-1.0)).status(),
            LengthResolutionStatus::InvalidNumeric { value: -1.0 }
        );
        assert!(matches!(
            length.resolve_with_status(Some(f32::NAN)).status(),
            LengthResolutionStatus::InvalidNumeric { value } if value.is_nan()
        ));
        assert_eq!(
            length.resolve_with_status(Some(f32::INFINITY)).status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f32::INFINITY,
            }
        );
    }

    #[test]
    fn value_raw_optional_resolution_rejects_invalid_bases_before_success() {
        let px_only = LengthOf::<f32>::px(12.0);
        let auto = LengthAutoOf::<f32>::AUTO;

        assert_eq!(px_only.resolve_optional(Some(f32::NAN)), None);
        assert!(matches!(
            px_only.resolve_with_status(Some(f32::NAN)).status(),
            LengthResolutionStatus::InvalidNumeric { value } if value.is_nan()
        ));
        assert_eq!(
            auto.resolve_with_status(Some(f32::INFINITY)).status(),
            LengthResolutionStatus::InvalidNumeric {
                value: f32::INFINITY,
            }
        );
    }

    #[test]
    fn value_length_percentage_missing_basis_only_when_needed() {
        let basis_independent =
            LengthPercentageOf::<f32>::from_coefficients(7.0, 0.0).expect("finite value");
        assert_eq!(
            basis_independent.resolve_against(PercentageBasisOf::Missing),
            NumericResolutionOf::Resolved(7.0)
        );

        let basis_dependent =
            LengthPercentageOf::<f32>::from_coefficients(7.0, 0.5).expect("finite value");
        assert_eq!(
            basis_dependent.resolve_against(PercentageBasisOf::Missing),
            NumericResolutionOf::MissingBasis {
                value: basis_dependent
            }
        );
    }

    #[test]
    fn value_length_percentage_reports_overflow_as_invalid_numeric() {
        let f32_value =
            LengthPercentageOf::<f32>::from_coefficients(f32::MAX, 1.0).expect("finite value");
        let f32_basis = PercentageBasisOf::<f32>::definite(f32::MAX).expect("valid basis");
        assert_eq!(
            f32_value.resolve_against(f32_basis),
            NumericResolutionOf::InvalidNumeric {
                value: f32_value,
                basis: f32_basis,
                resolved: f32::INFINITY,
            }
        );

        let f64_value =
            LengthPercentageOf::<f64>::from_coefficients(f64::MAX, 1.0).expect("finite value");
        let f64_basis = PercentageBasisOf::<f64>::definite(f64::MAX).expect("valid basis");
        assert_eq!(
            f64_value.resolve_against(f64_basis),
            NumericResolutionOf::InvalidNumeric {
                value: f64_value,
                basis: f64_basis,
                resolved: f64::INFINITY,
            }
        );
    }
}

#[cfg(test)]
mod orthogonal_fallback_tests {
    use super::*;
    use crate::{PhysicalAxis, Size};

    fn qualify<S: LayoutScalar>() {
        let nn = |v| NonNegativeFiniteOf::new(S::from_f64(v)).unwrap();
        let finite = |v| OrthogonalFallbackLimitOf::Finite(nn(v));
        let unbounded = Size::splat(OrthogonalFallbackLimitOf::Unbounded);
        let original = OrthogonalFallbackOf::new(
            Size::new(Some(nn(800.0)), Some(nn(900.0))),
            NearestScrollportOf::Present(Size::new(finite(100.0), finite(120.0))),
            unbounded,
        );
        assert_eq!(original.resolve(PhysicalAxis::Horizontal), Ok(nn(100.0)));
        assert_eq!(original.resolve(PhysicalAxis::Vertical), Ok(nn(120.0)));
        let entered = original.for_child(
            unbounded,
            ScrollportFallbackChangeOf::Enter(Size::new(
                OrthogonalFallbackLimitOf::Unbounded,
                finite(200.0),
            )),
        );
        assert_eq!(entered.resolve(PhysicalAxis::Horizontal), Ok(nn(800.0)));
        assert_eq!(entered.resolve(PhysicalAxis::Vertical), Ok(nn(200.0)));
        let opposite = original.for_child(
            unbounded,
            ScrollportFallbackChangeOf::Enter(Size::new(
                finite(200.0),
                OrthogonalFallbackLimitOf::Unbounded,
            )),
        );
        assert_eq!(opposite.resolve(PhysicalAxis::Horizontal), Ok(nn(200.0)));
        assert_eq!(opposite.resolve(PhysicalAxis::Vertical), Ok(nn(900.0)));
        let unknown = original.for_child(
            unbounded,
            ScrollportFallbackChangeOf::Enter(Size::new(
                OrthogonalFallbackLimitOf::Unknown,
                OrthogonalFallbackLimitOf::Unbounded,
            )),
        );
        assert_eq!(
            unknown.resolve(PhysicalAxis::Horizontal),
            Err(OrthogonalFallbackErrorOf::MissingScrollportLimit {
                axis: PhysicalAxis::Horizontal
            })
        );
        assert_eq!(unknown.resolve(PhysicalAxis::Vertical), Ok(nn(900.0)));
        let local = original.for_child(
            Size::new(finite(50.0), finite(60.0)),
            ScrollportFallbackChangeOf::Preserve,
        );
        assert_eq!(local.resolve(PhysicalAxis::Horizontal), Ok(nn(50.0)));
        let child = local.for_child(unbounded, ScrollportFallbackChangeOf::Preserve);
        assert_eq!(child.resolve(PhysicalAxis::Horizontal), Ok(nn(100.0)));
        assert_eq!(
            child.initial_containing_block(),
            original.initial_containing_block()
        );

        assert_eq!(
            OrthogonalFallbackLimitOf::<S>::from_containing_bounds(Some(nn(50.0)), None),
            OrthogonalFallbackLimitOf::Unbounded
        );
        assert_eq!(
            OrthogonalFallbackLimitOf::from_containing_bounds(Some(nn(200.0)), Some(nn(100.0))),
            finite(200.0)
        );
        assert_eq!(
            OrthogonalFallbackLimitOf::from_scrollport(
                Some(nn(300.0)),
                Some(nn(50.0)),
                Some(nn(200.0))
            ),
            finite(200.0)
        );
        assert_eq!(
            OrthogonalFallbackLimitOf::from_scrollport(None, Some(nn(50.0)), Some(nn(200.0))),
            finite(200.0)
        );
        assert_eq!(
            OrthogonalFallbackLimitOf::from_scrollport(
                Some(nn(30.0)),
                Some(nn(50.0)),
                Some(nn(200.0))
            ),
            finite(50.0)
        );
        assert_eq!(
            OrthogonalFallbackLimitOf::<S>::from_scrollport(None, Some(nn(50.0)), None),
            OrthogonalFallbackLimitOf::Unbounded
        );

        let no_scroll = OrthogonalFallbackOf::new(
            original.initial_containing_block(),
            NearestScrollportOf::Absent,
            unbounded,
        );
        let uncapped_scroll =
            no_scroll.for_child(unbounded, ScrollportFallbackChangeOf::Enter(unbounded));
        assert_eq!(
            no_scroll.resolve(PhysicalAxis::Horizontal),
            uncapped_scroll.resolve(PhysicalAxis::Horizontal)
        );
        assert_ne!(no_scroll, uncapped_scroll);
        let missing = OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Unknown,
            Size::new(finite(0.0), OrthogonalFallbackLimitOf::Unknown),
        );
        assert_eq!(missing.resolve(PhysicalAxis::Horizontal), Ok(nn(0.0)));
        assert_eq!(
            missing.resolve(PhysicalAxis::Vertical),
            Err(OrthogonalFallbackErrorOf::MissingInitialContainingBlock {
                axis: PhysicalAxis::Vertical
            })
        );
        let zero_icb = OrthogonalFallbackOf::new(
            Size::new(Some(nn(0.0)), None),
            NearestScrollportOf::Unknown,
            Size::splat(OrthogonalFallbackLimitOf::Unknown),
        );
        assert_eq!(zero_icb.resolve(PhysicalAxis::Horizontal), Ok(nn(0.0)));
        let zero_scroll = OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Present(Size::new(
                finite(0.0),
                OrthogonalFallbackLimitOf::Unknown,
            )),
            Size::splat(OrthogonalFallbackLimitOf::Unknown),
        );
        assert_eq!(zero_scroll.resolve(PhysicalAxis::Horizontal), Ok(nn(0.0)));
        let missing_nearest = OrthogonalFallbackOf::new(
            original.initial_containing_block(),
            NearestScrollportOf::Unknown,
            unbounded,
        );
        assert_eq!(
            missing_nearest.resolve(PhysicalAxis::Horizontal),
            Err(OrthogonalFallbackErrorOf::MissingNearestScrollport {
                axis: PhysicalAxis::Horizontal
            })
        );
        let partial = OrthogonalFallbackOf::new(
            Size::new(Some(nn(800.0)), None),
            NearestScrollportOf::Absent,
            unbounded,
        );
        assert_eq!(partial.resolve(PhysicalAxis::Horizontal), Ok(nn(800.0)));
        assert_eq!(
            partial.resolve(PhysicalAxis::Vertical),
            Err(OrthogonalFallbackErrorOf::MissingInitialContainingBlock {
                axis: PhysicalAxis::Vertical
            })
        );
    }

    #[test]
    fn qualifies_and_replaces_physical_fallbacks_f32() {
        qualify::<f32>();
    }
    #[test]
    fn qualifies_and_replaces_physical_fallbacks_f64() {
        qualify::<f64>();
    }
}
