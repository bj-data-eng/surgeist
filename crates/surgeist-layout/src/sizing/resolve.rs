use super::{
    DispatchedSizingRequest, FlexBasisOf, MaxSizeOf, MinSizeOf, PreferredSizeOf, SizingDispatch,
    dispatch_flex_basis, dispatch_maximum_size, dispatch_minimum_size, dispatch_preferred_size,
};
use crate::error::{sizing_resolution_error, value_resolution_error};
use crate::{
    Compute, Edges, FlowAxes, LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf,
    LengthResolutionOf, LengthResolutionStatus, PercentageBasisOf, PhysicalAxis, Size,
    SizingAlgorithm, Traverse, UnsupportedSizingBehavior,
};

pub(crate) fn select_relative_insets<S: LayoutScalar>(
    inset: Edges<LengthAutoOf<S>>,
    axes: FlowAxes,
) -> Edges<LengthAutoOf<S>> {
    let mut logical = axes.logical_edges(inset);
    if !logical.inline_start.is_auto() {
        logical.inline_end = LengthAutoOf::AUTO;
    }
    if !logical.block_start.is_auto() {
        logical.block_end = LengthAutoOf::AUTO;
    }
    axes.physical_edges(logical)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SizingResolutionError<S: LayoutScalar> {
    Status(LengthResolutionStatus<S>),
    Unsupported(UnsupportedSizingBehavior),
}

impl<S: LayoutScalar> From<LengthResolutionStatus<S>> for SizingResolutionError<S> {
    fn from(status: LengthResolutionStatus<S>) -> Self {
        Self::Status(status)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ResolvedPreferredSize<S: LayoutScalar> {
    Auto,
    Definite(S),
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ResolvedFlexBasis<S: LayoutScalar> {
    Auto,
    Content,
    MinContent,
    MaxContent,
    Definite(S),
}

fn percentage_basis<S: LayoutScalar>(basis: Option<S>) -> PercentageBasisOf<S> {
    basis.map_or(PercentageBasisOf::MISSING, |value| {
        let Ok(basis) = PercentageBasisOf::definite(value) else {
            unreachable!("validated compute inputs carry non-negative finite parent sizes")
        };
        basis
    })
}

fn resolve_dispatched_numeric<S: LayoutScalar>(
    request: DispatchedSizingRequest<'_, S>,
    basis: PercentageBasisOf<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let resolution = match request {
        DispatchedSizingRequest::Zero => return Ok(Some(S::ZERO)),
        DispatchedSizingRequest::Calculation(calculation) => calculation.resolve_against(basis),
        DispatchedSizingRequest::ResolvedCalcSize(resolution) => resolution,
        DispatchedSizingRequest::Auto | DispatchedSizingRequest::None => return Ok(None),
        DispatchedSizingRequest::Content
        | DispatchedSizingRequest::MinContent
        | DispatchedSizingRequest::MaxContent => {
            unreachable!("the property consumer must handle contextual supported states")
        }
    };

    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::MissingBasis if missing_basis_is_indefinite => Ok(None),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(SizingResolutionError::Status(resolution.status()))
        }
        LengthResolutionStatus::NonNumeric => {
            unreachable!("typed sizing dispatch never returns a nonnumeric numeric request")
        }
    }
}

pub(crate) fn resolve_preferred_sizing<S: LayoutScalar>(
    value: &PreferredSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<ResolvedPreferredSize<S>, SizingResolutionError<S>> {
    let basis = percentage_basis(basis);
    match dispatch_preferred_size(value, algorithm, axis, basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(DispatchedSizingRequest::Auto) => Ok(ResolvedPreferredSize::Auto),
        SizingDispatch::Supported(DispatchedSizingRequest::MinContent) => {
            Ok(ResolvedPreferredSize::MinContent)
        }
        SizingDispatch::Supported(DispatchedSizingRequest::MaxContent) => {
            Ok(ResolvedPreferredSize::MaxContent)
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, missing_basis_is_indefinite).map(|value| {
                value.map_or(ResolvedPreferredSize::Auto, ResolvedPreferredSize::Definite)
            })
        }
    }
}

pub(crate) fn resolve_preferred_optional<S: LayoutScalar>(
    value: &PreferredSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    match resolve_preferred_sizing(value, algorithm, axis, basis, missing_basis_is_indefinite)? {
        ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::MinContent
        | ResolvedPreferredSize::MaxContent => Ok(None),
        ResolvedPreferredSize::Definite(value) => Ok(Some(value)),
    }
}

/// Resolve the parent's numeric sizing inputs without consuming an authored
/// intrinsic axis that the child formatter will resolve.
pub(crate) fn resolve_child_preferred_size<S: LayoutScalar>(
    preferred: &Size<PreferredSizeOf<S>>,
    basis: Size<Option<S>>,
    algorithm: SizingAlgorithm,
    child_owned_axis: Option<PhysicalAxis>,
) -> Size<Result<Option<S>, SizingResolutionError<S>>> {
    let resolve = |value, axis, basis| {
        if child_owned_axis == Some(axis) {
            Ok(None)
        } else {
            resolve_preferred_optional(value, algorithm, axis, basis, true)
        }
    };
    Size::new(
        resolve(&preferred.width, PhysicalAxis::Horizontal, basis.width),
        resolve(&preferred.height, PhysicalAxis::Vertical, basis.height),
    )
}

pub(crate) fn resolve_length_or_zero_fallible<S>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    resolution_or_zero_fallible(length.resolve_with_status(basis))
}

pub(crate) fn resolve_auto_or_zero_fallible<S>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolution_optional_fallible(length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

pub(crate) fn resolution_or_zero_fallible<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => {
            let Some(value) = resolution.value else {
                unreachable!("resolved length resolution must carry a value")
            };
            Ok(value)
        }
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(resolution.status())
        }
        LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

pub(crate) fn resolution_optional_fallible<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            Err(resolution.status())
        }
        LengthResolutionStatus::NonNumeric => Ok(None),
    }
}

