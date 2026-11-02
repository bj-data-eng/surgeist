//! Capture-owned scope demands, prepared font contexts and selected frames.

pub(crate) mod accounting;
pub(crate) mod census;
#[cfg(test)]
mod lookup_work_tests;
mod preparation;
#[cfg(test)]
pub(crate) use preparation::visit_prepared_allocations;
#[cfg(test)]
mod work_tests;
pub(crate) use preparation::{
    ScopeMetricsData, cell_metric_index, prepare_cell_resources, prepare_cell_tabs,
    prepare_resources,
};

use std::sync::Arc;

use super::{AnalyzedData, PreparedData};
use crate::processing::contextual::alignment::{ScopeAlignmentBudget, operations};
use crate::{
    CapturedScope, FontBaseline, FontBaselineOutcome, FontData, FontSynthesis, Result,
    ScopeAlignmentBoundary, ScopeAlignmentError, ScopeAlignmentUsage, ScopeFontReference,
    SourceProcessingEvent, UprightMetricProvenance, UprightMetricReference,
};

pub(crate) fn refusal(reason: ScopeAlignmentError) -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::ScopeAlignmentRefused,
        "scope alignment refused",
    )
    .with_detail(crate::ErrorDetail::ScopeAlignment { reason })
}
/// Immutable capture policy, optionally reduced for one composed operation.
pub(crate) fn operation_limits(
    prepared: &PreparedData,
    allowance: Option<crate::ScopeAlignmentAllowance>,
) -> Result<Option<crate::ScopeAlignmentLimits>> {
    let Some(topology) = prepared.analyzed.paragraph.scope_topology.as_ref() else {
        return if allowance.is_some() {
            Err(refusal(ScopeAlignmentError::MissingCapture))
        } else {
            Ok(None)
        };
    };
    let root = CapturedScope {
        capture: Arc::clone(&topology.capture),
        index: topology.capture.domain_scopes[topology.domain],
    };
    let Some(limits) = root.scope_alignment_limits() else {
        return if allowance.is_some() {
            Err(refusal(ScopeAlignmentError::LimitsRequired))
        } else {
            Ok(None)
        };
    };
    let Some(allowance) = allowance else {
        return Ok(Some(limits));
    };
    for (kind, required, limit) in [
        (
            crate::ScopeAlignmentLimitKind::Records,
            allowance.records(),
            limits.records(),
        ),
        (
            crate::ScopeAlignmentLimitKind::PayloadBytes,
            allowance.payload_bytes(),
            limits.payload_bytes(),
        ),
    ] {
        if required > limit {
            return Err(refusal(ScopeAlignmentError::Limit {
                kind,
                required,
                limit,
            }));
        }
    }
    Ok(Some(crate::ScopeAlignmentLimits::new(
        allowance.records(),
        limits.font_queries(),
        limits.structural_work(),
        allowance.payload_bytes(),
    )))
}
pub(crate) const fn baseline_index(name: FontBaseline) -> usize {
    match name {
        FontBaseline::Alphabetic => 0,
        FontBaseline::Central => 1,
    }
}

/// Extraction-time association, retained in the actual stored glyph order.
/// Context does not invent source roots for generated/unqualified output.
#[derive(Clone, Copy, Debug)]
pub(crate) enum GlyphCustody {
    Unscoped,
    Input(usize),
    Context(usize),
    /// Actual invocation context; no qualified source membership is asserted.
    UnqualifiedContext(Option<usize>),
}

/// Actual shared-operand equality, expressed only in captured frame domains.
/// Unknown qualification is retained separately from positive action history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScopeRelation {
    pub(crate) first: Option<usize>,
    pub(crate) second: Option<usize>,
    pub(crate) unqualified: bool,
}

#[derive(Default)]
pub(crate) struct PreparedScopes {
    pub(crate) metrics: Vec<ScopeMetricsData>,
    pub(crate) cell_tabs: Vec<crate::tabs::cell::CellTabReference>,
    pub(crate) targets: Vec<usize>,
    pub(crate) atom_owners: Vec<Vec<usize>>,
    pub(crate) breaks: Vec<usize>,
    pub(crate) usage: Option<ScopeAlignmentUsage>,
    pub(crate) retention: Option<crate::ScopeAlignmentRetention>,
}

