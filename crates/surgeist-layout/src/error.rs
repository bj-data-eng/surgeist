use crate::DefaultScalar;
use crate::geometry::{FlowAxes, PhysicalAxis};
use crate::node_input::FloatExclusionIntervalErrorOf;
use crate::output::RunMode;
use crate::scalar::LayoutScalar;
use crate::sizing::resolve::SizingResolutionError;
use crate::value::{LengthResolutionStatus, NonNegativeFiniteScalarErrorOf};
use core::fmt;

pub type LayoutResultOf<Node, T, S, M = core::convert::Infallible> =
    Result<T, LayoutErrorOf<Node, S, M>>;
pub type LayoutResult<Node, T, M> = LayoutResultOf<Node, T, DefaultScalar, M>;

#[derive(Debug)]
pub struct LayoutErrorOf<Node, S: LayoutScalar = DefaultScalar, M = core::convert::Infallible> {
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    kind: LayoutErrorKindOf<S, M>,
}

pub type LayoutError<Node, M = core::convert::Infallible> = LayoutErrorOf<Node, DefaultScalar, M>;

impl<Node, S, M> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    pub const fn new(
        site: LayoutErrorSiteOf<Node>,
        operation: LayoutOperation,
        kind: LayoutErrorKindOf<S, M>,
    ) -> Self {
        Self {
            site,
            operation,
            kind,
        }
    }

    #[must_use]
    pub const fn site(&self) -> LayoutErrorSiteOf<Node>
    where
        Node: Copy,
    {
        self.site
    }

    #[must_use]
    pub const fn operation(&self) -> LayoutOperation {
        self.operation
    }

    #[must_use]
    pub const fn kind(&self) -> &LayoutErrorKindOf<S, M> {
        &self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutErrorSiteOf<Node> {
    Node(Node),
    ContainerSubject {
        container: Node,
        subject: Node,
    },
    FragmentContainingBlock {
        container: Node,
        subject: Node,
        fragmentainer_index: usize,
        axis: PhysicalAxis,
    },
    Standalone,
}

pub type LayoutErrorSite<Node> = LayoutErrorSiteOf<Node>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutOperation {
    RootLayout,
    ChildLayout,
    HiddenLayout,
    LeafMeasurement,
    ValueResolution,
    CacheAccess,
    CacheInvalidation,
    FloatExclusionQuery,
    RoundingFinalization,
    GridLanePlacement,
    TextCapture,
    InlineWrapping,
    TextSelection,
    TextPlacement,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LayoutErrorKindOf<S: LayoutScalar = DefaultScalar, M = core::convert::Infallible> {
    InvalidInput(LayoutInvalidInputOf<S>),
    BlockFragmentation(crate::BlockFragmentationFailure),
    MissingContext(LayoutMissingContext),
    UnsupportedCapability(LayoutUnsupportedCapability),
    Measurement(M),
    /// Layout could not allocate its actual owned native working storage.
    Allocation,
    /// The actual Text owner refused the captured protocol operation.
    Text(surgeist_text::Error),
    /// A scope-alignment limit, overflow or allocation budget was exhausted
    /// wherever layout raised the Text error. The Text error names the
    /// exhausted limit and is the error's source; a budget refusal is not a
    /// Text protocol refusal and not cancellation.
    ResourceLimit(surgeist_text::Error),
    /// Canonical scroll geometry rejected geometry produced by layout. The
    /// cause names the rejected value.
    ScrollGeometry(LayoutScrollGeometryErrorOf<S>),
    /// A complete canonical Text feedback basis repeated without acceptance.
    TextFeedbackCycle,
    /// Numeric inline avoidance repeated an actual metric-envelope/float-band
    /// basis without convergence. No chosen line has been published.
    InlineFeedbackCycle,
    /// Layout's completed justification correction still exceeds the actual
    /// line budget. Both quantities are logical Text advances, before scalar
    /// conversion or physical placement.
    TextSpacingBudget {
        available: f64,
        advance: f64,
    },
    TextNumeric {
        value: f64,
        reason: LayoutTextNumericError,
    },
    InternalInvariant(LayoutInternalInvariant),
}

pub type LayoutErrorKind<M = core::convert::Infallible> = LayoutErrorKindOf<DefaultScalar, M>;

/// Failure at an actual scalar geometry or Text-to-canonical conversion boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutTextNumericError {
    NonFinite,
    OutOfRange,
    NonZeroLost,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LayoutInvalidInputOf<S: LayoutScalar = DefaultScalar> {
    BlockFragmentationRequest,
    InlineBaseline(crate::InlineBaselineError),
    GridBaseline(crate::GridBaselineError),
    RootContext(crate::RootContextMismatch),
    MeasurementOutput(InvalidMeasurementOutputOf<S>),
    InvalidNumeric {
        value: S,
    },
    /// A visible inline control disagrees with its containing block's flow.
    InlineFlowMismatch {
        expected: FlowAxes,
        actual: FlowAxes,
    },
    /// Paired box-edge requests disagree on their actual parent operation/basis.
    InlineBoundaryParentMismatch {
        expected: Option<FlowAxes>,
        actual: Option<FlowAxes>,
    },
    /// The requested non-atomic parent edge runs on an incompatible physical axis.
    InlineBoundaryParentAxis {
        parent: FlowAxes,
        formatter: FlowAxes,
    },
    AtomicInlineParticipation {
        reason: AtomicInlineParticipationRoleError,
    },
    NonBoxNodeRole {
        reason: NonBoxNodeRoleError,
    },
    FloatExclusionRole {
        reason: FloatExclusionRoleError,
    },
    FloatExclusionProviderOutput {
        error: FloatExclusionIntervalErrorOf<S>,
    },
    AnonymousBlockRole,
    AnonymousIndentMismatch,
    InvalidationNodeNotReachable,
    /// The subject does not occur exactly once as a direct child of the container.
    InlineAtomicNotDirectChild,
    /// The direct child is also reachable along another path from its container.
    InlineAtomicAmbiguousOccurrence,
    TreeTopologyCycle,
    /// The grid container's named lines or template areas violate the Grid
    /// specification. The error site is the grid container.
    GridNames(crate::NamedGridErrorReport),
    TextBinding(crate::text_layout::TextBindingError),
    WrappingScope(crate::inline_wrapping::WrappingScopeError),
}

pub type LayoutInvalidInput = LayoutInvalidInputOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicInlineParticipationRoleError {
    MissingForAtomicInline,
    UnexpectedForNonAtomic,
    UnexpectedForPreparedAtomic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonBoxNodeRoleError {
    NonCanonicalNodeInput,
    HasChildren,
    HasLeafMeasurement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatExclusionRoleError {
    Hidden,
    NonFloating,
    Absolute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutMissingContext {
    NamedInlineBaseline,
    /// An eligible Grid participant did not establish the requested named set.
    NamedGridBaseline,
    RequiredBasis,
    FloatExclusionProvider,
    TextCapture,
    InlineWrapping,
    /// The selected hyphenation needs an owning completed-block continuation.
    TextBlockTail,
    TextOutOfFlowGeometry,
    /// A computed semantic cut has no proved side in the actual native flow.
    ComputedLineBoundaryIncidence,
    AnonymousGeneratingBlock,
}

impl core::hash::Hash for PhysicalAxis {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SizingProperty {
    Preferred,
    Minimum,
    Maximum,
    FlexBasis,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SizingAlgorithm {
    Leaf,
    Block,
    Flex,
    Grid,
    GridLanes,
    Positioned,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CalcSizeBehaviorBasis {
    Auto,
    None,
    Content,
    MinContent,
    MaxContent,
    Stretch,
    FitContent,
    Contain,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SizingBehavior {
    MinContent,
    MaxContent,
    FitContentFunction,
    Stretch,
    FitContent,
    Contain,
    CalcSize(CalcSizeBehaviorBasis),
}

/// A typed description of sizing behavior owned by a later algorithm initiative.
///
/// The payload is output-only; callers inspect descriptors returned through
/// [`LayoutUnsupportedCapability`] rather than constructing them.
///
/// ```compile_fail
/// use surgeist_layout::{
///     PhysicalAxis, SizingAlgorithm, SizingBehavior, SizingProperty,
///     UnsupportedSizingBehavior,
/// };
/// let _ = UnsupportedSizingBehavior {
///     property: SizingProperty::Preferred,
///     behavior: SizingBehavior::Stretch,
///     algorithm: SizingAlgorithm::Leaf,
///     axis: PhysicalAxis::Horizontal,
/// };
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{
///     PhysicalAxis, SizingAlgorithm, SizingBehavior, SizingProperty,
///     UnsupportedSizingBehavior,
/// };
/// let _ = UnsupportedSizingBehavior::new(
///     SizingProperty::Preferred,
///     SizingBehavior::Stretch,
///     SizingAlgorithm::Leaf,
///     PhysicalAxis::Horizontal,
/// );
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnsupportedSizingBehavior {
    property: SizingProperty,
    behavior: SizingBehavior,
    algorithm: SizingAlgorithm,
    axis: PhysicalAxis,
}

impl UnsupportedSizingBehavior {
    pub(crate) const fn new(
        property: SizingProperty,
        behavior: SizingBehavior,
        algorithm: SizingAlgorithm,
        axis: PhysicalAxis,
    ) -> Self {
        Self {
            property,
            behavior,
            algorithm,
            axis,
        }
    }

    #[must_use]
    pub const fn property(self) -> SizingProperty {
        self.property
    }

    #[must_use]
    pub const fn behavior(self) -> SizingBehavior {
        self.behavior
    }

    #[must_use]
    pub const fn algorithm(self) -> SizingAlgorithm {
        self.algorithm
    }

    #[must_use]
    pub const fn axis(self) -> PhysicalAxis {
        self.axis
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LayoutUnsupportedCapability {
    BlockFragmentationProfile,
    LaterFriBehavior,
    /// Size preparation requires a definite root-local Block and an empty,
    /// in-flow, unmeasured InlineBlock.
    InlineAtomicSizePreparation,
    /// Named Grid baseline placement is not supplied by the Lane owner.
    NamedGridLanesBaseline {
        axis: crate::GridAxisKind,
    },
    SizingBehavior(UnsupportedSizingBehavior),
    /// The actual Text-backed run needs an unsupported physical writing-mode adapter.
    TextWritingMode(crate::WritingMode),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutInternalInvariant {
    /// A length resolution that succeeded was reported on a failure path.
    ResolvedLengthOnFailurePath,
    InvalidRoundedScrollGeometry,
    InvalidRoundedInlineFragmentGeometry,
    /// A private balanced-line basis violated its finite/count/replay invariant.
    InvalidInlineBalance,
    /// A private paragraph-tail program did not reproduce its retained basis.
    InvalidInlineTailSelection,
    /// A private Pretty program did not reproduce its retained phase or fallback basis.
    InvalidInlinePrettySelection,
    /// A deferred receiving metric no longer matched its retained atomic source.
    InvalidAtomicReceiverBasis,
    MissingLeafMeasurementProvider,
    MissingStagedUnroundedOutput,
    /// A staged output was consumed in a different geometry/readback phase.
    InvalidOutputGeometryPhase,
    /// Grid's contribution incidence or resolved track state violated its native contract.
    InvalidGridTrackSizing,
    SubgridTrackInheritance,
    SubgridBaselineInheritance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidMeasurementOutputOf<S: LayoutScalar = DefaultScalar> {
    axis: PhysicalAxis,
    error: NonNegativeFiniteScalarErrorOf<S>,
}

pub type InvalidMeasurementOutput = InvalidMeasurementOutputOf<DefaultScalar>;

impl<S: LayoutScalar> InvalidMeasurementOutputOf<S> {
    pub(crate) const fn new(axis: PhysicalAxis, error: NonNegativeFiniteScalarErrorOf<S>) -> Self {
        Self { axis, error }
    }

    /// Returns the physical axis of the rejected measurement output.
    #[must_use]
    pub const fn axis(self) -> PhysicalAxis {
        self.axis
    }

    #[must_use]
    pub const fn error(self) -> NonNegativeFiniteScalarErrorOf<S> {
        self.error
    }
}

pub(crate) fn layout_own_geometry_error<Node, S, M, E>(
    node: Node,
    run_mode: RunMode,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
    E: Into<ScrollGeometryCauseOf<S>>,
{
    let operation = if run_mode == RunMode::PerformRootLayout {
        LayoutOperation::RootLayout
    } else {
        LayoutOperation::ChildLayout
    };
    scroll_geometry_error(LayoutErrorSiteOf::Node(node), operation, error)
}

pub(crate) fn layout_child_geometry_error<Node, S, M, E>(
    container: Node,
    subject: Node,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
    E: Into<ScrollGeometryCauseOf<S>>,
{
    scroll_geometry_error(
        LayoutErrorSiteOf::ContainerSubject { container, subject },
        LayoutOperation::ChildLayout,
        error,
    )
}

pub(crate) fn scroll_geometry_error<Node, S, M, E>(
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    error: E,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
    E: Into<ScrollGeometryCauseOf<S>>,
{
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::ScrollGeometry(LayoutScrollGeometryErrorOf {
            cause: error.into(),
        }),
    )
}

/// Classifies a Text refusal by its cause. An exhausted work, record or
/// storage budget of the scope-alignment accounting is a resource limit of
/// layout wherever layout raises it; any other refusal remains the Text
/// owner's.
pub(crate) fn text_error_kind<S: LayoutScalar, M>(
    error: surgeist_text::Error,
) -> LayoutErrorKindOf<S, M> {
    let budget = matches!(
        error.detail(),
        Some(surgeist_text::ErrorDetail::ScopeAlignment {
            reason: surgeist_text::scope_alignment::ScopeAlignmentError::Limit { .. }
                | surgeist_text::scope_alignment::ScopeAlignmentError::Overflow { .. }
                | surgeist_text::scope_alignment::ScopeAlignmentError::Allocation { .. },
        })
    );
    if budget {
        LayoutErrorKindOf::ResourceLimit(error)
    } else {
        LayoutErrorKindOf::Text(error)
    }
}

/// Builds the failure for a Text refusal, classified by its cause.
pub(crate) fn staging_text_error<Node, S: LayoutScalar, M>(
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    error: surgeist_text::Error,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(site, operation, text_error_kind(error))
}

pub(crate) fn value_resolution_error<Node, S, M>(
    node: Node,
    status: LengthResolutionStatus<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    value_resolution_error_at_site(LayoutErrorSiteOf::Node(node), status)
}

pub(crate) fn value_resolution_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    status: LengthResolutionStatus<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = match status {
        LengthResolutionStatus::MissingBasis => {
            LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
        }
        LengthResolutionStatus::InvalidNumeric { value } => {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
        }
        LengthResolutionStatus::NonNumeric => {
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior)
        }
        LengthResolutionStatus::Resolved => LayoutErrorKindOf::InternalInvariant(
            LayoutInternalInvariant::ResolvedLengthOnFailurePath,
        ),
    };

    LayoutErrorOf::new(site, LayoutOperation::ValueResolution, kind)
}

pub(crate) fn sizing_resolution_error<Node, S, M>(
    node: Node,
    error: SizingResolutionError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    sizing_resolution_error_at_site(LayoutErrorSiteOf::Node(node), error)
}

pub(crate) fn sizing_resolution_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    error: SizingResolutionError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    match error {
        SizingResolutionError::Status(status) => value_resolution_error_at_site(site, status),
        SizingResolutionError::Unsupported(unsupported) => LayoutErrorOf::new(
            site,
            LayoutOperation::ValueResolution,
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::SizingBehavior(
                unsupported,
            )),
        ),
    }
}

/// Refusal from the common immutable inline-scope owner.
pub(crate) fn wrapping_error<Node, S: LayoutScalar, M>(
    node: Node,
    reason: crate::inline_wrapping::WrappingScopeError,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::InlineWrapping,
        if reason == crate::inline_wrapping::WrappingScopeError::Allocation {
            LayoutErrorKindOf::Allocation
        } else {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(reason))
        },
    )
}

/// Canonical scroll geometry rejected geometry produced by layout.
///
/// The value keeps the rejected scroll rectangle, coordinate, contribution or
/// box fact. Its [`Debug`](fmt::Debug) output names that fact; no accessor
/// exposes the private classification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutScrollGeometryErrorOf<S: LayoutScalar = DefaultScalar> {
    pub(crate) cause: ScrollGeometryCauseOf<S>,
}

pub type LayoutScrollGeometryError = LayoutScrollGeometryErrorOf<DefaultScalar>;

/// The rejected canonical scroll fact behind a [`LayoutScrollGeometryErrorOf`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ScrollGeometryCauseOf<S: LayoutScalar> {
    Canonical(crate::scroll::CanonicalScrollGeometryErrorOf<S>),
    Contribution(crate::scroll::ScrollContributionErrorOf<S>),
    BoxClipGutter(crate::scroll::ScrollBoxClipGutterErrorOf<S>),
    Rect(crate::ScrollRectErrorOf<S>),
    Coordinate(crate::ScrollCoordinateErrorOf<S>),
}

macro_rules! scroll_geometry_cause_from {
    ($($variant:ident($source:ty)),* $(,)?) => {$(
        impl<S: LayoutScalar> From<$source> for ScrollGeometryCauseOf<S> {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    )*};
}

scroll_geometry_cause_from! {
    Canonical(crate::scroll::CanonicalScrollGeometryErrorOf<S>),
    Contribution(crate::scroll::ScrollContributionErrorOf<S>),
    BoxClipGutter(crate::scroll::ScrollBoxClipGutterErrorOf<S>),
    Rect(crate::ScrollRectErrorOf<S>),
    Coordinate(crate::ScrollCoordinateErrorOf<S>),
}

impl<S: LayoutScalar> fmt::Display for LayoutScrollGeometryErrorOf<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cause {
            ScrollGeometryCauseOf::Canonical(cause) => {
                write!(f, "canonical scroll geometry rejected: {cause:?}")
            }
            ScrollGeometryCauseOf::Contribution(cause) => {
                write!(f, "scroll contribution rejected: {cause:?}")
            }
            ScrollGeometryCauseOf::BoxClipGutter(cause) => {
                write!(f, "scroll box clip or gutter rejected: {cause:?}")
            }
            ScrollGeometryCauseOf::Rect(_) => f.write_str("scroll rectangle rejected"),
            ScrollGeometryCauseOf::Coordinate(cause) => {
                write!(f, "scroll coordinate rejected: {cause:?}")
            }
        }
    }
}

impl<S: LayoutScalar> std::error::Error for LayoutScrollGeometryErrorOf<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            ScrollGeometryCauseOf::Rect(cause) => Some(cause),
            _ => None,
        }
    }
}

impl fmt::Display for LayoutOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RootLayout => "root layout",
            Self::ChildLayout => "child layout",
            Self::HiddenLayout => "hidden layout",
            Self::LeafMeasurement => "leaf measurement",
            Self::ValueResolution => "value resolution",
            Self::CacheAccess => "cache access",
            Self::CacheInvalidation => "cache invalidation",
            Self::FloatExclusionQuery => "float exclusion query",
            Self::RoundingFinalization => "rounding finalization",
            Self::GridLanePlacement => "grid lane placement",
            Self::TextCapture => "text capture",
            Self::InlineWrapping => "inline wrapping",
            Self::TextSelection => "text selection",
            Self::TextPlacement => "text placement",
        })
    }
}