pub(crate) trait SizeResultExt<S: LayoutScalar> {
    type Output;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>;
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<S, LengthResolutionStatus<S>>> {
    type Output = Size<S>;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|status| value_resolution_error(node, status))?,
            self.height
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<Option<S>, LengthResolutionStatus<S>>> {
    type Output = Size<Option<S>>;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|status| value_resolution_error(node, status))?,
            self.height
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> SizeResultExt<S> for Size<Result<Option<S>, SizingResolutionError<S>>> {
    type Output = Size<Option<S>>;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Ok(Size::new(
            self.width
                .map_err(|error| sizing_resolution_error(node, error))?,
            self.height
                .map_err(|error| sizing_resolution_error(node, error))?,
        ))
    }
}

pub(crate) trait EdgesResultExt<S: LayoutScalar> {
    type Output;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>;
}

impl<S: LayoutScalar> EdgesResultExt<S> for Edges<Result<S, LengthResolutionStatus<S>>> {
    type Output = Edges<S>;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Ok(Edges::new(
            self.top
                .map_err(|status| value_resolution_error(node, status))?,
            self.right
                .map_err(|status| value_resolution_error(node, status))?,
            self.bottom
                .map_err(|status| value_resolution_error(node, status))?,
            self.left
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

impl<S: LayoutScalar> EdgesResultExt<S> for Edges<Result<Option<S>, LengthResolutionStatus<S>>> {
    type Output = Edges<Option<S>>;

    fn transpose_with_node<'input, Tree, M>(
        self,
        _tree: &Tree,
        node: <Tree as Traverse>::Node,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self::Output, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Ok(Edges::new(
            self.top
                .map_err(|status| value_resolution_error(node, status))?,
            self.right
                .map_err(|status| value_resolution_error(node, status))?,
            self.bottom
                .map_err(|status| value_resolution_error(node, status))?,
            self.left
                .map_err(|status| value_resolution_error(node, status))?,
        ))
    }
}

pub(crate) fn resolve_minimum_optional<S: LayoutScalar>(
    value: &MinSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let basis = percentage_basis(basis);
    match dispatch_minimum_size(value, algorithm, axis, basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, missing_basis_is_indefinite)
        }
    }
}

pub(crate) fn resolve_maximum_optional<S: LayoutScalar>(
    value: &MaxSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let basis = percentage_basis(basis);
    match dispatch_maximum_size(value, algorithm, axis, basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, missing_basis_is_indefinite)
        }
    }
}

pub(crate) fn resolve_flex_basis<S: LayoutScalar>(
    value: &FlexBasisOf<S>,
    axis: PhysicalAxis,
    basis: Option<S>,
) -> Result<ResolvedFlexBasis<S>, SizingResolutionError<S>> {
    let percentage_basis = percentage_basis(basis);
    match dispatch_flex_basis(value, SizingAlgorithm::Flex, axis, percentage_basis) {
        SizingDispatch::Unsupported(unsupported) => {
            Err(SizingResolutionError::Unsupported(unsupported))
        }
        SizingDispatch::Supported(DispatchedSizingRequest::Auto) => Ok(ResolvedFlexBasis::Auto),
        SizingDispatch::Supported(DispatchedSizingRequest::Content) => {
            Ok(ResolvedFlexBasis::Content)
        }
        SizingDispatch::Supported(DispatchedSizingRequest::MinContent) => {
            Ok(ResolvedFlexBasis::MinContent)
        }
        SizingDispatch::Supported(DispatchedSizingRequest::MaxContent) => {
            Ok(ResolvedFlexBasis::MaxContent)
        }
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, percentage_basis, true)
                .map(|value| value.map_or(ResolvedFlexBasis::Content, ResolvedFlexBasis::Definite))
        }
    }
}