#[derive(Clone, Copy)]
pub(crate) struct CustodyInput<'a> {
    capture: &'a crate::processing::contextual::ScopeCapture,
    owners: &'a [Vec<usize>],
    targets: &'a [usize],
}
impl<'a> CustodyInput<'a> {
    pub(crate) fn input_owners(
        &self,
        atom: usize,
        ledger: &mut accounting::Ledger,
    ) -> Result<&'a [usize]> {
        accounting::work(ledger, 1)?;
        Ok(&self.owners[atom])
    }
    pub(crate) fn target(
        &self,
        owner: usize,
        ledger: &mut accounting::Ledger,
    ) -> Result<Option<usize>> {
        let mut best: Option<(usize, usize)> = None;
        for scope in self.targets {
            accounting::work(ledger, 1)?;
            let scope = *scope;
            if is_descendant(self.capture, scope, owner) {
                let enter = self.capture.scopes[scope].enter;
                // Iterator::max_by_key retains the last equal maximum.
                if best.is_none_or(|(_, previous)| enter >= previous) {
                    best = Some((scope, enter));
                }
            }
        }
        Ok(best.map(|(scope, _)| scope))
    }
    pub(crate) fn observes(
        &self,
        atoms: std::ops::Range<usize>,
        ledger: &mut accounting::Ledger,
    ) -> Result<bool> {
        for atom in atoms {
            let owners = self.input_owners(atom, ledger)?;
            for owner in owners {
                accounting::work(ledger, 1)?;
                let owner = *owner;
                for target in self.targets {
                    accounting::work(ledger, 1)?;
                    let target = *target;
                    if is_descendant(self.capture, target, owner) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
}

impl PreparedScopes {
    pub(crate) fn custody<'a>(&'a self, analyzed: &'a AnalyzedData) -> Option<CustodyInput<'a>> {
        if self.targets.is_empty() {
            return None;
        }
        Some(CustodyInput {
            capture: &analyzed.paragraph.scope_topology.as_ref()?.capture,
            owners: &self.atom_owners,
            targets: &self.targets,
        })
    }
}
impl SelectionPlan {
    pub(crate) fn custody<'a>(&'a self, prepared: &'a PreparedData) -> Option<CustodyInput<'a>> {
        if prepared.scope_alignment.targets.is_empty() {
            return None;
        }
        Some(CustodyInput {
            capture: &prepared.analyzed.paragraph.scope_topology.as_ref()?.capture,
            owners: &self.owners,
            targets: &prepared.scope_alignment.targets,
        })
    }
}

pub(crate) fn metric_index(
    prepared: &PreparedData,
    scope: &CapturedScope,
    mut budget: Option<&mut ScopeAlignmentBudget>,
) -> Result<usize> {
    let topology = prepared
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?;
    if !Arc::ptr_eq(&topology.capture, &scope.capture) {
        return Err(refusal(ScopeAlignmentError::ForeignCapture));
    }
    if scope.capture.scopes[scope.index].domain != topology.domain {
        return Err(refusal(ScopeAlignmentError::WrongDomain));
    }
    for (index, row) in prepared.scope_alignment.metrics.iter().enumerate() {
        if let Some(budget) = budget.as_deref_mut() {
            budget.work(1)?;
        }
        if row.scope == *scope {
            return Ok(index);
        }
    }
    Err(refusal(ScopeAlignmentError::MissingMetrics {
        scope: scope.index,
    }))
}

fn push<T>(budget: &mut ScopeAlignmentBudget, values: &mut Vec<T>, value: T) -> Result<()> {
    budget.add_records(1)?;
    budget.work(1)?;
    budget.reserve(values, 1)?;
    values.push(value);
    Ok(())
}

fn original_atom_owners(
    analyzed: &AnalyzedData,
    atom: usize,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Vec<usize>> {
    use crate::transform::relation::ComponentOwner;
    let topology = analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("active scope capture");
    let mut owners = Vec::new();
    let mut insert = |owner: Option<usize>, budget: &mut ScopeAlignmentBudget| -> Result<()> {
        budget.work(owners.len())?;
        if let Some(owner) = owner
            && !owners.contains(&owner)
        {
            push(budget, &mut owners, owner)?;
        }
        Ok(())
    };
    match analyzed.transform.component_for_atom(atom).owner {
        ComponentOwner::Original(row) => {
            for piece in &analyzed.transform.pieces[analyzed.transform.rows[row].pieces.clone()] {
                budget.work(1)?;
                insert(
                    topology.capture.events[piece
                        .origin
                        .captured_event()
                        .expect("captured transform origin retains its event")]
                    .before,
                    budget,
                )?;
            }
        }
        ComponentOwner::Wbr(control) => {
            budget.work(1)?;
            insert(
                topology.capture.events[topology.control_orders[control]].before,
                budget,
            )?
        }
        ComponentOwner::Virtual(index) => {
            budget.work(1)?;
            let separator = &analyzed.transform.virtuals[index];
            insert(
                Some(analyzed.transform.scopes[separator.scope].ordinal),
                budget,
            )?;
        }
    }
    Ok(owners)
}

/// Builds feature-only ownership and mandatory formatting cuts before provider
/// itemization. The canonical source event stream, never a glyph/source hull,
/// supplies each original contribution's actual scope.
pub(crate) fn plan(
    analyzed: &AnalyzedData,
) -> Result<(PreparedScopes, Option<ScopeAlignmentBudget>)> {
    let Some(topology) = &analyzed.paragraph.scope_topology else {
        return Ok((PreparedScopes::default(), None));
    };
    let Some(mut budget) = topology.capture.scope_alignment_budget()? else {
        return Ok((PreparedScopes::default(), None));
    };
    budget.retain_bytes(std::mem::size_of::<PreparedScopes>())?;
    let capture = &topology.capture;
    let mut result = PreparedScopes::default();
    budget.work(analyzed.projection.atoms.len())?;
    for atom in 0..analyzed.projection.atoms.len() {
        let owners = original_atom_owners(analyzed, atom, &mut budget)?;
        push(&mut budget, &mut result.atom_owners, owners)?;
    }
    budget.work(capture.scopes.len())?;
    for (scope, fact) in capture.scopes.iter().enumerate() {
        let Some(formatting) = capture
            .scope_alignment_fact_with_budget(scope, &mut budget)?
            .and_then(|row| row.alignment_boundary)
        else {
            continue;
        };
        if fact.domain != topology.domain {
            continue;
        }
        push(&mut budget, &mut result.targets, scope)?;
        if formatting != ScopeAlignmentBoundary::NonInitial {
            continue;
        }
        // Canonical mixed cuts identify the actual sides; atom ownership below
        // detects a scope edge inside one indivisible grapheme before shaping.
        for edge in [fact.enter, fact.exit] {
            let mut before = None;
            let mut after = None;
            for (atom, owners) in result.atom_owners.iter().enumerate() {
                budget.work(owners.len() + 1)?;
                let mut inside = false;
                let mut outside = false;
                for &owner in owners {
                    let own = &capture.scopes[owner];
                    let enclosed = fact.enter <= own.enter && own.exit <= fact.exit;
                    inside |= enclosed;
                    outside |= !enclosed;
                }
                if inside && outside {
                    return Err(refusal(ScopeAlignmentError::UnpartitionableUnit { scope }));
                }
                if inside {
                    if edge == fact.enter {
                        after = Some(atom);
                        break;
                    }
                    before = Some(atom + 1);
                }
            }
            let at = if edge == fact.enter { after } else { before };
            if let Some(at) = at {
                budget.work(analyzed.analysis.graphemes.len() + result.breaks.len())?;
                if analyzed
                    .analysis
                    .graphemes
                    .iter()
                    .any(|g| g.start < at && at < g.end)
                {
                    return Err(refusal(ScopeAlignmentError::UnpartitionableUnit { scope }));
                }
                if !result.breaks.contains(&at) {
                    push(&mut budget, &mut result.breaks, at)?;
                }
            }
        }
    }
    operations::order_by(&mut result.breaks, Some(&mut budget), usize::cmp)?;
    result.usage = Some(budget.usage());
    Ok((result, Some(budget)))
}

pub(crate) struct SelectedScopeData {
    pub(crate) scope: CapturedScope,
    pub(crate) metrics: usize,
    pub(crate) parent_metrics: Option<usize>,
    pub(crate) parent: Option<usize>,
    pub(crate) direct: ContributionChain,
}
#[derive(Default)]
pub(crate) struct ContributionChain {
    pub(crate) first: Option<usize>,
    last: Option<usize>,
}
pub(crate) struct MetricContribution {
    /// Independently resolved aligned-component frame, not a source relabeling.
    pub(crate) owner: Option<usize>,
    pub(crate) metrics: crate::ScopeMetricContribution,
    pub(crate) next_direct: Option<usize>,
}
#[derive(Default)]
pub(crate) struct SelectedScopes {
    /// Exact closed chronological proof owners, never this future product.
    pub(crate) proof_frames: Vec<crate::protocol::FinalizedLine>,
    pub(crate) rows: Vec<SelectedScopeData>,
    pub(crate) contributions: Vec<MetricContribution>,
    pub(crate) unscoped: ContributionChain,
    pub(crate) cluster_owners: Vec<Option<usize>>,
    pub(crate) glyph_owners: Vec<Option<usize>>,
    pub(crate) unit_owners: Vec<Option<usize>>,
    pub(crate) strut_owners: Vec<Option<usize>>,
    pub(crate) shared: Vec<(Option<usize>, Option<usize>)>,
    pub(crate) uncertain: Vec<(Option<usize>, Option<usize>)>,
    pub(crate) usage: Option<ScopeAlignmentUsage>,
    pub(crate) retention: Option<crate::ScopeAlignmentRetention>,
}
impl SelectedScopes {
    pub(crate) fn contribution_chain(
        &self,
        mut next: Option<usize>,
    ) -> impl Iterator<Item = crate::ScopeMetricContribution> + '_ {
        std::iter::from_fn(move || {
            let row = &self.contributions[next?];
            next = row.next_direct;
            Some(row.metrics)
        })
    }

    fn push_contribution(
        &mut self,
        owner: Option<usize>,
        metrics: crate::ScopeMetricContribution,
        budget: &mut ScopeAlignmentBudget,
    ) -> Result<()> {
        budget.work(1)?;
        let chain = match owner {
            Some(index) => &mut self.rows[index].direct,
            None => &mut self.unscoped,
        };
        let previous = chain.last;
        if previous.is_some() {
            budget.work(1)?;
        }
        let index = self.contributions.len();
        push(
            budget,
            &mut self.contributions,
            MetricContribution {
                owner,
                metrics,
                next_direct: None,
            },
        )?;
        // Every link points forward to the newly admitted canonical row. Both
        // direct chains and the descendant view preserve native append order.
        if let Some(previous) = previous {
            self.contributions[previous].next_direct = Some(index);
        } else {
            chain.first = Some(index);
        }
        chain.last = Some(index);
        Ok(())
    }

    pub(crate) fn contains(&self, ancestor: usize, owner: Option<usize>) -> bool {
        let Some(owner) = owner else { return false };
        let scope = &self.rows[ancestor].scope;
        let own = &self.rows[owner].scope;
        let a = &scope.capture.scopes[scope.index];
        let b = &own.capture.scopes[own.index];
        a.enter <= b.enter && b.exit <= a.exit
    }
}

pub(crate) fn shares_operands(a: &crate::SelectedScope, b: &crate::SelectedScope) -> Result<bool> {
    if !Arc::ptr_eq(&a.selected, &b.selected) {
        return Err(refusal(ScopeAlignmentError::ForeignSelectedScope));
    }
    let rows = &a.selected.scopes;
    if rows.uncertain.iter().any(|&(x, y)| {
        (x == Some(a.index) && y == Some(b.index)) || (x == Some(b.index) && y == Some(a.index))
    }) {
        return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
            scope: rows.rows[a.index].scope.index,
        }));
    }
    Ok(a.index == b.index
        || rows.contains(a.index, Some(b.index))
        || rows.contains(b.index, Some(a.index))
        || rows.shared.iter().any(|&(x, y)| {
            (x == Some(a.index) && y == Some(b.index)) || (x == Some(b.index) && y == Some(a.index))
        }))
}

