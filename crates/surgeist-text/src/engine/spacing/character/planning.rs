//! Upfront source requirements; actual units remain with canonical graph construction.
use super::{
    Graph, Input, Ledger, PlanCoverage, PlanReason, Subject, Unit, account, storage,
    unit_properties,
};
use crate::Result;
use crate::engine::scope_alignment::census::{AllocationVisitor, AllocationWitness as W};
use crate::processing::contextual::alignment::operations;
use icu_properties::{
    CodePointSetData,
    props::{DefaultIgnorableCodePoint, Script},
};
use std::{ops::Range, sync::Arc};

#[derive(Debug, Default)]
pub(crate) struct PlanRequirements {
    pub(crate) observe: bool,
    pub(crate) coverage: Vec<PlanCoverage>,
    funded: bool,
}
impl PlanRequirements {
    pub(crate) fn funded(&self) -> bool {
        self.funded
    }
    pub(crate) fn masks(
        &self,
        run: Range<usize>,
        ledger: &mut Ledger,
    ) -> Result<Vec<Range<usize>>> {
        let mut native = None;
        let ledger = if self.funded {
            storage::require_ledger(ledger)?;
            ledger
        } else {
            &mut native
        };
        let mut ranges = Vec::new();
        for row in &self.coverage {
            account::work(ledger, 1)?;
            let start = row.atoms.start.max(run.start);
            let end = row.atoms.end.min(run.end);
            if start < end {
                account::push(ledger, &mut ranges, start - run.start..end - run.start)?;
            }
        }
        operations::order_by(&mut ranges, ledger.as_mut(), |a, b| {
            (a.start, a.end).cmp(&(b.start, b.end))
        })?;
        let mut output: Vec<Range<usize>> = Vec::new();
        for range in ranges.drain(..) {
            account::work(ledger, 1)?;
            if let Some(prior) = output.last_mut()
                && range.start <= prior.end
            {
                prior.end = prior.end.max(range.end);
            } else {
                account::push(ledger, &mut output, range)?;
            }
        }
        account::drop_vec(ledger, ranges)?;
        Ok(output)
    }
    fn add(
        &mut self,
        ranges: &[Range<usize>],
        reason: PlanReason,
        ledger: &mut Ledger,
    ) -> Result<()> {
        self.observe = true;
        for atoms in ranges {
            account::work(ledger, 1)?;
            account::push(
                ledger,
                &mut self.coverage,
                PlanCoverage {
                    atoms: atoms.clone(),
                    reason,
                },
            )?;
        }
        Ok(())
    }
    pub(crate) fn visit_allocations(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        if self.funded {
            visit(W::object(self, 0))?;
            visit(W::buffer(&self.coverage, self.coverage.len())?)?;
        }
        Ok(())
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
    fn insert(&mut self, endpoint: Endpoint, conditional: bool, ledger: &mut Ledger) -> Result<()> {
        for row in &mut self.0 {
            account::work(ledger, 1)?;
            if row.endpoint.protection == endpoint.protection {
                row.endpoint.operands.add(endpoint.operands);
                row.conditional |= conditional;
                return Ok(());
            }
        }
        account::push(
            ledger,
            &mut self.0,
            PoolRow {
                endpoint,
                conditional,
            },
        )
    }
    fn demand(
        &self,
        endpoint: Endpoint,
        conditional: bool,
        potential: bool,
        ledger: &mut Ledger,
    ) -> Result<Option<PlanReason>> {
        let mut result = None;
        for row in &self.0 {
            account::work(ledger, 1)?;
            if endpoint.demands(row.endpoint, potential) {
                if conditional || row.conditional {
                    return Ok(Some(PlanReason::ConditionalEnvelope));
                }
                result = Some(PlanReason::ExactBoundary);
            }
        }
        Ok(result)
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
    input: Input<'_>,
    potential: bool,
    ledger: &mut Ledger,
) -> Result<Arc<PlanRequirements>> {
    let mut native = None;
    let funded = input.has_cells();
    let ledger = if funded {
        storage::require_ledger(ledger)?;
        ledger
    } else {
        &mut native
    };
    let source = super::build(input, potential, None, ledger)?;
    let mut plan = PlanRequirements {
        funded,
        ..PlanRequirements::default()
    };
    for boundary in &source.boundaries {
        account::work(ledger, 1)?;
        if boundary.authored != 0.0
            || boundary.mean_lost
            || potential && boundary.geometry != crate::SpacingBoundaryGeometry::ProtectedCursive
        {
            for index in [boundary.before, boundary.after] {
                plan.add(
                    &source.units[index].atoms,
                    PlanReason::ExactBoundary,
                    ledger,
                )?;
            }
        }
    }
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    let mut has_di = false;
    for atom in &input.projection.atoms {
        account::work(ledger, 1)?;
        if di.contains32(atom.code_point) {
            has_di = true;
            break;
        }
    }
    if source.units.len() > 1 && has_di {
        conditional(&source, input, potential, &mut plan, ledger)?;
    }
    source.drop_scratch(ledger)?;
    if funded {
        account::header::<PlanRequirements>(ledger)?;
    }
    Ok(Arc::new(plan))
}

fn conditional(
    source: &Graph,
    input: Input<'_>,
    potential: bool,
    plan: &mut PlanRequirements,
    ledger: &mut Ledger,
) -> Result<()> {
    let projection = input.projection;
    let analysis = input.analysis;
    let styles = input.styles;
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    let mut cuts = account::clone_vec(ledger, &projection.barrier_atoms)?;
    account::push(ledger, &mut cuts, 0)?;
    account::push(ledger, &mut cuts, projection.atoms.len())?;
    input.visit_cells(ledger, |_, _, atoms, ledger| {
        account::push(ledger, &mut cuts, atoms.start)?;
        account::push(ledger, &mut cuts, atoms.end)?;
        Ok(())
    })?;
    let mut stops = Vec::new();
    for g in &analysis.graphemes {
        account::work(ledger, 1)?;
        account::push(ledger, &mut stops, g.start)?;
        account::push(ledger, &mut stops, g.end)?;
        let mut barrier = false;
        for a in &projection.atoms[g.clone()] {
            account::work(ledger, 1)?;
            if matches!(a.code_point, 9 | 10 | 13 | 0x2028 | 0x2029) {
                barrier = true;
                break;
            }
        }
        if barrier {
            account::push(ledger, &mut cuts, g.start)?;
            account::push(ledger, &mut cuts, g.end)?;
        }
    }
    operations::order_by(&mut stops, ledger.as_mut(), Ord::cmp)?;
    account::work(ledger, stops.len())?;
    stops.dedup();
    if let Some(p) = input.presentation {
        for domain in &p.domains {
            account::work(ledger, 1)?;
            let mut start = None;
            let mut end = 0;
            for unit in &p.units[domain.units.clone()] {
                account::work(ledger, 1)?;
                if !unit.atoms.is_empty() {
                    start.get_or_insert(unit.atoms.start);
                    end = unit.atoms.end;
                }
            }
            if let Some(start) = start {
                for cut in [start, end] {
                    if operations::binary_search_by(&stops, ledger.as_mut(), |x| x.cmp(&cut))?
                        .is_ok()
                    {
                        account::push(ledger, &mut cuts, cut)?;
                    }
                }
            }
        }
    }
    account::drop_vec(ledger, stops)?;
    operations::order_by(&mut cuts, ledger.as_mut(), Ord::cmp)?;
    account::work(ledger, cuts.len())?;
    cuts.dedup();
    let mut components = Vec::new();
    for pair in cuts.windows(2) {
        account::work(ledger, 1)?;
        let atoms = pair[0]..pair[1];
        let mut conditional = false;
        let mut all_di = true;
        for a in &projection.atoms[atoms.clone()] {
            account::work(ledger, 1)?;
            let value = di.contains32(a.code_point);
            conditional |= value;
            all_di &= value;
        }
        let endpoint = Endpoint {
            operands: Operands::Empty,
            protection: Protection::default(),
        };
        account::push(
            ledger,
            &mut components,
            Component {
                atoms,
                conditional,
                all_di,
                units: Vec::new(),
                endpoint,
                left: endpoint,
                right: endpoint,
                internal: false,
            },
        )?;
    }
    let representatives = account::collect(ledger, projection.segmentation.chars())?;
    let mut facts = Vec::new();
    for (index, unit) in source.units.iter().enumerate() {
        account::work(ledger, 1)?;
        let Some(atoms) = unit.atoms.first() else {
            account::push(
                ledger,
                &mut facts,
                UnitFacts {
                    endpoint: Endpoint {
                        operands: Operands::One(unit.left),
                        protection: protection(unit, true),
                    },
                    component: None,
                    anchor: false,
                    invariant: true,
                    context_conditional: false,
                },
            )?;
            continue;
        };
        let component =
            operations::partition_point(&cuts, ledger.as_mut(), |cut| *cut <= atoms.start)? - 1;
        let mut candidates = Vec::new();
        unit_properties::visit_possible_bases(
            &projection.atoms[atoms.clone()],
            |visit| match visit {
                unit_properties::BaseVisit::Inspect => account::work(ledger, 1),
                unit_properties::BaseVisit::Candidate(i) => {
                    account::push(ledger, &mut candidates, i)
                }
            },
        )?;
        let stable = candidates.len() == 1
            && unit.base == Some(atoms.start + candidates[0])
            && !di.contains32(projection.atoms[atoms.start + candidates[0]].code_point);
        let mut operands = Operands::Empty;
        for &base in &candidates {
            account::work(ledger, 1)?;
            operands.add(Operands::One(
                styles[analysis.infos[atoms.start + base].1]
                    .style
                    .letter_spacing,
            ));
        }
        // The canonical invariant reads at most one base scan and one DI scan.
        account::work(
            ledger,
            atoms.len().checked_mul(2).ok_or_else(work_overflow)?,
        )?;
        let invariant = unit_properties::di_properties_invariant(&projection.atoms[atoms.clone()]);
        let anchor = stable
            && unit.base.is_some_and(|base| {
                unit_properties::mandatory_ordinary_anchor(
                    &projection.atoms[base],
                    representatives[base],
                )
            });
        account::push(ledger, &mut components[component].units, index)?;
        account::push(
            ledger,
            &mut facts,
            UnitFacts {
                endpoint: Endpoint {
                    operands,
                    protection: protection(unit, stable && invariant),
                },
                component: Some(component),
                anchor,
                invariant,
                context_conditional: false,
            },
        )?;
        account::drop_vec(ledger, candidates)?;
    }
    account::drop_vec(ledger, representatives)?;
    for component in &mut components {
        account::work(ledger, 1)?;
        let mut invariant = !component.units.is_empty();
        for &u in &component.units {
            account::work(ledger, 1)?;
            invariant &= facts[u].anchor && facts[u].invariant;
        }
        if invariant {
            component.conditional = false;
        }
    }
    let mut invariant = Vec::new();
    for fact in &facts {
        account::work(ledger, 1)?;
        account::push(
            ledger,
            &mut invariant,
            fact.component.is_none_or(|i| !components[i].conditional),
        )?;
    }
    let affected = super::conditional_context_consumers(&source.units, &invariant, ledger)?;
    for &u in &affected {
        account::work(ledger, 1)?;
        facts[u].endpoint.protection.justify = None;
        facts[u].context_conditional = true;
    }
    account::drop_vec(ledger, affected)?;
    account::drop_vec(ledger, invariant)?;
    for component in &mut components {
        account::work(ledger, 1)?;
        let mut pool = Pool::default();
        let mut common = None;
        for &u in &component.units {
            account::work(ledger, 1)?;
            let endpoint = facts[u].endpoint;
            component.internal |= pool.demand(endpoint, false, potential, ledger)?.is_some();
            pool.insert(endpoint, false, ledger)?;
            component.endpoint.operands.add(endpoint.operands);
            common = Some(merge_protection(common, endpoint.protection));
        }
        account::drop_vec(ledger, pool.0)?;
        component.endpoint.protection = common.unwrap_or_default();
        component.left = component.endpoint;
        component.right = component.endpoint;
        let mut uniform = true;
        for a in component.atoms.clone() {
            account::work(ledger, 1)?;
            uniform &= analysis.levels[a] == analysis.levels[component.atoms.start];
        }
        let mut contiguous = true;
        for pair in component.units.windows(2) {
            account::work(ledger, 1)?;
            contiguous &= pair[1] == pair[0] + 1;
        }
        if uniform && contiguous {
            let mut first = None;
            let mut last = None;
            for (i, &u) in component.units.iter().enumerate() {
                account::work(ledger, 1)?;
                if facts[u].anchor {
                    first.get_or_insert(i);
                    last = Some(i);
                }
            }
            if let Some(i) = first {
                component.left = exposure(&component.units[..=i], &facts, ledger)?;
            }
            if let Some(i) = last {
                component.right = exposure(&component.units[i..], &facts, ledger)?;
            }
        }
    }
    let mut nodes = nodes(source, &facts, &components, potential, ledger)?;
    for node in &nodes {
        account::work(ledger, 1)?;
        if node.internal {
            plan.add(&node.coverage, PlanReason::ConditionalEnvelope, ledger)?;
        }
    }
    for backwards in [false, true] {
        let mut pool = Pool::default();
        for offset in 0..nodes.len() {
            account::work(ledger, 1)?;
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
            if let Some(reason) = pool.demand(incoming, node.conditional, potential, ledger)? {
                plan.add(&node.coverage, reason, ledger)?;
            }
            if !node.may_vanish {
                pool.0.clear();
            }
            pool.insert(
                if backwards { node.left } else { node.right },
                node.conditional,
                ledger,
            )?;
        }
        account::drop_vec(ledger, pool.0)?;
    }
    operations::order_by(&mut plan.coverage, ledger.as_mut(), |a, b| {
        (a.atoms.start, a.atoms.end).cmp(&(b.atoms.start, b.atoms.end))
    })?;
    account::work(ledger, plan.coverage.len())?;
    plan.coverage
        .dedup_by(|a, b| a.atoms == b.atoms && a.reason == b.reason);
    for node in &mut nodes {
        account::drop_vec(ledger, std::mem::take(&mut node.coverage))?;
    }
    account::drop_vec(ledger, nodes)?;
    for component in &mut components {
        account::drop_vec(ledger, std::mem::take(&mut component.units))?;
    }
    account::drop_vec(ledger, components)?;
    account::drop_vec(ledger, facts)?;
    account::drop_vec(ledger, cuts)
}

fn work_overflow() -> crate::Error {
    crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
        kind: crate::ScopeAlignmentLimitKind::StructuralWork,
    })
}

