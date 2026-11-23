//! The layout failure taxonomy and the descriptors of sizing behavior.

use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis, PhysicalSide};
use crate::node_input::FloatExclusionIntervalErrorOf;
use crate::output::RunMode;
use crate::scalar::LayoutScalar;
use crate::sizing::resolve::SizingResolutionError;
use crate::value::{LengthResolutionStatus, NonNegativeFiniteScalarErrorOf};
use crate::{grid::GridAxisKind, node_input::WritingMode, scalar::DefaultScalar};
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
    BlockFragmentation(crate::block_fragmentation::BlockFragmentationFailure),
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

impl core::fmt::Display for LayoutTextNumericError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "the value is not finite",
            Self::OutOfRange => "the value is outside the representable range",
            Self::NonZeroLost => "a nonzero contribution was lost to rounding or arithmetic",
        })
    }
}

impl std::error::Error for LayoutTextNumericError {}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LayoutInvalidInputOf<S: LayoutScalar = DefaultScalar> {
    BlockFragmentationRequest,
    InlineBaseline(crate::node_input::InlineBaselineError),
    GridBaseline(crate::node_input::GridBaselineError),
    RootContext(crate::output::RootContextMismatch),
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
    GridNames(crate::grid::NamedGridError),
    TextBinding(crate::text_layout::TextBindingError),
    WrappingScope(crate::inline_wrapping::WrappingScopeError),
}

pub type LayoutInvalidInput = LayoutInvalidInputOf<DefaultScalar>;

impl<S: LayoutScalar> fmt::Display for LayoutInvalidInputOf<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumeric { value } => {
                write!(f, "numeric value {} is invalid", value.to_f64())
            }
            Self::BlockFragmentationRequest => {
                f.write_str("the block fragmentation request does not match this root layout")
            }
            Self::InlineBaseline(_) => f.write_str("an inline baseline fact is invalid"),
            Self::GridBaseline(_) => f.write_str("a grid baseline fact is invalid"),
            Self::RootContext(_) => f.write_str("the root context disagrees with the node's input"),
            Self::MeasurementOutput(_) => f.write_str("a measurement output is invalid"),
            Self::InlineFlowMismatch { .. } => f.write_str(
                "a visible inline control's flow axes differ from its containing block's",
            ),
            Self::InlineBoundaryParentMismatch { .. } => f.write_str(
                "an inline box edge's parent flow axes differ from the expected parent flow axes",
            ),
            Self::InlineBoundaryParentAxis { .. } => f.write_str(
                "an inline box edge's parent runs on a physical axis incompatible with its \
                 formatter",
            ),
            Self::AtomicInlineParticipation { .. } => {
                f.write_str("an atomic inline's participation facts are invalid")
            }
            Self::NonBoxNodeRole { .. } => f.write_str("a non-box node's role is invalid"),
            Self::FloatExclusionRole { .. } => {
                f.write_str("a float exclusion's node role is invalid")
            }
            Self::FloatExclusionProviderOutput { .. } => {
                f.write_str("float exclusion provider output is invalid")
            }
            Self::AnonymousBlockRole => {
                f.write_str("an anonymous block has a role an anonymous block cannot have")
            }
            Self::AnonymousIndentMismatch => f.write_str(
                "an anonymous block's text indent differs from that of its generating block",
            ),
            Self::InvalidationNodeNotReachable => {
                f.write_str("an invalidated node is not reachable from the layout root")
            }
            Self::InlineAtomicNotDirectChild => {
                f.write_str("the atomic inline is not exactly one direct child of its container")
            }
            Self::InlineAtomicAmbiguousOccurrence => f.write_str(
                "the direct child is also reachable along another path from its container",
            ),
            Self::TreeTopologyCycle => f.write_str("the tree contains a cycle"),
            Self::GridNames(_) => {
                f.write_str("the grid container's named lines or template areas are invalid")
            }
            Self::TextBinding(_) => f.write_str("the text binding is invalid"),
            Self::WrappingScope(_) => f.write_str("the wrapping scopes are invalid"),
        }
    }
}

