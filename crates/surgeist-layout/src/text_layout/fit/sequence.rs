use super::super::{formatted, roles};
use super::*;
use crate::accounting::LayoutWork;

pub(super) struct AcceptedCandidate {
    candidate: EvaluatedCandidate,
    role_start: usize,
    box_use_start: usize,
    zone_basis: ZoneBasis,
    phrase_basis: Vec<PhraseBasis>,
}

#[derive(Clone, Copy)]
pub(super) struct SequenceLineSearch<'a> {
    pub(super) progress: &'a SequenceEntry,
    pub(super) accepted_prefix: &'a [FittedLine],
    pub(super) opportunities: &'a [Opportunity],
    pub(super) walk: OpportunityWalk,
    pub(super) rejected_choices: &'a [CandidateKey],
    pub(super) goal: Option<LineQualityGoal>,
    pub(super) constrained: bool,
    pub(super) scope: SequenceScope,
}

/// Facts about one run's opportunity window that decide how much of it a line search must visit.
#[derive(Clone, Copy)]
pub(super) struct OpportunityWalk {
    /// The opportunities are in nondecreasing boundary order, so a line search may begin at the
    /// first opportunity past its start instead of at the window's first opportunity.
    ordered: bool,
    /// The window and its context make fit monotone in the candidate end, apart from the
    /// advance and band checks made at the decision point in `choose_primary_line`.
    monotone: bool,
}

/// Every glyph of `line`, in run order, advances the pen by at least zero, exactly, and by at
/// least its raw provider advance, within `SLACK` for the f32 to f64 conversion. No run starts
/// before the previous run ends. The step of a run's last glyph is measured to the run advance,
/// which excludes any spacing trimmed at the line edge.
fn glyph_advances_hold(line: &FinalizedLine) -> bool {
    /// Relative slack of the comparison with the raw advance only; no sign test uses it.
    const SLACK: f64 = 1e-9;
    let holds = |step: f64, raw: f64| step >= 0.0 && step >= raw - SLACK * raw.abs().max(1.0);
    let mut previous_end: Option<f64> = None;
    for run in line.run_views() {
        let origin = run.origin().x;
        if previous_end.is_some_and(|end| origin < end) {
            return false;
        }
        let mut previous: Option<(f64, f64)> = None;
        for glyph in run.glyphs() {
            if let Some((pen, raw)) = previous
                && !holds(glyph.pen() - pen, raw)
            {
                return false;
            }
            previous = Some((glyph.pen(), f64::from(glyph.advance())));
        }
        if let Some((pen, raw)) = previous
            && !holds(run.advance() - pen, raw)
        {
            return false;
        }
        previous_end = Some(origin + run.advance());
    }
    true
}

impl OpportunityWalk {
    pub(super) fn of(input: &FitInput<'_>, opportunities: &[Opportunity]) -> Self {
        Self {
            ordered: opportunities.windows(2).all(|pair| pair[0].0 <= pair[1].0),
            monotone: matches!(
                input.wrap_style,
                crate::TextWrapStyle::Auto | crate::TextWrapStyle::Stable
            ) && input.wrapping.is_none()
                && input.inline_boxes.is_none()
                && input.atomics.is_empty()
                && input.controls.iter().all(|row| row.boundary.is_none())
                && matches!(input.orientation, LineOrientation::Horizontal)
                && input.available.operation() == crate::SizingOperation::Normal
                && crate::layout_math::normal_available_size(input.available).is_some()
                && opportunities.iter().all(|(_, _, kind, alternatives)| {
                    alternatives.is_empty() && *kind != PreparedBreakKind::Discretionary
                }),
        }
    }
}

struct PrimaryLineChoice {
    choice: Option<EvaluatedCandidate>,
    fitting_none: bool,
    first_ceiling: Option<surgeist_text::prepared::PreparedOpportunity>,
    skipped_ordinary: Vec<ConditionalDisposition>,
    zone_basis: ZoneBasis,
}

struct ConditionalLineSearch<'a> {
    line: SequenceLineSearch<'a>,
    position: &'a CandidatePosition,
    start_boundary: &'a PreparedBoundary,
    wrapping: Option<&'a WrappingClassifier<'a>>,
    purpose: ConditionalPurpose,
    ceiling: surgeist_text::prepared::PreparedOpportunity,
    skipped_ordinary: &'a [ConditionalDisposition],
}

