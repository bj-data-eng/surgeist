//! Upfront source requirements. Envelopes never establish actual TCUs or slots.

use super::{Graph, PlanCoverage, PlanReason, Unit, unit_properties};
use crate::engine::{analysis::Analysis, resolve::ResolvedStyleRun};
use crate::projection::SourceProjection;
use crate::protocol::prepared::SelectedPresentation;
use crate::{InlineBox, Result};
use icu_properties::{
    CodePointSetData,
    props::{DefaultIgnorableCodePoint, Script},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    sync::Arc,
};

#[derive(Debug, Default)]
pub(crate) struct PlanRequirements {
    pub(crate) observe: bool,
    pub(crate) coverage: Vec<PlanCoverage>,
}
impl PlanRequirements {
    pub(crate) fn masks(&self, run: Range<usize>) -> Vec<Range<usize>> {
        let mut ranges: Vec<_> = self
            .coverage
            .iter()
            .filter_map(|row| {
                let start = row.atoms.start.max(run.start);
                let end = row.atoms.end.min(run.end);
                (start < end).then(|| start - run.start..end - run.start)
            })
            .collect();
        ranges.sort_unstable_by_key(|r| (r.start, r.end));
        let mut output: Vec<Range<usize>> = Vec::new();
        for range in ranges {
            if let Some(prior) = output.last_mut()
                && range.start <= prior.end
            {
                prior.end = prior.end.max(range.end);
            } else {
                output.push(range);
            }
        }
        output
    }
    fn add(&mut self, ranges: &[Range<usize>], reason: PlanReason) {
        self.observe = true;
        self.coverage.extend(
            ranges
                .iter()
                .cloned()
                .map(|atoms| PlanCoverage { atoms, reason }),
        );
    }
}

#[derive(Clone, Copy, Default)]
enum Operands {
    #[default]
    Empty,
    One(f64),
    Many,
}
impl Operands {
    fn add(&mut self, other: Self) {
        *self = match (*self, other) {
            (Self::Empty, v) | (v, Self::Empty) => v,
            (Self::One(a), Self::One(b)) if a == b => Self::One(a),
            _ => Self::Many,
        };
    }
    fn material(self, other: Self) -> bool {
        match (self, other) {
            (Self::Empty, _) | (_, Self::Empty) => false,
            (Self::One(a), Self::One(b)) => a != -b,
            _ => true,
        }
    }
}
#[derive(Clone, Copy, Default, Eq, PartialEq)]
struct Protection {
    atomic: bool,
    authored: Option<Script>,
    justify: Option<Script>,
}
#[derive(Clone, Copy)]
struct Endpoint {
    operands: Operands,
    protection: Protection,
}
impl Endpoint {
    fn demands(self, other: Self, potential: bool) -> bool {
        if self.protection.atomic && other.protection.atomic {
            return false;
        }
        let authored = self.protection.authored.is_some()
            && self.protection.authored == other.protection.authored;
        let justify = self.protection.justify.is_some()
            && self.protection.justify == other.protection.justify;
        !authored && self.operands.material(other.operands) || potential && !justify
    }
}
struct PoolRow {
    endpoint: Endpoint,
    conditional: bool,
}
#[derive(Default)]
struct Pool(Vec<PoolRow>);
impl Pool {
    fn insert(&mut self, endpoint: Endpoint, conditional: bool) {
        if let Some(row) = self
            .0
            .iter_mut()
            .find(|r| r.endpoint.protection == endpoint.protection)
        {
            row.endpoint.operands.add(endpoint.operands);
            row.conditional |= conditional;
        } else {
            self.0.push(PoolRow {
                endpoint,
                conditional,
            });
        }
    }
    fn demand(&self, endpoint: Endpoint, conditional: bool, potential: bool) -> Option<PlanReason> {
        let mut result = None;
        for row in &self.0 {
            if endpoint.demands(row.endpoint, potential) {
                if conditional || row.conditional {
                    return Some(PlanReason::ConditionalEnvelope);
                }
                result = Some(PlanReason::ExactBoundary);
            }
        }
        result
    }
}
struct UnitFacts {
    endpoint: Endpoint,
    component: Option<usize>,
    anchor: bool,
    invariant: bool,
    context_conditional: bool,
}
struct Component {
    atoms: Range<usize>,
    conditional: bool,
    all_di: bool,
    units: Vec<usize>,
    endpoint: Endpoint,
    left: Endpoint,
    right: Endpoint,
    internal: bool,
}
struct Node {
    left: Endpoint,
    right: Endpoint,
    coverage: Vec<Range<usize>>,
    conditional: bool,
    may_vanish: bool,
    barrier: bool,
    internal: bool,
}
fn protection(unit: &Unit, stable: bool) -> Protection {
    let script = (stable && unit.resolved_scripts.len() == 1).then(|| unit.resolved_scripts[0]);
    Protection {
        atomic: !unit.atomics.is_empty(),
        authored: script.filter(|_| unit.qualified && unit.cursive),
        justify: script.filter(|_| unit.justify_qualified && unit.justify_cursive),
    }
}
fn merge_protection(prior: Option<Protection>, next: Protection) -> Protection {
    prior.map_or(next, |prior| Protection {
        atomic: false,
        authored: prior.authored.filter(|s| next.authored == Some(*s)),
        justify: prior.justify.filter(|s| next.justify == Some(*s)),
    })
}