/// `source()` returns the payload that carries the detail of the invalid input.
impl<S: LayoutScalar> std::error::Error for LayoutInvalidInputOf<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InlineBaseline(error) => Some(error),
            Self::GridBaseline(error) => Some(error),
            Self::RootContext(error) => Some(error),
            Self::MeasurementOutput(error) => Some(error),
            Self::AtomicInlineParticipation { reason } => Some(reason),
            Self::NonBoxNodeRole { reason } => Some(reason),
            Self::FloatExclusionRole { reason } => Some(reason),
            Self::FloatExclusionProviderOutput { error } => Some(error),
            Self::GridNames(error) => Some(error),
            Self::TextBinding(error) => Some(error),
            Self::WrappingScope(error) => Some(error),
            Self::BlockFragmentationRequest
            | Self::InvalidNumeric { .. }
            | Self::InlineFlowMismatch { .. }
            | Self::InlineBoundaryParentMismatch { .. }
            | Self::InlineBoundaryParentAxis { .. }
            | Self::AnonymousBlockRole
            | Self::AnonymousIndentMismatch
            | Self::InvalidationNodeNotReachable
            | Self::InlineAtomicNotDirectChild
            | Self::InlineAtomicAmbiguousOccurrence
            | Self::TreeTopologyCycle => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomicInlineParticipationRoleError {
    MissingForAtomicInline,
    UnexpectedForNonAtomic,
    UnexpectedForPreparedAtomic,
}

impl core::fmt::Display for AtomicInlineParticipationRoleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::MissingForAtomicInline => "an atomic inline requires atomic inline participation facts and has none",
            Self::UnexpectedForNonAtomic => "a node that is not an atomic inline carries atomic inline participation facts",
            Self::UnexpectedForPreparedAtomic => "a prepared atomic inline carries atomic inline participation facts that text already supplies",
        })
    }
}

impl std::error::Error for AtomicInlineParticipationRoleError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonBoxNodeRoleError {
    NonCanonicalNodeInput,
    HasChildren,
    HasLeafMeasurement,
}

impl core::fmt::Display for NonBoxNodeRoleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NonCanonicalNodeInput => {
                "a non-box node's node input is not the canonical non-box input"
            }
            Self::HasChildren => "a non-box node has children",
            Self::HasLeafMeasurement => "a non-box node has a leaf measurement",
        })
    }
}

impl std::error::Error for NonBoxNodeRoleError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatExclusionRoleError {
    Hidden,
    NonFloating,
    Absolute,
}

impl core::fmt::Display for FloatExclusionRoleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Hidden => "a hidden node cannot supply a float exclusion",
            Self::NonFloating => "a node that does not float cannot supply a float exclusion",
            Self::Absolute => "an absolutely positioned node cannot supply a float exclusion",
        })
    }
}

impl std::error::Error for FloatExclusionRoleError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LayoutMissingContext {
    NamedInlineBaseline,
    /// An eligible Grid participant did not establish the requested named set.
    NamedGridBaseline,
    RequiredBasis,
    FloatExclusionProvider,
    TextCapture,
    /// The selected hyphenation needs an owning completed-block continuation.
    TextBlockTail,
    TextOutOfFlowGeometry,
    /// A computed semantic cut has no proved side in the actual native flow.
    ComputedLineBoundaryIncidence,
    AnonymousGeneratingBlock,
}

impl core::fmt::Display for LayoutMissingContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NamedInlineBaseline => {
                "the named inline baseline that the request needs is missing"
            }
            Self::NamedGridBaseline => {
                "an eligible grid participant did not establish the requested named baseline"
            }
            Self::RequiredBasis => "a definite basis that the operation requires is missing",
            Self::FloatExclusionProvider => {
                "the tree supplied no float exclusion interval for the requested query"
            }
            Self::TextCapture => "the text capture that the flow requires is missing",
            Self::TextBlockTail => {
                "the selected hyphenation needs a completed-block continuation that is missing"
            }
            Self::TextOutOfFlowGeometry => {
                "the geometry of an out-of-flow box that text placement requires is missing"
            }
            Self::ComputedLineBoundaryIncidence => {
                "a computed line boundary has no proved side in the native flow"
            }
            Self::AnonymousGeneratingBlock => "an anonymous block has no generating block context",
        })
    }
}