fn selected_frame(
    rows: &[SelectedScopeData],
    owner: usize,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Option<usize>> {
    let mut best: Option<(usize, usize)> = None;
    for (index, row) in rows.iter().enumerate() {
        budget.work(1)?;
        if is_descendant(&row.scope.capture, row.scope.index, owner) {
            let enter = row.scope.capture.scopes[row.scope.index].enter;
            if best.is_none_or(|(_, previous)| enter >= previous) {
                best = Some((index, enter));
            }
        }
    }
    Ok(best.map(|(index, _)| index))
}
fn append_owner_frame(
    owner: usize,
    rows: &[SelectedScopeData],
    frames: &mut Vec<Option<usize>>,
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    let frame = selected_frame(rows, owner, budget)?;
    budget.work(frames.len())?;
    if !frames.contains(&frame) {
        push(budget, frames, frame)?;
    }
    Ok(())
}
fn finish_owner_group(
    frames: Vec<Option<usize>>,
    pairs: &mut Vec<(Option<usize>, Option<usize>)>,
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    for &first in &frames {
        for &second in &frames {
            budget.work(pairs.len() + 1)?;
            if first < second && !pairs.contains(&(first, second)) {
                push(budget, pairs, (first, second))?;
            }
        }
    }
    let bytes = capacity_bytes(&frames)?;
    drop(frames);
    budget.release_bytes(bytes)?;
    Ok(())
}
fn retain_scope_group(
    atoms: impl IntoIterator<Item = usize>,
    owners: &[Vec<usize>],
    rows: &[SelectedScopeData],
    pairs: &mut Vec<(Option<usize>, Option<usize>)>,
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    let mut frames = Vec::new();
    for atom in atoms {
        budget.work(1)?;
        for owner in &owners[atom] {
            budget.work(1)?;
            append_owner_frame(*owner, rows, &mut frames, budget)?;
        }
    }
    finish_owner_group(frames, pairs, budget)
}
fn retain_owner_group(
    owners: &[usize],
    rows: &[SelectedScopeData],
    pairs: &mut Vec<(Option<usize>, Option<usize>)>,
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    let mut frames = Vec::new();
    for owner in owners {
        budget.work(1)?;
        append_owner_frame(*owner, rows, &mut frames, budget)?;
    }
    finish_owner_group(frames, pairs, budget)
}

/// A text container is proved by actual selected source membership, not the
/// common ancestor that a transformed Part may retain for contextual lookup.
fn cell_container(
    analyzed: &AnalyzedData,
    binding: usize,
    budget: &mut ScopeAlignmentBudget,
) -> Result<usize> {
    let topology = analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?;
    let capture = &topology.capture;
    let output = analyzed.combination.as_ref().expect("actual source cell");
    budget.work(2)?;
    let binding = &output.bindings[binding];
    let group = &output.plan.groups[binding.group];
    let scope = group.scope;
    budget.work(1)?;
    let fact = &capture.scopes[scope];
    if fact.domain != topology.domain {
        return Err(refusal(ScopeAlignmentError::WrongDomain));
    }
    if fact.definition.is_some() {
        budget.work(1)?;
        if capture.domain_scopes[topology.domain] != scope {
            return Err(refusal(ScopeAlignmentError::WrongDomain));
        }
    }
    for part in &output.plan.parts[group.parts.clone()] {
        budget.work(1)?;
        if part.scope != scope {
            return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                scope,
            }));
        }
        for piece in &part.pieces {
            budget.work(1)?;
            let order = piece
                .origin
                .captured_event()
                .ok_or_else(|| refusal(ScopeAlignmentError::UnrepresentedOwnership { scope }))?;
            budget.work(1)?;
            if capture.events[order].before != Some(scope) {
                return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                    scope,
                }));
            }
        }
    }
    Ok(scope)
}

/// One indivisible cell retains one actual frame even without glyph relations.
/// Nonpainting computed/control anchors use their canonical cause/owner facets;
/// an empty origin alone does not activate an otherwise dormant target.
fn visit_cell_frame_owners(
    analyzed: &AnalyzedData,
    binding: usize,
    budget: &mut ScopeAlignmentBudget,
    mut emit: impl FnMut(usize, &mut ScopeAlignmentBudget) -> Result<()>,
) -> Result<()> {
    let scope = cell_container(analyzed, binding, budget)?;
    emit(scope, budget)?;
    let topology = analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("cell capture");
    let capture = &topology.capture;
    let output = analyzed.combination.as_ref().expect("actual source cell");
    budget.work(1)?;
    let binding = &output.bindings[binding];
    for event in &analyzed.transform.events[binding.events.clone()] {
        budget.work(1)?;
        let crate::transform::relation::RelationEvent::Anchor { order } = *event else {
            continue;
        };
        budget.work(1)?;
        use crate::protocol::paragraph::SemanticAnchorItem;
        let owner = match analyzed.paragraph.anchor_order[order].item {
            SemanticAnchorItem::ComputedLineBoundary(index) => {
                visit_computed_frame_owners(analyzed, index, budget, &mut emit)?;
                None
            }
            SemanticAnchorItem::Control(index) => {
                control_frame_owner(capture, topology, index, budget)?
            }
            SemanticAnchorItem::Atomic(index) => {
                budget.work(1)?;
                capture.events[capture.atomic_orders[index]].before
            }
            SemanticAnchorItem::EmptyOrigin(_) => None,
        };
        if let Some(owner) = owner {
            emit(owner, budget)?;
        }
    }
    Ok(())
}