struct ConditionalLineChoice {
    candidate: EvaluatedCandidate,
    minimal_progress: Option<EvaluatedCandidate>,
}

impl<B, C, A, R, X> CandidateEnvironment<B, C, A, R, X>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::allocation::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset) -> Result<(), FitError>,
{
    pub(super) fn choose_sequence_line(
        &mut self,
        input: &FitInput<'_>,
        search: SequenceLineSearch<'_>,
    ) -> Result<Option<AcceptedCandidate>, FitError> {
        input.meter.charge(LayoutWork::LineSearch, 1);
        let SequenceLineSearch {
            progress,
            constrained,
            ..
        } = search;
        let wrapping = input
            .wrapping
            .filter(|_| crate::layout_math::normal_available_size(input.available).is_some());
        let start = progress.boundary.view().participant_offset();
        let start_boundary = progress.boundary.clone();
        let mut position = CandidatePosition {
            phase: progress.phase,
            source_floor: progress.logical_start,
            source_minimum: progress.minimum_block,
            logical_start: progress.logical_start,
            minimum_block: progress.minimum_block,
            deferred_from: None,
        };
        let mut transitions = Vec::new();
        let role_start = self.roles.len();
        let box_use_start = self.box_uses.len();
        loop {
            let mut phrase_basis = Vec::new();
            let PrimaryLineChoice {
                choice,
                fitting_none,
                first_ceiling,
                skipped_ordinary,
                mut zone_basis,
            } = self.choose_primary_line(input, search, &position, &start_boundary, wrapping)?;
            // A rejected fitting nonhyphenating terminal branch does not
            // authorize overflow/emergency in an otherwise fitting word.
            let choice = choice.filter(|candidate| !constrained || candidate.fits || !fitting_none);
            let Some(candidate) = choice else {
                return Ok(None);
            };
            let purpose = match input.available.operation() {
                crate::SizingOperation::MinContent => Some(ConditionalPurpose::MinContent),
                crate::SizingOperation::Normal
                    if !candidate.fits
                        && crate::layout_math::normal_available_size(input.available).is_some() =>
                {
                    Some(ConditionalPurpose::Overflow)
                }
                crate::SizingOperation::Normal | crate::SizingOperation::MaxContent => None,
            };
            let ConditionalLineChoice {
                mut candidate,
                minimal_progress,
            } = if let Some(purpose) = purpose
                && let Some(ceiling) = first_ceiling
                && ceiling.view().boundary().view().participant_offset() > start
            {
                let Some(choice) = self.choose_conditional_line(
                    input,
                    ConditionalLineSearch {
                        line: search,
                        position: &position,
                        start_boundary: &start_boundary,
                        wrapping,
                        purpose,
                        ceiling,
                        skipped_ordinary: &skipped_ordinary,
                    },
                    candidate,
                    &mut zone_basis,
                    &mut phrase_basis,
                )?
                else {
                    return Ok(None);
                };
                choice
            } else {
                ConditionalLineChoice {
                    candidate,
                    minimal_progress: None,
                }
            };
            self.restore(input, &candidate)?;
            let basis = (
                candidate.request.clone(),
                candidate.boxes.clone(),
                candidate.amounts.clone(),
                candidate.encounters.clone(),
                candidate.bands,
                candidate.placed_start,
                candidate.frame.clone(),
                zone_basis.clone(),
            );
            if transitions.contains(&basis) {
                return Err(FitError::FeedbackCycle);
            }
            transitions.push(basis);
            if !candidate.fits
                && position.deferred_from.is_none()
                && candidate.encounters.iter().any(|event| event.preceding)
            {
                let deferred = (self.replay)(
                    &candidate.finalized,
                    EncounterReplay {
                        preceding: None,
                        boxes: candidate.boxes.clone(),
                        defer_preceding: true,
                        ..candidate.frame.replay(candidate.placed_start, None)
                    },
                )?;
                if deferred != candidate.encounters {
                    position.deferred_from = Some(candidate.request.clone());
                    continue;
                }
            }
            if !candidate.fits
                && let Some(progress) = minimal_progress
            {
                candidate = progress;
                self.restore(input, &candidate)?;
            }
            if !candidate.fits
                && let Some(next) = candidate
                    .bands
                    .next_transition(candidate.frame.logical_start, candidate.placed_start)
            {
                let next = candidate.frame.query_coordinate(next)?;
                position.logical_start = position.logical_start.max(next);
                position.minimum_block = Some(
                    position
                        .minimum_block
                        .map_or(next, |minimum| minimum.max(next)),
                );
                continue;
            }
            return Ok(Some(AcceptedCandidate {
                candidate,
                role_start,
                box_use_start,
                zone_basis,
                phrase_basis,
            }));
        }
    }