impl std::error::Error for LayoutMissingContext {}

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
/// use surgeist_layout::{error::{SizingAlgorithm, SizingBehavior, SizingProperty, UnsupportedSizingBehavior}, geometry::PhysicalAxis};
/// let _ = UnsupportedSizingBehavior {
///     property: SizingProperty::Preferred,
///     behavior: SizingBehavior::Stretch,
///     algorithm: SizingAlgorithm::Leaf,
///     axis: PhysicalAxis::Horizontal,
/// };
/// ```
///
/// ```compile_fail
/// use surgeist_layout::{error::{SizingAlgorithm, SizingBehavior, SizingProperty, UnsupportedSizingBehavior}, geometry::PhysicalAxis};
/// let _ = UnsupportedSizingBehavior::new(
///     SizingProperty::Preferred,
///     SizingBehavior::Stretch,
///     SizingAlgorithm::Leaf,
///     PhysicalAxis::Horizontal,
/// );
/// ```
///
/// The imports of the failing examples above resolve:
///
/// ```no_run
/// use surgeist_layout::{error::{SizingAlgorithm, SizingBehavior, SizingProperty, UnsupportedSizingBehavior}, geometry::PhysicalAxis};
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
    /// An inline text or text control node whose parent does not accept inline
    /// text.
    NonBoxNodeOutsideInlineFlow,
    /// Value resolution received `LengthResolutionStatus::NonNumeric`, a
    /// represented length that has no number.
    NonNumericLength,
    /// Size preparation requires a definite root-local Block and an empty,
    /// in-flow, unmeasured InlineBlock.
    InlineAtomicSizePreparation,
    /// Named Grid baseline placement is not supplied by the Lane owner.
    NamedGridLanesBaseline {
        axis: GridAxisKind,
    },
    SizingBehavior(UnsupportedSizingBehavior),
    /// The actual Text-backed run needs an unsupported physical writing-mode adapter.
    TextWritingMode(WritingMode),
}

impl fmt::Display for LayoutUnsupportedCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockFragmentationProfile => {
                f.write_str("the block fragmentation profile is not supported")
            }
            Self::NonBoxNodeOutsideInlineFlow => f.write_str(
                "an inline text or text control node has a parent that does not accept inline text",
            ),
            Self::NonNumericLength => {
                f.write_str("a non-numeric length value cannot be used where a number is required")
            }
            Self::InlineAtomicSizePreparation => f.write_str(
                "size preparation requires a definite root-local block and an empty, in-flow, \
                 unmeasured inline-block",
            ),
            Self::NamedGridLanesBaseline { axis } => write!(
                f,
                "named grid baseline placement on the {} axis is not supplied for grid lanes",
                grid_axis_label(*axis)
            ),
            Self::SizingBehavior(unsupported) => {
                f.write_str("the ")?;
                match unsupported.behavior() {
                    SizingBehavior::MinContent => f.write_str("min-content")?,
                    SizingBehavior::MaxContent => f.write_str("max-content")?,
                    SizingBehavior::FitContentFunction => f.write_str("fit-content function")?,
                    SizingBehavior::Stretch => f.write_str("stretch")?,
                    SizingBehavior::FitContent => f.write_str("fit-content")?,
                    SizingBehavior::Contain => f.write_str("contain")?,
                    SizingBehavior::CalcSize(basis) => write!(
                        f,
                        "calc-size with the {} basis",
                        calc_size_basis_label(basis)
                    )?,
                }
                write!(
                    f,
                    " sizing behavior of the {} is not supported by the {} algorithm on the {} \
                     axis",
                    sizing_property_label(unsupported.property()),
                    sizing_algorithm_label(unsupported.algorithm()),
                    physical_axis_label(unsupported.axis())
                )
            }
            Self::TextWritingMode(mode) => write!(
                f,
                "the {} writing mode needs a physical adapter that text-backed runs do not support",
                writing_mode_label(*mode)
            ),
        }
    }
}

impl std::error::Error for LayoutUnsupportedCapability {}

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