pub(crate) fn plan(
    projection: &SourceProjection,
    analysis: &Analysis,
    styles: &[ResolvedStyleRun],
    boxes: &[InlineBox],
    presentation: Option<&SelectedPresentation>,
    potential: bool,
) -> Result<Arc<PlanRequirements>> {
    // Only a temporary canonical source inventory: no handles or actual graph
    // escape before the completed provider receipts reconcile participation.
    let source = super::build(
        projection,
        analysis,
        styles,
        boxes,
        presentation,
        potential,
        None,
    )?;
    let mut plan = PlanRequirements::default();
    for boundary in &source.boundaries {
        if boundary.authored != 0.0
            || boundary.mean_lost
            || potential && boundary.geometry != crate::SpacingBoundaryGeometry::ProtectedCursive
        {
            for index in [boundary.before, boundary.after] {
                plan.add(&source.units[index].atoms, PlanReason::ExactBoundary);
            }
        }
    }
    if source.units.len() <= 1 {
        return Ok(Arc::new(plan));
    }
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    if !projection.atoms.iter().any(|a| di.contains32(a.code_point)) {
        return Ok(Arc::new(plan));
    }
    let mut cuts = projection.barrier_atoms.clone();
    cuts.extend([0, projection.atoms.len()]);
    let stops: BTreeSet<_> = analysis
        .graphemes
        .iter()
        .flat_map(|g| [g.start, g.end])
        .collect();
    for g in &analysis.graphemes {
        if projection.atoms[g.clone()]
            .iter()
            .any(|a| matches!(a.code_point, 9 | 10 | 13 | 0x2028 | 0x2029))
        {
            cuts.extend([g.start, g.end]);
        }
    }
    if let Some(p) = presentation {
        for domain in &p.domains {
            let mut atoms = p.units[domain.units.clone()]
                .iter()
                .flat_map(|u| u.atoms.clone());
            if let Some(start) = atoms.next() {
                let end = atoms.last().map_or(start + 1, |a| a + 1);
                // Domain isolation cannot invent a cut inside an original EGC.
                cuts.extend([start, end].into_iter().filter(|cut| stops.contains(cut)));
            }
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut components: Vec<_> = cuts
        .windows(2)
        .map(|pair| {
            let atoms = pair[0]..pair[1];
            Component {
                conditional: projection.atoms[atoms.clone()]
                    .iter()
                    .any(|a| di.contains32(a.code_point)),
                all_di: projection.atoms[atoms.clone()]
                    .iter()
                    .all(|a| di.contains32(a.code_point)),
                atoms,
                units: Vec::new(),
                endpoint: Endpoint {
                    operands: Operands::Empty,
                    protection: Protection::default(),
                },
                left: Endpoint {
                    operands: Operands::Empty,
                    protection: Protection::default(),
                },
                right: Endpoint {
                    operands: Operands::Empty,
                    protection: Protection::default(),
                },
                internal: false,
            }
        })
        .collect();
    let representatives: Vec<_> = projection.segmentation.chars().collect();
    let mut facts = Vec::with_capacity(source.units.len());
    for (index, unit) in source.units.iter().enumerate() {
        let Some(atoms) = unit.atoms.first() else {
            facts.push(UnitFacts {
                endpoint: Endpoint {
                    operands: Operands::One(unit.left),
                    protection: protection(unit, true),
                },
                component: None,
                anchor: false,
                invariant: true,
                context_conditional: false,
            });
            continue;
        };
        let component = cuts.partition_point(|&cut| cut <= atoms.start) - 1;
        let candidates = unit_properties::possible_bases(&projection.atoms[atoms.clone()]);
        let stable = candidates.len() == 1
            && unit.base == Some(atoms.start + candidates[0])
            && !di.contains32(projection.atoms[atoms.start + candidates[0]].code_point);
        let mut operands = Operands::Empty;
        for base in candidates {
            operands.add(Operands::One(
                styles[analysis.infos[atoms.start + base].1]
                    .style
                    .letter_spacing,
            ));
        }
        let invariant = unit_properties::di_properties_invariant(&projection.atoms[atoms.clone()]);
        let anchor = stable
            && unit.base.is_some_and(|base| {
                unit_properties::mandatory_ordinary_anchor(
                    &projection.atoms[base],
                    representatives[base],
                )
            });
        let endpoint = Endpoint {
            operands,
            protection: protection(unit, stable && invariant),
        };
        components[component].units.push(index);
        facts.push(UnitFacts {
            endpoint,
            component: Some(component),
            anchor,
            invariant,
            context_conditional: false,
        });
    }
    // A fixed base alone is not a proof about content-script counts: units can
    // coalesce or disappear elsewhere in the same resolver domain. First prove
    // complete source-component contributions invariant, independently of their
    // currently resolved content context.
    for component in &mut components {
        if !component.units.is_empty()
            && component
                .units
                .iter()
                .all(|&u| facts[u].anchor && facts[u].invariant)
        {
            component.conditional = false;
        }
    }
    let contributions_invariant: Vec<_> = facts
        .iter()
        .map(|fact| fact.component.is_none_or(|id| !components[id].conditional))
        .collect();
    for unit in super::conditional_context_consumers(&source.units, &contributions_invariant) {
        // Unknown domain resolution cannot prove same-script justification
        // protection. Do not substitute a guessed script or change final units.
        facts[unit].endpoint.protection.justify = None;
        facts[unit].context_conditional = true;
    }
    for component in &mut components {
        let mut pool = Pool::default();
        let mut common = None;
        for &unit in &component.units {
            let endpoint = facts[unit].endpoint;
            component.internal |= pool.demand(endpoint, false, potential).is_some();
            pool.insert(endpoint, false);
            component.endpoint.operands.add(endpoint.operands);
            common = Some(merge_protection(common, endpoint.protection));
        }
        component.endpoint.protection = common.unwrap_or_default();
        component.left = component.endpoint;
        component.right = component.endpoint;
        // Exposed sides can be narrower than internal uncertainty. Require one
        // contiguous uniform-level visual component; mixed cases keep fallback.
        let uniform = component
            .atoms
            .clone()
            .all(|a| analysis.levels[a] == analysis.levels[component.atoms.start]);
        let contiguous = component
            .units
            .windows(2)
            .all(|pair| pair[1] == pair[0] + 1);
        if uniform && contiguous {
            if let Some(first) = component.units.iter().position(|&u| facts[u].anchor) {
                component.left = exposure(&component.units[..=first], &facts);
            }
            if let Some(last) = component.units.iter().rposition(|&u| facts[u].anchor) {
                component.right = exposure(&component.units[last..], &facts);
            }
        }
    }
    let nodes = nodes(&source, &facts, &components, potential);
    for node in &nodes {
        if node.internal {
            plan.add(&node.coverage, PlanReason::ConditionalEnvelope);
        }
    }
    // One operand summary per finite protection class, in each direction.
    // Partial outcomes are covered without enumerating exclusion subsets.
    for backwards in [false, true] {
        let mut pool = Pool::default();
        for offset in 0..nodes.len() {
            let node = &nodes[if backwards {
                nodes.len() - 1 - offset
            } else {
                offset
            }];
            if node.barrier {
                pool.0.clear();
                continue;
            }
            let incoming = if backwards { node.right } else { node.left };
            if let Some(reason) = pool.demand(incoming, node.conditional, potential) {
                plan.add(&node.coverage, reason);
            }
            if !node.may_vanish {
                pool.0.clear();
            }
            pool.insert(
                if backwards { node.left } else { node.right },
                node.conditional,
            );
        }
    }
    plan.coverage
        .sort_unstable_by_key(|r| (r.atoms.start, r.atoms.end));
    plan.coverage
        .dedup_by(|a, b| a.atoms == b.atoms && a.reason == b.reason);
    Ok(Arc::new(plan))
}

fn nodes(
    source: &Graph,
    facts: &[UnitFacts],
    components: &[Component],
    potential: bool,
) -> Vec<Node> {
    // Existing visual order is authoritative. A component's hull can contain
    // other text when bidi interleaves it; include that interaction uncertainty.
    // Atomics and mandatory breaks stop hull expansion. Full text coverage still
    // remains within each canonical source component, never the whole font run.
    let mut blocks = Vec::new();
    let mut block_of = vec![None; source.units.len()];
    let mut i = 0;
    while i < source.units.len() {
        if source.units[i].barrier || !source.units[i].atomics.is_empty() {
            i += 1;
            continue;
        }
        let start = i;
        let domain = source.units[i].domain;
        while i < source.units.len()
            && !source.units[i].barrier
            && source.units[i].atomics.is_empty()
            && source.units[i].domain == domain
        {
            block_of[i] = Some(blocks.len());
            i += 1;
        }
        blocks.push(start..i);
    }
    let mut envelopes = vec![Vec::<Range<usize>>::new(); blocks.len()];
    for component in components.iter().filter(|c| c.conditional) {
        let mut prior = None;
        for &unit in &component.units {
            let Some(block) = block_of[unit] else {
                continue;
            };
            if prior == Some(block) {
                envelopes[block].last_mut().expect("component hull").end = unit + 1;
            } else {
                envelopes[block].push(unit..unit + 1);
                prior = Some(block);
            }
        }
    }
    let mut groups = vec![None; source.units.len()];
    for rows in &mut envelopes {
        rows.sort_unstable_by_key(|r| r.start);
        let mut merged: Vec<Range<usize>> = Vec::new();
        for range in rows.drain(..) {
            if let Some(prior) = merged.last_mut()
                && range.start < prior.end
            {
                prior.end = prior.end.max(range.end);
            } else {
                merged.push(range);
            }
        }
        for range in merged {
            groups[range.start] = Some(range.end);
        }
    }
    let mut output = Vec::new();
    let mut i = 0;
    while i < source.units.len() {
        let unit = &source.units[i];
        if unit.barrier || !unit.atomics.is_empty() {
            output.push(Node {
                left: Endpoint {
                    operands: Operands::One(unit.left),
                    protection: protection(unit, true),
                },
                right: Endpoint {
                    operands: Operands::One(unit.right),
                    protection: protection(unit, true),
                },
                coverage: Vec::new(),
                conditional: false,
                may_vanish: false,
                barrier: unit.barrier,
                internal: false,
            });
            i += 1;
            continue;
        }
        let conditional = groups[i].is_some();
        let end = groups[i].unwrap_or(i + 1);
        let mut ids = BTreeMap::<usize, Vec<usize>>::new();
        for (offset, fact) in facts[i..end].iter().enumerate() {
            if let Some(id) = fact.component {
                ids.entry(id).or_default().push(i + offset);
            }
        }
        let exposed = if conditional && ids.len() == 1 {
            let id = *ids.keys().next().expect("nonempty text group");
            let component = &components[id];
            (component.units.first() == Some(&i) && component.units.last() == Some(&(end - 1)))
                .then_some((component.left, component.right))
        } else {
            None
        };
        let mut coverage = Vec::new();
        let mut endpoint = Endpoint {
            operands: Operands::Empty,
            protection: Protection::default(),
        };
        let mut common = None;
        let mut mandatory = false;
        let mut internal = false;
        let mut pool = Pool::default();
        for (id, members) in ids {
            let component = &components[id];
            if conditional && component.conditional {
                coverage.push(component.atoms.clone());
                endpoint.operands.add(component.endpoint.operands);
                common = Some(merge_protection(common, component.endpoint.protection));
                mandatory |= !component.all_di
                    && component.units.first().is_some_and(|&u| u >= i)
                    && component.units.last().is_some_and(|&u| u < end);
                internal |= component.internal
                    || pool.demand(component.endpoint, false, potential).is_some();
                pool.insert(component.endpoint, false);
            } else {
                // Stable outside units stay exact; do not expand their component.
                for u in members {
                    {
                        coverage.extend(source.units[u].atoms.iter().cloned());
                        endpoint.operands.add(facts[u].endpoint.operands);
                        common = Some(merge_protection(common, facts[u].endpoint.protection));
                        internal |= pool.demand(facts[u].endpoint, false, potential).is_some();
                        pool.insert(facts[u].endpoint, false);
                        mandatory = true;
                    }
                }
            }
        }
        endpoint.protection = common.unwrap_or_default();
        output.push(Node {
            left: exposed.map_or(endpoint, |sides| sides.0),
            right: exposed.map_or(endpoint, |sides| sides.1),
            coverage,
            conditional: conditional || facts[i..end].iter().any(|f| f.context_conditional),
            may_vanish: !mandatory,
            barrier: false,
            internal: conditional && internal,
        });
        i = end;
    }
    output
}

fn exposure(units: &[usize], facts: &[UnitFacts]) -> Endpoint {
    let mut endpoint = Endpoint {
        operands: Operands::Empty,
        protection: Protection::default(),
    };
    let mut common = None;
    for &unit in units {
        endpoint.operands.add(facts[unit].endpoint.operands);
        common = Some(merge_protection(common, facts[unit].endpoint.protection));
    }
    endpoint.protection = common.unwrap_or_default();
    endpoint
}