    /// Whether every glyph of the group that ends at the first mandatory break at or after `rest`
    /// advances the pen by at least its own raw advance, and by at least zero, so that no
    /// negative letter or word spacing applies anywhere inside the group. One trial of the whole
    /// remaining group answers for every line of the group, so it runs once per group.
    fn advances_never_shrink(
        &mut self,
        input: &FitInput<'_>,
        query: &EndpointContext<'_>,
        rest: &[Opportunity],
    ) -> Result<bool, FitError> {
        let start = query.start.view().participant_offset();
        // The verdicts are sorted by group end and disjoint in end, so the group of this line is
        // the first one that ends after the line start. Line searches mostly move forward, which
        // makes a new verdict an append; a search that backtracks inserts or replaces one.
        let slot = self
            .spacing_verdicts
            .partition_point(|(_, to, _)| *to <= start);
        if let Some((from, _, verdict)) = self.spacing_verdicts.get(slot)
            && *from <= start
        {
            return Ok(*verdict);
        }
        let Some((end, opportunity, kind, _)) = rest
            .iter()
            .find(|(_, _, kind, _)| *kind != PreparedBreakKind::Allowed)
        else {
            return Ok(false);
        };
        let mut zone = ZoneBasis {
            policy: input.hyphenate_limit_zone,
            references: Vec::new(),
        };
        let trial = self.choose_endpoint(
            input,
            query,
            std::iter::once(LineEndpoint {
                boundary: opportunity.view().boundary(),
                termination: PreparedTermination::Ordinary(opportunity.clone()),
                alternative: None,
                ending: match kind {
                    PreparedBreakKind::Forced => LineEnd::Forced,
                    PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                    PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => LineEnd::Soft,
                },
            }),
            &mut zone,
        )?;
        // A line stretched by justification can hide negative spacing: the stretch lifts the very
        // steps the check reads. A stretched group line is only possible where a shorter
        // candidate of the group overflowed although the whole group fits, that is, where the
        // flow is not monotone, so it keeps the walk. With non-negative spacing the group's flow
        // is at least that of its overflowing prefix, the group line overflows as well and is
        // never stretched.
        let verdict = trial.candidate.is_some_and(|candidate| {
            candidate.amounts.iter().all(|amount| *amount == 0.0)
                && glyph_advances_hold(&candidate.finalized)
        });
        match self.spacing_verdicts.get_mut(slot) {
            // A search that backtracked to an earlier line of the same group covers more of it.
            Some(entry) if entry.1 == *end => *entry = (start, *end, verdict),
            _ => self.spacing_verdicts.insert(slot, (start, *end, verdict)),
        }
        Ok(verdict)
    }

