//! Candidate fitting over the actual immutable Text operations.

use surgeist_text::{
    Alignment, AtomicAllocation, ControlAllocation, FinalizedLine, HangingContext, LineContext,
    LineEnd, LineOrientation, LineSelectionRequest, LineStart, ParagraphBaseDirection,
    PreparedBreakKind, PreparedParticipant, PreparedText, SpacingAllocations, SpacingPolicy,
    TabContext,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Band {
    pub start: f64,
    pub end: f64,
    pub next_transition: Option<f64>,
}

/// Constraints from the two actual footprints, never their bounding union.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineBands {
    pub allocation: Band,
    pub metrics: Band,
}

impl LineBands {
    fn combined(self) -> Band {
        Band {
            start: self.allocation.start.max(self.metrics.start),
            end: self.allocation.end.min(self.metrics.end),
            next_transition: earliest(
                self.allocation.next_transition,
                self.metrics.next_transition,
            ),
        }
    }

    fn next_transition(self, logical_start: f64, placed_start: f64) -> Option<f64> {
        earliest(
            self.allocation
                .next_transition
                .filter(|next| next.is_finite() && *next > logical_start),
            self.metrics
                .next_transition
                .filter(|next| next.is_finite() && *next > placed_start),
        )
    }
}

fn earliest(first: Option<f64>, second: Option<f64>) -> Option<f64> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (first, second) => first.or(second),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Available {
    Definite(f64),
    Minimum,
    Maximum,
}

#[derive(Clone, Debug, PartialEq)]
struct CandidateBasis {
    request: LineSelectionRequest,
    logical_start: f64,
    placed_start: f64,
    bands: LineBands,
    height: f64,
    minimum_block: Option<f64>,
}

/// A proposed line position and genuine earlier float/clearance constraint.
pub(crate) struct BlockPosition {
    pub proposed: f64,
    pub minimum: Option<f64>,
}

/// Accepted logical allocation and any genuine following clearance movement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineFlow {
    pub start: f64,
    pub end: f64,
    pub next: f64,
}

pub(crate) struct FittedLine {
    pub finalized: FinalizedLine,
    pub inline_start: f64,
    pub placed_start: f64,
    pub available: f64,
    pub bands: LineBands,
    pub flow: LineFlow,
}

pub(crate) enum FitError {
    Text(surgeist_text::Error),
    FeedbackCycle,
    NonFinite(f64),
    Numeric(f64, crate::LayoutTextNumericError),
    InvalidGeometry,
    Dependency,
}

impl From<surgeist_text::Error> for FitError {
    fn from(error: surgeist_text::Error) -> Self {
        Self::Text(error)
    }
}

pub(crate) struct FitInput<'a> {
    pub prepared: &'a PreparedText,
    pub available: Available,
    pub base_direction: ParagraphBaseDirection,
    pub alignment: Alignment,
    pub orientation: LineOrientation,
    pub atomics: &'a [AtomicAllocation],
    pub controls: &'a [ControlAllocation],
    pub line_strut: Option<surgeist_text::LineStrut>,
    pub reusable: &'a [FinalizedLine],
}

pub(crate) struct LineAllocations {
    pub atomics: Vec<AtomicAllocation>,
    pub controls: Vec<ControlAllocation>,
    pub line_strut: Option<surgeist_text::LineStrut>,
}