fn selected_unit_index(
    presentation: &crate::protocol::prepared::SelectedPresentation,
    kind: crate::protocol::prepared::PresentationUnitKind,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Option<usize>> {
    for (index, unit) in presentation.units.iter().enumerate() {
        budget.work(1)?;
        if unit.kind == kind {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn computed_frame_owner_at(analyzed: &AnalyzedData, cut: usize) -> Result<usize> {
    let topology = analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("admitted computed capture");
    let capture = &topology.capture;
    let owner = if let Some(event) = capture.events.get(cut) {
        event.before
    } else {
        // Contextual admission requires the complete event stack to be empty
        // at global EOF. There is no exterior scope frame there; do not invent
        // an `after` snapshot or reuse the final exited child's frame.
        None
    };
    let owner = owner.ok_or_else(|| {
        refusal(ScopeAlignmentError::UnrepresentedOwnership {
            scope: capture.domain_scopes[topology.domain],
        })
    })?;
    if capture.scopes[owner].domain != topology.domain {
        return Err(refusal(ScopeAlignmentError::WrongDomain));
    }
    Ok(owner)
}
/// Each cause denotes an admitted EXTERIOR event cut. Its target identifies
/// why the boundary exists, not which child's alignment frame should move it.
/// These are generated-context owners, never invented scalar source roots.
pub(crate) fn visit_computed_frame_owners(
    analyzed: &AnalyzedData,
    occurrence: usize,
    budget: &mut ScopeAlignmentBudget,
    mut emit: impl FnMut(usize, &mut ScopeAlignmentBudget) -> Result<()>,
) -> Result<()> {
    let topology = analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("admitted computed capture");
    for cause in &analyzed.paragraph.computed_boundaries[occurrence].causes {
        budget.work(1)?;
        let cause = &topology.capture.computed[*cause];
        if cause.domain != topology.domain {
            return Err(refusal(ScopeAlignmentError::WrongDomain));
        }
        emit(
            computed_frame_owner_at(analyzed, cause.capture_cut)?,
            budget,
        )?;
    }
    Ok(())
}
fn canonical_computed_owner(prepared: &PreparedData, boundary: usize) -> Result<usize> {
    let occurrence = prepared.tables.computed_boundaries[boundary].occurrence;
    computed_frame_owner_at(
        &prepared.analyzed,
        prepared.analyzed.paragraph.computed_boundaries[occurrence].capture_cut,
    )
}

fn is_descendant(
    capture: &crate::processing::contextual::ScopeCapture,
    ancestor: usize,
    child: usize,
) -> bool {
    let a = &capture.scopes[ancestor];
    let b = &capture.scopes[child];
    a.domain == b.domain && a.enter <= b.enter && b.exit <= a.exit
}

fn alternative_owner(
    analyzed: &AnalyzedData,
    definition: crate::protocol::prepared::AlternativeDefinition,
) -> Option<usize> {
    use crate::protocol::prepared::AlternativeDefinition;
    let topology = analyzed.paragraph.scope_topology.as_ref()?;
    let order = match definition {
        AlternativeDefinition::Source { origin, .. }
        | AlternativeDefinition::Automatic { origin, .. } => {
            analyzed.paragraph.captured_origin_event(origin)?
        }
        AlternativeDefinition::Control { control, .. } => *topology.control_orders.get(control)?,
    };
    topology
        .capture
        .events
        .get(order)
        .and_then(|event| event.before)
}

fn control_frame_owner(
    capture: &crate::processing::contextual::ScopeCapture,
    topology: &crate::processing::contextual::ParagraphTopology,
    occurrence: usize,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Option<usize>> {
    budget.work(1)?;
    let event = &capture.events[topology.control_orders[occurrence]];
    let SourceProcessingEvent::Control { control, .. } = event.event else {
        return Ok(None);
    };
    for (index, scope) in capture.scopes.iter().enumerate() {
        budget.work(1)?;
        if scope
            .boundaries
            .is_some_and(|pair| pair.start == control || pair.end == control)
        {
            return Ok(Some(index));
        }
    }
    Ok(event.before)
}

fn visit_unit_source_owners(
    prepared: &PreparedData,
    unit: &crate::protocol::prepared::PresentationUnit,
    budget: &mut ScopeAlignmentBudget,
    mut emit: impl FnMut(usize, &mut ScopeAlignmentBudget) -> Result<()>,
) -> Result<()> {
    use crate::protocol::prepared::PresentationUnitKind as K;
    use crate::transform::relation::GeneratedSeparatorOwner as G;
    let topology = prepared
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("captured source owner");
    let capture = &topology.capture;
    budget.work(1)?;
    let owner = match unit.kind {
        K::Cell { cell } => {
            budget.work(1)?;
            visit_cell_frame_owners(
                &prepared.analyzed,
                prepared.shaped.cells[cell].binding,
                budget,
                &mut emit,
            )?;
            None
        }
        K::Atomic { occurrence } => capture.events[capture.atomic_orders[occurrence]].before,
        K::Control { occurrence } => capture.events[topology.control_orders[occurrence]].before,
        K::EmptyOrigin { origin } => {
            capture.events[prepared
                .analyzed
                .paragraph
                .captured_origin_event(origin)
                .expect("captured empty origin retains its event")]
            .before
        }
        K::Replacement { alternative, .. } => alternative_owner(
            &prepared.analyzed,
            prepared.tables.alternatives[alternative].definition,
        ),
        K::GeneratedSeparator {
            owner: G::Control(control),
        } => capture.events[topology.control_orders[control]].before,
        K::GeneratedSeparator {
            owner: G::Virtual(index),
        } => Some(
            prepared.analyzed.transform.scopes[prepared.analyzed.transform.virtuals[index].scope]
                .ordinal,
        ),
        K::Text | K::Deleted => {
            prepared
                .analyzed
                .paragraph
                .visit_origin_indices(unit.source, |event| {
                    match event {
                        crate::protocol::paragraph::OriginVisit::Inspect => budget.work(1)?,
                        crate::protocol::paragraph::OriginVisit::Intersection {
                            origin,
                            aggregate,
                        } => {
                            if !aggregate.is_empty() {
                                budget.work(1)?;
                                if let Some(owner) = capture.events[prepared
                                    .analyzed
                                    .paragraph
                                    .captured_origin_event(origin)
                                    .expect("captured source origin retains its event")]
                                .before
                                {
                                    emit(owner, budget)?;
                                }
                            }
                        }
                    }
                    Ok(())
                })?;
            None
        }
        K::ComputedLineBoundary { boundary } => {
            visit_computed_frame_owners(
                &prepared.analyzed,
                prepared.tables.computed_boundaries[boundary].occurrence,
                budget,
                &mut emit,
            )?;
            None
        }
    };
    if let Some(owner) = owner {
        emit(owner, budget)?;
    }
    Ok(())
}

fn selected_owners(
    prepared: &PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    budget: &mut ScopeAlignmentBudget,
) -> Result<Vec<Vec<usize>>> {
    use crate::protocol::prepared::PresentationUnitKind as K;
    use crate::transform::relation::GeneratedSeparatorOwner as G;
    let mut result = Vec::new();
    for (atom, original) in presentation.originals.iter().enumerate() {
        budget.work(1)?;
        let mut owners = Vec::new();
        if let Some(original) = original {
            for &owner in &prepared.scope_alignment.atom_owners[*original] {
                budget.work(1)?;
                push(budget, &mut owners, owner)?;
            }
        } else {
            budget.work(presentation.units.len())?;
            let unit = presentation
                .units
                .iter()
                .find(|unit| unit.atoms.contains(&atom))
                .expect("actual selected atom belongs to its selected unit");
            let owner = match unit.kind {
                K::Replacement { alternative, .. } => alternative_owner(
                    &prepared.analyzed,
                    prepared.tables.alternatives[alternative].definition,
                ),
                K::GeneratedSeparator {
                    owner: G::Control(control),
                } => {
                    let topology = prepared
                        .analyzed
                        .paragraph
                        .scope_topology
                        .as_ref()
                        .expect("active capture");
                    topology.capture.events[topology.control_orders[control]].before
                }
                K::GeneratedSeparator {
                    owner: G::Virtual(index),
                } => Some(
                    prepared.analyzed.transform.scopes
                        [prepared.analyzed.transform.virtuals[index].scope]
                        .ordinal,
                ),
                _ => None,
            };
            if let Some(owner) = owner {
                push(budget, &mut owners, owner)?;
            } else {
                return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                    scope: 0,
                }));
            }
        }
        push(budget, &mut result, owners)?;
    }
    Ok(result)
}

#[derive(Clone, Copy)]
pub(crate) enum PlanInput<'a> {
    Selection(&'a crate::LineSelectionRequest),
    Flow {
        segment: &'a crate::PreparedFlowSegment,
        request: &'a crate::FlowObservationRequest,
    },
}
impl<'a> PlanInput<'a> {
    pub(crate) fn inline_box_family(self) -> Option<&'a crate::PreparedInlineBoxFamily> {
        match self {
            Self::Selection(request) => request.inline_boxes.as_ref(),
            Self::Flow { segment, .. } => segment.inline_box_family(),
        }
    }
    fn visit(
        self,
        emit: &mut dyn FnMut(crate::protocol::scope_alignment::ScopeProofVisit<'a>) -> Result<()>,
    ) -> Result<()> {
        match self {
            Self::Selection(request) => {
                crate::protocol::scope_alignment::visit_request_scope_frames(request, emit)
            }
            Self::Flow { segment, .. } => segment.visit_scope_frames(emit),
        }
    }
}
fn insert_proof_frame(
    frames: &mut Vec<crate::protocol::FinalizedLine>,
    frame: &crate::protocol::FinalizedLine,
    budget: &mut ScopeAlignmentBudget,
) -> Result<()> {
    budget.work(frames.len() + 1)?;
    if !frames
        .iter()
        .any(|existing| Arc::ptr_eq(&existing.inner, &frame.inner))
    {
        push(budget, frames, frame.clone())?;
    }
    Ok(())
}

#[derive(Default)]
pub(crate) struct SelectionPlan {
    pub(crate) owners: Vec<Vec<usize>>,
    pub(crate) breaks: Vec<usize>,
    pub(crate) proof_frames: Vec<crate::protocol::FinalizedLine>,
    pub(crate) budget: Option<ScopeAlignmentBudget>,
}

pub(crate) fn selection_plan(
    prepared: &PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    allowance: Option<crate::ScopeAlignmentAllowance>,
    input: PlanInput<'_>,
) -> Result<SelectionPlan> {
    operation_limits(prepared, allowance)?;
    let Some(_retention) = prepared.scope_alignment.retention else {
        return Ok(SelectionPlan::default());
    };
    let topology = prepared
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("active capture");
    let capture = &topology.capture;
    let mut budget = match input {
        PlanInput::Selection(request) => census::begin_selection(prepared, request, allowance)?,
        PlanInput::Flow { segment, request } => {
            census::begin_flow(prepared, segment, request, allowance)?
        }
    }
    .expect("active captured input retains its ledger");
    let mut proof_frames = Vec::new();
    input.visit(&mut |event| {
        match event {
            crate::protocol::scope_alignment::ScopeProofVisit::Work(count) => budget.work(count)?,
            crate::protocol::scope_alignment::ScopeProofVisit::Family(_) => budget.work(1)?,
            crate::protocol::scope_alignment::ScopeProofVisit::Frame(frame) => {
                insert_proof_frame(&mut proof_frames, frame, &mut budget)?;
                for ancestor in &frame.inner.selected.scopes.proof_frames {
                    insert_proof_frame(&mut proof_frames, ancestor, &mut budget)?;
                }
            }
        }
        Ok(())
    })?;
    let owners = selected_owners(prepared, presentation, &mut budget)?;
    let mut breaks = Vec::new();
    for scope in &prepared.scope_alignment.targets {
        budget.work(1)?;
        let scope = *scope;
        if capture
            .scope_alignment_fact_with_budget(scope, &mut budget)?
            .and_then(|row| row.alignment_boundary)
            != Some(ScopeAlignmentBoundary::NonInitial)
        {
            continue;
        }
        let mut previous = None;
        for (atom, own) in owners.iter().enumerate() {
            budget.work(own.len() + 1)?;
            let mut inside = false;
            let mut outside = false;
            for &owner in own {
                let enclosed = is_descendant(capture, scope, owner);
                inside |= enclosed;
                outside |= !enclosed;
            }
            if inside && outside {
                return Err(refusal(ScopeAlignmentError::UnpartitionableUnit { scope }));
            }
            if previous.is_some_and(|old| old != inside) {
                budget.work(presentation.analysis.graphemes.len() + breaks.len())?;
                if presentation
                    .analysis
                    .graphemes
                    .iter()
                    .any(|range| range.start < atom && atom < range.end)
                {
                    return Err(refusal(ScopeAlignmentError::UnpartitionableUnit { scope }));
                }
                if !breaks.contains(&atom) {
                    push(&mut budget, &mut breaks, atom)?;
                }
            }
            previous = Some(inside);
        }
    }
    // A genuine original formatting boundary at the selected body's endpoint
    // remains a real body cut. Merely starting a new line inside a scope
    // does not create one; Unicode joining lookaround remains genuine.
    for (local, original) in [
        (0, presentation.originals.first().copied().flatten()),
        (
            presentation.originals.len(),
            presentation
                .originals
                .last()
                .copied()
                .flatten()
                .map(|atom| atom + 1),
        ),
    ] {
        let original_break = match original {
            Some(original) => operations::binary_search_by(
                &prepared.scope_alignment.breaks,
                Some(&mut budget),
                |value| value.cmp(&original),
            )?
            .is_ok(),
            None => false,
        };
        if original_break {
            budget.work(breaks.len())?;
        }
        if original_break && !breaks.contains(&local) {
            push(&mut budget, &mut breaks, local)?;
        }
    }
    operations::order_by(&mut breaks, Some(&mut budget), usize::cmp)?;
    Ok(SelectionPlan {
        owners,
        breaks,
        proof_frames,
        budget: Some(budget),
    })
}

/// Actual retained-font input is known before shaping. Admit selected resource
/// pins before feature extraction/itemization scratch can coexist with them.
#[derive(Clone)]
struct SelectedFonts<'a> {
    prepared: &'a PreparedData,
    presentation: &'a crate::protocol::prepared::SelectedPresentation,
    empty: Option<&'a super::layout::data::PreparedRun>,
    assignment: usize,
    atom: usize,
    unit: usize,
}
impl<'a> SelectedFonts<'a> {
    fn next(&mut self, budget: &mut ScopeAlignmentBudget) -> Result<Option<&'a FontData>> {
        if self.assignment < self.presentation.assignments.len() {
            budget.work(1)?;
            let row = &self.presentation.assignments[self.assignment];
            self.assignment += 1;
            return Ok(Some(&row.run.font));
        }
        if let Some(run) = self.empty.take() {
            budget.work(1)?;
            return Ok(Some(&run.font));
        }
        while self.atom < self.presentation.projection.atoms.len() {
            budget.work(1)?;
            let atom = self.atom;
            self.atom += 1;
            if self.prepared.options.tabs() != crate::TabPreparation::CssStops
                || self.presentation.projection.atoms[atom].code_point != 9
            {
                continue;
            }
            while self.unit < self.presentation.units.len() {
                budget.work(1)?;
                if self.presentation.units[self.unit].atoms.end > atom {
                    break;
                }
                self.unit += 1;
            }
            if self.presentation.units.get(self.unit).is_some_and(|unit| {
                unit.atoms.contains(&atom)
                    && matches!(
                        unit.kind,
                        crate::protocol::prepared::PresentationUnitKind::Cell { .. }
                    )
            }) {
                continue;
            }
            if let Some(font) = self.prepared.tab_metrics[self.presentation.analysis.infos[atom].1]
                .as_ref()
                .and_then(|metric| metric.font.as_ref())
            {
                return Ok(Some(&font.font));
            }
        }
        Ok(None)
    }
}