fn object(unit: &Unit) -> bool {
    unit.subject != Subject::Text
}

fn nodes(
    source: &Graph,
    facts: &[UnitFacts],
    components: &[Component],
    potential: bool,
    ledger: &mut Ledger,
) -> Result<Vec<Node>> {
    let mut blocks = Vec::new();
    let mut block_of = account::filled(ledger, source.units.len(), None)?;
    let mut i = 0;
    while i < source.units.len() {
        account::work(ledger, 1)?;
        if source.units[i].barrier || object(&source.units[i]) {
            i += 1;
            continue;
        }
        let start = i;
        let domain = source.units[i].domain;
        while i < source.units.len() {
            account::work(ledger, 1)?;
            if source.units[i].barrier
                || object(&source.units[i])
                || source.units[i].domain != domain
            {
                break;
            }
            block_of[i] = Some(blocks.len());
            i += 1;
        }
        account::push(ledger, &mut blocks, start..i)?;
    }
    let mut envelopes = Vec::new();
    for _ in &blocks {
        account::push(ledger, &mut envelopes, Vec::<Range<usize>>::new())?;
    }
    for component in components {
        account::work(ledger, 1)?;
        if !component.conditional {
            continue;
        }
        let mut prior = None;
        for &u in &component.units {
            account::work(ledger, 1)?;
            let Some(block) = block_of[u] else {
                continue;
            };
            if prior == Some(block) {
                envelopes[block].last_mut().expect("component hull").end = u + 1;
            } else {
                account::push(ledger, &mut envelopes[block], u..u + 1)?;
                prior = Some(block);
            }
        }
    }
    let mut groups = account::filled(ledger, source.units.len(), None)?;
    for rows in &mut envelopes {
        account::work(ledger, 1)?;
        operations::order_by(rows, ledger.as_mut(), |a, b| a.start.cmp(&b.start))?;
        let mut merged: Vec<Range<usize>> = Vec::new();
        for range in rows.drain(..) {
            account::work(ledger, 1)?;
            if let Some(prior) = merged.last_mut()
                && range.start < prior.end
            {
                prior.end = prior.end.max(range.end);
            } else {
                account::push(ledger, &mut merged, range)?;
            }
        }
        for range in &merged {
            account::work(ledger, 1)?;
            groups[range.start] = Some(range.end);
        }
        account::drop_vec(ledger, merged)?;
    }
    let mut output = Vec::new();
    let mut i = 0;
    while i < source.units.len() {
        account::work(ledger, 1)?;
        let unit = &source.units[i];
        if unit.barrier || object(unit) {
            account::push(
                ledger,
                &mut output,
                Node {
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
                },
            )?;
            i += 1;
            continue;
        }
        let conditional = groups[i].is_some();
        let end = groups[i].unwrap_or(i + 1);
        let mut ids: Vec<(usize, Vec<usize>)> = Vec::new();
        for (offset, fact) in facts[i..end].iter().enumerate() {
            account::work(ledger, 1)?;
            if let Some(id) = fact.component {
                let at = match operations::binary_search_by(&ids, ledger.as_mut(), |row| {
                    row.0.cmp(&id)
                })? {
                    Ok(at) => at,
                    Err(at) => {
                        account::reserve_vec(ledger, &mut ids, 1)?;
                        account::work(ledger, ids.len() - at + 1)?;
                        if let Some(b) = ledger {
                            b.add_records(1)?;
                        }
                        ids.insert(at, (id, Vec::new()));
                        at
                    }
                };
                account::push(ledger, &mut ids[at].1, i + offset)?;
            }
        }
        let exposed = if conditional && ids.len() == 1 {
            let component = &components[ids[0].0];
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
        for (id, members) in &ids {
            account::work(ledger, 1)?;
            let component = &components[*id];
            if conditional && component.conditional {
                account::push(ledger, &mut coverage, component.atoms.clone())?;
                endpoint.operands.add(component.endpoint.operands);
                common = Some(merge_protection(common, component.endpoint.protection));
                mandatory |= !component.all_di
                    && component.units.first().is_some_and(|&u| u >= i)
                    && component.units.last().is_some_and(|&u| u < end);
                internal |= component.internal
                    || pool
                        .demand(component.endpoint, false, potential, ledger)?
                        .is_some();
                pool.insert(component.endpoint, false, ledger)?;
            } else {
                for &u in members {
                    account::work(ledger, 1)?;
                    for atoms in &source.units[u].atoms {
                        account::push(ledger, &mut coverage, atoms.clone())?;
                    }
                    endpoint.operands.add(facts[u].endpoint.operands);
                    common = Some(merge_protection(common, facts[u].endpoint.protection));
                    internal |= pool
                        .demand(facts[u].endpoint, false, potential, ledger)?
                        .is_some();
                    pool.insert(facts[u].endpoint, false, ledger)?;
                    mandatory = true;
                }
            }
        }
        endpoint.protection = common.unwrap_or_default();
        let mut context = false;
        for fact in &facts[i..end] {
            account::work(ledger, 1)?;
            context |= fact.context_conditional;
        }
        account::push(
            ledger,
            &mut output,
            Node {
                left: exposed.map_or(endpoint, |x| x.0),
                right: exposed.map_or(endpoint, |x| x.1),
                coverage,
                conditional: conditional || context,
                may_vanish: !mandatory,
                barrier: false,
                internal: conditional && internal,
            },
        )?;
        account::drop_vec(ledger, pool.0)?;
        for row in &mut ids {
            account::drop_vec(ledger, std::mem::take(&mut row.1))?;
        }
        account::drop_vec(ledger, ids)?;
        i = end;
    }
    for rows in &mut envelopes {
        account::drop_vec(ledger, std::mem::take(rows))?;
    }
    account::drop_vec(ledger, envelopes)?;
    account::drop_vec(ledger, groups)?;
    account::drop_vec(ledger, block_of)?;
    account::drop_vec(ledger, blocks)?;
    Ok(output)
}

fn exposure(units: &[usize], facts: &[UnitFacts], ledger: &mut Ledger) -> Result<Endpoint> {
    let mut endpoint = Endpoint {
        operands: Operands::Empty,
        protection: Protection::default(),
    };
    let mut common = None;
    for &u in units {
        account::work(ledger, 1)?;
        endpoint.operands.add(facts[u].endpoint.operands);
        common = Some(merge_protection(common, facts[u].endpoint.protection));
    }
    endpoint.protection = common.unwrap_or_default();
    Ok(endpoint)
}
