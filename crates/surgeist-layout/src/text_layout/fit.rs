//! Candidate fitting over the actual immutable Text operations.

use crate::{TextAlign, TextAlignLast, TextJustification};
use surgeist_text::{
    Alignment, AtomicAllocation, ControlAllocation, FinalizedLine, HangingContext, LineContext,
    LineEnd, LineOrientation, LineSelectionRequest, LineStart, ParagraphBaseDirection,
    PreparedBreakKind, PreparedParticipant, PreparedText, SelectedLine, SpacingAllocation,
    SpacingAllocations, SpacingPolicy, SpacingSlotDisposition, TabContext,
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
    amounts: Vec<f64>,
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
    pub amounts: Vec<f64>,
    pub inline_start: f64,
    pub placed_start: f64,
    pub available: f64,
    pub bands: LineBands,
    pub flow: LineFlow,
}

#[derive(Clone)]
pub(crate) struct ReusableLine {
    pub finalized: FinalizedLine,
    pub amounts: Vec<f64>,
}

pub(crate) enum FitError {
    Text(surgeist_text::Error),
    FeedbackCycle,
    SpacingBudget { available: f64, advance: f64 },
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
    pub alignment: TextAlign,
    pub last_alignment: TextAlignLast,
    pub justification: TextJustification,
    pub indent: crate::text_indent::ResolvedTextIndent<f64>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub orientation: LineOrientation,
    pub atomics: &'a [AtomicAllocation],
    pub controls: &'a [ControlAllocation],
    pub line_strut: Option<surgeist_text::LineStrut>,
    pub reusable: &'a [ReusableLine],
}

pub(crate) struct LineAllocations {
    pub atomics: Vec<AtomicAllocation>,
    pub controls: Vec<ControlAllocation>,
    pub line_strut: Option<surgeist_text::LineStrut>,
}

struct FinalizedCandidate {
    request: LineSelectionRequest,
    finalized: FinalizedLine,
    amounts: Vec<f64>,
}

fn text_alignment(alignment: TextAlign) -> Alignment {
    match alignment {
        TextAlign::Start => Alignment::Start,
        TextAlign::End => Alignment::End,
        TextAlign::Left | TextAlign::LegacyLeft => Alignment::Left,
        TextAlign::Right | TextAlign::LegacyRight => Alignment::Right,
        TextAlign::Center | TextAlign::LegacyCenter => Alignment::Center,
        TextAlign::Justify => Alignment::Justify,
    }
}

fn natural_line(
    prepared: &PreparedText,
    reusable: &[ReusableLine],
    request: &LineSelectionRequest,
) -> Result<FinalizedLine, FitError> {
    if let Some(line) = reusable.iter().find(|line| {
        line.finalized.selected_line().request() == request
            && line.amounts.iter().all(|amount| *amount == 0.0)
    }) {
        return Ok(line.finalized.clone());
    }
    Ok(prepared
        .select(request.clone())?
        .finalize(SpacingAllocations::None)?)
}

fn finalize_candidate(
    prepared: &PreparedText,
    reusable: &[ReusableLine],
    mut request: LineSelectionRequest,
    normal: TextAlign,
    last: TextAlignLast,
    justification: TextJustification,
) -> Result<FinalizedCandidate, FitError> {
    let effective = normal.for_line(last, request.context.end != LineEnd::Soft);
    request.context.alignment = text_alignment(effective);
    let natural = natural_line(prepared, reusable, &request)?;
    let selected = natural.selected_line();
    let mut amounts = vec![0.0; selected.spacing_slots().len()];
    if effective == TextAlign::Justify {
        let retained: Vec<_> = natural
            .spacing_slots()
            .enumerate()
            .filter_map(|(index, row)| {
                matches!(row.disposition, SpacingSlotDisposition::Retained { .. }).then_some(index)
            })
            .collect();
        let expansion = request
            .context
            .available_inline
            .map(|width| {
                finite(width - request.context.indent).and_then(|capacity| {
                    finite(capacity - natural.feedback().advance).map(|extra| (capacity, extra))
                })
            })
            .transpose()?;
        if let Some((width, extra)) = expansion.filter(|(_, extra)| *extra >= 0.0)
            && (extra == 0.0
                || (justification == TextJustification::Spaces && !retained.is_empty()))
        {
            if extra > 0.0 {
                // RAW logical advance owns this target, independently of the
                // retained trailing/hanging or occupied-fit observations.
                let share = finite(extra / retained.len() as f64)?;
                let mut supplied = 0.0;
                for (ordinal, index) in retained.iter().enumerate() {
                    amounts[*index] = if ordinal + 1 == retained.len() {
                        finite(extra - supplied)?
                    } else {
                        share
                    };
                    supplied = finite(supplied + amounts[*index])?;
                }
                let finalized = if let Some(line) = reusable.iter().find(|line| {
                    line.finalized.selected_line().request() == &request && line.amounts == amounts
                }) {
                    line.finalized.clone()
                } else {
                    finalize_spacing(&selected, &amounts)?
                };
                let finalized =
                    constrain_spacing_budget(&selected, &retained, width, &mut amounts, finalized)?;
                return Ok(FinalizedCandidate {
                    request,
                    finalized,
                    amounts,
                });
            }
        } else {
            // Pinned CSS Text3 §6.4.3; a new alignment is a new actual request,
            // so no slot capabilities are transferred from the zero pass.
            request.context.alignment = text_alignment(normal.unexpandable(last));
            let finalized = natural_line(prepared, reusable, &request)?;
            amounts = vec![0.0; finalized.selected_line().spacing_slots().len()];
            return Ok(FinalizedCandidate {
                request,
                finalized,
                amounts,
            });
        }
    }
    Ok(FinalizedCandidate {
        request,
        finalized: natural,
        amounts,
    })
}

