use super::super::{formatted, roles};
use super::*;

pub(super) struct AcceptedCandidate {
    pub(super) candidate: EvaluatedCandidate,
    pub(super) role_start: usize,
    pub(super) box_use_start: usize,
    pub(super) zone_basis: ZoneBasis,
    pub(super) phrase_basis: Vec<PhraseBasis>,
}

#[derive(Clone, Copy)]
pub(super) struct SequenceLineSearch<'a> {
    pub(super) progress: &'a SequenceEntry,
    pub(super) accepted_prefix: &'a [FittedLine],
    pub(super) opportunities: &'a [Opportunity],
    pub(super) rejected_choices: &'a [CandidateKey],
    pub(super) goal: Option<LineQualityGoal>,
    pub(super) constrained: bool,
    #[cfg(test)]
    pub(super) scope: SequenceScope,
}

struct PrimaryLineChoice {
    choice: Option<EvaluatedCandidate>,
    fitting_none: bool,
    first_ceiling: Option<surgeist_text::PreparedOpportunity>,
    skipped_ordinary: Vec<ConditionalDisposition>,
    zone_basis: ZoneBasis,
}

impl<B, C, A, R, X> CandidateEnvironment<B, C, A, R, X>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset) -> Result<(), FitError>,
{
    pub(super) fn choose_sequence_line(
        &mut self,
        input: &FitInput<'_>,
        search: SequenceLineSearch<'_>,
    ) -> Result<Option<AcceptedCandidate>, FitError> {
        let SequenceLineSearch {
            progress,
            accepted_prefix,
            rejected_choices,
            goal,
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
            let Some(mut candidate) = choice else {
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
            let mut minimal_progress = None;
            if let Some(purpose) = purpose
                && let Some(ceiling) = first_ceiling
                && ceiling.view().boundary().view().participant_offset() > start
            {
                let frame = (purpose == ConditionalPurpose::Overflow)
                    .then(|| conditional_query_frame(input, &candidate))
                    .transpose()?;
                let recipe = frame.as_ref().map(|frame| ConditionalQueryRecipe {
                    frame: frame.clone(),
                    ceiling: ceiling.clone(),
                });
                let search = input.segment.conditional_search(
                    &start_boundary,
                    progress.incoming.as_ref(),
                    &ceiling,
                    purpose,
                    &skipped_ordinary,
                    frame.as_ref().map_or(
                        surgeist_text::ConditionalLineContext::Unframed {
                            inline_boxes: input.inline_boxes,
                        },
                        surgeist_text::ConditionalLineContext::Finalized,
                    ),
                )?;
                let query = EndpointContext {
                    accepted_prefix,
                    preceding: None,
                    start: &start_boundary,
                    line_box_context: progress.line_box_context,
                    incoming: progress.incoming.as_ref(),
                    position: &position,
                    indent_state: progress.indent_state,
                    hyphenation_state: progress.hyphenation_state,
                    rejected_choices,
                    conditional: true,
                    operation: CandidateOperation::Primary,
                    conditional_recipe: recipe.as_ref(),
                };
                let mut decisions = Vec::new();
                for portion in search.phrase_portions() {
                    let (decision, basis) =
                        self.measure_phrase(input, &query, &candidate, portion)?;
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
                                batch.opportunities()[from..to].iter().cloned().map(
                                    |opportunity| {
                                        let alternative = opportunity.alternative();
                                        LineEndpoint {
                                            boundary: opportunity.boundary(),
                                            termination: PreparedTermination::Conditional(
                                                opportunity,
                                            ),
                                            alternative,
                                            ending: LineEnd::Soft,
                                        }
                                    },
                                ),
                                &mut zone_basis,
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
            }
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
            rejected_choices,
            goal,
            #[cfg(test)]
            scope,
            ..
        } = search;
        let start = start_boundary.view().participant_offset();
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
            for (end, opportunity, kind, alternatives) in opportunities {
                #[cfg(test)]
                balance::observe_work(scope, true);
                if *end < start || (*end == start && *kind != PreparedBreakKind::ParagraphEnd) {
                    continue;
                }
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
                if let Some(candidate) = endpoint_choice {
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
                        |opportunity: &surgeist_text::PreparedOpportunity| {
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