    fn choose_primary_line(
        &mut self,
        input: &FitInput<'_>,
        search: SequenceLineSearch<'_>,
        position: &CandidatePosition,
        start_boundary: &PreparedBoundary,
        wrapping: Option<&WrappingClassifier<'_>>,
    ) -> Result<PrimaryLineChoice, FitError> {
        let SequenceLineSearch {
            progress,
            accepted_prefix,
            opportunities,
            walk,
            rejected_choices,
            goal,
            scope,
            ..
        } = search;
        let start = start_boundary.view().participant_offset();
        // Opportunities before the line start are skipped below. With them in boundary order they
        // form a prefix, which is skipped by search instead of by visiting each one.
        let first = if walk.ordered && !full_walk_forced() {
            opportunities.partition_point(|(end, ..)| *end < start)
        } else {
            0
        };
        let mut choice: Option<EvaluatedCandidate>;
        let mut fitting_none;
        let mut first_ceiling;
        let mut skipped_ordinary = Vec::new();
        let mut zone_basis = ZoneBasis {
            policy: input.hyphenate_limit_zone,
            references: Vec::new(),
        };
        let mut comparison_basis: Option<ComparisonBasis> = None;
        loop {
            choice = None;
            fitting_none = false;
            first_ceiling = None;
            skipped_ordinary.clear();
            zone_basis.references.clear();
            let mut ends = FittingEnds::default();
            let mut terminal = None;
            let mut scoped = ScopedChoices {
                goal,
                ..ScopedChoices::default()
            };
            let visit_meter = scope.meter(&input.meter);
            let mut visit_work = visit_meter.tally(LayoutWork::OpportunityVisit);
            let mut candidate_work = input.meter.tally(LayoutWork::Candidate);
            for (index, (end, opportunity, kind, alternatives)) in
                opportunities.iter().enumerate().skip(first)
            {
                visit_work.one();
                if *end < start || (*end == start && *kind != PreparedBreakKind::ParagraphEnd) {
                    continue;
                }
                candidate_work.one();
                let variants = (*kind != PreparedBreakKind::Discretionary)
                    .then_some(None)
                    .into_iter()
                    .chain(alternatives.iter().cloned().map(Some));
                let query = EndpointContext {
                    accepted_prefix,
                    preceding: None,
                    line_box_context: progress.line_box_context,
                    start: start_boundary,
                    incoming: progress.incoming.as_ref(),
                    position,
                    indent_state: progress.indent_state,
                    hyphenation_state: progress.hyphenation_state,
                    rejected_choices,
                    conditional: false,
                    operation: CandidateOperation::Primary,
                    conditional_recipe: None,
                };
                let endpoint = self.choose_endpoint(
                    input,
                    &query,
                    variants.map(|alternative| LineEndpoint {
                        boundary: opportunity.view().boundary(),
                        termination: PreparedTermination::Ordinary(opportunity.clone()),
                        alternative,
                        ending: match kind {
                            PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => {
                                LineEnd::Soft
                            }
                            PreparedBreakKind::Forced => LineEnd::Forced,
                            PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                        },
                    }),
                    &mut zone_basis,
                )?;
                fitting_none |= endpoint.fitting_none;
                let EndpointChoice {
                    candidate: endpoint_choice,
                    dispositions,
                    formatted: ordinary_formatted,
                    ..
                } = endpoint;
                if wrapping.is_some()
                    && !matches!(
                        kind,
                        PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                    )
                {
                    terminal = Some((
                        opportunity.view().boundary(),
                        endpoint_choice
                            .as_ref()
                            .map(|candidate| candidate.observed()),
                    ));
                }
                let mut overflows_for_good = false;
                if let Some(candidate) = endpoint_choice {
                    overflows_for_good = !candidate.fits
                        && !candidate.natural_fits
                        && candidate.disposition.is_formatted()
                        && candidate
                            .bands
                            .next_transition(candidate.frame.logical_start, candidate.placed_start)
                            .is_none();
                    if wrapping.is_some() {
                        ends.observe(&candidate, goal.is_some());
                    }
                    if comparison_basis.is_some()
                        && candidate.fits
                        && candidate.disposition.is_formatted()
                        && (goal.is_some() || candidate.request.context.end == LineEnd::Soft)
                    {
                        scoped.consider(
                            wrapping.expect("comparison has a scope loan"),
                            candidate.clone(),
                            &input.meter,
                        );
                    }
                    if (choice.is_none()
                        || ((input.available.operation() != crate::SizingOperation::MinContent)
                            && (candidate.fits
                                || (input.available.operation()
                                    == crate::SizingOperation::MaxContent))))
                        && prefer_candidate(goal, &candidate, choice.as_ref())?
                    {
                        choice = Some(candidate);
                    }
                }
                if first_ceiling.is_none() {
                    if ordinary_formatted {
                        first_ceiling = Some(opportunity.clone());
                    } else {
                        skipped_ordinary.extend(dispositions);
                    }
                }
                if input.available.operation() == crate::SizingOperation::MinContent
                    && ordinary_formatted
                {
                    break;
                }
                // Once a fitting line is chosen and a later candidate overflows, the remaining
                // candidates cannot replace the chosen line when no longer candidate fits where
                // a shorter one does not. The walk stops there only when all of these hold:
                // - the window is plain: no hyphenation alternatives or discretionary breaks
                //   (they add zone and ceiling facts per candidate), no inline boxes, atomics or
                //   inline boundary allocations (their margins can be negative), no wrapping
                //   classifier and no quality goal (both compare every fitting candidate), and an
                //   Auto or Stable ordinary Normal sizing operation, which keeps the last fitting
                //   candidate (`OpportunityWalk::of`);
                // - this candidate is formatted, a fitting line is already chosen and the first
                //   ordinary ceiling is set, so the skipped candidates could not change the
                //   ceiling, the skipped-ordinary list or the fitting-none fact;
                // - the line band has no later transition, so a taller candidate or a candidate
                //   placed lower on the pixel grid sees the same band or a narrower one, because
                //   floats and taller lines only remove inline space, never add it;
                // - no negative letter or word spacing applies inside the group: every glyph
                //   advances the pen by at least its raw advance (`glyph_advances_hold`);
                // - the candidate's natural line, before any justification spacing, overflows.
                //
                // Why the natural line decides. A justified line is the natural line with
                // non-negative spacing allocated to reach the capacity, and under the ordinary
                // operation it is admitted by its natural line, so `fits` and `natural_fits`
                // agree for it. A line kept in its natural form after a failed allocation is
                // admitted by its own fit, and `natural_fits` still reads the line before that
                // fallback, so the stop requires both to overflow. When the natural line
                // overflows, allocated spacing only widens it (the extra spacing is
                // non-negative, else the line keeps its natural form), so `fits` is false as
                // well.
                //
                // Why the natural width cannot shrink. With no inline boxes or atomics the line is
                // measured with the Native authority, whose required inline size is the larger of
                // the signed flow advance and the span of the occupied cluster intervals. Every
                // cluster interval starts where the previous one ends, so the span is at least
                // the flow and equals the width from the first to the last included cluster. When
                // every cluster advances by at least zero that width cannot shrink as clusters are
                // added. A longer candidate keeps every cluster of a shorter one in the same
                // order and places the shorter candidate's trimmed edge, a collapsible trailing
                // space, hanging punctuation or the spacing after its last glyph, inside the
                // line, where it only adds width; with no negative spacing the trimmed edge adds
                // at least zero, so it cannot make the shorter candidate the wider one. Hence
                // every skipped candidate has a natural width no smaller than the overflowing
                // one, overflows naturally and, by the previous paragraph, does not fit.
                if overflows_for_good
                    && !full_walk_forced()
                    && walk.monotone
                    && goal.is_none()
                    && wrapping.is_none()
                    && self.observed.is_none()
                    && first_ceiling.is_some()
                    && choice.as_ref().is_some_and(|candidate| candidate.fits)
                    && self.advances_never_shrink(input, &query, &opportunities[index..])?
                {
                    break;
                }
                if !matches!(
                    kind,
                    PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                ) {
                    break;
                }
            }
            if wrapping.is_some() {
                let observed = ComparisonBasis {
                    position: position.clone(),
                    normal: choice.as_ref().map(|candidate| candidate.observed()),
                    terminal,
                    progress: None,
                    ceiling: first_ceiling.as_ref().map(
                        |opportunity: &surgeist_text::prepared::PreparedOpportunity| {
                            opportunity.view().boundary()
                        },
                    ),
                    fitting_none,
                    ends,
                };
                if let Some(prior) = &comparison_basis {
                    if prior != &observed {
                        return Err(FitError::FeedbackCycle);
                    }
                    if let Some(preferred) = scoped.preferred()? {
                        choice = Some(preferred);
                    }
                } else if observed.ends.multiple
                    && !choice.as_ref().is_some_and(|candidate| {
                        candidate.fits && candidate.request.context.end != LineEnd::Soft
                    })
                {
                    comparison_basis = Some(observed);
                    continue;
                }
            }
            break;
        }
        Ok(PrimaryLineChoice {
            choice,
            fitting_none,
            first_ceiling,
            skipped_ordinary,
            zone_basis,
        })
    }