pub(crate) fn admit_selected_resources(
    prepared: &PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    empty: Option<&super::layout::data::PreparedRun>,
    proof_frames: &[crate::protocol::FinalizedLine],
    ledger: &mut accounting::Ledger,
) -> Result<()> {
    let Some(budget) = ledger else { return Ok(()) };
    let fonts = SelectedFonts {
        prepared,
        presentation,
        empty,
        assignment: 0,
        atom: 0,
        unit: 0,
    };
    let same = |left: &FontData, right: &FontData| {
        left.bytes().as_ptr() == right.bytes().as_ptr() && left.bytes().len() == right.bytes().len()
    };
    let mut current = fonts.clone();
    let mut index = 0;
    while let Some(font) = current.next(budget)? {
        let mut pinned = false;
        for metric in &prepared.scope_alignment.metrics {
            budget.work(1)?;
            if same(&metric.font, font) {
                pinned = true;
                break;
            }
        }
        if !pinned {
            for reference in &prepared.scope_alignment.cell_tabs {
                budget.work(1)?;
                for metric in [&reference.space, &reference.ch].into_iter().flatten() {
                    budget.work(1)?;
                    if let Some(other) = &metric.font {
                        pinned |= same(&other.font, font);
                    }
                }
            }
        }
        if !pinned && prepared.shaped.cell_arenas == super::layout::data::CellArenaOwnership::Owned
        {
            for run in &prepared.shaped.runs {
                budget.work(1)?;
                if same(&run.font, font) {
                    pinned = true;
                    break;
                }
            }
        }
        if !pinned {
            let mut previous_fonts = fonts.clone();
            for _ in 0..index {
                let previous = previous_fonts
                    .next(budget)?
                    .expect("actual earlier selected font");
                budget.work(1)?;
                if same(previous, font) {
                    pinned = true;
                    break;
                }
            }
        }
        if !pinned {
            pinned = census::proof_frames_contain_resource(proof_frames, font.bytes(), budget)?;
        }
        if !pinned {
            budget.retain_bytes(font.bytes().len())?;
        }
        index += 1;
    }
    Ok(())
}