/// Fixed-source evidence for orthogonal fallback, separate from query sizing.
/// A caller leaves an unevaluated source Unqualified; admitted parent None is
/// instead a declared indefinite percentage basis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FixedFallbackSource<S: LayoutScalar> {
    Fixed(S),
    NoFixedBound,
    Unqualified,
}

fn fixed_fallback_dispatch<S: LayoutScalar>(
    dispatch: SizingDispatch<'_, S>,
    basis: PercentageBasisOf<S>,
) -> Result<FixedFallbackSource<S>, SizingResolutionError<S>> {
    match dispatch {
        SizingDispatch::Unsupported(error) => Err(SizingResolutionError::Unsupported(error)),
        SizingDispatch::Supported(
            DispatchedSizingRequest::Auto
            | DispatchedSizingRequest::None
            | DispatchedSizingRequest::Content
            | DispatchedSizingRequest::MinContent
            | DispatchedSizingRequest::MaxContent,
        ) => Ok(FixedFallbackSource::NoFixedBound),
        SizingDispatch::Supported(request) => {
            resolve_dispatched_numeric(request, basis, true).map(|value| {
                value.map_or(
                    FixedFallbackSource::NoFixedBound,
                    FixedFallbackSource::Fixed,
                )
            })
        }
    }
}

/// Qualifies the original preferred source only when its owner demands it.
pub(crate) fn fixed_fallback_preferred<S: LayoutScalar>(
    value: &PreferredSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    admitted_basis: Option<S>,
) -> Result<FixedFallbackSource<S>, SizingResolutionError<S>> {
    use super::{PreferredSizeCalcBasis, PreferredSizeView};
    match value.view() {
        PreferredSizeView::Auto | PreferredSizeView::MinContent | PreferredSizeView::MaxContent => {
            return Ok(FixedFallbackSource::NoFixedBound);
        }
        PreferredSizeView::CalcSize(original, calculation)
            if admitted_basis.is_none()
                && matches!(
                    original,
                    PreferredSizeCalcBasis::Any | PreferredSizeCalcBasis::FullPercentage
                )
                && (original == PreferredSizeCalcBasis::FullPercentage
                    || calculation.depends_on_percentage_basis) =>
        {
            return Ok(FixedFallbackSource::NoFixedBound);
        }
        _ => {}
    }
    let basis = percentage_basis(admitted_basis);
    fixed_fallback_dispatch(
        dispatch_preferred_size(value, algorithm, axis, basis),
        basis,
    )
}

/// Qualifies the original minimum, before Auto or measured-minimum substitution.
pub(crate) fn fixed_fallback_minimum<S: LayoutScalar>(
    value: &MinSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    admitted_basis: Option<S>,
) -> Result<FixedFallbackSource<S>, SizingResolutionError<S>> {
    use super::{MinSizeCalcBasis, MinSizeView};
    match value.view() {
        MinSizeView::Auto | MinSizeView::MinContent | MinSizeView::MaxContent => {
            return Ok(FixedFallbackSource::NoFixedBound);
        }
        MinSizeView::CalcSize(original, calculation)
            if admitted_basis.is_none()
                && matches!(
                    original,
                    MinSizeCalcBasis::Any | MinSizeCalcBasis::FullPercentage
                )
                && (original == MinSizeCalcBasis::FullPercentage
                    || calculation.depends_on_percentage_basis) =>
        {
            return Ok(FixedFallbackSource::NoFixedBound);
        }
        _ => {}
    }
    let basis = percentage_basis(admitted_basis);
    fixed_fallback_dispatch(dispatch_minimum_size(value, algorithm, axis, basis), basis)
}

/// Qualifies the original maximum independently of intrinsic query suppression.
pub(crate) fn fixed_fallback_maximum<S: LayoutScalar>(
    value: &MaxSizeOf<S>,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
    admitted_basis: Option<S>,
) -> Result<FixedFallbackSource<S>, SizingResolutionError<S>> {
    use super::{MaxSizeCalcBasis, MaxSizeView};
    match value.view() {
        MaxSizeView::None | MaxSizeView::MinContent | MaxSizeView::MaxContent => {
            return Ok(FixedFallbackSource::NoFixedBound);
        }
        MaxSizeView::CalcSize(original, calculation)
            if admitted_basis.is_none()
                && matches!(
                    original,
                    MaxSizeCalcBasis::Any | MaxSizeCalcBasis::FullPercentage
                )
                && (original == MaxSizeCalcBasis::FullPercentage
                    || calculation.depends_on_percentage_basis) =>
        {
            return Ok(FixedFallbackSource::NoFixedBound);
        }
        _ => {}
    }
    let basis = percentage_basis(admitted_basis);
    fixed_fallback_dispatch(dispatch_maximum_size(value, algorithm, axis, basis), basis)
}