impl core::fmt::Display for LayoutInternalInvariant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::ResolvedLengthOnFailurePath => {
                "a length that resolved was reported on a failure path"
            }
            Self::InvalidRoundedScrollGeometry => "rounding produced invalid scroll geometry",
            Self::InvalidRoundedInlineFragmentGeometry => {
                "inline fitting produced invalid inline fragment geometry"
            }
            Self::InvalidInlineBalance => {
                "a balanced-line basis violated its finite, count or replay invariant"
            }
            Self::InvalidInlineTailSelection => {
                "a paragraph-tail program did not reproduce its retained basis"
            }
            Self::InvalidInlinePrettySelection => {
                "a pretty-wrapping program did not reproduce its retained phase or fallback basis"
            }
            Self::InvalidAtomicReceiverBasis => {
                "a deferred receiving metric no longer matched its retained atomic source"
            }
            Self::MissingLeafMeasurementProvider => {
                "the tree supplied no leaf measurement for a node that requires one"
            }
            Self::MissingStagedUnroundedOutput => {
                "a staged output, or the part of it that the phase needs, is missing"
            }
            Self::InvalidOutputGeometryPhase => {
                "a staged output was consumed in a different geometry or readback phase"
            }
            Self::InvalidGridTrackSizing => {
                "grid contribution incidence or resolved track state violated its native contract"
            }
            Self::SubgridTrackInheritance => {
                "subgrid track inheritance violated its native contract"
            }
            Self::SubgridBaselineInheritance => {
                "subgrid baseline inheritance violated its native contract"
            }
        })
    }
}

impl std::error::Error for LayoutInternalInvariant {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidMeasurementOutputOf<S: LayoutScalar = DefaultScalar> {
    axis: PhysicalAxis,
    error: NonNegativeFiniteScalarErrorOf<S>,
}

pub type InvalidMeasurementOutput = InvalidMeasurementOutputOf<DefaultScalar>;

impl<S: LayoutScalar> fmt::Display for InvalidMeasurementOutputOf<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "measurement output rejected on the {} axis",
            physical_axis_label(self.axis)
        )
    }
}

/// `source()` returns the scalar validation error of the rejected size.
impl<S: LayoutScalar> std::error::Error for InvalidMeasurementOutputOf<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

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
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::NonNumericLength)
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
    Rect(crate::scroll::ScrollRectErrorOf<S>),
    Coordinate(crate::scroll::ScrollCoordinateErrorOf<S>),
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
    Rect(crate::scroll::ScrollRectErrorOf<S>),
    Coordinate(crate::scroll::ScrollCoordinateErrorOf<S>),
}

impl<S: LayoutScalar> fmt::Display for LayoutScrollGeometryErrorOf<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match &self.cause {
            ScrollGeometryCauseOf::Canonical(_) => "canonical scroll geometry is invalid",
            ScrollGeometryCauseOf::Contribution(_) => "a scroll contribution is invalid",
            ScrollGeometryCauseOf::BoxClipGutter(_) => "a scroll box clip or gutter is invalid",
            ScrollGeometryCauseOf::Rect(_) => "a scroll rectangle is invalid",
            ScrollGeometryCauseOf::Coordinate(_) => "a scroll coordinate is invalid",
        })
    }
}

impl<S: LayoutScalar> std::error::Error for LayoutScrollGeometryErrorOf<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            ScrollGeometryCauseOf::Canonical(cause) => Some(cause),
            ScrollGeometryCauseOf::Contribution(cause) => Some(cause),
            ScrollGeometryCauseOf::BoxClipGutter(cause) => Some(cause),
            ScrollGeometryCauseOf::Rect(cause) => Some(cause),
            ScrollGeometryCauseOf::Coordinate(cause) => Some(cause),
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
                 (fragmentainer {fragmentainer_index}, {} axis)",
                physical_axis_label(*axis)
            ),
            Self::Standalone => f.write_str("a standalone request"),
        }
    }
}

