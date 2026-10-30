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
    placement: BoxPlacement,
}

#[derive(Clone, Debug, PartialEq)]
enum BoxPlacement {
    Boundary(BoundaryPlacement),
    ParagraphCompletion(Box<ParagraphCompletion>),
}

#[derive(Clone, Debug, PartialEq)]
struct BoundaryPlacement {
    incidence: Option<PreparedBoundaryIncidence>,
    native: NativeBoxCut,
    relations: Vec<(usize, CapturedScopeIncidence)>,
}

/// Completion owns the paragraph remainder, not a new spelling incidence.
/// The genuine force pin and its numerical receipt remain part of the proof.
#[derive(Clone, Debug, PartialEq)]
struct ParagraphCompletion {
    paragraph: surgeist_text::CapturedParagraph,
    termination: surgeist_text::PreparedOpportunity,
    pin: BoundaryPlacement,
    predecessor: CompletionPredecessor,
    relations: Vec<(usize, CapturedScopeIncidence)>,
}

#[derive(Clone, Debug)]
enum CompletionPredecessor {
    Finalized {
        line: surgeist_text::FinalizedLine,
        amounts: Vec<f64>,
        contributions: Vec<BoxContribution>,
    },
    Observed {
        recipe: surgeist_text::PreparedMeasurementFlow,
        observations: Vec<surgeist_text::FlowObservation>,
        contributions: Vec<BoxContribution>,
    },
}

impl PartialEq for CompletionPredecessor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Finalized {
                    line: a,
                    amounts: aa,
                    contributions: ac,
                },
                Self::Finalized {
                    line: b,
                    amounts: ba,
                    contributions: bc,
                },
            ) => {
                // Keep the capability, but compare its deterministic admitted
                // inputs, not the allocation identity of fresh finalization.
                a.selected_line().request() == b.selected_line().request()
                    && aa == ba
                    && ac == bc
                    && a.scope_allocations() == b.scope_allocations()
            }
            (
                Self::Observed {
                    recipe: a,
                    observations: ao,
                    contributions: ac,
                },
                Self::Observed {
                    recipe: b,
                    observations: bo,
                    contributions: bc,
                },
            ) => a == b && ao == bo && ac == bc,
            _ => false,
        }
    }
}

impl BoxFrontier {
    pub fn boundary(
        boundary: PreparedBoundary,
        incidence: Option<PreparedBoundaryIncidence>,
        native: NativeBoxCut,
        relations: Vec<(usize, CapturedScopeIncidence)>,
    ) -> Self {
        Self {
            boundary,
            placement: BoxPlacement::Boundary(BoundaryPlacement {
                incidence,
                native,
                relations,
            }),
        }
    }

    pub fn incidence(&self) -> Option<&PreparedBoundaryIncidence> {
        match &self.placement {
            BoxPlacement::Boundary(cut) => cut.incidence.as_ref(),
            BoxPlacement::ParagraphCompletion(_) => None,
        }
    }

    pub fn native(&self) -> NativeBoxCut {
        match &self.placement {
            BoxPlacement::Boundary(cut) => cut.native.clone(),
            // A paragraph horizon is not an original physical source seam.
            // An incident native encounter still needs its own side proof.
            BoxPlacement::ParagraphCompletion(_) => NativeBoxCut::Unavailable,
        }
    }

    pub fn relations(&self) -> &[(usize, CapturedScopeIncidence)] {
        match &self.placement {
            BoxPlacement::Boundary(cut) => &cut.relations,
            BoxPlacement::ParagraphCompletion(completion) => &completion.relations,
        }
    }

    pub fn relation(&self, row: usize) -> Option<CapturedScopeIncidence> {
        self.relations()
            .iter()
            .find(|(id, _)| *id == row)
            .map(|(_, relation)| *relation)
    }

    /// Shared by declared and undeclared actual boxes; neither can silently
    /// use the old force pin when the selected paragraph remainder owns them.
    pub fn scope_relation(
        &self,
        scope: &surgeist_text::CapturedScope,
    ) -> Result<Option<CapturedScopeIncidence>, FitError> {
        match &self.placement {
            BoxPlacement::Boundary(cut) => cut
                .incidence
                .as_ref()
                .map(|incidence| {
                    incidence
                        .boundary()
                        .prepared_text()
                        .incidence_scope_relation(incidence, scope)
                        .map_err(FitError::from)
                })
                .transpose(),
            BoxPlacement::ParagraphCompletion(completion) => {
                horizon_relation(&completion.paragraph, scope)
            }
        }
    }

