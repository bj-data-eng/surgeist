//! Bounded selection over rolling, genuinely accepted Forced-group entries.

use super::*;
use crate::inline_wrapping::balance::{GroupTrial, choose_group, variance_order};
use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BalanceBasis {
    groups: Vec<GroupBasis>,
    goals: Vec<Option<BalanceGoal>>,
    original_auto: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GroupBasis {
    pub(super) terminal: PreparedTermination,
    pub(super) boundary: PreparedBoundary,
    pub(super) ending: LineEnd,
    pub(super) count: usize,
    residuals: Vec<NaturalResidual>,
}

pub(super) struct GroupReference {
    pub(super) basis: GroupBasis,
    pub(super) window: Range<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LineProof {
    request: LineSelectionRequest,
    disposition: LineDisposition,
    boxes: Option<BoxAssignment>,
    box_uses: Vec<super::super::boxes::BoxTrialUse>,
    roles: Vec<RoleUse>,
    first: bool,
    line_box_end: crate::line_box::LineBoxContext,
    zone: ZoneBasis,
    phrases: Vec<PhraseBasis>,
    hyphenation: crate::hyphenation::LineHyphenationState,
    outgoing: Option<HyphenationContinuation>,
    encounters: Vec<EncounterBasis>,
    amounts: Vec<f64>,
    scopes: super::super::scope_alignment::ScopeBasis,
    natural: f64,
    inline: f64,
    placed: f64,
    available: f64,
    bands: LineBands,
    flow: LineFlow,
    feedback: surgeist_text::LineFeedback,
    fit: surgeist_text::InlineMeasure,
    margin_fit: surgeist_text::MarginEdgeFit,
    alignment: surgeist_text::InlineMeasure,
    metrics: surgeist_text::Rect,
}

impl LineProof {
    pub(super) fn of(line: &FittedLine) -> Self {
        Self {
            request: line.finalized.selected_line().request().clone(),
            disposition: line.disposition.clone(),
            boxes: line.boxes.clone(),
            box_uses: line.box_uses.clone(),
            roles: line.role_uses.clone(),
            first: line.first_formatted,
            line_box_end: line.line_box_end,
            zone: line.zone_basis.clone(),
            phrases: line.phrase_basis.clone(),
            hyphenation: line.hyphenation_end,
            outgoing: line.finalized.hyphenation_continuation(),
            encounters: line.encounters.clone(),
            amounts: line.amounts.clone(),
            scopes: line.scopes.clone(),
            natural: line.natural_advance,
            inline: line.inline_start,
            placed: line.placed_start,
            available: line.available,
            bands: line.bands,
            flow: line.flow,
            feedback: line.finalized.feedback(),
            fit: line.finalized.measurements().fit(),
            margin_fit: line.finalized.measurements().margin_edge_fit(),
            alignment: line.finalized.measurements().alignment(),
            metrics: line.finalized.metric_bounds(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GroupProof {
    pub(super) lines: Vec<LineProof>,
    pub(super) outgoing: SequenceEntry,
}

#[derive(Default)]
pub(super) struct Observations {
    roles: Vec<RoleUse>,
    boxes: Vec<super::super::boxes::BoxTrialUse>,
}

impl Observations {
    pub(super) fn role(&mut self, used: &RoleUse) {
        self.roles.push(used.clone());
    }
    pub(super) fn box_use(&mut self, used: &super::super::boxes::BoxTrialUse) {
        self.boxes.push(used.clone());
    }
    pub(super) fn record(&mut self, lines: &[FittedLine]) {
        // Repeated authentic observations are harmless. Global membership
        // scans would make unrelated earlier groups part of every local trial.
        self.roles
            .extend(lines.iter().flat_map(|line| line.role_uses.iter().cloned()));
        self.boxes
            .extend(lines.iter().flat_map(|line| line.box_uses.iter().cloned()));
    }

    pub(super) fn attach(self, lines: &mut [FittedLine]) {
        if let Some(last) = lines.last_mut() {
            last.role_uses.extend(self.roles);
            last.box_uses.extend(self.boxes);
        }
    }
}

fn residuals(lines: &[FittedLine]) -> Result<Vec<NaturalResidual>, FitError> {
    lines
        .iter()
        .filter(|line| line.disposition.is_formatted())
        .map(|line| {
            let request = line.finalized.selected_line().request().clone();
            NaturalResidual::from_components(
                request
                    .context
                    .available_inline
                    .ok_or(FitError::InvalidGeometry)?,
                request.context.indent,
                line.natural_advance,
            )
            .ok_or(FitError::InvalidGeometry)
        })
        .collect()
}

pub(super) fn references(
    lines: &[FittedLine],
    opportunities: &[Opportunity],
) -> Result<Vec<GroupReference>, FitError> {
    // Mandatory offers and executed hard endings share Text's issued order.
    // Retain the actual capability comparison, including distinct equal-cut
    // Forced and ParagraphEnd occurrences, rather than keying mutable handles.
    let mut terminals = opportunities
        .iter()
        .enumerate()
        .filter(|(_, (_, _, kind, _))| {
            matches!(
                kind,
                PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
            )
        });
    let mut groups = Vec::new();
    let mut first_line = 0;
    let mut first_opportunity = 0;
    for (index, line) in lines.iter().enumerate() {
        let selected = line.finalized.selected_line();
        let request = selected.request();
        if request.context.end == LineEnd::Soft {
            continue;
        }
        let (terminal_index, (_, offer, kind, _)) =
            terminals.next().ok_or(FitError::InvalidGeometry)?;
        let ending = match kind {
            PreparedBreakKind::Forced => LineEnd::Forced,
            PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
            _ => return Err(FitError::InvalidGeometry),
        };
        if request.termination != PreparedTermination::Ordinary(offer.clone())
            || request.end != offer.view().boundary()
            || request.context.end != ending
        {
            return Err(FitError::InvalidGeometry);
        }
        let end = terminal_index + 1;
        let residuals = residuals(&lines[first_line..=index])?;
        groups.push(GroupReference {
            basis: GroupBasis {
                terminal: request.termination.clone(),
                boundary: request.end.clone(),
                ending: request.context.end,
                count: residuals.len(),
                residuals,
            },
            window: first_opportunity..end,
        });
        first_line = index + 1;
        first_opportunity = end;
    }
    if first_line != lines.len()
        || terminals.next().is_some()
        || groups
            .last()
            .is_none_or(|group| group.basis.ending != LineEnd::Paragraph)
    {
        return Err(FitError::InvalidGeometry);
    }
    Ok(groups)
}

pub(super) fn authenticate(
    reference: &GroupReference,
    lines: &[FittedLine],
    outgoing: SequenceEntry,
) -> Result<Option<GroupTrial<GroupProof>>, FitError> {
    let Some(last) = lines.last() else {
        return Ok(None);
    };
    let selected = last.finalized.selected_line();
    let request = selected.request();
    if request.termination != reference.basis.terminal
        || request.end != reference.basis.boundary
        || request.context.end != reference.basis.ending
    {
        return Ok(None);
    }
    let residuals = residuals(lines)?;
    if residuals.len() != reference.basis.count {
        return Ok(None);
    }
    Ok(Some(GroupTrial {
        residuals,
        value: GroupProof {
            lines: lines.iter().map(LineProof::of).collect(),
            outgoing,
        },
    }))
}

impl<B, C, A, R, X, Accept, Clear> SequenceRunner<'_, B, C, A, R, X, Accept, Clear>
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
    X: FnMut(EncounterReset),
    Accept: FnMut() -> Result<(), FitError>,
    Clear: FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
{
    pub(super) fn group(
        &mut self,
        entry: &SequenceEntry,
        reference: &GroupReference,
        goal: Option<BalanceGoal>,
        result: &mut Vec<FittedLine>,
        observations: &mut Observations,
    ) -> Result<Option<GroupTrial<GroupProof>>, FitError> {
        let outcome = self.run(
            entry,
            SequenceScope {
                reset: EncounterReset::Group,
                stop_at_forced: true,
                max_formatted: Some(reference.basis.count),
                goal: goal.map(LineQualityGoal::Balance),
                goal_after: 0,
                goal_before: None,
                opportunity_start: reference.window.start,
                opportunity_end: reference.window.end,
            },
            result,
        )?;
        observations.record(&result[entry.prefix_len..]);
        let SequenceOutcome::Complete(outgoing) = outcome else {
            return Ok(None);
        };
        authenticate(reference, &result[entry.prefix_len..], outgoing)
    }

    fn replay_program(
        &mut self,
        initial: &SequenceEntry,
        references: &[GroupReference],
        goals: &[Option<BalanceGoal>],
        mut checkpoint: impl FnMut(),
        result: &mut Vec<FittedLine>,
        observations: &mut Observations,
    ) -> Result<bool, FitError> {
        (self.environment.reset)(EncounterReset::Sequence);
        result.clear();
        let mut entry = initial.clone();
        for (reference, goal) in references.iter().zip(goals) {
            checkpoint();
            let Some(trial) = self.group(&entry, reference, *goal, result, observations)? else {
                return Ok(false);
            };
            entry = trial.value.outgoing;
        }
        Ok(true)
    }
}

pub(super) fn fit<B, C, A, R, X, Accept, Clear>(
    runner: &mut SequenceRunner<'_, B, C, A, R, X, Accept, Clear>,
    initial: SequenceEntry,
    mut checkpoint: impl FnMut(),
) -> Result<Vec<FittedLine>, FitError>
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
    X: FnMut(EncounterReset),
    Accept: FnMut() -> Result<(), FitError>,
    Clear: FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
{
    let whole = SequenceScope {
        reset: EncounterReset::Sequence,
        stop_at_forced: false,
        max_formatted: None,
        goal: None,
        goal_after: 0,
        goal_before: None,
        opportunity_start: 0,
        opportunity_end: runner.opportunities.len(),
    };
    let mut result = Vec::new();
    if !matches!(
        runner.run(&initial, whole, &mut result)?,
        SequenceOutcome::Complete(_)
    ) {
        return Err(FitError::FeedbackCycle);
    }
    let references = references(&result, &runner.opportunities)?;
    let original: Vec<_> = result.iter().map(LineProof::of).collect();
    let mut observations = Observations::default();
    observations.record(&result);
    (runner.environment.reset)(EncounterReset::Sequence);
    result.clear();
    let mut entry = initial.clone();
    let mut goals = Vec::new();
    let mut improved = false;
    let mut admissible = true;
    for reference in &references {
        checkpoint();
        let auto = runner.group(&entry, reference, None, &mut result, &mut observations)?;
        let eligible = (2..=10).contains(&reference.basis.count);
        let winner = if eligible {
            choose_group(&reference.basis.residuals, auto, |goal| {
                runner.group(
                    &entry,
                    reference,
                    Some(goal),
                    &mut result,
                    &mut observations,
                )
            })?
            .map(|winner| (winner.goal, winner.trial))
        } else {
            auto.map(|trial| (None, trial))
        };
        let Some((goal, winner)) = winner else {
            admissible = false;
            break;
        };
        if !eligible {
            // No trial displaced this already-current actual Auto group.
            entry = winner.value.outgoing;
            goals.push(None);
            continue;
        }
        if variance_order(&winner.residuals, &reference.basis.residuals) == Some(Ordering::Less) {
            improved = true;
        }
        let Some(replayed) =
            runner.group(&entry, reference, goal, &mut result, &mut observations)?
        else {
            return Err(FitError::FeedbackCycle);
        };
        if replayed.value != winner.value || replayed.residuals != winner.residuals {
            return Err(FitError::FeedbackCycle);
        }
        entry = replayed.value.outgoing;
        goals.push(goal);
    }
    let chosen: Vec<_> = result.iter().map(LineProof::of).collect();
    let mut original_auto = !admissible || !improved;
    if original_auto {
        if !matches!(
            runner.run(&initial, whole, &mut result)?,
            SequenceOutcome::Complete(_)
        ) || result.iter().map(LineProof::of).ne(original)
        {
            return Err(FitError::FeedbackCycle);
        }
        goals.clear();
        goals.resize(references.len(), None);
    } else if !runner.replay_program(
        &initial,
        &references,
        &goals,
        &mut checkpoint,
        &mut result,
        &mut observations,
    )? {
        // This is a count/terminal-inadmissible heuristic program, not a new
        // unsupported input. The original actual Auto operation remains valid.
        original_auto = true;
        if !matches!(
            runner.run(&initial, whole, &mut result)?,
            SequenceOutcome::Complete(_)
        ) || result.iter().map(LineProof::of).ne(original)
        {
            return Err(FitError::FeedbackCycle);
        }
        goals.clear();
        goals.resize(references.len(), None);
    } else if result.iter().map(LineProof::of).ne(chosen) {
        return Err(FitError::FeedbackCycle);
    }
    let basis = Arc::new(SelectionProgram::Balance(BalanceBasis {
        groups: references.into_iter().map(|group| group.basis).collect(),
        goals,
        original_auto,
    }));
    for line in &mut result {
        line.program = Some(Arc::clone(&basis));
    }
    observations.attach(&mut result);
    Ok(result)
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
struct Work {
    runs: usize,
    opportunities: usize,
}

#[cfg(test)]
thread_local! {
    static GROUP_WORK: std::cell::RefCell<std::collections::BTreeMap<(usize, usize), Work>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

#[cfg(test)]
pub(super) fn observe_work(scope: SequenceScope, opportunity: bool) {
    if scope.stop_at_forced {
        GROUP_WORK.with_borrow_mut(|work| {
            let work = work
                .entry((scope.opportunity_start, scope.opportunity_end))
                .or_default();
            if opportunity {
                work.opportunities += 1;
            } else {
                work.runs += 1;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::text::{PreparedParagraph, ahem_style};

    fn fit_groups(
        groups: &[String],
        wrap_style: crate::TextWrapStyle,
    ) -> (PreparedParagraph, Vec<FittedLine>) {
        let mut style = ahem_style(10.0, 10.0);
        style.wrap = surgeist_text::Wrap::Word;
        style.white_space = surgeist_text::WhiteSpace::PreserveBreaks;
        style.hyphens = surgeist_text::Hyphens::None;
        let paragraph =
            PreparedParagraph::plain(1, surgeist_text::Source::new(groups.join("\n")), style);
        GROUP_WORK.with_borrow_mut(|work| work.clear());
        let scope_alignments = crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
            crate::text_layout::scope_alignment::ScopeAlignmentSnapshot::<f64>::admit::<()>(
                Vec::new(),
                &[],
                false,
            )
            .unwrap()
            .freeze(),
            crate::text_layout::fragmentation::InlineBoxFamilies::default(),
        );
        let lines = super::super::fit(
            FitInput {
                inline_boxes: None,
                groups: crate::group_alignment::Snapshot::default(),
                wrapping: None,
                formatted: &FormattedTailSnapshot::default(),
                line_box_context: crate::line_box::LineBoxContext::new(
                    crate::LineBoxDirectionPolicy::Normal,
                    crate::Direction::Ltr,
                ),
                prepared: &paragraph.prepared,
                segment: paragraph.prepared.source_segments().next().unwrap(),
                scope_alignments: &scope_alignments,
                available: Available::Definite(60.0),
                base_direction: ParagraphBaseDirection::LeftToRight,
                block_direction: ParagraphBaseDirection::LeftToRight,
                alignment: TextAlign::Start,
                wrap_style,
                last_alignment: TextAlignLast::Auto,
                justification: TextJustification::None,
                indent: crate::text_indent::ResolvedTextIndent::ZERO,
                indent_state: crate::text_indent::IndentLineState::default(),
                hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
                hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
                tail: TailContext::Unrestricted,
                orientation: LineOrientation::Horizontal,
                atomics: &[],
                controls: &[],
                line_strut: None,
                reusable: &[],
            },
            |_, _| {
                Ok(Band {
                    start: 0.0,
                    end: 60.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            |position, _| {
                Ok(position
                    .minimum
                    .map_or(position.proposed, |minimum| position.proposed.max(minimum)))
            },
            |_, _, _, struts| {
                Ok(LineAllocations {
                    line_box_struts: struts.to_vec(),
                    atomics: vec![],
                    controls: vec![],
                    line_strut: None,
                })
            },
            |_, _| Ok(None),
            EncounterCallbacks {
                replay: |_: &FinalizedLine, _: EncounterReplay| Ok(Vec::new()),
                accept: || Ok(()),
                reset: |_| {},
                checkpoint_group: || {},
            },
        )
        .unwrap_or_else(|_| panic!("real prepared forced groups fit"));
        (paragraph, lines)
    }

    #[test]
    fn real_forced_groups_bound_local_trials_and_never_rescan_unrelated_opportunities() {
        let short = "AAA BB C";
        let long = "A ".repeat(80) + "A";
        let mut groups = vec![short.to_owned(); 8];
        groups.push(long);
        groups.extend(vec![short.to_owned(); 8]);
        let (paragraph, lines) = fit_groups(&groups, crate::TextWrapStyle::Balance);
        let mut actual = Vec::new();
        let mut widths = Vec::new();
        for line in &lines {
            let selected = line.finalized.selected_line();
            assert_eq!(selected.request().end.prepared_text(), paragraph.prepared);
            if line.disposition.is_formatted() {
                widths.push(line.natural_advance);
            }
            if selected.request().context.end != LineEnd::Soft {
                actual.push(std::mem::take(&mut widths));
            }
        }
        assert_eq!(actual.len(), 17);
        for (index, group) in actual.iter().enumerate() {
            if index == 8 {
                assert_eq!(group, &vec![50.0; 27]);
            } else {
                assert_eq!(group, &[30.0, 40.0]);
            }
        }
        GROUP_WORK.with_borrow(|work| {
            assert_eq!(work.len(), 17);
            let (&long_window, long_work) = work.iter().max_by_key(|(window, _)| window.1 - window.0).unwrap();
            assert_eq!(long_work.runs, 2, "one local Auto plus one whole-program replay, independent of the16 short portfolios");
            assert!(long_work.opportunities <= 2 * 27 * (long_window.1 - long_window.0));
            for (&window, work) in work {
                if window != long_window {
                    assert!(work.runs <= 21, "<=20 local runs plus whole-program replay");
                    assert!(work.opportunities <= 21 * 3 * (window.1 - window.0), "only this group's actual opportunities are visited");
                }
            }
        });
    }

    #[test]
    fn paragraph_tail_trials_replay_only_the_final_real_group() {
        let mut groups = vec!["AAA BB C".to_owned(); 8];
        groups.push("A ".repeat(80) + "A");
        groups.extend(vec!["AAA BB C".to_owned(); 8]);
        let (paragraph, lines) = fit_groups(&groups, crate::TextWrapStyle::AvoidShortLastLine);
        let mut actual = Vec::new();
        let mut widths = Vec::new();
        for line in &lines {
            assert_eq!(line.finalized.prepared_text(), paragraph.prepared);
            if line.disposition.is_formatted() {
                widths.push(line.natural_advance);
            }
            if line.finalized.selected_line().request().context.end != LineEnd::Soft {
                actual.push(std::mem::take(&mut widths));
            }
            assert!(matches!(
                line.program.as_deref(),
                Some(SelectionProgram::Tail(_))
            ));
        }
        assert_eq!(actual.len(), 17);
        for (index, group) in actual.iter().enumerate() {
            if index == 8 {
                assert_eq!(group, &vec![50.0; 27]);
            } else if index == 16 {
                assert_eq!(group, &[30.0, 40.0]);
            } else {
                assert_eq!(group, &[60.0, 10.0]);
            }
        }
        GROUP_WORK.with_borrow(|work| {
            assert_eq!(work.len(), 17);
            let final_window = *work.keys().max_by_key(|window| window.0).unwrap();
            for (index, (&window, work)) in work.iter().enumerate() {
                if window == final_window {
                    assert!(
                        work.runs <= 19,
                        "seventeen goals and two winner validations"
                    );
                    assert!(work.opportunities <= 19 * 3 * (window.1 - window.0));
                } else {
                    assert_eq!(
                        work.runs, 2,
                        "ordinary rolling pass and final validation only"
                    );
                    let count = if index == 8 { 27 } else { 2 };
                    assert!(work.opportunities <= 2 * count * (window.1 - window.0));
                }
            }
        });
    }

    #[test]
    fn pretty_actual_group_work_does_not_replay_unrelated_prefixes_per_goal() {
        let mut groups = vec!["AAA BB C".to_owned(); 8];
        groups.push("A ".repeat(80) + "A");
        groups.extend(vec!["AAA BB C".to_owned(); 8]);
        let (paragraph, lines) = fit_groups(&groups, crate::TextWrapStyle::Pretty);
        assert!(
            lines
                .iter()
                .all(|line| line.finalized.prepared_text() == paragraph.prepared)
        );
        assert!(
            lines
                .iter()
                .all(|line| matches!(line.program.as_deref(), Some(SelectionProgram::Pretty(_))))
        );
        GROUP_WORK.with_borrow(|work| {
            assert_eq!(work.len(), 17);
            let final_window = *work.keys().max_by_key(|window| window.0).unwrap();
            for (index, (&window, actual)) in work.iter().enumerate() {
                // Ordinary reference, <=18 Body targets, restore/winner and
                // final validation. Only the last group also has <=17 Tail goals.
                let bound = if window == final_window { 43 } else { 23 };
                assert!(actual.runs <= bound);
                let count = if index == 8 { 27 } else { 2 };
                assert!(actual.opportunities <= bound * (count + 1) * 2 * (window.1 - window.0));
            }
        });
    }
}

#[cfg(test)]
#[path = "fit_balance_completion_tests.rs"]
pub(super) mod completion_tests;

#[cfg(test)]
#[path = "fit_balance_context_tests.rs"]
mod context_tests;