/// States what kind of failure occurred. The payload of a wrapper variant
/// carries the detail and is the `source()` of the failure, so it is not
/// repeated here.
impl<S: LayoutScalar, M> fmt::Display for LayoutErrorKindOf<S, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(_) => f.write_str("layout input is invalid"),
            Self::BlockFragmentation(_) => f.write_str("block fragmentation failed"),
            Self::MissingContext(_) => f.write_str("context that layout requires is missing"),
            Self::UnsupportedCapability(_) => {
                f.write_str("layout does not support a capability its input requires")
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
            Self::TextNumeric { value, .. } => {
                write!(f, "numeric operation on the value {value} failed")
            }
            Self::InternalInvariant(_) => f.write_str("an internal layout invariant was violated"),
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

/// `source()` returns the payload of the failure kind: the measurement error,
/// the Text error of a Text or resource-limit failure, the scroll geometry
/// cause, the reason of a numeric conversion failure, or the invalid input,
/// block fragmentation failure, missing context, unsupported capability or
/// internal invariant. A failure without a deeper cause has no source.
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
            LayoutErrorKindOf::InvalidInput(input) => Some(input),
            LayoutErrorKindOf::BlockFragmentation(failure) => Some(failure),
            LayoutErrorKindOf::MissingContext(context) => Some(context),
            LayoutErrorKindOf::UnsupportedCapability(capability) => Some(capability),
            LayoutErrorKindOf::Measurement(error) => Some(error),
            LayoutErrorKindOf::Text(error) | LayoutErrorKindOf::ResourceLimit(error) => Some(error),
            LayoutErrorKindOf::ScrollGeometry(error) => Some(error),
            LayoutErrorKindOf::TextNumeric { reason, .. } => Some(reason),
            LayoutErrorKindOf::InternalInvariant(invariant) => Some(invariant),
            LayoutErrorKindOf::Allocation
            | LayoutErrorKindOf::TextFeedbackCycle
            | LayoutErrorKindOf::InlineFeedbackCycle
            | LayoutErrorKindOf::TextSpacingBudget { .. } => None,
        }
    }
}

/// Words that name a physical axis in messages.
pub(crate) const fn physical_axis_label(axis: PhysicalAxis) -> &'static str {
    match axis {
        PhysicalAxis::Horizontal => "horizontal",
        PhysicalAxis::Vertical => "vertical",
    }
}

/// Words that name a logical axis in messages.
pub(crate) const fn logical_axis_label(axis: LogicalAxis) -> &'static str {
    match axis {
        LogicalAxis::Inline => "inline",
        LogicalAxis::Block => "block",
    }
}

/// Words that name a grid axis in messages.
pub(crate) const fn grid_axis_label(axis: GridAxisKind) -> &'static str {
    match axis {
        GridAxisKind::Column => "column",
        GridAxisKind::Row => "row",
    }
}

/// Words that name a writing mode in messages.
pub(crate) const fn writing_mode_label(mode: WritingMode) -> &'static str {
    match mode {
        WritingMode::HorizontalTb => "horizontal-tb",
        WritingMode::VerticalRl => "vertical-rl",
        WritingMode::VerticalLr => "vertical-lr",
        WritingMode::SidewaysRl => "sideways-rl",
        WritingMode::SidewaysLr => "sideways-lr",
    }
}

const fn sizing_property_label(property: SizingProperty) -> &'static str {
    match property {
        SizingProperty::Preferred => "preferred size",
        SizingProperty::Minimum => "minimum size",
        SizingProperty::Maximum => "maximum size",
        SizingProperty::FlexBasis => "flex basis",
    }
}

const fn sizing_algorithm_label(algorithm: SizingAlgorithm) -> &'static str {
    match algorithm {
        SizingAlgorithm::Leaf => "leaf",
        SizingAlgorithm::Block => "block",
        SizingAlgorithm::Flex => "flex",
        SizingAlgorithm::Grid => "grid",
        SizingAlgorithm::GridLanes => "grid lanes",
        SizingAlgorithm::Positioned => "positioned",
    }
}

const fn calc_size_basis_label(basis: CalcSizeBehaviorBasis) -> &'static str {
    match basis {
        CalcSizeBehaviorBasis::Auto => "auto",
        CalcSizeBehaviorBasis::None => "none",
        CalcSizeBehaviorBasis::Content => "content",
        CalcSizeBehaviorBasis::MinContent => "min-content",
        CalcSizeBehaviorBasis::MaxContent => "max-content",
        CalcSizeBehaviorBasis::Stretch => "stretch",
        CalcSizeBehaviorBasis::FitContent => "fit-content",
        CalcSizeBehaviorBasis::Contain => "contain",
    }
}

/// Words that name a physical side in messages.
pub(crate) const fn physical_side_label(side: PhysicalSide) -> &'static str {
    match side {
        PhysicalSide::Top => "top",
        PhysicalSide::Right => "right",
        PhysicalSide::Bottom => "bottom",
        PhysicalSide::Left => "left",
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "error_text_tests.rs"]
mod text_tests;