/// Every probe executes the complete table on this actual selection. Never
/// reconstruct Text's slot additions or its logical advance fold in Layout.
fn finalize_spacing(selected: &SelectedLine, amounts: &[f64]) -> Result<FinalizedLine, FitError> {
    Ok(selected.finalize(SpacingAllocations::Complete(
        selected
            .spacing_slots()
            .zip(amounts)
            .map(|(slot, &amount)| SpacingAllocation { slot, amount })
            .collect(),
    ))?)
}

fn constrain_spacing_budget(
    selected: &SelectedLine,
    retained: &[usize],
    width: f64,
    amounts: &mut [f64],
    mut finalized: FinalizedLine,
) -> Result<FinalizedLine, FitError> {
    if finalized.feedback().advance <= width {
        return Ok(finalized);
    }
    // The ideal sum of amounts does not capture rounding in each real slot's
    // natural+amount addition and the following Text-owned logical fold.
    if let Some(index) = retained
        .iter()
        .rev()
        .copied()
        .find(|&index| amounts[index] > 0.0)
    {
        let previous = amounts[index];
        let overshoot = finite(finalized.feedback().advance - width)?;
        let reduced = finite(previous - overshoot)?.max(0.0);
        amounts[index] = if reduced < previous {
            reduced
        } else {
            previous.next_down()
        };
        finalized = finalize_spacing(selected, amounts)?;
        if finalized.feedback().advance <= width {
            return Ok(finalized);
        }
    }

    // No search over assumed numeric admission: Text may refuse a nonzero
    // probe that vanishes in an actual slot or later logical addition. A still
    // over-budget completed correction is Layout's own truthful refusal.
    Err(FitError::SpacingBudget {
        available: width,
        advance: finalized.feedback().advance,
    })
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
        last_alignment,
        justification,
        indent,
        mut indent_state,
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
                            indent: indent.at(indent_state),
                            alignment: text_alignment(alignment.for_line(
                                last_alignment,
                                matches!(
                                    kind,
                                    PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
                                ),
                            )),
                            tabs: TabContext::FontAdvance,
                            hanging: HangingContext::None,
                            spacing: if justification == TextJustification::Spaces {
                                SpacingPolicy::Spaces
                            } else {
                                SpacingPolicy::None
                            },
                        },
                        atomics: selected_atomics.clone(),
                        controls: selected_controls.clone(),
                        line_strut,
                    };
                    let mut allocation_bases = Vec::new();
                    let (finalized, amounts) = loop {
                        let candidate = finalize_candidate(
                            prepared,
                            reusable,
                            request,
                            alignment,
                            last_alignment,
                            justification,
                        )?;
                        request = candidate.request;
                        let finalized = candidate.finalized;
                        let actual = allocate(&finalized, band, placed_start)?;
                        if actual.atomics == request.atomics
                            && actual.controls == request.controls
                            && actual.line_strut == request.line_strut
                        {
                            break (finalized, candidate.amounts);
                        }
                        let basis = (request.clone(), candidate.amounts);
                        if allocation_bases.contains(&basis) {
                            return Err(FitError::FeedbackCycle);
                        }
                        allocation_bases.push(basis);
                        request.atomics = actual.atomics;
                        request.controls = actual.controls;
                        request.line_strut = actual.line_strut;
                    };
                    let feedback = finalized.feedback();
                    let occupied = finite(feedback.occupied_end - feedback.occupied_start)?;
                    let fits = selected_width.is_none_or(|width| {
                        feedback.advance <= width - request.context.indent
                            && occupied <= width - request.context.indent
                    });
                    if endpoint_choice.is_none() || fits {
                        endpoint_choice = Some((request, finalized, amounts, fits));
                    }
                    // Declaration order is native preference; subsequent alternatives
                    // are reached only when earlier actual finalized candidates fail.
                    if fits {
                        break;
                    }
                }
                let Some((request, finalized, amounts, fits)) = endpoint_choice else {
                    return Err(FitError::FeedbackCycle);
                };
                if choice.is_none() || fits || matches!(available, Available::Maximum) {
                    choice = Some((*end, *kind, request, finalized, amounts));
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
            let Some((end, kind, request, finalized, amounts)) = choice else {
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
                amounts: amounts.clone(),
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
            let fits = float_fit_width.is_none_or(|width| {
                feedback.advance <= width - request.context.indent
                    && occupied <= width - request.context.indent
            });
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
            indent_state.commit(kind == PreparedBreakKind::Forced);
            result.push(FittedLine {
                finalized,
                amounts,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        text::{PreparedParagraph, ahem_style},
        text_fragment::fragment_for,
    };

    fn selected(style: surgeist_text::Style, source: &str, width: f64) -> SelectedLine {
        let paragraph = PreparedParagraph::plain(1, surgeist_text::Source::new(source), style);
        let original = fragment_for(&paragraph.prepared).finalized_line();
        let mut request = original.selected_line().request().clone();
        request.context.available_inline = Some(width);
        request.context.alignment = Alignment::Justify;
        request.context.spacing = SpacingPolicy::Spaces;
        paragraph.prepared.select(request).unwrap()
    }

    fn retained(line: &FinalizedLine) -> Vec<usize> {
        line.spacing_slots()
            .enumerate()
            .filter_map(|(index, row)| {
                matches!(row.disposition, SpacingSlotDisposition::Retained { .. }).then_some(index)
            })
            .collect()
    }

    #[test]
    fn native_spacing_budget_refuses_unresolved_actual_advance() {
        let selected = selected(ahem_style(10.0, 10.0), "X X X", 60.0);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(natural.feedback().advance, 50.0);
        let retained = retained(&natural);
        assert_eq!(retained, [0, 1]);
        // Real complete amounts produce151. The final1 cannot absorb the91
        // overshoot; the completed correction [100,0] still produces150.
        // Refusal preserves these actual facts instead of dropping more slots.
        let mut amounts = vec![100.0, 1.0];
        let proposed = finalize_spacing(&selected, &amounts)
            .unwrap_or_else(|_| panic!("real selected spacing finalizes"));
        assert_eq!(proposed.feedback().advance, 151.0);
        match constrain_spacing_budget(&selected, &retained, 60.0, &mut amounts, proposed) {
            Err(FitError::SpacingBudget { available, advance }) => {
                assert_eq!(available, 60.0);
                assert_eq!(advance, 150.0);
            }
            _ => panic!("unresolved actual advance must be a Layout spacing-budget refusal"),
        }
        assert_eq!(amounts, [100.0, 0.0]);
        let actual = finalize_spacing(&selected, &amounts)
            .unwrap_or_else(|_| panic!("actual corrected complete table finalizes"));
        assert_eq!(actual.feedback().advance, 150.0);
        assert_eq!(actual.selected_line(), selected);
    }

    #[test]
    fn native_spacing_budget_correction_preserves_a_signed_slot_base() {
        let mut style = ahem_style(10.0, 10.0);
        style.word_spacing = -15.0;
        let selected = selected(style, "X X", 25.0);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(natural.feedback().advance, 15.0);
        assert_eq!(natural.spacing_slots().next().unwrap().advance, -5.0);
        let mut amounts = vec![20.0];
        let proposed = finalize_spacing(&selected, &amounts)
            .unwrap_or_else(|_| panic!("real signed-base spacing finalizes"));
        assert_eq!(proposed.feedback().advance, 35.0);
        let actual =
            constrain_spacing_budget(&selected, &retained(&natural), 25.0, &mut amounts, proposed)
                .unwrap_or_else(|_| panic!("one actual overshoot correction fits"));
        assert_eq!(amounts, [10.0]);
        assert_eq!(actual.feedback().advance, 25.0);
        assert_eq!(actual.spacing_slots().next().unwrap().advance, 5.0);
        assert_eq!(actual.selected_line(), selected);
    }

    #[test]
    fn native_spacing_budget_keeps_actual_text_lost_contribution_refusal() {
        let selected = selected(ahem_style(10.0, 10.0), "X X", 40.0);
        let amount = 1.0628135541960295e-153;
        match finalize_spacing(&selected, &[amount]) {
            Err(FitError::Text(error)) => assert!(matches!(error.detail(),
                Some(surgeist_text::ErrorDetail::InvalidTextNumeric {
                    field:"selected spacing amount",phase:surgeist_text::TextNumericPhase::Finalization,
                    category:surgeist_text::TextNumericError::NonZeroLost,value,
                }) if *value==amount)),
            _ => panic!("actual Text numeric refusal must pass through unchanged"),
        }
    }
}