/// Retains exact selected descendants and actual shared operand relations.
pub(crate) fn select(
    prepared: &PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    basis: &super::SelectedShape,
    request: &crate::LineSelectionRequest,
    spacing_slots: &Vec<crate::protocol::selection::SpacingSlotData>,
    plan: SelectionPlan,
) -> Result<SelectedScopes> {
    let shaped = &basis.shaped;
    let Some(_retention) = prepared.scope_alignment.retention else {
        return Ok(SelectedScopes::default());
    };
    let topology = prepared
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("active capture");
    let capture = &topology.capture;
    let SelectionPlan {
        owners,
        breaks,
        proof_frames,
        budget,
    } = plan;
    let mut budget = budget.expect("active selected source plan");
    let mut active = Vec::new();
    let mut activate = |owner: usize, budget: &mut ScopeAlignmentBudget| -> Result<()> {
        budget.work(1)?;
        for &target in &prepared.scope_alignment.targets {
            budget.work(active.len() + 1)?;
            if is_descendant(capture, target, owner) && !active.contains(&target) {
                metric_index(
                    prepared,
                    &CapturedScope {
                        capture: Arc::clone(capture),
                        index: target,
                    },
                    Some(&mut *budget),
                )?;
                push(budget, &mut active, target)?;
            }
        }
        Ok(())
    };
    for glyph in &shaped.scope_glyph_custody {
        budget.work(1)?;
        match *glyph {
            GlyphCustody::Input(owner)
            | GlyphCustody::Context(owner)
            | GlyphCustody::UnqualifiedContext(Some(owner)) => activate(owner, &mut budget)?,
            GlyphCustody::Unscoped | GlyphCustody::UnqualifiedContext(None) => {}
        }
    }
    for relation in &shaped.scope_relations {
        budget.work(1)?;
        for owner in [relation.first, relation.second].into_iter().flatten() {
            activate(owner, &mut budget)?;
        }
    }
    use crate::protocol::prepared::PresentationUnitKind as K;
    for unit in &presentation.units {
        budget.work(1)?;
        let owner = match unit.kind {
            K::Cell { cell } => {
                budget.work(1)?;
                visit_cell_frame_owners(
                    &prepared.analyzed,
                    prepared.shaped.cells[cell].binding,
                    &mut budget,
                    &mut activate,
                )?;
                None
            }
            K::Atomic { occurrence } => capture.events[capture.atomic_orders[occurrence]].before,
            K::Control { occurrence } => {
                control_frame_owner(capture, topology, occurrence, &mut budget)?
            }
            K::ComputedLineBoundary { boundary } => {
                visit_computed_frame_owners(
                    &prepared.analyzed,
                    prepared.tables.computed_boundaries[boundary].occurrence,
                    &mut budget,
                    &mut activate,
                )?;
                None
            }
            _ => None,
        };
        if let Some(owner) = owner {
            activate(owner, &mut budget)?;
        }
    }
    for strut in &request.line_box_struts {
        budget.work(1)?;
        if let Some(scope) = &strut.scope {
            activate(scope.index, &mut budget)?;
        }
    }
    // Parent-before-child order permits a single cumulative frame pass later.
    operations::order_by(&mut active, Some(&mut budget), |left, right| {
        capture.scopes[*left]
            .enter
            .cmp(&capture.scopes[*right].enter)
    })?;
    budget.retain_bytes(std::mem::size_of::<SelectedScopes>())?;
    let mut result = SelectedScopes {
        proof_frames,
        ..SelectedScopes::default()
    };
    for &scope in &active {
        budget.work(1)?;
        let captured = CapturedScope {
            capture: Arc::clone(capture),
            index: scope,
        };
        let metrics = metric_index(prepared, &captured, Some(&mut budget))?;
        let parent = capture.scopes[scope].parent;
        let parent_metrics = parent
            .map(|scope| {
                metric_index(
                    prepared,
                    &CapturedScope {
                        capture: Arc::clone(capture),
                        index: scope,
                    },
                    Some(&mut budget),
                )
            })
            .transpose()?;
        let parent = selected_frame(&result.rows, scope, &mut budget)?;
        push(
            &mut budget,
            &mut result.rows,
            SelectedScopeData {
                scope: captured,
                metrics,
                parent_metrics,
                parent,
                direct: ContributionChain::default(),
            },
        )?;
    }
    let mut cluster_owners = Vec::new();
    let mut shared = Vec::new();
    for cluster in &shaped.clusters {
        budget.work(1)?;
        let mut source = None;
        for atom in cluster.atoms.clone() {
            budget.work(1)?;
            if let Some(owner) = owners[atom].first() {
                budget.work(1)?;
                source = Some(*owner);
                break;
            }
        }
        let owner = source
            .map(|owner| selected_frame(&result.rows, owner, &mut budget))
            .transpose()?
            .flatten();
        push(&mut budget, &mut cluster_owners, owner)?;
    }
    for grapheme in &presentation.analysis.graphemes {
        budget.work(1)?;
        retain_scope_group(
            grapheme.clone(),
            &owners,
            &result.rows,
            &mut shared,
            &mut budget,
        )?;
    }
    for cell in &shaped.cells {
        budget.work(1)?;
        let mut frames = Vec::new();
        visit_cell_frame_owners(
            &prepared.analyzed,
            cell.binding,
            &mut budget,
            |owner, budget| append_owner_frame(owner, &result.rows, &mut frames, budget),
        )?;
        finish_owner_group(frames, &mut shared, &mut budget)?;
    }
    let mut uncertain = Vec::new();
    for relation in &shaped.scope_relations {
        budget.work(1)?;
        let mut first = relation
            .first
            .map(|owner| selected_frame(&result.rows, owner, &mut budget))
            .transpose()?
            .flatten();
        let mut second = relation
            .second
            .map(|owner| selected_frame(&result.rows, owner, &mut budget))
            .transpose()?
            .flatten();
        if first == second {
            continue;
        }
        if first > second {
            std::mem::swap(&mut first, &mut second);
        }
        let pairs = if relation.unqualified {
            &mut uncertain
        } else {
            &mut shared
        };
        budget.work(pairs.len())?;
        if !pairs.contains(&(first, second)) {
            push(&mut budget, pairs, (first, second))?;
        }
    }
    for glyph in &shaped.scope_glyph_custody {
        budget.work(1)?;
        let owner = match *glyph {
            GlyphCustody::Input(owner)
            | GlyphCustody::Context(owner)
            | GlyphCustody::UnqualifiedContext(Some(owner)) => {
                selected_frame(&result.rows, owner, &mut budget)?
            }
            GlyphCustody::Unscoped | GlyphCustody::UnqualifiedContext(None) => None,
        };
        push(&mut budget, &mut result.glyph_owners, owner)?;
    }
    if !result.rows.is_empty() && result.glyph_owners.len() != shaped.glyphs.len() {
        return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
            scope: result.rows[0].scope.index,
        }));
    }
    let mut unit_owners = Vec::new();
    for unit in &presentation.units {
        budget.work(1)?;
        if let K::Cell { cell } = unit.kind {
            budget.work(2)?;
            let cell = &prepared.shaped.cells[cell];
            let source = prepared.scope_alignment.metrics[cell.metrics].scope.index;
            let owner = selected_frame(&result.rows, source, &mut budget)?;
            push(&mut budget, &mut unit_owners, owner)?;
            continue;
        }
        let mut sources = Vec::new();
        if !unit.atoms.is_empty() {
            for atom in unit.atoms.clone() {
                budget.work(1)?;
                for &owner in &owners[atom] {
                    budget.work(sources.len() + 1)?;
                    if !sources.contains(&owner) {
                        push(&mut budget, &mut sources, owner)?;
                    }
                }
            }
        } else {
            visit_unit_source_owners(prepared, unit, &mut budget, |owner, budget| {
                budget.work(sources.len() + 1)?;
                if !sources.contains(&owner) {
                    push(budget, &mut sources, owner)?;
                }
                Ok(())
            })?;
        }
        let source = match unit.kind {
            K::Control { occurrence } => {
                control_frame_owner(capture, topology, occurrence, &mut budget)?
            }
            K::ComputedLineBoundary { boundary } => {
                // One canonical native anchor can serve several real causes
                // only when all their admitted context frames are coherent.
                retain_owner_group(&sources, &result.rows, &mut shared, &mut budget)?;
                Some(canonical_computed_owner(prepared, boundary)?)
            }
            _ => {
                // Phase-I deletions split at every original origin endpoint;
                // EmptyOrigin and replaced SHY carry one exact occurrence.
                // An unexpected source-only multi-owner row cannot silently
                // acquire its first owner's frame.
                if unit.atoms.is_empty() && sources.len() > 1 {
                    return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                        scope: sources[0],
                    }));
                }
                sources.first().copied()
            }
        };
        let owner = source
            .map(|owner| selected_frame(&result.rows, owner, &mut budget))
            .transpose()?
            .flatten();
        push(&mut budget, &mut unit_owners, owner)?;
        let bytes = capacity_bytes(&sources)?;
        drop(sources);
        budget.release_bytes(bytes)?;
    }
    result.cluster_owners = cluster_owners;
    result.shared = shared;
    result.uncertain = uncertain;
    result.unit_owners = unit_owners;
    if shaped.cells.is_empty() {
        for (index, cluster) in shaped.clusters.iter().enumerate() {
            budget.work(1)?;
            if let Some(run) = cluster.content.font_run() {
                let metrics = super::layout::scope_font_contribution(
                    &shaped.runs[run].metrics,
                    cluster.line_height,
                )?;
                result.push_contribution(result.cluster_owners[index], metrics, &mut budget)?;
            }
        }
    } else {
        // The genuine outer item stream contains one Cell in place of its
        // inner clusters. Thus even zero-input cells contribute their square,
        // and inner font struts cannot become a second outer contribution.
        use super::layout::data::ShapedItem;
        for item in &shaped.items {
            budget.work(1)?;
            match *item {
                ShapedItem::Cell(index) => {
                    budget.work(2)?;
                    let cell = &shaped.cells[index];
                    let scope = prepared.scope_alignment.metrics[cell.metrics].scope.index;
                    let owner = selected_frame(&result.rows, scope, &mut budget)?;
                    result.push_contribution(
                        owner,
                        crate::ScopeMetricContribution {
                            before: cell.before,
                            after: cell.after,
                            alignment: crate::LineBoxAlignment::Baseline,
                        },
                        &mut budget,
                    )?;
                }
                ShapedItem::TextCluster(index) => {
                    budget.work(1)?;
                    let cluster = &shaped.clusters[index];
                    if let Some(run) = cluster.content.font_run() {
                        let metrics = super::layout::scope_font_contribution(
                            &shaped.runs[run].metrics,
                            cluster.line_height,
                        )?;
                        result.push_contribution(
                            result.cluster_owners[index],
                            metrics,
                            &mut budget,
                        )?;
                    }
                }
                ShapedItem::InlineBox { .. } | ShapedItem::Anchor { .. } => {}
            }
        }
    }
    for allocation in &request.atomics {
        budget.work(1)?;
        if let crate::AtomicLineAllocation::InFlow {
            flow, alignment, ..
        } = allocation.allocation
        {
            let owner = selected_unit_index(
                presentation,
                K::Atomic {
                    occurrence: prepared.tables.atomics[allocation.occurrence.index].occurrence,
                },
                &mut budget,
            )?
            .and_then(|index| result.unit_owners[index]);
            result.push_contribution(
                owner,
                crate::ScopeMetricContribution {
                    before: flow.before_baseline,
                    after: flow.after_baseline,
                    alignment,
                },
                &mut budget,
            )?;
        }
    }
    for allocation in &request.controls {
        budget.work(1)?;
        if let Some(strut) = allocation.strut {
            let occurrence = prepared.tables.controls[allocation.occurrence.index].occurrence;
            let owner = selected_unit_index(presentation, K::Control { occurrence }, &mut budget)?
                .and_then(|index| result.unit_owners[index]);
            if matches!(
                strut.alignment,
                crate::LineBoxAlignment::Top | crate::LineBoxAlignment::Bottom
            ) && let Some(index) = owner
                && matches!(capture.events[topology.control_orders[occurrence]].event,
                    SourceProcessingEvent::Control{control,..} if capture.scopes[result.rows[index].scope.index].boundaries.is_some_and(|pair|pair.start==control||pair.end==control))
            {
                return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                    scope: result.rows[index].scope.index,
                }));
            }
            result.push_contribution(
                owner,
                crate::ScopeMetricContribution {
                    before: strut.baseline,
                    after: strut.line_extent - strut.baseline,
                    alignment: strut.alignment,
                },
                &mut budget,
            )?;
        }
    }
    if prepared.analyzed.paragraph.paragraph_strut == crate::ParagraphStrut::DefaultFont {
        let run = prepared.shaped.empty_run.as_ref().ok_or_else(|| {
            crate::protocol::protocol_error(crate::ProtocolError::MissingLineMetrics)
        })?;
        let metrics =
            super::layout::scope_font_contribution(&run.metrics, run.metrics.line_height)?;
        result.push_contribution(None, metrics, &mut budget)?;
    }
    if let Some(strut) = request.line_strut {
        result.push_contribution(
            None,
            crate::ScopeMetricContribution {
                before: strut.baseline,
                after: strut.line_extent - strut.baseline,
                alignment: crate::LineBoxAlignment::Baseline,
            },
            &mut budget,
        )?;
    }
    for strut in &request.line_box_struts {
        budget.work(1)?;
        let owner = if let Some(scope) = &strut.scope {
            if !Arc::ptr_eq(capture, &scope.capture) {
                return Err(refusal(ScopeAlignmentError::ForeignCapture));
            }
            if capture.scopes[scope.index].domain != topology.domain {
                return Err(refusal(ScopeAlignmentError::WrongDomain));
            }
            selected_frame(&result.rows, scope.index, &mut budget)?
        } else {
            None
        };
        push(&mut budget, &mut result.strut_owners, owner)?;
        result.push_contribution(
            owner,
            crate::ScopeMetricContribution {
                before: strut.metrics.baseline,
                after: strut.metrics.line_extent - strut.metrics.baseline,
                alignment: strut.alignment,
            },
            &mut budget,
        )?;
    }
    let mut scratch = capacity_bytes(&owners)?;
    for row in &owners {
        budget.work(1)?;
        scratch = checked_sum(
            crate::ScopeAlignmentLimitKind::PayloadBytes,
            scratch,
            capacity_bytes(row)?,
        )?;
    }
    scratch = checked_sum(
        crate::ScopeAlignmentLimitKind::PayloadBytes,
        scratch,
        capacity_bytes(&active)?,
    )?;
    scratch = checked_sum(
        crate::ScopeAlignmentLimitKind::PayloadBytes,
        scratch,
        capacity_bytes(&breaks)?,
    )?;
    drop(owners);
    drop(active);
    drop(breaks);
    budget.release_bytes(scratch)?;
    result.retention = Some(census::retained_selected(
        prepared,
        &result,
        basis,
        request.context.spacing,
        spacing_slots,
        request,
        &mut budget,
    )?);
    result.usage = Some(budget.usage());
    Ok(result)
}