    /// Only an authentic terminal Forced -> Paragraph transition can complete
    /// boxes at an unchanged Text boundary. All ordinary choices remain issued
    /// boundary placements.
    pub fn complete(
        incoming: &Self,
        choice: &BoxChoice<'_>,
        relations: Vec<(usize, CapturedScopeIncidence)>,
    ) -> Result<Option<Self>, FitError> {
        use surgeist_text::{LineEnd, PreparedBreakKind as K, PreparedTermination};
        if choice.ending != LineEnd::Paragraph
            || choice.start != choice.end
            || incoming.boundary != *choice.start
        {
            return Ok(None);
        }
        let BoxPlacement::Boundary(pin) = &incoming.placement else {
            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
        };
        let (termination, predecessor) = match &choice.authority {
            BoxAuthority::Selected { request, preceding } => {
                let PreparedTermination::Ordinary(termination) = &request.termination else {
                    return Ok(None);
                };
                let Some(preceding) = preceding else {
                    return Ok(None);
                };
                let selected = preceding.line.selected_line();
                let prior = selected.request();
                let PreparedTermination::Ordinary(force) = &prior.termination else {
                    return Ok(None);
                };
                if prior.context.end != LineEnd::Forced
                    || force.view().kind() != K::Forced
                    || force.view().boundary() != *choice.start
                    || prior.end != *choice.start
                    || prior.start.prepared_text() != choice.start.prepared_text()
                    || preceding.boxes.outgoing != *incoming
                    || request.start != *choice.start
                    || request.end != *choice.end
                    || request.context.end != choice.ending
                {
                    return Ok(None);
                }
                (
                    termination.clone(),
                    CompletionPredecessor::Finalized {
                        line: preceding.line.clone(),
                        amounts: preceding.amounts.to_vec(),
                        contributions: preceding.boxes.contributions.clone(),
                    },
                )
            }
            BoxAuthority::Observed {
                recipe,
                segment,
                observations,
                preceding,
            } => {
                let Some(termination) = segment.mandatory_end() else {
                    return Ok(None);
                };
                let Some(previous) = observations.last() else {
                    return Ok(None);
                };
                let Some(preceding) = preceding else {
                    return Ok(None);
                };
                let prior = previous.segment();
                let Some(force) = prior.mandatory_end() else {
                    return Ok(None);
                };
                // Observations are Text-issued capabilities. Authenticate this
                // exact recipe prefix; the existing full-flow validation still
                // runs after the terminal observation is appended.
                if recipe.segments().nth(observations.len()).as_ref() != Some(*segment)
                    || !recipe
                        .segments()
                        .zip(observations.iter())
                        .all(|(a, b)| a == b.segment())
                    || prior.end() != *choice.start
                    || force.view().kind() != K::Forced
                    || previous.request().context.end
                        != if prior.role() == surgeist_text::FlowSegmentRole::Target {
                            LineEnd::Soft
                        } else {
                            LineEnd::Forced
                        }
                    || preceding.outgoing != *incoming
                    || segment.start() != *choice.start
                    || segment.end() != *choice.end
                    || recipe.completion() != termination
                {
                    return Ok(None);
                }
                (
                    termination,
                    CompletionPredecessor::Observed {
                        recipe: (*recipe).clone(),
                        observations: observations.to_vec(),
                        contributions: preceding.contributions.clone(),
                    },
                )
            }
        };
        if termination.view().kind() != K::ParagraphEnd
            || termination.view().boundary() != *choice.end
        {
            return Ok(None);
        }
        let Some(paragraph) = choice.end.prepared_text().captured_paragraph() else {
            return Ok(None);
        };
        if choice.start.prepared_text() != choice.end.prepared_text() {
            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
        }
        Ok(Some(Self {
            boundary: choice.end.clone(),
            placement: BoxPlacement::ParagraphCompletion(Box::new(ParagraphCompletion {
                paragraph,
                termination,
                pin: pin.clone(),
                predecessor,
                relations,
            })),
        }))
    }
}

pub(super) fn horizon_relation(
    paragraph: &surgeist_text::CapturedParagraph,
    scope: &surgeist_text::CapturedScope,
) -> Result<Option<CapturedScopeIncidence>, FitError> {
    use std::cmp::Ordering;
    use surgeist_text::CapturedScopeIncidence as R;
    let horizon = paragraph.exit_position();
    if scope.domain() != paragraph.domain() || !scope.exit_position().same_capture(&horizon) {
        return Ok(None);
    }
    let relation = match scope.exit_position().partial_cmp(&horizon) {
        Some(Ordering::Less | Ordering::Equal) => R::After,
        Some(Ordering::Greater) => match scope.enter_position().partial_cmp(&horizon) {
            Some(Ordering::Less) => R::Inside,
            Some(Ordering::Equal | Ordering::Greater) => R::Before,
            None => return Err(crate::WrappingScopeError::UnsupportedIncidence.into()),
        },
        None => return Err(crate::WrappingScopeError::UnsupportedIncidence.into()),
    };
    Ok(Some(relation))
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
            .map(|allocation| allocation.original.clone())
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

#[derive(Clone, Copy)]
pub(crate) struct FinalizedBoxReceipt<'a> {
    pub line: &'a surgeist_text::FinalizedLine,
    pub amounts: &'a [f64],
    pub boxes: &'a BoxAssignment,
}

pub(crate) enum BoxAuthority<'a> {
    Selected {
        request: &'a surgeist_text::LineSelectionRequest,
        preceding: Option<FinalizedBoxReceipt<'a>>,
    },
    Observed {
        recipe: &'a surgeist_text::PreparedMeasurementFlow,
        segment: &'a surgeist_text::PreparedFlowSegment,
        observations: &'a [surgeist_text::FlowObservation],
        preceding: Option<&'a BoxAssignment>,
    },
}

pub(crate) struct BoxChoice<'a> {
    pub start: &'a PreparedBoundary,
    pub end: &'a PreparedBoundary,
    pub incoming: Option<BoxEntry<'a>>,
    pub controls: &'a [ControlAllocation],
    pub ending: surgeist_text::LineEnd,
    pub authority: BoxAuthority<'a>,
}

/// One projection owns both global incidence and authentic box-order proof.
/// It enumerates allocations before the existing fitter evaluates them.
pub(crate) trait BoxResolver {
    fn choices(&self, choice: BoxChoice<'_>) -> Result<Vec<BoxAssignment>, FitError>;

    fn classify(&self, assignment: &BoxAssignment) -> Result<AvoidClass, FitError>;
}

/// The actual numerical trial and the distinct original assignment that caused
/// it. Rejected trials can influence the selected fit, hyphen policy or frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoxTrialUse {
    pub disposition: super::fit::LineDisposition,
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
    pub prepared: surgeist_text::PreparedText,
    pub nodes: Vec<Node>,
    pub input: crate::ComputeInputOf<S>,
    pub available: super::fit::Available,
    pub trials: Vec<BoxTrialUse>,
}
