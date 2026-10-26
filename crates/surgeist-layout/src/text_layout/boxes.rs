//! Original box placement is consumer state, separate from Text's glyph boundary.

use super::fit::FitError;
use crate::inline_wrapping::AvoidClass;
use surgeist_text::{
    CapturedScopeIncidence, ControlAllocation, LineBoxStrut, PreparedBoundary,
    PreparedBoundaryIncidence,
};

/// A single original placement, classified once against this immutable flow's
/// scope inventory. The capability retains its original issuing preparation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NativeBoxCut {
    /// Existing authentic source-prefix observation has no omitted box edge.
    Prefix,
    /// Real endpoint anchors bound possible native gaps, including equal bounds.
    Proven(super::capture::ComputedNativeCut),
    /// An omitted edge has no native side proof for an incident transparent node.
    Unavailable,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoxFrontier {
    pub boundary: PreparedBoundary,
    pub incidence: Option<PreparedBoundaryIncidence>,
    pub native: NativeBoxCut,
    pub relations: Vec<(usize, CapturedScopeIncidence)>,
}
impl BoxFrontier {
    pub fn relation(&self, row: usize) -> Option<CapturedScopeIncidence> {
        self.relations
            .iter()
            .find(|(id, _)| *id == row)
            .map(|(_, relation)| *relation)
    }
}

/// Scope indices are issued by the immutable WrappingContext, never authored
/// priorities. Retain attribution even when two allocations happen to be equal.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoxContribution {
    pub scope: usize,
    pub original: LineBoxStrut,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoxAssignment {
    pub incoming: BoxFrontier,
    pub outgoing: BoxFrontier,
    pub contributions: Vec<BoxContribution>,
}
impl BoxAssignment {
    pub fn struts(&self) -> Vec<LineBoxStrut> {
        self.contributions
            .iter()
            .map(|allocation| allocation.original)
            .collect()
    }
}

/// A detached measurement may replace a region whose earlier prefix was
/// genuinely phantom. Advancing Text progress does not consume those boxes.
#[derive(Clone, Copy)]
pub(crate) enum BoxEntry<'a> {
    ConsumedTo(&'a BoxFrontier),
    UnconsumedFrom(&'a BoxFrontier),
}
impl<'a> BoxEntry<'a> {
    pub fn frontier(self) -> &'a BoxFrontier {
        match self {
            Self::ConsumedTo(frontier) | Self::UnconsumedFrom(frontier) => frontier,
        }
    }
}

/// One projection owns both global incidence and authentic box-order proof.
/// It enumerates allocations before the existing fitter evaluates them.
pub(crate) trait BoxResolver {
    fn choices(
        &self,
        start: &PreparedBoundary,
        end: &PreparedBoundary,
        incoming: Option<BoxEntry<'_>>,
        controls: &[ControlAllocation],
        ending: surgeist_text::LineEnd,
    ) -> Result<Vec<BoxAssignment>, FitError>;

    fn classify(&self, assignment: &BoxAssignment) -> Result<AvoidClass, FitError>;
}

/// The actual numerical trial and the distinct original assignment that caused
/// it. Rejected trials can influence the selected fit, hyphen policy or frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoxTrialUse {
    pub assignment: BoxAssignment,
    pub request: surgeist_text::LineSelectionRequest,
    pub amounts: Vec<f64>,
    pub bands: super::fit::LineBands,
    pub encounters: Vec<super::fit::EncounterBasis>,
    pub placed_start: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoxRunUse<Node, S: crate::LayoutScalar> {
    pub container: Node,
    pub nodes: Vec<Node>,
    pub input: crate::ComputeInputOf<S>,
    pub available: super::fit::Available,
    pub trials: Vec<BoxTrialUse>,
}