impl<Node: fmt::Debug> fmt::Display for LayoutErrorSiteOf<Node> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(node) => write!(f, "node {node:?}"),
            Self::ContainerSubject { container, subject } => {
                write!(f, "subject {subject:?} in container {container:?}")
            }
            Self::FragmentContainingBlock {
                container,
                subject,
                fragmentainer_index,
                axis,
            } => write!(
                f,
                "fragment containing block of subject {subject:?} in container {container:?} \
                 (fragmentainer {fragmentainer_index}, {axis:?} axis)"
            ),
            Self::Standalone => f.write_str("a standalone request"),
        }
    }
}

/// Formats the kind without repeating a cause that `source()` of the error
/// returns: measurement, Text, resource-limit and scroll geometry failures, and
/// rejected measurement output and float exclusion provider output, state the
/// failure and the facts the cause does not carry. Other invalid input prints
/// its `Debug` form.
impl<S: LayoutScalar, M> fmt::Display for LayoutErrorKindOf<S, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(output)) => {
                write!(
                    f,
                    "measurement output rejected on the {:?} axis",
                    output.axis
                )
            }
            Self::InvalidInput(LayoutInvalidInputOf::FloatExclusionProviderOutput { .. }) => {
                f.write_str("float exclusion provider output rejected")
            }
            Self::InvalidInput(input) => write!(f, "invalid input: {input:?}"),
            Self::BlockFragmentation(failure) => {
                write!(f, "block fragmentation failed: {failure:?}")
            }
            Self::MissingContext(context) => write!(f, "missing context: {context:?}"),
            Self::UnsupportedCapability(capability) => {
                write!(f, "unsupported capability: {capability:?}")
            }
            Self::Measurement(_) => f.write_str("measurement failed"),
            Self::Allocation => f.write_str("layout could not allocate its working storage"),
            Self::Text(_) => f.write_str("text operation refused"),
            Self::ResourceLimit(_) => f.write_str("layout resource limit reached"),
            Self::ScrollGeometry(_) => f.write_str("scroll geometry rejected"),
            Self::TextFeedbackCycle => {
                f.write_str("text feedback repeated a complete basis without acceptance")
            }
            Self::InlineFeedbackCycle => {
                f.write_str("inline avoidance repeated a basis without convergence")
            }
            Self::TextSpacingBudget { available, advance } => write!(
                f,
                "justification advance {advance} exceeds the available line budget {available}"
            ),
            Self::TextNumeric { value, reason } => {
                write!(f, "text value {value} is invalid: {reason:?}")
            }
            Self::InternalInvariant(invariant) => {
                write!(f, "internal layout invariant violated: {invariant:?}")
            }
        }
    }
}