    fn choose_conditional_line(
        &mut self,
        input: &FitInput<'_>,
        query: ConditionalLineSearch<'_>,
        mut candidate: EvaluatedCandidate,
        zone_basis: &mut ZoneBasis,
        phrase_basis: &mut Vec<PhraseBasis>,
    ) -> Result<Option<ConditionalLineChoice>, FitError> {
        let ConditionalLineSearch {
            line:
                SequenceLineSearch {
                    progress,
                    accepted_prefix,
                    rejected_choices,
                    goal,
                    ..
                },
            position,
            start_boundary,
            wrapping,
            purpose,
            ceiling,
            skipped_ordinary,
        } = query;
        let mut minimal_progress = None;
        let frame = (purpose == ConditionalPurpose::Overflow)
            .then(|| conditional_query_frame(input, &candidate))
            .transpose()?;
        let recipe = frame.as_ref().map(|frame| ConditionalQueryRecipe {
            frame: frame.clone(),
            ceiling: ceiling.clone(),
        });
        let search = input.segment.conditional_search(
            start_boundary,
            progress.incoming.as_ref(),
            &ceiling,
            purpose,
            skipped_ordinary,
            frame.as_ref().map_or(
                surgeist_text::conditional::ConditionalLineContext::Unframed {
                    inline_boxes: input.inline_boxes,
                },
                surgeist_text::conditional::ConditionalLineContext::Finalized,
            ),
        )?;
        let query = EndpointContext {
            accepted_prefix,
            preceding: None,
            start: start_boundary,
            line_box_context: progress.line_box_context,
            incoming: progress.incoming.as_ref(),
            position,
            indent_state: progress.indent_state,
            hyphenation_state: progress.hyphenation_state,
            rejected_choices,
            conditional: true,
            operation: CandidateOperation::Primary,
            conditional_recipe: recipe.as_ref(),
        };
        let mut decisions = Vec::new();
        for portion in search.phrase_portions() {
            let (decision, basis) = self.measure_phrase(input, &query, &candidate, portion)?;
            decisions.push(decision);
            phrase_basis.push(basis);
        }
        let mut batch = search.begin(&decisions)?;
        let mut restored_fallback = false;
        loop {
            let stage = batch.stage();
            let mut dispositions = Vec::new();
            let mut stage_choice: Option<EvaluatedCandidate>;
            let mut blocked_none;
            let mut stage_progress;
            let progress_already_present = minimal_progress.is_some();
            let zone_start = zone_basis.references.len();
            let mut comparison_basis: Option<ComparisonBasis> = None;
            loop {
                dispositions.clear();
                zone_basis.references.truncate(zone_start);
                stage_choice = None;
                blocked_none = false;
                stage_progress = None;
                let mut ends = FittingEnds::default();
                let mut scoped = ScopedChoices {
                    goal,
                    ..ScopedChoices::default()
                };
                let mut from = 0;
                while from < batch.opportunities().len() {
                    let boundary = batch.opportunities()[from].boundary();
                    let mut to = from + 1;
                    while to < batch.opportunities().len()
                        && batch.opportunities()[to].boundary() == boundary
                    {
                        to += 1;
                    }
                    let endpoint = self.choose_endpoint(
                        input,
                        &query,
                        batch.opportunities()[from..to]
                            .iter()
                            .cloned()
                            .map(|opportunity| {
                                let alternative = opportunity.alternative();
                                LineEndpoint {
                                    boundary: opportunity.boundary(),
                                    termination: PreparedTermination::Conditional(opportunity),
                                    alternative,
                                    ending: LineEnd::Soft,
                                }
                            }),
                        zone_basis,
                    )?;
                    blocked_none |= endpoint.fitting_none && endpoint.candidate.is_none();
                    dispositions.extend(endpoint.dispositions);
                    if let Some(trial) = endpoint.candidate {
                        if wrapping.is_some() {
                            ends.observe(&trial, goal.is_some());
                        }
                        if comparison_basis.is_some()
                            && trial.fits
                            && trial.disposition.is_formatted()
                        {
                            scoped.consider(
                                wrapping.expect("comparison has a scope loan"),
                                trial.clone(),
                                &input.meter,
                            );
                        }
                        if purpose == ConditionalPurpose::MinContent {
                            stage_choice = Some(trial);
                            break;
                        }
                        if stage == ConditionalStage::Arbitrary
                            && !trial.fits
                            && !progress_already_present
                            && stage_progress.is_none()
                        {
                            stage_progress = Some(trial);
                        } else if (stage_choice.is_none() || trial.fits)
                            && prefer_candidate(goal, &trial, stage_choice.as_ref())?
                        {
                            stage_choice = Some(trial);
                        }
                    }
                    from = to;
                }
                if wrapping.is_some() {
                    let observed = ComparisonBasis {
                        position: position.clone(),
                        normal: stage_choice.as_ref().map(|candidate| candidate.observed()),
                        terminal: None,
                        progress: stage_progress
                            .as_ref()
                            .map(|candidate| candidate.observed()),
                        ceiling: None,
                        fitting_none: blocked_none,
                        ends,
                    };
                    if let Some(prior) = &comparison_basis {
                        if prior != &observed {
                            return Err(FitError::FeedbackCycle);
                        }
                        if let Some(preferred) = scoped.preferred()? {
                            stage_choice = Some(preferred);
                        }
                    } else if observed.ends.multiple {
                        comparison_basis = Some(observed);
                        continue;
                    }
                }
                break;
            }
            if !progress_already_present {
                minimal_progress = stage_progress;
            }
            if blocked_none && stage_choice.as_ref().is_none_or(|choice| !choice.fits) {
                return Ok(None);
            }
            if let Some(choice) = stage_choice {
                if choice.fits || purpose == ConditionalPurpose::MinContent {
                    candidate = choice;
                    break;
                }
                if stage != ConditionalStage::Arbitrary && !restored_fallback {
                    candidate = choice;
                    restored_fallback = true;
                }
            }
            let Some(next) = batch.advance(&dispositions)? else {
                break;
            };
            batch = next;
        }
        Ok(Some(ConditionalLineChoice {
            candidate,
            minimal_progress,
        }))
    }