fn checked_sum(kind: crate::ScopeAlignmentLimitKind, left: usize, right: usize) -> Result<usize> {
    left.checked_add(right)
        .ok_or_else(|| refusal(ScopeAlignmentError::Overflow { kind }))
}
fn capacity_bytes<T>(values: &Vec<T>) -> Result<usize> {
    values
        .capacity()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| {
            refusal(ScopeAlignmentError::Overflow {
                kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
            })
        })
}

fn argument_retention<T>(values: &Vec<T>) -> Result<crate::ScopeAlignmentRetention> {
    // The enum's Vec header moves on the stack. Only its real buffer is heap
    // custody; element metadata is already included in size_of::<T>().
    Ok(crate::ScopeAlignmentRetention {
        records: values.len(),
        payload_bytes: capacity_bytes(values)?,
    })
}
pub(crate) fn spacing_input_retention(
    values: &crate::SpacingAllocations,
) -> Result<crate::ScopeAlignmentRetention> {
    match values {
        crate::SpacingAllocations::None => Ok(crate::ScopeAlignmentRetention::default()),
        crate::SpacingAllocations::Complete(rows) => argument_retention(rows),
    }
}
fn input_budget(
    limits: crate::ScopeAlignmentLimits,
    base: crate::ScopeAlignmentRetention,
    scope: crate::ScopeAlignmentRetention,
    spacing: crate::ScopeAlignmentRetention,
) -> Result<ScopeAlignmentBudget> {
    let records = checked_sum(
        crate::ScopeAlignmentLimitKind::Records,
        checked_sum(
            crate::ScopeAlignmentLimitKind::Records,
            base.records(),
            scope.records(),
        )?,
        spacing.records(),
    )?;
    let bytes = checked_sum(
        crate::ScopeAlignmentLimitKind::PayloadBytes,
        checked_sum(
            crate::ScopeAlignmentLimitKind::PayloadBytes,
            base.payload_bytes(),
            scope.payload_bytes(),
        )?,
        spacing.payload_bytes(),
    )?;
    ScopeAlignmentBudget::new(limits, records, bytes)
}