/// Fit only preparation-issued opportunities. Every measured candidate executes
/// its exact retained finalizer; preliminary unit widths never establish fit.
pub(crate) fn fit(
    input: FitInput<'_>,
    mut band_source: impl FnMut(f64, f64) -> Result<Band, FitError>,
    mut canonical_block: impl FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    mut allocate: impl FnMut(&FinalizedLine, Band, f64) -> Result<LineAllocations, FitError>,
    mut clear_source: impl FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
) -> Result<Vec<FittedLine>, FitError> {
    let FitInput {
        prepared,
        available,
        base_direction,
        alignment,
        orientation,
        atomics,
        controls,
        line_strut,
        reusable,
    } = input;
    let participants: Vec<_> = prepared.participants().collect();
    let boundaries: Vec<_> = prepared
        .boundary_views()
        .map(|view| (view.participant_offset(), view.handle()))
        .collect();
    let opportunities: Vec<_> = prepared
        .opportunity_views()
        .map(|view| {
            let boundary = view.boundary();
            let end = boundaries
                .iter()
                .find(|(_, handle)| handle == &boundary)
                .expect("preparation-issued opportunity has an actual issued boundary")
                .0;
            let alternatives = view
                .alternatives()
                .map(|view| view.handle())
                .collect::<Vec<_>>();
            (end, view.handle(), view.kind(), alternatives)
        })
        .collect();
    let mut result = Vec::new();
    let mut start = 0;
    let mut logical_start = 0.0;
    let mut minimum_block: Option<f64> = None;
    let mut completed_end = false;
    while !completed_end {
        let start_boundary = boundaries
            .iter()
            .find(|(offset, _)| *offset == start)
            .expect("a consumed opportunity retains its mixed boundary")
            .1
            .clone();
        let mut height = 0.0;
        let mut placed_start = logical_start;
        let mut rejected = Vec::new();
        loop {
            let bands = query_bands(&mut band_source, logical_start, placed_start, height)?;
            let band = bands.combined();
            let width = finite((band.end - band.start).max(0.0))?;
            let selected_width = match available {
                Available::Definite(value) => Some(width.min(value)),
                Available::Minimum | Available::Maximum => None,
            };
            let mut choice = None;
            for (end, opportunity, kind, alternatives) in &opportunities {
                if *end < start || (*end == start && *kind != PreparedBreakKind::ParagraphEnd) {
                    continue;
                }
                let end_boundary = boundaries
                    .iter()
                    .find(|(offset, _)| offset == end)
                    .expect("actual opportunity boundary remains retained")
                    .1
                    .clone();
                let selected_participants = participants[start..*end].to_vec();
                let selected_atomics: Vec<_> = atomics.iter().filter(|allocation| {
                    selected_participants.iter().any(|participant| matches!(participant,
                        PreparedParticipant::Atomic(occurrence) if occurrence == &allocation.occurrence))
                }).cloned().collect();
                let selected_controls: Vec<_> = controls.iter().filter(|allocation| {
                    selected_participants.iter().any(|participant| matches!(participant,
                        PreparedParticipant::Control(occurrence) if occurrence == &allocation.occurrence))
                }).cloned().collect();
                let alternatives = if *kind == PreparedBreakKind::Discretionary {
                    alternatives.iter().cloned().map(Some).collect::<Vec<_>>()
                } else {
                    vec![None]
                };
                let mut endpoint_choice = None;
                for alternative in alternatives {
                    let mut request = LineSelectionRequest {
                        start: start_boundary.clone(),
                        end: end_boundary.clone(),
                        participants: selected_participants.clone(),
                        termination: opportunity.clone(),
                        alternative,
                        context: LineContext {
                            base_direction,
                            orientation,
                            start: if start == 0 {
                                LineStart::Paragraph
                            } else {
                                LineStart::Continuation
                            },
                            end: match kind {
                                PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => {
                                    LineEnd::Soft
                                }
                                PreparedBreakKind::Forced => LineEnd::Forced,
                                PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                            },
                            available_inline: selected_width,
                            indent: 0.0,
                            alignment,
                            tabs: TabContext::FontAdvance,
                            hanging: HangingContext::None,
                            spacing: SpacingPolicy::None,
                        },
                        atomics: selected_atomics.clone(),
                        controls: selected_controls.clone(),
                        line_strut,
                    };
                    let mut allocation_bases = Vec::new();
                    let finalized = loop {
                        let finalized = if let Some(line) = reusable
                            .iter()
                            .find(|line| line.selected_line().request() == &request)
                        {
                            line.clone()
                        } else {
                            prepared
                                .select(request.clone())?
                                .finalize(SpacingAllocations::None)?
                        };
                        let actual = allocate(&finalized, band, placed_start)?;
                        if actual.atomics == request.atomics
                            && actual.controls == request.controls
                            && actual.line_strut == request.line_strut
                        {
                            break finalized;
                        }
                        if allocation_bases.contains(&request) {
                            return Err(FitError::FeedbackCycle);
                        }
                        allocation_bases.push(request.clone());
                        request.atomics = actual.atomics;
                        request.controls = actual.controls;
                        request.line_strut = actual.line_strut;
                    };
                    let feedback = finalized.feedback();
                    let occupied = finite(feedback.occupied_end - feedback.occupied_start)?;
                    let fits = selected_width
                        .is_none_or(|width| feedback.advance <= width && occupied <= width);
                    if endpoint_choice.is_none() || fits {
                        endpoint_choice = Some((request, finalized, fits));
                    }
                    // Declaration order is native preference; subsequent alternatives
                    // are reached only when earlier actual finalized candidates fail.
                    if fits {
                        break;
                    }
                }
                let Some((request, finalized, fits)) = endpoint_choice else {
                    return Err(FitError::FeedbackCycle);
                };
                if choice.is_none() || fits || matches!(available, Available::Maximum) {
                    choice = Some((*end, *kind, request, finalized));
                }
                if matches!(available, Available::Minimum)
                    || !matches!(
                        kind,
                        PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                    )
                {
                    break;
                }
            }
            let Some((end, kind, request, finalized)) = choice else {
                // Missing/illegal complete coverage is the Text owner's error;
                // a valid preparation always offers a paragraph terminator.
                return Err(FitError::FeedbackCycle);
            };
            let feedback = finalized.feedback();
            let actual_height = finite(feedback.line_height)?;
            let actual_start = canonical_block(
                BlockPosition {
                    proposed: logical_start,
                    minimum: minimum_block,
                },
                feedback.baseline,
            )?;
            let actual_bands =
                query_bands(&mut band_source, logical_start, actual_start, actual_height)?;
            let basis = CandidateBasis {
                request: request.clone(),
                logical_start,
                placed_start,
                bands,
                height,
                minimum_block,
            };
            if actual_bands != bands || actual_start != placed_start {
                if rejected.contains(&basis) {
                    return Err(FitError::FeedbackCycle);
                }
                rejected.push(basis);
                height = actual_height;
                placed_start = actual_start;
                continue;
            }
            let occupied = finite(feedback.occupied_end - feedback.occupied_start)?;
            let float_fit_width = match available {
                Available::Minimum => Some(width),
                Available::Definite(_) | Available::Maximum => selected_width,
            };
            let fits =
                float_fit_width.is_none_or(|width| feedback.advance <= width && occupied <= width);
            // A nonzero shortened band can still admit no content from this
            // indivisible candidate. Consume actual float transitions before
            // allowing unavoidable overflow in a float-free band. Minimum uses
            // that real band only for float avoidance: Text's intrinsic request
            // stays unconstrained. Maximum retains its unwrapped side-by-side
            // contribution without imposing that minimum-width constraint.
            if !fits && let Some(next) = bands.next_transition(logical_start, placed_start) {
                logical_start = logical_start.max(next);
                minimum_block = Some(minimum_block.map_or(next, |minimum| minimum.max(next)));
                placed_start = logical_start;
                height = 0.0;
                continue;
            }
            let logical_end = finite(logical_start + actual_height)?;
            let cleared = clear_source(logical_end, &request)?
                .map(finite)
                .transpose()?
                .map_or(logical_end, |minimum| minimum.max(logical_end));
            if cleared > logical_end {
                minimum_block = Some(minimum_block.map_or(cleared, |minimum| minimum.max(cleared)));
            }
            result.push(FittedLine {
                finalized,
                inline_start: band.start,
                placed_start,
                available: selected_width.unwrap_or(feedback.advance),
                bands,
                flow: LineFlow {
                    start: logical_start,
                    end: logical_end,
                    next: cleared,
                },
            });
            logical_start = cleared;
            start = end;
            completed_end = kind == PreparedBreakKind::ParagraphEnd;
            break;
        }
    }
    Ok(result)
}

fn query_bands(
    source: &mut impl FnMut(f64, f64) -> Result<Band, FitError>,
    logical_start: f64,
    placed_start: f64,
    height: f64,
) -> Result<LineBands, FitError> {
    let allocation = source(logical_start, finite(logical_start + height)?)?;
    let metrics = if logical_start == placed_start {
        allocation
    } else {
        source(placed_start, finite(placed_start + height)?)?
    };
    for band in [allocation, metrics] {
        finite(band.start)?;
        finite(band.end)?;
    }
    Ok(LineBands {
        allocation,
        metrics,
    })
}

fn finite(value: f64) -> Result<f64, FitError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FitError::NonFinite(value))
    }
}