    pub(super) fn accept_line(
        &mut self,
        input: &FitInput<'_>,
        progress: &mut SequenceEntry,
        accepted: AcceptedCandidate,
        accept_encounters: &mut impl FnMut() -> Result<(), FitError>,
        clear_source: &mut impl FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
        result: &mut Vec<FittedLine>,
    ) -> Result<LineEnd, FitError> {
        let AcceptedCandidate {
            candidate,
            role_start,
            box_use_start,
            zone_basis,
            phrase_basis,
        } = accepted;
        // Deferred replay above may have been a no-op. Restore the exact
        // candidate once more before any clear/accept operation observes it.
        self.restore(input, &candidate)?;
        let logical_end = finite(
            candidate.frame.logical_start + candidate.disposition.height(&candidate.finalized),
        )?;
        let cleared = if candidate.disposition.is_formatted() {
            clear_source(logical_end, &candidate.request)?
                .map(finite)
                .transpose()?
                .map_or(logical_end, |minimum| minimum.max(logical_end))
        } else {
            logical_end
        };
        progress.minimum_block = candidate.frame.minimum_block;
        progress.phase = candidate.frame.phase;
        if cleared > logical_end {
            progress.minimum_block = Some(
                progress
                    .minimum_block
                    .map_or(cleared, |minimum| minimum.max(cleared)),
            );
        }
        let ending = candidate.request.context.end;
        progress.boundary = candidate.request.end.clone();
        let first_formatted = progress.indent_state.first_formatted();
        if candidate.disposition.is_formatted() {
            progress.indent_state.commit(ending == LineEnd::Forced);
            progress
                .line_box_context
                .commit(candidate.request.context.line_box_direction);
        }
        accept_encounters()?;
        if crate::layout_math::normal_available_size(input.available).is_some()
            && candidate.disposition.is_formatted()
        {
            progress
                .hyphenation_state
                .commit(candidate.request.alternative.is_some());
        }
        // Trials and hypothetical manual portions do not mutate this
        // state; only the accepted outgoing ending replaces it.
        progress.incoming = candidate.finalized.hyphenation_continuation();
        let available = candidate.finalized.metric_bounds().size.width;
        let mut role_copies = formatted::EntryCopies::default();
        let mut role_uses =
            role_copies.rows_ref(input.entry_funding, self.roles.len() - role_start)?;
        roles::visit(input.entry_funding, self.roles.len() - role_start)?;
        role_uses.extend(self.roles.drain(role_start..));
        let role_uses = roles::retained_rows(role_uses, role_copies, input.entry_funding)?;
        result.push(FittedLine {
            disposition: candidate.disposition,
            boxes: candidate.boxes,
            box_uses: self.box_uses.split_off(box_use_start),
            first_formatted,
            role_uses,
            line_box_end: progress.line_box_context,
            zone_basis,
            phrase_basis,
            hyphenation_end: progress.hyphenation_state,
            encounters: candidate.encounters,
            finalized: candidate.finalized,
            amounts: candidate.amounts,
            scopes: candidate.scopes,
            natural_advance: candidate.natural_advance,
            program: None,
            inline_start: candidate.bands.combined().start,
            placed_start: candidate.placed_start,
            available,
            bands: candidate.bands,
            flow: LineFlow {
                frame: candidate.frame,
                end: logical_end,
                next: cleared,
            },
        });
        progress.logical_start = cleared;
        Ok(ending)
    }
}