/// Formats as `<operation> failed at <site>: <kind>`.
///
/// `Node` needs `Debug` to name the site. The measurement error is reported
/// through `source()`, so `M` needs no bound.
impl<Node, S, M> fmt::Display for LayoutErrorOf<Node, S, M>
where
    Node: fmt::Debug,
    S: LayoutScalar,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} failed at {}: {}",
            self.operation, self.site, self.kind
        )
    }
}

/// `source()` returns the measurement error, the Text error of a Text or
/// resource-limit failure, the scroll geometry cause, the scalar error of a
/// rejected measurement output, or the rejected float exclusion interval.
///
/// Beyond `LayoutScalar` (which is `'static`), `Node` needs `Debug` and `M`
/// needs `Error + 'static`.
impl<Node, S, M> std::error::Error for LayoutErrorOf<Node, S, M>
where
    Node: fmt::Debug,
    S: LayoutScalar,
    M: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            LayoutErrorKindOf::Measurement(error) => Some(error),
            LayoutErrorKindOf::Text(error) | LayoutErrorKindOf::ResourceLimit(error) => Some(error),
            LayoutErrorKindOf::ScrollGeometry(error) => Some(error),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(output)) => {
                Some(&output.error)
            }
            LayoutErrorKindOf::InvalidInput(
                LayoutInvalidInputOf::FloatExclusionProviderOutput { error },
            ) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