pub(crate) fn allocation_frames(
    selected: &Arc<crate::protocol::selection::SelectedData>,
    allocations: crate::ScopeAllocations,
    allowance: Option<crate::ScopeAlignmentAllowance>,
    spacing: &crate::SpacingAllocations,
) -> Result<FrameAllocations> {
    let limits = operation_limits(&selected.prepared, allowance)?;
    let count = selected.scopes.rows.len();
    let Some(retention) = selected.scopes.retention else {
        return match allocations {
            crate::ScopeAllocations::None => Ok(FrameAllocations::default()),
            crate::ScopeAllocations::Complete(rows) if rows.is_empty() => {
                Ok(FrameAllocations::default())
            }
            _ => Err(refusal(ScopeAlignmentError::ForeignSelectedScope)),
        };
    };
    let scope_input = match &allocations {
        crate::ScopeAllocations::None => crate::ScopeAlignmentRetention::default(),
        crate::ScopeAllocations::Complete(rows) => argument_retention(rows)?,
    };
    let mut budget = input_budget(
        limits.expect("feature retention retains limits"),
        retention,
        scope_input,
        spacing_input_retention(spacing)?,
    )?;
    let mut own = Vec::new();
    budget.add_records(count)?;
    budget.work(count)?;
    budget.reserve(&mut own, count)?;
    own.resize(count, None);
    match allocations {
        crate::ScopeAllocations::None if count != 0 => {
            return Err(refusal(ScopeAlignmentError::MissingAllocation {
                scope: selected.scopes.rows[0].scope.index,
            }));
        }
        crate::ScopeAllocations::None => {}
        crate::ScopeAllocations::Complete(rows) => {
            for row in rows {
                budget.work(1)?;
                if !Arc::ptr_eq(selected, &row.scope.selected) || row.scope.index >= count {
                    return Err(refusal(ScopeAlignmentError::ForeignSelectedScope));
                }
                let index = row.scope.index;
                if own[index].is_some() {
                    return Err(refusal(ScopeAlignmentError::DuplicateAllocation {
                        scope: selected.scopes.rows[index].scope.index,
                    }));
                }
                own[index] = Some(row.displacement);
            }
        }
    }
    budget.release_bytes(scope_input.payload_bytes())?;
    let mut result: Vec<crate::ScopeFrameAllocation> = Vec::new();
    for (index, row) in selected.scopes.rows.iter().enumerate() {
        budget.work(1)?;
        let displacement = own[index].ok_or_else(|| {
            refusal(ScopeAlignmentError::MissingAllocation {
                scope: row.scope.index,
            })
        })?;
        let parent = row
            .parent
            .map_or(0.0, |parent| result[parent].cumulative_displacement.get());
        let cumulative = super::layout::add(
            "cumulative scope frame",
            crate::TextNumericPhase::Finalization,
            parent,
            displacement.get(),
        )?;
        let cumulative_displacement = crate::BaselineShift::try_new(cumulative)?;
        push(
            &mut budget,
            &mut result,
            crate::ScopeFrameAllocation {
                scope: row.scope.clone(),
                displacement,
                cumulative_displacement,
            },
        )?;
    }
    for &(first, second) in &selected.scopes.shared {
        budget.work(1)?;
        let value = |owner: Option<usize>| {
            owner.map_or(0.0, |index| result[index].cumulative_displacement.get())
        };
        if value(first) != value(second) {
            let (Some(first), Some(second)) = (first, second) else {
                let scope = first
                    .or(second)
                    .expect("a distinct frame pair has a target");
                return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                    scope: selected.scopes.rows[scope].scope.index,
                }));
            };
            return Err(refusal(ScopeAlignmentError::IncompatibleFrames {
                first: selected.scopes.rows[first].scope.index,
                second: selected.scopes.rows[second].scope.index,
            }));
        }
    }
    for &(first, second) in &selected.scopes.uncertain {
        budget.work(1)?;
        let value = |owner: Option<usize>| {
            owner.map_or(0.0, |index| result[index].cumulative_displacement.get())
        };
        if value(first) != value(second) {
            let scope = first
                .or(second)
                .expect("a distinct frame pair has a target");
            return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                scope: selected.scopes.rows[scope].scope.index,
            }));
        }
    }
    let own_bytes = capacity_bytes(&own)?;
    drop(own);
    budget.release_bytes(own_bytes)?;
    Ok(FrameAllocations {
        frames: result,
        budget: Some(budget),
    })
}

#[derive(Default)]
pub(crate) struct FrameAllocations {
    pub(crate) frames: Vec<crate::ScopeFrameAllocation>,
    pub(crate) budget: Option<ScopeAlignmentBudget>,
}
impl FrameAllocations {
    pub(crate) fn displacement(&self, owner: Option<usize>) -> f64 {
        owner.map_or(0.0, |index| {
            self.frames[index].cumulative_displacement.get()
        })
    }
}

pub(crate) fn replay_frames(
    line: &super::FinalizedLine,
    allowance: Option<crate::ScopeAlignmentAllowance>,
    spacing: &crate::SpacingAllocations,
) -> Result<FrameAllocations> {
    let limits = operation_limits(&line.selected.prepared, allowance)?;
    let Some(retention) = line.data.scope_retention else {
        return Ok(FrameAllocations::default());
    };
    // The borrowed previous finalization is simultaneous input, including all
    // of its actual feature geometry, not unrelated caller-retained history.
    let mut budget = input_budget(
        limits.expect("feature retention retains limits"),
        retention,
        crate::ScopeAlignmentRetention::default(),
        spacing_input_retention(spacing)?,
    )?;
    let mut frames = Vec::new();
    for frame in &line.data.scope_allocations {
        budget.work(1)?;
        push(&mut budget, &mut frames, frame.clone())?;
    }
    Ok(FrameAllocations {
        frames,
        budget: Some(budget),
    })
}

/// Publish the actually retained finalizer owner after all metered scratch has
/// dropped. The old replay input and borrowed caller buffers coexist through
/// execution; their departure changes live custody, never the measured peak.
pub(crate) fn finish_frames(
    selected: &crate::protocol::selection::SelectedData,
    data: &mut super::layout::data::FinalizedLineData,
    scopes: FrameAllocations,
) -> Result<()> {
    let FrameAllocations { frames, budget } = scopes;
    data.scope_allocations = frames;
    let Some(mut budget) = budget else {
        return Ok(());
    };
    let retention = census::retained_finalized(selected, data, &mut budget)?;
    let extra = budget
        .usage()
        .payload_bytes()
        .checked_sub(retention.payload_bytes())
        .ok_or_else(|| {
            refusal(ScopeAlignmentError::Overflow {
                kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
            })
        })?;
    budget.release_bytes(extra)?;
    data.scope_retention = Some(retention);
    data.scope_usage = Some(budget.usage());
    Ok(())
}
